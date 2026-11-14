/// <reference types="bun-types" />
import { afterEach, describe, expect, mock, test } from "bun:test";
import * as schema from "@/lib/db/schema";

let session: { id: string; username: string } | null = { id: "u1", username: "alice" };
mock.module("@/lib/auth/requestSession", () => ({
  getSessionFromRequest: async () => session,
}));

const selectResults: unknown[][] = [];
const updateValues: unknown[] = [];
function chain(result: unknown[]) {
  const builder: Record<string, unknown> = {};
  for (const method of ["from", "where", "limit"]) {
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
    update: () => ({
      set: (values: unknown) => {
        updateValues.push(values);
        return { where: async () => [] };
      },
    }),
  },
}));

const { GET, PATCH } = await import("./route");

function patch(body: unknown): Promise<Response> {
  return PATCH(
    new Request("http://localhost/api/settings/timezone", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }),
  );
}

afterEach(() => {
  // The mock outlives this file in the shared module registry; leave it
  // answering "no session", which is what an unauthenticated request sees.
  session = null;
});

describe("GET /api/settings/timezone", () => {
  test("is 401 without a session", async () => {
    session = null;
    const response = await GET(new Request("http://localhost/api/settings/timezone"));
    expect(response.status).toBe(401);
  });

  test("returns the saved zone, null when unset (auto-detect)", async () => {
    session = { id: "u1", username: "alice" };
    selectResults.push([{ timezone: "Asia/Shanghai" }]);
    const withZone = await GET(new Request("http://localhost/api/settings/timezone"));
    expect(withZone.status).toBe(200);
    expect(await withZone.json()).toEqual({ timezone: "Asia/Shanghai" });

    selectResults.push([{ timezone: null }]);
    const withoutZone = await GET(new Request("http://localhost/api/settings/timezone"));
    expect(await withoutZone.json()).toEqual({ timezone: null });
  });
});

describe("PATCH /api/settings/timezone", () => {
  test("is 401 without a session and writes nothing", async () => {
    session = null;
    const response = await patch({ timezone: "Asia/Shanghai" });
    expect(response.status).toBe(401);
    expect(updateValues).toHaveLength(0);
  });

  test("rejects non-IANA names with 400 and writes nothing", async () => {
    session = { id: "u1", username: "alice" };
    for (const bad of ["Not/AZone", "", "UTC+8", 8, undefined]) {
      const response = await patch(bad === undefined ? {} : { timezone: bad });
      expect(response.status).toBe(400);
    }
    expect(updateValues).toHaveLength(0);
  });

  test("stores a valid IANA zone", async () => {
    session = { id: "u1", username: "alice" };
    const response = await patch({ timezone: "America/New_York" });
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({ timezone: "America/New_York" });
    expect(updateValues.at(-1)).toMatchObject({ timezone: "America/New_York" });
  });

  test("null clears the override back to auto-detect", async () => {
    session = { id: "u1", username: "alice" };
    const response = await patch({ timezone: null });
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({ timezone: null });
    expect(updateValues.at(-1)).toMatchObject({ timezone: null });
  });
});
