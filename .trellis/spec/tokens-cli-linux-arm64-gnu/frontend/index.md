# tokens-cli-linux-arm64-gnu: Packaging Note

> This Trellis package maps to `packages/cli-linux-arm64-gnu/`, the npm shell
> that carries the prebuilt `tokens` binary for glibc Linux on arm64
> (`tokens-cli-linux-arm64-gnu` on npm). **There is no frontend here** — no
> JS/TS source, no components, no build script.

---

## What this package is

- In git: `packages/cli-linux-arm64-gnu/package.json` only. The `bin/`
  directory is staged by CI at publish time and is never committed.
- Manifest contract: `os: ["linux"]`, `cpu: ["arm64"]`, `libc: ["glibc"]`,
  `main: "bin/tokens"`, `files: ["bin"]` — npm installs this package only on
  matching machines, as an `optionalDependencies` entry of the `tokens-cli`
  launcher. The `libc` field is what keeps musl distros (Alpine, Void-musl)
  off this build.
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
