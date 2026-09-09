# Implement: Fill Project Development Guidelines

## Goal

Replace Trellis template placeholders in `.trellis/spec/` with source-backed conventions. Document what the code does now. English. No product source changes.

## Architecture (do not invent a backend/frontend split for the CLI)

| Reality | Path | Trellis package mapping |
|---|---|---|
| Rust library | `cli/tokens-core/` | `tokens-cli` backend |
| Rust binary | `cli/tokens-cli/` | `tokens-cli` backend |
| npm launcher | `packages/cli/` | `tokens-cli` frontend (not a UI) |
| Platform binary shells | `packages/cli-*` | keep package dirs; strip fake frontend templates |
| Web app | `web/` | `web` frontend (includes Drizzle/API; there is no separate web backend package) |

`cli/` is not a Trellis package (init mapped npm workspaces only). Do not add a new Trellis package or edit `.trellis/config.yaml`.

## Files to write / reshape

### `tokens-cli` backend (Rust)

Keep `index.md`. Rewrite:

- `directory-structure.md` — `tokens-core` vs `tokens-cli`; `sessions/`, `pricing/`, `scanner.rs`, `aggregator.rs`; clap lives in `main.rs`
- `error-handling.md` — CLI `anyhow::Result`; core public `Result<T, String>`; fail-soft settings; no new `thiserror` enums
- `logging-guidelines.md` — `TOKENS_LOG` + tracing to stderr; `colored` user output; never log JSON to stdout
- `quality-guidelines.md` — `#![deny(clippy::all)]` on core; clap/`ClientFilter` stays out of core; `canonical_model_id` vs grouping aliases; submit JSON camelCase; keep Cursor tokenless carve-out in sync with `web/src/lib/validation/submission.ts` (comment in `main.rs` still says `packages/frontend/...` — document the real path)

Delete `database-guidelines.md`. rusqlite reads client session DBs; it is not an app ORM.

Add:

- `session-parsers.md` — one parser per client under `cli/tokens-core/src/sessions/`; registry `clients.rs` `ClientId` / `ClientDef`; privacy: totals only
- `submit-pipeline.md` — scan → aggregate → `run_submit_command`; `serve` spawns child `tokens submit`; `--dry-run`; import does not upload

### `tokens-cli` frontend (npm launcher, not React)

Rewrite all files. Delete hook/component/state templates that describe UI.

Keep/replace with:

- `index.md` — this layer is `packages/cli` dispatcher only
- `directory-structure.md` — `bin.js`, `src/index.ts`, `dist/`, optionalDependencies
- `binary-resolution.md` — platform/libc detection, search paths, `TOKENS_LIBC`, anti-recursion `isSelfReference`, cargo `cli/target/` fallback
- `quality-guidelines.md` — no business logic in the launcher; version lockstep via `scripts/set-version.sh`

Delete: `component-guidelines.md`, `hook-guidelines.md`, `state-management.md`, `type-safety.md`.

Add `packaging.md` here (shared by all eight platform packages):

- eight `tokens-cli-<os>-<arch>` packages: git has `package.json` only; CI stages `bin/`
- `os`/`cpu`/`libc`, `files: ["bin"]`, `main` = `bin/tokens` or `bin/tokens.exe`
- publish order: platform packages then launcher
- `scripts/set-version.sh` + `.github/workflows/publish-cli.yml`
- `install.sh` is a GitHub tarball path, not npm

### Platform packages (`tokens-cli-darwin-arm64` … `tokens-cli-win32-x64-msvc`)

Each `frontend/` today is a fake UI template.

- Replace `index.md` with a short packaging note: this package is a native binary npm shell; do not add JS/React; follow `.trellis/spec/tokens-cli/frontend/packaging.md`
- Delete: `component-guidelines.md`, `directory-structure.md`, `hook-guidelines.md`, `quality-guidelines.md`, `state-management.md`, `type-safety.md`

### `web` frontend

Fill existing files. Add missing ones. Indexes must list the final file set.

- `index.md` — Next 16 App Router, OpenNext/CF Workers, shadcn v4 `@base-ui`, Tailwind v4, Drizzle/Hyperdrive; verification commands
- `directory-structure.md` — `app/`, `components/`, `lib/` (no `src/hooks/`)
- `component-guidelines.md` — shadcn in `components/ui/`; feature folders; `CONTAINER`; `tw()` only as styled-components leftover
- `hook-guidelines.md` — hooks live in `lib/use*.ts`; `useSettings`
- `state-management.md` — RSC `initialData`, URL query, localStorage+cookie, local `useState`
- `type-safety.md` — `lib/types.ts`, Zod submission, const-array unions, Drizzle `$inferSelect`
- `quality-guidelines.md` — no web unit tests; lint/typecheck/build/`test:migrations`; upstream policy: take data capability, never merge upstream UI
- Add `data-fetching.md` — RSC calls lib, not own `/api`; `unstable_cache` tags; client fetch only for mutations
- Add `error-handling.md` — `error.tsx`, API JSON errors, Alert/toast, missing `DATABASE_URL` fallback
- Add `database-guidelines.md` — `lib/db/schema.ts`, Hyperdrive vs `DATABASE_URL`, per-request WeakMap, additive migrations, hand-review drops
- Add `cloudflare-deployment.md` — wrangler, open-next, `worker.ts`, R2/DO cache, cron

## Proof sources (must cite real paths)

- `docs/upstream_policy.md`
- `cli/Cargo.toml`, `cli/tokens-core/src/lib.rs`, `cli/tokens-core/src/clients.rs`, `cli/tokens-cli/src/main.rs`
- `packages/cli/src/index.ts`, `packages/cli/package.json`, `scripts/set-version.sh`
- `packages/cli-darwin-arm64/package.json` (and siblings)
- `web/package.json`, `web/src/lib/db/index.ts`, `web/src/lib/types.ts`, `web/src/lib/validation/submission.ts`, `web/src/components/ui/button.tsx`, `web/src/lib/useSettings.ts`, `web/src/lib/publicProfileData.ts`

## Rules

- English. No "To be filled", "TODO: fill", "placeholder", or HTML comment questionnaires left in `.trellis/spec/`.
- `index.md` Pre-Development Checklist / Quality Check must point at the remaining files.
- Do not modify `cli/`, `web/`, or `packages/` product code.
- Do not git commit / push / merge.
- Skip formatters, linters, and test suites (markdown-only).
- Leave `.trellis/spec/guides/` unless a sentence is factually wrong for this repo.

## Acceptance

- [ ] No template placeholders remain under `.trellis/spec/`
- [ ] Platform package specs are packaging notes, not UI guides
- [ ] `tokens-cli` frontend describes the npm launcher, not React
- [ ] `tokens-cli` backend describes Rust core+binary, not a server ORM
- [ ] Web specs include data fetching, errors, DB, Cloudflare
- [ ] Claims cite real files
- [ ] Update checklist in `prd.md` when done
