import { describe, expect, test } from "bun:test";
import { NextRequest } from "next/server";
import { middleware } from "./middleware";

/**
 * Covers the two compatibility promises of the locale-prefix cutover:
 *
 * - a pre-prefix `tt_locale=zh` cookie still means Chinese, now spelled zh-CN;
 * - the old unprefixed Chinese URLs keep working — every unprefixed page
 *   renders (in English, by design) and none of them 404 or redirect away.
 *
 * Plus the routing invariants the middleware owns: prefix rewrite with the
 * locale header, the `/en` canonicalisation, bare-prefix redirects, and the
 * `/settings` session gate pointing at the email login page.
 */

const BASE = "https://tokens.example";

function req(path: string, init?: { cookie?: string; acceptLanguage?: string }) {
  const headers = new Headers();
  if (init?.cookie !== undefined) headers.set("cookie", `tt_locale=${init.cookie}`);
  if (init?.acceptLanguage) headers.set("accept-language", init.acceptLanguage);
  return new NextRequest(`${BASE}${path}`, { headers });
}

function authed(path: string) {
  return new NextRequest(`${BASE}${path}`, {
    headers: new Headers({ cookie: "tt_session=abc123" }),
  });
}

describe("legacy cookie mapping", () => {
  test("tt_locale=zh redirects / to the zh-CN board", () => {
    const res = middleware(req("/", { cookie: "zh" }));
    expect(res.status).toBe(307);
    expect(res.headers.get("location")).toBe(`${BASE}/zh-CN/leaderboard`);
  });

  test("current cookie values still work", () => {
    expect(middleware(req("/", { cookie: "zh-CN" })).headers.get("location")).toBe(
      `${BASE}/zh-CN/leaderboard`,
    );
    expect(middleware(req("/", { cookie: "ja" })).headers.get("location")).toBe(
      `${BASE}/ja/leaderboard`,
    );
  });

  test("an English cookie stays on the unprefixed root", () => {
    const res = middleware(req("/", { cookie: "en" }));
    expect(res.status).toBe(200);
  });

  test("a junk cookie falls back to Accept-Language negotiation", () => {
    const res = middleware(req("/", { cookie: "klingon", acceptLanguage: "ja,en;q=0.5" }));
    expect(res.headers.get("location")).toBe(`${BASE}/ja/leaderboard`);
  });
});

describe("unprefixed legacy URLs", () => {
  test.each([
    "/leaderboard",
    "/teams",
    "/teamboard",
    "/docs",
    "/terms",
    "/privacy",
    "/u/someone",
    "/settings",
  ])("%s does not 404 and is not redirected to a prefixed URL", (path) => {
    const res = middleware(authed(path));
    const location = res.headers.get("location");
    if (location) {
      expect(location).not.toMatch(/\/(zh-CN|ja)(\/|$)/);
    }
    // A pass-through has no rewrite either: the app routes on the path as-is.
    expect(res.headers.get("x-middleware-rewrite")).toBeNull();
  });
});

describe("prefix handling", () => {
  test("a prefixed page is rewritten unprefixed with the locale header", () => {
    const res = middleware(req("/zh-CN/leaderboard"));
    expect(res.headers.get("x-middleware-rewrite")).toBe(`${BASE}/leaderboard`);
    expect(res.headers.get("x-middleware-request-x-tokens-locale")).toBe("zh-CN");
  });

  test("a bare prefix redirects to that language's board", () => {
    const res = middleware(req("/ja"));
    expect(res.status).toBe(307);
    expect(res.headers.get("location")).toBe(`${BASE}/ja/leaderboard`);
  });

  test("/en is canonicalised to the unprefixed path", () => {
    const res = middleware(req("/en/leaderboard"));
    expect(res.status).toBe(307);
    expect(res.headers.get("location")).toBe(`${BASE}/leaderboard`);
  });

  test("an unprefixed page is stamped English", () => {
    const res = middleware(req("/leaderboard"));
    expect(res.headers.get("x-middleware-request-x-tokens-locale")).toBe("en");
  });
});

describe("settings auth gate", () => {
  test("unauthenticated /settings redirects to the email login", () => {
    const res = middleware(req("/settings"));
    expect(res.status).toBe(307);
    expect(res.headers.get("location")).toBe(
      `${BASE}/login?returnTo=${encodeURIComponent("/settings")}`,
    );
  });

  test("the redirect keeps the reader's prefix", () => {
    const res = middleware(req("/zh-CN/settings"));
    expect(res.headers.get("location")).toBe(
      `${BASE}/login?returnTo=${encodeURIComponent("/zh-CN/settings")}`,
    );
  });

  test("an authenticated request passes with the locale header", () => {
    const res = middleware(authed("/settings"));
    expect(res.headers.get("location")).toBeNull();
    expect(res.headers.get("x-middleware-request-x-tokens-locale")).toBe("en");
  });
});

describe("shared cache list", () => {
  const SHARED = "public, max-age=0, s-maxage=30, stale-while-revalidate=60";

  test("the board is shared-cacheable now that it ships whole", () => {
    const res = middleware(req("/leaderboard"));
    expect(res.headers.get("cache-control")).toBe(SHARED);
  });

  test("a prefixed board gets the same header as the English one", () => {
    const res = middleware(req("/zh-CN/leaderboard"));
    expect(res.headers.get("cache-control")).toBe(SHARED);
  });

  test("allowlisted view parameters keep the board cacheable", () => {
    const res = middleware(req("/leaderboard?period=week&sortBy=cost&page=2"));
    expect(res.headers.get("cache-control")).toBe(SHARED);
    expect(res.headers.get("location")).toBeNull();
  });

  test("a stray parameter is canonicalised away instead of splitting the cache", () => {
    const res = middleware(req("/leaderboard?utm_source=x"));
    expect(res.headers.get("location")).toBe(`${BASE}/leaderboard`);
  });

  test("the teamboard stays out of the shared cache: it answers per viewer", () => {
    const res = middleware(req("/teamboard"));
    expect(res.headers.get("cache-control")).toBeNull();
  });
});
