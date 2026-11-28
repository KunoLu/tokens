use super::UnifiedMessage;
use crate::{pricing, provider_identity, TokenBreakdown};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Whether `path` is an on-disk Antigravity conversation database rather than
/// a JSONL row file from the `antigravity sync` cache.
fn is_conversation_db(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("db"))
}

/// Parse every Antigravity source the scanner found and merge them.
///
/// Two sources describe the same conversations:
///
/// - The `antigravity sync` cache (`<config>/antigravity-cache/sessions/*.jsonl`),
///   pulled over RPC from a running language server. This is the only way to
///   read older builds, whose `~/.gemini/antigravity/conversations/*.pb` files
///   are encrypted.
/// - Conversation databases written by current Antigravity and Antigravity -
///   Insiders builds under `~/.gemini/antigravity{,-insiders}/conversations/*.db`,
///   in the same `gen_metadata` schema the Antigravity CLI uses. These need no
///   running app and no RPC, and Insiders' standalone language server no longer
///   exposes the RPC flags `antigravity sync` depends on.
///
/// Both are keyed by the cascade id (the `.db` file stem and the cache's
/// `sessionId`). A complete database scan that yields usage takes precedence for
/// token counts. Otherwise, prefer the cache to an incomplete database scan;
/// combining their usage could double count responses whose IDs differ.
/// Matching cache responses can still date database rows missing a turn time.
pub fn parse_antigravity_sources(paths: &[PathBuf]) -> Vec<UnifiedMessage> {
    let (db_paths, cache_paths): (Vec<&PathBuf>, Vec<&PathBuf>) =
        paths.iter().partition(|path| is_conversation_db(path));

    let db_scans: Vec<_> = db_paths
        .par_iter()
        .map(|path| {
            let mut rows = Vec::new();
            let complete = super::antigravity_cli::scan_antigravity_conversation_db(
                path,
                "antigravity",
                &mut |message, has_turn_timestamp| rows.push((message, has_turn_timestamp)),
            );
            (complete, rows)
        })
        .collect();
    let cache_messages: Vec<_> = cache_paths
        .par_iter()
        .flat_map(|path| parse_antigravity_file(path))
        .collect();
    if db_scans.iter().all(|(_, rows)| rows.is_empty()) {
        return cache_messages;
    }

    let complete_db_sessions: HashSet<_> = db_scans
        .iter()
        .filter(|(complete, _)| *complete)
        .filter_map(|(_, rows)| rows.first().map(|(message, _)| message.session_id.clone()))
        .collect();

    let mut cache_sessions = HashSet::new();
    let mut cached_turn_times = HashMap::new();
    for message in &cache_messages {
        let session = message.session_id.as_str();
        if complete_db_sessions.contains(session) {
            // The artifact's usage row dates its own generation. Do not
            // match by position or borrow times from a different cascade.
            if let Some(id) = message.dedup_key.as_deref() {
                cached_turn_times
                    .entry((session, id))
                    .or_insert(message.timestamp);
            }
        } else {
            cache_sessions.insert(session);
        }
    }

    // The same cascade could in principle be present under both the stable and
    // the Insiders data dir; keep the first copy of each response.
    let mut seen_response_ids: HashSet<String> = HashSet::new();
    let mut messages = Vec::new();
    for (complete, rows) in db_scans {
        if !complete {
            let Some((first, _)) = rows.first() else {
                continue;
            };
            let session = first.session_id.as_str();
            if complete_db_sessions.contains(session) || cache_sessions.contains(session) {
                continue;
            }
        }
        for (mut message, has_turn_timestamp) in rows {
            if let Some(id) = message.dedup_key.as_deref() {
                if !seen_response_ids.insert(id.to_owned()) {
                    continue;
                }
                if !has_turn_timestamp {
                    let identity = (message.session_id.as_str(), id);
                    if let Some(&timestamp) = cached_turn_times
                        .get(&identity)
                        .filter(|&&timestamp| timestamp != message.timestamp)
                    {
                        message.set_timestamp(timestamp);
                    }
                }
            }
            messages.push(message);
        }
    }
    messages.extend(
        cache_messages
            .into_iter()
            .filter(|message| !complete_db_sessions.contains(&message.session_id)),
    );
    messages
}

pub fn parse_antigravity_file(path: &Path) -> Vec<UnifiedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    let mut messages = Vec::new();
    let mut session_model: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };

        let row_type = value.get("type").and_then(Value::as_str).unwrap_or("");
        match row_type {
            "session_meta" => {
                if let Some(model_id) = value
                    .get("modelId")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    session_model = Some(model_id.to_string());
                }
            }
            "usage" => {
                if let Some(message) = parse_usage_row(&value, session_model.as_deref()) {
                    messages.push(message);
                }
            }
            _ => {}
        }
    }

    messages
}

fn parse_usage_row(value: &Value, fallback_model: Option<&str>) -> Option<UnifiedMessage> {
    let session_id = value.get("sessionId").and_then(Value::as_str)?.to_string();
    let timestamp = to_safe_i64(value.get("timestamp"));
    if timestamp <= 0 {
        return None;
    }

    let model_id = value
        .get("modelId")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.to_string())
        .or_else(|| fallback_model.map(|text| text.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let model_id = pricing::aliases::resolve_alias(&model_id)
        .unwrap_or(model_id.as_str())
        .to_string();

    let provider_id = value
        .get("providerId")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.to_string())
        .unwrap_or_else(|| infer_provider(&model_id).to_string());
    let provider_id = provider_identity::canonical_provider(&provider_id).unwrap_or(provider_id);

    let input = to_safe_i64(value.get("input"));
    let output = to_safe_i64(value.get("output"));
    let cache_read = to_safe_i64(value.get("cacheRead"));
    let cache_write = to_safe_i64(value.get("cacheWrite"));
    let reasoning = to_safe_i64(value.get("reasoning"));
    if input == 0 && output == 0 && cache_read == 0 && cache_write == 0 && reasoning == 0 {
        return None;
    }

    let dedup_key = value
        .get("responseId")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.to_string());

    Some(UnifiedMessage::new_with_dedup(
        "antigravity",
        model_id,
        provider_id,
        session_id,
        timestamp,
        TokenBreakdown {
            input,
            output,
            cache_read,
            cache_write,
            reasoning,
        },
        0.0,
        dedup_key,
    ))
}

fn infer_provider(model: &str) -> &'static str {
    provider_identity::inferred_provider_from_model(model).unwrap_or("antigravity")
}

fn to_safe_i64(value: Option<&Value>) -> i64 {
    value
        .and_then(|inner| {
            inner
                .as_i64()
                .or_else(|| inner.as_u64().and_then(|number| i64::try_from(number).ok()))
                .or_else(|| inner.as_str().and_then(|text| text.parse::<i64>().ok()))
        })
        .unwrap_or(0)
        .max(0)
}

#[cfg(test)]
mod tests {
    use super::super::antigravity_cli::tests::{
        build_failed_row, build_row, build_row_with_gen9, enc_len, enc_varint, write_conversation,
    };
    use super::*;
    use rusqlite::Connection;

    const CACHE_TIME: i64 = 1_711_200_000_000;

    fn write_cache(path: &Path, session_id: &str, rows: &[(&str, i64)]) {
        let mut lines = format!(
            "{{\"type\":\"session_meta\",\"sessionId\":\"{session_id}\",\"modelId\":\"gemini-3.1-pro\",\"timestamp\":null}}\n"
        );
        for (response_id, timestamp) in rows {
            lines.push_str(&format!(
                "{{\"type\":\"usage\",\"sessionId\":\"{session_id}\",\"modelId\":\"gemini-3.1-pro\",\"timestamp\":{timestamp},\"input\":10,\"output\":5,\"cacheRead\":0,\"cacheWrite\":0,\"reasoning\":0,\"responseId\":\"{response_id}\",\"messageId\":null}}\n"
            ));
        }
        std::fs::write(path, lines).unwrap();
    }

    #[test]
    fn conversation_db_supersedes_the_sync_cache_for_its_session() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = dir.path().join("antigravity-cache/sessions");
        let stable = dir.path().join(".gemini/antigravity/conversations");
        let insiders = dir
            .path()
            .join(".gemini/antigravity-insiders/conversations");
        for path in [&cache_dir, &stable, &insiders] {
            std::fs::create_dir_all(path).unwrap();
        }

        // `both` was synced over RPC before the upgrade and now also has a
        // conversation database; the RPC rows spell the responseId
        // differently, which must not lead to double counting.
        write_cache(
            &cache_dir.join("both.jsonl"),
            "both",
            &[("rpc-1", CACHE_TIME), ("rpc-2", CACHE_TIME + 1)],
        );
        write_conversation(
            &stable.join("both.db"),
            &[build_row(Some("gemini-3-flash-a"), None, "resp-both")],
        );
        // `legacy` only exists in the cache (an encrypted `.pb` conversation).
        write_cache(
            &cache_dir.join("legacy.jsonl"),
            "legacy",
            &[("rpc-legacy", CACHE_TIME)],
        );
        // `fresh` only exists as an Insiders database.
        write_conversation(
            &insiders.join("fresh.db"),
            &[build_row(Some("gemini-3-flash-a"), None, "resp-fresh")],
        );

        let paths = vec![
            cache_dir.join("both.jsonl"),
            cache_dir.join("legacy.jsonl"),
            insiders.join("fresh.db"),
            stable.join("both.db"),
        ];
        let mut messages = parse_antigravity_sources(&paths);
        messages.sort_by(|a, b| a.dedup_key.cmp(&b.dedup_key));

        let keys: Vec<_> = messages
            .iter()
            .map(|message| message.dedup_key.as_deref().unwrap())
            .collect();
        assert_eq!(keys, vec!["resp-both", "resp-fresh", "rpc-legacy"]);
        assert!(messages
            .iter()
            .all(|message| message.client == "antigravity"));
    }

    #[test]
    fn a_db_without_usage_does_not_hide_cached_rows() {
        // Only sessions whose database actually yields usage take over; one
        // holding nothing but rejected generations leaves the cache in charge.
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("s.jsonl");
        let db = dir.path().join("s.db");
        write_cache(&cache, "s", &[("rpc-1", CACHE_TIME)]);
        write_conversation(&db, &[build_failed_row("PERMISSION_DENIED (code 403)")]);

        let messages = parse_antigravity_sources(&[cache, db]);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].dedup_key.as_deref(), Some("rpc-1"));
    }

    #[test]
    fn incomplete_db_usage_does_not_hide_the_sync_cache() {
        for fault in [
            // The index delivers idx 0 before the SQL expression fails stepping idx 1.
            "ALTER TABLE gen_metadata RENAME TO generation_rows;
             CREATE INDEX generation_order ON generation_rows(idx);
             CREATE VIEW gen_metadata AS
               SELECT idx, CASE idx WHEN 1 THEN abs(-9223372036854775808) ELSE data END AS data
               FROM generation_rows;",
            // SQLite permits a TEXT value in this non-strict BLOB column.
            "UPDATE gen_metadata SET data = 'not a blob' WHERE idx = 1;",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let cache = dir.path().join("partial.jsonl");
            let db = dir.path().join("partial.db");
            let rows = [
                build_row(Some("gemini-3-flash-a"), None, "db-1"),
                build_row(Some("gemini-3-flash-a"), None, "db-2"),
            ];
            write_conversation(&db, &rows);
            Connection::open(&db).unwrap().execute_batch(fault).unwrap();
            write_cache(
                &cache,
                "partial",
                &[("rpc-1", CACHE_TIME), ("rpc-2", CACHE_TIME + 1)],
            );

            let db_only = parse_antigravity_sources(std::slice::from_ref(&db));
            assert_eq!(db_only.len(), 1, "{fault}");
            assert_eq!(db_only[0].dedup_key.as_deref(), Some("db-1"), "{fault}");

            let merged = parse_antigravity_sources(&[db.clone(), cache.clone()]);
            assert_eq!(merged.len(), 2, "{fault}");
            assert_eq!(merged[0].dedup_key.as_deref(), Some("rpc-1"), "{fault}");
            assert_eq!(merged[1].dedup_key.as_deref(), Some("rpc-2"), "{fault}");
            assert_eq!(merged.iter().map(|m| m.tokens.input).sum::<i64>(), 20);

            let complete = dir.path().join("other-profile");
            std::fs::create_dir(&complete).unwrap();
            let complete = complete.join("partial.db");
            write_conversation(&complete, &rows);
            let merged = parse_antigravity_sources(&[db, cache, complete]);
            assert_eq!(merged.len(), 2, "{fault}");
            assert_eq!(merged[0].dedup_key.as_deref(), Some("db-1"), "{fault}");
            assert_eq!(merged[1].dedup_key.as_deref(), Some("db-2"), "{fault}");
        }
    }

    #[test]
    fn cache_turn_times_replace_only_matching_db_fallbacks() {
        const SESSION_SECONDS: u64 = 1_711_200_000;
        const STEP_SECONDS: u64 = 1_711_400_000;
        const CACHED_TURN: i64 = 1_711_500_000_123;

        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("timed.db");
        let cache = dir.path().join("timed.jsonl");
        let other_cache = dir.path().join("other.jsonl");
        let model = Some("gemini-3-flash-a");
        let session_created = enc_varint(1, SESSION_SECONDS);
        write_conversation(
            &db,
            &[
                build_row(model, None, "matching"),
                // Equal to the session start, but recorded on the generation itself.
                build_row_with_gen9(&enc_len(4, &session_created), "explicit"),
                build_row(model, None, "step"),
                build_row(model, None, "no-match"),
            ],
        );
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE trajectory_metadata_blob (id text, data blob);
                 CREATE TABLE steps (idx integer, step_type integer, metadata blob);",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO trajectory_metadata_blob VALUES ('main', ?1)",
                [enc_len(2, &session_created)],
            )
            .unwrap();
            let mut step = enc_len(1, &enc_varint(1, STEP_SECONDS));
            step.extend(enc_len(9, &enc_len(11, b"step")));
            conn.execute("INSERT INTO steps VALUES (2, 15, ?1)", [step])
                .unwrap();
        }
        write_cache(
            &cache,
            "timed",
            &[
                ("matching", CACHED_TURN),
                ("explicit", CACHED_TURN + 1),
                ("step", CACHED_TURN + 2),
                ("cache-only", CACHED_TURN + 3),
            ],
        );
        write_cache(&other_cache, "other", &[("no-match", CACHED_TURN + 4)]);

        let messages = parse_antigravity_sources(&[cache, db, other_cache]);
        assert_eq!(messages.len(), 5);
        let times: Vec<_> = messages[..4].iter().map(|m| m.timestamp).collect();
        assert_eq!(
            times,
            [
                CACHED_TURN,
                SESSION_SECONDS as i64 * 1_000,
                STEP_SECONDS as i64 * 1_000,
                SESSION_SECONDS as i64 * 1_000,
            ]
        );
        assert_eq!(messages[0].tokens.input, 1632, "keep DB token counts");
        assert_eq!(messages[0].model_id, "gemini-3.5-flash-high");
        assert_eq!(
            messages[0].date,
            chrono::DateTime::from_timestamp_millis(CACHED_TURN)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d")
                .to_string()
        );
        assert_eq!(messages[4].session_id, "other");
    }

    #[test]
    fn cache_turn_time_outranks_db_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("mtime.db");
        let cache = dir.path().join("mtime.jsonl");
        write_conversation(&db, &[build_row(Some("gemini-3-flash-a"), None, "turn")]);
        write_cache(&cache, "mtime", &[("turn", CACHE_TIME)]);
        let mtime = std::fs::FileTimes::new().set_modified(
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_711_400_000),
        );
        std::fs::OpenOptions::new()
            .write(true)
            .open(&db)
            .unwrap()
            .set_times(mtime)
            .unwrap();

        let messages = parse_antigravity_sources(&[db, cache]);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].timestamp, CACHE_TIME);
        assert_eq!(messages[0].tokens.output, 300);
    }

    #[test]
    fn the_same_cascade_in_two_data_dirs_is_counted_once() {
        let dir = tempfile::tempdir().unwrap();
        let stable = dir.path().join("stable");
        let insiders = dir.path().join("insiders");
        std::fs::create_dir_all(&stable).unwrap();
        std::fs::create_dir_all(&insiders).unwrap();
        let row = build_row(Some("gemini-3-flash-a"), None, "resp-1");
        write_conversation(&stable.join("c.db"), std::slice::from_ref(&row));
        write_conversation(&insiders.join("c.db"), &[row]);

        let messages = parse_antigravity_sources(&[insiders.join("c.db"), stable.join("c.db")]);
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn parse_usage_row_with_meta_fallback() {
        let input = r#"{"type":"session_meta","sessionId":"abc","modelId":"claude-sonnet-4.6"}
{"type":"usage","sessionId":"abc","timestamp":1711200000000,"input":12,"output":4,"cacheRead":2,"cacheWrite":0,"reasoning":1,"responseId":"resp-1"}
"#;

        let path = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(path.path(), input).unwrap();

        let messages = parse_antigravity_file(path.path());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].client, "antigravity");
        assert_eq!(messages[0].model_id, "claude-sonnet-4-6");
        assert_eq!(messages[0].tokens.input, 12);
        assert_eq!(messages[0].tokens.reasoning, 1);
        assert_eq!(messages[0].dedup_key.as_deref(), Some("resp-1"));
    }

    #[test]
    fn parse_usage_row_resolves_placeholder_model_alias() {
        let input = r#"{"type":"usage","sessionId":"abc","modelId":"MODEL_PLACEHOLDER_M26","timestamp":1711200000000,"input":12,"output":4,"cacheRead":2,"cacheWrite":0,"reasoning":1}
"#;

        let path = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(path.path(), input).unwrap();

        let messages = parse_antigravity_file(path.path());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].model_id, "claude-opus-4-6");
        assert_eq!(messages[0].provider_id, "anthropic");
    }

    #[test]
    fn parse_usage_row_resolves_current_placeholder_models() {
        let input = r#"{"type":"usage","sessionId":"abc","modelId":"model_placeholder_m84","timestamp":1711200000000,"input":12,"output":4,"cacheRead":2,"cacheWrite":0,"reasoning":1}
{"type":"usage","sessionId":"abc","modelId":"model_placeholder_m16","timestamp":1711200000001,"input":8,"output":3,"cacheRead":0,"cacheWrite":0,"reasoning":0}
"#;

        let path = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(path.path(), input).unwrap();

        let messages = parse_antigravity_file(path.path());
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].model_id, "gemini-3-flash-preview");
        assert_eq!(messages[0].provider_id, "google");
        assert_eq!(messages[1].model_id, "gemini-3.1-pro");
        assert_eq!(messages[1].provider_id, "google");
    }
}
