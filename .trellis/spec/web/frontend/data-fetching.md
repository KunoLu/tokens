# Data Fetching

> RSC pages call `lib/` functions directly — they never HTTP-fetch their own
> `/api` routes. Client components `fetch` only for mutations and session
> state.

---

## The core pattern: shared lib handlers

One function serves both the API route and the page:

- `getPublicProfileResponse` in `web/src/lib/publicProfileData.ts:96` is called
  by `app/api/users/[username]/route.ts` over HTTP **and** internally by
  `loadPublicProfileForPage` (same file, lines 692-727), which invokes it with
  a synthetic `Request` (`http://tokens.internal/...`).
- `app/(main)/leaderboard/page.tsx` calls `getLeaderboardData`, `getSession`,
  `getUserRank` directly in an async server component with `Suspense` +
  skeleton fallback.
- `app/(main)/shame/page.tsx` runs `db.select()` straight in the server
  component (`force-dynamic`).

**Why not HTTP self-fetch:** server-side fetches to our own routes break
under deployment protection and add a pointless network hop; the profile page
documents this. Route handlers exist for external callers (the CLI, embeds,
third parties), not for our own pages.

## Caching: `unstable_cache` + tags

- Cacheable loaders wrap in `unstable_cache` with explicit keys and tags:
  `loadPublicProfileForPage` — `revalidate: 60`,
  `tags: [\`user:${normalizeUsernameCacheKey(username)}\`]`
  (`publicProfileData.ts:696-724`).
  `getLeaderboardData` — tag `"leaderboard"`; `getUserRank` (all-time) has its
  own key and `user-rank` tag (`lib/leaderboard/getLeaderboard.ts`).
- Mutations invalidate by tag: the submit route calls `revalidateTag` after
  writes, fanning out `leaderboard` + `user:<name>` invalidations (that
  write-heaviness is why the tag cache is a *sharded* Durable Object — see
  [Cloudflare Deployment](./cloudflare-deployment.md)).
- Pages/routes also export `revalidate` (`export const revalidate = 60` on
  `u/[username]/page.tsx:10` and the users API route).
- **`revalidate` + `searchParams` flaps on Workers** (the shame page alternated
  200/500): pages that depend on `searchParams` use
  `export const dynamic = 'force-dynamic'` instead.

## Client `fetch`: mutations and session only

Client components fetch when the user acts, not to render:

- `app/settings/SettingsClient.tsx` — `DELETE /api/settings/submitted-data`,
  token CRUD, device rename; `toast.success/error` on result.
- `components/layout/Navigation.tsx`, `app/device/DeviceClient.tsx` —
  `GET /api/auth/session` for auth state; DeviceClient distinguishes load
  failure from signed-out.

## Parallelism

Independent queries in one page go in one `Promise.all` — e.g. the profile
page loads profile, devices, and GitHub social links together, and
`publicProfileData.ts` batches stats queries the same way. Non-critical
sections degrade individually (devices wrapped in try/catch → `[]`).
