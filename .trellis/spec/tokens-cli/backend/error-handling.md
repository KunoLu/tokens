# Error Handling: Rust CLI

> Error strategy per zone: the binary uses `anyhow`, the core library uses
> plain string/IO errors, and configuration loading is fail-soft.

---

## By zone

| Zone | Pattern | Example |
|------|---------|---------|
| `tokens-cli` (binary) | `anyhow::Result<T>` everywhere; `main() -> Result<()>` propagates to process exit | `use anyhow::Result;` at `cli/tokens-cli/src/main.rs:12`; `bail!` / `.context()` in `commands/import.rs:22` |
| User-facing argument validation | `eprintln!` + `std::process::exit(1)` — a usage error is not an `Err` | `run_headless_command` unknown source/format (`main.rs`) |
| `tokens-core` public async report/graph API | `Result<T, String>` — callers pattern-match on message text | `generate_graph` in `cli/tokens-core/src/lib.rs` |
| `tokens-core` internals | `std::io::Result`, `Option`, early returns; bad records are skipped, not fatal | one malformed line drops one message, not the whole session (e.g. `sessions/jcode.rs` lenient parsing) |
| Config load (`settings.rs`) | **Fail-soft**: warn once per process to stderr, continue with `Settings::default()` | `Settings::load` at `cli/tokens-cli/src/settings.rs:287-304` |
| Cache failures | `tracing::warn!`; when no subscriber is installed, a one-time `eprintln!` fallback | `cli/tokens-core/src/message_cache.rs` |

## Rules

1. **Do not introduce `thiserror` enums in `tokens-core`.** `thiserror` is a
   declared workspace dependency (`cli/Cargo.toml:39`) but the core source has
   no error enums; public async surfaces already return `Result<T, String>`
   and callers expect plain strings. A new typed error layer would have to
   migrate every caller at once — that is a dedicated refactor, not a drive-by.
2. **Fail-soft on local state, fail-hard on submission.** A corrupt
   `settings.json` or unreadable cache must never stop a scan or submit — warn
   and continue. But anything that would send wrong data to the server (bad
   auth, unfixable payload) stops the command with a clear message.
3. **Parser robustness beats strictness.** Session files are written by other
   vendors' software and can be truncated or corrupt. Parsers clamp untrusted
   integers (see the varint clamp in `sessions/antigravity_cli.rs`) and skip
   bad records rather than failing the whole client.
4. **Error messages name the file and the cause.** `Settings::load` prints
   `failed to parse <path>: <error>` and says it is continuing with defaults —
   silently reverting the pinned timezone would double-count days
   (`settings.rs:289-301`).
5. **Warn-once for repeated paths.** `load()` runs several times per command;
   use a static `AtomicBool` guard (as in `settings.rs:296-301`) rather than
   spamming the terminal.
