/// <reference types="bun-types" />
import { describe, expect, mock, test } from "bun:test";
import * as schema from "./db/schema";

// Every db.select() chain resolves to the next queued result, in the order
// getPublicProfileResponse issues them: user, stats, latest submission,
// daily rows, archived count. db.execute() serves the rank query.
const selectResults: unknown[][] = [];
function chain(result: unknown[]) {
  const builder: Record<string, unknown> = {};
  for (const method of ["from", "where", "limit", "orderBy", "innerJoin"]) {
    builder[method] = () => builder;
  }
  builder.then = (resolve: (value: unknown) => unknown, reject: (e: unknown) => unknown) =>
    Promise.resolve(result).then(resolve, reject);
  return builder;
}
mock.module("@/lib/db", () => ({
  ...schema,
  db: {
    select: () => chain(selectResults.shift() ?? []),
    execute: async () => [{ rank: 1 }],
  },
}));

const {
  getProfilePeriodDateRange,
  getPublicProfileResponse,
  getRollingProfileDateRange,
} = await import("./publicProfileData");

const model = (tokens: number, speedUsage?: unknown[]) => ({
  tokens,
  cost: tokens / 100,
  input: tokens,
  output: 0,
  cacheRead: 0,
  cacheWrite: 0,
  reasoning: 0,
  messages: 1,
  ...(speedUsage ? { speedUsage } : {}),
});
const claudeRow = (models: Record<string, ReturnType<typeof model>>) => {
  const tokens = Object.values(models).reduce((sum, m) => sum + m.tokens, 0);
  return {
    date: new Date().toISOString().slice(0, 10),
    timestampMs: null,
    tokens,
    cost: String(tokens / 100),
    inputTokens: tokens,
    outputTokens: 0,
    sourceBreakdown: {
      claude: { ...model(tokens), messages: Object.keys(models).length, models },
    },
  };
};

test("public profile preserves and merges nested model speedUsage across devices", async () => {
  selectResults.push(
    [{ id: "u1", username: "alice", displayName: null, avatarUrl: null, createdAt: new Date("2026-01-01"), bannedAt: null, banReason: null }],
    [{ totalTokens: 600, totalCost: 6 }],
    [{ sourcesUsed: ["claude"], modelsUsed: ["opus", "sonnet"], updatedAt: new Date("2026-10-02"), cliVersion: "1.0.0", schemaVersion: 2, mcpServers: null }],
    [
      // Device A
      claudeRow({
        opus: model(200, [{ speed: "fast", source: "reported", tokens: 150, messages: 1 }]),
        sonnet: model(100),
      }),
      // Device B, same day/client/model
      claudeRow({
        opus: model(300, [
          { speed: "fast", source: "reported", tokens: 50, messages: 1 },
          { speed: "standard", source: "selected", tokens: 25, messages: 0 },
        ]),
      }),
    ],
    [{ days: 0 }],
  );

  const response = await getPublicProfileResponse(
    new Request("http://tokens.internal/api/users/alice"),
    { params: Promise.resolve({ username: "alice" }) },
  );
  expect(response.status).toBe(200);
  const body = await response.json();

  const [day] = body.contributions;
  expect(day.clients).toHaveLength(1);
  const { models } = day.clients[0];
  expect(Object.keys(models).sort()).toEqual(["opus", "sonnet"]);
  expect(models.opus.tokens).toBe(500);
  expect(models.opus.speedUsage).toEqual([
    { speed: "standard", source: "selected", tokens: 25, messages: 0 },
    { speed: "fast", source: "reported", tokens: 200, messages: 2 },
  ]);
  expect(models.sonnet.speedUsage).toBeUndefined();

  // Model identities and totals are unaffected by speed metadata.
  expect(body.modelUsage.map((m: { model: string; tokens: number }) => [m.model, m.tokens])).toEqual([
    ["opus", 500],
    ["sonnet", 100],
  ]);
  expect(body.modelUsage[0].speedUsage).toEqual([
    { speed: "standard", source: "selected", tokens: 25, messages: 0 },
    { speed: "fast", source: "reported", tokens: 200, messages: 2 },
  ]);
  expect(body.modelUsage[1].speedUsage).toBeUndefined();
  expect(day.totals.tokens).toBe(600);
});

test("model speed totals merge clients and follow the same period boundaries as tokens and cost", async () => {
  for (const [period, tokens, reportedFast, ultrafast] of [
    ["week", 600, 150, 0],
    ["month", 1250, 350, 350],
    ["all", 1300, 390, 350],
  ] as const) {
    const row = (client: string, daysAgo: number, tokens: number, entries: unknown[]) => {
      const day = claudeRow({ opus: model(tokens, entries) });
      const date = new Date();
      date.setUTCDate(date.getUTCDate() - daysAgo);
      return {
        ...day,
        date: date.toISOString().slice(0, 10),
        sourceBreakdown: { [client]: day.sourceBreakdown.claude },
      };
    };
    selectResults.push(
      [{ id: "u1", username: "alice", bannedAt: null }],
      [{ totalTokens: 1300, totalCost: 13 }],
      [],
      [
        row("claude", 0, 200, [{ speed: "fast", source: "reported", tokens: 150, messages: 1 }]),
        row("cc-mirror/work", 0, 300, [
          { speed: "fast", source: "selected", tokens: 120, messages: 1 },
          { speed: "standard", source: "reported", tokens: 80, messages: 0 },
        ]),
        row("claude", 6, 100, [{ speed: "standard", source: "selected", tokens: 60, messages: 1 }]),
        row("claude", 7, 400, [{ speed: "ultrafast", source: "selected", tokens: 350, messages: 1 }]),
        row("claude", 29, 250, [{ speed: "fast", source: "reported", tokens: 200, messages: 0 }]),
        row("claude", 30, 50, [{ speed: "fast", source: "reported", tokens: 40, messages: 0 }]),
      ],
      [{ days: 0 }],
    );
    const response = await getPublicProfileResponse(
      new Request(`http://tokens.internal/api/users/alice?period=${period}`),
      { params: Promise.resolve({ username: "alice" }) },
    );
    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.modelUsage).toEqual([{
      model: "opus",
      tokens,
      cost: tokens / 100,
      percentage: 100,
      speedUsage: [
        { speed: "standard", source: "reported", tokens: 80, messages: 0 },
        { speed: "standard", source: "selected", tokens: 60, messages: 1 },
        { speed: "fast", source: "reported", tokens: reportedFast, messages: 1 },
        { speed: "fast", source: "selected", tokens: 120, messages: 1 },
        ...(ultrafast ? [{ speed: "ultrafast", source: "selected", tokens: ultrafast, messages: 1 }] : []),
      ],
    }]);
  }
});

// issue #21: "today" on the profile follows the viewer's time zone, not the
// server's UTC date. Pure range functions take fixed instants; the route
// takes `tz` and answers ranges in that zone.
describe("viewer time zone day assignment (issue #21)", () => {
  // 2026-11-13 16:30 UTC = 2026-11-14 00:30 in UTC+8 — the issue's repro.
  const shanghaiMorningUtc = new Date("2026-11-13T16:30:00.000Z");
  // 2026-11-14 04:30 UTC = 2026-11-13 23:30 in UTC-5.
  const newYorkLateEveningUtc = new Date("2026-11-14T04:30:00.000Z");

  // Independent of lib/timezone's formatToParts assembly: en-CA renders
  // YYYY-MM-DD directly, so agreement is a real cross-check.
  const calendarDateIn = (now: Date, timeZone: string) =>
    new Intl.DateTimeFormat("en-CA", { timeZone, dateStyle: "short" }).format(now);

  test("rolling chart range ends on the viewer-zone today, defaulting to UTC", () => {
    expect(getRollingProfileDateRange(shanghaiMorningUtc, "Asia/Shanghai")).toEqual({
      start: "2025-11-14",
      end: "2026-11-14",
    });
    expect(getRollingProfileDateRange(shanghaiMorningUtc, "UTC")).toEqual({
      start: "2025-11-13",
      end: "2026-11-13",
    });
    expect(getRollingProfileDateRange(shanghaiMorningUtc)).toEqual({
      start: "2025-11-13",
      end: "2026-11-13",
    });
    expect(getRollingProfileDateRange(newYorkLateEveningUtc, "America/New_York")).toEqual({
      start: "2025-11-13",
      end: "2026-11-13",
    });
  });

  test("rolling range clamps February 29 to February 28 one year back", () => {
    expect(getRollingProfileDateRange(new Date("2028-02-29T12:00:00.000Z"), "UTC")).toEqual({
      start: "2027-02-28",
      end: "2028-02-29",
    });
  });

  test("week and month ranges follow the viewer zone", () => {
    expect(getProfilePeriodDateRange("week", shanghaiMorningUtc, "Asia/Shanghai")).toEqual({
      start: "2026-11-08",
      end: "2026-11-14",
    });
    expect(getProfilePeriodDateRange("month", shanghaiMorningUtc, "Asia/Shanghai")).toEqual({
      start: "2026-10-16",
      end: "2026-11-14",
    });
    expect(getProfilePeriodDateRange("week", shanghaiMorningUtc, "UTC")).toEqual({
      start: "2026-11-07",
      end: "2026-11-13",
    });
    expect(getProfilePeriodDateRange("all", shanghaiMorningUtc, "Asia/Shanghai")).toBeNull();
  });

  const emptyProfileRows = () => {
    selectResults.push(
      [{ id: "u1", username: "alice", displayName: null, avatarUrl: null, createdAt: new Date("2026-01-01"), bannedAt: null, banReason: null, isPrivate: false }],
      [{ totalTokens: 100, totalCost: 1 }],
      [],
      [],
      [{ days: 0 }],
    );
  };

  test("tz parameter drives the chart range end; no tz keeps the UTC date", async () => {
    const now = new Date();
    const utcToday = calendarDateIn(now, "UTC");
    // One of these always disagrees with the UTC date: UTC+14 crosses the
    // date line from 10:00 UTC on, UTC-12 lags a day until 12:00 UTC.
    const probe = utcToday === calendarDateIn(now, "Pacific/Kiritimati")
      ? "Etc/GMT+12"
      : "Pacific/Kiritimati";
    const probeToday = calendarDateIn(now, probe);

    emptyProfileRows();
    const zoned = await getPublicProfileResponse(
      new Request(`http://tokens.internal/api/users/alice?tz=${encodeURIComponent(probe)}`),
      { params: Promise.resolve({ username: "alice" }) },
    );
    expect(zoned.status).toBe(200);
    const zonedBody = await zoned.json();
    expect(zonedBody.chartRange.end).toBe(probeToday);
    expect(probeToday).not.toBe(utcToday);

    emptyProfileRows();
    const plain = await getPublicProfileResponse(
      new Request("http://tokens.internal/api/users/alice"),
      { params: Promise.resolve({ username: "alice" }) },
    );
    expect(plain.status).toBe(200);
    const plainBody = await plain.json();
    expect(plainBody.chartRange.end).toBe(utcToday);
  });

  test("an invalid tz parameter is rejected", async () => {
    emptyProfileRows();
    const response = await getPublicProfileResponse(
      new Request("http://tokens.internal/api/users/alice?tz=Not/AZone"),
      { params: Promise.resolve({ username: "alice" }) },
    );
    expect(response.status).toBe(400);
  });

  test("the profile payload never exposes the owner's time zone setting", async () => {
    emptyProfileRows();
    const response = await getPublicProfileResponse(
      new Request("http://tokens.internal/api/users/alice"),
      { params: Promise.resolve({ username: "alice" }) },
    );
    expect(response.status).toBe(200);
    const body = await response.json();
    expect("timezone" in body.user).toBe(false);
    expect("timezone" in body).toBe(false);
  });
});
