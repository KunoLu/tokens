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

    fn invoke(&self, args: &[&str]) -> Output {
        let home = self.path("home");
        // Keep proxy-aware HTTP(S) local and trap any accidental backend request.
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", proxy.local_addr().unwrap());
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
            command.env(var, &endpoint);
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

        let output = command.output().unwrap();
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
        if entry.file_type().unwrap().is_dir() {
            assert_private_cache(&path);
        } else {
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
    let state = fs::read(&state_path).unwrap();

    let page = (0..500)
        .map(|i| json!({"id": format!("T-page-{i}"), "updated": FIRST, "messageCount": 2}))
        .collect();
    fixture.response("list", Value::Array(page));
    let result = output_json(&fixture.run(&["amp", "sync", "--json"]));
    assert_eq!(result["synced"], false);
    assert!(result["error"].as_str().unwrap().contains("amp threads list"));
    fixture.assert_calls(&["list", "list"]);
    assert_eq!(fs::read(state_path).unwrap(), state);
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
    let state = fs::read(&state_path).unwrap();
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
    assert_eq!(fs::read(&state_path).unwrap(), state);
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
    fixture.assert_calls(BOTH_EXPORTS);
    assert!(!fixture.cached_thread(RECENT).exists());
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
    fixture.assert_calls(&["list", "search"]);
}

#[test]
fn amp_submit_replace_allows_legacy_only_but_refuses_sync_files_without_state() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
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
fn amp_submit_replace_refuses_symlinked_thread_when_reexport_fails() {
    let fixture = AmpFixture::new();
    fixture.prepare_offline_submit();
    fixture.response(RECENT, thread_at(RECENT, 23, NEXT_DAY));
    fixture.sync(&[], 2, 0);
    fixture.assert_calls(BOTH_EXPORTS);

    let cached = fixture.cached_thread(RECENT);
    let target = fixture.path("linked-thread.json");
    fs::rename(&cached, &target).unwrap();
    std::os::unix::fs::symlink(&target, &cached).unwrap();
    fixture.fail_export(RECENT);

    fixture.assert_replace_refused();
    fixture.assert_calls(&["list", "search", RECENT]);
}
