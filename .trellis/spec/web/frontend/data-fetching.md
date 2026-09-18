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
- `app/(main)/teamboard/page.tsx` calls `getTeamboard` + `getSession` the same
  way; `app/api/teamboard/route.ts` is the external JSON surface. Do not
  HTTP-self-fetch.
- `app/u/[username]/page.tsx` loads `loadProfileMembershipForPage` in the same
  `Promise.all` as profile/devices/social. Page-only (D-4): do not add
  Team/Group to `getPublicProfileResponse` / `GET /api/users/[username]`.
  Leave buttons are client `GET /api/auth/session` vs profile username because
  `/u/*` HTML is shared across visitors within its 60s revalidate window.


**Why not HTTP self-fetch:** server-side fetches to our own routes break
under deployment protection and add a pointless network hop; the profile page
documents this. Route handlers exist for external callers (the CLI, embeds,
third parties), not for our own pages.

## Caching: `unstable_cache` + tags

- Cacheable loaders wrap in `unstable_cache` with explicit keys and tags:
  `loadPublicProfileForPage` — `revalidate: 60`,
  `tags: [\`user:${normalizeUsernameCacheKey(username)}\`]`,
  cache key includes locale (`publicProfileData.ts`).
  `getLeaderboardData` — tag `"leaderboard"` (numbers, not copy — locale is
  not in this data cache); `getUserRank` (all-time) has its
  own key and `user-rank` tag (`lib/leaderboard/getLeaderboard.ts`).
  `getTeamboard` — tag `"leaderboard"` (membership/submit already invalidate
  it); cache key includes teamId + sorted groupIds + period(+from/to) +
  sortBy + page + search (`lib/teamboard/getTeamboard.ts`).
- Mutations invalidate by tag: the submit route calls `revalidateTag` after
  writes, fanning out `leaderboard` + `user:<name>` invalidations. The cache
  handler is Next's default (`.next/cache` filesystem + in-memory) — see
  [Self-host Deployment](./self-host-deployment.md).
- Pages/routes also export `revalidate` (`export const revalidate = 60` on
  `u/[username]/page.tsx:10` and the users API route).
- **`revalidate` + `searchParams` flapped on Workers** (a former `/shame`
  page alternated 200/500): pages that depend on `searchParams` keep
  `export const dynamic = 'force-dynamic'` — harmless on self-hosted Node.

## Client `fetch`: mutations and session only

Client components fetch when the user acts, not to render:

- `app/settings/SettingsClient.tsx` — `DELETE /api/settings/submitted-data`,
  token CRUD, device rename; `toast.success/error` on result.
- `components/layout/Navigation.tsx`, `app/device/DeviceClient.tsx`,
  `components/profile/ProfileMembership.tsx` — `GET /api/auth/session` for
  auth state. ProfileMembership compares session username to the profile
  username at render (leave buttons must not be baked into the shared,
  cached `/u/*` HTML). DeviceClient distinguishes load failure from signed-out.

## Parallelism

Independent queries in one page go in one `Promise.all` — e.g. the profile
page loads profile, devices, GitHub social links, and membership together, and
`publicProfileData.ts` batches stats queries the same way. Non-critical
sections degrade individually (devices → `[]`, membership → no block).

Leave mutations (`leaveTeam` / `leaveGroup`) call `bumpLeaderboard()` and look
up `users.username` by `userId` (never a client username) then
`revalidateTag('user:' + normalizeUsernameCacheKey(username))`.
`listMyInvitations` filters expiry with SQL `now()`, not a JS `Date` in raw
`sql`.

## Scenario: GET /api/teamboard

### 1. Scope / Trigger

- Trigger: new JSON API + RSC loader for Teamboard (T7).
- In: `getTeamboard`, `app/api/teamboard/route.ts`, teamboard page/client.
- Out: T8 Profile membership.

### 2. Signatures

- `getTeamboard(teamId, groupIds, viewerId, { period, sortBy, page, search, customFrom, customTo })`
- `GET /api/teamboard?teamId=&groupIds=&period=&sortBy=&page=&search=`

### 3. Contracts

- Missing `teamId` → `{ teams }` list, not a board.
- Present `teamId` → ranked board. Private/unknown/invalid UUID → 404, never 403.
- Repeatable `groupIds` is the union of IDs that exist on this team. Unknown IDs are dropped. If every supplied ID is invalid, `selectedGroupIds` is empty and the loader returns the unfiltered board (same as omitting `groupIds`). PAGE_SIZE is 50.
- Canonical API names are `teamId` / `groupIds`; page URL may still use `team` / `group` aliases.
- RSC never HTTP-self-fetches. Cache tag `"leaderboard"`; key includes teamId + sorted groupIds + period + sortBy + page + search.

### 4. Validation & Error Matrix

| Condition | Result |
|-----------|--------|
| Missing `DATABASE_URL` | Page: `isMissingDatabaseUrl` → empty data. API: 500 `{ error }` — not a fake 200 list |
| Private / unknown / garbage UUID | 404 |
| Missing `teamId` | 200 `{ teams }` |
| Unknown `groupIds` (not on this team) | Dropped. Remaining valid IDs filter to their union. If none remain, `selectedGroupIds=[]` and the loader returns the unfiltered board — not an empty member list |

### 5. Good / Base / Bad Cases

- Good: `teamId` + `groupIds=A&groupIds=B` returns the union of A and B.
- Base: missing `teamId` returns the filter list. All-invalid `groupIds` on a valid `teamId` returns the unfiltered board.
- Bad: swallow missing `DATABASE_URL` inside the loader; parse only `team`/`group`; 403 for private others; treat all-invalid `groupIds` as an empty member list.

### 6. Tests Required

- `bun run test:teams`: 51 members → page 1 length 50, page 2 remainder; `groupIds=[A,B]` union vs `[A]` only A.
- `bun run test:e2e` (`tests/e2e/teamboard.spec.ts`): empty state, FR-2 chrome, `teamId` board, private 404, `groupIds` A vs B, settled `sortBy` URL.

### 7. Wrong vs Correct

#### Wrong
Catch missing `DATABASE_URL` inside `getTeamboard` and return `{ teams: [] }`. Honor only `team`/`group` query names. Return 403 for a private team the viewer cannot see. Treat unknown `groupIds` as empty membership.

#### Correct
`getDb()` throws; the RSC page catches `isMissingDatabaseUrl` like Leaderboard; the API returns 500 `{ error }`. Parse `teamId`/`groupIds` (aliases optional). Private others are 404. Drop unknown `groupIds`; if none remain, return the unfiltered board.
