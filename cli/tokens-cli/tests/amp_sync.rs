//! Exercises `features/amp-sync.feature` with the real tokens CLI and synthetic Amp exports.

use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

const LISTED: &str = "T-listed";
const RECENT: &str = "T-recent";
const FIRST: &str = "2025-01-02T03:04:05Z";
const NEXT_DAY: &str = "2025-01-03T03:04:05Z";
const UPDATED: &str = "2025-01-02T04:04:05Z";
const CREATED: i64 = 1_735_787_045_000;
const MODEL: &str = "claude-sonnet-4-6";
const PRIVATE_EXPORT: &str = "AMP_PRIVATE_EXPORT_SENTINEL";
const PRIVATE_STDERR: &str = "AMP_PRIVATE_STDERR_SENTINEL";
const BOTH_EXPORTS: &[&str] = &["list", "search", LISTED, RECENT];
const SUBMIT: &[&str] = &["submit", "--dry-run", "--json", "--client", "amp"];
const SUBMIT_BOUNDS: &[&str] = &["--since", "2025-01-02", "--until", "2025-01-03"];

// Only OS shell builtins: no interpreter or tools resolved through the host PATH.
#[cfg(unix)]
const AMP_SCRIPT: &str = r#"case "$1 $2 $3" in
    'threads list '*) route=list ;;
    'threads search author:me') route=search ;;
    'threads export T-listed'|'threads export T-recent') route=$3 ;;
    *) route=unexpected ;;
esac
if [ "$route" = list ] && [ "$7" = --offset ] && [ -f "$AMP_FIXTURE/list-$8.json" ]; then
    route=list-$8
fi
if [ "${TOKENS_API_TOKEN+x}" = x ]; then route=unexpected-token; fi
printf 'called\n' >>"$AMP_FIXTURE/calls/$route" || exit 2
IFS= read -r response <"$AMP_FIXTURE/$route.json" || exit 2
printf '%s\n' "$response"
if [ -f "$AMP_FIXTURE/$route.stderr" ]; then
    IFS= read -r message <"$AMP_FIXTURE/$route.stderr"
    printf '%s\n' "$message" >&2
    exit 19
fi
"#;

#[cfg(windows)]
const AMP_SCRIPT: &str = r#"@echo off
setlocal EnableExtensions DisableDelayedExpansion
set "route=unexpected"
if "%~1 %~2"=="threads list" set "route=list"
if "%~1 %~2 %~3"=="threads search author:me" set "route=search"
if "%~1 %~2 %~3"=="threads export T-listed" set "route=T-listed"
if "%~1 %~2 %~3"=="threads export T-recent" set "route=T-recent"
if "%route%"=="list" if "%~7"=="--offset" if exist "%AMP_FIXTURE%\list-%~8.json" set "route=list-%~8"
if defined TOKENS_API_TOKEN set "route=unexpected-token"
>>"%AMP_FIXTURE%\calls\%route%" echo called
if errorlevel 1 exit /b 2
type "%AMP_FIXTURE%\%route%.json"
if errorlevel 1 exit /b 2
if exist "%AMP_FIXTURE%\%route%.stderr" (
    type "%AMP_FIXTURE%\%route%.stderr" >&2
    exit /b 19
)
exit /b 0
"#;

struct AmpFixture(TempDir);

impl AmpFixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        assert!(fixture.0.path().is_absolute());
        for dir in ["home", "tmp", "bin", "amp-fixture/calls"] {
            fs::create_dir_all(fixture.path(dir)).unwrap();
        }
        let amp = fixture.amp();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let shell = if cfg!(target_os = "android") {
                "/system/bin/sh"
            } else {
                "/bin/sh"
            };
            fs::write(&amp, format!("#!{shell}\n{AMP_SCRIPT}")).unwrap();
            fs::set_permissions(&amp, fs::Permissions::from_mode(0o700)).unwrap();
        }
        #[cfg(windows)]
        fs::write(&amp, AMP_SCRIPT.replace('\n', "\r\n")).unwrap();

        fixture.discovery(FIRST, FIRST);
        fixture.response(LISTED, thread(LISTED, 11));
        fixture.response(RECENT, thread(RECENT, 23));
        fixture
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.path().join(relative)
    }

    fn amp(&self) -> PathBuf {
        self.path("bin")
            .join(if cfg!(windows) { "amp.cmd" } else { "amp" })
    }

    fn cached_thread(&self, id: &str) -> PathBuf {
        self.path("config/amp-cache/threads").join(format!("{id}.json"))
    }

    fn response(&self, route: &str, json: Value) {
        fs::write(
            self.path("amp-fixture").join(format!("{route}.json")),
            format!("{json}\n"),
        )
        .unwrap();
    }

    fn discovery(&self, listed: &str, recent: &str) {
        self.response("list", json!([{"id": LISTED, "updated": listed, "messageCount": 2}]));
        self.response("search", json!([{"id": RECENT, "updatedAt": recent}]));
    }

    fn fail_export(&self, id: &str) -> PathBuf {
        let path = self.path("amp-fixture").join(format!("{id}.stderr"));
        fs::write(&path, format!("{PRIVATE_STDERR}\n")).unwrap();
        path
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "tokens {args:?} exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn command(&self, args: &[&str], endpoint: &str) -> Command {
        let home = self.path("home");
        let mut command = Command::new(env!("CARGO_BIN_EXE_tokens"));
        command
            .args(args)
            .timeout(Duration::from_secs(10))
            .env_clear()
            .current_dir(self.0.path())
            .env("PATH", std::env::join_paths([self.path("bin")]).unwrap())
            .env("AMP_FIXTURE", self.path("amp-fixture"))
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("APPDATA", home.join("AppData/Roaming"))
            .env("LOCALAPPDATA", home.join("AppData/Local"))
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("XDG_DATA_HOME", home.join(".local/share"))
            .env("XDG_CACHE_HOME", home.join(".cache"))
            .env("TOKENS_CONFIG_DIR", self.path("config"));
        for var in ["TMPDIR", "TMP", "TEMP"] {
            command.env(var, self.path("tmp"));
        }
        for var in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "TOKENS_API_URL"] {
            command.env(var, endpoint);
        }
        #[cfg(windows)]
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", system_root);
        }
        if args[0] == "submit" {
            command
                .env("TOKENS_API_TOKEN", "synthetic-amp-test-token")
                .env("TOKENS_DEVICE_ID", "amp-integration-test");
        }
        command
    }

    fn invoke(&self, args: &[&str]) -> Output {
        // Trap accidental network access, including proxy-aware HTTP(S).
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", proxy.local_addr().unwrap());
        let output = self.command(args, &endpoint).output().unwrap();
        proxy.set_nonblocking(true).unwrap();
        assert!(
            proxy.accept().is_err_and(|error| error.kind() == ErrorKind::WouldBlock),
            "tokens {args:?} opened a network connection to the local proxy"
        );
        assert_private_text_absent(&output.stdout, "stdout");
        assert_private_text_absent(&output.stderr, "stderr");
        let cache = self.path("config/amp-cache");
        if cache.exists() {
            assert_private_cache(&cache);
        }
        output
    }

    fn sync(&self, options: &[&str], exported: u64, failed: u64) {
        let mut args = vec!["amp", "sync", "--json"];
        args.extend_from_slice(options);
        let result = output_json(&self.run(&args));
        assert_eq!(result["synced"], failed == 0 || exported > 0, "{result}");
        assert_eq!(result["threads"], 2);
        assert_eq!(result["exported"], exported);
        assert_eq!(result["failed"], failed);
        if failed == 0 {
            assert!(result["error"].is_null());
        } else {
            assert!(result["error"].as_str().is_some_and(|error| !error.is_empty()));
        }
    }

    fn assert_calls(&self, expected: &[&str]) {
        let mut actual = Vec::new();
        // Exports run in parallel and log to separate files, so order is irrelevant.
        for entry in fs::read_dir(self.path("amp-fixture/calls")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = entry.file_name().into_string().unwrap();
            for _ in fs::read_to_string(&path).unwrap().lines() {
                actual.push(name.clone());
            }
            fs::remove_file(path).unwrap();
        }
        actual.sort_unstable();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(actual, expected);
    }

    fn prepare_offline_submit(&self) {
        let cache = self.path("config/cache");
        fs::create_dir_all(&cache).unwrap();
        let timestamp = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let pricing = json!({"timestamp": timestamp, "data": {}}).to_string();
        for source in ["litellm", "models-dev", "openrouter"] {
            fs::write(cache.join(format!("pricing-{source}.json")), &pricing).unwrap();
        }
        fs::write(self.path("config/settings.json"), r#"{"timezone":"UTC"}"#).unwrap();
        // A local thread makes the pre-opt-in preview produce a payload, not a no-data message.
        let legacy = self.path("home/.local/share/amp/threads");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("T-local.json"), thread("T-local", 100).to_string()).unwrap();
    }

    fn assert_replace_refused(&self) {
        let mut args = vec!["submit", "--replace", "--client", "amp"];
        args.extend_from_slice(SUBMIT_BOUNDS);
        let replacement = self.invoke(&args);
        assert!(!replacement.status.success());
        let error = String::from_utf8_lossy(&replacement.stderr);
        assert!(error.contains("--replace") && error.contains("Amp"), "{error}");
    }
}

fn output_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("expected tokens JSON output")
}

fn thread(id: &str, output: i64) -> Value {
    thread_at(id, output, FIRST)
}

fn thread_at(id: &str, output: i64, timestamp: &str) -> Value {
    json!({
        "id": id,
        "created": CREATED,
        "title": PRIVATE_EXPORT,
        "messages": [
            {"role": "user", "messageId": 1, "content": [{"text": PRIVATE_EXPORT}]},
            {
                "role": "assistant",
                "messageId": 2,
                "content": [{"text": PRIVATE_EXPORT}, {"toolOutput": {"body": PRIVATE_EXPORT}}],
                "usage": {
                    "model": MODEL,
                    "timestamp": timestamp,
                    "inputTokens": 2,
                    "outputTokens": output,
                    "cacheReadInputTokens": 3,
                    "cacheCreationInputTokens": 4,
                    "metadata": {"credentials": [{"apiKey": PRIVATE_EXPORT}]}
                }
            }
        ],
        "usageLedger": {
            "metadata": {"prompt": PRIVATE_EXPORT},
            "events": [{
                "model": MODEL,
                "timestamp": timestamp,
                "credits": 0.25,
                "toMessageId": 2,
                "metadata": {"reply": PRIVATE_EXPORT},
                "tokens": {
                    "input": 2, "output": output,
                    "cacheReadInputTokens": 3,
                    "cacheCreationInputTokens": 4,
                    "metadata": {"details": [{(PRIVATE_EXPORT): true}]}
                }
            }]
        }
    })
}

fn assert_private_text_absent(bytes: &[u8], location: &str) {
    let text = String::from_utf8_lossy(bytes);
    assert!(!text.contains(PRIVATE_EXPORT), "export data leaked into {location}");
    assert!(!text.contains(PRIVATE_STDERR), "Amp stderr leaked into {location}");
}

fn assert_private_cache(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            assert_private_cache(&path);
        } else if kind.is_file() {
            assert_private_text_absent(&fs::read(&path).unwrap(), &path.display().to_string());
        }
    }
}

#[test]
fn amp_status_does_not_run_amp_or_create_state() {
    let fixture = AmpFixture::new();
    let status = output_json(&fixture.run(&["amp", "status", "--json"]));
    assert_eq!(status["ampCli"], fixture.amp().display().to_string());
    assert_eq!(status["hasCache"], false);
    assert_eq!(status["cachedThreads"], 0);
    assert!(status["lastSyncedAt"].is_null());
    assert!(!fixture.path("config/amp-cache").exists());
    fixture.assert_calls(&[]);
}

#[test]
fn amp_status_counts_only_thread_files_that_still_exist() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);

    fs::remove_file(fixture.cached_thread(LISTED)).unwrap();
    let threads = fixture.path("config/amp-cache/threads");
    fs::write(threads.join("notes.json"), "{}").unwrap();
    fs::write(threads.join("T-temp.tmp"), "{}").unwrap();
    fs::create_dir(threads.join("T-directory.json")).unwrap();

    let status = output_json(&fixture.run(&["amp", "status", "--json"]));
    assert_eq!(status["hasCache"], true);
    assert_eq!(status["cachedThreads"], 1);
    assert!(status["lastSyncedAt"].is_string());
    fixture.assert_calls(&[]);
}

#[test]
fn amp_sync_combines_list_and_search_and_only_exports_changed_threads_unless_forced() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let cached: Value =
        serde_json::from_slice(&fs::read(fixture.cached_thread(RECENT)).unwrap()).unwrap();
    assert_eq!(cached["id"], RECENT);
    assert_eq!(cached["created"], CREATED);
    assert_eq!(cached["messages"].as_array().unwrap().len(), 1);
    let message = &cached["messages"][0];
    assert_eq!(message["role"], "assistant");
    assert_eq!(message["messageId"], 2);
    assert_eq!(message["usage"]["model"], MODEL);
    assert_eq!(message["usage"]["timestamp"], FIRST);
    assert_eq!(message["usage"]["outputTokens"], 23);
    assert_eq!(cached["usageLedger"]["events"][0]["tokens"]["output"], 23);
    assert_eq!(cached["usageLedger"]["events"][0]["credits"], 0.25);

    let state_path = fixture.path("config/amp-cache/state.json");
    let state = fs::read(&state_path).unwrap();
    let status = output_json(&fixture.run(&["amp", "status", "--json"]));
    assert_eq!(status["hasCache"], true);
    assert_eq!(status["cachedThreads"], 2);
    assert!(status["lastSyncedAt"].is_string());
    assert_eq!(fs::read(state_path).unwrap(), state);
    fixture.assert_calls(&[]);

    fixture.sync(&[], 0, 0);
    fixture.assert_calls(&["list", "search"]);
    fixture.sync(&["--force"], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
}

#[test]
fn amp_sync_stops_on_a_repeated_full_list_page() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let state_path = fixture.path("config/amp-cache/state.json");
    let state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();

    let page = (0..500)
        .map(|i| json!({"id": format!("T-page-{i}"), "updated": FIRST, "messageCount": 2}))
        .collect();
    fixture.response("list", Value::Array(page));
    let result = output_json(&fixture.run(&["amp", "sync", "--json"]));
    assert_eq!(result["synced"], false);
    assert!(result["error"].as_str().unwrap().contains("amp threads list"));
    fixture.assert_calls(&["list", "list"]);
    let after: Value = serde_json::from_slice(&fs::read(state_path).unwrap()).unwrap();
    assert_eq!(after["threads"], state["threads"]);
    assert_eq!(after["syncedAt"], state["syncedAt"]);
    assert_eq!(after["pending"].as_object().unwrap().len(), 500);
    assert_eq!(result["pending"], 500);
    assert_eq!(result["autoRefreshEnabled"], true);
}

#[test]
fn amp_sync_keeps_newer_nonempty_thread_when_later_list_page_returns_stale_empty_version() {
    let fixture = AmpFixture::new();
    let mut first_page = vec![json!({"id": LISTED, "updated": UPDATED, "messageCount": 2})];
    first_page.extend(
        (1..500)
            .map(|i| json!({"id": format!("T-page-{i}"), "updated": FIRST, "messageCount": 0})),
    );
    fixture.response("list", Value::Array(first_page));
    fixture.response(
        "list-500",
        json!([{"id": LISTED, "updated": FIRST, "messageCount": 0}]),
    );
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(&["list", "list-500", "search", LISTED, RECENT]);

    fixture.discovery(UPDATED, FIRST);
    fixture.sync(&[], 0, 0);
    fixture.assert_calls(&["list", "search"]);
}

#[test]
fn amp_partial_sync_retains_failed_thread_and_retries_only_that_thread() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let listed = fixture.cached_thread(LISTED);
    let recent = fixture.cached_thread(RECENT);
    let old_listed = fs::read(&listed).unwrap();
    let old_recent = fs::read(&recent).unwrap();

    fixture.discovery(UPDATED, UPDATED);
    fixture.response(LISTED, thread(LISTED, 31));
    fixture.response(RECENT, thread(RECENT, 43));
    let failure = fixture.fail_export(LISTED);
    fixture.sync(&[], 1, 1);
    fixture.assert_calls(BOTH_EXPORTS);
    assert_eq!(fs::read(&listed).unwrap(), old_listed);
    assert_ne!(fs::read(&recent).unwrap(), old_recent);

    fs::remove_file(failure).unwrap();
    fixture.sync(&[], 1, 0);
    fixture.assert_calls(&["list", "search", LISTED]);
    assert_ne!(fs::read(listed).unwrap(), old_listed);
}

#[test]
fn amp_sync_keeps_state_and_retries_when_all_exports_fail() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let state_path = fixture.path("config/amp-cache/state.json");
    let state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    let listed = fixture.cached_thread(LISTED);
    let recent = fixture.cached_thread(RECENT);
    let old_listed = fs::read(&listed).unwrap();
    let old_recent = fs::read(&recent).unwrap();

    fixture.discovery(UPDATED, UPDATED);
    fixture.response(LISTED, thread(LISTED, 31));
    fixture.response(RECENT, thread(RECENT, 43));
    let failures = [fixture.fail_export(LISTED), fixture.fail_export(RECENT)];
    fixture.sync(&[], 0, 2);
    fixture.assert_calls(BOTH_EXPORTS);
    let failed_state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    assert_eq!(failed_state["threads"], state["threads"]);
    assert_eq!(failed_state["syncedAt"], state["syncedAt"]);
    assert_eq!(fs::read(&listed).unwrap(), old_listed);
    assert_eq!(fs::read(&recent).unwrap(), old_recent);

    for failure in failures {
        fs::remove_file(failure).unwrap();
    }
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    assert_ne!(fs::read(listed).unwrap(), old_listed);
    assert_ne!(fs::read(recent).unwrap(), old_recent);
    fixture.sync(&[], 0, 0);
    fixture.assert_calls(&["list", "search"]);
}

#[test]
fn amp_sync_does_not_echo_a_malformed_usage_value() {
    let fixture = AmpFixture::new();
    let mut malformed = thread(LISTED, 11);
    malformed["messages"][1]["usage"]["inputTokens"] = json!(PRIVATE_EXPORT);
    fixture.response(LISTED, malformed);
    fixture.sync(&[], 1, 1);
    fixture.assert_calls(BOTH_EXPORTS);
    assert!(!fixture.cached_thread(LISTED).exists());
    assert!(!fixture.path("config/amp-cache/state.json").exists());
    fixture.prepare_offline_submit();
    let preview = output_json(&fixture.run(SUBMIT));
    assert_eq!(preview["summary"]["totalTokens"], 141);
    fixture.assert_calls(&[]);
}

#[test]
fn amp_submit_uses_cached_usage_after_failure_but_replace_refuses_partial_refresh() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    let before = output_json(&fixture.run(SUBMIT));
    assert_eq!(before["summary"]["totalTokens"], 109);
    assert!(!fixture.path("config/amp-cache").exists());
    fixture.assert_calls(&[]);

    // Give the search-only thread a second day that replacement could otherwise erase.
    fixture.response(RECENT, thread_at(RECENT, 23, NEXT_DAY));
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    fixture.discovery(UPDATED, FIRST);
    fixture.response(LISTED, thread(LISTED, 31));
    let listed_failure = fixture.fail_export(LISTED);
    let preview = fixture.run(SUBMIT);
    fixture.assert_calls(&["list", "search", LISTED]);
    // Local 109 + old listed 20 + recent 32. The failed 40-token update is not counted.
    assert_eq!(output_json(&preview)["summary"]["totalTokens"], 161);
    assert!(String::from_utf8_lossy(&preview.stderr).contains("amp threads export"));

    fs::remove_file(listed_failure).unwrap();
    fs::remove_file(fixture.cached_thread(RECENT)).unwrap();
    fixture.fail_export(RECENT);
    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
    assert!(!fixture.cached_thread(RECENT).exists());
}

#[test]
fn amp_sync_counts_a_ledger_only_export_without_retaining_private_content() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    let mut ledger_only = thread(LISTED, 11);
    ledger_only.as_object_mut().unwrap().remove("messages");
    fixture.response(LISTED, ledger_only);
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let cached: Value = serde_json::from_slice(&fs::read(fixture.cached_thread(LISTED)).unwrap()).unwrap();
    assert_eq!(cached["messages"], json!([]));
    assert_eq!(cached["usageLedger"]["events"][0]["tokens"]["output"], 11);
    assert_eq!(output_json(&fixture.run(SUBMIT))["summary"]["totalTokens"], 161);
}

#[test]
fn amp_replace_refuses_an_id_only_cache_file() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    fixture.response("search", json!([]));
    fs::write(fixture.cached_thread(RECENT), json!({"id": RECENT}).to_string()).unwrap();
    fixture.assert_replace_refused();
}

#[test]
fn amp_failed_update_outside_the_search_window_cannot_authorize_replacement() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    fixture.discovery(FIRST, UPDATED);
    let error = fixture.fail_export(RECENT);
    fixture.sync(&[], 0, 1);
    fixture.assert_calls(&["list", "search", RECENT]);
    fixture.response("search", json!([]));
    fixture.assert_replace_refused();

    fs::remove_file(error).unwrap();
    fixture.discovery(FIRST, UPDATED);
    fixture.sync(&[], 1, 0);
    fixture.assert_calls(&["list", "search", RECENT]);
    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
}

#[test]
fn amp_submit_previews_remaining_usage_but_replace_refuses_a_missing_undiscovered_thread() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    fixture.response(RECENT, thread_at(RECENT, 23, NEXT_DAY));
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);

    fixture.response("search", json!([]));
    fs::remove_file(fixture.cached_thread(RECENT)).unwrap();

    let preview = output_json(&fixture.run(&[SUBMIT, SUBMIT_BOUNDS].concat()));
    assert_eq!(preview["summary"]["activeDays"], 1);
    assert_eq!(preview["summary"]["totalTokens"], 129);
    fixture.assert_calls(&["list", "search"]);

    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
}

#[test]
fn amp_submit_replace_allows_legacy_only_but_refuses_sync_files_without_state() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    assert_eq!(output_json(&fixture.run(SUBMIT))["summary"]["totalTokens"], 109);
    fixture.assert_calls(&[]);
    let preview = output_json(&fixture.run(&[SUBMIT, SUBMIT_BOUNDS, &["--replace"]].concat()));
    assert_eq!(preview["summary"]["totalTokens"], 109);
    let amp = &preview["clientManifest"]["clients"][0];
    assert_eq!(amp["client"], "amp");
    assert_eq!(amp["coverage"]["mode"], "full");
    assert!(!fixture.path("config/amp-cache").exists());
    fixture.assert_calls(&[]);

    // An interrupted first sync can publish a thread before it writes state.
    let synced = AmpFixture::new();
    synced.sync(&[], 2, 0);
    synced.assert_calls(BOTH_EXPORTS);
    let orphan = fixture.cached_thread(LISTED);
    fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    fs::copy(synced.cached_thread(LISTED), orphan).unwrap();

    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
    assert!(!fixture.path("config/amp-cache/state.json").exists());
}

#[cfg(unix)]
#[test]
fn amp_replace_refuses_a_dangling_cache_or_previously_synced_config() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    let cache = fixture.path("config/amp-cache");
    std::os::unix::fs::symlink(fixture.path("missing"), &cache).unwrap();
    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
    fs::remove_file(&cache).unwrap();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    fs::rename(&cache, fixture.path("saved-cache")).unwrap();
    fixture.assert_replace_refused();
    fixture.assert_calls(&[]);
}

#[cfg(unix)]
#[test]
fn amp_sync_does_not_publish_through_a_linked_cache_directory() {
    let fixture = AmpFixture::new();
    let outside = fixture.path("outside");
    let cache = fixture.path("config/amp-cache");
    fs::create_dir(&outside).unwrap();
    fs::create_dir_all(&cache).unwrap();
    std::os::unix::fs::symlink(&outside, cache.join("threads")).unwrap();
    let result = output_json(&fixture.run(&["amp", "sync", "--json"]));
    assert_eq!(result["synced"], false);
    assert!(result["error"].is_string());
    assert!(fs::read_dir(outside).unwrap().next().is_none());
    fixture.assert_calls(&[]);
}

#[test]
fn legacy_replace_blocks_first_cloud_sync_until_the_submission_returns() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::time::Instant;

    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let args = [&["submit", "--replace", "--client", "amp"][..], SUBMIT_BOUNDS].concat();
    let mut command = fixture.command(&args, &endpoint);
    std::thread::scope(|scope| {
        let submit = scope.spawn(move || command.output().unwrap());
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(err) if err.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                other => panic!("did not receive the legacy replacement: {other:?}"),
            }
        };
        // BSD may inherit O_NONBLOCK from the listener. Read the request with
        // the socket timeout rather than racing the client's first write.
        socket.set_nonblocking(false).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut request = BufReader::new(&socket);
        let mut line = String::new();
        request.read_line(&mut line).unwrap();
        assert!(line.starts_with("POST ") && line.contains("/api/submit "));
        let mut length = None;
        loop {
            line.clear();
            assert_ne!(request.read_line(&mut line).unwrap(), 0);
            if line == "\r\n" {
                break;
            }
            if let Some((_, v)) = line.split_once(':').filter(|(k, _)| k.eq_ignore_ascii_case("content-length")) {
                length = Some(v.trim().parse::<u64>().unwrap());
            }
        }
        std::io::copy(&mut request.take(length.unwrap()), &mut std::io::sink()).unwrap();
        let sync = output_json(&fixture.run(&["amp", "sync", "--json"]));
        assert_eq!(sync["synced"], false);
        assert!(sync["error"].is_string());
        fixture.assert_calls(&[]);
        assert!(!fixture.path("config/amp-cache").exists());
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap();
        assert!(submit.join().unwrap().status.success());
    });
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
}

#[test]
fn amp_force_drops_only_invisible_retry_work_and_reexports_on_rediscovery() {
    let fixture = AmpFixture::new();
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);
    let recent = fixture.cached_thread(RECENT);
    let cached = fs::read(&recent).unwrap();
    fixture.discovery(FIRST, UPDATED);
    let failure = fixture.fail_export(RECENT);
    fixture.sync(&[], 0, 1);
    fixture.assert_calls(&["list", "search", RECENT]);

    fixture.response("search", json!([]));
    let kept = output_json(&fixture.run(&["amp", "sync", "--json"]));
    assert_eq!(kept["pending"], 1);
    fixture.assert_calls(&["list", "search"]);
    let listed_failure = fixture.fail_export(LISTED);
    let blocked = output_json(&fixture.run(&["amp", "sync", "--json", "--force"]));
    assert_eq!(blocked["pending"], 2);
    assert_eq!(blocked["droppedPending"], 0);
    assert_eq!(blocked["failed"], 1);
    assert_eq!(fs::read(&recent).unwrap(), cached);
    fixture.assert_calls(&["list", "search", LISTED]);
    fs::remove_file(listed_failure).unwrap();
    let recovered = output_json(&fixture.run(&["amp", "sync", "--json", "--force"]));
    assert_eq!(recovered["droppedPending"], 1);
    assert_eq!(recovered["pending"], 0);
    assert_eq!(fs::read(&recent).unwrap(), cached);
    fixture.assert_calls(&["list", "search", LISTED]);

    fs::remove_file(failure).unwrap();
    // The returned ID is visible but list still reports its stale zero message count.
    fixture.response("list", json!([
        {"id":LISTED, "updated":FIRST, "messageCount":2},
        {"id":RECENT, "updated":FIRST, "messageCount":0}
    ]));
    fixture.response(RECENT, thread(RECENT, 41));
    fixture.sync(&[], 1, 0);
    fixture.assert_calls(&["list", "search", RECENT]);
    let refreshed: Value = serde_json::from_slice(&fs::read(&recent).unwrap()).unwrap();
    assert_eq!(refreshed["usageLedger"]["events"][0]["tokens"]["output"], 41);
}
