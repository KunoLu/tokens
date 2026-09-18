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

## LESSON-20260918-640-e2e-locator-dollar-prefix: Exact-text e2e locators must include the CommandBlock prompt prefix

- Date: 2026-09-18
- Tags: validation, playwright, e2e, locator, docs
- Applicable scenarios: Writing or editing e2e assertions against docs-page CommandBlock commands; when an exact-text getByText times out on a command that is visibly on screen
- Severity: medium
- Source: Windows docs tab change (commit `1355308ae048b6827a1da430d55f38c8b416762a`); failure report `tests/e2e/reports/html/playwright-report-docs-page-feature_teamboard-teams-auth-2026_09_18-10_18_35.md`
- Problem: `getByText("tokens login", { exact: true })` timed out for 15s even though the Windows panel was mounted and showing the command.
- Root cause: `web/src/components/docs/CommandBlock.tsx:55-56` renders a decorative `<span>$ </span>` inside the `<code>`, so the element's full text is `$ tokens login`. Exact matching compares the whole normalized text. The failure's error-context a11y snapshot already showed `code: $ tokens login`.
- Fix: `windowsPanel.getByRole("code").filter({ hasText: "tokens login" })` — substring match scoped to the active tabpanel. Green rerun preserved as `...-10_22_33` report pair.
- Prevention: Check the component for decorative inline children (prompt prefixes, icons) before writing exact-text locators on rendered output. Prefer role-scoped `filter({ hasText })` for command rows. When a locator fails, read the error-context a11y snapshot first — it shows the text as the tree actually exposes it.

## LESSON-20260918-640-test-migrations-database-url: test:migrations needs DATABASE_URL, bare run fails with url undefined

- Date: 2026-09-18
- Tags: validation, migrations, database-url, env-prerequisite
- Applicable scenarios: Running `bun run test:migrations` locally; before claiming migration tests passed or failed; when the checker errors with `url: undefined`
- Severity: low
- Source: `09-18-scripts-ref-and-automation` 验证轮（裸跑 `bun run test:migrations` 报 `Please provide required params for Postgres driver: url: undefined`，补 env 后同轮即绿）
- Problem: `bun run test:migrations` without env exits 1 with `url: undefined` — looks like a checker regression but is only a missing env prerequisite.
- Root cause: The migrations checker opens a real Postgres connection; the spec's Quality Check lists the bare command without the connection string. Local Postgres is OrbStack compose at `127.0.0.1:5433`.
- Fix: `DATABASE_URL="postgresql://tokens:tokens@127.0.0.1:5433/tokens" bun run test:migrations`.
- Prevention: Any repo validation command that touches the DB needs `DATABASE_URL` set first; treat `url: undefined` as an env issue, not a code failure. The same prefix applies to the Playwright e2e runs in this repo.

<!-- lessons:640:end -->
