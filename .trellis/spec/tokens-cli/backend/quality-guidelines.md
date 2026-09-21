# Quality Guidelines: Rust CLI

> Hard rules and cross-repo invariants for `cli/`. Automated gates are
> `cargo check --manifest-path cli/Cargo.toml --workspace --all-targets`,
> **`cargo test --manifest-path cli/Cargo.toml --workspace`**, and clippy
> (see `docs/upstream_policy.md` §4.1).

---

## Hard rules

1. **`#![deny(clippy::all)]` on core.** Set at `cli/tokens-core/src/lib.rs:1`.
   Any new warning is a build failure there; do not `#[allow]` around it.
2. **clap stays out of `tokens-core`.** The `--client` filter enum
   (`ClientFilter`) lives in `cli/tokens-cli/src/main.rs:432+` and converts to
   `tokens_core::ClientId`; core takes `ClientId` and never parses arguments.
   Keep it that way — core must remain usable without the CLI.
3. **`canonical_model_id` for anything that leaves the machine.** Submission,
   export, and persistence paths MUST use
   `tokens_core::canonical_model_id` (`cli/tokens-core/src/lib.rs:74`), which
   only does structural normalization (lowercase, strip reasoning-tier
   suffixes and date suffixes, fold vendor prefixes). Display grouping uses
   `normalize_model_for_grouping`, which additionally folds the user's
   machine-local `modelAliases`. The alias fold is presentation-only: if a
   local alias ever rewrites a persisted model id, usage history fragments
   across the user's devices (`lib.rs:62-83`).
4. **Submit JSON is camelCase.** Every struct serialized into a submission or
   status payload carries `#[serde(rename_all = "camelCase")]` — see the
   `Ts*` payload structs in `cli/tokens-cli/src/main.rs:947-1130` and
   `Settings` in `settings.rs:92-94`. New fields must match the Zod schemas in
   `web/src/lib/validation/submission.ts` exactly.
5. **Keep the Cursor tokenless carve-out in sync with the server.** Cursor's
   pre-2025-05 exports bill `premium-tool-call` rows per invocation with no
   token attribution; both sides grandfather them (cost > 0, tokens = 0).
   Client side: `is_legacy_tokenless_cursor_row` /
   `exclude_tokenless_cost_contributions` (`cli/tokens-cli/src/main.rs:1931+`).
   Server side: `CURSOR_LEGACY_TOKENLESS_MODELS` in
   `web/src/lib/validation/submission.ts:420-429`. The doc comment in
   `main.rs` still points at the pre-move path
   `packages/frontend/src/lib/validation/submission.ts` — the real path is
   `web/src/lib/validation/submission.ts`. Change one side and you change the
   other, or users with historical Cursor data are locked out of submitting.
6. **Atomic writes for local state.** Settings, caches, and credentials are
   written temp-then-rename via `fs_atomic::replace_file`
   (`cli/tokens-core/src/fs_atomic.rs`; `settings.rs:307-316`). Never
   write-then-hope.
7. **Stable date bucketing.** The bucket timezone is pinned once at startup
   (`timezone::install()`, `main.rs:325-327`) and persisted on first submit,
   so traveling does not shift usage between calendar days. Date logic goes
   through `tokens_core::bucket_tz`, never ad-hoc local-time calls.

## Tests

- Inline `#[cfg(test)]` modules remain in `tokens-core` source files.
- After the 2026-09-21 upstream CLI sync, `cli/tokens-core/tests/` also holds
  integration tests and fixtures. Do not strip those tests on cherry-pick
  unless they require TUI/report modules this fork does not carry.
- `tokens-cli` still has little binary-level coverage; do not claim a CLI
  integration suite that does not exist.

## Upstream sync (Tokscale fork policy)

Canonical rules: `docs/upstream_policy.md`. Do not fork a second policy here.

- **Always merge**: new provider/client scanners, parser fixes, submit-pipeline
  and correctness fixes. Path map for `grand`: `crates/tokscale-core` →
  `cli/tokens-core`.
- **Skip**: TUI themes, report commands, CLI display polish.
- **Never merge**: `tokscale` / `tokens.ci` branding and marketing copy.
  **Do merge** `.github/assets/client-*` scanner icons; `web` `build` copies
  them to gitignored `public/clients/`.
- Mixed core-sync (e.g. tokscale 4.17.0): `cherry-pick -n`, keep `/clients`
  catalog and version `1.0.0`, then commit with `(cherry picked from …)`.

## Forbidden patterns

- Business logic or parsing in `packages/cli` (that layer is a dispatcher; see
  `.trellis/spec/tokens-cli/frontend/`).
- Fabricating token counts from request counters — cost without tokens is
  dropped client-side (`exclude_tokenless_cost_contributions`) because the
  server rejects it.
- Logging to stdout on JSON commands (see
  [Logging Guidelines](./logging-guidelines.md)).
