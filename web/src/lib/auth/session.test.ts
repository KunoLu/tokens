/// <reference types="bun-types" />
import { afterEach, describe, expect, mock, test } from "bun:test";
import * as schema from "@/lib/db/schema";

// The cookie jar getSession() reads. Tests flip it via `sessionCookie`.
let sessionCookie: string | null = "raw-session-token";
mock.module("next/headers", () => ({
  cookies: async () => ({
    get: (name: string) =>
      sessionCookie === null ? undefined : { name, value: sessionCookie },
    set: () => {},
    delete: () => {},
  }),
}));

// Queued answers, one per db.select() call: an array resolves, an Error
// rejects — that is how an unmigrated database (no timezone column) shows up.
const selectQueue: Array<unknown[] | Error> = [];
function chain(result: unknown[] | Error) {
  const builder: Record<string, unknown> = {};
  for (const method of ["from", "where", "limit", "innerJoin"]) {
    builder[method] = () => builder;
  }
  builder.then = (
    resolve: (value: unknown) => unknown,
    reject: (error: unknown) => unknown,
  ) =>
    result instanceof Error
      ? Promise.reject(result).then(resolve, reject)
      : Promise.resolve(result).then(resolve, reject);
  return builder;
}
mock.module("@/lib/db", () => ({
  ...schema,
  db: { select: () => chain(selectQueue.shift() ?? []) },
}));

// Dynamic import so the mock.module registrations above win: a static import
// would be hoisted and load the real module graph before the mocks exist
// (same boundary pattern as the settings route test).
const { getSession } = await import("./session");

function joinRow(user: Record<string, unknown>) {
  return [{ session: { id: "s1" }, user }];
}

const alice = {
  id: "u1",
  username: "alice",
  displayName: "Alice",
  avatarUrl: null,
  bannedAt: null,
};

/** Postgres reports a missing column as 42703. */
function missingColumnError() {
  return Object.assign(
    new Error('column "timezone" of relation "users" does not exist'),
    { code: "42703" },
  );
}

afterEach(() => {
  // Shared module registry: leave the jar empty for whatever runs next.
  sessionCookie = null;
  selectQueue.length = 0;
});

describe("getSession", () => {
  test("returns null without a session cookie", async () => {
    sessionCookie = null;
    expect(await getSession()).toBeNull();
  });

  test("returns null for a banned user", async () => {
    sessionCookie = "raw-session-token";
    selectQueue.push(joinRow({ ...alice, bannedAt: new Date() }));
    expect(await getSession()).toBeNull();
  });

  test("carries the account time zone read by a dedicated query", async () => {
    sessionCookie = "raw-session-token";
    selectQueue.push(joinRow(alice), [{ timezone: "Asia/Shanghai" }]);
    const session = await getSession();
    expect(session?.username).toBe("alice");
    expect(session?.timezone).toBe("Asia/Shanghai");
  });

  test("a missing timezone column (migration not run) resolves to auto-detect null, not an error", async () => {
    sessionCookie = "raw-session-token";
    selectQueue.push(joinRow(alice), missingColumnError());
    const session = await getSession();
    expect(session?.username).toBe("alice");
    expect(session?.timezone).toBeNull();
  });

  test("any other time zone read failure also degrades to null", async () => {
    sessionCookie = "raw-session-token";
    selectQueue.push(joinRow(alice), new Error("connection reset"));
    const session = await getSession();
    expect(session?.timezone).toBeNull();
  });

  test("no row from the time zone query means null", async () => {
    sessionCookie = "raw-session-token";
    selectQueue.push(joinRow(alice), []);
    const session = await getSession();
    expect(session?.timezone).toBeNull();
  });
});
