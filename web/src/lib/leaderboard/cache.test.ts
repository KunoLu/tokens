/// <reference types="bun-types" />
import { describe, expect, mock, test } from "bun:test";
import * as schema from "@/lib/db/schema";

// next/cache must be mocked before the modules under test are imported: the
// real unstable_cache throws outside the Next server, and the real
// revalidateTag/revalidatePath do nothing observable here. The mock records
// every call and lets unstable_cache call through, so the cached functions
// still execute (against the mocked db below) and register their keys/tags.
const tagCalls: Array<[string, unknown]> = [];
const pathCalls: string[] = [];
const cacheEntries: Array<{ key: string; tags: string[] }> = [];

mock.module("next/cache", () => ({
  unstable_cache: (
    fn: (...args: unknown[]) => unknown,
    key: string[],
    options: { tags?: string[] },
  ) => {
    cacheEntries.push({ key: key.join("|"), tags: options?.tags ?? [] });
    return fn;
  },
  revalidateTag: (tag: string, profile: unknown) => {
    tagCalls.push([tag, profile]);
  },
  revalidatePath: (path: string) => {
    pathCalls.push(path);
  },
}));

// Any query-builder method returns the same thenable proxy; awaiting it
// resolves the queued rows (empty by default). The leaderboard readers only
// need select(); update() is here so the privacy endpoints can run end to end.
function queryChain(result: unknown[] = []) {
  const builder: unknown = new Proxy(
    {},
    {
      get: (_target, prop) => {
        if (prop === "then") {
          return (
            resolve: (value: unknown) => unknown,
            reject: (error: unknown) => unknown,
          ) => Promise.resolve(result).then(resolve, reject);
        }
        return () => builder;
      },
    },
  );
  return builder;
}

mock.module("@/lib/db", () => ({
  ...schema,
  db: {
    select: () => queryChain(),
    update: () => queryChain(),
    execute: async () => [],
    transaction: async (fn: (tx: unknown) => Promise<unknown>) =>
      fn({ update: () => queryChain() }),
  },
}));

mock.module("@/lib/auth/requestSession", () => ({
  getSessionFromRequest: async () => ({ id: "u1", username: "alice" }),
}));

mock.module("@/lib/auth/personalTokens", () => ({
  authenticatePersonalToken: async () => ({
    status: "valid",
    userId: "u1",
    username: "alice",
  }),
}));

mock.module("@/lib/auth/readTokens", () => ({
  READ_TOKEN_PREFIX: "tkr_",
  DEFAULT_READ_TOKEN_NAME: "Read token",
  issueReadTokenInTransaction: async () => ({
    id: "rt1",
    name: "default",
    token: "tt_read_secret",
    createdAt: new Date("2026-11-14"),
  }),
  normalizeReadTokenName: (name: unknown) => name ?? "default",
  verifyReadTokenForUser: async () => false,
}));

// Dynamic imports, deliberately: a static import would be hoisted above the
// mock.module calls and bind the real next/cache (unstable_cache throws
// outside the Next server) and the real @/lib/db (throws without
// DATABASE_URL). The mocks must own the registry before these modules load.
const { LEADERBOARD_CACHE_TAG, revalidateLeaderboard } = await import("./cache");
const { getLeaderboardData, getUserRank } = await import("./getLeaderboard");
const { revalidateUserVisibility } = await import("@/lib/privacy");
const { PATCH } = await import("@/app/api/settings/privacy/route");
const { POST: postMePrivate } = await import("@/app/api/me/private/route");

function resetCalls() {
  tagCalls.length = 0;
  pathCalls.length = 0;
}

function expectBoardPurged() {
  expect(tagCalls).toContainEqual([LEADERBOARD_CACHE_TAG, { expire: 0 }]);
  for (const path of ["/leaderboard", "/zh-CN/leaderboard", "/ja/leaderboard"]) {
    expect(pathCalls).toContain(path);
  }
}

describe("going private purges the board", () => {
  test("PATCH /api/settings/privacy expires the unified tag and every language's page", async () => {
    resetCalls();
    const response = await PATCH(
      new Request("https://tokens.test/api/settings/privacy", {
        method: "PATCH",
        body: JSON.stringify({ private: true }),
      }),
    );
    expect(response.status).toBe(200);
    expectBoardPurged();
  });

  test("POST /api/me/private (tokens login --private) expires the unified tag and every language's page", async () => {
    resetCalls();
    const response = await postMePrivate(
      new Request("https://tokens.test/api/me/private", {
        method: "POST",
        headers: { Authorization: "Bearer tt_submit_secret" },
        body: JSON.stringify({}),
      }),
    );
    expect(response.status).toBe(200);
    expectBoardPurged();
  });

  test("revalidateUserVisibility reaches all three locale board pages", () => {
    resetCalls();
    revalidateUserVisibility("Alice");
    expectBoardPurged();
  });
});

describe("leaderboard cache entries", () => {
  test("every parameter and language variant hangs on the unified tag", async () => {
    cacheEntries.length = 0;

    // Period rankings: period, viewer-local date and sort variants.
    await getLeaderboardData("today");
    await getLeaderboardData("today", 1, 5000, "tokens", "", "2026-11-14");
    await getLeaderboardData("week", 2, 100, "cost");
    await getLeaderboardData("custom", 1, 50, "tokens", "", "2026-10-01", "2026-10-31");
    // All-time and directive listings: page, limit, sort and search variants.
    await getLeaderboardData("all");
    await getLeaderboardData("all", 3, 100, "cost");
    await getLeaderboardData("all", 1, 50, "tokens", "model:kimi");
    await getLeaderboardData("month", 1, 50, "tokens", "client:codex");
    // All-time per-user rank, both sorts.
    await getUserRank("alice", "all");
    await getUserRank("alice", "all", "cost");

    // Distinct parameter sets produce distinct cache entries…
    expect(new Set(cacheEntries.map((entry) => entry.key)).size).toBe(10);
    // …and every one of them hangs on the unified tag, so a single
    // revalidateTag drops all variants in all three languages at once — the
    // entries are locale-blind because translations happen at render.
    for (const entry of cacheEntries) {
      expect(entry.tags).toContain(LEADERBOARD_CACHE_TAG);
    }
  });

  test("revalidateLeaderboard expires the tag before touching the pages", () => {
    resetCalls();
    revalidateLeaderboard();
    expect(tagCalls).toEqual([[LEADERBOARD_CACHE_TAG, { expire: 0 }]]);
    expect(pathCalls).toEqual(["/leaderboard", "/zh-CN/leaderboard", "/ja/leaderboard"]);
  });
});
