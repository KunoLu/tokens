/// <reference types="bun-types" />
import { afterAll, afterEach, describe, expect, mock, test } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";
import type { ReactElement } from "react";
import type { DailyContribution } from "@/lib/types";
import type { ProfileData } from "./ProfilePageClient";

// DOM globals must exist before react-dom or the component module load, so
// everything below that touches them is a deliberate dynamic import (module
// loading boundary — static imports would hoist above the registration).
// All happy-dom test files share one bun process; register exactly once.
if (!GlobalRegistrator.isRegistered) {
  GlobalRegistrator.register();
}
(globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = true;

// Viewer identity and zone arrive over fetch; the router, locale and settings
// hooks are irrelevant to day assignment and are stubbed at the module level.
mock.module("nextjs-toploader/app", () => ({
  useRouter: () => ({ push: () => {} }),
}));
mock.module("@/components/layout/LocaleProvider", () => ({
  useLocale: () => "en",
  useT: () => (key: string) => key,
}));
mock.module("@/lib/useSettings", () => ({
  useSettings: () => ({ paletteName: "blue", setPalette: () => {} }),
}));

// The graph module is imported for real: the day-assignment helpers the
// component consumes (getDefaultContributionDate, getContributionDayForDate,
// range resolution) must be the production implementations, not rewrites.
const graphModule = await import("@/components/profile/ProfileContributionGraph");
mock.module("@/components/profile", () => ({
  createContributionRangeOptions: graphModule.createContributionRangeOptions,
  getContributionDayForDate: graphModule.getContributionDayForDate,
  getDefaultContributionDate: graphModule.getDefaultContributionDate,
  reconcileContributionSelectionRange: graphModule.reconcileContributionSelectionRange,
  resolveContributionRange: graphModule.resolveContributionRange,
  resolveContributionSelectedDate: graphModule.resolveContributionSelectedDate,
  ProfileContributionGraph: () => null,
  ProfileContributionBreakdown: () => null,
  ProfileDevices: () => null,
  ProfileHabits: () => null,
  ProfileModels: () => null,
  ProfileUsageChart: () => null,
  TokenBreakdown: () => null,
  ProfileMembership: () => null,
}));

// Only the markers matter: which calendar date the "today" card renders.
mock.module("@/components/profile/ProfileView", () => ({
  ProfileView: (props: { today?: ReactElement; activity?: ReactElement }) => (
    <div data-testid="profile-view">
      {props.today}
      {props.activity}
    </div>
  ),
}));
mock.module("@/components/profile/ProfileToday", () => ({
  ProfileToday: ({
    day,
    yesterday,
  }: {
    day?: { date?: string } | null;
    yesterday?: { date?: string } | null;
  }) => (
    <div data-testid="profile-today">
      {JSON.stringify({ today: day?.date ?? null, yesterday: yesterday?.date ?? null })}
    </div>
  ),
}));
mock.module("@/components/profile/ProfileEmbedDialog", () => ({
  ProfileEmbedDialog: () => null,
}));

const { default: ProfilePageClient } = await import("./ProfilePageClient");
const { createRoot } = await import("react-dom/client");
const { act } = await import("react");

const REAL_DATE = Date;
const REAL_RESOLVED_OPTIONS = Intl.DateTimeFormat.prototype.resolvedOptions;
const REAL_FETCH = globalThis.fetch;

/** The issue's repro window: 2026-11-13 16:30 UTC = 2026-11-14 in UTC+8. */
const FIXED_NOW_ISO = "2026-11-13T16:30:00.000Z";

function freezeClock(detectedZone: string) {
  const fixed = new REAL_DATE(FIXED_NOW_ISO);
  class FrozenDate extends REAL_DATE {
    constructor(...args: unknown[]) {
      super(...((args.length === 0 ? [fixed.getTime()] : args) as [number]));
    }
    static now() {
      return fixed.getTime();
    }
  }
  globalThis.Date = FrozenDate as DateConstructor;
  Intl.DateTimeFormat.prototype.resolvedOptions = function (
    this: Intl.DateTimeFormat,
    ...args: Parameters<Intl.DateTimeFormat["resolvedOptions"]>
  ) {
    return { ...REAL_RESOLVED_OPTIONS.apply(this, args), timeZone: detectedZone };
  };
}

function unfreezeClock() {
  globalThis.Date = REAL_DATE;
  Intl.DateTimeFormat.prototype.resolvedOptions = REAL_RESOLVED_OPTIONS;
}

function contribution(date: string, tokens: number): DailyContribution {
  return {
    date,
    totals: { tokens, cost: tokens / 1000, messages: 1 },
    intensity: 1,
    tokenBreakdown: { input: tokens, output: 0, cacheRead: 0, cacheWrite: 0, reasoning: 0 },
    clients: [],
  };
}

/** Server payload bucketed by UTC: chart range ends on the UTC date. */
function profilePayload(chartEnd: string): ProfileData {
  return {
    user: {
      id: "u-bob",
      username: "bob",
      displayName: "Bob",
      avatarUrl: null,
      createdAt: "2026-01-01T00:00:00.000Z",
      rank: null,
    },
    stats: {
      totalTokens: 300,
      totalCost: 0.3,
      inputTokens: 300,
      outputTokens: 0,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
      submissionCount: 2,
      activeDays: 2,
      sessionCount: 2,
    },
    dateRange: { start: "2026-11-13", end: "2026-11-14" },
    chartRange: { start: "2025-11-14", end: chartEnd },
    updatedAt: null,
    clients: [],
    models: [],
    contributions: [contribution("2026-11-13", 100), contribution("2026-11-14", 200)],
    period: "all",
  };
}

let sessionPayload: unknown = { user: null };
const fetchCalls: string[] = [];
globalThis.fetch = (async (input: RequestInfo | URL) => {
  const url = String(input);
  fetchCalls.push(url);
  if (url === "/api/auth/session") {
    return new Response(JSON.stringify(sessionPayload), { status: 200 });
  }
  if (url.startsWith("/api/users/")) {
    // The server honoring `tz` answers with the viewer-zone chart end.
    return new Response(JSON.stringify(profilePayload("2026-11-14")), { status: 200 });
  }
  return new Response("not found", { status: 404 });
}) as typeof fetch;

const roots: Array<{ root: { unmount: () => void }; container: HTMLElement }> = [];

async function flushEffects() {
  for (let round = 0; round < 5; round += 1) {
    await act(async () => {
      const { promise, resolve } = Promise.withResolvers<void>();
      setTimeout(resolve, 0);
      await promise;
    });
  }
}

async function renderProfile() {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <ProfilePageClient initialData={profilePayload("2026-11-13")} username="bob" />,
    );
  });
  await flushEffects();
  roots.push({ root, container });
  return container;
}

function todayMarker(container: HTMLElement) {
  const element = container.querySelector('[data-testid="profile-today"]');
  return JSON.parse(element?.textContent ?? "{}") as { today: string | null };
}

afterEach(() => {
  sessionPayload = { user: null };
  fetchCalls.length = 0;
  unfreezeClock();
  while (roots.length > 0) {
    const { root, container } = roots.pop()!;
    act(() => {
      root.unmount();
    });
    container.remove();
  }
});

afterAll(() => {
  globalThis.fetch = REAL_FETCH;
  (globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = false;
  // happy-dom's globals (Request/Response/window/…) must not leak into the
  // other test files that share this bun process; under --randomize any
  // file can run after this one.
  GlobalRegistrator.unregister();
});

describe("ProfilePageClient viewer-zone day assignment (issue #21, rule 3)", () => {
  test("viewing someone else's page: the account setting wins over browser detection", async () => {
    freezeClock("America/New_York");
    sessionPayload = { user: { username: "alice", timezone: "Asia/Shanghai" } };
    const container = await renderProfile();

    // 2026-11-13T16:30Z is already 2026-11-14 for the UTC+8 viewer, while the
    // browser reports a zone where it is still 2026-11-13.
    expect(todayMarker(container).today).toBe("2026-11-14");
    // The UTC-bucketed payload ends on 2026-11-13, so exactly one refetch
    // carries the viewer's zone; the once-per-key guard stops repeats.
    const tzFetches = fetchCalls.filter((url) => url.includes("tz="));
    expect(tzFetches).toEqual(["/api/users/bob?period=all&tz=Asia%2FShanghai"]);
  });

  test("viewing someone else's page: no setting falls back to browser detection", async () => {
    freezeClock("America/New_York");
    sessionPayload = { user: { username: "alice", timezone: null } };
    const container = await renderProfile();

    // Detection says it is still 2026-11-13 — the same date the UTC server
    // payload already ends on, so no refetch is needed.
    expect(todayMarker(container).today).toBe("2026-11-13");
    expect(fetchCalls.filter((url) => url.includes("tz="))).toEqual([]);
    expect(fetchCalls).toEqual(["/api/auth/session"]);
  });
});
