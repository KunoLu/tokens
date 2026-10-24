# Changelog

[中文](./CHANGELOG.zh.md)

This file records tags published by this repository. The version line is `1.0.x`.
Tags such as `v27.x` belong to upstream `missuo/tokens` and are not releases of this fork.

## [Unreleased]

## [1.0.4] - 2026-10-25

### Added

- Model-level inference speed usage from Claude's reported speed and Codex's selected speed, carried through the CLI cache, submissions, and profile data without changing token totals or pricing (#14; upstream v27.1.5).
- Profile token-composition shares and inference speed tiers in today's usage, day details, and period model tables; missing speed remains unknown (#15).
- Discovery of Paseo-managed Claude Code and Codex CLI sessions through native provider data directories, with existing parsers and file deduplication (#15).
- This repository's CLI source adds `tokens cloud submit` and `tokens cloud status` for Claude Code on the web sessions, with hashed session devices and provider-grouped profile/settings rows (#16). The binary installed by `/install.sh` comes from upstream releases, which also include `tokens cloud` starting with v27.1.4. Codex cloud is not supported. Server rules validate `cloud_` keys and provider/client correspondence, reject backfill and replacement, force device names to null, disallow renaming, and prevent adoption of legacy history or replacement of the account's MCP list.

### Fixed

- Antigravity CLI synchronization fixes from upstream v27.1.4: sync before submit/autosubmit/serve when a language server is active, a shared 60-second implicit-sync RPC budget, and partial-refresh reporting that retains cached sessions (#13). Explicit sync exits successfully on partial session refresh failures but still fails on discovery or lock errors; a Windows test-helper clippy fix is included.
- Profile uses the official OMP icon (#15).
- Device-name masking descriptions now explain that names of four characters or fewer expose only the first character; longer names retain the first and last two characters.
- Cloud setup documentation now uses the site-served `/install.sh`, which by default installs the latest GitHub Release binary from upstream `missuo/tokens` (currently v27.1.5; `tokens --version` reports 27.1.5). Token setup now points to `tokens.camp`, submissions require `TOKENS_API_URL=https://tokens.camp`, and the documentation lists the actual GitHub download domains.

### Changed

- Vercel Git auto-deploy for `main` is disabled via `git.deploymentEnabled` in `web/vercel.json` (#11). Production deploys are triggered manually.
- Local cache format advanced from 8 to 9 (#14). **Downgrading to an older CLI rebuilds the local cache from session logs.** History that exists only in the newer cache cannot be recovered from deleted source logs; back up both cache and source logs before downgrading.
- This repository's manifest versions (Cargo workspace and `packages/cli*`) and pinned script refs advanced to `1.0.4` and `v1.0.4`, respectively, for this repository's `v1.0.4` tag. This repository publishes tags only, with no GitHub Releases; these version and ref changes do not change the upstream binary version installed by `/install.sh`.

## [1.0.3] - 2026-10-17

Corresponds to commit `a752960`, the version running in Production on tokens.camp.

### Added

- First production release on tokens.camp (https://tokens.camp).
- Vercel Hobby deploy path (#9). Upstash rate limiting is used only when both the REST URL and token are set; self-host and Docker keep the in-process 10/60s limiter. Vercel Cron accepts GET on the daily maintenance route. New Vercel Hobby deploy runbook.

### Fixed

- Upstash auth limiter keys are namespaced by `VERCEL_ENV` (fallback `local`), so Preview and Production no longer share one bucket (#10).

### Changed

- README headers split by language. `Changelog`, `Deploy`, and `CLI` open the English documents. `更新日志`, `部署`, and `手册` open the Chinese documents. The English deploy checklist and CLI guide are `docs/deploy/self-host-production.en.md` and `docs/deploy/tokens-cli-usage.en.md`. (#8)
- Docs use the Vercel project name `tokens` in custom domain sections, and the Vercel free address is corrected to `tokens-blush.vercel.app`.

## [1.0.2] - 2026-10-13

### Added

- Private mode. `tokens login --private` hides an account from public rankings and prints a one-time read token. Settings can turn it off; a submit token cannot.
- Docs page section for private mode, plus a note that CLI login needs a signed-in site account and the `/device` code.
- Changelog for this fork's tags.

### Changed

- Cherry-picked upstream parser fixes onto the `1.0.x` line: Pi usage follows the served model, Claude 1-hour cache writes use the 1-hour price, and model ID prefixes can be stripped from `settings.json`.
- Usage charts no longer ship megabyte-sized SVG paths.
- READMEs no longer say the CLI is unchanged, no longer recommend `bunx` for login or submit, and count 54 catalog clients.
- Finished Teamboard checklist moved to `docs/archive/`. The HTML demo was removed.
- Script download URLs and package versions advanced to `v1.0.2`.

## [1.0.1] - 2026-09-21

### Added

- Team, Group, and Teamboard landed on the published line with this tag.
- Craft Agent scanning and `WORKBUDDY_CONFIG_DIR`.

### Fixed

- CLI core brought in line with upstream `27.1.1` by cherry-pick, without adopting the `27.x` version numbers.
- Codex no longer double-counts exec captures; the submit parser revision is 3.
- Grok session spans stay on one log. Command Code counts cached prompt tokens once. Antigravity's capped reader stays off Windows. Usage helpers stay macOS-only.
- Script download URLs advanced to `v1.0.1`.

## [1.0.0] - 2026-09-18

### Added

- First tag of this self-hosted fork: email and password sign-in, Team data already in the tree, English/中文 UI, and purple brand mark.
- Production Docker assets, local Compose reference, and the deploy checklist.
- Pre-install scripts and resident submit scripts that pin `TOKENS_API_URL` to the operator's site.
- Docs install tabs that point at this site and do not use `brew services` or `bunx` for login and submit.
