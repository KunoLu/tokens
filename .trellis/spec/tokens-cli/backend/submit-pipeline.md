# Submit Pipeline

> How local session files become a server submission: scan → aggregate →
> validate/trim → POST. Entry point: `run_submit_command` in
> `cli/tokens-cli/src/main.rs:2218`.

---

## Pipeline stages

1. **Auth gate.** `auth::resolve_api_token()` (stored `credentials.json` or
   `TOKENS_API_TOKEN`). Interactive submit without a token prints
   "Not logged in." and exits 1; autosubmit returns an error instead
   (`main.rs:2232-2247`).
2. **Timezone pin.** The detected IANA timezone is persisted on first submit
   so later runs — including from other timezones — bucket usage into the same
   calendar dates (`main.rs:2261-2263`).
3. **Provider sync.** Some clients have no parseable local sessions and are
   synced from vendor APIs instead: Cursor usage is pulled into a local CSV
   cache before scanning (`main.rs:2274-2289`; Tokens deliberately does not
   parse `~/.cursor` session data — see the messages at `main.rs:790,804`).
   Sync failures fall back to the cached data with a warning.
4. **Scan.** `tokens-core`'s scanner (`scanner.rs`, walkdir + rayon) discovers
   session files for the selected clients; parsers emit `UnifiedMessage`s.
5. **Aggregate.** `generate_graph` builds the `GraphResult` — per-day,
   per-client, per-model totals with canonical model ids.
6. **Trim.** `exclude_tokenless_cost_contributions` drops rows the server would
   reject (cost with zero tokens — historical Cursor request charges, Warp/Oz
   aggregate counters) and prints exactly what was excluded
   (`main.rs:1914-2026`). The legacy `premium-tool-call` carve-out is exempt;
   see [Quality Guidelines](./quality-guidelines.md).
7. **Upload.** The payload is serialized with camelCase serde (`Ts*` structs,
   `main.rs:947-1130`) and POSTed to the web API, where
   `web/src/lib/validation/submission.ts` validates it and
   `web/src/app/api/submit/route.ts` writes it and calls `revalidateTag`.

## `--dry-run`

`tokens submit --dry-run` runs the full scan/aggregate/trim pipeline and shows
what would be submitted without uploading (`main.rs:71-75`). It is the way to
inspect parser output; use it when changing a parser.

## `serve`: submissions as child processes

`tokens serve` is a long-running daemon that submits on an interval (default
30 minutes, `--interval` or `TOKENS_SUBMIT_INTERVAL`). Each run is spawned as a
**child `tokens submit` process** via `std::env::current_exe()`
(`run_submit_subprocess`, `main.rs:2185-2202`), not an in-process call: the
child holds the entire parsed-session working set and then exits, keeping the
daemon's memory flat. Client filters are forwarded verbatim as `--client`
flags. A startup jitter staggers fleets (`serve_startup_jitter`,
`main.rs:2204-2216`).

## Import boundary: normalize, never upload

`tokens import` (`cli/tokens-cli/src/commands/import.rs`) converts third-party
aggregate exports (currently clawdboard) into tokens-shaped JSON for review
or archival. **It does not submit anything** — backfilled aggregates are not
independently verifiable the way locally scanned sessions are, and uploading
them requires server-side backfill tagging
(`import.rs:15-20`, upstream issue junhoyeo/tokscale#888). Suspicious rows
(cost without tokens, future dates, non-finite costs) are reported as
warnings, not silently dropped, because the file is meant for human review.
