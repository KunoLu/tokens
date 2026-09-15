# Validation

<!-- lessons:640:start -->

## LESSON-20260911-640-migration-checker-dropped-tables: Migration checker must drop deleted tables and keep snapshot tail

- Date: 2026-09-11
- Tags: migrations, drizzle, check-migrations, snapshots
- Applicable scenarios: After a drop migration; when SQL lands without a matching `meta/*_snapshot.json`; before `bun run db:generate`
- Severity: high
- Source: T0 `09-11-t0-baseline-migrations` (`0020_drop_group_tables.sql` vs still-required `groups` tables; journal tail 23 vs snapshot 0021)
- Problem: `scripts/check-migrations.ts` still required `groups` / `group_members` / `group_invites` (tables, indexes, representative INSERT) after those tables were dropped. `0022`/`0023` SQL existed in the journal with no snapshots, so the newest snapshot lagged the journal tail.
- Root cause: The checker’s required-tables list was not updated when the drop migration landed. Snapshots were omitted from the SQL-only commits. `db:generate` cannot backfill those snapshots in the live migrations directory: it diffs `schema.ts` against the stale snapshot and emits a new migration (0024), not 0022/0023 metadata.
- Fix: Remove dropped tables from the checker lists. Reconstruct `0022_snapshot.json` / `0023_snapshot.json` from `0021_snapshot.json` plus the SQL deltas. Verify with `bun run test:migrations`.
- Prevention: After any drop migration, search `check-migrations.ts` for the old table/index names. After merging SQL, confirm newest snapshot idx equals `_journal.json` tail. Never run `bun run db:generate` in `web/src/lib/db/migrations` to “fill” missing historical snapshots.


## LESSON-20260915-640-lint-preexisting-device-effect: HEAD DeviceClient set-state-in-effect is not T11

- Date: 2026-09-15
- Tags: validation, lint, eslint, react-hooks, i18n
- Applicable scenarios: After wrapping copy with `useI18n()`, when `bun run lint` fails on a file T11 touched, or before “fixing” DeviceClient as part of an i18n wrap
- Severity: medium
- Source: T11 `09-11-t11-i18n-copy` (`web/src/app/device/DeviceClient.tsx` `react-hooks/set-state-in-effect` at `loadSession()`)
- Problem: `bun run lint` failed (1 error, 3 warnings). The error is `DeviceClient.tsx` calling `loadSession()` inside `useEffect`. A T11-caused unused `colorPalettes` import in `ProfileContributionGraph.tsx` was a separate warning.
- Root cause: T11 wrapping added `useI18n()` and dictionary keys; it did not change the session-load effect. `git show HEAD` already has `useEffect(() => { loadSession(); }, [loadSession]);`. Treating that error as T11-introduced would expand wrap remaining copy into an unrelated React effect refactor.
- Fix: Remove the unused `colorPalettes` import (T11-caused). Leave the DeviceClient effect alone. Do not claim lint/E2E passed while the pre-existing error remains.
- Prevention: Diff the failing line against HEAD before editing. If the pattern predates the wrap, record it as a remaining lint risk; do not fold it into the i18n task.

<!-- lessons:640:end -->
