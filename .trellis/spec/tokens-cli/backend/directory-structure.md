# Directory Structure: `cli/` Rust Workspace

> Where code lives in the Rust workspace and which crate owns what.
> Workspace manifest: `cli/Cargo.toml` (members `tokens-core`, `tokens-cli`;
> shared dependencies pinned under `[workspace.dependencies]`).

---

## Two crates, one boundary

```
cli/
├── Cargo.toml                  # workspace root; [workspace.package].version is
│                               # the release version (written by scripts/set-version.sh)
├── tokens-core/                # library crate — no clap, no user I/O
│   └── src/
│       ├── lib.rs              # public API surface; #![deny(clippy::all)];
│       │                       # canonical_model_id / normalize_model_for_grouping
│       ├── clients.rs          # ClientId enum + ClientDef registry (one entry per AI client)
│       ├── sessions/           # one parser module per client → UnifiedMessage
│       │   └── mod.rs          # UnifiedMessage, CostSource; ~45 client modules
│       ├── scanner.rs          # walkdir + rayon parallel discovery of session files
│       ├── aggregator.rs       # daily/per-model aggregation of UnifiedMessage
│       ├── pricing/            # aliases.rs, lookup.rs, cache.rs, custom.rs,
│       │                       # litellm.rs, models_dev.rs, openrouter.rs
│       ├── sessionize.rs       # session-interval computation (active-time metrics)
│       ├── model_alias.rs      # user-configured display-name folding (presentation only)
│       ├── opencode_model_name.rs
│       ├── message_cache.rs    # bincode disk cache of parsed messages
│       ├── paths.rs            # config/cache dir resolution (TOKENS_CONFIG_DIR override)
│       ├── fs_atomic.rs        # write-temp-then-rename file replacement
│       ├── bucket_tz.rs        # pinned IANA timezone for date bucketing
│       ├── cc_mirror.rs, mcp.rs, provider_identity.rs
└── tokens-cli/                 # binary crate — clap UX and all user-facing I/O
    └── src/
        ├── main.rs             # Cli::parse() → command dispatch; ClientFilter enum;
        │                       # init_tracing(); run_submit_command; run_serve_command
        ├── commands/           # autosubmit.rs, codex_activity.rs, import.rs,
        │                       # status.rs, usage/ (shared types for integrations)
        ├── auth.rs             # credentials.json; TOKENS_API_TOKEN bypass
        ├── settings.rs         # settings.json (camelCase serde); atomic save
        ├── device.rs           # device.json identity for submissions
        ├── cursor.rs, trae.rs, warp.rs, antigravity.rs   # per-provider integrations
        ├── timezone.rs         # installs bucket timezone before any scan
        └── paths.rs            # thin re-export of core paths
```

## Ownership rules

- **Core owns the domain**: if it scans, parses, aggregates, prices, or
  normalizes, it lives in `tokens-core`. The binary must stay thin enough that
  every behavior is reachable from library calls.
- **The binary owns UX**: clap definitions, colored terminal output, exit
  codes, auth, and OS integrations live in `tokens-cli`. `main.rs` is the only
  place clap is used; `ClientFilter` (the `--client` CLI enum) deliberately
  stays in `main.rs` so `tokens-core` never depends on clap
  (`cli/tokens-cli/src/main.rs:432+`).
- **No global flags**: `Cli` has only a subcommand. Filters (`--json`,
  `--client`, `--since/--until`, …) are declared on the subcommands that use
  them — the old global flags only fed report commands that no longer exist
  (comment at `cli/tokens-cli/src/main.rs:22-24`).
- **Startup hooks before dispatch**: `main()` installs model aliases, OpenCode
  display names, and the bucket timezone before running any command
  (`cli/tokens-cli/src/main.rs:313-327`). Anything every command needs belongs
  in that block, not inside individual commands.

## Where new things go

| New thing | Location |
|-----------|----------|
| Parser for a new AI client | `cli/tokens-core/src/sessions/<client>.rs` + a `ClientDef` row in `clients.rs` (see [Session Parsers](./session-parsers.md)) |
| New top-level command | `Commands` enum in `main.rs`; logic module alongside `commands/` or a dedicated integration file |
| New pricing source | `cli/tokens-core/src/pricing/` |
| New setting | `Settings` in `cli/tokens-cli/src/settings.rs` (camelCase serde, `#[serde(default)]` so old files parse) |
| New env override | Documented next to the code that reads it (e.g. `TOKENS_CONFIG_DIR` in `paths.rs`, `TOKENS_LOG` in `main.rs::init_tracing`) |
