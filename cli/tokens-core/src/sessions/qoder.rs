//! Qoder and Qoder CN session parsers.
//!
//! Qoder is Alibaba's agentic coding product, shipped as two fully separate
//! editions with separate accounts, model pools and data directories:
//!
//! | | international | China |
//! |---|---|---|
//! | CLI transcripts | `~/.qoder/projects/` | `~/.qoder-cn/projects/` |
//! | CLI config override | `QODER_CONFIG_DIR` | `QODERCN_CONFIG_DIR` |
//! | Structured logs | `~/.qoder/logs/sessions/` | `~/.qoder-cn/logs/sessions/` |
//! | IDE SQLite | `Qoder/SharedClientCache/cache/db/local.db` | `QoderCN/…` |
//!
//! Both editions are exposed through one client (`qoder`). The edition only
//! namespaces the deduplication keys and the diagnostics; it never becomes a
//! provider, and the two editions never deduplicate each other's requests.
//!
//! # Three local sources
//!
//! 1. **Transcripts** (`<config>/projects/<project>/<session>.jsonl` and
//!    `<session>/subagents/agent-*.jsonl`): Claude-Code-shaped JSONL. One
//!    assistant reply is written as several `type:"assistant"` lines sharing
//!    `message.id`, and only the last one carries `message.usage`. Requests are
//!    keyed by `message.usage.request_id` so a forked session replaying the
//!    parent's history cannot count a request twice.
//! 2. **Segment logs** (`<config>/logs/sessions/<project>/<session>/segments/*.jsonl`):
//!    Qoder CLI 1.1.13+ writes requests made with `--no-session-persistence`
//!    here only, so a transcript-only reader silently loses that usage. Only
//!    `type:"model.response.completed"` is a per-request record; the
//!    `turn.finished` roll-up is ignored so a turn is never counted twice.
//! 3. **IDE SQLite** (`…/SharedClientCache/cache/db/local.db`): the desktop IDE
//!    persists `chat_message.token_info` (`prompt_tokens` already *includes*
//!    `cached_tokens`) even when its embedded agent reports zeroed token
//!    columns on the stream-json channel.
//!
//! # What is counted
//!
//! Only token counts Qoder actually recorded. Managed (first-party) models bill
//! in credits: Qoder zeroes every token field and leaves `credits` /
//! `context_usage_ratio` behind, or omits `usage` entirely (schema v1.1.5).
//! Those rows are **not** estimated from reply length, `credits`, or context
//! occupancy — they produce no message at all. `QODER_EXPOSE_TOKEN_USAGE=1`
//! (international) / `QODERCN_EXPOSE_TOKEN_USAGE=1` (China) make Qoder record
//! real counters for *subsequent* requests; nothing backfills history.
//!
//! # Cache-read accounting
//!
//! Qoder's agent engine reports `input_tokens` as the whole prompt, with
//! `cache_read_input_tokens` a subset of it (OpenAI/`prompt_tokens` semantics,
//! unlike Anthropic's net `input_tokens`). The recorded
//! `context_usage_ratio == input_tokens / context_window` relationship proves
//! it on real 1.1.76+ transcripts, and it matches the Go engine's documented
//! usage conversion in usageBar. Net input is therefore `input_tokens -
//! cache_read_input_tokens`, and the cached subset is never added twice. The
//! IDE's `token_info.prompt_tokens` carries the same relationship.
//!
//! # Model attribution
//!
//! `message.model` wins; Qoder 1.1.82+ writes `"auto"` there and puts the real
//! slot in the preceding `user.businessModelKey`, with mid-session switches in
//! `runtime-config.model`. Slot identifiers (`qmodel`, `qfmodel`, `dfmodel`,
//! `gfmodel`, …) are routing slots whose backing model changes over time, so
//! they are kept verbatim and attributed to the `qoder` provider — never mapped
//! to a guessed vendor, and never priced through another client's `auto` entry.
//! BYOK models keep their vendor (`deepseek/chat/deepseek-v3` →
//! `deepseek-v3` + `deepseek`), including when Qoder qualifies them with its own
//! name (`qoder/claude-sonnet-4-6` → `claude-sonnet-4-6` + `anthropic`).
//! Across sources, a model reported for the request beats business fallbacks
//! and session defaults.
//! Provider evidence for the winning model is evaluated separately from time.

use super::utils::{
    extract_i64, file_modified_timestamp_ms, open_readonly_sqlite, parse_timestamp_value,
    read_json_lines, sqlite_for_each_row_on,
};
use super::{
    normalize_agent_name, normalize_workspace_key, workspace_label_from_key, UnifiedMessage,
};
use crate::message_cache::SourceMessageCache;
use crate::scanner::ScanResult;
use crate::{ClientId, TokenBreakdown};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Bumped whenever a change makes byte-identical input parse differently.
///
/// - Version 1: initial implementation.
/// - Version 2: edition-scoped source keys and gross-prompt merging.
/// - Version 3: partial counters, request attribution and fragment timestamps.
/// - Version 4: measured time and model evidence across all sources.
/// - Version 5: independent provider evidence and complete Qoder observation snapshots.
/// - Version 6: reject incomplete JSONL prefixes and unverified SQLite cache entries.
/// - Version 7: share the absolute JSONL path between parsing and caching.
pub const QODER_PARSER_BASE_VERSION: u32 = 7;

/// Public client id. Both editions report under it.
pub const CLIENT_ID: &str = "qoder";

/// Provider recorded for rows whose slot names a router, not a vendor.
const QODER_PROVIDER: &str = "qoder";

/// Model recorded when the record names none at all.
const DEFAULT_MODEL: &str = "unknown";

/// `BUSY` wait (ms) for the IDE database, which the IDE may hold write-locked.
const QODER_SQLITE_BUSY_TIMEOUT_MS: u64 = 5_000;

/// Which edition a path belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoderSource {
    International,
    China,
}

impl QoderSource {
    /// Client/source spelling used in dedup keys and diagnostics.
    pub const fn as_str(self) -> &'static str {
        match self {
            QoderSource::International => "qoder",
            QoderSource::China => "qoder-cn",
        }
    }

    fn dedup_key(self, key: std::fmt::Arguments<'_>) -> String {
        format!("{CLIENT_ID}:{}:{key}", self.as_str())
    }

    /// Edition fallback for extra paths without a registered scan root.
    pub fn from_path(path: &Path) -> Self {
        for component in path.components().rev() {
            let name = component.as_os_str();
            if name.eq_ignore_ascii_case(".qoder-cn") || name.eq_ignore_ascii_case("qodercn") {
                return Self::China;
            }
            if name.eq_ignore_ascii_case(".qoder") || name.eq_ignore_ascii_case("qoder") {
                return Self::International;
            }
            let name = name.to_string_lossy();
            if let Some(bundle) = name.get(.."com.qodercn.".len()) {
                if bundle.eq_ignore_ascii_case("com.qodercn.") {
                    return Self::China;
                }
            }
            if let Some(bundle) = name.get(.."com.qoder.".len()) {
                if bundle.eq_ignore_ascii_case("com.qoder.") {
                    return Self::International;
                }
            }
        }
        Self::International
    }
}

/// True for `.jsonl` files under a `segments/` directory.
pub fn is_qoder_segment_path(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("jsonl"))
        .unwrap_or(false)
        && path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name.eq_ignore_ascii_case("segments"))
}

/// True for Qoder's non-transcript bookkeeping files that share `.jsonl`.
///
/// `subagents/**/journal.jsonl` is a subagent run journal and
/// `<session>.checkpoints.jsonl` a checkpoint index; neither carries usage.
pub fn should_skip_qoder_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return true;
    };
    if file_name.ends_with(".checkpoints.jsonl") {
        return true;
    }
    file_name.eq_ignore_ascii_case("journal.jsonl")
        && path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("subagents"))
        })
}

/// Routing slots. These name a router or a moving business slot, not a model.
fn is_routing_slot(model: &str) -> bool {
    let model = model.trim();
    [
        "ultimate",
        "performance",
        "efficient",
        "lite",
        "qmodel",
        "qfmodel",
        "dfmodel",
        "gfmodel",
        "kmodel",
        "gmodel",
        "mmodel",
        "q35model",
    ]
    .iter()
    .any(|slot| model.eq_ignore_ascii_case(slot))
        || model.get(.."qmodel_".len()).is_some_and(|prefix| {
            prefix.eq_ignore_ascii_case("qmodel_") || prefix.eq_ignore_ascii_case("qmodel-")
        })
}

fn is_synthetic_placeholder_model(model: &str) -> bool {
    let trimmed = model.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("<synthetic>")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum ModelKind {
    Unknown,
    Auto,
    Slot,
    Named,
}

fn model_kind(model: &str) -> ModelKind {
    if is_synthetic_placeholder_model(model) || model.eq_ignore_ascii_case(DEFAULT_MODEL) {
        ModelKind::Unknown
    } else if model.eq_ignore_ascii_case("auto") {
        ModelKind::Auto
    } else if is_routing_slot(model) {
        ModelKind::Slot
    } else {
        ModelKind::Named
    }
}

fn is_selected_model(raw_model: &str) -> bool {
    model_kind(split_model_qualifier(raw_model).0) >= ModelKind::Slot
}

fn is_qoder_namespace(namespace: &str) -> bool {
    namespace.eq_ignore_ascii_case("qoder") || namespace.eq_ignore_ascii_case("qoder-cn")
}

fn strip_qoder_namespace(model: &str) -> &str {
    match model.split_once('/') {
        Some((head, tail)) if is_qoder_namespace(head) => tail.trim(),
        _ => model,
    }
}

fn same_runtime_model(left: &str, right: &str) -> bool {
    strip_qoder_namespace(left).eq_ignore_ascii_case(strip_qoder_namespace(right))
}

/// Split a client-qualified or vendor-qualified model into its bare id.
///
/// `qoder/claude-sonnet-4-6` and `qoder-cn/deepseek-v3` are Qoder's own
/// namespaces, not providers; `deepseek/chat/deepseek-v3` is
/// `<provider>/<type>/<model>`.
fn split_model_qualifier(model: &str) -> (&str, Option<&str>) {
    let trimmed = model.trim();
    let Some((head, _)) = trimmed.split_once('/') else {
        return (trimmed, None);
    };
    let bare = trimmed
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|bare| !bare.is_empty())
        .unwrap_or(trimmed);
    (bare, (!is_qoder_namespace(head)).then_some(head))
}

fn canonical_qoder_provider(provider: &str) -> Option<String> {
    let provider = provider.trim();
    if ["custom", QODER_PROVIDER, "qoder-cn"]
        .iter()
        .any(|placeholder| provider.eq_ignore_ascii_case(placeholder))
    {
        return None;
    }
    crate::provider_identity::canonical_provider(provider)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum ModelEvidence {
    Placeholder,
    SessionDefault,
    InheritedBusinessFallback,
    BusinessFallback,
    Reported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ModelRank {
    evidence: ModelEvidence,
    kind: ModelKind,
}

impl ModelRank {
    fn new(kind: ModelKind, evidence: ModelEvidence) -> Self {
        Self {
            evidence: if kind >= ModelKind::Slot {
                evidence
            } else {
                ModelEvidence::Placeholder
            },
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum ProviderEvidence {
    Unattributed,
    Inferred,
    Configured,
    ModelPrefix,
    Record,
}

impl ProviderEvidence {
    fn for_model(evidence: ModelEvidence) -> Self {
        if evidence == ModelEvidence::Reported {
            Self::ModelPrefix
        } else {
            Self::Configured
        }
    }
}

struct RequestModel {
    id: String,
    rank: ModelRank,
    provider: Option<(ProviderEvidence, String)>,
}

impl RequestModel {
    fn new(
        raw_model: &str,
        evidence: ModelEvidence,
        explicit_provider: Option<&str>,
        runtime_provider: Option<&str>,
    ) -> Self {
        let (id, prefix) = split_model_qualifier(raw_model);
        let kind = model_kind(id);
        let mut model = Self {
            id: if kind == ModelKind::Unknown {
                DEFAULT_MODEL.to_string()
            } else {
                id.to_string()
            },
            rank: ModelRank::new(kind, evidence),
            provider: None,
        };
        model.observe_provider(explicit_provider, ProviderEvidence::Record);
        model.observe_provider(prefix, ProviderEvidence::for_model(evidence));
        if model.provider.is_none() && kind == ModelKind::Named {
            model.provider = runtime_provider
                .map(|provider| (ProviderEvidence::Configured, provider.to_string()));
        }
        model
    }

    fn observe_provider(&mut self, provider: Option<&str>, evidence: ProviderEvidence) {
        if self.provider.as_ref().is_some_and(|(old, _)| *old >= evidence) {
            return;
        }
        if let Some(provider) = provider.and_then(canonical_qoder_provider) {
            self.provider = Some((evidence, provider));
        }
    }

    fn merge_fragment(
        &mut self,
        selection: Option<(&str, ModelEvidence)>,
        explicit_provider: Option<&str>,
    ) {
        let mut prefix = None;
        let mut prior_request_provider = None;
        if let Some((raw_model, evidence)) = selection {
            let (id, qualifier) = split_model_qualifier(raw_model);
            let same_model = self.id.eq_ignore_ascii_case(id);
            let kind = if same_model {
                self.rank.kind
            } else {
                model_kind(id)
            };
            let rank = ModelRank::new(kind, evidence);
            if same_model || rank > self.rank {
                if !same_model {
                    // Keep provider-only evidence only as a fallback. A provider
                    // tied to the old model must not follow a different model.
                    let old_provider = self.provider.take();
                    if self.rank.evidence < ModelEvidence::BusinessFallback {
                        prior_request_provider = old_provider
                            .filter(|(evidence, _)| *evidence == ProviderEvidence::Record);
                    }
                    self.id = id.to_string();
                }
                self.rank = self.rank.max(rank);
                prefix = qualifier.map(|provider| (provider, ProviderEvidence::for_model(evidence)));
            } else if kind >= ModelKind::Slot {
                return;
            }
        }
        self.observe_provider(explicit_provider, ProviderEvidence::Record);
        if let Some((provider, evidence)) = prefix {
            self.observe_provider(Some(provider), evidence);
        }
        if self.provider.is_none() {
            self.provider = prior_request_provider;
        }
    }

    fn into_ids(self) -> (String, String, ProviderEvidence) {
        // Never price a router or an unknown model as if it were a vendor model.
        let (evidence, provider) = if self.rank.kind == ModelKind::Named {
            self.provider.or_else(|| {
                crate::provider_identity::inferred_provider_from_model_delimited(&self.id)
                    .map(|provider| (ProviderEvidence::Inferred, provider.to_string()))
            })
        } else {
            None
        }
        .unwrap_or_else(|| (ProviderEvidence::Unattributed, QODER_PROVIDER.to_string()));
        (self.id, provider, evidence)
    }
}

/// Token counters from one `usage` block, with cache reads inside gross input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct QoderUsage {
    gross_input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
    cache_write_1h: i64,
}

impl QoderUsage {
    fn total(self) -> i64 {
        self.gross_input
            .saturating_add(self.output)
            .saturating_add(self.cache_write)
    }

    fn merge_max(&mut self, other: Self) {
        self.gross_input = self.gross_input.max(other.gross_input);
        self.output = self.output.max(other.output);
        self.cache_read = self.cache_read.max(other.cache_read);
        self.cache_write = self.cache_write.max(other.cache_write);
        self.cache_write_1h = self.cache_write_1h.max(other.cache_write_1h);
    }

    fn to_breakdown(self) -> TokenBreakdown {
        TokenBreakdown {
            input: self.gross_input - self.cache_read,
            output: self.output,
            cache_read: self.cache_read,
            cache_write: self.cache_write,
            reasoning: 0,
        }
    }
}

fn first_i64(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| extract_i64(value.get(*key)))
}

/// Parse one `usage`-shaped JSON object into raw prompt and token counters.
///
/// Returns `None` when the block names no token counter at all (the managed
/// model credits-only shape) or when every counter is zero — neither is
/// measured usage, and neither may be estimated.
fn usage_from_value(usage: &Value) -> Option<QoderUsage> {
    if !usage.is_object() {
        return None;
    }

    let input_raw = first_i64(
        usage,
        &[
            "input_tokens",
            "inputTokens",
            "prompt_tokens",
            "promptTokens",
            "promptTokenCount",
        ],
    )
    .unwrap_or(0);
    let output_raw = first_i64(
        usage,
        &[
            "output_tokens",
            "outputTokens",
            "completion_tokens",
            "completionTokens",
            "candidatesTokenCount",
        ],
    )
    .unwrap_or(0);
    let cache_read_raw = first_i64(
        usage,
        &[
            "cache_read_input_tokens",
            "cacheReadInputTokens",
            "cache_read_tokens",
            "cacheReadTokens",
            "cached_tokens",
            "cachedTokens",
            "cachedContentTokenCount",
        ],
    )
    .unwrap_or(0);
    let cache_write_raw = first_i64(
        usage,
        &[
            "cache_creation_input_tokens",
            "cacheCreationInputTokens",
            "cache_write_tokens",
            "cacheWriteTokens",
        ],
    )
    .unwrap_or(0);

    let input_raw = input_raw.max(0);
    let output = output_raw.max(0);
    let cache_read = cache_read_raw.max(0).min(input_raw);
    let cache_write = cache_write_raw.max(0);
    let cache_write_1h = cache_write_1h_from_value(usage, cache_write);

    let usage = QoderUsage {
        gross_input: input_raw,
        output,
        cache_read,
        cache_write,
        cache_write_1h,
    };
    (usage.total() > 0).then_some(usage)
}

/// The 1-hour cache-write subset, clamped to the recorded cache-write total.
fn cache_write_1h_from_value(usage: &Value, cache_write: i64) -> i64 {
    let parsed = usage
        .get("cache_creation")
        .and_then(|split| {
            first_i64(
                split,
                &["ephemeral_1h_input_tokens", "ephemeral1hInputTokens"],
            )
        })
        .unwrap_or(0)
        .max(0);
    parsed.min(cache_write)
}

fn usage_request_id(usage: &Value) -> Option<&str> {
    ["request_id", "requestId"]
        .iter()
        .find_map(|key| usage.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

/// One source's account of a request, with evidence used only by the Qoder merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoderObservation {
    message: UnifiedMessage,
    measured_timestamp: bool,
    model_rank: ModelRank,
    provider_evidence: ProviderEvidence,
}

impl QoderObservation {
    pub fn as_message(&self) -> &UnifiedMessage {
        &self.message
    }

    pub fn into_message(self) -> UnifiedMessage {
        self.message
    }
}

/// One assembled assistant turn.
struct Turn {
    /// Identity recorded in the dedup key: the request id when the reply
    /// carried one, else the message id, else the file-position anchor.
    key: String,
    session_id: String,
    agent: Option<String>,
    workspace_key: Option<String>,
    workspace_label: Option<String>,
    model: RequestModel,
    prompt_timestamp: Option<i64>,
    timestamp: Option<i64>,
    last_timestamp: Option<i64>,
    is_turn_start: bool,
    usage: Option<QoderUsage>,
}

impl Turn {
    fn observe_time(&mut self, timestamp: i64) {
        self.timestamp = Some(self.timestamp.map_or(timestamp, |start| start.min(timestamp)));
    }

    fn into_observation(
        self,
        source: QoderSource,
        fallback_timestamp: i64,
    ) -> Option<QoderObservation> {
        let usage = self.usage?;
        if usage.total() <= 0 {
            return None;
        }
        let measured_timestamp = self.prompt_timestamp.or(self.timestamp);
        let timestamp = measured_timestamp.unwrap_or(fallback_timestamp);
        let model_rank = self.model.rank;
        let (model_id, provider_id, provider_evidence) = self.model.into_ids();
        let mut message = UnifiedMessage::new_with_agent(
            CLIENT_ID,
            model_id,
            provider_id,
            self.session_id,
            timestamp,
            usage.to_breakdown(),
            0.0,
            self.agent,
        );
        message.dedup_key = Some(source.dedup_key(format_args!("{}", self.key)));
        message.duration_ms = self
            .last_timestamp
            .and_then(|last| last.checked_sub(timestamp))
            .filter(|span| *span > 0);
        message.is_turn_start = self.is_turn_start;
        message.cache_write_1h = usage.cache_write_1h;
        message.set_workspace(self.workspace_key, self.workspace_label);
        Some(QoderObservation {
            message,
            measured_timestamp: measured_timestamp.is_some(),
            model_rank,
            provider_evidence,
        })
    }
}

fn workspace_for_cwd(cwd: &str) -> Option<(String, Option<String>)> {
    let key = normalize_workspace_key(cwd)?;
    let label = workspace_label_from_key(&key);
    Some((key, label))
}

/// Project workspace from a transcript path.
///
/// `<config>/projects/<project>/<session>.jsonl` and
/// `<project>/<session>/subagents/agent-*.jsonl` both name the project
/// directory. A pre-project-version transcript directly under `projects/` has
/// no project, and the layout directory itself must never be reported as one.
fn workspace_key_from_qoder_path(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let components: Vec<&str> = parent
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect();

    for window in components.windows(2).rev() {
        if window[0].eq_ignore_ascii_case("projects")
            && !window[1].is_empty()
            && !window[1].eq_ignore_ascii_case("subagents")
        {
            if let Some(key) = normalize_workspace_key(window[1]) {
                return Some(key);
            }
        }
    }

    None
}

/// Session id recorded in a segment path:
/// `<config>/logs/sessions/<project>/<session>/segments/<file>.jsonl`.
fn segment_session_id(path: &Path) -> Option<String> {
    path.parent()?
        .parent()?
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn subagent_id(path: &Path) -> Option<&str> {
    path.file_stem()?
        .to_str()?
        .strip_prefix("agent-")
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

/// Sidecar used for subagent attribution, whether or not it exists yet.
pub(crate) fn subagent_meta_path(path: &Path) -> Option<PathBuf> {
    if is_qoder_segment_path(path) || should_skip_qoder_file(path) {
        return None;
    }
    let is_subagent = subagent_id(path).is_some()
        || path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("subagents"))
        });
    is_subagent.then(|| path.with_extension("meta.json"))
}

/// Subagent identity for a transcript: `(normalized name, agent id)`.
fn subagent_context(path: &Path) -> Option<(String, Option<String>)> {
    let meta_path = subagent_meta_path(path)?;
    let agent_id = subagent_id(path);
    if let Ok(text) = std::fs::read_to_string(meta_path) {
        if let Ok(meta) = serde_json::from_str::<Value>(&text) {
            let agent_type = meta
                .get("agentType")
                .or_else(|| meta.get("agent_type"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|agent| !agent.is_empty());
            if let Some(agent_type) = agent_type {
                return Some((normalize_agent_name(agent_type), agent_id.map(str::to_owned)));
            }
        }
    }

    Some((
        normalize_agent_name(agent_id.unwrap_or("qoder-subagent")),
        agent_id.map(str::to_owned),
    ))
}

fn content_len(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::Array(blocks)) if !blocks.is_empty())
        || matches!(value, Some(Value::String(text)) if !text.trim().is_empty())
}

fn is_tool_result_only(content: Option<&Value>) -> bool {
    let Some(blocks) = content.and_then(Value::as_array) else {
        return false;
    };
    let mut has_tool_result = false;
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("tool_result") => has_tool_result = true,
            Some(_) => return false,
            None => return false,
        }
    }
    has_tool_result
}

/// Parse a Qoder or Qoder CN transcript into one observation per model request.
pub fn parse_qoder_transcript_file(path: &Path, source: QoderSource) -> Vec<QoderObservation> {
    if is_qoder_segment_path(path) {
        return Vec::new();
    }
    scan_qoder_jsonl_file(path, source).0
}

fn parse_qoder_transcript(
    path: &Path,
    source: QoderSource,
    reader: &mut impl BufRead,
) -> (Vec<QoderObservation>, bool) {
    let fallback_timestamp = file_modified_timestamp_ms(path);
    let fallback_session_id = super::utils::session_id_from_path(path);
    let path_workspace_key = workspace_key_from_qoder_path(path);
    let subagent = subagent_context(path);

    let mut session_id: Option<String> = None;
    let mut workspace: Option<(String, Option<String>)> = path_workspace_key
        .as_deref()
        .map(|key| (key.to_string(), workspace_label_from_key(key)));

    // Session-wide model state.
    let mut runtime_model: Option<String> = None;
    let mut runtime_provider: Option<String> = None;
    let mut last_business_model: Option<String> = None;
    let mut active_model: Option<String> = None;

    // Per-turn state.
    let mut pending_turn_start = false;
    let mut pending_prompt_ms: Option<i64> = None;

    let mut turns: Vec<Turn> = Vec::new();
    let mut index_by_key: HashMap<String, usize> = HashMap::new();
    let mut anonymous_path_hash = None;

    let read_result = read_json_lines(reader, &mut |line_index, trimmed| {
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            return;
        };
        let entry_type = value
            .get("type")
            .or_else(|| value.get("role"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let message = value.get("message");

        if let Some(recorded) = value
            .get("sessionId")
            .or_else(|| value.get("session_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|id| !id.is_empty() && session_id.as_deref() != Some(*id))
        {
            session_id = Some(recorded.to_string());
        }

        if let Some(cwd) = value
            .get("cwd")
            .or_else(|| value.get("relocatedCwd"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|cwd| !cwd.is_empty())
        {
            if let Some(resolved) = workspace_for_cwd(cwd) {
                workspace = Some(resolved);
            }
        }

        match entry_type {
            "runtime-config" => {
                if let Some(model) = value
                    .get("model")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|model| is_selected_model(model))
                {
                    if !runtime_model
                        .as_deref()
                        .is_some_and(|current| same_runtime_model(current, model))
                    {
                        runtime_model = Some(model.to_string());
                        runtime_provider = None;
                    }
                    last_business_model = None;
                }
                if let Some(provider) = value
                    .get("provider")
                    .or_else(|| value.get("providerId"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|provider| !provider.is_empty() && runtime_model.is_some())
                {
                    runtime_provider = canonical_qoder_provider(provider);
                }
            }
            "user" => {
                let business_model = value
                    .get("businessModelKey")
                    .or_else(|| message.and_then(|message| message.get("businessModelKey")))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|model| is_selected_model(model));
                if let Some(business_model) = business_model {
                    last_business_model = Some(business_model.to_string());
                }

                let content = message
                    .and_then(|message| message.get("content"))
                    .or_else(|| value.get("content"));
                let is_synthetic = value
                    .get("isSynthetic")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    || value
                        .get("isMeta")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                if !is_synthetic && !is_tool_result_only(content) && content_len(content) {
                    pending_turn_start = true;
                    pending_prompt_ms = value.get("timestamp").and_then(parse_timestamp_value);
                }
            }
            "assistant" => {
                if value
                    .get("isApiErrorMessage")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return;
                }

                let fragment_model = message
                    .and_then(|message| message.get("model"))
                    .or_else(|| value.get("model"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|model| !is_synthetic_placeholder_model(model));

                let fragment_business = value
                    .get("businessModelKey")
                    .or_else(|| message.and_then(|message| message.get("businessModelKey")))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|model| is_selected_model(model));

                if let Some(business_model) = fragment_business {
                    last_business_model = Some(business_model.to_string());
                }
                let reported_model = fragment_model.filter(|model| is_selected_model(model));
                if let Some(model) =
                    reported_model.filter(|model| active_model.as_deref() != Some(*model))
                {
                    active_model = Some(model.to_string());
                }

                let fragment_selection = reported_model
                    .map(|model| (model, ModelEvidence::Reported))
                    .or_else(|| {
                        fragment_business.map(|model| (model, ModelEvidence::BusinessFallback))
                    });

                let explicit_provider = message
                    .and_then(|message| {
                        message
                            .get("providerId")
                            .or_else(|| message.get("provider_id"))
                            .or_else(|| message.get("provider"))
                    })
                    .or_else(|| {
                        value
                            .get("providerId")
                            .or_else(|| value.get("provider_id"))
                            .or_else(|| value.get("provider"))
                    })
                    .and_then(Value::as_str);

                let usage_value = message
                    .and_then(|message| message.get("usage"))
                    .or_else(|| value.get("usage"))
                    .or_else(|| value.get("token_usage"));
                let usage = usage_value.and_then(usage_from_value);

                let fragment_ts = value.get("timestamp").and_then(parse_timestamp_value);

                let message_id = message
                    .and_then(|message| message.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|id| !id.is_empty());
                let request_id = value
                    .get("requestId")
                    .or_else(|| value.get("request_id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .or_else(|| usage_value.and_then(usage_request_id));
                let uuid = value
                    .get("uuid")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|uuid| !uuid.is_empty());

                let group_key = if let Some(message_id) = message_id {
                    format!("msg:{message_id}")
                } else if let Some(request_id) = request_id {
                    format!("req:{request_id}")
                } else if let Some(uuid) = uuid {
                    format!("uuid:{uuid}")
                } else {
                    anonymous_key(path, &mut anonymous_path_hash, line_index)
                };

                if let Some(existing) = index_by_key.get(&group_key).copied() {
                    let turn = &mut turns[existing];
                    if let Some(request_id) =
                        request_id.filter(|id| turn.key.strip_prefix("req:") != Some(*id))
                    {
                        turn.key = format!("req:{request_id}");
                    }
                    turn.model.merge_fragment(
                        fragment_selection.or_else(|| {
                            fragment_model.map(|model| (model, ModelEvidence::Placeholder))
                        }),
                        explicit_provider,
                    );
                    if let Some(usage) = usage {
                        match &mut turn.usage {
                            Some(existing) => existing.merge_max(usage),
                            None => turn.usage = Some(usage),
                        }
                    }
                    if let Some(ts) = fragment_ts {
                        turn.observe_time(ts);
                        turn.last_timestamp =
                            Some(turn.last_timestamp.map_or(ts, |last| last.max(ts)));
                    }
                    pending_turn_start = false;
                    pending_prompt_ms = None;
                    return;
                }

                let resolved_session = session_id.as_deref().unwrap_or(&fallback_session_id);

                let is_sidechain = value
                    .get("isSidechain")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let entry_agent_id = value
                    .get("agentId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|id| !id.is_empty());

                let (agent, effective_session) = match (&subagent, is_sidechain) {
                    (Some((name, id)), _) => {
                        let child_session = id
                            .as_deref()
                            .or(entry_agent_id)
                            .map(|id| format!("{resolved_session}:agent:{id}"))
                            .unwrap_or_else(|| resolved_session.to_string());
                        (Some(name.clone()), child_session)
                    }
                    (None, true) => {
                        let child_session = entry_agent_id
                            .map(|id| format!("{resolved_session}:agent:{id}"))
                            .unwrap_or_else(|| resolved_session.to_string());
                        (Some(normalize_agent_name("qoder-subagent")), child_session)
                    }
                    (None, false) => (None, resolved_session.to_string()),
                };

                let (raw_model, evidence) = fragment_selection
                    .or_else(|| {
                        last_business_model
                            .as_deref()
                            .map(|model| (model, ModelEvidence::InheritedBusinessFallback))
                    })
                    .or_else(|| {
                        runtime_model
                            .as_deref()
                            .or(active_model.as_deref())
                            .map(|model| (model, ModelEvidence::SessionDefault))
                    })
                    .unwrap_or((
                        fragment_model.unwrap_or(DEFAULT_MODEL),
                        ModelEvidence::Placeholder,
                    ));
                let scoped_provider = runtime_model
                    .as_deref()
                    .filter(|model| same_runtime_model(model, raw_model))
                    .and(runtime_provider.as_deref());
                let model = RequestModel::new(raw_model, evidence, explicit_provider, scoped_provider);
                let turn_key = request_id.map_or_else(|| group_key.clone(), |id| format!("req:{id}"));
                index_by_key.insert(group_key, turns.len());
                turns.push(Turn {
                    key: turn_key,
                    session_id: effective_session,
                    agent,
                    workspace_key: workspace.as_ref().map(|(key, _)| key.clone()),
                    workspace_label: workspace.as_ref().and_then(|(_, label)| label.clone()),
                    model,
                    prompt_timestamp: pending_prompt_ms,
                    timestamp: fragment_ts,
                    last_timestamp: fragment_ts,
                    is_turn_start: pending_turn_start,
                    usage,
                });

                pending_turn_start = false;
                pending_prompt_ms = None;
            }
            _ => {}
        }
    });

    let (messages, without_usage) = collect_measured_turns(turns, source, fallback_timestamp);
    if without_usage > 0 {
        // Not a fault: managed-model rows record credits and zero out (or omit)
        // every token counter. Saying so at debug level is the whole diagnostic
        // story — the rows are deliberately absent, never estimated.
        tracing::debug!(
            path = %path.display(),
            turns_without_recorded_usage = without_usage,
            "Qoder transcript rows with no recorded token counts were skipped"
        );
    }
    (messages, read_result.is_ok())
}

/// Convert measured turns to observations, reporting how many had no recorded usage.
fn collect_measured_turns(
    turns: Vec<Turn>,
    source: QoderSource,
    fallback_timestamp: i64,
) -> (Vec<QoderObservation>, usize) {
    let mut messages = Vec::with_capacity(turns.len());
    let mut without_usage = 0usize;
    for turn in turns {
        match turn.into_observation(source, fallback_timestamp) {
            Some(message) => messages.push(message),
            None => without_usage += 1,
        }
    }
    (messages, without_usage)
}

fn anonymous_key(path: &Path, path_hash: &mut Option<String>, line_index: usize) -> String {
    // The file scopes the line number; a hash keeps private paths out of keys.
    let path_hash = path_hash.get_or_insert_with(|| {
        format!("{:x}", Sha256::digest(path.as_os_str().as_encoded_bytes()))
    });
    format!("anon:{path_hash}:{line_index}")
}

/// Parse a structured segment log into one observation per completed model request.
///
/// Only `model.response.completed` events are per-request records. The
/// `turn.finished` roll-up carries the same counters a second time and is
/// deliberately ignored.
pub fn parse_qoder_segment_file(path: &Path, source: QoderSource) -> Vec<QoderObservation> {
    if !is_qoder_segment_path(path) {
        return Vec::new();
    }
    scan_qoder_jsonl_file(path, source).0
}

fn parse_qoder_segment(
    path: &Path,
    source: QoderSource,
    reader: &mut impl BufRead,
) -> (Vec<QoderObservation>, bool) {
    let fallback_timestamp = file_modified_timestamp_ms(path);
    let fallback_session = segment_session_id(path)
        .or_else(|| path.parent().map(super::utils::session_id_from_path))
        .unwrap_or_else(|| "unknown".to_string());

    let mut session_id: Option<String> = None;
    let mut workspace: Option<(String, Option<String>)> = None;
    let mut requests: Vec<Turn> = Vec::new();
    let mut index_by_key: HashMap<String, usize> = HashMap::new();
    let mut anonymous_path_hash = None;

    let read_result = read_json_lines(reader, &mut |line_index, trimmed| {
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            return;
        };
        let entry_type = value.get("type").and_then(Value::as_str).unwrap_or("");

        if entry_type == "session.config.loaded" {
            if workspace.is_none() {
                if let Some(data) = value.get("data") {
                    for key in ["project_root", "target_dir"] {
                        if let Some(root) = data.get(key).and_then(Value::as_str) {
                            if let Some(resolved) = workspace_for_cwd(root) {
                                workspace = Some(resolved);
                                break;
                            }
                        }
                    }
                }
            }
            return;
        }

        if entry_type != "model.response.completed" {
            return;
        }

        let Some(data) = value.get("data") else {
            return;
        };
        let Some(usage) = usage_from_value(data) else {
            return;
        };

        let recorded_session = value
            .get("session_id")
            .or_else(|| value.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|id| !id.is_empty());
        if let Some(recorded) = recorded_session {
            session_id = Some(recorded.to_string());
        }

        let request_id = value
            .get("request_id")
            .or_else(|| data.get("request_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string);

        let event_key = match &request_id {
            Some(request_id) => format!("req:{request_id}"),
            None => anonymous_key(path, &mut anonymous_path_hash, line_index),
        };

        let timestamp = value
            .get("ts")
            .or_else(|| value.get("timestamp"))
            .and_then(parse_timestamp_value);

        let raw_model = data
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|model| !is_synthetic_placeholder_model(model));
        let explicit_provider = data.get("provider").and_then(Value::as_str);

        if let Some(existing) = index_by_key.get(&event_key).copied() {
            let turn = &mut requests[existing];
            if let Some(existing_usage) = &mut turn.usage {
                existing_usage.merge_max(usage);
            } else {
                turn.usage = Some(usage);
            }
            turn.model.merge_fragment(
                raw_model.map(|model| (model, ModelEvidence::Reported)),
                explicit_provider,
            );
            if let Some(timestamp) = timestamp {
                turn.observe_time(timestamp);
            }
            return;
        }

        index_by_key.insert(event_key.clone(), requests.len());
        requests.push(Turn {
            key: event_key,
            session_id: session_id
                .clone()
                .unwrap_or_else(|| fallback_session.clone()),
            agent: None,
            workspace_key: workspace.as_ref().map(|(key, _)| key.clone()),
            workspace_label: workspace.as_ref().and_then(|(_, label)| label.clone()),
            model: RequestModel::new(
                raw_model.unwrap_or(DEFAULT_MODEL),
                ModelEvidence::Reported,
                explicit_provider,
                None,
            ),
            prompt_timestamp: None,
            timestamp,
            last_timestamp: None,
            is_turn_start: false,
            usage: Some(usage),
        });
    });

    let (messages, without_usage) = collect_measured_turns(requests, source, fallback_timestamp);
    if without_usage > 0 {
        tracing::debug!(
            path = %path.display(),
            requests_without_recorded_usage = without_usage,
            "Qoder segment events with no recorded token counts were skipped"
        );
    }
    (messages, read_result.is_ok())
}

/// One IDE database read.
pub struct QoderIdeScan {
    pub messages: Vec<QoderObservation>,
    /// False when the database could not be opened or the scan stopped partway.
    ///
    /// A partial read is reported for this scan but must not be cached: cached,
    /// it would stand in for the whole table on every warm scan until the file
    /// happened to change.
    pub complete: bool,
}

/// Session/project metadata joined onto a `chat_message` row.
struct IdeRow {
    id: String,
    session_id: String,
    request_id: Option<String>,
    token_info: String,
    model_info: Option<String>,
    created: Option<i64>,
    project_uri: Option<String>,
    project_name: Option<String>,
    /// The session's configured model, used only when the message names none.
    preferred_model_info: Option<String>,
}

/// Read one text column, tolerating the numeric/BLOB spellings rusqlite would
/// otherwise reject outright.
fn text_column(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<String>> {
    use rusqlite::types::ValueRef;
    Ok(match row.get_ref(index)? {
        ValueRef::Null => None,
        ValueRef::Text(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
        ValueRef::Integer(value) => Some(value.to_string()),
        ValueRef::Real(value) => Some(value.to_string()),
        ValueRef::Blob(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
    })
}

/// Read the message creation time, which the IDE stores as milliseconds but
/// some builds persist as a numeric string.
fn timestamp_column(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<i64>> {
    use rusqlite::types::ValueRef;
    Ok(match row.get_ref(index)? {
        ValueRef::Null => None,
        ValueRef::Integer(value) => Some(value),
        ValueRef::Real(value) => Some(value as i64),
        ValueRef::Text(bytes) => std::str::from_utf8(bytes)
            .ok()
            .and_then(|text| extract_i64(Some(&Value::String(text.trim().to_string())))),
        ValueRef::Blob(_) => None,
    })
}

fn collect_ide_rows(rows: &mut Vec<IdeRow>, row: &rusqlite::Row<'_>) -> rusqlite::Result<()> {
    rows.push(IdeRow {
        id: text_column(row, 0)?.unwrap_or_default(),
        session_id: text_column(row, 1)?.unwrap_or_default(),
        request_id: text_column(row, 2)?,
        token_info: text_column(row, 3)?.unwrap_or_default(),
        model_info: text_column(row, 4)?,
        created: timestamp_column(row, 5)?,
        project_uri: text_column(row, 6)?,
        project_name: text_column(row, 7)?,
        preferred_model_info: text_column(row, 8)?,
    });
    Ok(())
}

/// Read the IDE's local `chat_message` table.
///
/// Only token/model/timing columns are selected. Message bodies, tool output,
/// titles and credentials are never read.
pub fn scan_qoder_ide_db(db_path: &Path, source: QoderSource) -> QoderIdeScan {
    let resolved = db_path.canonicalize().ok();
    let db_path = resolved.as_deref().unwrap_or(db_path);
    with_qoder_ide_connection(db_path, |mut conn| {
        let tx = begin_qoder_ide_read(&mut conn)?;
        Ok(scan_qoder_ide_snapshot(&tx, db_path, source))
    })
    .unwrap_or(QoderIdeScan {
        messages: Vec::new(),
        complete: false,
    })
}

pub(crate) fn with_qoder_ide_connection<T>(
    db_path: &Path,
    read: impl FnOnce(rusqlite::Connection) -> rusqlite::Result<T>,
) -> Option<T> {
    let result = open_readonly_sqlite(db_path).and_then(|conn| {
        conn.busy_timeout(std::time::Duration::from_millis(QODER_SQLITE_BUSY_TIMEOUT_MS))?;
        read(conn)
    });
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::debug!(
                db_path = %db_path.display(),
                %error,
                "Failed to start Qoder IDE database read"
            );
            None
        }
    }
}

pub(crate) fn begin_qoder_ide_read(
    conn: &mut rusqlite::Connection,
) -> rusqlite::Result<rusqlite::Transaction<'_>> {
    let tx = conn.transaction()?;
    // A deferred BEGIN alone does not pin a snapshot of the main database.
    tx.query_row("PRAGMA main.schema_version", [], |_| Ok(()))?;
    Ok(tx)
}

pub(crate) fn scan_qoder_ide_snapshot(
    conn: &rusqlite::Transaction<'_>,
    db_path: &Path,
    source: QoderSource,
) -> QoderIdeScan {

    // The `chat_session` join supplies the project/workspace attribution. Older
    // Qoder CN builds have no `chat_session` table at all; usage is still valid
    // without it, so that case degrades to the un-attributed query instead of
    // dropping the source.
    const JOINED_SQL: &str = "SELECT m.id, m.session_id, m.request_id, m.token_info, m.model_info,          m.gmt_create, s.project_uri, s.project_name, s.preferred_model_info          FROM chat_message AS m LEFT JOIN chat_session AS s ON s.session_id = m.session_id          WHERE m.role = 'assistant'";
    const PLAIN_SQL: &str = "SELECT id, session_id, request_id, token_info, model_info, gmt_create,          NULL, NULL, NULL FROM chat_message WHERE role = 'assistant'";

    let mut rows: Vec<IdeRow> = Vec::new();
    let mut scan = sqlite_for_each_row_on(conn, db_path, JOINED_SQL, None, &mut |row| {
        collect_ide_rows(&mut rows, row)
    });
    if !scan.prepared() {
        // Only use the plain query after confirming older schema. A lock or
        // other transient prepare failure must not cache a metadata-free scan.
        let schema = conn.query_row(
            "SELECT EXISTS (
                 SELECT 1 FROM main.sqlite_schema
                 WHERE name = 'chat_message' COLLATE NOCASE AND type IN ('table', 'view')
             ), (
                 SELECT count(*) = 4 FROM pragma_table_info('chat_session')
                 WHERE name COLLATE NOCASE IN
                     ('session_id', 'project_uri', 'project_name', 'preferred_model_info')
             )",
            [],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?)),
        );
        match schema {
            Ok((false, _)) => {
                return QoderIdeScan {
                    messages: Vec::new(),
                    complete: true,
                };
            }
            Ok((true, false)) => {
                scan = sqlite_for_each_row_on(conn, db_path, PLAIN_SQL, None, &mut |row| {
                    collect_ide_rows(&mut rows, row)
                });
            }
            Ok((true, true)) | Err(_) => {
                tracing::debug!(
                    db_path = %db_path.display(),
                    "Failed to prepare Qoder IDE usage query"
                );
            }
        }
        if !scan.prepared() {
            return QoderIdeScan {
                messages: Vec::new(),
                complete: false,
            };
        }
    }

    let mut messages = Vec::new();
    let mut without_usage = 0usize;
    for row in &rows {
        match ide_row_message(row, source) {
            Some(message) => messages.push(message),
            None => without_usage += 1,
        }
    }
    if without_usage > 0 {
        // Interrupted or errored chats persist an empty `token_info`.
        tracing::debug!(
            db_path = %db_path.display(),
            rows_without_recorded_usage = without_usage,
            "Qoder IDE rows with no recorded token counts were skipped"
        );
    }

    QoderIdeScan {
        messages,
        complete: scan.completed(),
    }
}

fn ide_row_message(row: &IdeRow, source: QoderSource) -> Option<QoderObservation> {
    let tokens = parse_ide_token_info(&row.token_info)?;

    let timestamp = row
        .created
        .filter(|timestamp| *timestamp > 0)
        .map(|timestamp| {
            if timestamp >= 1_000_000_000_000 {
                timestamp
            } else {
                timestamp.saturating_mul(1000)
            }
        })?;

    let model_key = row
        .model_info
        .as_deref()
        .and_then(parse_ide_model_info)
        .map(|model| (model, ModelEvidence::Reported))
        .or_else(|| {
            row.preferred_model_info
                .as_deref()
                .and_then(parse_ide_model_info)
                .map(|model| (model, ModelEvidence::SessionDefault))
        })
        .filter(|(model, _)| !is_synthetic_placeholder_model(model));
    let (raw_model, evidence) = model_key
        .as_ref()
        .map_or((DEFAULT_MODEL, ModelEvidence::Placeholder), |(model, evidence)| {
            (model.as_str(), *evidence)
        });
    let model = RequestModel::new(raw_model, evidence, None, None);
    let model_rank = model.rank;
    let (model_id, provider_id, provider_evidence) = model.into_ids();

    let session_id = if row.session_id.trim().is_empty() {
        "unknown".to_string()
    } else {
        row.session_id.trim().to_string()
    };
    let dedup_key = match row.request_id.as_deref().map(str::trim) {
        Some(request_id) if !request_id.is_empty() => {
            source.dedup_key(format_args!("req:{request_id}"))
        }
        _ => {
            let id = row.id.trim();
            if id.is_empty() {
                source.dedup_key(format_args!("ide:{session_id}:{timestamp}"))
            } else {
                source.dedup_key(format_args!("ide:{id}"))
            }
        }
    };

    let mut message = UnifiedMessage::new_with_dedup(
        CLIENT_ID,
        model_id,
        provider_id,
        session_id,
        timestamp,
        tokens.to_breakdown(),
        0.0,
        Some(dedup_key),
    );
    message.cache_write_1h = tokens.cache_write_1h;
    message.set_workspace(ide_workspace_key(row), None);
    Some(QoderObservation {
        message,
        measured_timestamp: true,
        model_rank,
        provider_evidence,
    })
}

fn ide_workspace_key(row: &IdeRow) -> Option<String> {
    let uri = row
        .project_uri
        .as_deref()
        .map(str::trim)
        .filter(|uri| !uri.is_empty());
    if let Some(uri) = uri {
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        if let Some(key) = normalize_workspace_key(path) {
            return Some(key);
        }
    }
    // `.` is Qoder's "no project" sentinel.
    row.project_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != ".")
        .map(str::to_string)
}

fn parse_ide_json(value: &str) -> Option<Value> {
    let parsed: Value = serde_json::from_str(value).ok()?;
    parsed.is_object().then_some(parsed)
}

fn parse_ide_model_info(value: &str) -> Option<String> {
    let info = parse_ide_json(value)?;
    ["model_key", "modelKey", "model"]
        .iter()
        .find_map(|key| info.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string)
}

/// `{"prompt_tokens":39005,"cached_tokens":37485,"completion_tokens":67}`.
///
/// `prompt_tokens` already includes `cached_tokens`, so the net input is the
/// difference. A row with no counters at all (an interrupted or errored chat)
/// is not measured usage.
fn parse_ide_token_info(value: &str) -> Option<QoderUsage> {
    if value.trim().is_empty() {
        return None;
    }
    usage_from_value(&parse_ide_json(value)?)
}

/// Candidate IDE databases, in preference order.
///
/// `QODER_HOME` / `QODER_CN_HOME` are Qoder's own IDE data-root overrides: when
/// one is set it replaces that edition's default location (as the community
/// implementations read it) rather than adding to it, so a redirected install
/// never reaches into the machine's real profile. Each override is accepted in
/// both documented spellings — the data root itself, or the full
/// `SharedClientCache/...` parent.
pub fn qoder_ide_db_candidates(
    app_data_root: &Path,
    use_env_roots: bool,
) -> Vec<(PathBuf, QoderSource)> {
    const RELATIVE_DB: &str = "SharedClientCache/cache/db/local.db";

    let env_root = |var: &str| -> Option<PathBuf> {
        if !use_env_roots {
            return None;
        }
        let value = std::env::var(var).ok()?;
        let value = value.trim();
        (!value.is_empty()).then(|| PathBuf::from(value))
    };

    let mut candidates = Vec::new();
    for (var, edition_dir, source) in [
        ("QODER_HOME", "Qoder", QoderSource::International),
        ("QODER_CN_HOME", "QoderCN", QoderSource::China),
    ] {
        match env_root(var) {
            Some(root) => {
                candidates.push((root.join("cache/db/local.db"), source));
                candidates.push((root.join(RELATIVE_DB), source));
            }
            None => candidates.push((app_data_root.join(edition_dir).join(RELATIVE_DB), source)),
        }
    }
    candidates
}

/// Parse and deduplicate all Qoder sources before emitting report messages.
pub(crate) fn parse_qoder_lane(
    scan: &ScanResult,
    cache: Option<&mut SourceMessageCache>,
) -> Vec<UnifiedMessage> {
    let cache = cache.and_then(|cache| cache.qoder_cache());
    if let Some(cache) = &cache {
        cache.prune_orphans();
    }
    let paths = scan.get(ClientId::Qoder);
    if paths.is_empty() && scan.qoder_ide_dbs.is_empty() {
        return Vec::new();
    }
    let sources = paths
        .par_iter()
        .map(|path| (path.as_path(), scan.qoder_source(path)))
        .chain(
            scan.qoder_ide_dbs
                .par_iter()
                .map(|(path, source)| (path.as_path(), *source)),
        );
    let observations = sources
        .flat_map_iter(|(path, source)| {
            if is_qoder_ide_db(path) {
                return match &cache {
                    Some(cache) => cache.load_or_scan_ide(path, source),
                    None => scan_qoder_ide_db(path, source).messages,
                };
            }
            // Use one path for the cache key, sidecar fingerprint and parse.
            // Canonicalization would lose the link's project and sidecar paths.
            let Ok(absolute) = std::path::absolute(path) else {
                return scan_qoder_jsonl_file(path, source).0;
            };
            match &cache {
                Some(cache) => cache.load_or_parse(&absolute, source, scan_qoder_jsonl_file),
                None => scan_qoder_jsonl_file(&absolute, source).0,
            }
        })
        .collect();
    collapse_qoder_messages(observations)
}

fn scan_qoder_jsonl_file(path: &Path, source: QoderSource) -> (Vec<QoderObservation>, bool) {
    if should_skip_qoder_file(path) {
        return (Vec::new(), true);
    }
    let Ok(file) = File::open(path) else {
        return (Vec::new(), false);
    };
    scan_qoder_jsonl(path, source, &mut BufReader::new(file))
}

// Keep observations from the readable prefix, but only EOF makes it cacheable.
fn scan_qoder_jsonl(
    path: &Path,
    source: QoderSource,
    reader: &mut impl BufRead,
) -> (Vec<QoderObservation>, bool) {
    if is_qoder_segment_path(path) {
        parse_qoder_segment(path, source, reader)
    } else {
        parse_qoder_transcript(path, source, reader)
    }
}

fn merge_model_evidence(observations: &mut [QoderObservation]) {
    if observations.len() < 2 {
        return;
    }
    let (model_index, model) = observations
        .iter()
        .enumerate()
        .max_by_key(|(index, observation)| (observation.model_rank, Reverse(*index)))
        .expect("a request has observations");
    let (provider_index, _) = observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| {
            observation.message.model_id.eq_ignore_ascii_case(&model.message.model_id)
        })
        .max_by(|(_, left), (_, right)| {
            left.provider_evidence
                .cmp(&right.provider_evidence)
                .then_with(|| right.message.provider_id.cmp(&left.message.provider_id))
        })
        .expect("the winning model has a provider, even if unattributed");

    if model_index != 0 {
        observations[0].message.model_id =
            std::mem::take(&mut observations[model_index].message.model_id);
    }
    if provider_index != 0 {
        observations[0].message.provider_id =
            std::mem::take(&mut observations[provider_index].message.provider_id);
    }
}

/// Merge duplicates and order the lane deterministically.
///
/// Qoder writes the same request more than once — a forked session replays the
/// parent's turns, a segment log repeats a completed event, and one request can
/// appear across all three sources. The edition-scoped request key counts each
/// request once regardless of the order files were scanned in.
///
/// Keep the earliest measured timestamp and its session, or use mtime if
/// nothing was measured. Select the best model, then its best provider evidence.
/// Merge gross prompt and cache reads separately before deriving net input.
/// Callers price the resulting messages, never the individual observations.
pub fn collapse_qoder_messages(mut observations: Vec<QoderObservation>) -> Vec<UnifiedMessage> {
    observations.sort_unstable_by(|left, right| {
        let time_order = right.measured_timestamp.cmp(&left.measured_timestamp);
        let left = &left.message;
        let right = &right.message;
        left.dedup_key
            .cmp(&right.dedup_key)
            .then(time_order)
            .then_with(|| left.timestamp.cmp(&right.timestamp))
            .then_with(|| left.session_id.cmp(&right.session_id))
            .then_with(|| left.model_id.cmp(&right.model_id))
            .then_with(|| left.provider_id.cmp(&right.provider_id))
            .then_with(|| left.workspace_key.cmp(&right.workspace_key))
            .then_with(|| left.workspace_label.cmp(&right.workspace_label))
            .then_with(|| left.agent.cmp(&right.agent))
            .then_with(|| left.duration_ms.cmp(&right.duration_ms))
    });
    // Resolve the final model first so evidence for it is not lost if another
    // model with intermediate rank appears between two observations.
    for group in observations.chunk_by_mut(|left, right| {
        left.message.dedup_key.is_some() && left.message.dedup_key == right.message.dedup_key
    }) {
        merge_model_evidence(group);
    }
    observations.dedup_by(|candidate, existing| {
        let candidate = &mut candidate.message;
        let existing = &mut existing.message;
        if existing.dedup_key.is_none() || existing.dedup_key != candidate.dedup_key {
            return false;
        }

        let tokens = &mut existing.tokens;
        let other = &candidate.tokens;
        let gross_input = tokens
            .input
            .saturating_add(tokens.cache_read)
            .max(other.input.saturating_add(other.cache_read));
        tokens.cache_read = tokens.cache_read.max(other.cache_read);
        tokens.input = gross_input - tokens.cache_read;
        tokens.output = tokens.output.max(other.output);
        tokens.cache_write = tokens.cache_write.max(other.cache_write);
        tokens.reasoning = tokens.reasoning.max(other.reasoning);
        existing.cache_write_1h = existing.cache_write_1h.max(candidate.cache_write_1h);
        existing.is_turn_start |= candidate.is_turn_start;
        if existing.duration_ms.is_none() {
            existing.duration_ms = candidate.duration_ms;
        }
        if candidate.workspace_key.is_some() {
            if existing.workspace_key.is_none() {
                existing.set_workspace(
                    candidate.workspace_key.take(),
                    candidate.workspace_label.take(),
                );
            } else if existing.workspace_key == candidate.workspace_key
                && existing.workspace_label.is_none()
            {
                existing.workspace_label = candidate.workspace_label.take();
            }
        }
        if existing.agent.is_none() {
            existing.agent = candidate.agent.take();
        }
        true
    });
    let mut messages: Vec<_> = observations
        .into_iter()
        .map(QoderObservation::into_message)
        .collect();
    messages.sort_unstable_by(|left, right| {
        left.timestamp
            .cmp(&right.timestamp)
            .then_with(|| left.dedup_key.cmp(&right.dedup_key))
    });
    messages
}

/// Whether an explicit extra path names a Qoder IDE database.
pub fn is_qoder_ide_db(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "local.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_transcript(dir: &Path, name: &str, lines: &[&str]) -> PathBuf {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
        path
    }

    #[test]
    fn usage_treats_prompt_as_including_the_cached_subset() {
        let usage: Value = serde_json::from_str(
            r#"{"input_tokens":39005,"output_tokens":67,"cache_read_input_tokens":37485,
                "cache_creation_input_tokens":0}"#,
        )
        .unwrap();
        let parsed = usage_from_value(&usage).unwrap();
        assert_eq!(parsed.to_breakdown().input, 1520);
        assert_eq!(parsed.cache_read, 37485);
        assert_eq!(parsed.output, 67);
        assert_eq!(parsed.total(), 39072);
    }

    #[test]
    fn usage_ignores_credit_only_and_zero_token_blocks() {
        let credits: Value = serde_json::from_str(
            r#"{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,
                "cache_creation_input_tokens":0,"credits":8.229,"billable":true}"#,
        )
        .unwrap();
        assert_eq!(usage_from_value(&credits), None);

        let credits_only: Value =
            serde_json::from_str(r#"{"credits":8.229,"billable":true,"context_usage_ratio":0.2}"#)
                .unwrap();
        assert_eq!(usage_from_value(&credits_only), None);
    }

    #[test]
    fn usage_keeps_measured_counters_without_prompt() {
        let output = usage_from_value(&serde_json::json!({"output_tokens": 7})).unwrap();
        assert_eq!((output.gross_input, output.output, output.total()), (0, 7, 7));

        let write = usage_from_value(&serde_json::json!({
            "cache_creation_input_tokens": 11,
            "cache_creation": {"ephemeral_1h_input_tokens": 5}
        }))
        .unwrap();
        assert_eq!((write.gross_input, write.cache_write_1h, write.total()), (0, 5, 11));
        assert!(usage_from_value(&serde_json::json!({"cache_read_input_tokens": 20})).is_none());
    }

    #[test]
    fn usage_clamps_impossible_cache_and_1h_subsets() {
        let usage: Value = serde_json::from_str(
            r#"{"input_tokens":100,"output_tokens":10,"cache_read_input_tokens":500,
                "cache_creation_input_tokens":20,
                "cache_creation":{"ephemeral_1h_input_tokens":80}}"#,
        )
        .unwrap();
        let parsed = usage_from_value(&usage).unwrap();
        // A cached subset larger than the prompt cannot be gross input, so the
        // prompt is kept whole and the subset is clamped to it.
        assert_eq!(parsed.cache_read, 100);
        assert_eq!(parsed.to_breakdown().input, 0);
        assert_eq!(parsed.cache_write, 20);
        assert_eq!(parsed.cache_write_1h, 20);
    }

    #[test]
    fn routing_slots_never_borrow_a_vendor() {
        let resolve = |model| RequestModel::new(model, ModelEvidence::Reported, None, None).into_ids();
        for slot in [
            "auto",
            "qmodel",
            "qmodel_38max",
            "qfmodel",
            "dfmodel",
            "gfmodel",
        ] {
            assert_eq!(resolve(slot), (slot.to_string(), QODER_PROVIDER.to_string(), ProviderEvidence::Unattributed));
        }
        assert_eq!(
            resolve("deepseek/chat/deepseek-v3"),
            ("deepseek-v3".to_string(), "deepseek".to_string(), ProviderEvidence::ModelPrefix)
        );
        assert_eq!(
            resolve("qoder/claude-sonnet-4-6"),
            ("claude-sonnet-4-6".to_string(), "anthropic".to_string(), ProviderEvidence::Inferred)
        );
        assert_eq!(
            resolve("qoder-cn/qwen3-coder-plus"),
            ("qwen3-coder-plus".to_string(), "qwen".to_string(), ProviderEvidence::Inferred)
        );
    }

    #[test]
    fn parses_multi_fragment_turn_once_with_request_key() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join(".qoder/projects/-work-app");
        let path = write_transcript(
            &project,
            "sess-1.jsonl",
            &[
                r#"{"type":"runtime-config","sessionId":"sess-1","model":"qmodel","timestamp":1794041573406}"#,
                r#"{"type":"user","uuid":"u1","sessionId":"sess-1","cwd":"/work/app","timestamp":"2026-11-07T10:00:00.000Z","message":{"role":"user","content":"hello"}}"#,
                r#"{"type":"assistant","uuid":"a1","sessionId":"sess-1","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"msg_1","role":"assistant","model":"qmodel","content":[{"type":"thinking","thinking":"plan"}]}}"#,
                r#"{"type":"assistant","uuid":"partial","sessionId":"sess-1","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"msg_1","role":"assistant","model":"qmodel","usage":{"input_tokens":1200,"output_tokens":5}}}"#,
                r#"{"type":"assistant","uuid":"a2","sessionId":"sess-1","timestamp":"2026-11-07T10:00:04.000Z","message":{"id":"msg_1","role":"assistant","model":"qmodel","content":[{"type":"text","text":"done"}],"usage":{"input_tokens":1200,"output_tokens":40,"cache_read_input_tokens":800,"cache_creation_input_tokens":100,"cache_creation":{"ephemeral_1h_input_tokens":50},"request_id":"req_1"}}}"#,
            ],
        );

        let messages = parse_qoder_transcript_file(&path, QoderSource::International);
        assert_eq!(messages.len(), 1);
        let message = messages[0].as_message();
        assert_eq!(message.client, "qoder");
        assert_eq!(message.model_id, "qmodel");
        assert_eq!(message.provider_id, "qoder");
        assert_eq!(message.session_id, "sess-1");
        assert_eq!(message.tokens.input, 400);
        assert_eq!(message.tokens.output, 40);
        assert_eq!(message.tokens.cache_read, 800);
        assert_eq!(message.tokens.cache_write, 100);
        assert_eq!(message.cache_write_1h, 50);
        assert_eq!(message.tokens.reasoning, 0);
        assert_eq!(message.timestamp, 1_794_045_600_000);
        assert_eq!(message.duration_ms, Some(4000));
        assert!(message.is_turn_start);
        assert_eq!(message.workspace_key.as_deref(), Some("/work/app"));
        assert_eq!(message.workspace_label.as_deref(), Some("app"));
        assert_eq!(message.dedup_key.as_deref(), Some("qoder:qoder:req:req_1"));
    }

    #[test]
    fn fills_model_provider_and_real_time_from_late_fragments() {
        let dir = TempDir::new().unwrap();
        let path = write_transcript(
            dir.path(),
            "sess-late.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess-late","message":{"id":"msg_1","model":"<synthetic>","usage":{"input_tokens":1200,"cache_read_input_tokens":800}}}"#,
                r#"{"type":"assistant","timestamp":"2026-11-07T10:00:04Z","message":{"id":"msg_1","model":"auto","usage":{"output_tokens":40}}}"#,
                r#"{"type":"assistant","timestamp":"2026-11-07T10:00:01Z","message":{"id":"msg_1","model":"claude-sonnet-4-6","usage":{"cache_creation_input_tokens":100,"cache_creation":{"ephemeral_1h_input_tokens":50}}}}"#,
                r#"{"type":"assistant","timestamp":"2026-11-07T10:00:05Z","message":{"id":"msg_1","providerId":"openrouter","usage":{"output_tokens":45,"request_id":"req_1"}}}"#,
            ],
        );
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_794_132_000))
            .unwrap();

        let messages = parse_qoder_transcript_file(&path, QoderSource::International);
        assert_eq!(messages.len(), 1);
        let message = messages[0].as_message();
        assert_eq!(message.model_id, "claude-sonnet-4-6");
        assert_eq!(message.provider_id, "openrouter");
        assert_eq!(message.timestamp, 1_794_045_601_000);
        assert_eq!(message.duration_ms, Some(4000));
        assert_eq!(message.tokens.input, 400);
        assert_eq!(message.tokens.cache_read, 800);
        assert_eq!(message.tokens.output, 45);
        assert_eq!(message.tokens.cache_write, 100);
        assert_eq!(message.cache_write_1h, 50);
        assert_eq!(message.dedup_key.as_deref(), Some("qoder:qoder:req:req_1"));
    }

    #[test]
    fn zeroed_and_missing_usage_produce_no_message() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join(".qoder-cn/projects/-work-app");
        let path = write_transcript(
            &project,
            "sess-cn.jsonl",
            &[
                r#"{"type":"runtime-config","sessionId":"sess-cn","model":"qmodel_preview","timestamp":1784988512000}"#,
                r#"{"type":"user","uuid":"u1","sessionId":"sess-cn","timestamp":"2026-07-25T14:08:32.000Z","message":{"role":"user","content":"hello"}}"#,
                r#"{"type":"assistant","uuid":"a1","sessionId":"sess-cn","timestamp":"2026-07-25T14:12:05.000Z","message":{"id":"msg_no_usage","role":"assistant","model":"qmodel_preview","content":[{"type":"text","text":"reply"}]}}"#,
                r#"{"type":"assistant","uuid":"a2","sessionId":"sess-cn","timestamp":"2026-07-25T14:12:06.000Z","message":{"id":"msg_credits","role":"assistant","model":"qmodel_preview","content":[{"type":"text","text":"reply"}],"usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"credits":1.25,"billable":true}}}"#,
            ],
        );

        assert!(parse_qoder_transcript_file(&path, QoderSource::China).is_empty());
        assert_eq!(QoderSource::from_path(&path), QoderSource::China);
    }

    #[test]
    fn resolves_auto_model_from_business_key_and_follows_switches() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join(".qoder-cn/projects/-work-app");
        let path = write_transcript(
            &project,
            "sess-auto.jsonl",
            &[
                r#"{"type":"runtime-config","sessionId":"sess-auto","model":"gfmodel","timestamp":1794041573406}"#,
                r#"{"type":"user","uuid":"u1","sessionId":"sess-auto","businessModelKey":"gfmodel","timestamp":"2026-11-07T08:56:16.000Z","message":{"role":"user","content":"run tests"}}"#,
                r#"{"type":"assistant","uuid":"a1","sessionId":"sess-auto","timestamp":"2026-11-07T08:56:20.000Z","message":{"id":"msg_auto_1","role":"assistant","model":"auto","content":[{"type":"text","text":"one"}],"usage":{"input_tokens":28000,"output_tokens":270,"cache_read_input_tokens":9000,"request_id":"req_auto_1"}}}"#,
                r#"{"type":"user","uuid":"u2","sessionId":"sess-auto","timestamp":"2026-11-07T08:56:25.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"call_1","content":"ok"}]}}"#,
                r#"{"type":"runtime-config","sessionId":"sess-auto","model":"dfmodel","timestamp":1794041600000}"#,
                r#"{"type":"assistant","uuid":"a2","sessionId":"sess-auto","timestamp":"2026-11-07T08:57:00.000Z","message":{"id":"msg_auto_2","role":"assistant","model":"auto","content":[{"type":"text","text":"two"}],"usage":{"input_tokens":32000,"output_tokens":150,"cache_read_input_tokens":12000,"request_id":"req_auto_2"}}}"#,
            ],
        );

        let messages = parse_qoder_transcript_file(&path, QoderSource::China);
        assert_eq!(messages.len(), 2);
        let first = messages[0].as_message();
        let second = messages[1].as_message();
        assert_eq!(first.model_id, "gfmodel");
        assert_eq!(first.provider_id, "qoder");
        assert_eq!(first.tokens.input, 19000);
        assert_eq!(first.tokens.cache_read, 9000);
        assert_eq!(first.tokens.output, 270);
        assert!(first.is_turn_start);
        assert!(!second.is_turn_start);
        assert_eq!(second.model_id, "dfmodel");
        assert_eq!(second.tokens.input, 20000);
        assert_eq!(second.tokens.cache_read, 12000);
    }

    #[test]
    fn runtime_provider_follows_only_its_model_and_yields_to_a_prefix() {
        let dir = TempDir::new().unwrap();
        let path = write_transcript(
            dir.path(),
            "sess-runtime.jsonl",
            &[
                r#"{"type":"runtime-config","model":"claude-sonnet-4-6","provider":"openrouter"}"#,
                r#"{"type":"assistant","message":{"id":"m1","model":"auto","usage":{"input_tokens":100}}}"#,
                r#"{"type":"assistant","message":{"id":"m2","model":"deepseek/chat/deepseek-v3","usage":{"input_tokens":100}}}"#,
                r#"{"type":"assistant","message":{"id":"m3","model":"auto","usage":{"input_tokens":100}}}"#,
                r#"{"type":"assistant","message":{"id":"m3","model":"gpt-4o","usage":{"output_tokens":2}}}"#,
                r#"{"type":"runtime-config","model":"deepseek/chat/deepseek-v3","providerId":"openrouter"}"#,
                r#"{"type":"assistant","message":{"id":"m4","model":"auto","usage":{"input_tokens":100}}}"#,
                r#"{"type":"runtime-config","model":"qwen3-coder-plus"}"#,
                r#"{"type":"assistant","message":{"id":"m5","model":"auto","usage":{"input_tokens":100}}}"#,
            ],
        );
        let messages = parse_qoder_transcript_file(&path, QoderSource::International);
        let models: Vec<_> = messages
            .iter()
            .map(QoderObservation::as_message)
            .map(|message| (message.model_id.as_str(), message.provider_id.as_str()))
            .collect();
        assert_eq!(
            models,
            [
                ("claude-sonnet-4-6", "openrouter"),
                ("deepseek-v3", "deepseek"),
                ("gpt-4o", "openai"),
                ("deepseek-v3", "deepseek"),
                ("qwen3-coder-plus", "qwen"),
            ],
        );
    }

    #[test]
    fn fork_replay_keeps_parent_session_and_attributes_new_requests_to_child() {
        let dir = TempDir::new().unwrap();
        let replayed = r#"{"type":"assistant","sessionId":"parent","timestamp":"2026-11-07T10:00:00Z","message":{"id":"m1","model":"qmodel","usage":{"input_tokens":100,"request_id":"r1"}}}"#;
        let parent = write_transcript(dir.path(), "parent.jsonl", &[replayed]);
        let child = write_transcript(
            dir.path(),
            "child.jsonl",
            &[
                replayed,
                r#"{"type":"user","sessionId":"child","timestamp":"2026-11-07T10:01:00Z","message":{"role":"user","content":"continue"}}"#,
                r#"{"type":"assistant","timestamp":"2026-11-07T10:01:01Z","message":{"id":"m2","model":"qmodel","usage":{"input_tokens":200,"request_id":"r2"}}}"#,
            ],
        );
        let mut messages = parse_qoder_transcript_file(&child, QoderSource::International);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].as_message().session_id, "parent");
        assert_eq!(messages[1].as_message().session_id, "child");
        messages.extend(parse_qoder_transcript_file(&parent, QoderSource::International));
        let collapsed = collapse_qoder_messages(messages);
        assert_eq!(collapsed.len(), 2);
        assert_eq!(collapsed[0].session_id, "parent");
        assert_eq!(collapsed[1].session_id, "child");
        assert_eq!(collapsed[0].tokens.total() + collapsed[1].tokens.total(), 300);
    }

    #[test]
    fn subagent_transcript_keeps_parent_and_agent() {
        let dir = TempDir::new().unwrap();
        let session_dir = dir
            .path()
            .join(".qoder-cn/projects/-repo/sess-main/subagents");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::write(
            session_dir.join("agent-abc.meta.json"),
            r#"{"agentType":"code-reviewer"}"#,
        )
        .unwrap();
        let path = write_transcript(
            &session_dir,
            "agent-abc.jsonl",
            &[
                r#"{"type":"assistant","isSidechain":true,"sessionId":"sess-main","agentId":"abc","timestamp":"2026-11-07T11:00:00.000Z","message":{"id":"msg_sub","model":"deepseek/chat/deepseek-v3","usage":{"input_tokens":500,"output_tokens":200,"request_id":"req_sub"}}}"#,
            ],
        );

        let messages = parse_qoder_transcript_file(&path, QoderSource::China);
        assert_eq!(messages.len(), 1);
        let message = messages[0].as_message();
        assert_eq!(message.agent.as_deref(), Some("Code Reviewer"));
        assert_eq!(message.session_id, "sess-main:agent:abc");
        assert_eq!(message.model_id, "deepseek-v3");
        assert_eq!(message.provider_id, "deepseek");
        assert_eq!(message.workspace_key.as_deref(), Some("-repo"));
    }

    #[test]
    fn skips_journal_and_checkpoint_files() {
        let dir = TempDir::new().unwrap();
        let subagents = dir.path().join(".qoder/projects/-repo/sess/subagents");
        let journal = write_transcript(
            &subagents,
            "journal.jsonl",
            &[
                r#"{"type":"assistant","message":{"id":"j","model":"qmodel","usage":{"input_tokens":9,"output_tokens":9}}}"#,
            ],
        );
        assert!(parse_qoder_transcript_file(&journal, QoderSource::International).is_empty());

        let checkpoints = write_transcript(
            &dir.path().join(".qoder/projects/-repo"),
            "sess.checkpoints.jsonl",
            &[
                r#"{"type":"assistant","message":{"id":"c","model":"qmodel","usage":{"input_tokens":9,"output_tokens":9}}}"#,
            ],
        );
        assert!(parse_qoder_transcript_file(&checkpoints, QoderSource::International).is_empty());
    }

    #[test]
    fn malformed_and_half_written_lines_do_not_drop_valid_records() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join(".qoder/projects/-repo");
        let path = write_transcript(
            &project,
            "sess-broken.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess-broken","timestamp":"2026-11-07T10:00:00.000Z","message":{"id":"m1","model":"qmodel","usage":{"input_tokens":100,"output_tokens":10,"request_id":"req_1"}}}"#,
                r#"{"type":"assistant","sessionId":"sess-broken","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"m2","model":"qm"#,
                r#"{"type":"assistant","sessionId":"sess-broken","timestamp":"2026-11-07T10:00:02.000Z","message":{"id":"m3","model":"qmodel","usage":{"input_tokens":200,"output_tokens":20,"request_id":"req_3"}}}"#,
            ],
        );

        let messages = parse_qoder_transcript_file(&path, QoderSource::International);
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0].as_message().dedup_key.as_deref(),
            Some("qoder:qoder:req:req_1")
        );
        assert_eq!(
            messages[1].as_message().dedup_key.as_deref(),
            Some("qoder:qoder:req:req_3")
        );
        assert_eq!(messages[1].as_message().tokens.input, 200);
    }

    #[test]
    fn jsonl_scan_marks_read_failures_incomplete() {
        use std::io::{self, Read};

        struct ReadError;
        impl Read for ReadError {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("synthetic JSONL read failure"))
            }
        }

        let dir = TempDir::new().unwrap();
        let source = QoderSource::International;
        for (filename, record) in [
            (
                "transcript.jsonl",
                r#"{"type":"assistant","message":{"model":"qmodel","usage":{"input_tokens":17}}}"#,
            ),
            (
                "segments/events.jsonl",
                r#"{"type":"model.response.completed","data":{"model":"qmodel","input_tokens":17}}"#,
            ),
        ] {
            let path = write_transcript(dir.path(), filename, &[record, "{bad json", record]);
            let (messages, complete) = scan_qoder_jsonl_file(&path, source);
            assert!(complete);
            assert_eq!(messages.len(), 2);

            let prefix = format!("{record}\n{record}");
            let mut reader = prefix.as_bytes().chain(BufReader::new(ReadError));
            let (messages, complete) = scan_qoder_jsonl(&path, source, &mut reader);
            assert!(!complete);
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].as_message().tokens.input, 17);

            let missing = path.with_file_name("missing.jsonl");
            let (messages, complete) = scan_qoder_jsonl_file(&missing, source);
            assert!(!complete);
            assert!(messages.is_empty());
        }
    }

    #[test]
    fn negative_and_huge_counters_are_safe() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join(".qoder/projects/-repo");
        let path = write_transcript(
            &project,
            "sess-max.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess-max","timestamp":"2026-11-07T10:00:00.000Z","message":{"id":"m1","model":"qmodel","usage":{"input_tokens":-5,"output_tokens":-7,"cache_read_input_tokens":-9}}}"#,
                r#"{"type":"assistant","sessionId":"sess-max","timestamp":"2026-11-07T10:00:01.000Z","message":{"id":"m2","model":"qmodel","usage":{"input_tokens":9223372036854775807,"output_tokens":9223372036854775807,"cache_read_input_tokens":9223372036854775807}}}"#,
                r#"{"type":"assistant","sessionId":"sess-max","timestamp":"99999999999999999999999","message":{"id":"m3","model":"qmodel","usage":{"input_tokens":10,"output_tokens":1}}}"#,
            ],
        );

        let messages = parse_qoder_transcript_file(&path, QoderSource::International);
        // The all-negative row is not measured usage.
        assert_eq!(messages.len(), 2);
        let first = messages[0].as_message();
        assert_eq!(first.tokens.input, 0);
        assert_eq!(first.tokens.cache_read, i64::MAX);
        assert!(first.tokens.total() > 0);
        assert_eq!(messages[1].as_message().tokens.input, 10);
    }

    #[test]
    fn segment_completed_events_are_counted_and_rollups_ignored() {
        let dir = TempDir::new().unwrap();
        let segments = dir
            .path()
            .join(".qoder/logs/sessions/-work-app/sess-seg/segments");
        std::fs::create_dir_all(&segments).unwrap();
        let path = segments.join("run-1.jsonl");
        std::fs::write(
            &path,
            concat!(
                r#"{"ts":"2026-09-12T17:08:58.000+08:00","type":"session.config.loaded","data":{"project_root":"/work/app"}}"#,
                "\n",
                r#"{"ts":"2026-09-12T17:09:02.500+08:00","type":"model.response.completed","request_id":"r1","data":{"model":"pro","input_tokens":1000,"output_tokens":45,"cache_read_input_tokens":400}}"#,
                "\n",
                r#"{"ts":"2026-09-12T17:09:10.000+08:00","type":"model.response.completed","request_id":"r2","data":{"model":"pro","input_tokens":1000,"output_tokens":30,"cache_read_input_tokens":0}}"#,
                "\n",
                r#"{"ts":"2026-09-12T17:09:11.000+08:00","type":"turn.finished","data":{"model":"pro","input_tokens":2000,"output_tokens":75,"cache_read_input_tokens":400}}"#,
                "\n",
            ),
        )
        .unwrap();

        let messages = parse_qoder_segment_file(&path, QoderSource::International);
        assert_eq!(messages.len(), 2);
        let first = messages[0].as_message();
        let second = messages[1].as_message();
        assert_eq!(first.session_id, "sess-seg");
        assert_eq!(first.workspace_key.as_deref(), Some("/work/app"));
        assert_eq!(first.model_id, "pro");
        assert_eq!(first.provider_id, "qoder");
        assert_eq!(first.tokens.input, 600);
        assert_eq!(first.tokens.cache_read, 400);
        assert_eq!(first.tokens.output, 45);
        assert_eq!(first.dedup_key.as_deref(), Some("qoder:qoder:req:r1"));
        assert_eq!(second.tokens.input, 1000);
        assert_eq!(second.tokens.output, 30);
    }

    #[test]
    fn segment_without_request_id_never_merges_distinct_events() {
        let dir = TempDir::new().unwrap();
        let segments = dir
            .path()
            .join(".qoder/logs/sessions/-work-app/sess-seg/segments");
        std::fs::create_dir_all(&segments).unwrap();
        let path = segments.join("run-2.jsonl");
        std::fs::write(
            &path,
            concat!(
                r#"{"ts":"2026-09-12T17:09:02.000+08:00","type":"model.response.completed","data":{"model":"pro","input_tokens":100,"output_tokens":10}}"#,
                "\n",
                r#"{"ts":"2026-09-12T17:09:03.000+08:00","type":"model.response.completed","data":{"model":"pro","input_tokens":200,"output_tokens":20}}"#,
                "\n",
            ),
        )
        .unwrap();

        let messages = parse_qoder_segment_file(&path, QoderSource::International);
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages
                .iter()
                .map(|row| row.as_message().tokens.input)
                .sum::<i64>(),
            300
        );
    }

    #[test]
    fn collapse_counts_a_request_once_across_transcript_and_segment() {
        let dir = TempDir::new().unwrap();
        let transcript = write_transcript(
            &dir.path().join(".qoder/projects/-work-app"),
            "sess-1.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess-1","timestamp":"2026-11-07T10:00:00.000Z","message":{"id":"msg_1","model":"qmodel","usage":{"input_tokens":1200,"output_tokens":40,"request_id":"req_1"}}}"#,
            ],
        );
        let segments = dir
            .path()
            .join(".qoder/logs/sessions/-work-app/sess-1/segments");
        std::fs::create_dir_all(&segments).unwrap();
        let segment = segments.join("run.jsonl");
        std::fs::write(
            &segment,
            concat!(
                r#"{"ts":"2026-11-07T10:00:02.000Z","type":"model.response.completed","request_id":"req_1","data":{"model":"qmodel","input_tokens":1200,"output_tokens":40}}"#,
                "\n",
                r#"{"ts":"2026-11-07T10:05:00.000Z","type":"model.response.completed","request_id":"req_2","data":{"model":"qmodel","input_tokens":300,"output_tokens":10}}"#,
                "\n",
            ),
        )
        .unwrap();

        let mut parsed = parse_qoder_transcript_file(&transcript, QoderSource::International);
        parsed.extend(parse_qoder_segment_file(&segment, QoderSource::International));
        let collapsed = collapse_qoder_messages(parsed);

        assert_eq!(collapsed.len(), 2);
        assert_eq!(collapsed[0].timestamp, 1_794_045_600_000);
        assert_eq!(collapsed[0].tokens.input, 1200);
        assert_eq!(collapsed[1].tokens.input, 300);
    }

    #[test]
    fn collapse_is_independent_of_input_order() {
        let build = |session: &str, timestamp: i64, tokens: i64| {
            let mut message = UnifiedMessage::new_with_dedup(
                CLIENT_ID,
                "qmodel",
                QODER_PROVIDER,
                session,
                timestamp,
                TokenBreakdown {
                    input: tokens,
                    output: 1,
                    ..Default::default()
                },
                0.0,
                Some("qoder:qoder:req:shared".to_string()),
            );
            message.is_turn_start = true;
            QoderObservation {
                message,
                measured_timestamp: true,
                model_rank: ModelRank::new(ModelKind::Slot, ModelEvidence::Reported),
                provider_evidence: ProviderEvidence::Unattributed,
            }
        };
        let mut first = build("s1", 1_000, 100);
        first
            .message
            .set_workspace(Some("/work/app".into()), Some("app".into()));
        let mut second = build("s2", 900, 120);
        second.message.set_workspace(Some("/work/app".into()), None);

        let forward = collapse_qoder_messages(vec![first.clone(), second.clone()]);
        let reverse = collapse_qoder_messages(vec![second, first]);
        assert_eq!(forward.len(), 1);
        assert_eq!(reverse.len(), 1);
        assert_eq!(forward[0].session_id, reverse[0].session_id);
        assert_eq!(forward[0].timestamp, reverse[0].timestamp);
        assert_eq!(forward[0].tokens.input, 120);
        assert_eq!(reverse[0].tokens.input, 120);
        assert_eq!(forward[0].workspace_label.as_deref(), Some("app"));
        assert_eq!(forward[0].workspace_label, reverse[0].workspace_label);
    }

    #[test]
    fn editions_do_not_deduplicate_each_other() {
        let dir = TempDir::new().unwrap();
        let intl = write_transcript(
            &dir.path().join(".qoder/projects/-app"),
            "sess.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess","timestamp":"2026-11-07T10:00:00.000Z","message":{"id":"msg_1","model":"qmodel","usage":{"input_tokens":100,"output_tokens":10,"request_id":"shared"}}}"#,
            ],
        );
        let cn = write_transcript(
            &dir.path().join(".qoder-cn/projects/-app"),
            "sess.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess","timestamp":"2026-11-07T11:00:00.000Z","message":{"id":"msg_1","model":"qmodel","usage":{"input_tokens":200,"output_tokens":20,"request_id":"shared"}}}"#,
            ],
        );

        let mut parsed = parse_qoder_transcript_file(&intl, QoderSource::International);
        parsed.extend(parse_qoder_transcript_file(&cn, QoderSource::China));
        let collapsed = collapse_qoder_messages(parsed);

        assert_eq!(collapsed.len(), 2);
        let total: i64 = collapsed.iter().map(|message| message.tokens.input).sum();
        assert_eq!(total, 300);
    }

    fn create_ide_db(path: &Path, with_chat_session: bool, rows: &[(i64, &str, &str, &str)]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
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
        if with_chat_session {
            conn.execute_batch(
                "CREATE TABLE chat_session (
                     session_id TEXT PRIMARY KEY,
                     project_uri TEXT,
                     project_name TEXT,
                     preferred_model_info TEXT
                 );
                 INSERT INTO chat_session VALUES
                     ('sess-ide', 'file:///work/ide-app', 'ide-app', '{\"model_key\":\"qmodel\"}');",
            )
            .unwrap();
        }
        for (created, token_info, model_info, request_id) in rows {
            conn.execute(
                "INSERT INTO chat_message (id, session_id, request_id, role, token_info, model_info, gmt_create)
                 VALUES (?1, 'sess-ide', ?2, 'assistant', ?3, ?4, ?5)",
                rusqlite::params![
                    format!("row-{created}"),
                    request_id,
                    token_info,
                    model_info,
                    created
                ],
            )
            .unwrap();
        }
        conn.execute_batch(
            "INSERT INTO chat_message (id, session_id, request_id, role, token_info, model_info, gmt_create)
             VALUES ('user-row', 'sess-ide', NULL, 'user', '{\"prompt_tokens\":999}', NULL, 1);",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn prices_one_enriched_request_without_mixing_workspace_metadata() {
        use crate::pricing::{ModelPricing, PricingService};

        let dir = TempDir::new().unwrap();
        let home = dir.path();
        let app_data =
            crate::PathRoot::AppData.resolve_with_env_strategy(home.to_str().unwrap(), false);
        create_ide_db(
            &Path::new(&app_data).join("Qoder/SharedClientCache/cache/db/local.db"),
            true,
            &[(
                1_794_045_600_000,
                r#"{"prompt_tokens":1000,"cached_tokens":800,"completion_tokens":15}"#,
                r#"{"model_key":"auto"}"#,
                "shared",
            )],
        );
        write_transcript(
            home,
            ".qoder/projects/-work-transcript-app/sess-cli.jsonl",
            &[
                r#"{"type":"assistant","sessionId":"sess-cli","cwd":"/work/transcript-app","timestamp":"2026-11-07T10:00:01Z","message":{"model":"gfmodel","usage":{"input_tokens":1200,"output_tokens":5,"request_id":"shared"}}}"#,
            ],
        );
        write_transcript(
            home,
            ".qoder/logs/sessions/-work-segment-app/sess-cli/segments/run.jsonl",
            &[
                r#"{"type":"session.config.loaded","data":{"project_root":"/work/segment-app"}}"#,
                r#"{"type":"model.response.completed","request_id":"shared","data":{"model":"auto","input_tokens":1100,"output_tokens":10}}"#,
                r#"{"type":"model.response.completed","ts":"2026-11-07T10:00:02Z","request_id":"shared","data":{"model":"claude-sonnet-4-6","provider":"anthropic","output_tokens":40,"cache_creation_input_tokens":100,"cache_creation":{"ephemeral_1h_input_tokens":50}}}"#,
            ],
        );
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
        let messages = crate::parse_local_unified_messages_with_pricing_uncached(
            crate::LocalParseOptions {
                home_dir: Some(home.to_str().unwrap().into()),
                use_env_roots: false,
                clients: Some(vec![CLIENT_ID.into()]),
                scanner_settings: crate::scanner::ScannerSettings {
                    bucket_timezone: Some("UTC".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Some(&pricing),
        )
        .await
        .unwrap();

        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(message.session_id, "sess-ide");
        assert_eq!(message.timestamp, 1_794_045_600_000);
        assert_eq!(message.workspace_key.as_deref(), Some("/work/ide-app"));
        assert_eq!(message.workspace_label, None);
        assert_eq!(message.model_id, "claude-sonnet-4-6");
        assert_eq!(message.provider_id, "anthropic");
        assert_eq!(message.tokens.input, 400);
        assert_eq!(message.tokens.cache_read, 800);
        assert_eq!(message.tokens.output, 40);
        assert_eq!(message.tokens.cache_write, 100);
        assert_eq!(message.cache_write_1h, 50);
        assert_eq!(message.cost_source, crate::CostSource::Estimated);
        assert!((message.cost - 1.0425).abs() < 1e-9);
    }

    #[test]
    fn ide_token_info_splits_cached_from_prompt() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("Qoder/SharedClientCache/cache/db/local.db");
        create_ide_db(
            &db,
            true,
            &[(
                1_762_509_600_000,
                r#"{"prompt_tokens":39005,"cached_tokens":37485,"completion_tokens":67,"max_input_tokens":200000}"#,
                r#"{"model_key":"auto"}"#,
                "req-ide-1",
            )],
        );

        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.complete);
        assert_eq!(scan.messages.len(), 1);
        let message = scan.messages[0].as_message();
        assert_eq!(message.client, "qoder");
        assert_eq!(message.tokens.input, 1520);
        assert_eq!(message.tokens.cache_read, 37485);
        assert_eq!(message.tokens.output, 67);
        assert_eq!(message.tokens.total(), 39072);
        assert_eq!(message.model_id, "auto");
        assert_eq!(message.provider_id, "qoder");
        assert_eq!(message.session_id, "sess-ide");
        assert_eq!(message.workspace_key.as_deref(), Some("/work/ide-app"));
        assert_eq!(
            message.dedup_key.as_deref(),
            Some("qoder:qoder:req:req-ide-1")
        );
    }

    #[test]
    fn ide_without_chat_session_still_reports_usage() {
        let dir = TempDir::new().unwrap();
        let db = dir
            .path()
            .join("QoderCN/SharedClientCache/cache/db/local.db");
        create_ide_db(
            &db,
            false,
            &[(
                1_762_509_600_000,
                r#"{"prompt_tokens":100,"cached_tokens":0,"completion_tokens":5}"#,
                r#"{"model_key":"qmodel_latest"}"#,
                "",
            )],
        );

        let scan = scan_qoder_ide_db(&db, QoderSource::China);
        assert!(scan.complete);
        assert_eq!(scan.messages.len(), 1);
        let message = scan.messages[0].as_message();
        assert_eq!(message.tokens.input, 100);
        assert_eq!(message.tokens.output, 5);
        assert_eq!(message.workspace_key, None);
        assert_eq!(
            message.dedup_key.as_deref(),
            Some("qoder:qoder-cn:ide:row-1762509600000")
        );
    }

    #[test]
    fn ide_preferred_model_fills_in_a_missing_message_model() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("Qoder/SharedClientCache/cache/db/local.db");
        create_ide_db(
            &db,
            true,
            &[(
                1_762_509_600_000,
                r#"{"prompt_tokens":10,"cached_tokens":0,"completion_tokens":1}"#,
                "{}",
                "req-nomodel",
            )],
        );

        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.complete);
        assert_eq!(scan.messages.len(), 1);
        assert_eq!(scan.messages[0].as_message().model_id, "qmodel");
        assert_eq!(scan.messages[0].as_message().provider_id, "qoder");
    }

    #[test]
    fn ide_only_confirms_empty_when_chat_message_is_absent() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("Qoder/SharedClientCache/cache/db/local.db");
        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.messages.is_empty());
        assert!(!scan.complete);

        // A database with no `chat_message` table is an empty *read*: no usage
        // exists, and the source should be cached as such.
        std::fs::create_dir_all(db.parent().unwrap()).unwrap();
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE other (id INTEGER);")
            .unwrap();
        drop(conn);
        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.messages.is_empty());
        assert!(scan.complete);

        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE Chat_Message (id TEXT);")
            .unwrap();
        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.messages.is_empty());
        assert!(!scan.complete, "required usage columns are missing");
    }

    #[test]
    fn ide_ignores_rows_without_measured_tokens() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("Qoder/SharedClientCache/cache/db/local.db");
        create_ide_db(
            &db,
            true,
            &[
                (
                    1_762_509_600_000,
                    r#"{"prompt_tokens":0,"cached_tokens":0,"completion_tokens":0}"#,
                    r#"{"model_key":"auto"}"#,
                    "req-zero",
                ),
                (
                    1_762_509_601_000,
                    "",
                    r#"{"model_key":"auto"}"#,
                    "req-empty",
                ),
            ],
        );

        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert!(scan.complete);
        assert!(scan.messages.is_empty());
    }

    #[test]
    fn ide_wal_writes_are_visible_to_a_second_reader() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("Qoder/SharedClientCache/cache/db/local.db");
        create_ide_db(
            &db,
            true,
            &[(
                1_762_509_600_000,
                r#"{"prompt_tokens":100,"cached_tokens":0,"completion_tokens":5}"#,
                r#"{"model_key":"auto"}"#,
                "req-1",
            )],
        );
        assert_eq!(
            scan_qoder_ide_db(&db, QoderSource::International)
                .messages
                .len(),
            1
        );

        // The IDE keeps its connection open, so the newest row lives in the
        // -wal sidecar and the main database's mtime does not move.
        let writer = rusqlite::Connection::open(&db).unwrap();
        writer
            .execute_batch("PRAGMA journal_mode = WAL; PRAGMA wal_autocheckpoint = 0;")
            .unwrap();
        writer
            .execute(
                "INSERT INTO chat_message (id, session_id, request_id, role, token_info, model_info, gmt_create)
                 VALUES ('row-2', 'sess-ide', 'req-2', 'assistant', ?1, '{\"model_key\":\"auto\"}', 1762509601000)",
                rusqlite::params![r#"{"prompt_tokens":200,"cached_tokens":50,"completion_tokens":7}"#],
            )
            .unwrap();

        let wal = PathBuf::from(format!("{}-wal", db.display()));
        assert!(wal.exists(), "the writer must leave a -wal sidecar");

        let scan = scan_qoder_ide_db(&db, QoderSource::International);
        assert_eq!(scan.messages.len(), 2);
        let message = scan.messages[1].as_message();
        assert_eq!(message.tokens.input, 150);
        assert_eq!(message.tokens.cache_read, 50);
        drop(writer);
    }

    #[test]
    fn ide_candidates_honour_edition_layouts() {
        let app_data = Path::new("/data");
        let candidates = qoder_ide_db_candidates(app_data, false);
        assert!(candidates.contains(&(
            app_data.join("Qoder/SharedClientCache/cache/db/local.db"),
            QoderSource::International,
        )));
        assert!(candidates.contains(&(
            app_data.join("QoderCN/SharedClientCache/cache/db/local.db"),
            QoderSource::China,
        )));
    }
}
