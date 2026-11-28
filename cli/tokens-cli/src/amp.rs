//! Amp (ampcode.com) thread sync.
//!
//! Current Amp keeps threads on ampcode.com and no longer writes the local
//! `~/.local/share/amp/threads` directory the Amp parser reads, so usage from
//! any recent thread, including threads that ran in the cloud, was missing
//! (missuo/tokens#64).
//!
//! `tokens amp sync` asks the signed-in `amp` CLI for the account's threads,
//! exports each new or changed one (`amp threads export`), and writes only
//! what the Amp parser reads to
//! `<tokens config>/amp-cache/threads/T-<id>.json`: the thread ID, creation
//! time, assistant message IDs and usage, and the usage ledger fields used
//! for accounting. Prompts, replies, tool output and titles never reach
//! disk. The `amp` CLI makes the requests with its own login; tokens never
//! reads Amp's credentials.
//!
//! Threads are found two ways, because neither is enough alone:
//! `amp threads list --json` pages through every thread but leaves out some
//! that ran on Amp's servers, and its `updated` time moves with the user's
//! messages, not the agent's, so a thread exported mid-turn would keep the
//! rest of that turn out of the cache. `amp threads search author:me` returns
//! only the most recently active threads, unlisted ones included, with an
//! `updatedAt` that follows every message. A thread found by search is
//! re-exported when that time changes; any other thread, when its listed time
//! does. Outside the search window an agent-only change can therefore be
//! missed if the list watermark is unchanged. Force re-exports discovered
//! threads but cannot discover an old thread absent from both results.
//!
//! An export holds the requests of the thread's main agent only. Amp bills a
//! thread's subagent (Task) and system-model requests (titles and the like) to
//! the same thread, but they are not in the export, so they are not counted.

use crate::{t, tr};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tokens_core::sessions::amp::{is_thread_id, AmpMessageUsage, AmpUsageLedger};

mod process;

const LIST_PAGE_SIZE: usize = 500;
/// The most `amp threads search` returns.
const SEARCH_LIMIT: usize = 100;
const LIST_TIMEOUT: Duration = Duration::from_secs(120);
const EXPORT_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_STDOUT_MIB: usize = 32;
const EXPORT_WORKERS: usize = 4;
const STATE_VERSION: u32 = 1;

pub fn get_amp_cache_dir() -> PathBuf {
    crate::paths::get_config_dir().join("amp-cache")
}

/// Must match `tokens_core::scanner::amp_sync_cache_root`.
fn threads_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join("threads")
}

fn state_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("state.json")
}

/// An absent state is a normal local-only install. An inaccessible or invalid
/// state must reach the validation path, never silently authorize replacement.
pub fn has_amp_sync_state() -> bool {
    !matches!(fs::symlink_metadata(state_path(&get_amp_cache_dir())),
        Err(error) if error.kind() == ErrorKind::NotFound)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AmpSyncState {
    version: u32,
    synced_at: DateTime<Utc>,
    /// Thread id -> the update time the cached copy was exported at.
    threads: BTreeMap<String, DateTime<Utc>>,
}

/// One entry of `amp threads list --json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListedThread {
    id: String,
    updated: DateTime<Utc>,
    message_count: Option<i64>,
}

/// One entry of `amp threads search --json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchedThread {
    id: String,
    updated_at: DateTime<Utc>,
}

/// The parts of `amp threads export` kept in the cache; serde skips the rest
/// of the payload without retaining it.
#[derive(Serialize, Deserialize)]
struct CachedThread {
    id: String,
    created: Option<i64>,
    messages: Vec<CachedMessage>,
    #[serde(
        rename = "usageLedger",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    usage_ledger: Option<AmpUsageLedger>,
}

#[derive(Serialize, Deserialize)]
struct CachedMessage {
    role: Option<String>,
    #[serde(rename = "messageId")]
    message_id: Option<i64>,
    usage: Option<AmpMessageUsage>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAmpResult {
    pub synced: bool,
    /// Threads with messages on the account.
    pub threads: usize,
    /// Threads exported this run because they were new or changed.
    pub exported: usize,
    /// Exports that failed; they keep any previous cache file and watermark.
    pub failed: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AmpStatus {
    amp_cli: Option<String>,
    cache_dir: String,
    has_cache: bool,
    cached_threads: usize,
    last_synced_at: Option<DateTime<Utc>>,
}

pub fn run_amp_sync(force: bool, json: bool) -> Result<()> {
    let result = sync_amp_cache_for_request(force, true);
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if result.synced {
        println!(
            "{}",
            tr!(
                "top.amp.synced",
                threads = result.threads,
                exported = result.exported
            )
            .green()
        );
        if result.failed > 0 {
            eprintln!(
                "{}",
                tr!(
                    "top.amp.exportsFailed",
                    failed = result.failed,
                    error = result.error.unwrap_or_default()
                )
                .yellow()
            );
        }
    } else if let Some(error) = result.error {
        eprintln!("{}", tr!("top.amp.syncFailed", error = error).yellow());
    }
    Ok(())
}

/// Hold the writer lock until the caller has scanned its authoritative input.
/// A successful discovery cannot vouch for an older, now-unlisted thread.
pub fn guard_replacement_cache(refreshed: bool) -> Result<Option<fs::File>> {
    let cache_dir = get_amp_cache_dir();
    if !refreshed && !cache_dir.try_exists()? {
        // A legacy-only submit must not create any cloud sync files. The
        // caller checks that no first sync started while it scanned.
        return Ok(None);
    }
    let lock = acquire_sync_lock(&cache_dir)?;
    let state = load_state(&cache_dir)?;
    let incomplete = || anyhow::anyhow!(t!("cli.submit.ampReplaceIncomplete"));
    if state.is_some() != refreshed {
        return Err(incomplete());
    }
    let mut expected = state.map_or_else(BTreeMap::new, |state| state.threads);
    match fs::read_dir(threads_dir(&cache_dir)) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json"))
                    .filter(|id| is_thread_id(id)) else { continue };
                if expected.remove(id).is_none()
                    || !tokens_core::sessions::amp::is_valid_cached_thread(&entry.path())
                {
                    return Err(incomplete());
                }
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if !expected.is_empty() {
        return Err(incomplete());
    }
    Ok(Some(lock))
}

pub fn run_amp_status(json: bool) -> Result<()> {
    let cache_dir = get_amp_cache_dir();
    let state = load_state(&cache_dir)?;
    let dir = threads_dir(&cache_dir);
    let mut cached_threads = 0;
    match fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file()
                    && entry.file_name().to_str()
                        .and_then(|name| name.strip_suffix(".json"))
                        .is_some_and(is_thread_id)
                {
                    cached_threads += 1;
                }
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let status = AmpStatus {
        amp_cli: find_amp_binary().map(|path| path.display().to_string()),
        cache_dir: dir.display().to_string(),
        has_cache: state.is_some(),
        cached_threads,
        last_synced_at: state.map(|state| state.synced_at),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!("\n  {}", t!("top.amp.statusTitle").cyan());
    println!(
        "  {} {}",
        t!("top.amp.cliLabel").bright_black(),
        match &status.amp_cli {
            Some(path) => path.green(),
            None => t!("top.amp.cliMissing").yellow(),
        }
    );
    println!(
        "  {} {}",
        t!("top.amp.cacheLabel").bright_black(),
        if status.has_cache {
            status.cache_dir.green()
        } else {
            status.cache_dir.yellow()
        }
    );
    if status.has_cache {
        println!(
            "  {} {}",
            t!("top.amp.threadsLabel").bright_black(),
            status.cached_threads
        );
        if let Some(synced_at) = &status.last_synced_at {
            println!(
                "  {} {}",
                t!("top.amp.lastSyncLabel").bright_black(),
                synced_at.to_rfc3339()
            );
        }
    } else {
        println!("{}", t!("top.amp.syncHint").bright_black());
    }
    println!();
    Ok(())
}

pub fn sync_amp_cache(force: bool) -> SyncAmpResult {
    sync_amp_cache_for_request(force, false)
}

fn sync_amp_cache_for_request(force: bool, allow_initial_sync: bool) -> SyncAmpResult {
    sync_amp_cache_with(
        &get_amp_cache_dir(),
        force,
        allow_initial_sync,
        find_amp_binary,
    )
    .unwrap_or_else(|error| SyncAmpResult {
        error: Some(error.to_string()),
        ..SyncAmpResult::default()
    })
}

fn sync_amp_cache_with(
    cache_dir: &Path,
    force: bool,
    allow_initial_sync: bool,
    amp_binary: impl FnOnce() -> Option<PathBuf>,
) -> Result<SyncAmpResult> {
    // Recheck opt-in under the same lock that guards discovery and publication.
    let _lock = acquire_sync_lock(cache_dir)?;
    let state = load_state(cache_dir)?;
    if !allow_initial_sync && state.is_none() {
        return Ok(SyncAmpResult::default());
    }
    let amp = amp_binary().ok_or_else(|| anyhow::anyhow!(t!("top.amp.cliNotFound")))?;
    let amp = Amp::new(amp)?;
    let threads = find_threads(&amp)?;

    let watermarks = state.map_or_else(BTreeMap::new, |state| state.threads);
    let dir = threads_dir(cache_dir);
    let thread_count = threads.len();
    let pending: Vec<_> = threads
        .into_iter()
        .filter(|(id, updated)| {
            force
                || watermarks.get(id) != Some(updated)
                || !fs::symlink_metadata(dir.join(format!("{id}.json")))
                    .is_ok_and(|metadata| metadata.is_file())
        })
        .collect();

    // A first sync exports every thread; a few at a time keeps it to minutes
    // without flooding ampcode.com.
    let worker_count = EXPORT_WORKERS.min(pending.len());
    let pending = Mutex::new(pending.into_iter());
    let progress = Mutex::new((
        watermarks,
        SyncAmpResult {
            threads: thread_count,
            ..SyncAmpResult::default()
        },
    ));
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| loop {
                if amp.processes.stopped() {
                    break;
                }
                let next = pending.lock().unwrap().next();
                let Some((id, updated)) = next else { break };
                let outcome = export_thread(&amp, &dir, &id);
                let (watermarks, result) = &mut *progress.lock().unwrap();
                match outcome {
                    Ok(()) => {
                        watermarks.insert(id, updated);
                        result.exported += 1;
                    }
                    Err(error) => {
                        result.failed += 1;
                        if result.error.is_none() {
                            result.error = Some(error.to_string());
                        }
                    }
                }
            });
        }
    });
    anyhow::ensure!(
        !amp.processes.stopped(),
        tr!("top.amp.runFailed", command = "amp threads export")
    );
    let (watermarks, mut result) = progress.into_inner().unwrap();
    if result.failed > 0 && result.exported == 0 {
        return Ok(result);
    }
    // Threads deleted on ampcode.com keep their cached usage and watermark.
    save_state(
        cache_dir,
        &AmpSyncState {
            version: STATE_VERSION,
            synced_at: Utc::now(),
            threads: watermarks,
        },
    )?;
    result.synced = true;
    Ok(result)
}

/// Every thread to keep in the cache, with the update time that marks a
/// change (see the module docs for why it comes from two commands).
fn find_threads(amp: &Amp) -> Result<BTreeMap<String, DateTime<Utc>>> {
    let mut threads = list_threads(amp)?;
    threads.extend(
        search_recent_threads(amp)?
            .into_iter()
            .map(|thread| (thread.id, thread.updated_at)),
    );
    Ok(threads)
}

fn search_recent_threads(amp: &Amp) -> Result<Vec<SearchedThread>> {
    let limit = SEARCH_LIMIT.to_string();
    let args = ["author:me", "--json", "--limit", &limit];
    let output = run_amp(amp, "search", &args, LIST_TIMEOUT)?;
    let threads: Vec<SearchedThread> = parse_output("search", &output)?;
    anyhow::ensure!(
        threads.iter().all(|thread| is_thread_id(&thread.id)),
        unexpected_output("search")
    );
    Ok(threads)
}

fn list_threads(amp: &Amp) -> Result<BTreeMap<String, DateTime<Utc>>> {
    let mut threads = BTreeMap::new();
    let mut offset = 0usize;
    let limit = LIST_PAGE_SIZE.to_string();
    loop {
        let offset_arg = offset.to_string();
        let args = [
            "--json",
            "--include-archived",
            "--limit",
            &limit,
            "--offset",
            &offset_arg,
        ];
        let output = run_amp(amp, "list", &args, LIST_TIMEOUT)?;
        let page: Vec<ListedThread> = parse_output("list", &output)?;
        anyhow::ensure!(
            page.len() <= LIST_PAGE_SIZE && page.iter().all(|thread| is_thread_id(&thread.id)),
            unexpected_output("list")
        );
        let last_page = page.len() < LIST_PAGE_SIZE;
        offset += page.len();
        let before = threads.len();
        for thread in page {
            let candidate = (thread.updated, thread.message_count);
            threads.entry(thread.id)
                .and_modify(|current: &mut (DateTime<Utc>, Option<i64>)| {
                    if candidate.0 > current.0
                        || (candidate.0 == current.0 && current.1 == Some(0) && candidate.1 != Some(0))
                    {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }
        anyhow::ensure!(last_page || threads.len() > before, unexpected_output("list"));
        if last_page {
            return Ok(threads.into_iter().filter_map(|(id, (updated, count))| {
                (count != Some(0)).then_some((id, updated))
            }).collect());
        }
    }
}

fn export_thread(amp: &Amp, dir: &Path, thread_id: &str) -> Result<()> {
    let contents =
        usage_only_thread(thread_id, &run_amp(amp, "export", &[thread_id], EXPORT_TIMEOUT)?)?;
    crate::commands::usage::helpers::atomic_write_secret(
        &dir.join(format!("{thread_id}.json")),
        &contents,
    )?;
    Ok(())
}

/// The export reduced to the fields the Amp parser reads, with only the
/// messages that carry usage.
fn usage_only_thread(thread_id: &str, export: &[u8]) -> Result<Vec<u8>> {
    let mut thread: CachedThread = parse_output("export", export)?;
    anyhow::ensure!(thread.id == thread_id, t!("top.amp.exportOfOtherThread"));
    thread.messages.retain(|message| {
        message.role.as_deref() == Some("assistant")
            && message
                .usage
                .as_ref()
                .is_some_and(|usage| usage.model.is_some())
    });
    for usage in thread
        .messages
        .iter()
        .filter_map(|message| message.usage.as_ref())
    {
        validate_usage(usage.model.as_deref(), usage.timestamp.as_deref())?;
    }
    if let Some(events) = thread
        .usage_ledger
        .as_mut()
        .and_then(|ledger| ledger.events.as_mut())
    {
        events.retain(|event| event.model.is_some());
        for event in events {
            validate_usage(event.model.as_deref(), event.timestamp.as_deref())?;
        }
    }
    Ok(serde_json::to_vec(&thread)?)
}

fn validate_usage(model: Option<&str>, timestamp: Option<&str>) -> Result<()> {
    // Accept new model IDs without accepting free-form text in the model field.
    let valid_model = model.is_some_and(|id| {
        id.len() <= 256
            && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            && !id.contains("://")
            && id.bytes().all(|b| {
                b.is_ascii_alphanumeric()
                    || matches!(b, b'-' | b'_' | b'.' | b'/' | b':' | b'@' | b'+')
            })
    });
    anyhow::ensure!(
        valid_model && timestamp.is_none_or(|ts| DateTime::parse_from_rfc3339(ts).is_ok()),
        unexpected_output("export")
    );
    Ok(())
}

fn parse_output<T: serde::de::DeserializeOwned>(operation: &str, output: &[u8]) -> Result<T> {
    // Serde errors can quote values from the payload. Never attach the source.
    serde_json::from_slice(output).map_err(|_| unexpected_output(operation))
}

fn unexpected_output(operation: &str) -> anyhow::Error {
    anyhow::anyhow!(tr!(
        "top.amp.unexpectedOutput",
        command = format_args!("amp threads {operation}")
    ))
}

/// The `amp` executable: on `PATH`, else where Amp's installer or Homebrew
/// puts it. `tokens serve` often runs under launchd or systemd with a minimal
/// `PATH` that leaves those directories out.
fn find_amp_binary() -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut install_dirs = crate::paths::home_dir()
        .map(|home| {
            vec![
                home.join(".amp").join("bin"),
                home.join(".local").join("bin"),
            ]
        })
        .unwrap_or_default();
    if cfg!(unix) {
        install_dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    }

    find_amp_in_dirs(std::env::split_paths(&path).chain(install_dirs))
}

fn find_amp_in_dirs(dirs: impl Iterator<Item = PathBuf>) -> Option<PathBuf> {
    let names: &[&str] = if cfg!(windows) {
        &["amp.exe", "amp.cmd"]
    } else {
        &["amp"]
    };
    dirs.filter(|dir| dir.is_absolute())
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|path| {
            let Ok(metadata) = fs::metadata(path) else {
                return false;
            };
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    return false;
                }
            }
            metadata.is_file()
        })
}

struct Amp {
    path: PathBuf,
    processes: process::Supervisor,
}

impl Amp {
    fn new(path: PathBuf) -> Result<Self> {
        let processes = process::Supervisor::new()
            .map_err(|_| anyhow::anyhow!(tr!("top.amp.runFailed", command = "amp threads")))?;
        Ok(Self { path, processes })
    }
}

/// Return complete stdout only after a successful exit; never capture stderr.
fn run_amp(amp: &Amp, operation: &str, args: &[&str], timeout: Duration) -> Result<Vec<u8>> {
    use process::CaptureError;

    let mut command = Command::new(&amp.path);
    command
        .args(["threads", operation])
        .args(args)
        .env_remove("TOKENS_API_TOKEN")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    amp.processes
        .bounded_output(&mut command, timeout, MAX_STDOUT_MIB * 1024 * 1024)
        .map_err(|error| match error {
            CaptureError::Run => tr!(
                "top.amp.runFailed",
                command = format_args!("amp threads {operation}")
            ),
            CaptureError::Stdout => tr!("top.amp.missingStreams"),
            CaptureError::TimedOut => tr!(
                "top.amp.commandTimedOut",
                command = format_args!("amp threads {operation}"),
                seconds = timeout.as_secs()
            ),
            CaptureError::OutputTooLarge => tr!(
                "top.amp.outputTooLarge",
                command = format_args!("amp threads {operation}"),
                mib = MAX_STDOUT_MIB
            ),
            CaptureError::Exit(status) => tr!(
                "top.amp.commandFailed",
                command = format_args!("amp threads {operation}"),
                status = status
            ),
        })
        .map_err(anyhow::Error::msg)
}

fn acquire_sync_lock(cache_dir: &Path) -> Result<fs::File> {
    fs::create_dir_all(cache_dir).context(t!("top.amp.lockFailed"))?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(cache_dir.join(".sync.lock"))
        .context(t!("top.amp.lockFailed"))?;
    match fs2::FileExt::try_lock_exclusive(&lock) {
        Ok(()) => Ok(lock),
        Err(error) if crate::commands::autosubmit::is_lock_contention(&error) => {
            anyhow::bail!(t!("top.amp.syncInProgress"))
        }
        Err(error) => Err(error).context(t!("top.amp.lockFailed")),
    }
}

fn load_state(cache_dir: &Path) -> Result<Option<AmpSyncState>> {
    let path = state_path(cache_dir);
    let invalid = || anyhow::anyhow!(t!("top.amp.invalidState"));
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        _ => return Err(invalid()),
    }
    let bytes = fs::read(path).map_err(|_| invalid())?;
    let state: AmpSyncState = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    anyhow::ensure!(
        state.version == STATE_VERSION && state.threads.keys().all(|id| is_thread_id(id)),
        invalid()
    );
    Ok(Some(state))
}

fn save_state(cache_dir: &Path, state: &AmpSyncState) -> Result<()> {
    let contents = serde_json::to_vec_pretty(state)?;
    crate::commands::usage::helpers::atomic_write_secret(&state_path(cache_dir), &contents)
        .context(t!("top.amp.writeStateFailed"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_thread_keeps_usage_and_ledger_without_conversation() {
        let export = serde_json::json!({
            "id": "T-1",
            "title": "secret project",
            "created": 1_775_304_000_000_i64,
            "messages": [
                {
                    "role": "user",
                    "messageId": 1,
                    "content": [{ "type": "text", "text": "my prompt" }],
                    "usage": { "model": "claude-opus-5-5", "inputTokens": 100 }
                },
                {
                    "role": "assistant",
                    "messageId": 2,
                    "content": [{ "type": "text", "text": "my reply" }],
                    "usage": {
                        "model": "zai-org/GLM-5.3-Flash",
                        "timestamp": "2026-11-19T05:20:21.230Z",
                        "inputTokens": 2,
                        "outputTokens": 1630,
                        "cacheReadInputTokens": 0,
                        "cacheCreationInputTokens": 71025,
                        "credits": 0.17,
                        "extra": { "prompt": "private nested usage" }
                    }
                },
                {
                    "role": "private arbitrary role",
                    "usage": { "model": "claude-opus-5-5", "inputTokens": 100 }
                }
            ],
            "usageLedger": {
                "extra": "private ledger root",
                "events": [
                    {
                        "model": "zai-org/GLM-5.3-Flash",
                        "toMessageId": 2,
                        "operationType": "private operation",
                        "fromMessageId": 1,
                        "extra": "private ledger event",
                        "tokens": {
                            "input": 2,
                            "output": 1630,
                            "cacheReadInputTokens": 0,
                            "cacheCreationInputTokens": 71025,
                            "extra": "private ledger tokens"
                        }
                    },
                    {
                        "timestamp": "2026-11-21T05:20:21.230Z",
                        "model": "claude-sonnet-4-0",
                        "credits": 0.22,
                        "tokens": { "input": 9, "output": 3 }
                    }
                ]
            }
        });

        let cached = usage_only_thread("T-1", export.to_string().as_bytes()).unwrap();
        let text = std::str::from_utf8(&cached).unwrap();
        for field in [
            "secret project",
            "my prompt",
            "my reply",
            "private ",
            "content",
            "extra",
            "operationType",
            "fromMessageId",
            "\"user\"",
        ] {
            assert!(!text.contains(field), "{field:?} leaked into {text}");
        }
        let json: serde_json::Value = serde_json::from_slice(&cached).unwrap();
        assert_eq!(json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(json["messages"][0]["role"], "assistant");
        assert_eq!(json["messages"][0]["messageId"], 2);
        assert_eq!(json["usageLedger"]["events"][0]["toMessageId"], 2);

        // The Amp parser reads the reduced file exactly as it read the export.
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("T-1.json");
        fs::write(&path, &cached).unwrap();
        let messages = tokens_core::sessions::amp::parse_amp_file(&path);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].session_id, "T-1");
        assert_eq!(messages[0].model_id, "GLM-5.3-Flash");
        assert_eq!(messages[0].tokens.output, 1630);
        assert_eq!(messages[0].tokens.cache_write, 71025);
        assert_eq!(messages[0].cost, 0.17);
        assert_eq!(
            messages[0].timestamp,
            chrono::DateTime::parse_from_rfc3339("2026-11-19T05:20:21.230Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(messages[1].tokens.input, 9);
        assert_eq!(messages[1].cost, 0.22);
        assert_ne!(messages[0].date, messages[1].date);
    }

    #[test]
    fn private_export_values_never_reach_errors() {
        for export in [
            r#"{"id":"T-private-value","messages":[]}"#,
            r#"{"id":"T-1","messages":[{"role":"assistant","usage":{"outputTokens":"private-value"}}]}"#,
            r#"{"id":"T-1","messages":[{"role":"assistant","usage":{"model":"private-value exfil"}}]}"#,
            r#"{"id":"T-1","messages":[{"role":"assistant","usage":{"model":"claude-sonnet-4-0","timestamp":"private-value"}}]}"#,
        ] {
            let error = usage_only_thread("T-1", export.as_bytes()).unwrap_err();
            assert!(!format!("{error:#}").contains("private-value"));
        }
    }

    #[test]
    fn invalid_state_never_grants_opt_in_or_is_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = state_path(dir.path());
        let must_not_run =
            || -> Option<PathBuf> { panic!("state must be checked before Amp is resolved") };
        let result = sync_amp_cache_with(dir.path(), false, false, must_not_run).unwrap();
        assert!(!result.synced);
        assert!(!path.exists());
        for state in [
            r#"{"version":2,"syncedAt":"2026-11-19T05:20:21Z","threads":{}}"#,
            r#"{"version":1,"syncedAt":null,"threads":{}}"#,
            r#"{"version":1,"syncedAt":"2026-11-19T05:20:21Z","threads":{"T-1":"private-value"}}"#,
            r#"{"version":1,"syncedAt":"2026-11-19T05:20:21Z","threads":{"../private-value":"2026-11-19T05:20:21Z"}}"#,
        ] {
            fs::write(&path, state).unwrap();
            for explicit in [false, true] {
                let error =
                    sync_amp_cache_with(dir.path(), true, explicit, must_not_run).unwrap_err();
                assert!(!format!("{error:#}").contains("private-value"));
                assert_eq!(fs::read_to_string(&path).unwrap(), state);
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn concurrent_sync_cannot_reach_amp() {
        let dir = tempfile::tempdir().unwrap();
        let _lock = acquire_sync_lock(dir.path()).unwrap();
        let error = sync_amp_cache_with(dir.path(), true, true, || {
            panic!("another writer holds the sync lock")
        })
        .unwrap_err();
        assert_eq!(error.to_string(), t!("top.amp.syncInProgress"));
        assert!(!state_path(dir.path()).exists());
    }

    #[test]
    fn non_absolute_search_dirs_cannot_supply_amp() {
        let dir = tempfile::tempdir_in(".").unwrap();
        let name = if cfg!(windows) { "amp.exe" } else { "amp" };
        let path = dir.path().join(name);
        fs::write(&path, b"synthetic; never executed").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let relative = PathBuf::from(dir.path().file_name().unwrap());
        let absolute = fs::canonicalize(dir.path()).unwrap();
        assert!(find_amp_in_dirs([PathBuf::new(), relative].into_iter()).is_none());
        assert_eq!(
            find_amp_in_dirs([absolute.clone()].into_iter()),
            Some(absolute.join(name))
        );
    }

    #[cfg(unix)]
    #[test]
    fn failed_children_do_not_return_output_and_inherited_pipes_time_out() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let amp = Amp::new(dir.path().join("synthetic-amp")).unwrap();
        for (completion, timeout) in [
            ("exit 7", Duration::from_secs(1)),
            ("/bin/sleep 1 &\nexit 0", Duration::from_millis(150)),
        ] {
            fs::write(
                &amp.path,
                format!(
                    "#!/bin/sh\nprintf 'private stderr' >&2\nprintf '%s' '{{\"id\":\"T-1\",\"messages\":[]}}'\n{completion}\n"
                ),
            )
            .unwrap();
            fs::set_permissions(&amp.path, fs::Permissions::from_mode(0o700)).unwrap();
            let error = run_amp(&amp, "export", &["T-1"], timeout).unwrap_err();
            assert!(!format!("{error:#}").contains("private stderr"));
        }
    }

    #[test]
    fn thread_ids_that_are_not_file_names_are_refused() {
        assert!(is_thread_id("T-01a206d8-0e69-75aa-bd05-ec4b6664d307"));
        for id in ["", "T-", "01a206d8", "T-../../x", "T-a/b", "T-a.json"] {
            assert!(!is_thread_id(id), "{id:?}");
        }
        assert!(!is_thread_id(&format!("T-{}", "a".repeat(127))));
    }
}
