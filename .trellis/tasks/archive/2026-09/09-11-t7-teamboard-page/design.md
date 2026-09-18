# T7 Teamboard page

Parent implement.md T7, D-2, `web/features/leaderboard-and-teamboard.feature`, and docs PRD FR-2 / §9.2 are the source of truth.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | on-demand | — | not-required |
| book-ddia-data-design | getTeamboard loader, API 404 vs 403, cache | before implement | passed |
| book-legacy-change-safety | existing teamboard placeholder | before first existing-file edit | passed |
| book-refactoring-pass | teamboard page + reuse Leaderboard columns | before those edits | passed |
| book-release-readiness | `/api/teamboard` + PAGE_CACHEABLE HTML | after validation | passed |

grill-with-docs: skipped. D-2 locked.

## Change boundary

- In: replace `app/(main)/teamboard/page.tsx`, `components/teamboard/`, `lib/teamboard/getTeamboard.ts`, `app/api/teamboard/route.ts`, OG metadata, `t()` keys, Playwright for empty state / 404 if fixture-cheap.
- Out: T8 Profile membership, T11 remaining copy, new schema.

## Visibility (D-2)

Filter list = `public` AND `status=active` ∪ viewer's membership (including private). Unauthenticated visitors see public active teams only. `/api/teamboard` and the page for another user's `private` team return **404**, not 403. Use `canViewTeam`.

## UI contract

- Filters in URL searchParams. HTML page: `team` (single) + `group` (multi) + Leaderboard's `period` / `sortBy` / `search` / `page` (`from` for the viewer-local today correction). `export const dynamic = 'force-dynamic'` (searchParams + Workers).
- RSC loads via `lib/teamboard` (no HTTP self-fetch). Client only updates the URL.
- Columns: `# / Developer / Group / Tokens / Cost` — **no Team column**. Reuse T6 membership cell for Group only.
- No team selected → guide empty state (keep the filter).
- Tokens/Cost header remains format toggle, not sort.
- **FR-2 controls (required, matching Leaderboard):** Period five-way (`All time` / `Today` / `Week` / `Month` / `Last month`; landing period is `all`), Sort by (`Tokens` / `Cost`), search (username / displayName plus `client:` / `model:` directives), pagination (same 50-row page size; Previous/Next hidden until there is a second page). Team single-select + Group multi-select stay. Reuse `PERIODS`, `DeveloperRow`, `FormatToggle`, `pageRanking` / `aggregatePeriodRows` / `compareLeaderboardUsers` — do not split `Leaderboard.tsx`.
- **API (PRD §9.3):** `GET /api/teamboard?teamId=&groupIds=&period=&sortBy=&page=&search=`. Missing `teamId` (and missing page-URL alias `team`) returns `{ teams }`. With `teamId`, return the board (404 if `canViewTeam` is false). Page-URL aliases `team` / `group` are accepted on the API so existing links keep working.

## DDIA Data Design Review

Status: passed

Data owner and source of truth: existing `teams` / `groups` / `team_members` / submissions. No new tables.

Write / read / async / failure paths: read-only page. Mutations stay on T4/T5. Invalid/private team id → 404. `getDb()` throws if `DATABASE_URL` is unset (`db/index.ts`). The RSC page catches `isMissingDatabaseUrl` like leaderboard and renders empty filter/table — do **not** swallow that error inside the loader. The API route does not invent a 200 empty list; it returns `{ error }` 500 like other loaders when the DB URL is missing.

Consistency model: same as leaderboard (`unstable_cache` + `leaderboard` tag). Teamboard numbers change when submissions or membership change; `bumpLeaderboard` already invalidates that tag.

Idempotency / ordering / retry / deduplication: N/A (GET).

Schema / migration / backfill / rollback / replay: none. Rollback = revert UI + route.

Observability and repair: `{ error }` JSON on API 404/500. Root `error.tsx` for render throws.

Required tests: `test:teams` if visibility helper is touched; Playwright signed-out empty state; optional 404 with fixture team if cheap.

## Legacy Change Safety Review

Status: characterized

Behavior to change: placeholder copy → real Teamboard with filters and table.

Behavior to preserve: `CONTAINER`, `PageHeader`, `id="main-content"`, nav `/teamboard` link, worker PAGE_CACHEABLE signed-out cache (URL already includes searchParams; `__locale` stays).

Current reproduction evidence: page is static placeholder, no DB.

Safety net: lint/typecheck; Playwright empty-state spec.

Hidden dependencies / seam: none.

Validation plan: `bun run lint`, `typecheck`, `test:e2e`.

Review mode: normal

## Refactoring Review

Status: proceed

Review mode: normal

Existing-code scope: `teamboard/page.tsx` rewrite; reuse `MembershipCells` / Leaderboard row pieces without extracting a new table framework.

Behavior that must remain unchanged: Leaderboard still has Team+Group columns.

Structural friction: none worth extracting first.

Decision and smallest safe step: new `getTeamboard` + client shell; pass `hideTeam` or a Teamboard-specific column set. Do not split Leaderboard.tsx.

Safety net and validation: existing Leaderboard Playwright LocaleToggle + new teamboard empty-state spec.

Deferred refactors: none.

## Ponytail

Replace the placeholder. One loader, one client shell, URL filters. Reuse leaderboard types/formatters. Skip new component libraries.

## Check (2026-09-14, URL Back + groupIds)

Verdict: **pass**. No commit.

FR-2 and §9.3 hold: Period five-way (`PERIODS`, landing `all`) + Sort by + search + pagination (50-row, Previous/Next hidden until a second page); Team single-select + Group multi-select; no Team column (`hideTeam` / `includeTeam={false}`); page URL keeps `team` / `group`; API parses `teamId` / `groupIds` / `period` / `sortBy` / `page` / `search` (page aliases accepted). Missing `teamId` → `{teams}`; with `teamId` → board; private others 404 not 403 via `canViewTeam`. RSC calls `lib/teamboard` (no HTTP self-fetch); `force-dynamic`; `isMissingDatabaseUrl` on the PAGE not the loader; API 500 `{ error }` when `DATABASE_URL` is missing; `users.ts` loopback fail-closed unchanged. Feature file includes the Period/Sort/search/pagination and §9.3 scenarios (`groupIds` named on the API scenario).

URL Back: `TeamboardClient` keeps local `search` / `sortBy` for typing and optimistic toggles, then follows the landed URL/server snapshot during render (`prevUrlSearch` / `prevServerSortBy`). The implementer's `useEffect(() => setSearch/setSortBy)` failed `react-hooks/set-state-in-effect`; check rewrote it to the same React-during-render pattern already used for `pendingPeriod`. E2E: after search submit, first Back clears the box while `sortBy=cost` remains; second Back drops `sortBy` and Tokens is `aria-pressed=true`.

`groupIds` E2E: create groups A/B, put the admin in A; `GET ?teamId=&groupIds=A` returns that admin; `groupIds=B` returns `[]`. `addGroupMember` already `bumpLeaderboard()`.

Validation: `bun run lint` 0 errors (2 pre-existing warnings in `docs/page.tsx` and `worker.ts`); `bun run typecheck` pass; native `bun run test:e2e` 8 passed (1.6m). `rtk`: skipped-for-report. `test:teams` / `test:migrations` / `build` skipped (no schema change; assignment did not require build).

Reports: `tests/e2e/reports/html/playwright-report-leaderboard-and-teamboard-feature_teamboard-teams-auth-2026_09_14-21_57_03.html` + same-stem `.md`.

## Check (2026-09-14, URL-derived sortBy)

Verdict: **pass**. No commit.

Displayed Sort by follows the URL: `resolvedSortBy = valid url sortBy ?? serverSortBy` with the during-render snapshot (`prevResolvedSortBy`). A tokens URL (missing `sortBy`) resets Tokens even when the Cost RSC never became current. Invalid `sortBy` query values are ignored (`urlSort` null → server).

FR-2 + §9.3 unchanged from the prior check. `users.ts` loopback fail-closed unchanged. Immediate Cost-click → Back is not asserted (flaky). E2E still covers Back-after-search (`sortBy=cost` kept, then dropped / Tokens pressed) and now also `goto` a settled `sortBy=cost` URL (Cost pressed) then `goto` the tokens URL (Tokens `aria-pressed=true`).

Validation: `bun run lint` 0 errors (2 pre-existing warnings in `docs/page.tsx` and `worker.ts`); `bun run typecheck` pass; native `bun run test:e2e` 8 passed (1.2m). `rtk`: skipped-for-report. `test:teams` / `test:migrations` / `build` skipped (no schema change; assignment did not require build).

Reports: `tests/e2e/reports/html/playwright-report-leaderboard-and-teamboard-feature_teamboard-teams-auth-2026_09_14-22_09_06.html` + same-stem `.md`.

Code Readability Review

Scope: modified hand-written production code and tests

Findings: none

Ponytail conflicts resolved: none

Changes applied: deterministic settled-URL sort assertion in `tests/e2e/teamboard.spec.ts` only

Revalidation required: no (the native e2e run already included the new assertion)


## Release Readiness Review

Status: ready

Production path and affected users / systems: signed-out HTML `/teamboard` (PAGE_CACHEABLE) and `GET /api/teamboard` for guests and members ranking usage inside one team.

Failure modes and safeguards: unknown / private-to-viewer team → 404 (not 403). Missing `DATABASE_URL` → page empty board, API 500 `{ error }` (not a fake 200 list). UUID garbage → 404 (avoids Postgres 22P02). Cache tag `leaderboard` already bumped on team/group/membership mutations. Displayed Sort by follows a valid URL `sortBy` (`cost` / `tokens`) even if the Cost RSC never landed; missing or invalid param falls back to the server/cookie default.

Capacity / backpressure / limits: read-only GET; `unstable_cache` 60s per (team, groups, period, sort, page, search) + per-team bundle + public-active list. No new write path or queue.

Observability / alerts / runbook: API `{ error }` JSON + `teamErrorResponse` / `console.error`; page `notFound()` and root `error.tsx`. No new alerts.

Rollout / migration / rollback / cleanup: no schema. Rollback = revert page, client, loader, API route. Feature flag: none.

Required validation and result: lint 0 errors; typecheck pass; native Playwright 8/8 including FR-2 chrome, Back search/sortBy URL sync, settled `sortBy=cost` → tokens URL Tokens pressed, §9.3 `teamId` vs `{teams}`, `groupIds` A-hit / B-empty, and 404 for private others.

Optional checks, accountable owner acceptance, and residual risk: `bun run build` / `cf:build` not run this check (assignment did not require them; owner 640). Dirty worktree evidence is local-only. Pagination Next and multi-value `groupIds` (two occupied groups) not fixture-exercised (one-member team hides pager); code path matches Leaderboard page size 50. Single-group `groupIds` is E2E-covered. Immediate Cost-click → Back race not asserted (flaky); settled-URL navigation covers the URL-derived control.

## Check (2026-09-14, FR-2 pagination + multi groupIds)

Verdict: **pass**. No commit.

Pagination and multi-value `groupIds` are required FR-2 contracts, exercised this check — not residuals. `web/scripts/check-teams-invariants.ts` seeds 51 members plus disjoint groups A (25) / B (25) / ungrouped (1) via SQL. Assertions: page 1 length 50, `totalUsers === 51`, `hasNext`; page 2 remainder 1, `hasPrev` and not `hasNext`; the two pages tile 51 usernames without overlap; `groupIds=[A,B]` returns the 50-member union with `selectedGroupIds.length === 2`; `groupIds=[A]` returns only A (25) and none of B. Mutation proof of those cases: `TEAMBOARD_PAGE_SIZE` 50→100000 fails page-1 (`members.length === 50`); dropping extra `groupIds` so the loader keeps only `[A]` fails the union.

`next/cache` `unstable_cache` is mocked as a call-through in the invariant script (dynamic `import` of `getTeamboard` after `mock.module`) so the Next incremental cache is not required outside the server; each call re-reads the fixtures.

FR-2 / §9.3 / D-2 unchanged: Period five-way landing `all`, Sort by, search, 50-row pager (Previous/Next hidden until a second page), Team single-select + Group multi-select, no Team column, page URL `team`/`group`, API `teamId`/`groupIds`/aliases, private others 404 not 403, RSC calls `lib/teamboard` (no HTTP self-fetch), `force-dynamic`, missing `DATABASE_URL` degrades on the page and 500s the API. `tests/e2e/users.ts` loopback fail-closed unchanged (not in this diff).

Validation: `bun run lint` 0 errors (2 pre-existing warnings in `docs/page.tsx` and `worker.ts`); `bun run typecheck` pass; native `bun run test:teams` pass including the five FR-2 cases above (20.5s); native `bun run test:e2e` 8 passed (1.4m). `rtk`: skipped-for-report. `test:migrations` / `build` skipped (no schema change; assignment did not require build).

Reports: `tests/e2e/reports/html/playwright-report-leaderboard-and-teamboard-feature_teamboard-teams-auth-2026_09_14-22_32_08.html` + same-stem `.md`.

Code Readability Review

Scope: modified hand-written production code and tests

Findings: none

Ponytail conflicts resolved: none

Changes applied this check: invariant FR-2 cases already landed by implement; this check added the named Playwright snapshot and this design note. No production-code edits.

Revalidation required: no (lint, typecheck, `test:teams`, and native e2e already ran on the invariant-bearing tree)

## Release Readiness Review

Status: passed

Production path and affected users / systems: signed-out HTML `/teamboard` (PAGE_CACHEABLE) and `GET /api/teamboard` for guests and members ranking usage inside one team.

Failure modes and safeguards: unknown / private-to-viewer team → 404 (not 403). Missing `DATABASE_URL` → page empty board, API 500 `{ error }` (not a fake 200 list). UUID garbage → 404 (avoids Postgres 22P02). Cache tag `leaderboard` already bumped on team/group/membership mutations. Displayed Sort by follows a valid URL `sortBy` (`cost` / `tokens`) even if the Cost RSC never landed; missing or invalid param falls back to the server/cookie default. Board page size is 50 (`TEAMBOARD_PAGE_SIZE`); group filter is a set-union of valid `groupIds`.

Capacity / backpressure / limits: read-only GET; `unstable_cache` 60s per (team, groups, period, sort, page, search) + per-team bundle + public-active list. Page and search clamped (`MAX_PAGE` / `MAX_SEARCH_LENGTH`) because they join the cache key. No new write path or queue.

Observability / alerts / runbook: API `{ error }` JSON + `teamErrorResponse` / `console.error`; page `notFound()` and root `error.tsx`. No new alerts.

Rollout / migration / rollback / cleanup: no schema. Rollback = revert page, client, loader, API route, invariant cases. Feature flag: none.

Required validation and result: lint 0 errors; typecheck pass; native `test:teams` including FR-2 page1=50 / page2 remainder / `groupIds=[A,B]` union vs `[A]`; native Playwright 8/8 including FR-2 chrome, Back search/sortBy URL sync, settled `sortBy=cost` → tokens URL Tokens pressed, §9.3 `teamId` vs `{teams}`, single-group `groupIds` A-hit / B-empty, and 404 for private others.

Optional checks, accountable owner acceptance, and residual risk: `bun run build` passed at close-out (`/teamboard` in the Next route table). `cf:build` skipped. Dirty worktree evidence was local-only at check time; work is now `21a84933` / `87a456b1` / `31fcf9d1`. Immediate Cost-click → Back race not asserted (flaky); settled-URL navigation covers the URL-derived control. E2E still uses a one-member team so Previous/Next stay hidden; the 51-member page slice and multi-value union are proven by `test:teams`, not left unexercised.

## Close-out (2026-09-14)

Book Gate Plan states use the required enum: DDIA `passed`, release-readiness `passed` (not `confirmed` / `ready`).

`cd web && bun run build` passed after the last check; `/teamboard` is in the route table. `cf:build` remains skipped. `test:migrations` remains skipped (no schema change).

Work commits: `21a84933` feat, `87a456b1` contracts, `31fcf9d1` archive. This close-out note is a follow-up docs correction (no amend).
