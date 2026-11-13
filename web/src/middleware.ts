import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import {
  DEFAULT_LOCALE,
  LOCALE_COOKIE,
  isLocale,
  negotiateLocale,
  stripLocale,
  type Locale,
} from "@/lib/i18n/config";
import { LOCALE_HEADER } from "@/lib/i18n/config";

// Routes that require authentication
const PROTECTED_ROUTES = ["/settings"];

/** The default locale as a prefix, which redirects to the unprefixed path. */
const PREFIXED_DEFAULT = "/en";

/**
 * Reads the remembered language, mapping the pre-prefix spelling onto the
 * current one.
 *
 * The self-made i18n this replaces wrote `tt_locale=zh`; the upstream scheme
 * this adopts spells it `zh-CN`. Without the mapping, everyone who picked
 * Chinese under the old scheme would land on English at `/` until they
 * switched once — the cookie outlives the deploy by a year.
 */
function rememberedLocale(request: NextRequest): Locale | null {
  const remembered = request.cookies.get(LOCALE_COOKIE)?.value;
  if (remembered === "zh") return "zh-CN";
  return isLocale(remembered) ? remembered : null;
}

/**
 * Pages that render the same bytes for every reader in this tree, and the
 * query parameters that actually change what each one renders.
 *
 * These are safe in a shared cache because none of them reads a cookie or a
 * session on the server. Several render dynamically anyway, and Next marks
 * *every* dynamic response `private, no-store`. That default is right in
 * general and wrong here: it costs a full render and several queries on every
 * view of pages nobody needs a private copy of.
 *
 * This list is shorter than upstream's on purpose: their `/leaderboard` entry
 * rests on a perf change (the board ships whole, the client finds the viewer)
 * that this tree has not synced — our board still reads the session and a
 * sort cookie server-side, so it stays private. `/u/...` qualifies here
 * because a private profile renders the same "not found" for everyone and the
 * owner upgrades it in the browser, so the server HTML never depends on the
 * reader.
 *
 * Patterns are tested against the path with its locale prefix stripped, so
 * `/zh-CN/docs` and `/ja/u/x` get exactly the English page's cache header and
 * query allowlist. Each locale is its own URL, so its own entry.
 *
 * `/` is deliberately absent: see `ROOT_CACHE_CONTROL`.
 */
const SHARED_CACHE_ROUTES: ReadonlyArray<{
  pattern: RegExp;
  query: ReadonlySet<string>;
}> = [
  { pattern: /^\/docs$/, query: new Set() },
  { pattern: /^\/terms$/, query: new Set() },
  { pattern: /^\/privacy$/, query: new Set() },
  { pattern: /^\/u\/[^/]+$/, query: new Set(["period"]) },
];

/**
 * How long the edge may answer without asking.
 *
 * Half the 60s `revalidate` on the data underneath, so the edge is never the
 * thing deciding how old a page gets — at worst it repeats one data-cache
 * generation, rather than adding a second window on top of it.
 *
 * The short stale window is the part that matters. A submission invalidates the
 * origin's data cache immediately by tag, but nothing purges the edge, so
 * whatever is set here *is* how long a new number takes to appear. At
 * `stale-while-revalidate=300` a rarely-visited profile could answer from a
 * copy six minutes old; 60s bounds that to two.
 */
const SHARED_CACHE_CONTROL =
  "public, max-age=0, s-maxage=30, stale-while-revalidate=60";

/**
 * What the unprefixed root sends, whichever way it answers.
 *
 * `/` is the one URL whose response depends on the reader: a remembered or
 * negotiated language turns it into a redirect to that language's board. A
 * shared cache keys on the URL alone, so a cached copy would hand the first
 * visitor's language to everyone behind it. It is never stored anywhere.
 */
const ROOT_CACHE_CONTROL = "private, no-store";

/**
 * Next fetches the RSC payload for a client-side navigation from the same path,
 * distinguished only by an `_rsc` query parameter and an `RSC` header.
 *
 * Normalising those away would hand a browser asking for flight data a full
 * HTML document. The cache keys on the whole query string, so `?_rsc=…` is
 * already its own cache entry and needs no help from us; it only needs to be
 * left alone.
 */
function isRscRequest(request: NextRequest): boolean {
  return (
    request.headers.has("RSC") || request.nextUrl.searchParams.has("_rsc")
  );
}

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // The URL is the only source of the locale: a prefix means that language,
  // and its absence means English. Nothing here reads a cookie to decide what
  // an unprefixed page renders — a cookie-dependent body would let the first
  // visitor to reach a shared cache decide what everyone behind it gets.
  //
  // That is the one intentional behaviour change of the locale-prefix cutover:
  // the old unprefixed Chinese URLs keep answering (they render English now),
  // and the remembered cookie still sends `/` to the right language.
  const { locale, path } = stripLocale(pathname);
  const prefixed = locale !== DEFAULT_LOCALE;

  // English is a real prefix too, just not the canonical one, so `/en/docs`
  // redirects to `/docs` rather than 404ing. That keeps `/en` a valid thing to
  // type or link to — and the switcher can generate it like any other language
  // without a special case — while leaving exactly one address per page.
  if (pathname === PREFIXED_DEFAULT || pathname.startsWith(`${PREFIXED_DEFAULT}/`)) {
    // `/en` alone has no page of its own for the same reason `/` does not, and
    // the board is where both end up — going via `/` would cost a second hop.
    const withoutPrefix =
      pathname.slice(PREFIXED_DEFAULT.length) || "/leaderboard";
    const target = new URL(request.nextUrl);
    target.pathname = withoutPrefix;
    return NextResponse.redirect(target, 307);
  }

  // `/zh-CN` on its own is a second name for the board.
  if (prefixed && path === "/") {
    return NextResponse.redirect(new URL(`/${locale}/leaderboard`, request.url));
  }

  // The root is the one route with no content of its own — it redirects to the
  // board. That makes it the only place a remembered or negotiated language can
  // be applied by redirect rather than by rendering two documents on one URL.
  if (!prefixed && path === "/") {
    const preferred =
      rememberedLocale(request) ??
      negotiateLocale(request.headers.get("accept-language"));

    if (preferred && preferred !== DEFAULT_LOCALE) {
      const redirect = NextResponse.redirect(
        new URL(`/${preferred}/leaderboard`, request.url),
      );
      redirect.headers.set("Cache-Control", ROOT_CACHE_CONTROL);
      return redirect;
    }
  }

  const requestHeaders = new Headers(request.headers);
  requestHeaders.set(LOCALE_HEADER, locale);

  /**
   * The two ways this can let a request through.
   *
   * A prefixed URL has to be *rewritten* to the unprefixed path the app routes
   * on — the address bar keeps `/zh-CN/leaderboard`, the renderer sees
   * `/leaderboard`. An unprefixed one is already where it needs to be and only
   * carries the header. Both stamp the locale, so pages never have to guess.
   */
  const pass = () => {
    if (!prefixed) {
      return NextResponse.next({ request: { headers: requestHeaders } });
    }
    const target = request.nextUrl.clone();
    target.pathname = path;
    return NextResponse.rewrite(target, { request: { headers: requestHeaders } });
  };

  if (PROTECTED_ROUTES.some((route) => path.startsWith(route))) {
    const sessionToken = request.cookies.get("tt_session")?.value;

    if (!sessionToken) {
      // This fork signs in with email at /login; upstream's GitHub OAuth route
      // does not exist here. The prefix is kept so signing in lands back on
      // the language the reader was already using.
      const loginUrl = new URL("/login", request.url);
      loginUrl.searchParams.set("returnTo", pathname);
      return NextResponse.redirect(loginUrl);
    }

    // Session exists, allow access.
    // Note: We don't validate the session here to avoid database calls in
    // middleware. The actual validation happens in the page/API route.
    return pass();
  }

  if (!prefixed && path === "/") {
    const response = pass();
    response.headers.set("Cache-Control", ROOT_CACHE_CONTROL);
    return response;
  }

  const route = SHARED_CACHE_ROUTES.find((entry) => entry.pattern.test(path));
  if (!route) {
    return pass();
  }

  if (!isRscRequest(request)) {
    // Every distinct query string is a distinct cache entry. A shared link
    // carrying a utm tag, or anyone appending a counter, would otherwise split
    // one page into unlimited copies and push the entries that matter out.
    //
    // Redirecting rather than silently ignoring the extras keeps one canonical
    // URL per page, which is also what the cache, the logs and any share of
    // that link want. 307 rather than 308: the normalisation is a policy this
    // code owns, and a permanent redirect would be cached by browsers long
    // after a future parameter became meaningful.
    const canonical = canonicalUrl(request, route.query);
    if (canonical) {
      return NextResponse.redirect(canonical, 307);
    }
  }

  const response = pass();
  response.headers.set("Cache-Control", SHARED_CACHE_CONTROL);
  return response;
}

/** The URL this request should have used, or null when it already did. */
function canonicalUrl(
  request: NextRequest,
  allowed: ReadonlySet<string>,
): URL | null {
  const { searchParams } = request.nextUrl;
  const extras = [...searchParams.keys()].filter((key) => !allowed.has(key));
  if (extras.length === 0) return null;

  const url = new URL(request.nextUrl);
  // Rebuilt from the allowlist rather than deleting the extras, so parameter
  // order is fixed too — `?a=1&b=2` and `?b=2&a=1` are one cache entry, not two.
  url.search = "";
  for (const key of [...allowed].sort()) {
    const value = searchParams.get(key);
    if (value !== null) url.searchParams.set(key, value);
  }
  return url;
}

export const config = {
  // Every path whose auth, locale or caching is decided above. One absent from
  // this list never reaches the function, however its conditions are written.
  //
  // Localised routes are listed rather than matched with a catch-all: the
  // matcher also has to keep the middleware off `/api` (the CLI, badges and
  // embeds have no locale prefix and no HTML to cache) and off every static
  // asset. A `/((?!api|_next).*)` pattern would sweep both in, and a plain
  // `.*\..*` exclusion would also catch a username with a dot in it
  // (`/u/foo.bar`) — which has to reach the function like any other profile.
  //
  // Adding a locale means adding its two lines here; there is no way to spot
  // that from the prefix list alone, hence this note. `/en` is listed as well
  // even though it only ever redirects: without an entry here it never reaches
  // the function and `/en/leaderboard` 404s instead.
  //
  // This tree has no `/shame` (the fork replaced it with /teamboard), so
  // upstream's entry for it is dropped.
  matcher: [
    "/",
    "/en",
    "/en/:path*",
    "/zh-CN",
    "/zh-CN/:path*",
    "/ja",
    "/ja/:path*",
    "/settings/:path*",
    "/docs",
    "/terms",
    "/privacy",
    "/u/:username",
  ],
};
