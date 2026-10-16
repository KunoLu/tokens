/// <reference types="bun-types" />
import { afterEach, describe, expect, test } from "bun:test";
import { GET, POST } from "./route";

const ORIGINAL = process.env.CRON_SECRET;

afterEach(() => {
  if (ORIGINAL === undefined) delete process.env.CRON_SECRET;
  else process.env.CRON_SECRET = ORIGINAL;
});

function call(method: "GET" | "POST", authorization?: string): Promise<Response> {
  const headers = new Headers();
  if (authorization !== undefined) headers.set("authorization", authorization);
  const request = new Request(
    "http://localhost/api/cron/refresh-social-links",
    { method, headers },
  );
  return method === "GET" ? GET(request) : POST(request);
}

describe("maintenance cron auth", () => {
  test("GET and POST are 503 when CRON_SECRET is unset", async () => {
    delete process.env.CRON_SECRET;
    for (const method of ["GET", "POST"] as const) {
      const response = await call(method, "Bearer anything");
      expect(response.status).toBe(503);
      expect(await response.json()).toEqual({
        error: "CRON_SECRET is not configured",
      });
    }
  });

  test("GET and POST are 401 without a matching Bearer secret", async () => {
    process.env.CRON_SECRET = "cron-secret-value";
    const cases = [undefined, "Bearer wrong", "cron-secret-value", "Bearer cron-secret-valu"];
    for (const method of ["GET", "POST"] as const) {
      for (const authorization of cases) {
        const response = await call(method, authorization);
        expect(response.status).toBe(401);
        expect(await response.json()).toEqual({ error: "Unauthorized" });
      }
    }
  });
});
