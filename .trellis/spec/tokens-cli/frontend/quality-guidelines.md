# Quality Guidelines: npm Launcher

> Rules for `packages/cli/`. The launcher is the smallest layer in the repo
> and stays that way.

---

## Rules

1. **No business logic.** The dispatcher detects the platform, resolves a
   binary path, and `spawnSync`s it (`packages/cli/src/index.ts:235-236`). If
   you are adding parsing, config, network calls, or formatting here, the code
   belongs in the Rust workspace (`.trellis/spec/tokens-cli/backend/`).
2. **Zero runtime dependencies.** `dependencies` stays `{}`
   (`packages/cli/package.json:30`). Only `node:*` builtins — the dispatcher
   must run on whatever Node the user already has.
3. **Version lockstep via `scripts/set-version.sh` only.** Never hand-edit a
   `version` field in `packages/cli/package.json`, a platform package, or
   `cli/Cargo.toml`. The launcher pins its eight `optionalDependencies` by
   exact version; a hand edit that misses one publishes an uninstallable
   release (see [Packaging](./packaging.md)).
4. **Preserve the safety invariants in `index.ts`** when editing: the
   sibling-platform-package-first search order, and the realpath-based
   `isSelfReference` anti-recursion guard (see
   [Binary Resolution](./binary-resolution.md)).
5. **Errors are plain and actionable.** The one failure path (no binary found)
   prints which platform package was expected and exits non-zero. Match that
   style; do not add a logging framework.
6. **TypeScript compiled by plain `tsc`.** No bundler, no build plugins;
   `bun run build` in `packages/cli` is the whole build. `dist/` is gitignored
   output, regenerated on publish via `prepublishOnly`.

## Verification

- `bun run --cwd packages/cli build` passes.
- Smoke the dispatcher against a local build:
  `cargo build --manifest-path cli/Cargo.toml -p tokens-cli` then
  `bun packages/cli/src/index.ts --version` (or `scripts/cli.sh`).
- CI (`.github/workflows/ci.yml`) runs a `--version` smoke on Linux, macOS,
  and Windows — that is the dispatcher's test coverage.
