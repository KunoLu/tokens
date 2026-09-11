# Database Guidelines

> Drizzle ORM over Postgres (Neon), reached through a Hyperdrive binding in
> production and `DATABASE_URL` locally. Schema, migrations, and helpers live
> in `web/src/lib/db/`.

---

## Connection lifecycle (`web/src/lib/db/index.ts`)

- On Workers, the connection string comes from the `HYPERDRIVE` binding; off
  Workers (local dev, scripts) from `DATABASE_URL`
  (`getConnectionString`).
- TLS: `require` against Neon, off for local Postgres; `DATABASE_SSL` toggles
  explicitly. Hyperdrive terminates TLS to the origin itself, so the driver
  must not negotiate TLS a second time on that hop (`resolveSsl`).
- The client is created via Drizzle's config-based API — passing a `postgres`
  `Sql` instance directly breaks types in the monorepo (duplicate `postgres`
  package copies with incompatible branded types).
- **Per-request clients, keyed by execution context.** Reusing one pool across
  Cloudflare requests caused production 500s; `getDb()` keeps a
  `WeakMap<object, DbClient>` keyed by the request `ctx`
  (`db/index.ts:105-139`). The exported `db` is a lazy `Proxy` over `getDb()`.
  Never cache a `DbClient` in module scope beyond the singleton/globalForDb
  hot-start mechanism already there.

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
- **Checker required-tables must track drops.** After a drop migration (e.g. `0020_drop_group_tables.sql`), remove those tables from `scripts/check-migrations.ts` required-tables, required-indexes, and representative-insert lists. Leaving them makes `test:migrations` fail on a correct schema.

## Query conventions

- Import `db`, tables, and operators (`eq`, `and`, `sql`) from
  `@/lib/db` / `drizzle-orm`; use `sql<>` typed aggregates for computed
  columns.
- Independent queries in one loader go in a single `Promise.all`
  (`publicProfileData.ts`).
- Cacheable reads are wrapped in `unstable_cache` with tags — see
  [Data Fetching](./data-fetching.md).
