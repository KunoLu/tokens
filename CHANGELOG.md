# Changelog

[中文](./CHANGELOG.zh.md)

This file records tags published by this repository. The version line is `1.0.x`.
Tags such as `v27.x` belong to upstream `missuo/tokens` and are not releases of this fork.

## [Unreleased]

## [1.0.5] - 2026-11-14

### Added

- Multilingual interface from upstream v27.2.1 (#20): the web UI and the CLI now ship English, 中文, and 日本語 catalogs with a language switcher and locale-prefixed URLs. The legacy `tt_locale=zh` cookie maps to `zh-CN`, and old unprefixed Chinese URLs redirect instead of returning 404. The CLI accepts `--lang` and `TOKENS_LANG`.
- A "yesterday" period on the leaderboard and profile pages (#22; upstream v27.2.1).
- Redesigned Open Graph cards (#23; upstream v27.2.1): site, page, and profile cards render in all three locales, and the profile card shows tokens, spend, rank, active days, top clients, and a one-year activity strip.
- Cursor desktop sign-in and unified Cursor billing (#23; upstream v27.2.1).
- Account time zone setting (#21, #24): the settings page offers a time zone, and leaving it blank falls back to browser auto-detection. The profile page's "today" and the chart end date follow the viewer's time zone — the viewer's setting when present, otherwise the browser-detected one. Only standard IANA names and UTC are accepted.

### Fixed

- Cursor synchronization is incremental (#22; upstream v27.2.1): the CLI fetches only the recent delta from a watermark that accounts for the correction window and clock skew, merges by event identity without losing history, leaves the cache untouched on an empty delta, and still performs a full sync on a cold cache.
- The leaderboard ships the full board in one payload and locates the viewer client-side (#22; upstream v27.2.1). It no longer streams, so a failure surfaces as a 500 instead of a broken stream.
- GitHub rate limiting no longer wipes profile social links: snapshots are written only when both GitHub requests complete (#23; upstream v27.2.1).
- Parser fixes from upstream v27.2.1 (#23): Devin sessions deduplicate by request ID and Codex `.jsonl.zst` logs are scanned; Pi and DSH undercounting, Codex compaction accounting, Cursor cross-event snapshot duplication, and negative DSH values are corrected.
- Windows autosubmit fixes from upstream v27.2.1 (#23), including the windowless `tokensw` launcher packaged with the CLI.
- Turning a profile private now purges the leaderboard caches immediately (#23): the shared data cache and the en, zh-CN, and ja leaderboard pages are invalidated at once, with no staleness window.
- A database hiccup on a profile page no longer turns into a cached 404: 5xx responses propagate instead of being cached (#23; upstream v27.2.1).
- Empty-state icons on the settings page are centered (#22; upstream v27.2.1).

### Changed

- **This release carries one database migration, `0028_add_user_timezone` (a nullable `timezone` column on `users`); run it before the new code serves traffic.** Vercel Production builds run `db:migrate` automatically before compiling (`web/vercel.json`, only when `VERCEL_ENV` is `production`; the `main` branch does not auto-deploy). Self-hosted installs must run `DATABASE_URL=… bun run db:migrate` by hand before building and starting the new web code (`docs/deploy/self-host-production.md`); Docker upgrades run `docker compose exec web bun run --cwd web db:migrate` inside the web container (`docs/deploy/docker/prod/README.md`). On an unmigrated database, session parsing keeps working (the time zone degrades to auto-detection), but password sign-in, device polling, team-related queries, and the settings time zone endpoints (503) fail — migrate first.
- MCP server names are no longer scanned or submitted by default (#23; upstream v27.2.1). Opt in with `tokens mcp-names`, the `shareMcpServerNames` setting in `settings.json`, or `submit --include-mcp`; `submit --dry-run --json` prints the exact payload. The server stores names only when the payload carries `mcpServersOptIn: true`, drops names sent by older CLIs, and clears the stored list on the next local submission; the privacy and docs pages describe this default.
- This repository's manifest versions (Cargo workspace and `packages/cli*`) and pinned script refs advanced to `1.0.5` and `v1.0.5`, respectively, for this repository's `v1.0.5` tag. This repository publishes tags only, with no GitHub Releases; these version and ref changes do not change the upstream binary version installed by `/install.sh`.

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
