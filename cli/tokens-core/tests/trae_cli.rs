//! TRAE CLI 2.0 rollouts end to end: discovered under `~/.trae/cli`, parsed
//! through the Codex parser, and reported as `trae-cli` rather than `codex`.

mod common;

use std::path::Path;

use tokens_core::scanner::{scan_all_clients_with_scanner_settings, ScannerSettings};
use tokens_core::sessions::trae_cli::parse_trae_cli_file;
use tokens_core::{
    parse_local_clients, parse_local_unified_messages_with_pricing_uncached, ClientId,
    LocalParseOptions, UnifiedMessage,
};

// Shapes from the TRAE CLI 0.200.19 fixtures in kenn-io/agentsview
// (`internal/parser/testdata/traex/`): a `token_count` rollout and a newer
// `history_mutation` rollout that carries usage in `token_usage_record`.
const TOKEN_COUNT_ROLLOUT: &[&str] = &[
    r#"{"timestamp":"2026-08-01T18:07:04.550Z","type":"session_meta","payload":{"id":"019fbcca-9fd4-7d20-83dc-0762b2f839b3","timestamp":"2026-08-01T18:07:03.636Z","cwd":"/home/user/code/api","originator":"codex-tui","cli_version":"0.200.19","model_provider":"trae"}}"#,
    r#"{"timestamp":"2026-08-01T18:07:04.570Z","type":"turn_context","payload":{"turn_id":"turn-1","cwd":"/home/user/code/api","model":"gpt-5-codex","model_provider":"trae"}}"#,
    r#"{"timestamp":"2026-08-01T18:07:04.590Z","type":"event_msg","payload":{"type":"user_message","message":"Explore the parser package."}}"#,
    r#"{"timestamp":"2026-08-01T18:07:14.100Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":4200,"cached_input_tokens":3600,"output_tokens":180,"reasoning_output_tokens":64,"total_tokens":4380},"total_token_usage":{"input_tokens":4200,"cached_input_tokens":3600,"output_tokens":180,"reasoning_output_tokens":64,"total_tokens":4380},"model_context_window":272000}}}"#,
];

const HISTORY_MUTATION_ROLLOUT: &[&str] = &[
    r#"{"timestamp":"2026-09-01T10:00:00.000Z","type":"session_meta","payload":{"id":"019fcf70-1111-7000-8000-000000000001","timestamp":"2026-09-01T10:00:00.000Z","cwd":"/home/user/code/app","originator":"codex-tui","cli_version":"0.200.19","model_provider":"trae"}}"#,
    r#"{"timestamp":"2026-09-01T10:00:01.000Z","type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5-codex","model_provider":"trae"}}"#,
    r#"{"timestamp":"2026-09-01T10:00:02.000Z","type":"history_mutation","payload":{"operation":"append","items":[{"type":"message","role":"user","content":[{"type":"input_text","text":"Inspect the parser."}]}]}}"#,
    r#"{"timestamp":"2026-09-01T10:00:03.000Z","type":"token_usage_record","payload":{"response_id":"resp_1","usage":{"input_tokens":100,"cached_input_tokens":40,"cache_creation_input_tokens":0,"output_tokens":12,"reasoning_output_tokens":2,"total_tokens":112}}}"#,
];

fn options(home: &Path, client: &str) -> LocalParseOptions {
    LocalParseOptions {
        home_dir: Some(home.to_string_lossy().into_owned()),
        use_env_roots: false,
        clients: Some(vec![client.to_string()]),
        since: None,
        until: None,
        year: None,
        scanner_settings: ScannerSettings::default(),
    }
}

fn write_jsonl(path: &Path, lines: &[&str]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, lines.join("\n") + "\n").unwrap();
}

fn total_tokens(messages: &[UnifiedMessage]) -> i64 {
    messages.iter().map(|message| message.tokens.total()).sum()
}

#[tokio::test]
async fn trae_cli_rollouts_are_counted_under_their_own_client() {
    let home = common::temp_home();
    write_jsonl(
        &home.path().join(
            ".trae/cli/sessions/2026/08/01/rollout-2026-08-01T18-07-04-019fbcca-9fd4-7d20-83dc-0762b2f839b3.jsonl",
        ),
        TOKEN_COUNT_ROLLOUT,
    );
    write_jsonl(
        &home.path().join(
            ".trae/cli/archived_sessions/rollout-2026-09-01T10-00-00-019fcf70-1111-7000-8000-000000000001.jsonl",
        ),
        HISTORY_MUTATION_ROLLOUT,
    );
    // Prompt history beside the rollouts carries no usage and is not scanned.
    write_jsonl(
        &home.path().join(".trae/cli/history.jsonl"),
        &[
            r#"{"session_id":"019fbcca-9fd4-7d20-83dc-0762b2f839b3","ts":1785561000,"text":"Explore the parser package."}"#,
        ],
    );

    let scan = scan_all_clients_with_scanner_settings(
        home.path().to_str().unwrap(),
        &["trae-cli".to_string()],
        false,
        &ScannerSettings::default(),
    );
    assert_eq!(scan.get(ClientId::TraeCli).len(), 2);

    let parsed = parse_local_clients(options(home.path(), "trae-cli")).unwrap();
    assert_eq!(parsed.counts.get(ClientId::TraeCli), 2);

    let messages =
        parse_local_unified_messages_with_pricing_uncached(options(home.path(), "trae-cli"), None)
            .await
            .expect("uncached local parse");
    assert_eq!(messages.len(), 2);
    assert!(messages.iter().all(|message| message.client == "trae-cli"));
    assert!(messages
        .iter()
        .all(|message| message.model_id == "gpt-5-codex"));
    assert_eq!(total_tokens(&messages), 4380 + 112);

    // The rollouts live outside `~/.codex`, so they never count as Codex.
    let codex =
        parse_local_unified_messages_with_pricing_uncached(options(home.path(), "codex"), None)
            .await
            .expect("uncached codex parse");
    assert!(codex.is_empty());
}

#[test]
fn only_appended_human_input_marks_a_turn() {
    let home = common::temp_home();
    let path = home.path().join("rollout-mutations.jsonl");
    let replacement = HISTORY_MUTATION_ROLLOUT[2].replace(r#""append""#, r#""replace""#);
    let second = HISTORY_MUTATION_ROLLOUT[3].replace("resp_1", "resp_2");
    let third = HISTORY_MUTATION_ROLLOUT[3].replace("resp_1", "resp_3");
    let mut lines = HISTORY_MUTATION_ROLLOUT.to_vec();
    lines.extend([
        replacement.as_str(),
        second.as_str(),
        HISTORY_MUTATION_ROLLOUT[2],
        third.as_str(),
    ]);
    write_jsonl(&path, &lines);

    let messages = parse_trae_cli_file(&path);
    assert_eq!(
        messages
            .iter()
            .map(|message| (message.is_turn_start, message.tokens.total()))
            .collect::<Vec<_>>(),
        [(true, 112), (false, 112), (true, 112)]
    );
}

#[tokio::test]
async fn usage_only_fork_drops_parent_replay_without_losing_own_usage() {
    let home = common::temp_home();
    let root = home.path().join(".trae/cli/archived_sessions");
    let parent = [
        HISTORY_MUTATION_ROLLOUT[0],
        r#"{"type":"turn_context","payload":{"turn_id":"019fcf70-1112-7000-8000-000000000001","model":"gpt-5-codex"}}"#,
        HISTORY_MUTATION_ROLLOUT[2],
        HISTORY_MUTATION_ROLLOUT[3],
    ];
    write_jsonl(
        &root.join("rollout-2026-09-01T10-00-00-019fcf70-1111-7000-8000-000000000001.jsonl"),
        &parent,
    );
    let own_usage = HISTORY_MUTATION_ROLLOUT[3].replace("resp_1", "resp_child");
    let mut child = vec![
        r#"{"type":"session_meta","payload":{"id":"019fcf70-2222-7000-8000-000000000001","forked_from_id":"019fcf70-1111-7000-8000-000000000001","thread_source":"user","model_provider":"trae","cwd":"/synthetic/child"}}"#,
    ];
    child.extend(parent);
    child.extend([
        r#"{"type":"turn_context","payload":{"turn_id":"019fcf70-2223-7000-8000-000000000001","model":"gpt-5-codex"}}"#,
        HISTORY_MUTATION_ROLLOUT[3],
        own_usage.as_str(),
        HISTORY_MUTATION_ROLLOUT[2],
        r#"{"timestamp":"2026-09-01T10:00:05Z","type":"token_usage_record","payload":{"response_id":"resp_next","usage":{"input_tokens":20,"output_tokens":2}}}"#,
    ]);
    write_jsonl(
        &root.join("rollout-2026-09-01T10-00-04-019fcf70-2222-7000-8000-000000000001.jsonl"),
        &child,
    );

    let messages =
        parse_local_unified_messages_with_pricing_uncached(options(home.path(), "trae-cli"), None)
            .await
            .unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(total_tokens(&messages), 246);
    let mut child_turns: Vec<_> = messages
        .iter()
        .filter(|message| message.workspace_key.as_deref() == Some("/synthetic/child"))
        .map(|message| (message.tokens.total(), message.is_turn_start))
        .collect();
    child_turns.sort_unstable();
    assert_eq!(child_turns, [(22, true), (112, false)]);
}

#[test]
fn forked_trae_rollout_uses_only_the_childs_usage_dialect() {
    let home = common::temp_home();
    let parent_snapshot = r#"{"timestamp":"2026-09-01T10:00:03.100Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":12,"total_tokens":112},"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":12,"total_tokens":112}}}}"#;
    let trailing_snapshot = parent_snapshot.replace("10:00:03.100Z", "10:00:07.000Z");
    let parent_empty_total = r#"{"timestamp":"2026-09-01T10:00:03.200Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{},"last_token_usage":{"input_tokens":80,"output_tokens":6,"total_tokens":86}}}}"#;
    let trailing_empty_total = parent_empty_total.replace("10:00:03.200Z", "10:00:06.900Z");
    let prefix = [
        r#"{"timestamp":"2026-09-01T10:00:04.000Z","type":"session_meta","payload":{"id":"019fcf70-2222-7000-8000-000000000001","forked_from_id":"019fcf70-1111-7000-8000-000000000001","thread_source":"user","model_provider":"trae"}}"#,
        HISTORY_MUTATION_ROLLOUT[0],
        r#"{"timestamp":"2026-09-01T10:00:01.000Z","type":"turn_context","payload":{"turn_id":"019fcf70-1112-7000-8000-000000000001","model":"gpt-5.4"}}"#,
        HISTORY_MUTATION_ROLLOUT[2],
        HISTORY_MUTATION_ROLLOUT[3],
        parent_snapshot,
        parent_empty_total,
        r#"{"timestamp":"2026-09-01T10:00:04.100Z","type":"turn_context","payload":{"turn_id":"019fcf70-2223-7000-8000-000000000001","model":"gpt-5-codex"}}"#,
        r#"{"timestamp":"2026-09-01T10:00:05.000Z","type":"event_msg","payload":{"type":"user_message","message":"Continue in the fork."}}"#,
        r#"{"timestamp":"2026-09-01T10:00:06.000Z","type":"token_usage_record","payload":{"response_id":"resp_child","usage":{"input_tokens":20,"output_tokens":2,"total_tokens":22}}}"#,
        r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{}}}}"#,
        trailing_empty_total.as_str(),
        trailing_snapshot.as_str(),
        HISTORY_MUTATION_ROLLOUT[3],
    ];
    let child_snapshot = r#"{"timestamp":"2026-09-01T10:00:08.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":120,"cached_input_tokens":40,"output_tokens":14,"total_tokens":134},"last_token_usage":{"input_tokens":20,"output_tokens":2,"total_tokens":22}}}}"#;
    let zero_snapshot = child_snapshot.replace(
        r#""last_token_usage":{"input_tokens":20,"output_tokens":2,"total_tokens":22}"#,
        r#""last_token_usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}"#,
    );

    for (name, snapshot, expected_key_prefix) in [
        (
            "inherited",
            None,
            Some("trae-cli:response:019fcf70-2222-7000-8000-000000000001:resp_child"),
        ),
        (
            "owned",
            Some(child_snapshot),
            Some("trae-cli:token_count-total:"),
        ),
        ("owned-zero", Some(zero_snapshot.as_str()), None),
    ] {
        let path = home.path().join(format!("rollout-{name}.jsonl"));
        let mut lines = prefix.to_vec();
        lines.extend(snapshot);
        write_jsonl(&path, &lines);

        let messages = parse_trae_cli_file(&path);
        if let Some(key_prefix) = expected_key_prefix {
            assert_eq!(messages.len(), 1, "{name}");
            assert_eq!(messages[0].tokens.total(), 22, "{name}");
            assert_eq!(messages[0].model_id, "gpt-5-codex", "{name}");
            assert!(messages[0].is_turn_start, "{name}");
            assert!(
                messages[0]
                    .dedup_key
                    .as_deref()
                    .is_some_and(|key| key.starts_with(key_prefix)),
                "{name}"
            );
        } else {
            assert!(messages.is_empty(), "{name}");
        }
    }
}

#[test]
fn fork_without_inherited_totals_counts_its_own_last_usage() {
    let home = common::temp_home();
    let path = home.path().join("rollout-fork-empty-total.jsonl");
    write_jsonl(
        &path,
        &[
            r#"{"type":"session_meta","payload":{"id":"019fcf70-2222-7000-8000-000000000001","forked_from_id":"019fcf70-1111-7000-8000-000000000001","thread_source":"user","model_provider":"trae"}}"#,
            HISTORY_MUTATION_ROLLOUT[0],
            r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{},"last_token_usage":{"input_tokens":100,"output_tokens":12}}}}"#,
            r#"{"type":"turn_context","payload":{"turn_id":"019fcf70-2223-7000-8000-000000000001","model":"gpt-5-codex"}}"#,
            r#"{"timestamp":"2026-09-01T10:00:05Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{},"last_token_usage":{"input_tokens":20,"output_tokens":2}}}}"#,
        ],
    );

    let messages = parse_trae_cli_file(&path);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].tokens.input, 20);
    assert_eq!(messages[0].tokens.output, 2);
}

#[test]
fn trae_usage_records_yield_only_to_a_usable_token_count() {
    let home = common::temp_home();
    for (name, info, expected) in [
        ("null", "null", 112),
        ("metadata-only", r#"{"model_context_window":272000}"#, 112),
        (
            "no-counters",
            r#"{"last_token_usage":{},"total_token_usage":{"input_tokens":null}}"#,
            112,
        ),
        (
            "invalid-counters",
            r#"{"last_token_usage":{"input_tokens":-1,"cache_write_input_tokens":"invalid"}}"#,
            112,
        ),
        (
            "positive-total-only",
            r#"{"total_token_usage":{"total_tokens":900}}"#,
            112,
        ),
        (
            "positive-last-aggregate-only",
            r#"{"last_token_usage":{"total_tokens":90}}"#,
            112,
        ),
        (
            "unbucketable-positive-subsets",
            r#"{"last_token_usage":{"cached_input_tokens":40,"reasoning_output_tokens":2}}"#,
            112,
        ),
        (
            "explicit-zero-aggregate-only",
            r#"{"total_token_usage":{"total_tokens":0}}"#,
            0,
        ),
        (
            "explicit-zero",
            r#"{"last_token_usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}}"#,
            0,
        ),
    ] {
        let path = home.path().join(format!("rollout-{name}.jsonl"));
        let event = format!(
            r#"{{"timestamp":"2026-09-01T10:00:02.500Z","type":"event_msg","payload":{{"type":"token_count","info":{info}}}}}"#
        );
        let mut lines = HISTORY_MUTATION_ROLLOUT.to_vec();
        lines.insert(3, &event);
        write_jsonl(&path, &lines);

        let messages = parse_trae_cli_file(&path);
        assert_eq!(messages.len(), usize::from(expected != 0), "{name}");
        assert_eq!(total_tokens(&messages), expected, "{name}");
    }
}

#[test]
fn trae_response_usage_splits_inclusive_and_disjoint_cache_buckets() {
    let home = common::temp_home();
    let path = home.path().join("rollout-cache-buckets.jsonl");
    write_jsonl(
        &path,
        &[
            HISTORY_MUTATION_ROLLOUT[0],
            HISTORY_MUTATION_ROLLOUT[1],
            r#"{"type":"token_usage_record","payload":{"response_id":"inclusive","usage":{"input_tokens":100,"cached_input_tokens":40,"cache_creation_input_tokens":10,"output_tokens":12,"reasoning_output_tokens":2,"total_tokens":112}}}"#,
            r#"{"type":"token_usage_record","payload":{"response_id":"disjoint","usage":{"input_tokens":100,"cache_read_input_tokens":40,"cache_creation_input_tokens":10,"output_tokens":12,"reasoning_output_tokens":2,"total_tokens":162}}}"#,
            r#"{"type":"token_usage_record","payload":{"response_id":"separate-write","usage":{"input_tokens":100,"cached_input_tokens":40,"cache_creation_input_tokens":10,"output_tokens":12,"reasoning_output_tokens":2,"total_tokens":122}}}"#,
        ],
    );

    let messages = parse_trae_cli_file(&path);
    assert_eq!(
        messages
            .iter()
            .map(|message| {
                let tokens = &message.tokens;
                [
                    tokens.input,
                    tokens.cache_read,
                    tokens.cache_write,
                    tokens.output,
                    tokens.reasoning,
                ]
            })
            .collect::<Vec<_>>(),
        [[50, 40, 10, 10, 2], [100, 40, 10, 10, 2], [60, 40, 10, 10, 2]]
    );
    assert_eq!(total_tokens(&messages), 396);
}

#[test]
fn trae_token_count_preserves_cache_write_only_delta() {
    let home = common::temp_home();
    let path = home.path().join("rollout-cache-delta.jsonl");
    write_jsonl(
        &path,
        &[
            TOKEN_COUNT_ROLLOUT[0],
            TOKEN_COUNT_ROLLOUT[1],
            r#"{"timestamp":"2026-08-01T18:07:05Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"cache_creation_input_tokens":10,"output_tokens":12,"total_tokens":112}}}}"#,
            r#"{"timestamp":"2026-08-01T18:07:06Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":106,"cached_input_tokens":40,"cache_creation_input_tokens":16,"output_tokens":12,"total_tokens":118}}}}"#,
        ],
    );

    let messages = parse_trae_cli_file(&path);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].tokens.input, 50);
    assert_eq!(messages[0].tokens.cache_write, 10);
    assert_eq!(messages[1].tokens.input, 0);
    assert_eq!(messages[1].tokens.cache_write, 6);
    assert_eq!(total_tokens(&messages), 118);
}

#[test]
fn damaged_json_and_truncated_tail_keep_complete_responses_once() {
    let home = common::temp_home();
    let path = home.path().join("rollout-damaged.jsonl");
    let archive = path.with_extension("jsonl.zst");
    let zero = r#"{"type":"token_usage_record","payload":{"response_id":"resp_1","usage":{"input_tokens":0}}}"#;
    let valid = HISTORY_MUTATION_ROLLOUT[3];
    let prefix = format!(
        "{}\n{zero}\n{{bad json}}\n",
        HISTORY_MUTATION_ROLLOUT[..3].join("\n")
    );
    let tail = format!("{valid}\n{valid}\n{{\"type\":\"token_usage_record\",\"payload\":");
    let contents = [prefix.as_bytes(), &b"\xff\n"[..], tail.as_bytes()].concat();
    std::fs::write(&path, &contents).unwrap();
    std::fs::write(&archive, zstd::encode_all(contents.as_slice(), 0).unwrap()).unwrap();

    for path in [path, archive] {
        let messages = parse_trae_cli_file(&path);
        assert_eq!(messages.len(), 1);
        assert_eq!(total_tokens(&messages), 112);
        assert!(messages[0].is_turn_start);
    }
}

#[test]
fn damaged_zstd_frame_rejects_even_the_valid_archive_prefix() {
    let home = common::temp_home();
    let path = home.path().join("rollout-corrupt.jsonl.zst");
    let rollout = HISTORY_MUTATION_ROLLOUT.join("\n") + "\n";
    let prefix = zstd::encode_all(rollout.as_bytes(), 0).unwrap();
    std::fs::write(&path, &prefix).unwrap();
    assert_eq!(total_tokens(&parse_trae_cli_file(&path)), 112);

    let mut truncated = zstd::encode_all(rollout.as_bytes(), 0).unwrap();
    truncated.pop().unwrap();
    for tail in [truncated.as_slice(), &b"not a zstd frame"[..]] {
        std::fs::write(&path, [prefix.as_slice(), tail].concat()).unwrap();
        assert!(parse_trae_cli_file(&path).is_empty());
    }
}
