# Lessons Index

Search this file by id, tags, or `read_when`. Details live under `topics/`.

<!-- lessons:640:start -->

| id | tags | read_when | summary | detail |
|---|---|---|---|---|
| LESSON-20260911-640-migration-checker-dropped-tables | migrations, drizzle, check-migrations, snapshots | Before db:generate, after a drop migration, or when test:migrations fails on missing tables | Checker required-tables must track drops; journal tail snapshot cannot be filled with live db:generate | topics/validation.md#lesson-20260911-640-migration-checker-dropped-tables-migration-checker-must-drop-deleted-tables-and-keep-snapshot-tail |
| LESSON-20260914-640-archive-requires-task-name | trellis, archive, workflow, task-py | Before task.py archive, or when archive fails with required: name | archive --no-commit still requires the task directory name | topics/workflow.md#lesson-20260914-640-archive-requires-task-name-taskpy-archive-requires-the-task-name |
| LESSON-20260915-640-legacy-gate-before-edit | trellis, workflow, book-legacy-change-safety, subagent | Before dispatching trellis-implement for an existing-behavior bug fix, or when a worker edited before the legacy gate | Implement workers must reach legacy characterized before the first production edit; halt, gate, then resume | topics/workflow.md#lesson-20260915-640-legacy-gate-before-edit-run-legacy-gate-before-implement-worker-edits
| LESSON-20260915-640-cancelled-worker-truncated-files | trellis, workflow, subagent, truncation | Before cancelling a writing implement worker, or when recovering truncated source from JSONL | Do not cancel mid-write; truncated Read output is not a file; JSONL replay is not source-equivalent | topics/workflow.md#lesson-20260915-640-cancelled-worker-truncated-files-do-not-cancel-an-implement-worker-mid-write |
| LESSON-20260915-640-lint-preexisting-device-effect | validation, lint, eslint, react-hooks, i18n | After wrapping copy with useI18n, or when lint fails on a file T11 touched | Diff the failing line against HEAD; DeviceClient set-state-in-effect predates T11 | topics/validation.md#lesson-20260915-640-lint-preexisting-device-effect-head-deviceclient-set-state-in-effect-is-not-t11
| LESSON-20260918-640-e2e-locator-dollar-prefix | validation, playwright, e2e, locator | Before writing exact-text assertions on CommandBlock commands, or when getByText exact times out on a visible command | CommandBlock renders a `$ ` prefix span inside code; exact-text locators miss — use role=code filter hasText, and read the error-context a11y snapshot first | topics/validation.md#lesson-20260918-640-e2e-locator-dollar-prefix-exact-text-e2e-locators-must-include-the-commandblock-prompt-prefix
| LESSON-20260918-640-test-migrations-database-url | validation, migrations, database-url | Before running test:migrations locally, or when the checker errors with url: undefined | DB-touching validation commands need DATABASE_URL first; url: undefined is an env prerequisite, not a code failure | topics/validation.md#lesson-20260918-640-test-migrations-database-url-testmigrations-needs-database_url-bare-run-fails-with-url-undefined



<!-- lessons:640:end -->
