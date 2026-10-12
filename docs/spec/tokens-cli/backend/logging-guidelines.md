# Logging Guidelines: Rust CLI

> Two separate output channels: diagnostics go through `tracing` to **stderr**
> (opt-in), user-facing messages use `colored` + `println!`/`eprintln!`.
> **stdout carries command output only** — never diagnostics.

---

## The one rule that matters

`--json` commands (e.g. `tokens status --json`, `tokens cursor accounts --json`)
print machine-parseable JSON on stdout. Any log line on stdout breaks
consumers. `init_tracing()` therefore wires the subscriber to stderr
explicitly (`cli/tokens-cli/src/main.rs:288-306`):

```rust
/// Logs go to stderr so `--json` output on stdout stays parseable.
fn init_tracing() {
    let Ok(filter) = std::env::var("TOKENS_LOG") else { return; };
    // …
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .try_init();
}
```

## Conventions

- **Opt-in only.** Tracing is active only when `TOKENS_LOG` is set
  (`TOKENS_LOG=warn`, `TOKENS_LOG=debug`, or any `EnvFilter` directive). An
  empty or invalid value is ignored with at most one `eprintln!` note — the
  command still runs (`main.rs:289-305`). Never enable a subscriber by
  default.
- **`tokens-core` logs with `tracing` only.** The library never prints; it
  emits e.g. `tracing::warn!` when an extra scan path escapes `$HOME`
  (`cli/tokens-core/src/scanner.rs:17-26`) or when the message cache fails
  (`message_cache.rs`). Cache warnings also have a one-time `eprintln!`
  fallback because core code can run without a subscriber installed.
- **User-facing output uses `colored`.** Interactive messages are
  `println!`/`eprintln!` with `colored` styling, e.g.
  `"  Syncing Cursor usage data...".bright_black()` in
  `run_submit_command` (`main.rs:2275`) and `"Not logged in.".yellow()`.
  Match the existing style: two-space indent, `.cyan()` for banners,
  `.yellow()` for warnings, `.bright_black()` for progress notes.
- **No structured logging in `packages/cli`.** The npm launcher only calls
  `console.error` when no binary can be resolved
  (`packages/cli/src/index.ts`); it does not log otherwise.

## Forbidden

- Logging anything to stdout on a command that can emit JSON.
- `println!` debugging left in `tokens-core` — use `tracing::debug!`.
- Writing diagnostics to files unless the user explicitly asked for a capture
  (`tokens headless --output` is the existing pattern).
