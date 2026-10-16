/// <reference types="bun-types" />
import { afterEach, describe, expect, mock, test } from "bun:test";

const upstash = {
  calls: 0,
  mode: "allow" as "allow" | "deny" | "throw" | "timeout",
};

mock.module("@upstash/ratelimit", () => ({
  Ratelimit: class Ratelimit {
    static fixedWindow(tokens: number, window: string) {
      return { tokens, window };
    }
    async limit(key: string) {
      upstash.calls += 1;
      if (upstash.mode === "throw") throw new Error(`redis down for ${key}`);
      if (upstash.mode === "timeout") {
        return {
          success: true,
          reason: "timeout",
          pending: Promise.resolve(),
        };
      }
      return {
        success: upstash.mode === "allow",
        pending: Promise.resolve(),
      };
    }
  },
}));

mock.module("@upstash/redis", () => ({
  Redis: class Redis {
    constructor(public config: { url: string; token: string }) {}
  },
}));

// mock.module must run before rateLimit.ts evaluates its Upstash imports.
// A static import would bind the real clients first.
const { authRateLimitAllowed, clientIp } = await import("./rateLimit");

const ENV_KEYS = [
  "UPSTASH_REDIS_REST_URL",
  "UPSTASH_REDIS_REST_TOKEN",
  "VERCEL",
  "TRUST_PROXY",
] as const;

const saved = Object.fromEntries(
  ENV_KEYS.map((key) => [key, process.env[key]]),
) as Record<(typeof ENV_KEYS)[number], string | undefined>;

function restoreEnv(): void {
  for (const key of ENV_KEYS) {
    const value = saved[key];
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}

afterEach(() => {
  restoreEnv();
  upstash.mode = "allow";
});

let ipSeq = 0;

function requestFor(ip: string, extra?: Record<string, string>): Request {
  return new Request("http://localhost/api/auth/login", {
    headers: { "x-forwarded-for": ip, ...extra },
  });
}

function freshIp(): string {
  ipSeq += 1;
  return `203.0.113.${ipSeq}`;
}

describe("clientIp default", () => {
  test("uses the first X-Forwarded-For hop and ignores platform headers", () => {
    delete process.env.VERCEL;
    delete process.env.TRUST_PROXY;
    const request = requestFor("203.0.113.5, 10.0.0.1", {
      "x-real-ip": "198.51.100.9",
      "x-vercel-forwarded-for": "198.51.100.8",
      "cf-connecting-ip": "198.51.100.7",
    });
    expect(clientIp(request)).toBe("203.0.113.5");
  });

  test("falls back to 0.0.0.0 when X-Forwarded-For is missing", () => {
    delete process.env.VERCEL;
    delete process.env.TRUST_PROXY;
    expect(clientIp(new Request("http://localhost/api/auth/login"))).toBe(
      "0.0.0.0",
    );
  });

  test("VERCEL=1 prefers x-vercel-forwarded-for over a forged first hop", () => {
    process.env.VERCEL = "1";
    delete process.env.TRUST_PROXY;
    const request = requestFor("203.0.113.5, 10.0.0.1", {
      "x-vercel-forwarded-for": "198.51.100.8, 10.0.0.2",
      "x-real-ip": "198.51.100.9",
      "cf-connecting-ip": "198.51.100.7",
    });
    expect(clientIp(request)).toBe("198.51.100.8");
  });

  test("TRUST_PROXY prefers x-real-ip when the Vercel header is absent", () => {
    delete process.env.VERCEL;
    process.env.TRUST_PROXY = "true";
    const request = requestFor("203.0.113.5", {
      "x-real-ip": "198.51.100.9",
      "cf-connecting-ip": "198.51.100.7",
    });
    expect(clientIp(request)).toBe("198.51.100.9");
  });

  test("platform trust falls back to the first XFF hop when platform headers are absent", () => {
    process.env.VERCEL = "1";
    const request = requestFor("203.0.113.5, 10.0.0.1");
    expect(clientIp(request)).toBe("203.0.113.5");
  });
});

describe("authRateLimitAllowed fallback", () => {
  test("uses the in-process window when Upstash env is unset", async () => {
    delete process.env.UPSTASH_REDIS_REST_URL;
    delete process.env.UPSTASH_REDIS_REST_TOKEN;
    const before = upstash.calls;
    const ip = freshIp();
    for (let i = 0; i < 10; i += 1) {
      expect(await authRateLimitAllowed(requestFor(ip))).toBe(true);
    }
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(false);
    expect(await authRateLimitAllowed(requestFor(freshIp()))).toBe(true);
    expect(upstash.calls).toBe(before);
  });

  test("stays in-process unless both URL and token are set", async () => {
    process.env.UPSTASH_REDIS_REST_URL = "https://example.upstash.io";
    delete process.env.UPSTASH_REDIS_REST_TOKEN;
    const ip = freshIp();
    for (let i = 0; i < 10; i += 1) {
      expect(await authRateLimitAllowed(requestFor(ip))).toBe(true);
    }
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(false);

    delete process.env.UPSTASH_REDIS_REST_URL;
    process.env.UPSTASH_REDIS_REST_TOKEN = "token";
    const other = freshIp();
    for (let i = 0; i < 10; i += 1) {
      expect(await authRateLimitAllowed(requestFor(other))).toBe(true);
    }
    expect(await authRateLimitAllowed(requestFor(other))).toBe(false);
  });

  test("uses Upstash when both env vars are set and does not consume the process bucket", async () => {
    process.env.UPSTASH_REDIS_REST_URL = "https://example.upstash.io";
    process.env.UPSTASH_REDIS_REST_TOKEN = "token";
    upstash.mode = "allow";
    const before = upstash.calls;
    const ip = freshIp();
    for (let i = 0; i < 11; i += 1) {
      expect(await authRateLimitAllowed(requestFor(ip))).toBe(true);
    }
    expect(upstash.calls).toBe(before + 11);

    delete process.env.UPSTASH_REDIS_REST_URL;
    delete process.env.UPSTASH_REDIS_REST_TOKEN;
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(true);
  });

  test("fails closed on Upstash errors and timeouts without touching the process bucket", async () => {
    process.env.UPSTASH_REDIS_REST_URL = "https://example.upstash.io";
    process.env.UPSTASH_REDIS_REST_TOKEN = "token";
    const ip = freshIp();

    upstash.mode = "throw";
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(false);
    upstash.mode = "timeout";
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(false);

    delete process.env.UPSTASH_REDIS_REST_URL;
    delete process.env.UPSTASH_REDIS_REST_TOKEN;
    expect(await authRateLimitAllowed(requestFor(ip))).toBe(true);
  });
});
