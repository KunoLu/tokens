//! kwwk session parser
//!
//! kwwk (https://github.com/yellowplushq/kwwk) is a Swift port of pi and keeps
//! each session as a pi-format JSONL transcript: a `session` header, then
//! `message` records whose assistant turns carry `usage.input / output /
//! cacheRead / cacheWrite`, `model`, `provider` and `responseId`. The shared
//! `super::pi::parse_pi_format_file` parser reads kwwk's usage extensions only
//! for this client.
//!
//! kwwk entries store an `Int64` timestamp in Unix milliseconds. They carry
//! no entry `id`, so assistant turns deduplicate on `message.responseId`.
//!
//! A `compaction` record's top-level `usage` accounts for summary requests
//! absent from the transcript. `replacementMessages` only projects prior
//! context for replay and must not be counted again.
//! The writer has no compaction response ID. Its marker identity uses the
//! session ID, recorded time and occurrence at that time. Neither a rewrite
//! of `replacementMessages` nor repair of an earlier entry changes this key.
//!
//! Layout: the kwwk CLI writes `~/.kwwk/sessions/<session-id>.jsonl`. Programs
//! embedding kwwk can keep sessions anywhere; a bot running one agent per task
//! uses `<tasks>/<task>/.kwwk-sessions/<session-id>.jsonl`. Point an extra scan
//! root at either directory (or at `<tasks>`); the `kwwk-session` pattern only
//! admits files directly in the root or inside a `.kwwk-sessions` directory, so
//! the code checkouts that share a task directory are not read.

use super::pi::parse_pi_format_file;
use super::UnifiedMessage;
use std::path::Path;

pub fn parse_kwwk_file(path: &Path) -> Vec<UnifiedMessage> {
    parse_pi_format_file(path, "kwwk", "kwwk")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::Options;
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Shapes copied from a real kwwk session (content replaced): epoch-ms
    // timestamps, no entry ids, a compaction record and an aborted turn.
    const SESSION: &str = r#"{"type":"session","version":1,"id":"66ff7a63-f706-405b-ba34-e5b4b437f29c","createdAt":1795279000000,"cwd":"/work/tasks/T0555","model":"claude-opus-5-5","provider":"anthropic"}
{"type":"message","timestamp":1795279500000,"message":{"role":"user","content":[{"type":"text","text":"hi"}],"timestamp":1795279500000}}
{"type":"message","timestamp":1795279528810,"message":{"role":"assistant","content":[{"type":"text","text":"ok"}],"api":"anthropic-messages","provider":"anthropic","model":"claude-opus-5-5","responseId":"msg_01A","stopReason":"toolUse","timestamp":1795279528810,"usage":{"input":4,"output":294,"cacheRead":3262,"cacheWrite":8485,"cacheWrite1h":0,"reasoning":17,"totalTokens":12045,"cost":{"input":0.000016,"output":0.00588,"cacheRead":0.00065,"cacheWrite":0.042425,"total":0.04897}}}}
{"type":"compaction","timestamp":1795279600000,"tokensBefore":137732,"replacementMessages":[{"role":"assistant","content":[],"api":"anthropic-messages","provider":"anthropic","model":"claude-opus-5-5","responseId":"msg_01A","timestamp":1795279528810,"usage":{"input":4,"output":294,"cacheRead":3262,"cacheWrite":8485,"totalTokens":12045}}]}
{"type":"message","timestamp":1795279700123,"message":{"role":"assistant","content":[{"type":"text","text":"done"}],"api":"anthropic-messages","provider":"anthropic","model":"claude-opus-5-5","responseId":"msg_01B","stopReason":"stop","timestamp":1795279700123,"usage":{"input":10,"output":50,"cacheRead":15000,"cacheWrite":0,"totalTokens":15060}}}
"#;

    fn session_file(body: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(body.as_bytes()).unwrap();
        file
    }

    #[test]
    fn parses_kwwk_sessions_with_epoch_millisecond_timestamps() {
        let file = session_file(SESSION);
        let messages = parse_kwwk_file(file.path());

        assert_eq!(messages.len(), 2, "two assistant turns; the compaction copy is not counted");
        let first = &messages[0];
        assert_eq!(first.client, "kwwk");
        assert_eq!(first.session_id, "66ff7a63-f706-405b-ba34-e5b4b437f29c");
        assert_eq!(first.provider_id, "anthropic");
        assert_eq!(first.model_id, "claude-opus-5-5");
        assert_eq!(first.timestamp, 1_795_279_528_810);
        assert_eq!(first.tokens.input, 4);
        assert_eq!(first.tokens.output, 294);
        assert_eq!(first.tokens.cache_read, 3262);
        assert_eq!(first.tokens.cache_write, 8485);
        assert_eq!(messages[1].timestamp, 1_795_279_700_123);
        assert_eq!(messages[1].tokens.cache_read, 15000);
    }

    #[test]
    fn kwwk_accepts_signed_integer_milliseconds_without_a_date_floor() {
        let cases = [
            ("-1", Some(-1)),
            ("0", Some(0)),
            ("1", Some(1)),
            ("1795279700", Some(1_795_279_700)),
            ("1795279700123", Some(1_795_279_700_123)),
            ("1795279700123.5", None),
            ("9223372036854775807", None),
            ("18446744073709551615", None),
            (r#""1795279700123""#, None),
        ];
        let mut file = session_file(&format!("{}\n", SESSION.lines().next().unwrap()));
        for (id, (timestamp, _)) in cases.iter().enumerate() {
            writeln!(
                file,
                r#"{{"type":"message","timestamp":{timestamp},"message":{{"role":"assistant","model":"claude-sonnet-5","responseId":"msg_{id}","usage":{{"input":3}}}}}}"#,
            )
            .unwrap();
        }

        let fallback = super::super::utils::file_modified_timestamp_ms(file.path());
        let messages = parse_kwwk_file(file.path());
        assert_eq!(messages.len(), cases.len());
        for (message, (wire, timestamp)) in messages.iter().zip(cases) {
            assert_eq!(message.timestamp, timestamp.unwrap_or(fallback), "{wire}");
        }
    }

    #[test]
    fn kwwk_records_dedupe_on_response_id() {
        let file = session_file(SESSION);
        let messages = parse_kwwk_file(file.path());
        let keys: Vec<_> = messages.iter().map(|m| m.dedup_key.clone().unwrap_or_default()).collect();
        assert!(keys[0].contains("msg_01A"), "{keys:?}");
        assert!(keys[1].contains("msg_01B"), "{keys:?}");
        assert_ne!(keys[0], keys[1]);
    }

    #[test]
    fn kwwk_compaction_counts_only_top_level_usage() {
        let summaries = r#"{"type":"compaction","timestamp":1795279800001,"messagesCompacted":2,"model":"claude-sonnet-5","provider":"anthropic","usage":{"input":8,"output":3,"cacheWrite":7,"cacheWrite1h":5,"cost":{"total":3}},"replacementMessages":[{"role":"assistant","model":"claude-opus-5-5","provider":"anthropic","responseId":"replacement","usage":{"input":9999,"output":12}}]}
{"type":"compaction","timestamp":1795279800002,"messagesCompacted":1,"usage":{"input":2,"output":1},"replacementMessages":[]}
"#;
        let file = session_file(&format!("{SESSION}{summaries}"));
        let messages = parse_kwwk_file(file.path());

        assert_eq!(messages.len(), 4);
        let summary = &messages[2];
        assert_eq!(summary.session_id, messages[0].session_id);
        assert_eq!(summary.model_id, "claude-sonnet-5");
        assert_eq!(summary.provider_id, "anthropic");
        assert_eq!(summary.timestamp, 1_795_279_800_001);
        assert_eq!(summary.tokens.input, 8);
        assert_eq!(summary.tokens.output, 3);
        assert_eq!(summary.tokens.cache_write, 7);
        assert_eq!(summary.cache_write_1h, 5);
        assert!(!summary.has_authoritative_cost());
        assert_eq!(messages[3].model_id, "unknown");
        assert_eq!(messages[3].tokens.input, 2);
        assert_eq!(super::super::pi::parse_pi_file(file.path()).len(), 2);
    }

    #[test]
    fn kwwk_compaction_dedup_survives_copy_and_projection_rewrite() {
        let marker = r#"{"type":"compaction","timestamp":1795279800001,"messagesCompacted":2,"usage":{"input":8,"output":3},"replacementMessages":[{"role":"user","timestamp":1795279800000,"content":[{"type":"text","text":"before"}]}]}"#;
        let file = session_file(&format!("{SESSION}corrupt entry\n{marker}\n{marker}\n"));
        let messages = parse_kwwk_file(file.path());
        assert_eq!(messages.len(), 4);
        let mut seen = HashSet::new();
        for message in &messages[2..] {
            assert!(message.dedup_key.is_some());
            assert!(crate::should_keep_deduped_message(&mut seen, message));
        }

        let rewritten = marker.replace("\"before\"", "\"after\"");
        for (prefix, marker) in [("corrupt entry\n", marker), ("", rewritten.as_str())] {
            let copy = session_file(&format!("{SESSION}{prefix}{marker}\n"));
            let copied = parse_kwwk_file(copy.path());
            assert_eq!(copied.len(), 3);
            assert!(!crate::should_keep_deduped_message(&mut seen, &copied[2]));
        }

        let other = SESSION.replace(
            "66ff7a63-f706-405b-ba34-e5b4b437f29c",
            "17f34dfb-b30d-4dc4-970c-21e88b0df56e",
        );
        let file = session_file(&format!("{other}{marker}\n"));
        let distinct = parse_kwwk_file(file.path());
        assert_eq!(distinct.len(), 3);
        assert!(crate::should_keep_deduped_message(&mut seen, &distinct[2]));
    }

    #[test]
    fn empty_usage_does_not_shift_same_millisecond_compaction_ids() {
        let header = SESSION.lines().next().unwrap();
        let first = r#"{"type":"compaction","timestamp":1,"usage":{"input":20}}"#;
        let second = r#"{"type":"compaction","timestamp":1,"usage":{"input":3}}"#;
        let empty = first.replace("20", "0");
        let partial = session_file(&format!("{header}\n{empty}\n{second}\n"));
        let complete = session_file(&format!("{header}\n{first}\n{second}\n"));
        let mut seen = HashSet::new();
        let total: i64 = [partial, complete]
            .iter()
            .flat_map(|file| parse_kwwk_file(file.path()))
            .filter(|message| crate::should_keep_deduped_message(&mut seen, message))
            .map(|message| message.tokens.input)
            .sum();
        assert_eq!(total, 23);
    }

    #[test]
    fn removing_a_malformed_marker_does_not_recount_later_usage() {
        let header = SESSION.lines().next().unwrap();
        let marker = r#"{"type":"compaction","timestamp":1,"usage":{"input":3}}"#;
        let damaged = marker.replace(":3", ":\"bad\"");
        let old = session_file(&format!("{header}\n{damaged}\n{marker}\n"));
        let recovered = session_file(&format!("{header}\n{marker}\n"));
        let mut seen = HashSet::new();
        let inputs: Vec<_> = [old, recovered]
            .iter()
            .flat_map(|file| parse_kwwk_file(file.path()))
            .filter(|row| crate::should_keep_deduped_message(&mut seen, row))
            .map(|row| row.tokens.input)
            .collect();
        assert_eq!(inputs, [3]);
    }

    #[test]
    fn kwwk_compaction_skips_empty_usage_and_damaged_counter_keys() {
        let summaries = r#"{"type":"compaction","timestamp":1795279800001,"messagesCompacted":1,"replacementMessages":[],"usage":{}}
{"type":"compaction","timestamp":1795279800002,"messagesCompacted":1,"replacementMessages":[],"usage":{"input":-1,"output":0,"cacheWrite":-10}}
{"type":"compaction","timestamp":1795279800003,"messagesCompacted":1,"replacementMessages":[],"usage":{"in\uFFFDput":100,"output":5}}
{"type":"compaction","timestamp":1795279800004,"messagesCompacted":1,"replacementMessages":[],"usage":{"future\uFFFD-field":100,"output":7,"cacheWrite":-10,"cacheWrite1h":5}}
"#;
        let file = session_file(&format!("{SESSION}{summaries}"));
        let messages = parse_kwwk_file(file.path());

        assert_eq!(messages.len(), 3);
        let summary = &messages[2];
        assert_eq!(summary.tokens.output, 7);
        assert_eq!(summary.tokens.cache_write, 0);
        assert_eq!(summary.cache_write_1h, 0);
    }

    #[test]
    fn kwwk_cache_retention_and_billed_cost_do_not_change_other_pi_clients() {
        let cases = [
            (
                r#""cacheWrite1h":4,"cost":{"input":0.1,"total":0.25,"billed":true}"#,
                4,
                0.25,
                true,
            ),
            (r#""cacheWrite1h":30,"cost":{"total":0,"billed":true}"#, 10, 0.0, true),
            (r#""cacheWrite1h":-4,"cost":{"total":3,"billed":false}"#, 0, 0.0, false),
            (r#""cacheWrite1h":"4","cost":{"total":3}"#, 0, 0.0, false),
            (r#""cost":{"total":3,"billed":"true"}"#, 0, 0.0, false),
            (r#""cost":{"billed":true}"#, 0, 0.0, false),
            (r#""cost":{"total":-1,"billed":true}"#, 0, 0.0, false),
        ];
        let mut file = session_file(&format!("{}\n", SESSION.lines().next().unwrap()));
        for (id, (details, ..)) in cases.iter().enumerate() {
            writeln!(
                file,
                r#"{{"type":"message","timestamp":1795279700123,"message":{{"role":"assistant","model":"claude-sonnet-5","provider":"openrouter","responseId":"msg_{id}","usage":{{"input":3,"cacheWrite":10,"speed":"fast",{details}}}}}}}"#,
            )
            .unwrap();
        }

        let messages = parse_kwwk_file(file.path());
        assert_eq!(messages.len(), cases.len());
        for (message, (_, cache_write_1h, cost, billed)) in messages.iter().zip(cases) {
            assert_eq!(message.tokens.cache_write, 10);
            assert_eq!(message.cache_write_1h, cache_write_1h);
            assert_eq!(message.cost, cost);
            assert_eq!(message.has_authoritative_cost(), billed);
        }
        let payload = bincode::options().serialize(&messages).unwrap();
        let restored: Vec<UnifiedMessage> = bincode::options().deserialize(&payload).unwrap();
        assert_eq!(restored, messages);
        let pi_messages = super::super::pi::parse_pi_file(file.path());
        assert_eq!(pi_messages.len(), cases.len());
        assert!(pi_messages.iter().all(|message| message.tokens.cache_write == 10
            && message.cache_write_1h == 0
            && message.cost == 0.0
            && !message.has_authoritative_cost()));
    }

    #[test]
    fn kwwk_reported_speed_is_labelled_and_missing_speed_stays_unknown() {
        use crate::speed::{Speed, SpeedSource};

        let turn = |id: &str, speed: &str| {
            format!(
                r#"{{"type":"message","timestamp":1795279700123,"message":{{"role":"assistant","content":[],"api":"anthropic-messages","provider":"anthropic","model":"claude-opus-5-5","responseId":"{id}","usage":{{"input":10,"output":50,"cacheRead":0,"cacheWrite":0,"totalTokens":60{speed}}}}}}}"#
            )
        };
        let header = SESSION.lines().next().unwrap();
        let body = [
            header.to_string(),
            turn("msg_fast", r#","speed":"fast""#),
            turn("msg_standard", r#","speed":"standard""#),
            turn("msg_none", ""),
            turn("msg_odd", r#","speed":"turbo""#),
            turn("msg_typed", r#","speed":7"#),
        ]
        .join("\n");
        let file = session_file(&body);
        let messages = parse_kwwk_file(file.path());

        let speeds: Vec<_> = messages
            .iter()
            .map(|m| m.inference_speed.map(|s| (s.speed, s.source)))
            .collect();
        assert_eq!(
            speeds,
            vec![
                Some((Speed::Fast, SpeedSource::Reported)),
                Some((Speed::Standard, SpeedSource::Reported)),
                None,
                None,
                None,
            ]
        );
        // A malformed speed never costs the record its tokens.
        assert!(messages.iter().all(|m| m.tokens.output == 50));
    }
}
