# Workflow

<!-- lessons:640:start -->

## LESSON-20260914-640-archive-requires-task-name: task.py archive requires the task name

- Date: 2026-09-14
- Tags: trellis, archive, workflow, task-py
- Applicable scenarios: Before `task.py archive`; when archive fails with `required: name`
- Severity: medium
- Source: T7 `09-11-t7-teamboard-page` close-out (`python3 ./.trellis/scripts/task.py archive --no-commit` without `name`)
- Problem: `task.py archive --no-commit` exited 2 with `the following arguments are required: name`. The task stayed in `tasks/` until a second invocation passed `09-11-t7-teamboard-page`.
- Root cause: `--no-commit` only skips the git commit. It does not make `name` optional. Usage is `task.py archive [-h] [--no-commit] [--skip-branch-validation] name`.
- Fix: Re-run `python3 ./.trellis/scripts/task.py archive --no-commit 09-11-t7-teamboard-page`, then commit the archive move separately.
- Prevention: Always pass the task directory name to `archive`. Do not infer it from “current task” or omit it because `--no-commit` is present.

## LESSON-20260915-640-legacy-gate-before-edit: run legacy gate before implement-worker edits

- Date: 2026-09-15
- Tags: trellis, workflow, book-legacy-change-safety, subagent
- Applicable scenarios: Dispatching `trellis-implement` for a bug fix in existing production code; when a worker reports it already edited before `book-legacy-change-safety`
- Severity: high
- Source: T11 `09-11-t11-i18n-copy` P1 missing-key fix. Worker `FixT11P1Keys` confirmed i18n type edits landed before the mandatory legacy gate.
- Problem: Main dispatched `trellis-implement` for an existing-behavior bug fix after loading only `book-refactoring-pass`. The worker started typing `t()` / `Translate` before `book-legacy-change-safety`. That is a Trellis stage error: production behavior changed without a characterized safety net.
- Root cause: Dispatch prompt omitted the legacy gate. Main treated refactoring-pass as sufficient. Subagent default is to edit; it will not self-block on a gate that was never assigned.
- Fix: Hub-steered the running worker to halt further production edits, read `skill://book-legacy-change-safety`, emit Legacy Change Safety Review (`characterized`), record it in T11 `implement.md`, keep the early type edits (no revert), then resume remaining P1 call-site edits.
- Prevention: For existing-behavior bug fixes, put `book-legacy-change-safety` in the implement dispatch prompt and require `characterized` (or `seam-required` then seam) before the first production edit. Do not treat a later “gate done” chat as the lesson; write this topic entry.

## LESSON-20260915-640-cancelled-worker-truncated-files: do not cancel an implement worker mid-write

- Date: 2026-09-15
- Tags: trellis, workflow, subagent, truncation
- Applicable scenarios: Cancelling a `trellis-implement` worker that is writing production files; recovering uncommitted T11 work from session JSONL
- Severity: high
- Source: T11 `09-11-t11-i18n-copy`. Worker `FixT11P2Copy` was cancelled while stalled on Teamboard. `ProfileContributionGraph.tsx` shrank from ~2467 lines to 300; `ProfileEmbedDialog.tsx` from ~860 to 297. Tails contained the Read-tool footer `[Showing lines 1-300 of …]`.
- Problem: Uncommitted T11 wrapping in those two files is not source-equivalent to any recovered tree. rtk `git diff` hunks were truncated; OMP `PUT` replay vs the same HEAD copies produced graph +160/-102 (target +150/-94) and embed +88/-77 (target +69/-58). Line count 860 does not prove content.
- Root cause: Cancelling a writer mid-flight can persist a truncated read as the file. Session JSONL stores compressed reads and line-numbered edits against vanished snapshots, not a byte-identical pre-cancel blob.
- Fix: Damaged copies kept in `/tmp/t11-truncated`. Recovery trees stayed in `/tmp`; working tree not overwritten. Do not `git checkout` those paths (that would also drop T11 wrapping).
- Prevention: Do not cancel an implement worker that has started writing. If it stalls, hub-steer it to yield; copy the live files first. Never write a Read-tool truncated view back to source.



<!-- lessons:640:end -->
