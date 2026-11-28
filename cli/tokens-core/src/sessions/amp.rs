//! Amp (Sourcegraph) session parser
//!
//! Parses JSON files from ~/.local/share/amp/threads/

use super::utils::read_file_or_none;
use super::UnifiedMessage;
use crate::{provider_identity, TokenBreakdown};
use serde::{Deserialize, Serialize};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<AmpTokens>,
    #[serde(rename = "operationType", skip_serializing)]
    pub _operation_type: Option<String>,
    #[serde(rename = "fromMessageId", skip_serializing)]
    pub _from_message_id: Option<i64>,
    #[serde(rename = "toMessageId")]
    pub to_message_id: Option<i64>,
}

impl AmpUsageEvent {
    fn has_metering(&self) -> bool {
        self.credits.is_some()
            || self.tokens.as_ref().is_some_and(|tokens| {
                tokens.input.is_some()
                    || tokens.output.is_some()
                    || tokens.cache_read_input_tokens.is_some()
                    || tokens.cache_creation_input_tokens.is_some()
            })
    }
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
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AmpMessageUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// When the request was made. Threads exported from ampcode.com
    /// (`amp threads export`) carry it on every usage; older local thread
    /// files do not.
    pub timestamp: Option<String>,
    #[serde(rename = "inputTokens", skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<i64>,
    #[serde(rename = "outputTokens", skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<i64>,
    #[serde(rename = "cacheReadInputTokens", skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<i64>,
    #[serde(rename = "cacheCreationInputTokens", skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
}

impl AmpMessageUsage {
    fn has_metering(&self) -> bool {
        self.credits.is_some()
            || self.input_tokens.is_some()
            || self.output_tokens.is_some()
            || self.cache_read_input_tokens.is_some()
            || self.cache_creation_input_tokens.is_some()
    }
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
    let messages = thread.messages.as_mut().ok_or(UnexpectedOutput)?;
    retain_projected_records(messages, |message| {
        if message.role.as_deref() != Some("assistant") {
            return Ok(false);
        }
        let Some(usage) = &mut message.usage else {
            return Ok(false);
        };
        match projected_kind(
            usage.model.as_deref(),
            usage.timestamp.as_deref(),
            usage.credits,
            usage.has_metering(),
            message.message_id,
        )? {
            CachedRecordKind::Metered => Ok(true),
            CachedRecordKind::Timing => {
                *usage = AmpMessageUsage {
                    timestamp: usage.timestamp.take(),
                    ..Default::default()
                };
                Ok(true)
            }
            CachedRecordKind::Discard => Ok(false),
        }
    })?;
    if let Some(events) = thread
        .usage_ledger
        .as_mut()
        .and_then(|ledger| ledger.events.as_mut())
    {
        retain_projected_records(events, |event| {
            match projected_kind(
                event.model.as_deref(),
                event.timestamp.as_deref(),
                event.credits,
                event.has_metering(),
                event.to_message_id,
            )? {
                CachedRecordKind::Metered => Ok(true),
                CachedRecordKind::Timing => {
                    event.model = None;
                    event.credits = None;
                    event.tokens = None;
                    Ok(true)
                }
                CachedRecordKind::Discard => Ok(false),
            }
        })?;
    }
    serde_json::to_vec(&thread).map_err(|_| UnexpectedOutput)
}

fn retain_projected_records<T>(
    records: &mut Vec<T>,
    mut project: impl FnMut(&mut T) -> Result<bool, AmpCacheProjectionError>,
) -> Result<(), AmpCacheProjectionError> {
    let mut result = Ok(());
    records.retain_mut(|record| {
        if result.is_err() {
            return false;
        }
        match project(record) {
            Ok(keep) => keep,
            Err(error) => {
                result = Err(error);
                false
            }
        }
    });
    result
}

enum CachedRecordKind {
    Metered,
    Timing,
    Discard,
}

fn projected_kind(
    model: Option<&str>,
    timestamp: Option<&str>,
    credits: Option<f64>,
    metered: bool,
    message_id: Option<i64>,
) -> Result<CachedRecordKind, AmpCacheProjectionError> {
    use AmpCacheProjectionError::UnexpectedOutput;
    use CachedRecordKind::{Discard, Metered, Timing};

    let has_timing_fields = message_id.is_some_and(|id| id > 0) && timestamp.is_some();
    if model.is_none() && !has_timing_fields {
        return Ok(Discard);
    }
    // Validate strings before writing them or reducing the row to timing evidence.
    if !model.is_none_or(valid_cached_model) || !credits.is_none_or(f64::is_finite) {
        return Err(UnexpectedOutput);
    }
    let timestamp = timestamp
        .map(chrono::DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| UnexpectedOutput)?;
    if model.is_some() && metered {
        Ok(Metered)
    } else if has_timing_fields && timestamp.is_some_and(|ts| ts.timestamp_millis() != 0) {
        Ok(Timing)
    } else {
        Ok(Discard)
    }
}

fn valid_cached_model(id: &str) -> bool {
    id.len() <= 256
        && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && !id.contains("://")
        && id.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/' | b':' | b'@' | b'+')
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
    cost: Option<f64>,
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
            self.cost.unwrap_or(0.0),
        )
    }
}

fn parse_amp_timestamp(timestamp: Option<String>) -> Option<i64> {
    timestamp
        .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
        .map(|dt| dt.timestamp_millis())
        .filter(|timestamp| *timestamp != 0)
}

fn record_amp_timing(
    timings: Option<&mut HashMap<i64, i64>>,
    message_id: Option<i64>,
    timestamp: Option<String>,
) {
    let (Some(timings), Some(id)) = (timings, message_id) else {
        return;
    };
    if let Some(timestamp) = parse_amp_timestamp(timestamp) {
        timings.entry(id).or_insert(timestamp);
    }
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
    mut timing_only: Option<&mut HashMap<i64, i64>>,
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
            let to_message_id = event.to_message_id.filter(|id| *id > 0);
            let metered = event.has_metering();
            let Some(model) = event.model.filter(|_| metered) else {
                record_amp_timing(timing_only.as_deref_mut(), to_message_id, event.timestamp);
                return None;
            };
            let tokens = event.tokens.unwrap_or_default();
            let explicit_timestamp = parse_amp_timestamp(event.timestamp);
            let timestamp =
                fallback_amp_timestamp(explicit_timestamp, thread_created_ms, file_mtime_ms);

            Some(AmpUsageRecord {
                model,
                timestamp,
                has_explicit_timestamp: explicit_timestamp.is_some(),
                message_id: None,
                ledger_to_message_id: to_message_id,
                tokens: TokenBreakdown {
                    input: tokens.input.unwrap_or(0).max(0),
                    output: tokens.output.unwrap_or(0).max(0),
                    cache_read: tokens.cache_read_input_tokens.unwrap_or(0).max(0),
                    cache_write: tokens.cache_creation_input_tokens.unwrap_or(0).max(0),
                    reasoning: 0,
                },
                cost: event.credits.filter(|cost| *cost >= 0.0),
            })
        })
        .collect()
}

fn parse_amp_message_records(
    thread_messages: Option<Vec<AmpMessage>>,
    thread_created_ms: i64,
    file_mtime_ms: i64,
    ledger_timings: &HashMap<i64, i64>,
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
            let metered = usage.has_metering();
            let Some(model) = usage.model.filter(|_| metered) else {
                record_amp_timing(timing_only.as_deref_mut(), message_id, usage.timestamp);
                return None;
            };
            // Without a recorded time, older local messages are spaced a
            // second apart from the thread's creation.
            let explicit_timestamp = parse_amp_timestamp(usage.timestamp)
                .or_else(|| message_id.and_then(|id| ledger_timings.get(&id).copied()));
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
                cost: usage.credits.filter(|cost| *cost >= 0.0),
            })
        })
        .collect()
}

fn next_ledger_match(
    count: usize,
    start: usize,
    mut matches: impl FnMut(usize) -> bool,
) -> Option<usize> {
    (start..count).chain(0..start).find(|&index| matches(index))
}

fn merge_amp_records(ledger_record: &mut AmpUsageRecord, message_record: &AmpUsageRecord) {
    if !ledger_record.has_explicit_timestamp {
        ledger_record.timestamp = message_record.timestamp;
    }
    ledger_record.cost = ledger_record.cost.or(message_record.cost);
}

fn read_amp_thread(path: &Path) -> Option<AmpThread> {
    let mut bytes = read_file_or_none(path)?;
    if bytes.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{') {
        return None;
    }
    simd_json::from_slice(&mut bytes).ok()
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
    let mut ledger_timings = HashMap::new();
    let mut ledger_records = parse_amp_ledger_records(
        thread.usage_ledger,
        thread_created_ms,
        file_mtime_ms,
        thread
            .messages
            .as_ref()
            .is_some_and(|messages| !messages.is_empty())
            .then_some(&mut ledger_timings),
    );
    let mut message_timings = HashMap::new();
    let mut message_records = parse_amp_message_records(
        thread.messages,
        thread_created_ms,
        file_mtime_ms,
        &ledger_timings,
        (!ledger_records.is_empty()).then_some(&mut message_timings),
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
            let Some(timestamp) = record
                .ledger_to_message_id
                .and_then(|id| message_timings.get(&id))
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
    // Resolve exact IDs first. An earlier row with identical token counts must
    // not consume an event belonging to a later message.
    message_records.retain(|message| {
        let Some(id) = message.message_id else { return true };
        let Some(index) = next_ledger_match(ledger_records.len(), search_start, |i| {
            !consumed[i] && ledger_records[i].ledger_to_message_id == Some(id)
        }) else { return true };
        consumed[index] = true;
        search_start = index + 1;
        merge_amp_records(&mut ledger_records[index], message);
        false
    });

    search_start = 0;
    message_records.retain(|message| {
        if message.message_id.is_some_and(|id| ledger_timings.contains_key(&id)) {
            return true;
        }
        let Some(index) = next_ledger_match(ledger_records.len(), search_start, |i| {
            let event = &ledger_records[i];
            !consumed[i]
                && (message.message_id.is_none() || event.ledger_to_message_id.is_none())
                && event.matches_message_usage(message)
        }) else { return true };
        consumed[index] = true;
        search_start = index + 1;
        merge_amp_records(&mut ledger_records[index], message);
        false
    });
    ledger_records.extend(message_records);
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
    synced_count: usize,
    mut sources: Vec<T>,
    messages: impl Fn(&mut T) -> &mut [UnifiedMessage] + 'a,
) -> impl Iterator<Item = (T, bool)> + 'a {
    let selected: Vec<_> = {
        let mut seen_ids = HashMap::new();
        paths
            .iter()
            .zip(&mut sources)
            .enumerate()
            .map(|(index, (path, source))| {
                let Some(message) = messages(source).first() else {
                    return false;
                };
                let id = message.session_id.as_str();
                if id.is_empty() {
                    return seen_ids.get(fallback_session_id(path)) != Some(&true);
                }
                if let Entry::Vacant(entry) = seen_ids.entry(id) {
                    entry.insert(index < synced_count);
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
    use super::{parse_amp_file, project_cached_thread, AmpCacheProjectionError};
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
    fn test_amp_projection_rejects_invalid_usage_or_timing() {
        for export in [
            r#"{"id":"T-empty"}"#,
            r#"{"id":"T-empty","messages":null}"#,
            r#"{"id":"T-empty","messages":{}}"#,
            r#"{"id":"T-empty","id":"T-empty","messages":[]}"#,
            r#"["T-empty",null,[],null]"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":{"inputTokens":"private"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","usage":{"model":"private text"}}]}"#,
            r#"{"id":"T-empty","messages":[{"role":"assistant","messageId":7,"usage":{"timestamp":"private"}}]}"#,
            r#"{"id":"T-empty","messages":[],"usageLedger":{"events":[{"toMessageId":7,"timestamp":"private"}]}}"#,
        ] {
            assert_eq!(
                project_cached_thread("T-empty", export.as_bytes()),
                Err(AmpCacheProjectionError::UnexpectedOutput),
                "{export}"
            );
        }
        let export = br#"{"id":"T-empty","messages":[]}"#;
        assert!(project_cached_thread("T-empty", export).is_ok());
    }

    #[test]
    fn test_amp_projection_keeps_only_metering_and_exact_timing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("T-projected.json");
        let export = br#"{
            "id":"T-projected","title":"not cached","created":null,
            "messages":[
                {"role":"user","usage":{"model":"ignored"}},
                {"role":"assistant","messageId":7,"content":["not cached"],"usage":{
                    "timestamp":"2026-04-06T12:00:00Z","extra":"not cached"
                }},
                {"role":"assistant","messageId":8,"usage":{"model":"claude-sonnet-4-0","inputTokens":3}},
                {"role":"assistant","messageId":10,"usage":{
                    "model":"claude-sonnet-4-0","timestamp":"2026-04-10T12:00:00Z","inputTokens":4
                }},
                {"role":"assistant","messageId":0,"usage":{"timestamp":"2026-04-09T12:00:00Z"}}
            ],
            "usageLedger":{"events":[
                {"model":"claude-sonnet-4-0","toMessageId":7,"operationType":"not cached",
                 "tokens":{"input":5,"extra":"not cached"}},
                {"model":"claude-sonnet-4-0","toMessageId":8,"timestamp":"2026-04-07T12:00:00Z"},
                {"toMessageId":9,"timestamp":"2026-04-08T12:00:00Z","credits":12,"tokens":{"input":11}},
                {"toMessageId":10,"timestamp":"2026-04-01T12:00:00Z","extra":"not cached"},
                {"model":"claude-sonnet-4-0","toMessageId":11,
                 "timestamp":"2026-04-05T12:00:00Z","tokens":{"input":3}},
                {"fromMessageId":8,"timestamp":"2026-04-09T12:00:00Z","model":"claude-sonnet-4-0"}
            ]}
        }"#;
        let projected = project_cached_thread("T-projected", export).unwrap();
        assert!(!std::str::from_utf8(&projected).unwrap().contains("not cached"));
        let json: serde_json::Value = serde_json::from_slice(&projected).unwrap();
        assert_eq!(json["messages"].as_array().unwrap().len(), 3);
        assert_eq!(
            json["messages"][0],
            serde_json::json!({
                "role": "assistant", "messageId": 7, "usage": {"timestamp": "2026-04-06T12:00:00Z"}
            })
        );
        let events = json["usageLedger"]["events"].as_array().unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(
            events[1],
            serde_json::json!({"toMessageId": 8, "timestamp": "2026-04-07T12:00:00Z"})
        );
        assert_eq!(
            events[2],
            serde_json::json!({"toMessageId": 9, "timestamp": "2026-04-08T12:00:00Z"})
        );

        for source in [export.as_slice(), projected.as_slice()] {
            std::fs::write(&path, source).unwrap();
            let rows: Vec<_> = parse_amp_file(&path)
                .iter()
                .map(|message| (message.timestamp, message.tokens.input))
                .collect();
            assert_eq!(
                rows,
                [
                    (timestamp_ms("2026-04-05T12:00:00Z"), 3),
                    (timestamp_ms("2026-04-06T12:00:00Z"), 5),
                    (timestamp_ms("2026-04-07T12:00:00Z"), 3),
                    (timestamp_ms("2026-04-10T12:00:00Z"), 4),
                ]
            );
        }
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
                 "timestamp":"2026-04-05T12:00:00Z"},
                {"model":"claude-sonnet-4-0","credits":0,"toMessageId":11,
                 "timestamp":"2026-04-10T12:00:00Z"}
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
                }},
                {"role":"assistant","messageId":11,"usage":{
                    "model":"claude-sonnet-4-0","credits":1
                }}
            ]
        });
        write_amp_thread(&path, &thread.to_string());
        let messages = parse_amp_file(&path);
        let rows: Vec<_> = messages
            .iter()
            .map(|message| (message.timestamp, message.tokens.input, message.cost))
            .collect();
        assert_eq!(
            rows,
            [
                (timestamp_ms("2026-04-05T12:00:00Z"), 0, 0.0),
                (timestamp_ms("2026-04-06T12:00:00Z"), 17, 0.0),
                (timestamp_ms("2026-04-09T12:00:00Z"), 0, 0.0),
                (timestamp_ms("2026-04-10T12:00:00Z"), 0, 0.0),
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
