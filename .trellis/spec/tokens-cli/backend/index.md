# tokens-cli "Backend": Rust Core + Binary Guidelines

> Conventions for the Rust workspace under `cli/` — the `tokens-core` library
> crate and the `tokens-cli` binary crate. Trellis labels this layer "backend",
> but it is not a server: it is a local CLI that scans AI client session files,
> aggregates token usage, and submits totals to the web platform.

---

## Overview

The Rust workspace is declared in `cli/Cargo.toml` with two members:

| Crate | Path | Role |
|-------|------|------|
| `tokens-core` | `cli/tokens-core/` | Domain library: client registry, session parsers, scanner, aggregation, pricing |
| `tokens-cli` | `cli/tokens-cli/` | Binary crate (`[[bin]] tokens`): clap command surface, auth, per-provider integrations, settings |

The npm launcher (`packages/cli/`) and platform binary packages
(`packages/cli-*`) are documented separately under
`.trellis/spec/tokens-cli/frontend/` (Trellis's name for the second layer — it
is a native-binary dispatcher, not a UI).

---

## Guidelines Index

| Guide | Description |
|-------|-------------|
| [Directory Structure](./directory-structure.md) | `tokens-core` vs `tokens-cli` module layout and ownership |
| [Session Parsers](./session-parsers.md) | One parser per AI client under `cli/tokens-core/src/sessions/`; the `clients.rs` registry |
| [Submit Pipeline](./submit-pipeline.md) | scan → aggregate → `run_submit_command`; `serve` child processes; `--dry-run`; import boundary |
| [Error Handling](./error-handling.md) | `anyhow::Result` in the binary, `Result<T, String>` on core's public async paths, fail-soft config |
| [Logging Guidelines](./logging-guidelines.md) | `TOKENS_LOG` + tracing to stderr; `colored` user output; stdout stays JSON-clean |
| [Quality Guidelines](./quality-guidelines.md) | `#![deny(clippy::all)]`, core/binary boundary, model-id canonicalization, cross-repo invariants |

---

## Pre-Development Checklist

Before changing code under `cli/`:

1. Read [Directory Structure](./directory-structure.md) to decide whether the
   change belongs in `tokens-core` (scan/parse/aggregate/price) or
   `tokens-cli` (clap, auth, integrations, settings).
2. If touching a client parser, read [Session Parsers](./session-parsers.md)
   and the `ClientDef` registry in `cli/tokens-core/src/clients.rs`.
3. If touching submission behavior, read [Submit Pipeline](./submit-pipeline.md)
   and the server-side validator `web/src/lib/validation/submission.ts` — the
   two must agree.

## Quality Check

Before considering CLI work done:

- `cargo check --manifest-path cli/Cargo.toml --workspace --all-targets` passes
  (the repo's only automated CLI gate per `docs/upstream_policy.md`).
- Clippy is clean — `tokens-core` enforces `#![deny(clippy::all)]` at
  `cli/tokens-core/src/lib.rs:1`.
- No log output on stdout for `--json` commands (see
  [Logging Guidelines](./logging-guidelines.md)).
- New submission fields use camelCase serde and match the Zod schemas in
  `web/src/lib/validation/submission.ts` (see
  [Quality Guidelines](./quality-guidelines.md)).

---

**Language**: All documentation is written in **English**.
