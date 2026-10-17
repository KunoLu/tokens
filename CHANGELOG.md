# Changelog

[中文](./CHANGELOG.zh.md)

This file records tags published by this repository. The version line is `1.0.x`.
Tags such as `v27.x` belong to upstream `missuo/tokens` and are not releases of this fork.

## [Unreleased]

### Changed

- Vercel Git auto-deploy for `main` is disabled via `git.deploymentEnabled` in `web/vercel.json` (#11). Production deploys are triggered manually.

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
