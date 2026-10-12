# T8 Profile team membership

Parent implement.md T8, FR-6, INV-8, and `web/features/profile-team-membership.feature` are the source of truth. D-4 cancelled the public profile HTTP Team/Group contract.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | admin leave, INV-8 cascade | before design | passed |
| book-ddia-data-design | leave writes, profile `user:` cache, PROFILE_CACHEABLE | before implement | passed |
| book-legacy-change-safety | existing ProfileView / publicProfileData | before first existing-file edit | passed |
| book-refactoring-pass | ProfileView + leaveTeam/leaveGroup | before those edits | passed |
| book-release-readiness | leave buttons + DELETE /members/me | after validation | passed |

grill-with-docs: skipped (logged in `grill.log`). Post-grill DDD not required; independent DDD ran because admin-leave / INV-8 are domain rules.

## Change boundary

- In: page-only membership query; ProfileView Team/Group block; client leave buttons from session; `leaveTeam`/`leaveGroup` also `revalidateTag('user:…')`.
- Out: new leave endpoints (T4 already has them); GET `/api/users/[username]` Team/Group JSON (D-4); T11 remaining copy; schema.

## DDD Boundary Review

Status: confirmed

Ubiquitous language: Team, Group, admin, member, leave Team, leave Group. `isOwner` means the signed-in user is this profile's user (username match), not Team admin.

Bounded contexts: Profile display (public names) vs Team lifecycle (leave mutations). Do not mix admin role into the cached HTML.

Invariants: INV-8 leave Team deletes `group_members` then `team_members` in one txn (already in `leaveTeam`). Admin leave Team → 409 "Transfer admin or disband the team before leaving". Leave Group does not leave Team.

Core / supporting / generic: Team membership core; profile chrome supporting.

Corrections to grill-with-docs: none (grill skipped).

Open conflicts: none.

## DDIA Data Design Review

Status: confirmed

Data owner and source of truth: `team_members` / `group_members`. No new tables.

Write / read / async / failure paths: reads — page-only query in parallel with `loadPublicProfileForPage`, not inside `getPublicProfileResponse`. Writes — existing `DELETE /api/teams/:teamId/members/me` and `DELETE /api/teams/:teamId/groups/:groupId/members/me`. Admin leave 409. Missing `DATABASE_URL`: membership query fails soft to "no block" like devices → `[]`.

Consistency model: `leaveTeam`/`leaveGroup` already `bumpLeaderboard()`. After the membership txn commits, look up `users.username` by the same `userId` (server-side only — never a client-supplied username) and `revalidateTag('user:' + normalizeUsernameCacheKey(username), 'max')`. Otherwise `router.refresh()` can still serve the 60s cached profile. PROFILE_CACHEABLE caches `/u/*` for everyone — leave buttons must not be in RSC HTML. Membership names are loaded on the page path outside `getPublicProfileResponse` so D-4 stays intact.
Idempotency: second leave → 403 not a member. Retry after 409 admin is a no-op until transfer/disband.

Schema / migration / rollback: none. Rollback = revert page/client/view + user-tag invalidation.

Observability: existing `teamErrorResponse` `{ error }` + toast.

Required tests: Playwright for `profile-team-membership.feature` (no block, names visible to visitors, leave only for owner, unsigned-out/other user no leave, admin 409). `test:teams` already covers `leaveTeam` admin 409 and INV-8.

## Legacy Change Safety Review

Status: characterized

Behavior to change: add Team/Group block and owner-only leave.

Behavior to preserve: profile stats/charts/social/embed; GET `/api/users/[username]` JSON shape; `leaveTeam`/`leaveGroup` HTTP contracts; TeamsClient leave; username case canonical 308; PROFILE_CACHEABLE 200-only.

Current reproduction evidence: ProfileUser has no team/group; ProfileView has no membership block; leave APIs exist.

Safety net: existing feature file + new Playwright; `test:teams` leave cases.

Hidden dependencies: PROFILE_CACHEABLE (no per-reader identity). Seam: client `GET /api/auth/session` vs profile username (`usernameEqualsIgnoreCase`).

Validation: `bun run lint` + `typecheck` + `test:teams` + native `bun run test:e2e`.

Review mode: normal.

## Refactoring Review

Status: proceed

Review mode: normal

Existing-code scope: ProfileView layout, ProfilePageClient, `u/[username]/page.tsx`, `leaveTeam`/`leaveGroup`.

Behavior that must remain unchanged: listed above.

Structural friction: none worth extracting first.

Decision: no refactor first. Add page-only membership loader + optional props on ProfileView. Do not split ProfileView. Do not add team/group to `getPublicProfileResponse`.

Safety net: feature file + Playwright + test:teams.

Deferred refactors: none.

## Ponytail

Reuse T4 leave APIs and TeamsClient error/toast pattern. One membership query helper. `t()` keys for new copy. No new libraries.

## UI contract

- No membership → do not render the block.
- Team name always when member; Group name only when in a group.
- Leave buttons only when client session username matches profile username.
- Admin leave Team: toast/alert the 409 message (transfer or disband). Stay on the team.
- After successful leave: refresh profile data (router.refresh) so the block updates.

## Release Readiness Review

Status: ready

Production path and affected users / systems: `/u/[username]` Team/Group block; owner-only `DELETE /api/teams/:teamId/members/me` and `DELETE /api/teams/:teamId/groups/:groupId/members/me`; invitees `GET /api/me/invitations`. Visitors see names only. Signed-in owners leave. Cached `/u/*` (`PROFILE_CACHEABLE`, `user:<username>` tag).

Failure modes and safeguards: admin leave Team → 409 and stay member; missing `DATABASE_URL` → no block; leave Team deletes `group_members` then `team_members` in one txn (INV-8); after commit, `bumpUserProfile(userId)` looks up username server-side and `revalidateTag('user:…')`; `listMyInvitations` filters `expiresAt > now()` (SQL), not a JS `Date` in raw sql; E2E cleanup refuses non-loopback `DATABASE_URL`.

Capacity / backpressure / limits: one extra membership join on the profile page path; one client `GET /api/auth/session` after paint. No new queues, cron, or public JSON contract.

Observability / alerts / runbook: existing `teamErrorResponse` `{ error }` + toast (409 copy as-is). `revalidateTag` failures log `Failed to revalidate ${tag}`.

Rollout / migration / rollback / cleanup: no migration. Rollout is the feature branch. Rollback = revert page/client/view + `bumpUserProfile`. Fixture users are `t5e2e-*` and cascade on delete.

Required validation and result: `bun run lint` (0 errors; 2 pre-existing warnings outside T8), `bun run typecheck` pass, native `bun run test:teams` pass (includes pending-invite list), native `bun run test:e2e` 9 passed including `GET /api/me/invitations` 200. `rtk`: skipped-for-report.

Optional checks, accountable owner acceptance, and residual risk: `bun run build` / `cf:build` / `test:migrations` not-required (no schema change; T8 did not ask). Dirty worktree cannot prove PR head. 409 admin copy stays English until T11. If username lookup in `bumpUserProfile` finds no row, the 60s profile cache is not flushed — user is already gone.

Production risk summary: leave mutations and invitation list are existing T4 APIs; T8 adds page-only reads, owner buttons, and profile-tag invalidation.

Failure modes covered: admin 409, INV-8, no-membership hide, non-owner hide, invitations Date-in-sql 500, cache stale after leave.

Observability and alerting notes: toast + existing JSON error; no new metrics.

Rollout and rollback path: revert the T8 files; no data backfill.

Validation performed and skipped checks: lint, typecheck, test:teams, test:e2e passed; build/migrations/cf not-required.

Residual risk: dirty evidence; T11 copy; 60s cache if username lookup misses.

## After implement

trellis-check, then `/review` (caveman-review + reviewer). Fix every finding. Then Phase 3.4. No commit from implement. No merge to main.
