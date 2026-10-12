# T5 Team management UI

Parent implement.md T5 and `docs/demo/teamboard-demo.html` Teams page are the source of truth. BDD: `web/features/team-management.feature` and `web/features/group-management.feature`.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | on-demand | — | not-required |
| book-ddia-data-design | persist auto-assign group on invitation | before schema edit | confirmed |
| book-legacy-change-safety | Navigation NAV_LINKS | before first existing-file edit | passed |
| book-refactoring-pass | Navigation | before those edits | passed |
| book-release-readiness | /teams mutations via existing APIs + 0026 | after validation | ready |

grill-with-docs: skipped. Parent PRD confirmed 2026-09-11.

## Change boundary

- In: `/teams` page, `components/teams/`, missing shadcn primitives, invite combobox, dictionary keys (T10 `t()`), nav link, additive `team_invitations.group_id` so auto-assign survives accept.
- Out: Teamboard (T7), Profile membership (T8), remaining site copy (T11).

## UI contract

- RSC page loads session + team via `lib/teams` (no HTTP self-fetch). Client shell mutates through `/api/teams*` and `/api/users/search`.
- Create-team form **requires** visibility `public` | `private` (DB default private is fail-safe, not a form default skip).
- Invite dialog: searchable multi-select combobox. Options show username + displayName only, never email. Checkbox + count. Full email unmatched → “invite this email”. Optional “auto-assign to group on join” Select (parent T5). Candidate source `GET /api/users/search` (403 for non-admin/subadmin). Persist `groupId` on the invitation row; apply in `acceptInvitation`.
- Dangerous ops (disband/delete/remove): type the name to confirm.
- Admin/subadmin vs member: hide or disable invite and admin-only actions.
- Copy goes through `t()` keys; English dictionary matches visible strings.

## Validation

- `bun run lint` / `typecheck`
- `bun run test:migrations` + `bun run test:teams` after `0026`
- Playwright: signed-out `/teams` login-gate only unless a test account exists
- Review before archive (user rule)

## Legacy Change Safety Review

Status: characterized
Behavior to change: add `/teams` route + `components/teams/`; optional one `NAV_LINKS` entry for signed-in users.
Behavior to preserve: LocaleToggle (cookie+reload, left of theme), TokensMark `#7C3AED`, existing Leaderboard/Teamboard/Docs/Profile links, AUTH_PAGES returnTo, session fetch, UserMenu.
Current reproduction evidence: `NAV_LINKS` is leaderboard/teamboard/docs/profile only; no `/teams` page. Playwright LocaleToggle 4/4 on T10.
Safety net: existing `bun run test:e2e` LocaleToggle specs; lint/typecheck after Navigation edit.
Hidden dependencies / seam: none. Do not restructure Navigation.
Validation plan: `bun run lint`, `typecheck`, `test:e2e`.
Review mode: normal

## Refactoring Review

Status: proceed
Review mode: normal
Existing-code scope: `Navigation.tsx` `NAV_LINKS` array only.
Behavior that must remain unchanged: LocaleToggle, theme toggle, auth chrome.
Structural friction: none.
Decision and smallest safe step: append one `{ href: "/teams", key: "nav.teams", authOnly: true }` (or demo placement). No extract/split.
Safety net and validation: Playwright LocaleToggle + lint/typecheck.
Deferred refactors: none.

## Ponytail

New `/teams` RSC + `components/teams/` client shell. Add listed shadcn primitives with `bunx --bun shadcn@latest add` from the default registry (select dialog checkbox popover form label textarea; combobox if `@shadcn/combobox` exists, else Command+Popover+Checkbox). Toasts stay `react-toastify`. Skip unit tests, extra registries, T7/T8.


## DDIA Data Design Review

Status: confirmed

Data owner and source of truth: `team_invitations` row. `inviteMembers` writes optional `group_id`; `acceptInvitation` is the only consumer.

Write / read / async / failure paths: invite validates group belongs to team and `status=active`, then stores `group_id`. Accept runs later: after inserting `team_members` in the same transaction, if `group_id` still points at an active group on that team, insert `group_members` (INV-2). If the group was disbanded/deleted (`ON DELETE SET NULL`) or is no longer active, accept still succeeds as a team member only.

Consistency model: strong inside `acceptInvitation`’s transaction. No separate job.

Idempotency / ordering / retry / deduplication: accept remains once-only (pending → accepted). `group_members.user_id` UNIQUE (INV-3) makes a second insert a no-op/conflict handled like existing addGroupMember. Re-invite of a pending row does not create a second invitation (existing uniques).

Schema / migration / backfill / rollback / replay: additive `0026` `ALTER TABLE team_invitations ADD COLUMN group_id uuid NULL REFERENCES groups(id) ON DELETE SET NULL`. No backfill — existing pending invites stay NULL (no auto-assign). Rollback: `DROP COLUMN group_id`. Do not run `db:generate` against a stale snapshot; hand-write SQL + snapshot + checker column.

Observability and repair: `TeamError` 400 if invite `groupId` is missing/wrong team/not active. No new metrics.

Required tests: `test:migrations` asserts column + FK. `test:teams`: invite+accept with valid group → `group_members` row; bad groupId → 400; accept after group disbanded → team member only, no group row.


## Release Readiness Review

Status: ready

Production path and affected users / systems: signed-in `/teams` RSC (`loadTeamsPageData`) plus client mutations to existing T4 `/api/teams*` and `GET /api/users/search`. Invite auto-assign persists `team_invitations.group_id` (0026) and `acceptInvitation` inserts `group_members` after `team_members` when the locked group is still active. Signed-out visitors hit the login gate only. No new Worker cron, queue, or deploy topology.

Failure modes and safeguards: mutation errors return `{ error }` JSON and surface via `toast.error`; create-team submit is disabled until visibility is chosen; disband/delete/remove require typing the name. Present non-string / malformed / other-team / inactive `groupId` → 400. Accept `SELECT … FOR UPDATE`s the group row so auto-assign cannot land on a group `disbandGroup` is clearing. Disbanded or deleted group (`ON DELETE SET NULL`) → team member only. E2E fixture cleanup (`tests/e2e/users.ts`) parses `DATABASE_URL` and refuses to construct `postgres()` unless host (and `host` / `hostname` / `hostaddr` query overrides) is loopback `127.0.0.1` / `localhost` / `::1`; username prefix `t5e2e-` is additional, not sufficient.

Capacity / backpressure / limits: user search already caps at 10 and 403s non-admin/subadmin. Invite batch is the existing T4 loop. No new rate limit.

Observability / alerts / runbook: `console.error` in T4 `teamErrorResponse`; UI toasts English `TeamError` messages. No new metrics/alerts.

Rollout / migration / rollback / cleanup: additive `0026` (`group_id` NULL + FK SET NULL). Rollback: `DROP COLUMN group_id` plus revert `/teams` + nav link. Existing pending invites stay NULL (no auto-assign).

Required validation and result: this check ran native `cd web && bun run test:e2e` (`rtk`: `skipped-for-report`) 6/6 in 1.0m (i18n 4 + teams-gate + teams-signed-in create / new group / auto-assign Select / username invite / disband). lint / typecheck / `test:migrations` / `test:teams` were not re-run this check (no production-code edits this pass). `build` skipped per task. Named report: `tests/e2e/reports/html/playwright-report-team-management-feature_teamboard-teams-auth-2026_09_14-20_25_48.html` + same-stem Chinese `.md`.

Optional checks, accountable owner acceptance, and residual risk: signed-in Team E2E in this check (`tests/e2e/teams-signed-in.spec.ts`) traces `邀请对话框可选择自动归入分组`: creates an active group, asserts InviteDialog auto-assign Select (visible, default "No group", lists the group, selects it), then username invite (search/check/count, no email echo) and type-the-name disband. Synthetic `t5e2e-*@example.test` fixtures; afterAll cascade-delete. That is required evidence, not an owner-accepted skip. Still not covered by Playwright (no owner-acceptance claimed): invitation accept/decline (accept is not on `/teams`), non-admin search 403. Those remain API-level (`test:teams`) only. Invite widget is a search list, not an ARIA combobox. Fixture cleanup fail-closes on non-loopback `DATABASE_URL`.

Production risk summary: `/teams` can create/manage team/group via existing APIs; invite auto-assign persists on the invitation and applies on accept under INV-2.

Failure modes covered: signed-out gate, visibility required at form, name-confirm on danger ops, search 403 for non-managers, toast on mutation failure, invalid auto-assign groupId 400, accept after disband team-only, group-row lock vs disband race, E2E cleanup refused unless loopback, InviteDialog auto-assign Select when a group exists.

Observability and alerting notes: toasts only; no new alert.

Rollout and rollback path: ship `/teams` with 0026. Rollback = revert UI commit + `DROP COLUMN group_id`.

Validation performed and skipped checks: Playwright (i18n + teams-gate + teams-signed-in with auto-assign Select). Not re-run this check: lint, typecheck, test:migrations, test:teams. Skipped: build.

Residual risk: accept/decline and non-admin search 403 remain without Playwright. HTML report is local-only dirty worktree evidence and cannot prove a PR head.




## Legacy Change Safety Review (invite/accept)

Status: characterized
Behavior to change: optional `group_id` on `team_invitations`; `inviteMembers` writes it; `acceptInvitation` inserts `group_members` after `team_members` in the same txn.
Behavior to preserve: username/email invite paths; pending uniques; already-member 409; accept once-only (pending → accepted); email-verify-before-accept; expire 409; INV-1 `23505` → “Leave your current team”; `bumpLeaderboard` after accept; decline/revoke unchanged.
Current reproduction evidence: `inviteMembers` inserts invitations without `group_id`; `acceptInvitation` inserts `team_members` then returns.
Safety net: `bun run test:teams` (INV/permission checker). Add cases: invite+accept with valid group → `group_members` row; bad groupId → 400; accept after group disbanded → team member only.
Hidden dependencies / seam: INV-2 is application-layer. Do not add a DB FK from `group_members` to `team_members`.
Validation plan: `bun run test:migrations`, `bun run test:teams`, `typecheck`.
Review mode: normal

## Refactoring Review (invite/accept)

Status: proceed
Review mode: normal
Existing-code scope: `inviteMembers` insert `.values({…})`; `acceptInvitation` after `tx.insert(teamMembers)`.
Behavior that must remain unchanged: listed above.
Structural friction: none.
Decision and smallest safe step: optional `groupId` on `InviteInput`; validate active group on team at invite; persist column; after membership insert, if group still active, insert `group_members`. No extract/split of acceptInvitation.
Safety net and validation: `test:teams` + `test:migrations`.
Deferred refactors: none.

