# T0 实施

**依赖**：无
**父任务**：`.trellis/tasks/09-10-teamboard-teams-auth/implement.md` 的 T0 节
**推进清单**：`docs/TODO.md`

## 步骤

1. 读 `web/scripts/check-migrations.ts` 全文，定位对 `groups` / `group_members` / `group_invites` 的断言并删除。先搜全仓确认没有第二处断言。
2. 读 `web/src/lib/db/migrations/meta/_journal.json`，确认尾部 idx=23。
3. 对照 0021 snapshot 与 0022/0023 SQL，补齐 `0022` / `0023` snapshot。在隔离副本上从上一份 snapshot 加 SQL delta 重建；**不要**在 live `web/src/lib/db/migrations` 跑 `db:generate`（会按 `schema.ts` 对 stale snapshot 发出新的 0024）。
4. 运行 `bun run test:migrations`（报告型，不用 rtk 当唯一证据）。

## 验收

- `bun run test:migrations` 退出码 0
- 脚本不再断言已删 group 表
- 最新 snapshot 编号 = journal 尾部 idx 23

## 回滚

还原 `check-migrations.ts` 与 `meta/` 新增 snapshot。

## Check notes (2026-09-11)

Gates: `bun run lint` (0 errors, 2 pre-existing warnings in docs/page.tsx and worker.ts), `bun run typecheck` (tsc --noEmit clean), `bun run test:migrations` (exit 0; pg_ctl start/stop, not brew services).

```text
Legacy Change Safety Review
Status: characterized
Behavior to change: check-migrations.ts still required groups / group_members / group_invites tables, indexes, and a representative INSERT after 0020_drop_group_tables.sql; newest snapshot lagged journal tail at 0021 vs idx 23.
Behavior to preserve: journal continuity, when-order, orphan-sql, snapshot-tail equality, remaining required tables/columns/indexes, pgcrypto, representative inserts for users/submissions/devices/daily_breakdown.
Current reproduction evidence: source at required-tables, required-indexes, and INSERT INTO "groups"; test:migrations would fail on a post-0020 schema. Reproduced by reading the checker; full migrate+check passed after the edit (exit 0).
Safety net: bun run test:migrations (drizzle-kit migrate + check-migrations.ts). No web unit tests by project policy.
Hidden dependencies / seam: none. Checker is a standalone script.
Validation plan: test:migrations on a throwaway local Postgres via pg_ctl; brew services stop so it is not a login service.
Review mode: normal
```

```text
Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: web/scripts/check-migrations.ts required-tables / required-indexes / representative-insert lists; meta/0022 and 0023 snapshots reconstructed from 0021 + SQL.
Behavior that must remain unchanged: all non-group assertions; snapshot JSON shape consumed by drizzle-kit.
Structural friction: none that blocks the deletion.
Decision and smallest safe step: no refactor needed — delete the stale group assertions; reconstruct tail snapshots; do not restructure the checker.
Safety net and validation: bun run test:migrations.
Deferred refactors: none.
```

```text
DDIA Data Design Review
Status: confirmed
Data owner and source of truth: Postgres schema via Drizzle SQL migrations; journal in meta/_journal.json; drizzle-kit generate baseline is the newest meta/*_snapshot.json.
Write / read / async / failure paths: T0 writes no production rows. Checker reads information_schema after migrate. Snapshots are metadata only.
Consistency model: strong for applied migrations; snapshot tail must equal journal tail or the next generate re-emits applied DDL.
Idempotency / ordering / retry / deduplication: journal idx 0..N-1 and strictly increasing `when`; historical snapshot gaps allowed, tail is not.
Schema / migration / backfill / rollback / replay: no new SQL, no schema.ts change, no backfill. Rollback = restore checker + delete 0022/0023 snapshots. Do not db:generate in the live migrations dir to fill gaps.
Observability and repair: test:migrations; checker prints ok/fail per assertion.
Required tests: bun run test:migrations (passed).
```

update-spec: added journal-tail snapshot and dropped-table checker bullets to `.trellis/spec/web/frontend/database-guidelines.md`. No new API/schema contract.

lessons-record: LESSON-20260911-640-migration-checker-dropped-tables (topics/validation.md).

```text
Code Readability Review
Scope: modified hand-written production code and tests
Findings: none
Ponytail conflicts resolved: none
Changes applied: none
Revalidation required: no
```
