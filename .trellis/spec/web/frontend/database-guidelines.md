# Database Guidelines



> Drizzle ORM over a self-hosted Postgres, reached through `DATABASE_URL` in
> every environment. Schema, migrations, and helpers live in
> `web/src/lib/db/`.


---

## Connection lifecycle (`web/src/lib/db/index.ts`)

- The connection string always comes from `DATABASE_URL`
  (`getConnectionString`) — the Workers/Hyperdrive layer was removed in the
  self-host cutover.
- TLS: `require` in production by default, off for a plain local Postgres;
  `DATABASE_SSL` toggles explicitly (`resolveSsl`).
- The client is created via Drizzle's config-based API — passing a `postgres`
  `Sql` instance directly breaks types in the monorepo (duplicate `postgres`
  package copies with incompatible branded types).
- **One process-wide pool.** The server is a long-lived Node process, so
  `getDb()` keeps a single client on `globalThis` (HMR-safe in dev). Default
  `max=5`, overridable with `DATABASE_POOL_MAX` (clamped 1..5). The exported
  `db` is a lazy `Proxy` over `getDb()`. Never cache a `DbClient` in module
  scope beyond that singleton.


## Schema (`web/src/lib/db/schema.ts`)
- `pgTable` definitions with section comments, explicit indexes, and `jsonb`
  for breakdowns/social links (users, sessions, submissions, dailyBreakdown,
  …).
- Row types are inferred (`$inferSelect`); queries import tables and helpers
  via `@/lib/db`.
- Username matching is case-insensitive through `usernameEqualsIgnoreCase` /
  `getSingleUsernameMatch` with normalized cache keys
  (`lib/db/usernameLookup.ts`) — usernames are unique ignoring case, and
  ambiguous matches are an error (409), not a guess.
- Email matching is case-insensitive via partial unique index
  `users_email_lower_unique` (`lower(email) WHERE email IS NOT NULL`).
  `github_id` is nullable (UNIQUE kept; several NULLs are allowed). Password
  auth columns: `password_hash`, `email_verified_at`. Token table:
  `email_verification_tokens` (hash only). See [Authentication](./auth.md).
  Before applying `0024` to a database that already has emails, check
  `SELECT lower(email), count(*) FROM users WHERE email IS NOT NULL GROUP BY 1 HAVING count(*) > 1`.
  Teams/groups from `0025`: `teams`, `team_members`, `groups`,
  `group_members`, `team_invitations`. `0026` adds nullable
  `team_invitations.group_id` FK `groups(id)` ON DELETE SET NULL (invite
  auto-assign). `teams.visibility` defaults to `private`. INV-1 / INV-3 are
  UNIQUE on `user_id`. Pending invitation uniques are partial and mutually
  exclusive (`invited_user_id IS NOT NULL` vs `IS NULL`). Rollback is
  `DROP TABLE` for 0025; `DROP COLUMN group_id` for 0026. Domain unions live
  in `lib/teams/types.ts`; row types are `$inferSelect` on the tables.

## Migrations

- SQL migrations in `src/lib/db/migrations/` with `meta/_journal.json`;
  generated with `bun run db:generate` (drizzle-kit), applied by
  `bun run test:migrations` (`drizzle-kit migrate` +
  `scripts/check-migrations.ts` post-checks).
- **Additive by convention** (`docs/upstream_policy.md`): a migration that
  drops or rewrites data needs a plan for the rows already in the table before
  merge.
- **Hand-review every migration, including upstream ones.** A migration that
  is correct upstream can be wrong here because the data shape differs — on
  2026-07-24 a sync-related change double-counted usage cross-device in
  production and the repair left four backup tables behind. Read what a
  migration does to existing rows, not just to the schema.
- **Journal tail snapshot must exist.** `check-migrations.ts` allows historical `meta/*_snapshot.json` gaps, but the newest snapshot idx must equal `_journal.json` tail. SQL merged without its snapshot (as with 0022/0023) makes the next `db:generate` re-emit already-applied DDL. Reconstruct a missing tail snapshot from the previous snapshot plus the SQL delta. **Do not** run `bun run db:generate` in the live migrations directory to "fill" them — that diffs `schema.ts` against the stale snapshot and emits a new migration.
- **Checker required-tables must track drops and adds.** After a drop
  migration, remove those tables from `scripts/check-migrations.ts`. After an
  add (`email_verification_tokens` in `0024`, teams/groups tables in `0025`,
  `team_invitations.group_id` in `0026`), add the table/columns/indexes/FKs
  to the same checker. Leaving the list stale makes `test:migrations` fail
  on a correct schema.

## Query conventions

- Import `db`, tables, and operators (`eq`, `and`, `sql`) from
  `@/lib/db` / `drizzle-orm`; use `sql<>` typed aggregates for computed
  columns.
- Independent queries in one loader go in a single `Promise.all`
  (`publicProfileData.ts`).
- Cacheable reads are wrapped in `unstable_cache` with tags — see
  [Data Fetching](./data-fetching.md).

## Scenario: 0025 teams and groups

### 1. Scope / Trigger

- Trigger: additive schema/migration (`0025_add_teams_and_groups.sql`).
- In: five new tables + Drizzle defs + `lib/teams/types.ts` unions + checker.
- Out: Team/Group APIs, UI, invitation cron sweep (T4+).

### 2. Signatures

| Table | PK | Notable columns |
|-------|----|-----------------|
| `teams` | `id uuid` | `name varchar(100)`, `slug varchar(100)` UNIQUE, `visibility varchar(10)` DEFAULT `'private'`, `status varchar(10)` DEFAULT `'active'`, `created_by uuid` FK users CASCADE, `disbanded_at timestamptz` |
| `team_members` | `id uuid` | `team_id` FK teams CASCADE, `user_id` FK users CASCADE UNIQUE (INV-1), `role varchar(10)` DEFAULT `'member'`, `invited_by` FK users SET NULL |
| `groups` | `id uuid` | `team_id` FK teams CASCADE, `name varchar(100)`, UNIQUE(`team_id`,`name`), `status` DEFAULT `'active'`, `created_by` FK users CASCADE |
| `group_members` | `id uuid` | `group_id` FK groups CASCADE, `user_id` FK users CASCADE UNIQUE (INV-3) |
| `team_invitations` | `id uuid` | `team_id` FK teams CASCADE, `invited_email varchar(255)`, `invited_username varchar(39)`, `invited_user_id` FK users CASCADE, `invited_by` FK users CASCADE NOT NULL, `status` DEFAULT `'pending'`, `token_hash varchar(64)` UNIQUE, `expires_at timestamptz` NOT NULL |

Indexes: `teams_public_active_idx` on `name` WHERE `visibility='public' AND status='active'`; `team_invitations_pending_user_unique` (`team_id`,`invited_user_id`) WHERE pending AND `invited_user_id IS NOT NULL`; `team_invitations_pending_email_unique` (`team_id`, `lower(invited_email)`) WHERE pending AND `invited_user_id IS NULL`.

CHECK `team_invitations_target_present`: `invited_user_id IS NOT NULL OR invited_email IS NOT NULL`.

Row types: `$inferSelect` / `$inferInsert` on the tables. Domain unions: `lib/teams/types.ts`.

### 3. Contracts

- DB default `visibility='private'` (fail-safe). Form-layer required choice is T5.
- INV-1 / INV-3: one user, one team, one group — UNIQUE on `user_id`, not `(team_id, user_id)`.
- Pending invitation uniques are mutually exclusive; both firing for the same person is a no-op unique.
- `teams.created_by` survives disband; delete auth reads this column, not `team_members.role`.
- INV-2 (group member must already be a team member) is application-layer (T4), not a DB constraint.
- Operational rollback: `DROP TABLE` the five tables. Forward SQL is CREATE only — do not put DROP in `0025`.
- Env: none new. Apply with existing `DATABASE_URL`.


### 4. Validation & Error Matrix

| Condition | SQLSTATE / checker |
|-----------|-------------------|
| Second `team_members` row for same `user_id` | `23505` (INV-1) |
| Invitation with both `invited_user_id` and `invited_email` NULL | `23514` (CHECK) |
| Two pending invites same team + same `invited_user_id` | `23505` (`pending_user_unique`) |
| Two pending email-only invites same team + same `lower(email)` | `23505` (`pending_email_unique`) |
| Journal tail snapshot missing | `test:migrations` fails before apply |
| Checker required-tables omit the five names | `test:migrations` fails on a correct schema |

Negative INSERTs inside the checker transaction MUST use `SAVEPOINT` / `ROLLBACK TO SAVEPOINT`; a failed statement otherwise aborts the outer txn.

### 5. Good / Base / Bad Cases

- Good: `INSERT INTO teams (name, slug, created_by)` with no visibility → `visibility='private'`, `status='active'`.
- Base: one team + admin `team_members` + one group + `group_members` + invitation with email only (user_id NULL).
- Bad: two `team_members` for the same user; invitation with neither target; `DROP TABLE` inside the forward migration.

### 6. Tests Required

`bun run test:migrations` (native, not `rtk` cache) must assert:

- journal idx 25 / tag `0025_add_teams_and_groups` / `0025_snapshot.json` tail
- five tables exist
- `teams.visibility` NOT NULL default contains `private`
- partial `teams_public_active_idx` and the two pending uniques
- CHECK `team_invitations_target_present`
- UNIQUE `team_members_user_unique` and `group_members_user_unique`
- representative inserts + INV-1 `23505` and CHECK `23514` via savepoints

`bun run typecheck` after schema exports. `bun run build` is out of T3 complete criteria; record skip if not run.

### 7. Wrong vs Correct

#### Wrong
Put `DROP TABLE` in `0025` as “rollback SQL”. Run `bun run db:generate` against a stale snapshot to “fill” 0025. Encode INV-2 as a DB FK. Sweep `team_invitations` from `worker.ts` cron in T3 (file later removed in the self-host cutover).

#### Correct
Forward migration is CREATE / INDEX / FK / CHECK only. Commit `0025_snapshot.json` with journal idx 25. INV-2 stays in T4 domain code. Cron sweep waits for T4; tables exist as of 0025. The sweep now lives in `POST /api/cron/refresh-social-links` (see [Self-host Deployment](./self-host-deployment.md)).


## Scenario: 0026 invitation auto-assign group_id

### 1. Scope / Trigger

- Trigger: additive schema/migration (`0026_invitation_group_id.sql`) plus
  invite/accept applying optional auto-assign.
- In: nullable `team_invitations.group_id`, Drizzle column, invite persist,
  accept INV-2 insert, checker column + FK.
- Out: Teamboard (T7), Profile membership (T8).

### 2. Signatures

| Object | Contract |
|--------|----------|
| Column | `team_invitations.group_id uuid NULL REFERENCES groups(id) ON DELETE SET NULL` |
| Invite input | optional `groupId?: string` on each `InviteInput` item |
| Accept | after `team_members` insert in the same txn, if the locked group row is still `active` on that team, insert `group_members` |

### 3. Contracts

- Omit / `null` / `""` `groupId` → store NULL (no auto-assign).
- Present non-string `groupId`, non-UUID, wrong team, or non-active group → `TeamError` 400 `"Group is not available for auto-assign"`.
- Accept: `SELECT … FOR UPDATE` the group by id+team (not filtered by status). Insert `group_members` only when `status='active'`. Disbanded or deleted (`ON DELETE SET NULL`) → team member only.
- INV-2 stays application-layer. Do not add a DB FK from `group_members` to `team_members`.
- Rollback: `DROP COLUMN group_id`. No backfill; existing pending invites stay NULL.

### 4. Validation & Error Matrix

| Condition | Result |
|-----------|--------|
| Valid active group on this team | persist `group_id`; accept inserts `group_members` |
| Non-string / malformed / other-team / disbanded group at invite | 400 |
| Group disbanded after invite, before accept | accept 200, no `group_members` row |
| Group row deleted | `group_id` SET NULL; accept team-only |
| Missing FK `ON DELETE SET NULL` | `test:migrations` fails (`confdeltype !== 'n'`) |

### 5. Good / Base / Bad Cases

- Good: invite with this team's active group → accept → one `group_members` row.
- Base: invite with no `groupId` → accept → team member only.
- Bad: `groupId: 1` (non-string) silently stored as NULL; accept inserting into a group without `FOR UPDATE` while `disbandGroup` runs.

### 6. Tests Required

`bun run test:migrations` must assert nullable `team_invitations.group_id` and FK `ON DELETE SET NULL`. Journal tail snapshot is `0026_snapshot.json`.

`bun run test:teams` must assert: invite+accept with valid group → `group_members`; bad/non-string groupId → 400; accept after disband → team member only.

### 7. Wrong vs Correct

#### Wrong
Treat a supplied non-string `groupId` as “no group”. `SELECT` the auto-assign group without `FOR UPDATE` (accept can insert after `disbandGroup` deleted members). Encode INV-2 as a DB FK. Run `bun run db:generate` against a stale 0025 snapshot to “fill” 0026.

#### Correct
Reject a present non-string `groupId` with 400. Lock the group row on accept (and on `disbandGroup`) so auto-assign cannot land on a disbanded group. Hand-write `0026` SQL + `0026_snapshot.json`. INV-2 stays in `acceptInvitation`.
