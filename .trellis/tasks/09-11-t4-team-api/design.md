# T4 Team/Group API design

Parent design (`09-10-teamboard-teams-auth/design.md` §4–§6) and docs PRD §4–§5 / §9.2 are the source of truth. This file only records T4-local boundaries.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | on-demand; parent confirmed | — | not-required |
| book-ddia-data-design | writes + invitations + INV | before implement | passed |
| book-legacy-change-safety | web has no unit tests; verify-email/worker/email reused | before first existing-file edit | passed |
| book-refactoring-pass | verify-email/worker/email/send.ts | before those edits | passed |
| book-release-readiness | API + email + cron expiry | after validation | passed |

grill-with-docs: skipped. See repo-root `grill.log`.

## Legacy Change Safety Review

Status: characterized
Review mode: normal
Behavior to change: none of register/login/session/device-flow JSON. After a successful verify-email consume, T4 adds an independent invite-backfill txn that must not fail the 200. Cron currently runs three independent `waitUntil` tasks (`refreshAllSocialLinks`, `deleteExpiredEmailTokens`, `expireInvitations` in `web/worker.ts`). Email `send.ts` currently has two templates (verify, reset); T4 adds invite mail using the same `sendInBackground` path (never throws, missing secrets skip).
Behavior to preserve: `getSessionFromRequest` / `hasAllowedOrigin` signatures; `{ error: string }` JSON; register still returns `{ ok: true }` with no invite backfill; verify-email still returns `{ ok: true }` even if backfill fails; session cookie unchanged.
Current reproduction evidence: read `verify-email/route.ts` after consume; `worker.ts` scheduled independent `waitUntil`; `email/send.ts` invite template. Schema net `DATABASE_URL=... bun run test:teams` 2026-09-14: INV-1 23505, INV-3 23505, invitation CHECK 23514, all ok.
Safety net: `web/scripts/check-teams-invariants.ts` + `bun run test:teams` (exists and ran green on 0025 schema). Domain permission cases extend this runner — not a vitest framework.
Hidden dependencies / seam: none. Backfill is a new exported function called after successful verify-email, not after register.
Validation plan: `bun run test:teams`; lint; typecheck.

## Refactoring Review

Status: proceed
Review mode: normal
Existing-code scope: append-only after verify-email consume, three independent cron `waitUntil` tasks, one new `sendEmail` export. New domain lives in `lib/teams/`.
Behavior that must remain unchanged: CSRF Origin gate, PBKDF2 register, cron still logs social-link counts even if invitation expiry is a separate task.
Structural friction: none — routes already thin try/catch + lib.
Decision and smallest safe step: no refactor needed. Do not extract a generic API wrapper.
Safety net and validation: `test:teams` + typecheck.
Deferred refactors: none.

## 1. Boundary

In:

- `web/src/lib/teams/` domain functions
- Route handlers under `web/src/app/api/teams/`, `/api/me/invitations`, `/api/invitations/`, `/api/users/search`
- Invitation email send via existing `lib/email/send.ts` (extend templates)
- Expire pending invitations: call from existing daily cron in `worker.ts` (tables exist as of 0025)

Out: `/teams` UI (T5), leaderboard columns (T6), teamboard page (T7), profile leave buttons (T8), i18n (T10/T11).

## 2. Auth

Mutations: `getSessionFromRequest(request, { allowAuthorizationHeader: false })`. Null session → 401. Forged/missing Origin already collapsed into null; also 401.

Reads that need a viewer: cookie session via `getSession()` (no CSRF). Private team non-member → **404** not 403 (`canViewTeam`).

`GET /api/users/search`: session required; caller must be admin/subadmin of **any** team else 403.

## 3. Visibility

Single helper `canViewTeam(team, isMember)`:

- member of team → true regardless of visibility or status (INV-10)
- else `visibility === 'public' && status === 'active'`
- no `created_by` exception; disbanded GET is 404 unless the viewer is still a member (disband clears memberships, so delete uses `created_by` separately)

## 4. Writes / transactions

One DB transaction per mutation. INV-2 checked in app before inserting `group_members`. INV-8: leave team deletes `group_members` for that user in the same txn.

Concurrency (parent design §6):

- subadmin cap: conditional update / count inside txn; unique not enough
- accept invite: `UPDATE ... WHERE status='pending'` rowcount 0 → already handled
- duplicate pending invites: unique indexes; catch 23505 → "already pending"
- verify-email backfill: T4 adds `linkPendingInvitationsForEmail(userId, email)` and invokes it from verify-email **after** a successful consume (independent txn). Failures log only. Register does not backfill unverified addresses.

## 5. Errors

`{ error: string }` plus `{ error, details: string[] }` for batch invite item failures. Never leak whether a private team exists (404). Never return emails from search.

## 6. Tests

No vitest. Add `web/scripts/check-teams-invariants.ts` (same postgres.js + ROLLBACK/SAVEPOINT pattern as `check-migrations.ts`). Covers permission matrix cells that are server-side, disband/delete preconditions, subadmin cap, invite dedupe/expiry, leave-team cascades leave-group.

## DDIA Data Design Review

Status: confirmed  
Data owner and source of truth: Postgres team tables; `created_by` for delete.  
Write / read / async / failure paths: route → domain fn → txn; invite email fire-and-forget like auth mail; cron expires pending independently of social-link refresh.  
Consistency: single-DB strong.  
Idempotency: pending uniques; accept conditional update; backfill ordered superseded-then-fill.  
Schema: no new migration.  
Observability: `console.error` in routes and backfill; cron logs per `waitUntil` task.  
Required tests: `check-teams-invariants.ts` + typecheck + lint.

## Release Readiness Review

Status: ready

Production path and affected users / systems: Worker APIs under `/api/teams`, `/api/me/invitations`, `/api/invitations`, `/api/users/search`; verify-email now backfills pending invitations after consume; daily cron (`20 3 * * *`) adds independent `expireInvitations`. Callers are signed-in web clients (T5+) and email invitees. No CLI contract change. Top-level wrangler config is production (`tokens-staging` → tokens.ci).

Failure modes and safeguards:
- Invite mail missing `RESEND_API_KEY`/`EMAIL_FROM`: log and skip; mutation still succeeds (same as auth mail).
- verify-email backfill failure: log only; consume still returns `{ ok: true }`.
- Social-link cron failure must not skip invitation expiry: three independent `waitUntil` tasks.
- Duplicate pending invites: partial uniques + `23505` → skipped item.
- Accept races: `UPDATE … WHERE status='pending' AND expires_at > now()`; membership unique → 409 leave-first.
- Disband vs accept: `FOR UPDATE` on team + memberships; accept requires `teams.status = 'active'`.
- Email-targeted accept without `email_verified_at`: 403; username-only invites do not persist `invited_email`.
- Private team existence: 404, not 403.

Capacity / backpressure / limits: `searchUsers` caps at 10 (email exact match 1); subadmin cap 2 with conditional UPDATE; no new `AUTH_RATE_LIMITER` on team routes (session cookie required). Invite batch has no hard max — residual: a malicious admin can enqueue many Resend calls; same fire-and-forget pattern as existing auth mail.

Observability / alerts / runbook: cron logs each task's count or error; invite/backfill failures `console.error`. No new dashboard. Operator rollback is Worker redeploy of previous bundle; schema `0025` stays.

Rollout / migration / rollback / cleanup: no new migration. Feature-branch commit `253c1e99`. Not `cf:deploy`.

Required validation and result:
- `bun run lint` (web/): 0 errors, 2 pre-existing warnings (`docs/page.tsx` no-img-element, `worker.ts` anonymous default export)
- `bun run typecheck` (web/): pass
- `DATABASE_URL=postgresql://tokens:tokens@127.0.0.1:5433/tokens bun run test:teams`: pass (native, not rtk)
- TrellisCheckT4-2: pass
- `bun run build` (web/, native): pass
- `bun run cf:build`: pass — Worker saved in `.open-next/worker.js`
- `opennextjs-cloudflare preview` on `http://127.0.0.1:8787`: GET `/api/teams` → 200 `{"teams":[]}`; GET `/api/users/search?q=t4` → 401; GET `/api/me/invitations` → 401

Optional checks, accountable owner acceptance, and residual risk: `cf:deploy` not run (production). Local preview used Hyperdrive `localConnectionString` against OrbStack Postgres. Invite batch still unbounded.
