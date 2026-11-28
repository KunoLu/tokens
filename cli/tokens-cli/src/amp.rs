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
//! `updatedAt` that follows every message. An export uses the newer available
//! watermark for each thread. Outside the search window an agent-only change
//! can be missed if the list watermark is unchanged. Force re-exports
//! discovered threads but cannot discover an old thread absent from both
//! results. After successfully exporting every discovered thread, a manual
//! force sync can clear retry entries for undiscovered threads. Their cached
//! usage stays on disk, and a later discovery triggers another export.
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
use tokens_core::sessions::amp::{is_thread_id, project_cached_thread, AmpCacheProjectionError};

mod process;

const LIST_PAGE_SIZE: usize = 500;
/// The most `amp threads search` returns.
const SEARCH_LIMIT: usize = 100;
const LIST_TIMEOUT: Duration = Duration::from_secs(120);
const EXPORT_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_STDOUT_MIB: usize = 32;
const EXPORT_WORKERS: usize = 4;
const STATE_VERSION: u32 = 3;

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

fn initial_state_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("initial-state.json")
}

/// Only a complete first sync publishes state.json. An inaccessible or invalid
/// state must reach the validation path, never silently authorize replacement.
pub fn has_amp_sync_state() -> bool {
    state_file_exists(&state_path(&get_amp_cache_dir()))
}

fn state_file_exists(path: &Path) -> bool {
    !matches!(fs::symlink_metadata(path), Err(error) if error.kind() == ErrorKind::NotFound)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AmpSyncState {
    version: u32,
    synced_at: Option<DateTime<Utc>>,
    /// Last successful export watermark. A null entry only requests a fresh
    /// export if the ID reappears; it is not active work or proof of coverage.
    threads: BTreeMap<String, Option<DateTime<Utc>>>,
    /// Thread id -> latest observed or requested export still outstanding.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pending: BTreeMap<String, DateTime<Utc>>,
}

impl AmpSyncState {
    fn eligible(&self, id: &str, count: Option<i64>) -> bool {
        count != Some(0) || self.pending.contains_key(id) || matches!(self.threads.get(id), Some(None))
    }

    fn needs_export(&self, id: &str, updated: DateTime<Utc>) -> bool {
        self.threads.get(id).copied().flatten().is_none_or(|known| updated > known)
    }

    fn record_pending(&mut self, id: &str, updated: DateTime<Utc>) -> bool {
        if let Some(pending) = self.pending.get_mut(id) {
            if updated <= *pending {
                return false;
            }
            *pending = updated;
        } else {
            self.pending.insert(id.to_owned(), updated);
        }
        true
    }
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

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAmpResult {
    /// Discovery finished and either exports made progress or no updates remain.
    pub synced: bool,
    /// Distinct eligible threads discovered this run, including before a discovery error.
    pub threads: usize,
    /// Thread cache files successfully written this run.
    pub exported: usize,
    /// Exports that failed; they keep any previous cache file and watermark.
    pub failed: usize,
    /// Persisted updates still awaiting export, or unknown if state could not be read.
    pub pending: Option<usize>,
    /// Invisible retry entries dropped by a successful manual force sync.
    pub dropped_pending: usize,
    /// Whether a complete first sync has published valid state.
    pub auto_refresh_enabled: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AmpStatus {
    amp_cli: Option<String>,
    cache_dir: String,
    has_cache: bool,
    cached_threads: usize,
    pending: usize,
    auto_refresh_enabled: bool,
    last_synced_at: Option<DateTime<Utc>>,
}

pub fn run_amp_sync(force: bool, json: bool) -> Result<()> {
    let result = sync_amp_cache_for_request(force, true);
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if result.synced || result.threads != 0 || result.exported != 0 {
        let summary = tr!(
            "top.amp.synced",
            threads = result.threads,
            exported = result.exported
        );
        println!(
            "{}",
            if result.synced && result.pending == Some(0) {
                summary.green()
            } else {
                summary.yellow()
            }
        );
    }
    if result.dropped_pending != 0 {
        println!(
            "{}",
            tr!("top.amp.droppedPending", count = result.dropped_pending).yellow()
        );
    }
    if let Some(error) = result.error.as_deref() {
        let warning = if result.synced && result.failed != 0 {
            tr!("top.amp.exportsFailed", failed = result.failed, error = error)
        } else {
            tr!("top.amp.syncFailed", error = error)
        };
        eprintln!("{}", warning.yellow());
    }
    if let (Some(enabled), Some(pending)) = (result.auto_refresh_enabled, result.pending) {
        print_amp_refresh_status(enabled, pending);
    }
    Ok(())
}

fn print_amp_refresh_status(enabled: bool, pending: usize) {
    println!(
        "{}",
        if enabled {
            t!("top.amp.autoRefreshEnabled").green()
        } else {
            t!("top.amp.autoRefreshDisabled").yellow()
        }
    );
    if pending != 0 {
        eprintln!("{}", tr!("top.amp.pendingExports", pending = pending).yellow());
    }
}

/// A legacy-only replacement keeps the first cloud sync out until upload finishes.
pub fn guard_replacement_cache() -> Result<fs::File> {
    let dir = get_amp_cache_dir();
    let lock = policy_lock(&dir).map_err(|_| anyhow::anyhow!(t!("cli.submit.ampReplaceIncomplete")))?;
    anyhow::ensure!(
        lock.metadata()?.len() == 0
            && fs::symlink_metadata(&dir).is_err_and(|err| err.kind() == ErrorKind::NotFound),
        t!("cli.submit.ampReplaceIncomplete")
    );
    Ok(lock)
}

pub fn run_amp_status(json: bool) -> Result<()> {
    let status = amp_status_at(&get_amp_cache_dir(), find_amp_binary())?;
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
    print_amp_refresh_status(status.auto_refresh_enabled, status.pending);
    println!();
    Ok(())
}

fn amp_status_at(cache_dir: &Path, amp_cli: Option<PathBuf>) -> Result<AmpStatus> {
    let (state, auto_refresh_enabled) = load_sync_state(cache_dir)?;
    let dir = threads_dir(cache_dir);
    let mut cached_threads = 0;
    match fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file()
                    && entry
                        .file_name()
                        .to_str()
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
    Ok(AmpStatus {
        amp_cli: amp_cli.map(|path| path.display().to_string()),
        cache_dir: dir.display().to_string(),
        has_cache: cached_threads != 0,
        cached_threads,
        pending: state.as_ref().map_or(0, |state| state.pending.len()),
        auto_refresh_enabled,
        last_synced_at: if auto_refresh_enabled {
            state.and_then(|state| state.synced_at)
        } else {
            None
        },
    })
}

pub fn sync_amp_cache(force: bool) -> SyncAmpResult {
    sync_amp_cache_for_request(force, false)
}

fn sync_amp_cache_for_request(force: bool, manual: bool) -> SyncAmpResult {
    sync_amp_cache_with(&get_amp_cache_dir(), force, manual, || {
        Amp::new(find_amp_binary().ok_or_else(|| anyhow::anyhow!(t!("top.amp.cliNotFound")))?)
    })
    .unwrap_or_else(|error| SyncAmpResult {
        error: Some(error.to_string()),
        ..SyncAmpResult::default()
    })
}

fn sync_amp_cache_with(
    cache_dir: &Path,
    force: bool,
    manual: bool,
    prepare_amp: impl FnOnce() -> Result<Amp>,
) -> Result<SyncAmpResult> {
    let lock = policy_lock(cache_dir)?;
    let dir = threads_dir(cache_dir);
    for path in [cache_dir, dir.as_path()] {
        match fs::symlink_metadata(path) {
            Ok(meta) => anyhow::ensure!(meta.is_dir(), t!("top.amp.invalidState")),
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
    }
    let (state, initialized) = load_sync_state(cache_dir)?;
    let mut state = state.unwrap_or_else(|| AmpSyncState {
        version: STATE_VERSION,
        synced_at: None,
        threads: BTreeMap::new(),
        pending: BTreeMap::new(),
    });
    let result = SyncAmpResult {
        pending: Some(state.pending.len()),
        auto_refresh_enabled: Some(initialized),
        ..SyncAmpResult::default()
    };
    if !manual && !initialized {
        return Ok(result);
    }

    let working_path = if initialized {
        state_path(cache_dir)
    } else {
        initial_state_path(cache_dir)
    };
    state.version = STATE_VERSION;
    let mut progress = SyncProgress {
        state,
        path: &working_path,
        result,
    };
    let sync = || {
        let amp = prepare_amp()?;
        // Unavailable Amp must not mark a legacy-only configuration as cloud-backed.
        if lock.metadata()?.len() == 0 {
            lock.set_len(1)?;
            lock.sync_data()?;
        }
        fs::create_dir_all(&dir)?;
        sync_discovered_threads(&amp, cache_dir, &dir, force, manual, initialized, &mut progress)
    };
    if let Err(error) = sync() {
        progress.result.error = Some(error.to_string());
    }
    Ok(progress.result)
}

struct SyncProgress<'a> {
    state: AmpSyncState,
    path: &'a Path,
    result: SyncAmpResult,
}

impl SyncProgress<'_> {
    fn checkpoint(&mut self) -> Result<()> {
        save_state(self.path, &self.state)?;
        self.result.pending = Some(self.state.pending.len());
        Ok(())
    }

    fn observe<'a>(
        &mut self,
        observations: impl Iterator<Item = (&'a str, DateTime<Utc>, Option<i64>)>,
    ) -> Result<()> {
        let mut recorded = false;
        for (id, updated, count) in observations {
            if self.state.eligible(id, count) && self.state.needs_export(id, updated) {
                recorded |= self.state.record_pending(id, updated);
            }
        }
        if recorded {
            self.checkpoint()?;
        }
        Ok(())
    }
}

fn sync_discovered_threads(
    amp: &Amp,
    cache_dir: &Path,
    dir: &Path,
    force: bool,
    manual: bool,
    initialized: bool,
    progress: &mut SyncProgress<'_>,
) -> Result<()> {
    let threads = find_threads(amp, progress)?;
    let state = &progress.state;
    let exports: Vec<_> = threads
        .into_iter()
        .filter_map(|(id, (updated, _))| {
            (force
                || state.pending.contains_key(&id)
                || state.needs_export(&id, updated)
                || !fs::symlink_metadata(dir.join(format!("{id}.json")))
                    .is_ok_and(|metadata| metadata.is_file()))
            .then_some((id, updated))
        })
        .collect();

    // Publish intent before a worker can touch a cache file. Cancellation or
    // a crash must leave evidence even if an unfinished thread falls out of discovery.
    let mut recorded = false;
    for (id, updated) in &exports {
        recorded |= progress.state.record_pending(id, *updated);
    }
    if recorded {
        progress.checkpoint()?;
    }

    // A first sync exports every thread; a few at a time keeps it to minutes
    // without flooding ampcode.com.
    let worker_count = EXPORT_WORKERS.min(exports.len());
    let exports = Mutex::new(exports.into_iter());
    let progress = Mutex::new(progress);
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| loop {
                if amp.processes.stopped() {
                    break;
                }
                let next = exports.lock().unwrap().next();
                let Some((id, updated)) = next else { break };
                let outcome = export_thread(amp, dir, &id);
                let SyncProgress { state, result, .. } = &mut **progress.lock().unwrap();
                match outcome {
                    Ok(()) => {
                        // Export fetches the current thread by ID, even when discovery
                        // is stale. Confirm the newest observed watermark.
                        let updated = state
                            .pending
                            .remove(&id)
                            .map_or(updated, |pending| pending.max(updated));
                        let known = state.threads.entry(id).or_default();
                        *known = Some(known.map_or(updated, |previous| previous.max(updated)));
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
    let progress = progress.into_inner().unwrap();
    let stopped = amp.processes.stopped();
    let dropped_pending = if force && manual && !stopped && progress.result.failed == 0 {
        // Every discovered thread exported. Only unseen retries remain.
        let pending = std::mem::take(&mut progress.state.pending);
        let count = pending.len();
        for id in pending.into_keys() {
            // Retain only the rediscovery obligation, never poll this invisible ID.
            progress.state.threads.insert(id, None);
        }
        count
    } else {
        0
    };
    let complete = !stopped && progress.result.failed == 0 && progress.state.pending.is_empty();
    let partial = !stopped && progress.result.exported != 0;
    let record_sync = complete || (initialized && partial);
    if record_sync {
        progress.state.synced_at = Some(Utc::now());
    }
    if record_sync || progress.result.exported != 0 {
        // Keep confirmed exports even after cancellation. Omitted discoveries
        // and unfinished updates keep their previous entries.
        progress.checkpoint()?;
    }
    anyhow::ensure!(
        !amp.processes.stopped(),
        tr!("top.amp.runFailed", command = "amp threads export")
    );
    if complete && !initialized {
        // Publish the opt-in key only after all known first-sync work succeeds.
        fs::rename(progress.path, state_path(cache_dir)).context(t!("top.amp.writeStateFailed"))?;
        progress.result.auto_refresh_enabled = Some(true);
    }
    progress.result.dropped_pending = dropped_pending;
    progress.result.synced = complete || partial;
    Ok(())
}

type DiscoveredThreads = BTreeMap<String, (DateTime<Utc>, Option<i64>)>;

/// Discover the export candidates, retaining each valid page even if a later command fails.
fn find_threads(amp: &Amp, progress: &mut SyncProgress<'_>) -> Result<DiscoveredThreads> {
    let mut threads = BTreeMap::new();
    let listed = list_threads(amp, &mut threads, progress);
    // A stale list count must not hide a previously discovered unfinished thread.
    threads.retain(|id, (_, count)| progress.state.eligible(id, *count));
    progress.result.threads = threads.len();
    listed?;

    let recent = search_recent_threads(amp)?;
    let checkpoint = progress.observe(
        recent
            .iter()
            .map(|thread| (thread.id.as_str(), thread.updated_at, None)),
    );
    for thread in recent {
        threads
            .entry(thread.id)
            .and_modify(|(updated, _)| *updated = (*updated).max(thread.updated_at))
            .or_insert((thread.updated_at, None));
    }
    progress.result.threads = threads.len();
    checkpoint?;
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

fn list_threads(
    amp: &Amp,
    threads: &mut DiscoveredThreads,
    progress: &mut SyncProgress<'_>,
) -> Result<()> {
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
        // Commit valid observations before asking Amp for another page or search.
        let checkpoint = progress.observe(
            page.iter()
                .map(|thread| (thread.id.as_str(), thread.updated, thread.message_count)),
        );
        let last_page = page.len() < LIST_PAGE_SIZE;
        offset += page.len();
        let before = threads.len();
        for thread in page {
            let candidate = (thread.updated, thread.message_count);
            threads
                .entry(thread.id)
                .and_modify(|current| {
                    if candidate.0 > current.0
                        || (candidate.0 == current.0 && current.1 == Some(0) && candidate.1 != Some(0))
                    {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }
        checkpoint?;
        anyhow::ensure!(last_page || threads.len() > before, unexpected_output("list"));
        if last_page {
            return Ok(());
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

/// Retain usage and the message IDs and timestamps needed to date it.
fn usage_only_thread(thread_id: &str, export: &[u8]) -> Result<Vec<u8>> {
    project_cached_thread(thread_id, export).map_err(|error| match error {
        AmpCacheProjectionError::OtherThread => anyhow::anyhow!(t!("top.amp.exportOfOtherThread")),
        AmpCacheProjectionError::UnexpectedOutput => unexpected_output("export"),
    })
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

fn policy_lock(cache_dir: &Path) -> Result<fs::File> {
    let config = cache_dir.parent().context(t!("top.amp.lockFailed"))?;
    fs::create_dir_all(config).context(t!("top.amp.lockFailed"))?;
    let path = config.join(".amp-cloud.lock");
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let lock = options.open(path).context(t!("top.amp.lockFailed"))?;
    anyhow::ensure!(lock.metadata()?.is_file(), t!("top.amp.lockFailed"));
    match fs2::FileExt::try_lock_exclusive(&lock) {
        Ok(()) => Ok(lock),
        Err(error) if crate::commands::autosubmit::is_lock_contention(&error) => {
            anyhow::bail!(t!("top.amp.syncInProgress"))
        }
        Err(error) => Err(error).context(t!("top.amp.lockFailed")),
    }
}

fn load_state(cache_dir: &Path) -> Result<Option<AmpSyncState>> {
    read_state(&state_path(cache_dir), true)
}

fn load_sync_state(cache_dir: &Path) -> Result<(Option<AmpSyncState>, bool)> {
    let initial_path = initial_state_path(cache_dir);
    if let Some(state) = load_state(cache_dir)? {
        // These files cannot coexist after normal publication.
        anyhow::ensure!(
            !state_file_exists(&initial_path),
            t!("top.amp.invalidState")
        );
        return Ok((Some(state), true));
    }
    let draft = read_state(&initial_path, false);
    // Status doesn't lock; an initial state may have moved between reads.
    if let Some(state) = load_state(cache_dir)? {
        return Ok((Some(state), true));
    }
    Ok((draft?, false))
}

fn read_state(path: &Path, committed: bool) -> Result<Option<AmpSyncState>> {
    let invalid = || anyhow::anyhow!(t!("top.amp.invalidState"));
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        _ => return Err(invalid()),
    }
    let bytes = fs::read(path).map_err(|_| invalid())?;
    let state: AmpSyncState = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let supported_legacy = (state.version == 2 || (committed && state.version == 1 && state.pending.is_empty()))
        && state.threads.values().all(Option::is_some);
    anyhow::ensure!(
        (state.version == STATE_VERSION || supported_legacy)
            && (!committed || state.synced_at.is_some())
            && state
                .threads
                .keys()
                .chain(state.pending.keys())
                .all(|id| is_thread_id(id)),
        invalid()
    );
    Ok(Some(state))
}

fn save_state(path: &Path, state: &AmpSyncState) -> Result<()> {
    let contents = serde_json::to_vec_pretty(state)?;
    crate::commands::usage::helpers::atomic_write_secret(path, &contents)
        .context(t!("top.amp.writeStateFailed"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use serde_json::{json, Value};

    #[cfg(unix)]
    const FIRST: &str = "2025-01-02T00:00:00Z";
    #[cfg(unix)]
    const UPDATED: &str = "2025-01-03T00:00:00Z";

    #[cfg(unix)]
    struct SyntheticAmp {
        _config: tempfile::TempDir,
        cache: PathBuf,
    }

    #[cfg(unix)]
    impl SyntheticAmp {
        fn new() -> Self {
            use std::os::unix::fs::PermissionsExt;

            let config = tempfile::tempdir().unwrap();
            let cache = config.path().join("amp-cache");
            fs::create_dir(&cache).unwrap();
            let fixture = Self { _config: config, cache };
            let amp = fixture.dir().join("amp");
            fs::write(
                &amp,
                r#"#!/bin/sh
root=${0%/*}
case "$1 $2" in
    'threads list')
        route=list
        while [ "$#" -gt 1 ]; do
            if [ "$1" = '--offset' ] && [ "$2" != 0 ]; then
                route="list-$2"
                break
            fi
            shift
        done
        ;;
    'threads search') route=search ;;
    'threads export') route=$3 ;;
    *) exit 2 ;;
esac
state="$root/state.json"
[ -f "$state" ] || state="$root/initial-state.json"
if [ -f "$state" ]; then
    while IFS= read -r line || [ -n "$line" ]; do
        printf '%s\n' "$line"
    done <"$state" >"$root/$route.before" || exit 2
fi
IFS= read -r response <"$root/$route.response" || exit 2
printf '%s\n' "$response"
"#,
            )
            .unwrap();
            fs::set_permissions(amp, fs::Permissions::from_mode(0o700)).unwrap();
            fixture.response("list", json!([]));
            fixture.response("search", json!([]));
            fixture
        }

        fn dir(&self) -> &Path {
            &self.cache
        }

        fn response(&self, route: &str, response: Value) {
            fs::write(
                self.dir().join(format!("{route}.response")),
                format!("{response}\n"),
            )
            .unwrap();
        }

        fn read_json(&self, name: &str) -> Value {
            serde_json::from_slice(&fs::read(self.dir().join(name)).unwrap()).unwrap()
        }

        fn sync(&self, force: bool, explicit: bool) -> SyncAmpResult {
            sync_amp_cache_with(self.dir(), force, explicit, || Amp::new(self.dir().join("amp")))
                .unwrap()
        }
    }

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
        let cache = dir.path().join("amp-cache");
        let path = state_path(&cache);
        let must_not_run =
            || -> Result<Amp> { panic!("state must be checked before Amp is prepared") };
        let result = sync_amp_cache_with(&cache, false, false, must_not_run).unwrap();
        assert!(!result.synced);
        assert!(!path.exists());

        fs::create_dir(&cache).unwrap();
        let draft = initial_state_path(&cache);
        fs::write(&draft, b"not JSON").unwrap();
        assert!(sync_amp_cache_with(&cache, true, true, must_not_run).is_err());
        assert_eq!(fs::read(&draft).unwrap(), b"not JSON");
        fs::remove_file(draft).unwrap();

        for state in [
            r#"{"version":4,"syncedAt":"2026-11-19T05:20:21Z","threads":{}}"#,
            r#"{"version":2,"syncedAt":"2026-11-19T05:20:21Z","threads":{"T-1":null}}"#,
            r#"{"version":1,"syncedAt":null,"threads":{}}"#,
            r#"{"version":1,"syncedAt":"2026-11-19T05:20:21Z","threads":{"T-1":"private-value"}}"#,
            r#"{"version":1,"syncedAt":"2026-11-19T05:20:21Z","threads":{"../private-value":"2026-11-19T05:20:21Z"}}"#,
            r#"{"version":2,"syncedAt":null,"threads":{}}"#,
            r#"{"version":2,"syncedAt":"2026-11-19T05:20:21Z","threads":{},"pending":{"../private-value":"2026-11-19T05:20:21Z"}}"#,
        ] {
            fs::write(&path, state).unwrap();
            for explicit in [false, true] {
                let error = sync_amp_cache_with(&cache, true, explicit, must_not_run).unwrap_err();
                assert!(!format!("{error:#}").contains("private-value"));
                assert_eq!(fs::read_to_string(&path).unwrap(), state);
            }
        }
        assert!(!threads_dir(&cache).exists());
        assert_eq!(policy_lock(&cache).unwrap().metadata().unwrap().len(), 0);
    }

    #[test]
    fn failed_amp_preparation_keeps_legacy_policy_unmarked() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("amp-cache");
        let result = sync_amp_cache_with(&cache, false, true, || {
            assert!(policy_lock(&cache).is_err());
            anyhow::bail!("synthetic Amp preparation failure");
        })
        .unwrap();

        assert_eq!(result.error.as_deref(), Some("synthetic Amp preparation failure"));
        assert!(!cache.exists());
        assert_eq!(policy_lock(&cache).unwrap().metadata().unwrap().len(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn newer_list_watermark_wins_over_stale_search_with_upstream_v1_state() {
        let amp = SyntheticAmp::new();
        fs::write(
            state_path(amp.dir()),
            r#"{"version":1,"syncedAt":"2025-01-02T00:00:00+00:00","threads":{"T-1":"2025-01-02T00:00:00Z"}}"#,
        )
        .unwrap();
        fs::create_dir(threads_dir(amp.dir())).unwrap();
        fs::write(
            threads_dir(amp.dir()).join("T-1.json"),
            r#"{"id":"T-1","messages":[]}"#,
        )
        .unwrap();
        amp.response("list", json!([{"id":"T-1","updated":UPDATED}]));
        amp.response("search", json!([{"id":"T-1","updatedAt":FIRST}]));
        amp.response("T-1", json!({"id":"T-1","messages":[]}));

        let result = amp.sync(false, false);
        assert!(result.synced);
        assert_eq!(result.exported, 1);
        assert_eq!(amp.read_json("state.json")["threads"]["T-1"], UPDATED);

        amp.response("list", json!([{"id":"T-1","updated":FIRST}]));
        assert_eq!(amp.sync(true, true).exported, 1);
        assert_eq!(amp.read_json("state.json")["threads"]["T-1"], UPDATED);
        assert_eq!(amp.sync(false, false).exported, 0);
    }

    #[cfg(unix)]
    #[test]
    fn first_sync_checkpoints_list_before_failed_search_without_opt_in() {
        let amp = SyntheticAmp::new();
        amp.response("list", json!([{"id": "T-1", "updated": UPDATED}]));
        amp.response("T-1", json!({"id": "T-1", "messages": []}));
        fs::remove_file(amp.dir().join("search.response")).unwrap();

        let result = serde_json::to_value(amp.sync(false, true)).unwrap();
        assert_eq!(result["synced"], false);
        assert_eq!(result["threads"], 1);
        assert_eq!(result["exported"], 0);
        assert_eq!(result["failed"], 0);
        assert_eq!(result["pending"], 1);
        assert_eq!(result["droppedPending"], 0);
        assert_eq!(result["autoRefreshEnabled"], false);
        assert!(result["error"].as_str().unwrap().contains("amp threads search"));
        for name in ["search.before", "initial-state.json"] {
            assert_eq!(amp.read_json(name)["pending"], json!({"T-1": UPDATED}));
        }
        assert!(!amp.dir().join("T-1.before").exists());
        assert!(!state_path(amp.dir()).exists());

        amp.response("list", json!([]));
        amp.response("search", json!([]));
        let result = amp.sync(false, true);
        assert!(!result.synced);
        assert_eq!(
            (result.exported, result.failed, result.pending),
            (0, 0, Some(1))
        );
        assert_eq!(result.auto_refresh_enabled, Some(false));
        assert!(result.error.is_none());
        assert!(!amp.dir().join("T-1.before").exists());
        assert!(!state_path(amp.dir()).exists());

        amp.response("list", json!([{"id": "T-1", "updated": UPDATED}]));
        let result = amp.sync(false, true);
        assert!(result.synced);
        assert_eq!(
            (result.exported, result.pending, result.auto_refresh_enabled),
            (1, Some(0), Some(true))
        );
        assert_eq!(amp.read_json("state.json")["threads"]["T-1"], UPDATED);
    }

    #[cfg(unix)]
    #[test]
    fn later_discovery_failure_keeps_every_completed_list_page() {
        for (failed, command) in [
            ("list-1000", "amp threads list"),
            ("search", "amp threads search"),
        ] {
            let amp = SyntheticAmp::new();
            fs::write(
                state_path(amp.dir()),
                json!({"version": 2, "syncedAt": FIRST, "threads": {"T-1": FIRST, "T-2": FIRST}})
                    .to_string(),
            )
            .unwrap();
            for (route, id) in [("list", "T-1"), ("list-500", "T-2")] {
                let mut page = vec![json!({"id": id, "updated": UPDATED, "messageCount": 1})];
                page.extend((1..LIST_PAGE_SIZE).map(|i| {
                    json!({"id": format!("T-{route}-empty-{i}"), "updated": FIRST, "messageCount": 0})
                }));
                amp.response(route, Value::Array(page));
            }
            if failed == "search" {
                amp.response("list-1000", json!([]));
                fs::remove_file(amp.dir().join("search.response")).unwrap();
            }

            let result = amp.sync(false, false);
            assert!(!result.synced);
            assert_eq!(result.threads, 2);
            assert_eq!(
                (result.exported, result.failed, result.pending),
                (0, 0, Some(2))
            );
            assert_eq!(result.auto_refresh_enabled, Some(true));
            assert!(result.error.unwrap().contains(command));
            assert_eq!(amp.read_json("list-500.before")["pending"], json!({"T-1": UPDATED}));
            let pending = json!({"T-1": UPDATED, "T-2": UPDATED});
            assert_eq!(amp.read_json(&format!("{failed}.before"))["pending"], pending);
            let state = amp.read_json("state.json");
            assert_eq!(state["pending"], pending);
            assert_eq!(state["threads"], json!({"T-1": FIRST, "T-2": FIRST}));
            assert_eq!(state["syncedAt"], FIRST);
            assert!(!amp.dir().join("T-1.before").exists());
            assert!(!amp.dir().join("T-2.before").exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn rediscovered_pending_retries_even_with_a_stale_list_watermark() {
        let amp = SyntheticAmp::new();
        amp.response("search", json!([{"id":"T-1","updatedAt":FIRST}]));
        amp.response("T-1", json!({"id":"T-1","messages":[]}));
        assert_eq!(amp.sync(false, true).exported, 1);

        amp.response("search", json!([{"id":"T-1","updatedAt":UPDATED}]));
        fs::remove_file(amp.dir().join("T-1.response")).unwrap();
        assert_eq!(amp.sync(false, false).failed, 1);
        for name in ["T-1.before", "state.json"] {
            let state = amp.read_json(name);
            assert_eq!(state["threads"]["T-1"], FIRST);
            assert_eq!(state["pending"]["T-1"], UPDATED);
        }
        fs::remove_file(amp.dir().join("T-1.before")).unwrap();

        amp.response("search", json!([{"id":"T-2","updatedAt":FIRST}]));
        amp.response("T-2", json!({"id":"T-2","messages":[]}));
        let result = amp.sync(true, false);
        assert_eq!((result.exported, result.failed), (1, 0));
        assert_eq!(result.pending, Some(1));
        assert_eq!(result.dropped_pending, 0);
        assert_eq!(result.auto_refresh_enabled, Some(true));
        assert!(!amp.dir().join("T-1.before").exists());

        amp.response("search", json!([]));
        amp.response("list", json!([{"id":"T-1","updated":FIRST,"messageCount":0}]));
        amp.response("T-1", json!({"id":"T-1","messages":[]}));
        assert_eq!(amp.sync(false, false).exported, 1);
        let state = amp.read_json("state.json");
        assert_eq!(state["threads"]["T-1"], UPDATED);
        assert!(state.get("pending").is_none());
        amp.response("list", json!([{"id":"T-1","updated":FIRST,"messageCount":1}]));
        assert_eq!(amp.sync(false, false).exported, 0);
    }

    #[cfg(unix)]
    #[test]
    fn manual_force_retains_unseen_pending_on_export_failure_and_reexports_on_return() {
        let amp = SyntheticAmp::new();
        amp.response(
            "search",
            json!([{"id":"T-1","updatedAt":FIRST}, {"id":"T-3","updatedAt":FIRST}]),
        );
        for id in ["T-1", "T-3"] {
            amp.response(id, json!({"id":id,"messages":[]}));
        }
        assert_eq!(amp.sync(false, true).exported, 2);
        let cache = threads_dir(amp.dir()).join("T-1.json");
        let cached = fs::read(&cache).unwrap();

        amp.response("search", json!([{"id":"T-1","updatedAt":UPDATED}]));
        fs::remove_file(amp.dir().join("T-1.response")).unwrap();
        assert_eq!(amp.sync(false, false).failed, 1);
        fs::remove_file(amp.dir().join("T-1.before")).unwrap();

        amp.response(
            "search",
            json!([{"id":"T-2","updatedAt":FIRST}, {"id":"T-4","updatedAt":FIRST}]),
        );
        amp.response("T-2", json!({"id":"T-2","messages":[]}));
        let result = amp.sync(true, true);
        assert_eq!((result.exported, result.failed, result.pending), (1, 1, Some(2)));
        assert_eq!(result.dropped_pending, 0);
        assert_eq!(amp.read_json("state.json")["pending"], json!({"T-1":UPDATED,"T-4":FIRST}));

        amp.response("T-4", json!({"id":"T-4","messages":[]}));
        let result = amp.sync(true, true);
        assert!(result.synced);
        assert_eq!((result.exported, result.pending, result.dropped_pending), (2, Some(0), 1));
        assert_eq!(
            amp.read_json("state.json")["threads"],
            json!({"T-1":null,"T-2":FIRST,"T-3":FIRST,"T-4":FIRST}),
        );
        assert!(!amp.dir().join("T-1.before").exists());
        assert_eq!(fs::read(&cache).unwrap(), cached);

        amp.response("search", json!([{"id":"T-1","updatedAt":FIRST}]));
        amp.response("T-1", json!({"id":"T-1","messages":[]}));
        assert_eq!(amp.sync(false, false).exported, 1);
        assert_eq!(amp.read_json("state.json")["threads"]["T-1"], FIRST);
    }

    #[cfg(unix)]
    #[test]
    fn manual_force_enables_first_sync_only_after_discovery_and_state_write_succeed() {
        let amp = SyntheticAmp::new();
        amp.response("search", json!([{"id":"T-1","updatedAt":UPDATED}]));
        assert_eq!(amp.sync(false, true).failed, 1);
        let draft = initial_state_path(amp.dir());
        let original = fs::read(&draft).unwrap();

        fs::remove_file(amp.dir().join("search.response")).unwrap();
        let result = amp.sync(true, true);
        assert!(result.error.as_deref().unwrap().contains("amp threads search"));
        assert_eq!(result.dropped_pending, 0);
        assert_eq!(fs::read(&draft).unwrap(), original);
        assert!(!state_path(amp.dir()).exists());

        amp.response("search", json!([]));
        let blocked = draft.with_extension(format!("{}.tmp", std::process::id()));
        fs::create_dir(&blocked).unwrap();
        let result = amp.sync(true, true);
        assert!(!result.synced);
        assert!(result.error.is_some());
        assert_eq!((result.pending, result.dropped_pending), (Some(1), 0));
        assert_eq!(fs::read(&draft).unwrap(), original);
        assert!(!state_path(amp.dir()).exists());
        fs::remove_dir(blocked).unwrap();

        let result = serde_json::to_value(amp.sync(true, true)).unwrap();
        assert_eq!(result["synced"], true);
        assert_eq!(result["exported"], 0);
        assert_eq!(result["pending"], 0);
        assert_eq!(result["droppedPending"], 1);
        assert_eq!(result["autoRefreshEnabled"], true);
        assert!(!draft.exists());
        assert!(state_path(amp.dir()).is_file());
    }

    #[cfg(unix)]
    #[test]
    fn partial_first_sync_preserves_progress_without_enabling_refresh() {
        let amp = SyntheticAmp::new();
        amp.response(
            "search",
            json!([{"id":"T-1","updatedAt":FIRST}, {"id":"T-2","updatedAt":FIRST}]),
        );
        let result = amp.sync(false, true);
        assert!(!result.synced);
        assert_eq!((result.exported, result.failed), (0, 2));
        assert_eq!(result.pending, Some(2));
        assert_eq!(result.auto_refresh_enabled, Some(false));
        assert_eq!(
            amp.read_json("T-1.before")["pending"],
            json!({"T-1":FIRST,"T-2":FIRST}),
        );
        assert!(!state_file_exists(&state_path(amp.dir())));

        amp.response("T-1", json!({"id":"T-1","messages":[]}));
        let result = amp.sync(false, true);
        assert!(result.synced);
        assert_eq!((result.exported, result.failed), (1, 1));
        assert_eq!(result.pending, Some(1));
        assert_eq!(result.auto_refresh_enabled, Some(false));
        assert!(!state_file_exists(&state_path(amp.dir())));
        let draft = amp.read_json("initial-state.json");
        assert_eq!(draft["threads"], json!({"T-1":FIRST}));
        assert_eq!(draft["pending"], json!({"T-2":FIRST}));
        let status = serde_json::to_value(amp_status_at(amp.dir(), None).unwrap()).unwrap();
        assert_eq!(status["hasCache"], true);
        assert_eq!(status["cachedThreads"], 1);
        assert_eq!(status["pending"], 1);
        assert_eq!(status["autoRefreshEnabled"], false);
        assert!(status["lastSyncedAt"].is_null());
        assert!(!sync_amp_cache_with(amp.dir(), true, false, || {
            panic!("an incomplete first sync must not authorize a refresh")
        })
        .unwrap()
        .synced);

        amp.response("search", json!([{"id":"T-1","updatedAt":FIRST}]));
        let result = amp.sync(false, true);
        assert!(!result.synced);
        assert_eq!(
            (result.exported, result.failed, result.pending),
            (0, 0, Some(1))
        );
        assert_eq!(result.auto_refresh_enabled, Some(false));
        assert!(result.error.is_none());
        assert!(!state_file_exists(&state_path(amp.dir())));

        amp.response("search", json!([{"id":"T-2","updatedAt":FIRST}]));
        amp.response("T-2", json!({"id":"T-2","messages":[]}));
        let result = amp.sync(false, true);
        assert!(result.synced);
        assert_eq!(result.exported, 1);
        assert_eq!(result.pending, Some(0));
        assert_eq!(result.auto_refresh_enabled, Some(true));
        assert_eq!(
            amp.read_json("state.json")["threads"],
            json!({"T-1":FIRST,"T-2":FIRST}),
        );
        assert!(!state_file_exists(&initial_state_path(amp.dir())));
    }

    #[test]
    fn legacy_replacement_and_first_sync_cannot_hold_the_same_policy_lock() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("amp-cache");
        let guard = policy_lock(&cache).unwrap();
        assert!(policy_lock(&cache).is_err());
        assert!(!cache.exists());
        drop(guard);
        assert_eq!(policy_lock(&cache).unwrap().metadata().unwrap().len(), 0);
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
