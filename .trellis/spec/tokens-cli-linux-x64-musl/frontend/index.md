# tokens-cli-linux-x64-musl: Packaging Note

> This Trellis package maps to `packages/cli-linux-x64-musl/`, the npm shell
> that carries the prebuilt `tokens` binary for musl Linux on x86_64
> (`tokens-cli-linux-x64-musl` on npm). **There is no frontend here** — no
> JS/TS source, no components, no build script.

---

## What this package is

- In git: `packages/cli-linux-x64-musl/package.json` only. The `bin/`
  directory is staged by CI at publish time and is never committed.
- Manifest contract: `os: ["linux"]`, `cpu: ["x64"]`, `libc: ["musl"]`,
  `main: "bin/tokens"`, `files: ["bin"]` — npm installs this package only on
  matching machines, as an `optionalDependencies` entry of the `tokens-cli`
  launcher. Users whose loader detection fails can force this build with
  `TOKENS_LIBC=musl`.
- The launcher (`packages/cli/src/index.ts`) resolves and executes
  `bin/tokens` from this package at runtime.

## Rules

1. **Do not add JS, React, or any source code.** A binary carrier has no code
   to review; logic belongs in `cli/` (Rust) or `packages/cli` (dispatcher).
2. Follow the shared conventions in
   [`.trellis/spec/tokens-cli/frontend/packaging.md`](../../tokens-cli/frontend/packaging.md):
   publish order (platform packages before the launcher), CI staging of
   `bin/`, and the GitHub tarball channel.
3. Version changes only via `scripts/set-version.sh` — this package's
   `version` must always equal the launcher's.
