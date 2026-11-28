//! End-to-end Qoder / Qoder CN coverage.
//!
//! Every fixture here is synthetic: a temp home with JSONL transcripts, segment
//! logs and a real (tempfile) SQLite database. Nothing reads the developer's
//! `~/.qoder`, and no test depends on Qoder being installed.
//! Only tests of persistent caching need a child process with a temporary
//! `TOKENS_CONFIG_DIR`; count-only and in-memory parses use the fixture home.
//!
//! The token assertions are exact on purpose. The IDE case pins the documented
//! `prompt_tokens` / `cached_tokens` / `completion_tokens` relationship
//! (39005 / 37485 / 67 → 1520 uncached input, 37485 cache read, 67 output,
//! 39072 total) so "cached is already inside prompt" cannot regress silently.

mod common;

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokens_core::pricing::{litellm::ModelPricing, PricingService};
use tokens_core::scanner::{scan_all_clients_with_scanner_settings, ScannerSettings};
use tokens_core::sessions::qoder::{
    collapse_qoder_messages, parse_qoder_segment_file, parse_qoder_transcript_file, QoderSource,
};
use tokens_core::{
    parse_local_clients, parse_local_unified_messages_with_pricing,
    parse_local_unified_messages_with_pricing_uncached, ClientId, LocalParseOptions, PathRoot,
    UnifiedMessage,
};

const QODER_TEST_CHILD: &str = "TOKENS_QODER_TEST_CHILD";

fn is_qoder_test_child(test: &str) -> bool {
    std::env::var(QODER_TEST_CHILD).ok().as_deref() == Some(test)
}

fn qoder_test_command(test: &str, home: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("locate Qoder test binary"));
    command
        .args(["--exact", test, "--format", "pretty"])
        .env(QODER_TEST_CHILD, test)
        .env("TOKENS_CONFIG_DIR", home.join("tokens-config"))
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("TOKENS_EXTRA_DIRS")
        .env_remove("TOKENS_HEADLESS_DIR");
    command
}

fn run_qoder_test_child(command: &mut Command) {
    let output = command.output().expect("run isolated Qoder test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // libtest also exits successfully when an exact filter matches no tests.
    assert!(
        output.status.success() && stdout.contains("test result: ok. 1 passed;"),
        "Qoder child {command:?} failed ({}):\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Run the named test in a child, then return true to stop the parent copy.
fn rerun_with_isolated_cache(test: &str) -> bool {
    if is_qoder_test_child(test) {
        return false;
    }
    let profile = common::temp_home();
    run_qoder_test_child(&mut qoder_test_command(test, profile.path()));
    true
}

/// Parse a synthetic home with the uncached (in-memory) source cache.
async fn parse_uncached(home: &Path) -> Vec<UnifiedMessage> {
    parse_local_unified_messages_with_pricing_uncached(qoder_options(home), None)
        .await
        .expect("uncached local parse")
}

async fn assert_qoder_cache_parity(
    home: &Path,
    pricing: Option<&PricingService>,
) -> Vec<UnifiedMessage> {
    let scan = || parse_local_unified_messages_with_pricing(qoder_options(home), pricing);
    let first = scan().await.unwrap();
    assert_eq!(scan().await.unwrap(), first);
    assert_eq!(
        parse_local_unified_messages_with_pricing_uncached(qoder_options(home), pricing)
            .await
            .unwrap(),
        first
    );
    first
}

fn qoder_options(home: &Path) -> LocalParseOptions {
    LocalParseOptions {
        home_dir: Some(home.to_string_lossy().into_owned()),
        use_env_roots: false,
        clients: Some(vec!["qoder".to_string()]),
        since: None,
        until: None,
        year: None,
        scanner_settings: ScannerSettings {
            bucket_timezone: Some("UTC".into()),
            ..ScannerSettings::default()
        },
    }
}

fn write_jsonl(path: &Path, lines: &[&str]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, lines.join("\n") + "\n").unwrap();
}

fn write_ide_request(path: &Path, request: &str, model_info: &str, token_info: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let db = rusqlite::Connection::open(path).unwrap();
    db.execute_batch(
        "CREATE TABLE chat_message (
             id TEXT PRIMARY KEY, session_id TEXT, request_id TEXT, role TEXT,
             token_info TEXT, model_info TEXT, gmt_create INTEGER
         );",
    )
    .unwrap();
    db.execute(
        "INSERT INTO chat_message VALUES ('m1', 'sess-ide', ?1, 'assistant', ?2, ?3, 1794045602000)",
        rusqlite::params![request, token_info, model_info],
    )
    .unwrap();
}

fn international_ide_db(home: &Path) -> PathBuf {
    PathBuf::from(PathRoot::AppData.resolve_with_env_strategy(home.to_str().unwrap(), false))
        .join("Qoder/SharedClientCache/cache/db/local.db")
}

fn qoder_observation_dir() -> PathBuf {
    tokens_core::paths::get_cache_dir().join("source-message-cache-v2/qoder")
}

fn has_qoder_cache_entry() -> bool {
    qoder_observation_dir().read_dir().is_ok_and(|mut entries| {
        entries.any(|entry| entry.unwrap().path().extension().is_some_and(|ext| ext == "bin"))
    })
}

fn total_tokens(messages: &[UnifiedMessage]) -> i64 {
    messages.iter().map(|message| message.tokens.total()).sum()
}

fn buckets(messages: &[UnifiedMessage]) -> (i64, i64, i64, i64) {
    messages.iter().fold((0, 0, 0, 0), |accumulator, message| {
        (
            accumulator.0 + message.tokens.input,
            accumulator.1 + message.tokens.output,
            accumulator.2 + message.tokens.cache_read,
            accumulator.3 + message.tokens.cache_write,
        )
    })
}

const INTL_ONE_REQUEST: &str = concat!(
    r#"{"type":"runtime-config","sessionId":"sess-intl","model":"qmodel","timestamp":1794041573406}"#,
    "\n",
    r#"{"type":"user","uuid":"u1","sessionId":"sess-intl","cwd":"/work/intl-app","timestamp":"2026-11-07T10:00:00.000Z","message":{"role":"user","content":"hello"}}"#,
    "\n",
    r#"{"type":"assistant","uuid":"a1","sessionId":"sess-intl","cwd":"/work/intl-app","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"msg_intl","role":"assistant","model":"qmodel","content":[{"type":"thinking","thinking":"plan"}]}}"#,
    "\n",
    r#"{"type":"assistant","uuid":"a2","sessionId":"sess-intl","cwd":"/work/intl-app","timestamp":"2026-11-07T10:00:05.000Z","message":{"id":"msg_intl","role":"assistant","model":"qmodel","content":[{"type":"text","text":"done"}],"usage":{"input_tokens":1200,"output_tokens":40,"cache_read_input_tokens":800,"cache_creation_input_tokens":100,"cache_creation":{"ephemeral_1h_input_tokens":50},"request_id":"req-intl-1"}}}"#,
    "\n",
);

const CN_ONE_REQUEST: &str = concat!(
    r#"{"type":"runtime-config","sessionId":"sess-cn","model":"dfmodel","timestamp":1794041573406}"#,
    "\n",
    r#"{"type":"user","uuid":"u1","sessionId":"sess-cn","cwd":"C:\\work\\cn-app","timestamp":"2026-11-07T11:00:00.000Z","message":{"role":"user","content":"hello"}}"#,
    "\n",
    r#"{"type":"assistant","uuid":"a1","sessionId":"sess-cn","cwd":"C:\\work\\cn-app","timestamp":"2026-11-07T11:00:03.000Z","message":{"id":"msg_cn","role":"assistant","model":"dfmodel","content":[{"type":"text","text":"done"}],"usage":{"input_tokens":1000,"output_tokens":20,"cache_read_input_tokens":600,"cache_creation_input_tokens":0,"request_id":"req-cn-1"}}}"#,
    "\n",
);

/// T01 / T02 / T03: both editions are discovered under one client, counted
/// separately, and never deduplicated against each other even when their
/// request ids collide.
#[tokio::test]
async fn test_qoder_both_editions_are_merged_without_cross_deduplication() {
    let home = common::temp_home();
    write_jsonl(
        &home
            .path()
            .join(".qoder/projects/-work-intl-app/sess-intl.jsonl"),
        &[INTL_ONE_REQUEST],
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder-cn/projects/-C-work-cn-app/sess-cn.jsonl"),
        &[CN_ONE_REQUEST],
    );
    // Same request id in the other edition: a different account's request.
    write_jsonl(
        &home
            .path()
            .join(".qoder-cn/projects/-C-work-cn-app/sess-cn-dup.jsonl"),
        &[
            r#"{"type":"assistant","sessionId":"sess-cn-dup","cwd":"C:\\work\\cn-app","timestamp":"2026-11-07T11:05:00.000Z","message":{"id":"msg_cn_dup","role":"assistant","model":"dfmodel","usage":{"input_tokens":1000,"output_tokens":20,"cache_read_input_tokens":600,"request_id":"req-intl-1"}}}"#,
        ],
    );

    let scan = scan_all_clients_with_scanner_settings(
        home.path().to_str().unwrap(),
        &["qoder".to_string()],
        false,
        &ScannerSettings::default(),
    );
    assert_eq!(scan.get(ClientId::Qoder).len(), 3);

    let parsed = parse_local_clients(qoder_options(home.path())).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 3);

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 3);
    assert!(messages.iter().all(|message| message.client == "qoder"));

    // (input, output, cache_read, cache_write)
    assert_eq!(
        buckets(&messages),
        (400 + 400 + 400, 40 + 20 + 20, 800 + 600 + 600, 100)
    );
    assert_eq!(total_tokens(&messages), 1200 + 80 + 2000 + 100);

    let models: Vec<&str> = messages.iter().map(|m| m.model_id.as_str()).collect();
    assert!(models.contains(&"qmodel"));
    assert!(models.contains(&"dfmodel"));
}

/// T18: the model a request ran on follows `runtime-config` switches inside one
/// session, and `auto` resolves through the preceding `businessModelKey`.
#[tokio::test]
async fn test_qoder_model_switch_and_auto_resolution() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-switch.jsonl"),
        &[concat!(
            r#"{"type":"runtime-config","sessionId":"sess-switch","model":"qmodel","timestamp":1794041573406}"#,
            "\n",
            r#"{"type":"user","uuid":"u1","sessionId":"sess-switch","businessModelKey":"qfmodel","timestamp":"2026-11-07T23:59:59.000Z","message":{"role":"user","content":"one"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-switch","timestamp":"2026-11-08T00:00:02.000Z","message":{"id":"msg_1","role":"assistant","model":"auto","usage":{"input_tokens":100,"output_tokens":10,"request_id":"req-1"}}}"#,
            "\n",
            r#"{"type":"user","uuid":"u2","sessionId":"sess-switch","timestamp":"2026-11-08T00:01:00.000Z","message":{"role":"user","content":"two"}}"#,
            "\n",
            r#"{"type":"runtime-config","sessionId":"sess-switch","model":"gfmodel","timestamp":1794041600000}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-switch","timestamp":"2026-11-08T00:02:00.000Z","message":{"id":"msg_2","role":"assistant","model":"auto","usage":{"input_tokens":200,"output_tokens":20,"request_id":"req-2"}}}"#,
            "\n",
        )],
    );

    let mut options = qoder_options(home.path());
    options.since = Some("2026-11-07".into());
    options.until = Some("2026-11-08".into());
    let messages = parse_local_unified_messages_with_pricing_uncached(options, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].model_id, "qfmodel");
    assert_eq!(messages[0].tokens.input, 100);
    assert_eq!(messages[1].model_id, "gfmodel");
    assert_eq!(messages[1].tokens.input, 200);
    assert!(messages.iter().all(|m| m.provider_id == "qoder"));
    // A switch does not move the request to the other turn's day.
    assert_eq!(messages[0].date, "2026-11-07");
    assert_eq!(messages[1].date, "2026-11-08");
}

/// T04 / T05: fragments of one reply collapse, distinct replies in one session
/// stay separate.
#[tokio::test]
async fn test_qoder_multi_fragment_and_multi_request_session() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-multi.jsonl"),
        &[concat!(
            r#"{"type":"user","uuid":"u1","sessionId":"sess-multi","timestamp":"2026-11-07T10:00:00.000Z","message":{"role":"user","content":"one"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-multi","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"msg_a","role":"assistant","model":"qmodel","usage":{"input_tokens":1000,"output_tokens":10,"request_id":"req-a"}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-multi","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_a","role":"assistant","model":"qmodel","usage":{"input_tokens":1000,"output_tokens":90,"request_id":"req-a"}}}"#,
            "\n",
            r#"{"type":"user","uuid":"u2","sessionId":"sess-multi","timestamp":"2026-11-07T10:01:00.000Z","message":{"role":"user","content":"two"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a3","sessionId":"sess-multi","timestamp":"2026-11-07T10:01:05.000Z","message":{"id":"msg_b","role":"assistant","model":"qmodel","usage":{"input_tokens":2000,"output_tokens":20,"request_id":"req-b"}}}"#,
            "\n",
        )],
    );

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].tokens.input, 1000);
    assert_eq!(messages[0].tokens.output, 90);
    // Measured from the prompt (10:00:00) to the reply's last fragment (10:00:02).
    assert_eq!(messages[0].duration_ms, Some(2000));
    assert_eq!(messages[1].tokens.input, 2000);
    assert_eq!(messages[1].tokens.output, 20);
    assert_eq!(total_tokens(&messages), 3110);
}

/// T06: a forked session replaying the parent's turn counts it once, wherever
/// the two files land in scan order.
#[tokio::test]
async fn test_qoder_forked_session_replay_counts_once() {
    let home = common::temp_home();
    let parent = concat!(
        r#"{"type":"assistant","uuid":"a1","sessionId":"sess-parent","cwd":"/work/app","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_parent","role":"assistant","model":"qmodel","usage":{"input_tokens":500,"output_tokens":120,"cache_read_input_tokens":300,"cache_creation_input_tokens":50,"request_id":"req-parent"}}}"#,
        "\n",
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder/projects/-work-app/sess-parent.jsonl"),
        &[parent],
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder/projects/-work-app/sess-fork.jsonl"),
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-fork","cwd":"/work/app","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_parent","role":"assistant","model":"qmodel","usage":{"input_tokens":500,"output_tokens":120,"cache_read_input_tokens":300,"cache_creation_input_tokens":50,"request_id":"req-parent"}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-fork","cwd":"/work/app","timestamp":"2026-11-07T10:01:00.000Z","message":{"id":"msg_fork","role":"assistant","model":"qmodel","usage":{"input_tokens":600,"output_tokens":80,"request_id":"req-fork"}}}"#,
            "\n",
        )],
    );

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(buckets(&messages), (200 + 600, 120 + 80, 300, 50));
    assert_eq!(total_tokens(&messages), 1350);
}

/// T07: subagent transcripts are counted, attributed to the agent, and never
/// confused with the parent session's own requests.
#[tokio::test]
async fn test_qoder_subagent_is_counted_once_with_attribution() {
    if rerun_with_isolated_cache("test_qoder_subagent_is_counted_once_with_attribution") {
        return;
    }
    let home = common::temp_home();
    let meta = home
        .path()
        .join(".qoder/projects/-repo/sess-main/subagents/agent-xyz.meta.json");
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-main.jsonl"),
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-main","cwd":"/work/repo","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_main","role":"assistant","model":"qmodel","usage":{"input_tokens":500,"output_tokens":50,"request_id":"req-main"}}}"#,
            "\n",
        )],
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder/projects/-repo/sess-main/subagents/agent-xyz.jsonl"),
        &[concat!(
            r#"{"type":"assistant","isSidechain":true,"sessionId":"sess-main","agentId":"xyz","cwd":"/work/repo","timestamp":"2026-11-07T10:00:10.000Z","message":{"id":"msg_sub","role":"assistant","model":"qmodel","usage":{"input_tokens":300,"output_tokens":30,"request_id":"req-sub"}}}"#,
            "\n",
        )],
    );
    // The journal shares the `.jsonl` suffix but is not a transcript.
    write_jsonl(
        &home
            .path()
            .join(".qoder/projects/-repo/sess-main/subagents/journal.jsonl"),
        &[
            r#"{"type":"assistant","message":{"id":"journal","model":"qmodel","usage":{"input_tokens":9999,"output_tokens":9999}}}"#,
        ],
    );

    let scan = || parse_local_unified_messages_with_pricing(qoder_options(home.path()), None);
    let messages = scan().await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(total_tokens(&messages), 880);
    let subagent = messages
        .iter()
        .find(|message| message.session_id.contains(":agent:xyz"))
        .expect("subagent message");
    assert_eq!(subagent.agent.as_deref(), Some("Xyz"));
    assert_eq!(subagent.tokens.input, 300);
    let parent = messages
        .iter()
        .find(|message| message.session_id == "sess-main")
        .expect("parent message");
    assert_eq!(parent.agent, None);

    for (metadata, expected) in [
        (r#"{"agentType":"code-reviewer"}"#, "Code Reviewer"),
        (r#"{"agentType":"test-generator"}"#, "Test Generator"),
    ] {
        std::fs::write(&meta, metadata).unwrap();
        let updated = scan().await.unwrap();
        assert_eq!(buckets(&updated), buckets(&messages));
        let subagent = updated
            .iter()
            .find(|message| message.session_id == "sess-main:agent:xyz")
            .unwrap();
        assert_eq!(subagent.agent.as_deref(), Some(expected));
    }
    std::fs::remove_file(&meta).unwrap();
    assert_eq!(scan().await.unwrap(), messages);
}

/// T08 / T09: credits-only and usage-less rows are not turned into token or
/// dollar estimates.
#[tokio::test]
async fn test_qoder_credit_only_rows_are_never_estimated() {
    let home = common::temp_home();
    write_jsonl(
        &home
            .path()
            .join(".qoder-cn/projects/-repo/sess-credits.jsonl"),
        &[concat!(
            r#"{"type":"user","uuid":"u1","sessionId":"sess-credits","timestamp":"2026-08-22T06:35:31.000Z","message":{"role":"user","content":"hello"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-credits","timestamp":"2026-08-22T06:36:32.000Z","message":{"id":"msg_credits","role":"assistant","model":"qmodel_38max","content":[{"type":"text","text":"a long reply that must not be estimated"}],"usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"credits":8.229,"original_credits":8.229,"billable":true,"context_usage_ratio":0.2}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-credits","timestamp":"2026-08-22T06:40:00.000Z","message":{"id":"msg_no_usage","role":"assistant","model":"qmodel_38max","content":[{"type":"text","text":"another reply with no usage field at all"}]}}"#,
            "\n",
        )],
    );

    let messages = parse_uncached(home.path()).await;
    assert!(
        messages.is_empty(),
        "credit-only rows carry no measured tokens"
    );

    let parsed = parse_local_clients(qoder_options(home.path())).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 0);
    assert!(parsed.messages.is_empty());
}

/// T10: a BYOK row keeps its vendor and every cache bucket, including the 1h
/// cache-write subset.
#[tokio::test]
async fn test_qoder_byok_cache_buckets() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-byok.jsonl"),
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-byok","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_byok","role":"assistant","model":"deepseek/chat/deepseek-v3","usage":{"input_tokens":2000,"output_tokens":300,"cache_read_input_tokens":1500,"cache_creation_input_tokens":400,"cache_creation":{"ephemeral_1h_input_tokens":250,"ephemeral_5m_input_tokens":150},"request_id":"req-byok"}}}"#,
            "\n",
        )],
    );

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 1);
    let message = &messages[0];
    assert_eq!(message.model_id, "deepseek-v3");
    assert_eq!(message.provider_id, "deepseek");
    assert_eq!(message.tokens.input, 500);
    assert_eq!(message.tokens.cache_read, 1500);
    assert_eq!(message.tokens.cache_write, 400);
    assert_eq!(message.cache_write_1h, 250);
    assert_eq!(message.tokens.output, 300);
    assert_eq!(message.tokens.total(), 2700);
}

/// T19: `qoder/` and `qoder-cn/` qualify the client, not the vendor.
#[tokio::test]
async fn test_qoder_model_namespaces_are_not_providers() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-ns.jsonl"),
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-ns","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_ns_1","role":"assistant","model":"qoder/claude-sonnet-4-6","usage":{"input_tokens":100,"output_tokens":10,"request_id":"req-ns-1"}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-ns","timestamp":"2026-11-07T10:01:02.000Z","message":{"id":"msg_ns_2","role":"assistant","model":"qoder-cn/qwen3-coder-plus","usage":{"input_tokens":200,"output_tokens":20,"request_id":"req-ns-2"}}}"#,
            "\n",
        )],
    );

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].model_id, "claude-sonnet-4-6");
    assert_eq!(messages[0].provider_id, "anthropic");
    assert_eq!(messages[1].model_id, "qwen3-coder-plus");
    assert_eq!(messages[1].provider_id, "qwen");
}

/// T17: a routing slot never inherits another client's `auto` price, while a
/// genuinely identified BYOK model is still priced.
#[tokio::test]
async fn test_qoder_routing_slots_stay_unpriced() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-pricing.jsonl"),
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-pricing","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_auto","role":"assistant","model":"auto","usage":{"input_tokens":1000,"output_tokens":100,"request_id":"req-auto"}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","sessionId":"sess-pricing","timestamp":"2026-11-07T10:01:02.000Z","message":{"id":"msg_byok","role":"assistant","model":"claude-sonnet-4-6","usage":{"input_tokens":1000,"output_tokens":100,"request_id":"req-byok"}}}"#,
            "\n",
        )],
    );

    let mut dataset = HashMap::new();
    dataset.insert(
        "auto".to_string(),
        ModelPricing {
            input_cost_per_token: Some(1.0),
            output_cost_per_token: Some(1.0),
            ..Default::default()
        },
    );
    dataset.insert(
        "claude-sonnet-4-6".to_string(),
        ModelPricing {
            input_cost_per_token: Some(0.001),
            output_cost_per_token: Some(0.002),
            ..Default::default()
        },
    );
    let pricing = PricingService::new(dataset, HashMap::new());

    let messages = parse_local_unified_messages_with_pricing_uncached(
        qoder_options(home.path()),
        Some(&pricing),
    )
    .await
    .unwrap();

    let auto = messages
        .iter()
        .find(|message| message.model_id == "auto")
        .unwrap();
    assert_eq!(auto.provider_id, "qoder");
    assert_eq!(
        auto.cost, 0.0,
        "a routing slot must not borrow another client's price"
    );
    assert_eq!(auto.cost_source, tokens_core::CostSource::Unknown);

    let byok = messages
        .iter()
        .find(|message| message.model_id == "claude-sonnet-4-6")
        .unwrap();
    assert_eq!(byok.provider_id, "anthropic");
    assert!((byok.cost - (1000.0 * 0.001 + 100.0 * 0.002)).abs() < 1e-9);
    assert_eq!(byok.cost_source, tokens_core::CostSource::Estimated);
}

/// T14 / T15 / T16: segment logs are counted per completed request, a request
/// that also exists in a transcript is counted once, and `turn.finished` never
/// adds a second copy.
#[tokio::test]
async fn test_qoder_segments_and_transcript_dedupe() {
    let home = common::temp_home();
    let transcript = home.path().join(".qoder/projects/-work-app/sess-1.jsonl");
    let segments = home
        .path()
        .join(".qoder/logs/sessions/-work-app/sess-1/segments");
    let first_segment = segments.join("run-1.jsonl");
    let second_segment = segments.join("run-2.jsonl");
    write_jsonl(
        &transcript,
        &[concat!(
            r#"{"type":"assistant","uuid":"a1","sessionId":"sess-1","cwd":"/work/alternate","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_1","role":"assistant","model":"qmodel","usage":{"input_tokens":1200,"output_tokens":4,"request_id":"req-shared"}}}"#,
            "\n",
        )],
    );
    write_jsonl(
        &first_segment,
        &[concat!(
            r#"{"ts":"2026-11-07T10:00:01.000Z","type":"session.config.loaded","data":{"project_root":"/work/app"}}"#,
            "\n",
            r#"{"ts":"2026-11-07T10:00:02.000Z","type":"model.response.completed","request_id":"req-shared","data":{"model":"qmodel","input_tokens":1200,"output_tokens":40,"cache_read_input_tokens":800}}"#,
            "\n",
            r#"{"ts":"2026-11-07T10:05:00.000Z","type":"model.response.completed","request_id":"req-segment-only","data":{"model":"qmodel","input_tokens":600,"output_tokens":20,"cache_read_input_tokens":100}}"#,
            "\n",
            r#"{"ts":"2026-11-07T10:05:01.000Z","type":"turn.finished","data":{"model":"qmodel","input_tokens":1800,"output_tokens":60,"cache_read_input_tokens":900}}"#,
            "\n",
        )],
    );
    // The same completed event replayed into a second segment file (a restarted
    // run) must not add a second copy.
    write_jsonl(
        &second_segment,
        &[concat!(
            r#"{"ts":"2026-11-07T10:05:00.000Z","type":"model.response.completed","request_id":"req-segment-only","data":{"model":"qmodel","input_tokens":600,"output_tokens":20,"cache_read_input_tokens":100}}"#,
            "\n",
        )],
    );

    // Diagnostics beside the segment files are not usage: `runs/` holds the
    // same completion event for its own purposes.
    write_jsonl(
        &home
            .path()
            .join(".qoder/logs/sessions/-work-app/sess-1/runs/run-1.jsonl"),
        &[
            r#"{"ts":"2026-11-07T10:05:00.000Z","type":"model.response.completed","request_id":"req-run-dir","data":{"model":"qmodel","input_tokens":9999,"output_tokens":9999}}"#,
        ],
    );

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(total_tokens(&messages), 1240 + 620);
    let (input, output, cache_read, cache_write) = buckets(&messages);
    assert_eq!(
        (input, output, cache_read, cache_write),
        (400 + 500, 40 + 20, 800 + 100, 0)
    );

    let segment_only = messages
        .iter()
        .find(|message| message.tokens.input == 500)
        .expect("the --no-session-persistence request is counted");
    assert_eq!(segment_only.session_id, "sess-1");
    assert_eq!(segment_only.workspace_key.as_deref(), Some("/work/app"));

    // Exercise the merge with duplicates still present, including a metadata tie.
    let source = QoderSource::International;
    let mut raw = parse_qoder_transcript_file(&transcript, source);
    raw.extend(parse_qoder_segment_file(&first_segment, source));
    raw.extend(parse_qoder_segment_file(&second_segment, source));
    assert_eq!(raw.len(), 4);
    let forward = collapse_qoder_messages(raw.clone());
    let backward = collapse_qoder_messages(raw.into_iter().rev().collect());
    assert_eq!(forward.len(), 2);
    assert_eq!(forward, backward);
    assert_eq!(buckets(&forward), buckets(&messages));
}

#[tokio::test]
async fn test_qoder_anonymous_requests_are_scoped_to_their_file() {
    let home = common::temp_home();
    for session in ["sess-a", "sess-b"] {
        write_jsonl(
            &home.path().join(format!(
                ".qoder/logs/sessions/-work-app/{session}/segments/run.jsonl"
            )),
            &[r#"{"ts":"2026-11-07T10:00:00Z","type":"model.response.completed","data":{"input_tokens":100,"output_tokens":10}}"#],
        );
        write_jsonl(
            &home
                .path()
                .join(format!(".qoder/projects/{session}/same-name.jsonl")),
            &[r#"{"type":"assistant","timestamp":"2026-11-07T10:00:00Z","message":{"usage":{"input_tokens":200,"output_tokens":20}}}"#],
        );
    }

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 4);
    assert_eq!(buckets(&messages), (600, 60, 0, 0));
}

#[tokio::test]
async fn test_qoder_three_sources_merge_within_an_edition_before_pricing() {
    if rerun_with_isolated_cache("test_qoder_three_sources_merge_within_an_edition_before_pricing") {
        return;
    }
    let home = common::temp_home();

    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-cli.jsonl"),
        &[r#"{"type":"assistant","sessionId":"sess-cli","timestamp":"2026-11-07T10:00:00Z","message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":1200,"output_tokens":5,"request_id":"shared"}}}"#],
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder/logs/sessions/-repo/sess-cli/segments/run.jsonl"),
        &[r#"{"ts":"2026-11-07T10:00:01Z","type":"model.response.completed","request_id":"shared","data":{"model":"claude-sonnet-4-6","input_tokens":1100,"output_tokens":40,"cache_creation_input_tokens":100,"cache_creation":{"ephemeral_1h_input_tokens":50}}}"#],
    );
    let app_data = PathBuf::from(
        PathRoot::AppData.resolve_with_env_strategy(home.path().to_str().unwrap(), false),
    );
    write_ide_request(
        &app_data.join("Qoder/SharedClientCache/cache/db/local.db"),
        "shared",
        r#"{"model_key":"claude-sonnet-4-6"}"#,
        r#"{"prompt_tokens":1000,"cached_tokens":800,"completion_tokens":15}"#,
    );
    // Same IDE message id, session id and request id in the other edition.
    write_ide_request(
        &app_data.join("QoderCN/SharedClientCache/cache/db/local.db"),
        "shared",
        r#"{"model_key":"dfmodel"}"#,
        r#"{"prompt_tokens":20,"completion_tokens":2}"#,
    );

    let parsed = parse_local_clients(qoder_options(home.path())).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 2);
    let pricing = PricingService::new(
        HashMap::from([(
            "claude-sonnet-4-6".into(),
            ModelPricing {
                input_cost_per_token: Some(0.001),
                output_cost_per_token: Some(0.002),
                cache_read_input_token_cost: Some(0.0005),
                cache_creation_input_token_cost: Some(0.00125),
                ..Default::default()
            },
        )]),
        HashMap::new(),
    );
    let scan = || {
        parse_local_unified_messages_with_pricing(qoder_options(home.path()), Some(&pricing))
    };
    let cold = scan().await.unwrap();
    assert_eq!(cold.len(), 2);
    let byok = cold
        .iter()
        .find(|message| message.session_id == "sess-cli")
        .expect("oldest observation determines the session");
    assert_eq!(buckets(std::slice::from_ref(byok)), (400, 40, 800, 100));
    assert_eq!(byok.cache_write_1h, 50);
    assert_eq!(byok.cost_source, tokens_core::CostSource::Estimated);
    assert!((byok.cost - 1.0425).abs() < 1e-9);

    let cn = cold.iter().find(|message| message.model_id == "dfmodel").unwrap();
    assert_eq!(buckets(std::slice::from_ref(cn)), (20, 2, 0, 0));
    assert_eq!(cn.cost_source, tokens_core::CostSource::Unknown);
    assert_eq!(scan().await.unwrap(), cold);
}

/// T11 / T12: the IDE database's `prompt_tokens` already includes
/// `cached_tokens`; a missing `chat_session` table only drops attribution.
#[tokio::test]
async fn test_qoder_ide_database_exact_body_and_missing_session_table() {
    let home = common::temp_home();
    let app_data =
        PathRoot::AppData.resolve_with_env_strategy(home.path().to_str().unwrap(), false);
    let db = PathBuf::from(app_data)
        .join("Qoder")
        .join("SharedClientCache/cache/db/local.db");
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE chat_message (
                 id TEXT PRIMARY KEY,
                 session_id TEXT,
                 request_id TEXT,
                 role TEXT,
                 token_info TEXT,
                 model_info TEXT,
                 gmt_create INTEGER
             );
             CREATE TABLE chat_session (
                 session_id TEXT PRIMARY KEY,
                 project_uri TEXT,
                 project_name TEXT,
                 preferred_model_info TEXT
             );
             INSERT INTO chat_session VALUES
                 ('sess-ide', 'file:///work/ide-app', 'ide-app', '{\"model_key\":\"qmodel\"}');
             INSERT INTO chat_message VALUES
                 ('m1','sess-ide','req-ide-1','assistant',
                  '{\"prompt_tokens\":39005,\"cached_tokens\":37485,\"completion_tokens\":67,\"max_input_tokens\":200000}',
                  '{\"model_key\":\"qwen3.8-max\"}', 1762509600000),
                 ('u1','sess-ide',NULL,'user',
                  '{\"prompt_tokens\":39005,\"cached_tokens\":37485,\"completion_tokens\":0}',
                  NULL, 1762509599000);",
        )
        .unwrap();
    }
    // A second edition with no chat_session table at all.
    let cn_db = PathBuf::from(
        PathRoot::AppData.resolve_with_env_strategy(home.path().to_str().unwrap(), false),
    )
    .join("QoderCN")
    .join("SharedClientCache/cache/db/local.db");
    std::fs::create_dir_all(cn_db.parent().unwrap()).unwrap();
    {
        let conn = rusqlite::Connection::open(&cn_db).unwrap();
        conn.execute_batch(
            "CREATE TABLE chat_message (
                 id TEXT PRIMARY KEY,
                 session_id TEXT,
                 request_id TEXT,
                 role TEXT,
                 token_info TEXT,
                 model_info TEXT,
                 gmt_create INTEGER
             );
             INSERT INTO chat_message VALUES
                 ('c1','sess-cn',NULL,'assistant',
                  '{\"prompt_tokens\":1000,\"cached_tokens\":250,\"completion_tokens\":100}',
                  '{\"model_key\":\"auto\"}', 1762509700000);",
        )
        .unwrap();
    }

    let scan = scan_all_clients_with_scanner_settings(
        home.path().to_str().unwrap(),
        &["qoder".to_string()],
        false,
        &ScannerSettings::default(),
    );
    assert_eq!(scan.qoder_ide_dbs.len(), 2);

    let messages = parse_uncached(home.path()).await;
    assert_eq!(messages.len(), 2);

    let ide = messages
        .iter()
        .find(|message| message.session_id == "sess-ide")
        .expect("IDE row");
    assert_eq!(ide.tokens.input, 1520);
    assert_eq!(ide.tokens.cache_read, 37485);
    assert_eq!(ide.tokens.output, 67);
    assert_eq!(ide.tokens.total(), 39072);
    assert_eq!(ide.model_id, "qwen3.8-max");
    assert_eq!(ide.provider_id, "qwen");
    assert_eq!(ide.workspace_key.as_deref(), Some("/work/ide-app"));

    let cn = messages
        .iter()
        .find(|message| message.session_id == "sess-cn")
        .expect("CN IDE row without chat_session");
    assert_eq!(cn.tokens.input, 750);
    assert_eq!(cn.tokens.cache_read, 250);
    assert_eq!(cn.tokens.output, 100);
    assert_eq!(cn.model_id, "auto");
    assert_eq!(cn.provider_id, "qoder");
    assert_eq!(cn.workspace_key, None);
}

#[tokio::test]
async fn test_qoder_accepts_only_explicit_extra_local_databases() {
    let home = common::temp_home();
    let explicit = home.path().join("alternate/local.db");
    let unlisted = home.path().join("other/local.db");
    for path in [&explicit, &unlisted] {
        write_ide_request(
            path,
            "extra",
            r#"{"model_key":"qmodel"}"#,
            r#"{"prompt_tokens":300,"cached_tokens":100,"completion_tokens":10}"#,
        );
    }
    let mut options = qoder_options(home.path());
    assert!(parse_uncached(home.path()).await.is_empty());

    options.scanner_settings.extra_scan_paths.insert(
        "qoder".into(),
        vec![explicit.clone(), explicit, unlisted.parent().unwrap().to_path_buf()],
    );
    let scan = scan_all_clients_with_scanner_settings(
        home.path().to_str().unwrap(),
        &["qoder".to_string()],
        false,
        &options.scanner_settings,
    );
    assert_eq!(scan.qoder_ide_dbs.len(), 1);
    let parsed = parse_local_clients(options.clone()).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 1);
    let messages = parse_local_unified_messages_with_pricing_uncached(options, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(buckets(&messages), (200, 10, 100, 0));
}

// Pause SQLite after a real WAL fsync and before it publishes the commit in
// the index. Keep the writer alive so closing it cannot checkpoint the WAL.
fn pause_wal_commit(
    writer: rusqlite::Connection,
    sql: &'static str,
) -> (
    std::sync::mpsc::Receiver<()>,
    std::sync::mpsc::Sender<()>,
    std::thread::JoinHandle<rusqlite::Connection>,
) {
    use rusqlite::ffi;
    use std::cell::RefCell;
    use std::sync::{mpsc, OnceLock};

    struct Gate {
        original_sync: unsafe extern "C" fn(*mut ffi::sqlite3_file, i32) -> i32,
        signal: Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>,
    }
    thread_local! {
        static GATE: RefCell<Option<Gate>> = const { RefCell::new(None) };
    }
    static WAL_METHODS: OnceLock<ffi::sqlite3_io_methods> = OnceLock::new();

    unsafe extern "C" fn sync_and_pause(file: *mut ffi::sqlite3_file, flags: i32) -> i32 {
        GATE.with(|gate| {
            let mut gate = gate.borrow_mut();
            let Some(gate) = gate.as_mut() else {
                return ffi::SQLITE_IOERR_FSYNC;
            };
            // SAFETY: SQLite passes the original VFS file and sync flags.
            let result = unsafe { (gate.original_sync)(file, flags) };
            if let Some((paused, resume)) = gate.signal.take() {
                if result == ffi::SQLITE_OK && paused.send(()).is_ok() {
                    let _ = resume.recv();
                }
            }
            result
        })
    }

    let (paused, wait_for_pause) = mpsc::channel();
    let (resume, wait_for_resume) = mpsc::channel();
    let join = std::thread::spawn(move || {
        let mut wal = std::ptr::null_mut::<ffi::sqlite3_file>();
        // SAFETY: This thread owns the writer. The file control writes its
        // live WAL handle; the modified method table has static storage.
        let methods = unsafe {
            assert_eq!(
                ffi::sqlite3_file_control(
                    writer.handle(),
                    c"main".as_ptr(),
                    ffi::SQLITE_FCNTL_JOURNAL_POINTER,
                    (&mut wal as *mut *mut ffi::sqlite3_file).cast(),
                ),
                ffi::SQLITE_OK
            );
            assert!(!wal.is_null());
            (*wal).pMethods
        };
        // SAFETY: The WAL is open for an initialized writer, and this thread
        // restores the VFS's pointer before returning the writer.
        unsafe {
            assert!(!methods.is_null());
            GATE.with(|gate| {
                *gate.borrow_mut() = Some(Gate {
                    original_sync: (*methods).xSync.unwrap(),
                    signal: Some((paused, wait_for_resume)),
                });
            });
            (*wal).pMethods = WAL_METHODS.get_or_init(|| ffi::sqlite3_io_methods {
                xSync: Some(sync_and_pause),
                ..*methods
            });
        }
        let commit = writer.execute(sql, []);
        // SAFETY: The statement did not close or replace the WAL handle.
        unsafe { (*wal).pMethods = methods };
        commit.unwrap();
        writer
    });
    (wait_for_pause, resume, join)
}

/// T13: rows the IDE writes while its own connection is open live in the
/// `-wal` sidecar. A warm cache keyed on the database file alone would never
/// see them.
#[tokio::test]
async fn test_qoder_ide_wal_writes_invalidate_the_source_cache() {
    if rerun_with_isolated_cache("test_qoder_ide_wal_writes_invalidate_the_source_cache") {
        return;
    }
    let home = common::temp_home();

    let db = PathBuf::from(
        PathRoot::AppData.resolve_with_env_strategy(home.path().to_str().unwrap(), false),
    )
    .join("Qoder/SharedClientCache/cache/db/local.db");
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    let writer = rusqlite::Connection::open(&db).unwrap();
    writer
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             PRAGMA wal_autocheckpoint = 0;
             CREATE TABLE chat_message (
                 id TEXT PRIMARY KEY,
                 session_id TEXT,
                 request_id TEXT,
                 role TEXT,
                 token_info TEXT,
                 model_info TEXT,
                 gmt_create INTEGER
             );",
        )
        .unwrap();
    writer
        .execute_batch(
            "CREATE TABLE chat_session (
                 session_id TEXT PRIMARY KEY, project_uri TEXT, project_name TEXT,
                 preferred_model_info TEXT
             );
             INSERT INTO chat_session VALUES
                 ('sess-wal', NULL, NULL, '{\"model_key\":\"openrouter/claude-sonnet-4-6\"}');",
        )
        .unwrap();
    writer
        .execute(
            "INSERT INTO chat_message VALUES
                 ('m1','sess-wal','req-1','assistant', ?1, '{}', 1762509600000)",
            rusqlite::params![r#"{"prompt_tokens":100,"cached_tokens":0,"completion_tokens":10}"#],
        )
        .unwrap();

    let first = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].tokens.input, 100);
    assert_eq!(first[0].model_id, "claude-sonnet-4-6");
    assert_eq!(first[0].provider_id, "openrouter");

    // The IDE is still running: the model switch and new row only update WAL.
    let main_mtime = db.metadata().unwrap().modified().unwrap();
    writer
        .execute(
            "UPDATE chat_session SET preferred_model_info = '{\"model_key\":\"gpt-5\"}'",
            [],
        )
        .unwrap();
    writer
        .execute(
            "INSERT INTO chat_message VALUES
                 ('m2','sess-wal','req-2','assistant', ?1, '{\"model_key\":\"auto\"}', 1762509601000)",
            rusqlite::params![r#"{"prompt_tokens":300,"cached_tokens":100,"completion_tokens":30}"#],
        )
        .unwrap();
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    assert!(wal.exists());
    assert_eq!(db.metadata().unwrap().modified().unwrap(), main_mtime);

    let second = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    assert_eq!(
        second.len(),
        2,
        "the -wal write must invalidate the cached read"
    );
    assert_eq!(buckets(&second), (100 + 200, 10 + 30, 100, 0));
    assert_eq!(second[0].model_id, "gpt-5");
    assert_eq!(second[0].provider_id, "openai");
    assert_eq!(second, parse_uncached(home.path()).await);

    let (paused, resume, commit) =
        pause_wal_commit(writer, "DELETE FROM chat_message WHERE id = 'm1'");
    paused
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let unpublished_wal = std::fs::read(&wal).unwrap();
    let scan = || parse_local_unified_messages_with_pricing(qoder_options(home.path()), None);
    assert_eq!(
        scan().await.unwrap(),
        second,
        "the WAL index has not published the deletion"
    );
    resume.send(()).unwrap();
    let writer = commit.join().unwrap();
    assert_eq!(std::fs::read(&wal).unwrap(), unpublished_wal);
    assert_eq!(scan().await.unwrap().as_slice(), &second[1..]);
    assert_eq!(scan().await.unwrap(), parse_uncached(home.path()).await);
    assert!(
        !has_qoder_cache_entry(),
        "WAL bytes are not a committed snapshot ID"
    );
    drop(writer);
    std::fs::remove_file(&db).unwrap();
    assert!(scan().await.unwrap().is_empty());
    write_ide_request(
        &db,
        "req-1",
        r#"{"model_key":"gfmodel"}"#,
        r#"{"prompt_tokens":9}"#,
    );
    let recreated = scan().await.unwrap();
    assert_eq!(recreated.len(), 1);
    assert_eq!(recreated[0].model_id, "gfmodel");
    assert_eq!(total_tokens(&recreated), 9);
}

#[cfg(unix)]
#[tokio::test]
async fn test_qoder_ide_uses_symlink_target_wal() {
    if rerun_with_isolated_cache("test_qoder_ide_uses_symlink_target_wal") {
        return;
    }
    let home = common::temp_home();
    let target = home.path().join("actual/messages.sqlite");
    let link = home.path().join("local.db");
    write_ide_request(&target, "linked", "{}", r#"{"prompt_tokens":17}"#);
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let writer = rusqlite::Connection::open(&target).unwrap();
    writer
        .execute_batch("PRAGMA journal_mode = WAL; PRAGMA wal_autocheckpoint = 0;")
        .unwrap();

    let mut options = qoder_options(home.path());
    options
        .scanner_settings
        .extra_scan_paths
        .insert("qoder".into(), vec![link]);
    let scan = || parse_local_unified_messages_with_pricing(options.clone(), None);
    assert_eq!(total_tokens(&scan().await.unwrap()), 17);
    let main_mtime = target.metadata().unwrap().modified().unwrap();
    writer
        .execute(
            "UPDATE chat_message SET token_info = '{\"prompt_tokens\":23}'",
            [],
        )
        .unwrap();
    assert_eq!(target.metadata().unwrap().modified().unwrap(), main_mtime);
    assert_eq!(total_tokens(&scan().await.unwrap()), 23);
}

#[tokio::test]
async fn test_qoder_cache_removes_only_owned_orphans() {
    if rerun_with_isolated_cache("test_qoder_cache_removes_only_owned_orphans") {
        return;
    }
    let home = common::temp_home();
    let other_home = common::temp_home();
    let db = international_ide_db(home.path());
    for path in [&db, &international_ide_db(other_home.path())] {
        write_ide_request(path, "orphan", "{}", r#"{"prompt_tokens":17}"#);
    }

    let entries = || {
        qoder_observation_dir()
            .read_dir()
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "bin"))
            .collect::<Vec<_>>()
    };
    let scan = |home: &Path| parse_local_unified_messages_with_pricing(qoder_options(home), None);
    assert_eq!(total_tokens(&scan(home.path()).await.unwrap()), 17);
    let orphan = entries().pop().unwrap();
    assert_eq!(total_tokens(&scan(other_home.path()).await.unwrap()), 17);
    let retained = entries().into_iter().find(|path| path != &orphan).unwrap();

    let foreign = qoder_observation_dir().join("00.bin");
    let legacy = qoder_observation_dir().join(format!("observations-{}.bin", "0".repeat(64)));
    std::fs::write(&foreign, b"another cache").unwrap();
    std::fs::write(&legacy, b"\x01\x05\x09legacy observation without a path").unwrap();
    std::fs::remove_file(&db).unwrap();
    assert!(scan(home.path()).await.unwrap().is_empty());
    assert!(!orphan.exists());
    assert!(retained.exists(), "a live source outside this scan must be kept");
    assert_eq!(std::fs::read(foreign).unwrap(), b"another cache");
    assert_eq!(
        std::fs::read(legacy).unwrap(),
        b"\x01\x05\x09legacy observation without a path"
    );
}

#[tokio::test]
async fn test_qoder_ide_caches_only_a_complete_read() {
    if rerun_with_isolated_cache("test_qoder_ide_caches_only_a_complete_read") {
        return;
    }
    let home = common::temp_home();
    let db = international_ide_db(home.path());
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    let writer = rusqlite::Connection::open(&db).unwrap();
    writer
        .execute_batch(
            "CREATE TABLE fixture (id TEXT, tokens TEXT);
             INSERT INTO fixture VALUES ('good', '{\"prompt_tokens\":17}'), ('bad', '{}');
             CREATE VIEW chat_message AS
                 SELECT id, 's' AS session_id, id AS request_id, 'assistant' AS role,
                     CASE WHEN id = 'bad' THEN abs(-9223372036854775808)
                          ELSE tokens END AS token_info,
                     '{\"model_key\":\"gfmodel\"}' AS model_info, 1794045602000 AS gmt_create
                 FROM fixture;",
        )
        .unwrap();

    let scan = || parse_local_unified_messages_with_pricing(qoder_options(home.path()), None);
    assert_eq!(total_tokens(&scan().await.unwrap()), 17);
    assert!(
        !has_qoder_cache_entry(),
        "do not cache the prefix before a SQLite step error"
    );

    writer.execute("DELETE FROM fixture", []).unwrap();
    assert!(scan().await.unwrap().is_empty());
    assert!(has_qoder_cache_entry(), "a complete empty scan is cacheable");

    writer
        .execute(
            "INSERT INTO fixture VALUES ('new', '{\"prompt_tokens\":23}')",
            [],
        )
        .unwrap();
    assert_eq!(total_tokens(&scan().await.unwrap()), 23);
}

/// T22: rescanning the same home twice — warm cache and cold parse — agrees.
#[tokio::test]
async fn test_qoder_repeated_scans_are_stable() {
    if rerun_with_isolated_cache("test_qoder_repeated_scans_are_stable") {
        return;
    }
    let home = common::temp_home();

    let transcript = home.path().join(".qoder/projects/-work-app/sess-1.jsonl");
    let segment = home
        .path()
        .join(".qoder/logs/sessions/-work-app/sess-2/segments/run.jsonl");
    write_jsonl(&transcript, &[INTL_ONE_REQUEST]);
    write_jsonl(
        &segment,
        &[
            r#"{"ts":"2026-11-07T12:00:00.000Z","type":"model.response.completed","request_id":"req-seg","data":{"model":"qmodel","input_tokens":700,"output_tokens":30,"cache_read_input_tokens":200}}"#,
        ],
    );

    let cold = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    let warm = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    let uncached =
        parse_local_unified_messages_with_pricing_uncached(qoder_options(home.path()), None)
            .await
            .unwrap();

    assert_eq!(cold.len(), 2);
    assert_eq!(total_tokens(&cold), 1340 + 730);
    assert_eq!(total_tokens(&warm), total_tokens(&cold));
    assert_eq!(total_tokens(&uncached), total_tokens(&cold));
    assert_eq!(buckets(&warm), buckets(&cold));

    // An append to each already-cached file must invalidate its old parse.
    for (path, line) in [
        (
            &transcript,
            r#"{"type":"assistant","sessionId":"sess-intl","timestamp":"2026-11-07T13:00:00Z","message":{"id":"msg_3","model":"qmodel","usage":{"input_tokens":50,"output_tokens":5,"request_id":"req-3"}}}"#,
        ),
        (
            &segment,
            r#"{"ts":"2026-11-07T13:01:00Z","type":"model.response.completed","request_id":"req-4","data":{"model":"qmodel","input_tokens":120,"output_tokens":8,"cache_read_input_tokens":20}}"#,
        ),
    ] {
        let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
        writeln!(file, "{line}").unwrap();
    }
    let grown = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    assert_eq!(grown.len(), 4);
    assert_eq!(total_tokens(&grown), total_tokens(&cold) + 55 + 128);
    assert_eq!(grown, parse_uncached(home.path()).await);
}

#[tokio::test]
async fn test_qoder_measured_request_time_beats_earlier_file_mtime() {
    if rerun_with_isolated_cache("test_qoder_measured_request_time_beats_earlier_file_mtime") {
        return;
    }
    let home = common::temp_home();
    let transcript = home.path().join(".qoder/projects/-repo/sess-cli.jsonl");
    write_jsonl(
        &transcript,
        &[r#"{"type":"assistant","sessionId":"sess-cli","message":{"model":"claude-sonnet-4-6","provider":"openrouter","usage":{"output_tokens":40,"request_id":"shared"}}}"#],
    );
    std::fs::OpenOptions::new()
        .write(true)
        .open(&transcript)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_793_959_200))
        .unwrap();

    // Warm the transcript before the measured IDE observation is discovered.
    let transcript_only = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    assert_eq!(transcript_only.len(), 1);
    assert_eq!(transcript_only[0].date, "2026-11-06");
    assert_eq!(transcript_only[0].provider_id, "openrouter");
    assert!(has_qoder_cache_entry());
    std::fs::OpenOptions::new()
        .write(true)
        .open(&transcript)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_793_962_800))
        .unwrap();
    let touched = parse_local_unified_messages_with_pricing(qoder_options(home.path()), None)
        .await
        .unwrap();
    assert_eq!(touched[0].timestamp, 1_793_962_800_000);

    write_ide_request(
        &international_ide_db(home.path()),
        "shared",
        r#"{"model_key":"claude-sonnet-4-6"}"#,
        r#"{"prompt_tokens":100,"cached_tokens":60,"completion_tokens":15}"#,
    );
    let messages = assert_qoder_cache_parity(home.path(), None).await;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].timestamp, 1_794_045_602_000);
    assert_eq!(messages[0].date, "2026-11-07");
    assert_eq!(messages[0].session_id, "sess-ide");
    assert_eq!(messages[0].model_id, "claude-sonnet-4-6");
    assert_eq!(messages[0].provider_id, "openrouter");
    assert_eq!(buckets(&messages), (40, 40, 60, 0));

    let mut options = qoder_options(home.path());
    options.since = Some("2026-11-07".into());
    options.until = Some("2026-11-07".into());
    let parsed = parse_local_clients(options).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 1);
    assert_eq!(parsed.messages.len(), 1);
    assert_eq!(parsed.messages[0].date, "2026-11-07");
    assert_eq!(parsed.messages[0].output, 40);
}

#[tokio::test]
async fn test_qoder_provider_evidence_follows_only_the_winning_model() {
    if rerun_with_isolated_cache("test_qoder_provider_evidence_follows_only_the_winning_model") {
        return;
    }
    let home = common::temp_home();
    let db = international_ide_db(home.path());
    write_ide_request(
        &db,
        "same-model",
        r#"{"model_key":"claude-sonnet-4-6"}"#,
        r#"{"prompt_tokens":100}"#,
    );
    rusqlite::Connection::open(&db)
        .unwrap()
        .execute(
            "INSERT INTO chat_message VALUES
                ('m2', 'sess-ide', 'other-model', 'assistant', '{\"prompt_tokens\":200}',
                 '{\"model_key\":\"gpt-5\"}', 1794045603000)",
            [],
        )
        .unwrap();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-cli.jsonl"),
        &[
            r#"{"type":"assistant","timestamp":"2026-11-07T09:59:00Z","businessModelKey":"claude-sonnet-4-6","message":{"model":"auto","provider":"openrouter","usage":{"output_tokens":10,"request_id":"same-model"}}}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T09:59:01Z","businessModelKey":"claude-sonnet-4-6","message":{"model":"auto","provider":"openrouter","usage":{"output_tokens":20,"request_id":"other-model"}}}"#,
        ],
    );
    write_jsonl(
        &home
            .path()
            .join(".qoder/logs/sessions/-repo/sess-cli/segments/run.jsonl"),
        &[r#"{"ts":"2026-11-07T10:00:01Z","type":"model.response.completed","request_id":"same-model","data":{"model":"gfmodel","input_tokens":60}}"#],
    );

    let messages = assert_qoder_cache_parity(home.path(), None).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].model_id, "claude-sonnet-4-6");
    assert_eq!(messages[0].provider_id, "openrouter");
    assert_eq!(messages[1].model_id, "gpt-5");
    assert_eq!(messages[1].provider_id, "openai");
}

#[tokio::test]
async fn test_qoder_client_model_prefix_preserves_runtime_provider() {
    if rerun_with_isolated_cache("test_qoder_client_model_prefix_preserves_runtime_provider") {
        return;
    }
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-runtime.jsonl"),
        &[
            r#"{"type":"runtime-config","model":"claude-sonnet-4-6","provider":"openrouter"}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:00:00Z","message":{"id":"m1","model":"qoder/claude-sonnet-4-6","usage":{"input_tokens":100}}}"#,
            r#"{"type":"runtime-config","model":"QODER-CN/claude-sonnet-4-6"}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:01:00Z","message":{"id":"m2","model":"claude-sonnet-4-6","usage":{"input_tokens":200}}}"#,
        ],
    );
    let messages = assert_qoder_cache_parity(home.path(), None).await;
    assert_eq!(messages.len(), 2);
    for message in messages {
        assert_eq!(message.model_id, "claude-sonnet-4-6");
        assert_eq!(message.provider_id, "openrouter");
    }
}

#[tokio::test]
async fn test_qoder_late_report_replaces_business_fallback_slot() {
    if rerun_with_isolated_cache("test_qoder_late_report_replaces_business_fallback_slot") {
        return;
    }
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-fragments.jsonl"),
        &[
            r#"{"type":"assistant","timestamp":"2026-11-07T10:00:00Z","businessModelKey":"qmodel","message":{"id":"m1","model":"auto","usage":{"input_tokens":100}}}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:00:01Z","message":{"id":"m1","model":"dfmodel","usage":{"output_tokens":10,"request_id":"real"}}}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:00:02Z","businessModelKey":"gfmodel","message":{"id":"m1","model":"auto","usage":{"output_tokens":12}}}"#,
        ],
    );
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-inherited.jsonl"),
        &[
            r#"{"type":"user","timestamp":"2026-11-07T10:00:59Z","businessModelKey":"qmodel","message":{"content":"next"}}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:01:00Z","message":{"id":"m2","model":"auto","usage":{"input_tokens":20}}}"#,
            r#"{"type":"assistant","timestamp":"2026-11-07T10:01:02Z","businessModelKey":"gfmodel","message":{"id":"m2","model":"auto","usage":{"output_tokens":2,"request_id":"business"}}}"#,
        ],
    );
    let messages = assert_qoder_cache_parity(home.path(), None).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].model_id, "dfmodel");
    assert_eq!(messages[0].provider_id, "qoder");
    assert_eq!(messages[0].duration_ms, Some(2000));
    assert_eq!(messages[0].dedup_key.as_deref(), Some("qoder:qoder:req:real"));
    assert_eq!(messages[1].model_id, "gfmodel");
    assert_eq!(messages[1].dedup_key.as_deref(), Some("qoder:qoder:req:business"));
    assert_eq!(buckets(&messages), (120, 14, 0, 0));
}

#[tokio::test]
async fn test_qoder_reported_slots_beat_named_ide_session_default() {
    if rerun_with_isolated_cache("test_qoder_reported_slots_beat_named_ide_session_default") {
        return;
    }
    let home = common::temp_home();
    let db = international_ide_db(home.path());
    write_ide_request(
        &db,
        "reported",
        "{}",
        r#"{"prompt_tokens":120,"cached_tokens":20,"completion_tokens":15}"#,
    );
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE chat_session (
             session_id TEXT PRIMARY KEY, project_uri TEXT, project_name TEXT,
             preferred_model_info TEXT
         );
         INSERT INTO chat_session VALUES
             ('sess-ide', NULL, NULL, '{\"model_key\":\"openrouter/claude-sonnet-4-6\"}');
         INSERT INTO chat_message VALUES
             ('m2', 'sess-ide', 'fallback', 'assistant',
              '{\"prompt_tokens\":30,\"completion_tokens\":8}', '{}', 1794045603000);",
    )
    .unwrap();
    drop(conn);
    let pricing = PricingService::new(
        HashMap::from([(
            "openrouter/claude-sonnet-4-6".into(),
            ModelPricing {
                input_cost_per_token: Some(0.001),
                output_cost_per_token: Some(0.002),
                ..Default::default()
            },
        )]),
        HashMap::new(),
    );
    let ide_only = assert_qoder_cache_parity(home.path(), Some(&pricing)).await;
    assert_eq!(ide_only.len(), 2);
    assert!(ide_only.iter().all(|message| {
        message.model_id == "claude-sonnet-4-6"
            && message.provider_id == "openrouter"
            && message.cost_source == tokens_core::CostSource::Estimated
    }));
    assert!(has_qoder_cache_entry());

    let writer = rusqlite::Connection::open(&db).unwrap();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let assert_locked = || {
        let reader =
            rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        reader.busy_timeout(std::time::Duration::ZERO).unwrap();
        let error = reader
            .query_row("SELECT count(*) FROM chat_message", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_err();
        assert!(matches!(
            error,
            rusqlite::Error::SqliteFailure(failure, _)
                if matches!(
                    failure.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                )
        ));
    };
    assert_locked();

    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-reported.jsonl"),
        &[r#"{"type":"assistant","timestamp":"2026-11-07T10:01:00Z","message":{"model":"gfmodel","usage":{"input_tokens":100,"output_tokens":50,"request_id":"reported"}}}"#],
    );
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-fallback.jsonl"),
        &[r#"{"type":"assistant","timestamp":"2026-11-07T10:01:01Z","businessModelKey":"qmodel","message":{"model":"auto","usage":{"input_tokens":50,"output_tokens":5,"request_id":"fallback"}}}"#],
    );
    let scan = || {
        parse_local_unified_messages_with_pricing(qoder_options(home.path()), Some(&pricing))
    };
    let messages = scan().await.unwrap();
    assert_eq!(scan().await.unwrap(), messages);
    assert_locked();
    writer.execute_batch("ROLLBACK").unwrap();
    assert_eq!(
        parse_local_unified_messages_with_pricing_uncached(
            qoder_options(home.path()),
            Some(&pricing),
        )
        .await
        .unwrap(),
        messages,
    );
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].model_id, "gfmodel");
    assert_eq!(messages[1].model_id, "qmodel");
    assert!(messages.iter().all(|message| {
        message.session_id == "sess-ide"
            && message.provider_id == "qoder"
            && message.cost_source == tokens_core::CostSource::Unknown
            && message.cost == 0.0
    }));
    assert_eq!(buckets(&messages), (150, 58, 20, 0));
}

/// T20 / T21: damaged lines, absurd counters and nonsense timestamps neither
/// crash the parse nor inflate a day.
#[tokio::test]
async fn test_qoder_damaged_input_never_inflates_totals() {
    let home = common::temp_home();
    let path = home.path().join(".qoder/projects/-repo/sess-bad.jsonl");
    write_jsonl(
        &path,
        &[
            r#"{"type":"assistant","sessionId":"sess-bad","timestamp":"2026-11-07T10:00:00.000Z","message":{"id":"m1","model":"qmodel","usage":{"input_tokens":100,"output_tokens":10,"request_id":"req-1"}}}"#,
            r#"{"type":"assistant","sessionId":"sess-bad","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"m2","model":"qm"#,
            r#"{"type":"assistant","sessionId":"sess-bad","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"m3","model":"qmodel","usage":{"input_tokens":100,"output_tokens":10,"request_id":"req-1"}}}"#,
            r#"{"type":"assistant","sessionId":"sess-bad","timestamp":"2026-11-07T10:00:03.000Z","message":{"id":"m4","model":"qmodel","usage":{"input_tokens":-100,"output_tokens":-10,"cache_read_input_tokens":-5}}}"#,
            r#"{"type":"assistant","sessionId":"sess-bad","timestamp":"99999999999999999999999","message":{"id":"m5","model":"qmodel","usage":{"input_tokens":7,"output_tokens":1,"request_id":"req-5"}}}"#,
            "",
        ],
    );

    // November 8 at 00:00 UTC, deliberately outside the valid row's day.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_794_096_000))
        .unwrap();
    let mut options = qoder_options(home.path());
    options.since = Some("2026-11-07".into());
    options.until = Some("2026-11-08".into());
    let messages = parse_local_unified_messages_with_pricing_uncached(options, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(total_tokens(&messages), 110 + 8);
    assert_eq!(messages[0].date, "2026-11-07");
    assert_eq!(messages[0].tokens.total(), 110);
    assert_eq!(messages[1].date, "2026-11-08");
    assert_eq!(messages[1].timestamp, 1_794_096_000_000);
    assert_eq!(messages[1].tokens.total(), 8);
}

/// T23: explicit CLI and IDE roots preserve edition even without path markers.
#[tokio::test]
async fn test_qoder_config_dir_overrides_are_honoured() {
    const TEST: &str = "test_qoder_config_dir_overrides_are_honoured";
    const EXPECTED_KEY: &str = "TOKENS_QODER_TEST_EXPECTED_KEY";
    if is_qoder_test_child(TEST) {
        let home = PathBuf::from(std::env::var_os("HOME").expect("fixture home"));
        let expected_key = std::env::var(EXPECTED_KEY).expect("expected edition request key");
        assert_qoder_override_scan(&home, &expected_key).await;
        return;
    }

    let home = common::temp_home();
    let intl = home.path().join("redirected-a");
    let cn = home.path().join("redirected-b");
    let intl_ide = intl.join("desktop");
    let cn_ide = cn.join("desktop/SharedClientCache");

    write_jsonl(
        &intl.join("projects/-repo/sess-intl.jsonl"),
        &[INTL_ONE_REQUEST],
    );
    write_jsonl(
        &cn.join("projects/-repo/sess-cn.jsonl"),
        &[&CN_ONE_REQUEST.replace("req-cn-1", "req-intl-1")],
    );
    for root in [&intl, &cn] {
        write_jsonl(
            &root.join("logs/sessions/-repo/segment-only/segments/run.jsonl"),
            &[r#"{"ts":"2026-11-07T12:00:00Z","type":"model.response.completed","request_id":"same-segment-id","data":{"model":"qmodel","input_tokens":50,"output_tokens":5}}"#],
        );
    }
    write_ide_request(
        &intl_ide.join("SharedClientCache/cache/db/local.db"),
        "req-intl-1",
        r#"{"model_key":"qmodel"}"#,
        r#"{"prompt_tokens":1200,"cached_tokens":800,"completion_tokens":45}"#,
    );
    write_ide_request(
        &cn_ide.join("cache/db/local.db"),
        "req-intl-1",
        r#"{"model_key":"dfmodel"}"#,
        r#"{"prompt_tokens":1000,"cached_tokens":600,"completion_tokens":25}"#,
    );
    // Replaced roots must not also discover the ordinary installation.
    write_jsonl(
        &home.path().join(".qoder-cn/projects/-repo/ignored.jsonl"),
        &[CN_ONE_REQUEST],
    );

    // The second process scans the same files against the first process's cache.
    // Only scanner-assigned editions change.
    for (cli_roots, ide_roots, expected_key) in [
        (
            [&intl, &cn],
            [&intl_ide, &cn_ide],
            "qoder:qoder-cn:req:req-intl-1",
        ),
        (
            [&cn, &intl],
            [&cn_ide, &intl_ide],
            "qoder:qoder:req:req-intl-1",
        ),
    ] {
        run_qoder_test_child(
            qoder_test_command(TEST, home.path())
                .env("QODER_CONFIG_DIR", cli_roots[0])
                .env("QODERCN_CONFIG_DIR", cli_roots[1])
                .env("QODER_HOME", ide_roots[0])
                .env("QODER_CN_HOME", ide_roots[1])
                .env(EXPECTED_KEY, expected_key),
        );
    }
}

async fn assert_qoder_override_scan(home: &Path, expected_key: &str) {
    let mut options = qoder_options(home);
    options.use_env_roots = true;
    let scan = scan_all_clients_with_scanner_settings(
        home.to_str().unwrap(),
        &["qoder".to_string()],
        true,
        &options.scanner_settings,
    );
    assert_eq!(scan.get(ClientId::Qoder).len(), 4);
    assert_eq!(scan.qoder_ide_dbs.len(), 2);

    let messages = parse_local_unified_messages_with_pricing(options.clone(), None)
        .await
        .unwrap();
    assert!(
        tokens_core::paths::get_cache_dir().is_dir(),
        "persistent parse must create the isolated cache"
    );
    assert_eq!(messages.len(), 4);
    assert_eq!(buckets(&messages), (900, 80, 1400, 100));
    assert!(messages.iter().any(|message| {
        message.model_id == "dfmodel"
            && message.dedup_key.as_deref() == Some(expected_key)
            && message.tokens.output == 25
    }));
    let uncached = parse_local_unified_messages_with_pricing_uncached(options, None)
        .await
        .unwrap();
    assert_eq!(messages, uncached);
}

/// A Qoder-only filter must not pick up other clients, and other clients must
/// not pick up Qoder (T24 at the core level; the CLI enum is tested in
/// `tokens-cli`).
#[tokio::test]
async fn test_qoder_filter_is_isolated() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(".qoder/projects/-repo/sess-1.jsonl"),
        &[INTL_ONE_REQUEST],
    );
    write_jsonl(
        &home
            .path()
            .join(".qwen/projects/-repo/chats/sess-qwen.jsonl"),
        &[
            r#"{"type":"assistant","model":"qwen3.5-plus","timestamp":"2026-11-07T14:24:56.857Z","sessionId":"sess-qwen","usageMetadata":{"promptTokenCount":100,"candidatesTokenCount":10}}"#,
        ],
    );

    let qoder_only = parse_uncached(home.path()).await;
    assert_eq!(qoder_only.len(), 1);
    assert_eq!(qoder_only[0].client, "qoder");

    let qwen_only = parse_local_unified_messages_with_pricing_uncached(
        LocalParseOptions {
            clients: Some(vec!["qwen".to_string()]),
            ..qoder_options(home.path())
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(qwen_only.len(), 1);
    assert_eq!(qwen_only[0].client, "qwen");
}

/// A home with no Qoder data at all is an empty source, not an error.
#[tokio::test]
async fn test_qoder_absent_installation_degrades_to_empty() {
    let home = common::temp_home();
    let messages = parse_uncached(home.path()).await;
    assert!(messages.is_empty());
    let parsed = parse_local_clients(qoder_options(home.path())).unwrap();
    assert_eq!(parsed.counts.get(ClientId::Qoder), 0);
}
