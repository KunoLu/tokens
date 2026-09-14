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

<!-- lessons:640:end -->
