# T6 Leaderboard Team/Group columns

Parent design and docs PRD FR-5 / §7.3–§7.4 are the source of truth.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | on-demand | — | not-required |
| book-ddia-data-design | leaderboard cache payload | before implement | passed |
| book-legacy-change-safety | existing getLeaderboard + table | before first existing-file edit | passed |
| book-refactoring-pass | same files | before those edits | passed |
| book-release-readiness | public leaderboard API + page | after validation | passed |

grill-with-docs: skipped. No new domain terms; FR-5 and D-2 already decide column contract.

## Change boundary

- Gap: leaderboard has no Team/Group columns.
- Lives in `getLeaderboard.ts` (LEFT JOIN) + `LeaderboardUser` + table/skeleton.
- Shared render: `MembershipCells.tsx` for T7 (Group-only via `includeTeam={false}`).
- Not doing: Teamboard (T7), team UI (T5), i18n (T10), visibility filtering on this page.

## Contract

`LeaderboardUser.team: { id, name, slug } | null`
`LeaderboardUser.group: { id, name } | null`

D-2: names are visible to all visitors, including private teams. JOIN filters `status='active'` only (PRD §7.4). Empty membership renders blank.

Cache tag stays `leaderboard`. T4 `bumpLeaderboard` already revalidates on membership writes.

## Legacy / refactoring

Status: proceed. Append JOIN columns and cells. Do not rewrite ranking/aggregation. Tokens/Cost headers remain format toggles.

## DDIA

Status: passed. Source of truth: team_members / group_members UNIQUE 0..1 JOIN. Cache payload grows; same tag. No new migration.

## Release Readiness Review

Status: ready

Production path: public `/leaderboard` page and `GET /api/leaderboard` JSON now include `team: {id,name,slug}|null` and `group: {id,name}|null`. Additive fields. CLI unchanged. Not `cf:deploy`.

Failure modes: JOIN is 0..1 (UNIQUE user_id); disbanded teams have no members so names go blank; empty membership renders blank cells. Search subquery must alias membership columns (`team_id` / `team_name` / `group_id` / `group_name`) or Postgres rejects duplicate `id`/`name`. Cache tag stays `leaderboard`; T4 `bumpLeaderboard` already revalidates on membership writes.

Required validation:
- `bun run lint`: 0 errors, 2 pre-existing warnings
- `bun run typecheck`: pass
- TrellisCheckT6: pass
- `bun run test:teams`: pass (includes search-subquery alias regression)
- `bun run build` / `cf:build`: pass
- post-T6 `opennextjs-cloudflare preview` `GET /api/leaderboard?search=t6sm_mu0xzfnq` → 200 with `team:{id,name,slug}` and `group:{id,name}`

Residual: `cf:deploy` not run. Smoke fixture deleted after the preview request.
