//! Amp (Sourcegraph) session parser
//!
//! Parses JSON files from ~/.local/share/amp/threads/

use super::utils::read_file_or_none;
use super::UnifiedMessage;
use crate::{provider_identity, TokenBreakdown};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{hash_map::Entry, HashMap};
use std::path::{Path, PathBuf};

/// Synced thread IDs become file names. Leave room for the atomic write suffix.
pub fn is_thread_id(id: &str) -> bool {
    id.len() <= 128
        && id.strip_prefix("T-").is_some_and(|rest| {
            !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

/// Amp usage event from usageLedger
#[derive(Debug, Deserialize, Serialize)]
pub struct AmpUsageEvent {
    pub timestamp: Option<String>,
    pub model: Option<String>,
    pub credits: Option<f64>,
    pub tokens: Option<AmpTokens>,
    #[serde(rename = "operationType", skip_serializing)]
    pub _operation_type: Option<String>,
    #[serde(rename = "fromMessageId", skip_serializing)]
    pub _from_message_id: Option<i64>,
    #[serde(rename = "toMessageId")]
    pub to_message_id: Option<i64>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AmpTokens {
    pub input: Option<i64>,
    pub output: Option<i64>,
    #[serde(rename = "cacheReadInputTokens")]
    pub cache_read_input_tokens: Option<i64>,
    #[serde(rename = "cacheCreationInputTokens")]
    pub cache_creation_input_tokens: Option<i64>,
}

/// Amp message usage (per-message, more detailed)
#[derive(Debug, Deserialize, Serialize)]
pub struct AmpMessageUsage {
    pub model: Option<String>,
    /// When the request was made. Threads exported from ampcode.com
    /// (`amp threads export`) carry it on every usage; older local thread
    /// files do not.
    pub timestamp: Option<String>,
    #[serde(rename = "inputTokens")]
    pub input_tokens: Option<i64>,
    #[serde(rename = "outputTokens")]
    pub output_tokens: Option<i64>,
    #[serde(rename = "cacheReadInputTokens")]
    pub cache_read_input_tokens: Option<i64>,
    #[serde(rename = "cacheCreationInputTokens")]
    pub cache_creation_input_tokens: Option<i64>,
    pub credits: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmpMessage {
    pub role: Option<String>,
    #[serde(rename = "messageId")]
    pub message_id: Option<i64>,
    pub usage: Option<AmpMessageUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmpUsageLedger {
    pub events: Option<Vec<AmpUsageEvent>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmpThread {
    pub id: Option<String>,
    pub created: Option<i64>,
    pub messages: Option<Vec<AmpMessage>>,
    #[serde(rename = "usageLedger", skip_serializing_if = "Option::is_none")]
    pub usage_ledger: Option<AmpUsageLedger>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpCacheProjectionError {
    OtherThread,
    UnexpectedOutput,
}

/// Project a raw export into the usage-only cache. Never return raw Amp data in errors.
pub fn project_cached_thread(
    thread_id: &str,
    export: &[u8],
) -> Result<Vec<u8>, AmpCacheProjectionError> {
    use AmpCacheProjectionError::{OtherThread, UnexpectedOutput};

    if !is_thread_id(thread_id)
        || export.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{')
    {
        return Err(UnexpectedOutput);
    }
    let mut thread: AmpThread = serde_json::from_slice(export).map_err(|_| UnexpectedOutput)?;
    let id = thread.id.as_deref().ok_or(UnexpectedOutput)?;
    if id != thread_id {
        return Err(OtherThread);
    }
    thread
        .messages
        .as_mut()
        .ok_or(UnexpectedOutput)?
        .retain(|message| {
            message.role.as_deref() == Some("assistant")
                && message
                    .usage
                    .as_ref()
                    .is_some_and(|usage| usage.model.is_some())
        });
    if let Some(events) = thread
        .usage_ledger
        .as_mut()
        .and_then(|ledger| ledger.events.as_mut())
    {
        events.retain(|event| event.model.is_some());
    }
    if !has_valid_cached_usage(&thread) {
        return Err(UnexpectedOutput);
    }
    serde_json::to_vec(&thread).map_err(|_| UnexpectedOutput)
}

fn has_valid_cached_usage(thread: &AmpThread) -> bool {
    fn valid(model: Option<&str>, timestamp: Option<&str>, credits: Option<f64>) -> bool {
        model.is_some_and(|id| {
            id.len() <= 256
                && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                && !id.contains("://")
                && id.bytes().all(|b| {
                    b.is_ascii_alphanumeric()
                        || matches!(b, b'-' | b'_' | b'.' | b'/' | b':' | b'@' | b'+')
                })
        }) && timestamp.is_none_or(|ts| chrono::DateTime::parse_from_rfc3339(ts).is_ok())
            && credits.is_none_or(f64::is_finite)
    }

    let Some(messages) = &thread.messages else {
        return false;
    };
    messages.iter().all(|message| {
        message.role.as_deref() == Some("assistant")
            && message.usage.as_ref().is_some_and(|usage| {
                valid(usage.model.as_deref(), usage.timestamp.as_deref(), usage.credits)
            })
    }) && thread
        .usage_ledger
        .as_ref()
        .and_then(|ledger| ledger.events.as_ref())
        .is_none_or(|events| {
            events.iter().all(|event| {
                valid(event.model.as_deref(), event.timestamp.as_deref(), event.credits)
            })
        })
}

/// Amp names some models by their upstream org, such as
/// `zai-org/GLM-5.3-Flash`; other clients report the same model without it.
fn strip_org_prefix(model: &str) -> &str {
    match model.rsplit_once('/') {
        Some((_, id)) if !id.is_empty() => id,
        _ => model,
    }
}

/// Get provider from model name
fn get_provider_from_model(model: &str) -> &'static str {
    provider_identity::inferred_provider_from_model(model).unwrap_or("anthropic")
}

#[derive(Debug)]
struct AmpUsageRecord {
    model: String,
    timestamp: i64,
    has_explicit_timestamp: bool,
    message_id: Option<i64>,
    ledger_to_message_id: Option<i64>,
    tokens: TokenBreakdown,
    cost: f64,
}

impl AmpUsageRecord {
    fn matches_message_usage(&self, other: &Self) -> bool {
        self.model == other.model && self.tokens == other.tokens
    }

    fn into_unified(self, thread_id: &str) -> UnifiedMessage {
        let model = strip_org_prefix(&self.model);
        UnifiedMessage::new(
            "amp",
            model,
            get_provider_from_model(model),
            thread_id.to_string(),
            self.timestamp,
            self.tokens,
            self.cost,
        )
    }
}

fn parse_amp_timestamp(timestamp: Option<String>) -> Option<i64> {
    timestamp
        .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
        .map(|dt| dt.timestamp_millis())
        .filter(|timestamp| *timestamp != 0)
}

fn fallback_amp_timestamp(
    explicit: Option<i64>,
    thread_created_ms: i64,
    file_mtime_ms: i64,
) -> i64 {
    explicit
        .filter(|timestamp| *timestamp != 0)
        .or_else(|| (thread_created_ms != 0).then_some(thread_created_ms))
        .unwrap_or(file_mtime_ms)
}

fn parse_amp_ledger_records(
    usage_ledger: Option<AmpUsageLedger>,
    thread_created_ms: i64,
    file_mtime_ms: i64,
) -> Vec<AmpUsageRecord> {
    let Some(ledger) = usage_ledger else {
        return Vec::new();
    };
    let Some(events) = ledger.events else {
        return Vec::new();
    };

    events
        .into_iter()
        .filter_map(|event| {
            let model = event.model?;
            let tokens = event.tokens.unwrap_or_default();
            if event.credits.is_none()
                && tokens.input.is_none()
                && tokens.output.is_none()
                && tokens.cache_read_input_tokens.is_none()
                && tokens.cache_creation_input_tokens.is_none()
            {
                return None;
            }
            let explicit_timestamp = parse_amp_timestamp(event.timestamp);
            let timestamp =
                fallback_amp_timestamp(explicit_timestamp, thread_created_ms, file_mtime_ms);

            Some(AmpUsageRecord {
                model,
                timestamp,
                has_explicit_timestamp: explicit_timestamp.is_some(),
                message_id: None,
                ledger_to_message_id: event.to_message_id.filter(|id| *id > 0),
                tokens: TokenBreakdown {
                    input: tokens.input.unwrap_or(0).max(0),
                    output: tokens.output.unwrap_or(0).max(0),
                    cache_read: tokens.cache_read_input_tokens.unwrap_or(0).max(0),
                    cache_write: tokens.cache_creation_input_tokens.unwrap_or(0).max(0),
                    reasoning: 0,
                },
                cost: event.credits.unwrap_or(0.0).max(0.0),
            })
        })
        .collect()
}

fn parse_amp_message_records(
    thread_messages: Option<Vec<AmpMessage>>,
    thread_created_ms: i64,
    file_mtime_ms: i64,
    mut timing_only: Option<&mut HashMap<i64, i64>>,
) -> Vec<AmpUsageRecord> {
    let Some(thread_messages) = thread_messages else {
        return Vec::new();
    };

    let base_timestamp = if thread_created_ms != 0 {
        thread_created_ms
    } else {
        file_mtime_ms
    };

    thread_messages
        .into_iter()
        .filter_map(|msg| {
            if msg.role.as_deref() != Some("assistant") {
                return None;
            }

            let usage = msg.usage?;
            let message_id = msg.message_id.filter(|id| *id > 0);
            let metered = usage.credits.is_some()
                || usage.input_tokens.is_some()
                || usage.output_tokens.is_some()
                || usage.cache_read_input_tokens.is_some()
                || usage.cache_creation_input_tokens.is_some();
            let Some(model) = usage.model.filter(|_| metered) else {
                if let (Some(timestamps), Some(id)) = (timing_only.as_deref_mut(), message_id) {
                    if let Some(timestamp) = parse_amp_timestamp(usage.timestamp) {
                        timestamps.entry(id).or_insert(timestamp);
                    }
                }
                return None;
            };
            // Without a recorded time, older local messages are spaced a
            // second apart from the thread's creation.
            let explicit_timestamp = parse_amp_timestamp(usage.timestamp);
            let timestamp = explicit_timestamp.unwrap_or_else(|| {
                base_timestamp.saturating_add(message_id.unwrap_or(0).saturating_mul(1000))
            });

            Some(AmpUsageRecord {
                model,
                timestamp,
                has_explicit_timestamp: explicit_timestamp.is_some(),
                message_id,
                ledger_to_message_id: None,
                tokens: TokenBreakdown {
                    input: usage.input_tokens.unwrap_or(0).max(0),
                    output: usage.output_tokens.unwrap_or(0).max(0),
                    cache_read: usage.cache_read_input_tokens.unwrap_or(0).max(0),
                    cache_write: usage.cache_creation_input_tokens.unwrap_or(0).max(0),
                    reasoning: 0,
                },
                cost: usage.credits.unwrap_or(0.0).max(0.0),
            })
        })
        .collect()
}

fn find_matching_ledger_record(
    ledger_records: &[AmpUsageRecord],
    consumed: &[bool],
    search_start: usize,
    message_record: &AmpUsageRecord,
) -> Option<usize> {
    let find_match = |predicate: &dyn Fn(usize) -> bool| {
        (search_start..ledger_records.len())
            .find(|&index| predicate(index))
            .or_else(|| (0..search_start).find(|&index| predicate(index)))
    };

    if let Some(message_id) = message_record.message_id {
        if let Some(index) = find_match(&|index| {
            !consumed[index] && ledger_records[index].ledger_to_message_id == Some(message_id)
        }) {
            return Some(index);
        }
    }

    find_match(&|index| {
        !consumed[index] && ledger_records[index].matches_message_usage(message_record)
    })
}

fn merge_amp_records(ledger_record: &mut AmpUsageRecord, message_record: &AmpUsageRecord) {
    if !ledger_record.has_explicit_timestamp {
        ledger_record.timestamp = message_record.timestamp;
    }
    if ledger_record.cost <= 0.0 && message_record.cost > 0.0 {
        ledger_record.cost = message_record.cost;
    }
}

fn read_amp_thread(path: &Path) -> Option<AmpThread> {
    let mut bytes = read_file_or_none(path)?;
    if bytes.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{') {
        return None;
    }
    simd_json::from_slice(&mut bytes).ok()
}

/// An empty projected thread is valid. A legacy or incomplete JSON file is
/// not evidence that a synced thread is safe to use for replacement.
pub fn is_valid_cached_thread(path: &Path) -> bool {
    let Some(id) = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".json"))
        .filter(|id| is_thread_id(id))
    else {
        return false;
    };
    if !std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()) {
        return false;
    }
    let Some(bytes) = read_file_or_none(path) else {
        return false;
    };
    // Decode from the original bytes so duplicate recognized fields fail.
    // A separate shape check rejects unknown fields and struct-as-array input.
    serde_json::from_slice::<AmpThread>(&bytes).is_ok_and(|thread| {
        thread.id.as_deref() == Some(id) && has_valid_cached_usage(&thread)
    }) && serde_json::from_slice::<JsonValue>(&bytes)
        .is_ok_and(|value| has_cached_thread_shape(&value))
}

fn has_cached_thread_shape(thread: &JsonValue) -> bool {
    fn has_only_keys(value: &JsonValue, keys: &[&str]) -> bool {
        value
            .as_object()
            .is_some_and(|object| object.keys().all(|key| keys.contains(&key.as_str())))
    }

    fn message_shape(message: &JsonValue) -> bool {
        has_only_keys(message, &["role", "messageId", "usage"])
            && message.get("usage").is_some_and(|usage| {
                has_only_keys(
                    usage,
                    &[
                        "model",
                        "timestamp",
                        "inputTokens",
                        "outputTokens",
                        "cacheReadInputTokens",
                        "cacheCreationInputTokens",
                        "credits",
                    ],
                )
            })
    }

    fn event_shape(event: &JsonValue) -> bool {
        has_only_keys(event, &["timestamp", "model", "credits", "tokens", "toMessageId"])
            && event.get("tokens").is_none_or(|tokens| {
                tokens.is_null()
                    || has_only_keys(
                        tokens,
                        &["input", "output", "cacheReadInputTokens", "cacheCreationInputTokens"],
                    )
            })
    }

    has_only_keys(thread, &["id", "created", "messages", "usageLedger"])
        && thread
            .get("messages")
            .and_then(JsonValue::as_array)
            .is_some_and(|messages| messages.iter().all(message_shape))
        && thread.get("usageLedger").is_none_or(|ledger| {
            has_only_keys(ledger, &["events"])
                && ledger.get("events").is_none_or(|events| {
                    events.is_null()
                        || events
                            .as_array()
                            .is_some_and(|events| events.iter().all(event_shape))
                })
        })
}

fn fallback_session_id(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown")
}

fn restore_fallback_session_id(path: &Path, messages: &mut [UnifiedMessage]) {
    if messages
        .first()
        .is_some_and(|message| message.session_id.is_empty())
    {
        let stem = fallback_session_id(path);
        for message in messages {
            message.session_id.push_str(stem);
        }
    }
}

/// Parse an Amp thread JSON file
pub fn parse_amp_file(path: &Path) -> Vec<UnifiedMessage> {
    let mut messages = parse_amp_source(path);
    restore_fallback_session_id(path, &mut messages);
    messages
}

/// Keep missing IDs distinct from explicit IDs in the parsed source cache.
pub(crate) fn parse_amp_source(path: &Path) -> Vec<UnifiedMessage> {
    let Some(thread) = read_amp_thread(path) else {
        return Vec::new();
    };

    // Get file mtime as last-resort timestamp fallback
    let file_mtime_ms = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let thread_id = thread
        .id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or("");

    let thread_created_ms = thread.created.unwrap_or(0);
    let mut ledger_records =
        parse_amp_ledger_records(thread.usage_ledger, thread_created_ms, file_mtime_ms);
    let mut timing_only = HashMap::new();
    let mut message_records = parse_amp_message_records(
        thread.messages,
        thread_created_ms,
        file_mtime_ms,
        (!ledger_records.is_empty()).then_some(&mut timing_only),
    );

    if ledger_records.is_empty() {
        message_records.sort_by_key(|record| record.timestamp);
        return message_records
            .into_iter()
            .map(|record| record.into_unified(thread_id))
            .collect();
    }

    // Timestamp-only messages identify their exact ledger events, never
    // heuristic matches. Reserve those events before reconciling usage rows.
    let mut consumed: Vec<_> = ledger_records
        .iter_mut()
        .map(|record| {
            let Some(timestamp) = record.ledger_to_message_id.and_then(|id| timing_only.get(&id))
            else {
                return false;
            };
            if !record.has_explicit_timestamp {
                record.timestamp = *timestamp;
            }
            true
        })
        .collect();
    let mut search_start = 0usize;
    let mut unmatched_message_records = Vec::new();

    for message_record in message_records {
        if let Some(index) =
            find_matching_ledger_record(&ledger_records, &consumed, search_start, &message_record)
        {
            consumed[index] = true;
            search_start = index.saturating_add(1);
            merge_amp_records(&mut ledger_records[index], &message_record);
        } else {
            unmatched_message_records.push(message_record);
        }
    }

    ledger_records.extend(unmatched_message_records);
    ledger_records.sort_by_key(|record| record.timestamp);
    ledger_records
        .into_iter()
        .map(|record| record.into_unified(thread_id))
        .collect()
}

/// Select whole threads in scanner priority order. The scanner already
/// deduplicates canonical source paths, so ID-less files are independent
/// unless a usable synced thread explicitly names their filename stem.
pub(crate) fn mark_preferred_sources<'a, T: 'a>(
    paths: &'a [PathBuf],
    synced_root: &Path,
    mut sources: Vec<T>,
    messages: impl Fn(&mut T) -> &mut [UnifiedMessage] + 'a,
) -> impl Iterator<Item = (T, bool)> + 'a {
    let selected: Vec<_> = {
        let mut seen_ids = HashMap::new();
        paths
            .iter()
            .zip(&mut sources)
            .map(|(path, source)| {
                let Some(message) = messages(source).first() else {
                    return false;
                };
                let id = message.session_id.as_str();
                if id.is_empty() {
                    return seen_ids.get(fallback_session_id(path)) != Some(&true);
                }
                if let Entry::Vacant(entry) = seen_ids.entry(id) {
                    entry.insert(path.parent() == Some(synced_root));
                    true
                } else {
                    false
                }
            })
            .collect()
    };
    paths
        .iter()
        .zip(sources)
        .zip(selected)
        .map(move |((path, mut source), selected)| {
            if selected {
                restore_fallback_session_id(path, messages(&mut source));
            }
            (source, selected)
        })
}

#[cfg(test)]
mod tests {
    use super::{is_valid_cached_thread, parse_amp_file, project_cached_thread};
    use std::path::Path;

    fn write_amp_thread(path: &Path, content: &str) {
        std::fs::write(path, content).unwrap();
    }

    fn timestamp_ms(value: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(value)
            .unwrap()
            .timestamp_millis()
    }

    fn local_date(timestamp_ms: i64) -> String {
        use chrono::TimeZone;

        chrono::Local
            .timestamp_millis_opt(timestamp_ms)
            .single()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    }

    #[test]
    fn test_parse_amp_keeps_explicit_zero_usage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("T-zero.json");
        for fields in [
            r#""messages":[{"role":"assistant","usage":{"model":"claude-sonnet-4-0","inputTokens":0}}]"#,
            r#""messages":[{"role":"assistant","usage":{"model":"claude-sonnet-4-0","credits":0}}]"#,
            r#""usageLedger":{"events":[{"model":"claude-sonnet-4-0","tokens":{"cacheReadInputTokens":0}}]}"#,
            r#""usageLedger":{"events":[{"model":"claude-sonnet-4-0","credits":0}]}"#,
        ] {
            write_amp_thread(&path, &format!(r#"{{"id":"T-zero",{fields}}}"#));
            let messages = parse_amp_file(&path);
            assert_eq!(messages.len(), 1, "{fields}");
            assert_eq!(messages[0].tokens, crate::TokenBreakdown::default());
            assert_eq!(messages[0].cost, 0.0);
        }
    }

    #[test]
    fn test_amp_cache_check_distinguishes_empty_threads_from_damage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("T-empty.json");
        assert!(!is_valid_cached_thread(&path));
        for payload in [
            "",
            r#"{"id":"T-empty"}"#,
            r#"{"id":"T-empty","messages":null}"#,
            r#"{"id":"T-empty","messages":{}}"#,
            r#"{"id":"T-empty","messages":["#,
            r#"{"id":"T-empty","messages":[]} {}"#,
            r#"{"id":"T-empty","messages":[{"usage":{"outputTokens":"wrong type"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":null}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"user","usage":{"model":"claude-sonnet-4-0"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":{"model":"not a model"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":{"model":"ok","timestamp":"bad"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":{"model":"ok","inputToken":1}}]}"#,
            r#"{"id":"T-empty","messages":[["assistant",null,{"model":"ok","inputTokens":1}]]}"#,
            r#"{"id":"T-empty","messages":[],"usageLedger":{"events":[{"credits":1}]}}"#,
            r#"{"id":"T-empty","messages":[],"usageLedger":{"events":[{"model":"ok","tokens":{"wrong":1}}]}}"#,
            r#"{"id":"T-empty","messages":[],"title":"not projected"}"#,
            r#"{"id":"T-empty","id":"T-empty","messages":[]}"#,
            r#"["T-empty",null,[],null]"#,
            r#"{"messages":[]}"#,
            r#"{"id":"","messages":[]}"#,
            r#"{"id":" ","messages":[]}"#,
            r#"{"id":"T-other","messages":[]}"#,
        ] {
            write_amp_thread(&path, payload);
            assert!(!is_valid_cached_thread(&path), "{payload}");
        }
        write_amp_thread(&path, r#"{"id":"T-empty","messages":[]}"#);
        assert!(parse_amp_file(&path).is_empty());
        assert!(is_valid_cached_thread(&path));

        #[cfg(unix)]
        {
            let target = dir.path().join("target");
            std::fs::rename(&path, &target).unwrap();
            std::os::unix::fs::symlink(&target, &path).unwrap();
            assert!(!is_valid_cached_thread(&path));
        }
    }

    #[test]
    fn test_amp_projection_satisfies_cache_validation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("T-projected.json");
        let export = br#"{
            "id":"T-projected","title":"synthetic conversation","created":null,
            "messages":[
                {"role":"user","usage":{"model":"ignored"}},
                {"role":"assistant","messageId":7,"content":["not cached"],"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-06T12:00:00Z","extra":1
                }}
            ],
            "usageLedger":{"events":[{
                "model":"claude-sonnet-4-0","toMessageId":7,"operationType":"not cached",
                "tokens":{"input":5,"extra":1}
            }]}
        }"#;
        let projected = project_cached_thread("T-projected", export).unwrap();
        std::fs::write(&path, &projected).unwrap();
        assert!(is_valid_cached_thread(&path));
        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].tokens.input, 5);
        assert_eq!(messages[0].timestamp, timestamp_ms("2026-04-06T12:00:00Z"));
    }

    #[test]
    fn test_amp_unmetered_messages_only_date_exact_ledger_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("T-dates.json");
        let thread = serde_json::json!({
            "id": "T-dates",
            "created": timestamp_ms("2026-04-01T12:00:00Z"),
            "usageLedger": {"events": [
                {"model":"claude-sonnet-4-0","tokens":{"input":17},"toMessageId":7},
                {"model":"claude-sonnet-4-0","credits":0,"toMessageId":8,
                 "timestamp":"2026-04-05T12:00:00Z"}
            ]},
            "messages": [
                {"role":"assistant","messageId":7,"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-06T12:00:00Z"
                }},
                {"role":"assistant","messageId":8,"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-07T12:00:00Z"
                }},
                {"role":"assistant","messageId":9,"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-08T12:00:00Z"
                }},
                {"role":"assistant","messageId":10,"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-09T12:00:00Z","inputTokens":0
                }}
            ]
        });
        write_amp_thread(&path, &thread.to_string());
        let messages = parse_amp_file(&path);
        let rows: Vec<_> = messages
            .iter()
            .map(|message| (message.timestamp, message.tokens.input))
            .collect();
        assert_eq!(
            rows,
            [
                (timestamp_ms("2026-04-05T12:00:00Z"), 0),
                (timestamp_ms("2026-04-06T12:00:00Z"), 17),
                (timestamp_ms("2026-04-09T12:00:00Z"), 0),
            ]
        );
    }

    #[test]
    fn test_parse_amp_reconciles_partial_ledger_with_message_usage() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-partial.json");
        let thread_created = timestamp_ms("2026-04-04T12:00:00Z");
        let ledger_timestamp = "2026-04-08T12:00:00Z";

        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "thread-partial",
                "created": thread_created,
                "usageLedger": {
                    "events": [
                        {
                            "timestamp": ledger_timestamp,
                            "model": "claude-sonnet-4-0",
                            "credits": 0.75,
                            "tokens": { "input": 100, "output": 20 }
                        }
                    ]
                },
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 1,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 100,
                            "outputTokens": 20,
                            "credits": 0.75
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 2,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 50,
                            "outputTokens": 10,
                            "credits": 0.40
                        }
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].date, local_date(thread_created + 2000));
        assert_eq!(messages[1].date, local_date(timestamp_ms(ledger_timestamp)));
        assert_eq!(messages[0].tokens.input, 50);
        assert_eq!(messages[1].tokens.input, 100);
    }

    #[test]
    fn test_parse_amp_does_not_double_count_full_ledger() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-full.json");
        let thread_created = timestamp_ms("2026-04-04T12:00:00Z");
        let first_ledger_timestamp = "2026-04-04T12:00:00Z";
        let second_ledger_timestamp = "2026-04-05T12:00:00Z";

        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "thread-full",
                "created": thread_created,
                "usageLedger": {
                    "events": [
                        {
                            "timestamp": first_ledger_timestamp,
                            "model": "claude-sonnet-4-0",
                            "credits": 0.20,
                            "tokens": { "input": 20, "output": 5 }
                        },
                        {
                            "timestamp": second_ledger_timestamp,
                            "model": "claude-sonnet-4-0",
                            "credits": 0.25,
                            "tokens": { "input": 25, "output": 5 }
                        }
                    ]
                },
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 1,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 20,
                            "outputTokens": 5,
                            "credits": 0.20
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 2,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 25,
                            "outputTokens": 5,
                            "credits": 0.25
                        }
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0].date,
            local_date(timestamp_ms(first_ledger_timestamp))
        );
        assert_eq!(
            messages[1].date,
            local_date(timestamp_ms(second_ledger_timestamp))
        );
    }

    #[test]
    fn test_parse_amp_prefers_message_id_match_over_token_heuristic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-message-id-match.json");
        let thread_created = timestamp_ms("2026-04-04T12:00:00Z");
        let first_ledger_timestamp = "2026-04-10T12:00:00Z";
        let second_ledger_timestamp = "2026-04-05T12:00:00Z";

        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "thread-message-id-match",
                "created": thread_created,
                "usageLedger": {
                    "events": [
                        {
                            "timestamp": first_ledger_timestamp,
                            "model": "claude-sonnet-4-0",
                            "credits": 0.20,
                            "tokens": { "input": 20, "output": 5 },
                            "toMessageId": 2
                        },
                        {
                            "timestamp": second_ledger_timestamp,
                            "model": "claude-sonnet-4-0",
                            "credits": 0.20,
                            "tokens": { "input": 20, "output": 5 },
                            "toMessageId": 1
                        }
                    ]
                },
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 1,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 20,
                            "outputTokens": 5,
                            "credits": 0.20
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 2,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 20,
                            "outputTokens": 5,
                            "credits": 0.20
                        }
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].timestamp, timestamp_ms(second_ledger_timestamp));
        assert_eq!(messages[1].timestamp, timestamp_ms(first_ledger_timestamp));
    }

    #[test]
    fn test_parse_amp_prefers_message_timestamp_when_ledger_timestamp_missing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-missing-ledger-ts.json");
        let thread_created = timestamp_ms("2026-04-04T12:00:00Z");

        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "thread-missing-ts",
                "created": thread_created,
                "usageLedger": {
                    "events": [
                        {
                            "model": "claude-sonnet-4-0",
                            "credits": 0.20,
                            "tokens": { "input": 20, "output": 5 }
                        }
                    ]
                },
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 7,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 20,
                            "outputTokens": 5,
                            "credits": 0.20
                        }
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].timestamp, thread_created + 7000);
    }

    #[test]
    fn test_parse_amp_dates_exported_messages_by_their_usage_timestamp() {
        // `amp threads export` output: no usage ledger, and every usage carries
        // the time of its request. A thread created on the 4th that is still in
        // use on the 6th books that day's usage on the 6th.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-exported.json");
        let thread_created = timestamp_ms("2026-04-04T12:00:00Z");
        let first_usage_timestamp = "2026-04-04T12:00:05Z";
        let second_usage_timestamp = "2026-04-06T12:00:00Z";

        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "T-exported",
                "created": thread_created,
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 2,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "timestamp": first_usage_timestamp,
                            "inputTokens": 20,
                            "outputTokens": 5
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 4,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "timestamp": second_usage_timestamp,
                            "inputTokens": 30,
                            "outputTokens": 7
                        }
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].timestamp, timestamp_ms(first_usage_timestamp));
        assert_eq!(messages[1].timestamp, timestamp_ms(second_usage_timestamp));
        assert_eq!(
            messages[1].date,
            local_date(timestamp_ms(second_usage_timestamp))
        );
        assert_eq!(messages[1].tokens.input, 30);
    }

    #[test]
    fn test_parse_amp_drops_upstream_org_prefix_from_model() {
        // Amp reports some models with their upstream org; other clients
        // report the bare id, which is also what the pricing tables use.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-prefixed.json");
        write_amp_thread(
            &path,
            &serde_json::json!({
                "id": "T-prefixed",
                "created": timestamp_ms("2026-11-21T10:00:00Z"),
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 1,
                        "usage": {
                            "model": "zai-org/GLM-5.3-Flash",
                            "timestamp": "2026-11-21T10:00:05Z",
                            "inputTokens": 10,
                            "outputTokens": 5
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 3,
                        "usage": {
                            "model": "claude-opus-5-5",
                            "timestamp": "2026-11-21T10:00:09Z",
                            "inputTokens": 2,
                            "outputTokens": 1
                        }
                    },
                    {
                        "role": "assistant",
                        "messageId": 4,
                        "usage": {"model": "org/", "timestamp": "2026-11-21T10:00:10Z", "inputTokens": 1}
                    }
                ]
            })
            .to_string(),
        );

        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].model_id, "GLM-5.3-Flash");
        assert_eq!(messages[1].model_id, "claude-opus-5-5");
        assert_eq!(messages[2].model_id, "org/");
    }

    #[test]
    fn test_parse_amp_uses_file_mtime_when_thread_created_missing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("T-no-created.json");

        write_amp_thread(
            &path,
            r#"{
                "id": "thread-no-created",
                "messages": [
                    {
                        "role": "assistant",
                        "messageId": 5,
                        "usage": {
                            "model": "claude-sonnet-4-0",
                            "inputTokens": 10,
                            "outputTokens": 2,
                            "credits": 0.11
                        }
                    }
                ]
            }"#,
        );

        let file_mtime_ms = crate::sessions::utils::file_modified_timestamp_ms(&path);
        let messages = parse_amp_file(&path);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].timestamp >= file_mtime_ms);
        assert_ne!(messages[0].date, "1970-01-01");
    }
}
