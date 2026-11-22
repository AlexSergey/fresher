//! The recorder on a shell of fakes: every record folder is a temporary folder, the run's clock a fake.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{Build, CleanupSelection, Confirmation};
use fresher_testing::FakeClock;
use serde_json::{Value, json};

use crate::dev_runs::run::Run;
use crate::dev_runs::{BuildName, DevRuns, Launch};
use crate::testing::{Shell, is_cleanup_finished, is_scan_finished, until_finished};

/// The run's start, 2025-11-20T10:04:31Z.
const STARTED: u64 = 1_763_633_071;

fn launch(folder: &Path) -> Launch {
    let build = BuildName::new(env!("CARGO_PKG_VERSION"), Some("26e4cb0".as_ref()), None);
    Launch { folder: folder.to_path_buf(), build }
}

/// The run of a development build on `shell`, whose home is `home`, recorded in `folder`.
fn begin(shell: &Shell, folder: &Path, home: &Path) -> DevRuns {
    let clock = Arc::new(FakeClock::at(STARTED));
    let run = Run::begin(&launch(folder), &Build::Development, home, Some("27.2".into()), clock, &shell.engine());
    DevRuns::new(run.unwrap().unwrap())
}

/// The run's file in `folder` whose name ends with `suffix`.
fn file_ending(folder: &Path, suffix: &str) -> PathBuf {
    fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(suffix))
        .unwrap()
}

fn snapshot(folder: &Path) -> Value {
    serde_json::from_slice(&fs::read(file_ending(folder, ".json")).unwrap()).unwrap()
}

fn lines(folder: &Path) -> Vec<Value> {
    let text = fs::read_to_string(file_ending(folder, ".events.jsonl")).unwrap();
    text.lines().map(|line| serde_json::from_str(line).unwrap()).collect()
}

/// Scans the shell's home and folds every event of the scan into `runs`.
fn record_scan(shell: &Shell, runs: &DevRuns) {
    let events = shell.engine().subscribe();
    shell.engine().start_scan(fresher_engine::ScanMode::Full).unwrap();
    for event in until_finished(&events, is_scan_finished) {
        runs.event(&event, &shell.engine());
    }
}

/// Prepares `selection`, confirms it with `method` as the dialog does and folds every event of the cleanup into
/// `runs`.
fn record_cleanup(shell: &Shell, runs: &DevRuns, selection: &Value, method: &str) {
    let events = shell.engine().subscribe();
    let selection: CleanupSelection = serde_json::from_value(selection.clone()).unwrap();
    let view = shell.engine().prepare_cleanup(&selection).unwrap();
    let summary = view.summary;
    let confirmation = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                               "highest": summary.highest, "method": method });
    let confirmation: Confirmation = serde_json::from_value(confirmation).unwrap();
    shell.engine().confirm_cleanup(&confirmation).unwrap();
    for event in until_finished(&events, is_cleanup_finished) {
        runs.event(&event, &shell.engine());
    }
}

fn both_trial_categories() -> Value {
    json!({ "findings": [], "categories": [
        { "category": "trial.caches", "except": [] }, { "category": "trial.kept", "except": [] } ] })
}

fn caches_only() -> Value {
    json!({ "findings": [], "categories": [{ "category": "trial.caches", "except": [] }] })
}

/// Every item of the first cleanup's group `group`, as `(the path's last name, the outcome)`, sorted.
fn outcomes(record: &Value, group: &str) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = record["clear"][0]["groups"][group]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| (last_name(&item["path"]), item["outcome"].as_str().unwrap().to_owned()))
        .collect();
    items.sort();
    items
}

fn last_name(path: &Value) -> String {
    Path::new(path.as_str().unwrap()).file_name().unwrap().to_string_lossy().into_owned()
}

fn pair(name: &str, outcome: &str) -> (String, String) {
    (name.to_owned(), outcome.to_owned())
}

mod negative {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;

    use fresher_core::Build;
    use fresher_testing::FakeClock;
    use serde_json::{Value, json};
    use tauri::ipc::InvokeBody;

    use super::{STARTED, begin, file_ending, launch, lines, record_scan, snapshot};
    use crate::dev_runs::run::Run;
    use crate::testing::{Shell, trial_home};

    #[test]
    fn a_read_command_is_not_recorded() {
        let (shell, folder) = (Shell::new(), tempfile::tempdir().unwrap());
        let runs = begin(&shell, folder.path(), folder.path());

        runs.call("get_findings", &InvokeBody::Json(json!({ "query": {} })));

        assert_eq!(lines(folder.path()), Vec::<Value>::new());
        assert_eq!(snapshot(folder.path())["actions"], json!([]));
    }

    #[test]
    fn a_raw_payload_is_recorded_as_null() {
        let (shell, folder) = (Shell::new(), tempfile::tempdir().unwrap());
        let runs = begin(&shell, folder.path(), folder.path());

        runs.call("start_scan", &InvokeBody::Raw(vec![1, 2, 3]));

        let expected = json!({ "at": "2025-11-20T10:04:31Z", "action": "start_scan", "args": null });
        assert_eq!(lines(folder.path()), [expected]);
    }

    #[test]
    fn scan_progress_is_never_written() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());

        record_scan(&shell, &runs);

        let text = fs::read_to_string(file_ending(folder.path(), ".events.jsonl")).unwrap();
        assert!(text.contains(r#""type":"scan-finished""#), "{text}");
        assert!(!text.contains(r#""type":"scan-progress""#), "{text}");
    }

    #[test]
    fn a_snapshot_that_cannot_be_written_is_logged_and_the_run_goes_on() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());
        let before = fs::read(file_ending(folder.path(), ".json")).unwrap();
        let lines_before = lines(folder.path()).len();
        fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o555)).unwrap();

        record_scan(&shell, &runs);

        let after = fs::read(file_ending(folder.path(), ".json")).unwrap();
        let lines_after = lines(folder.path()).len();
        let names = fs::read_dir(folder.path()).unwrap().count();
        fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(after, before);
        assert!(lines_after > lines_before, "{lines_before} then {lines_after}");
        assert_eq!(names, 2);
    }

    #[test]
    fn a_release_build_begins_no_run() {
        let (shell, folder) = (Shell::new(), tempfile::tempdir().unwrap());
        let clock = Arc::new(FakeClock::at(STARTED));

        let run = Run::begin(&launch(folder.path()), &Build::Release, folder.path(), None, clock, &shell.engine());

        assert!(run.unwrap().is_none());
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::fs;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use serde_json::json;
    use tauri::ipc::InvokeBody;

    use super::{
        begin, both_trial_categories, caches_only, last_name, lines, outcomes, pair, record_cleanup, record_scan,
        snapshot,
    };
    use crate::dev_runs::commands::{ACTIONS, READS};
    use crate::testing::{Shell, is_scan_finished, trial_home, until_finished};

    #[test]
    fn every_command_of_the_handler_is_an_action_or_a_read() {
        let source = include_str!("../lib.rs");
        let (_, after) = source.split_once("generate_handler![").unwrap();
        // The list ends at its own closing bracket; a `#[cfg(...)]` of a debug-only command holds one too.
        let (list, _) = after.split_once("\n    ]").unwrap();
        let handled: BTreeSet<&str> = list
            .split(',')
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(|path| path.rsplit("::").next().unwrap())
            .collect();
        let actions: BTreeSet<&str> = ACTIONS.into_iter().collect();
        let reads: BTreeSet<&str> = READS.into_iter().collect();

        assert_eq!(actions.intersection(&reads).count(), 0);
        assert_eq!(handled, actions.union(&reads).copied().collect());
    }

    #[test]
    fn a_run_begins_with_its_session_settings_and_access() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());

        let _runs = begin(&shell, folder.path(), tree.root());

        let stem = format!("2025-11-20_10-04-31_{}+26e4cb0_mac", env!("CARGO_PKG_VERSION"));
        let mut names: Vec<String> = fs::read_dir(folder.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, [format!("{stem}.events.jsonl"), format!("{stem}.json")]);
        let record = snapshot(folder.path());
        let home = tree.root().to_string_lossy().into_owned();
        assert_eq!(record["format"], 1);
        assert_eq!(record["session"]["started"], "2025-11-20T10:04:31Z");
        assert_eq!(record["session"]["build"]["name"], format!("{}+26e4cb0", env!("CARGO_PKG_VERSION")));
        assert_eq!(record["session"]["mode"], "mac");
        assert_eq!(record["session"]["macos"], "27.2");
        assert_eq!(record["session"]["home"], home.as_str());
        assert_eq!(record["session"]["roots"], json!([home.as_str()]));
        assert_eq!(record["session"]["fullDiskAccess"], "unknown");
        // The plain stored value, not the tagged one of the settings view.
        assert_eq!(record["settings"]["core.scan.roots"], json!([home.as_str()]));
        assert_ne!(record["access"], json!([]));
        assert_eq!(record["actions"], json!([]));
        assert_eq!(record["scans"], json!([]));
        assert_eq!(record["clear"], json!([]));
    }

    #[test]
    fn actions_are_recorded_with_their_arguments() {
        let (shell, folder) = (Shell::new(), tempfile::tempdir().unwrap());
        let runs = begin(&shell, folder.path(), folder.path());
        let args = json!({ "update": { "key": "stale-projects.unused-for-days",
                                       "value": { "kind": "number", "value": 30 } } });

        runs.call("update_setting", &InvokeBody::Json(args.clone()));

        let expected = json!({ "at": "2025-11-20T10:04:31Z", "action": "update_setting", "args": args });
        assert_eq!(lines(folder.path()).last(), Some(&expected));
        // The snapshot holds it from its next rewrite, at the end of a scan.
        record_scan(&shell, &runs);
        let action = &snapshot(folder.path())["actions"][0];
        assert_eq!(action, &json!({ "at": "2025-11-20T10:04:31Z", "command": "update_setting", "args": args }));
    }

    #[test]
    fn an_action_does_not_wait_for_a_snapshot_being_written() {
        let (shell, folder) = (Shell::new(), tempfile::tempdir().unwrap());
        let runs = begin(&shell, folder.path(), folder.path());
        let engine = shell.engine();
        let events = engine.subscribe();
        engine.start_scan(fresher_engine::ScanMode::Full).unwrap();
        let mut scan = until_finished(&events, is_scan_finished);
        let finished = scan.pop().unwrap();
        for event in &scan {
            runs.event(event, &engine);
        }
        let before = lines(folder.path()).len();
        let (runs, engine, finished) = (&runs, &engine, &finished);

        thread::scope(|scope| {
            // Declared inside the scope, so a failed assertion releases it before the scope joins its threads.
            let writing = crate::dev_runs::lock(&runs.writer);
            let (snapshot_sender, snapshot_written) = mpsc::channel();
            scope.spawn(move || {
                runs.event(finished, engine);
                snapshot_sender.send(()).unwrap();
            });
            // The scan's end is appended under the run's lock; the snapshot then waits for the writer held above.
            let deadline = Instant::now() + Duration::from_secs(10);
            while lines(folder.path()).len() == before {
                assert!(Instant::now() < deadline, "the scan's end was never folded");
                thread::yield_now();
            }
            let (action_sender, action_recorded) = mpsc::channel();
            scope.spawn(move || {
                runs.call("start_scan", &InvokeBody::Json(json!({})));
                action_sender.send(()).unwrap();
            });

            action_recorded.recv_timeout(Duration::from_secs(10)).unwrap();
            assert!(snapshot_written.try_recv().is_err(), "the snapshot was written while its writer was held");
            drop(writing);
            snapshot_written.recv_timeout(Duration::from_secs(10)).unwrap();
        });
        assert_eq!(lines(folder.path()).len(), before + 2);
    }

    #[test]
    fn a_scan_is_recorded_with_its_findings_texts_and_left_outs() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());

        record_scan(&shell, &runs);

        let record = snapshot(folder.path());
        let scan = &record["scans"][0];
        assert_eq!(scan["kind"], json!({ "kind": "full" }));
        assert_eq!(scan["outcome"], json!({ "kind": "completed" }));
        assert!(scan["durationMs"].is_u64(), "{scan}");
        assert!(scan["finished"].is_string(), "{scan}");
        let trial = &scan["modules"]["trial"];
        assert_eq!(trial["group"], "debug");
        let findings = trial["findings"].as_array().unwrap();
        let mut names: Vec<String> = findings.iter().map(|finding| last_name(&finding["subject"]["value"])).collect();
        names.sort();
        assert_eq!(names, ["fresher-trial-cache", "fresher-trial-kept"]);
        for finding in findings {
            assert_ne!(finding["titleText"].as_str().unwrap(), "", "{finding}");
            assert_ne!(finding["reasonText"].as_str().unwrap(), "", "{finding}");
        }
        let left_out = scan["modules"]["demo"]["leftOut"].as_array().unwrap();
        assert_eq!(left_out.len(), 1, "{left_out:?}");
        assert_ne!(left_out[0]["reasonText"].as_str().unwrap(), "", "{left_out:?}");
    }

    #[test]
    fn a_dry_run_cleanup_is_recorded_by_group_as_would_move_and_would_empty() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());
        record_scan(&shell, &runs);
        assert_eq!(snapshot(folder.path())["clear"], json!([]));

        record_cleanup(&shell, &runs, &both_trial_categories(), "trash");

        let record = snapshot(folder.path());
        let clear = &record["clear"][0];
        assert_eq!(clear["scan"], 1);
        assert_eq!(clear["method"], "trash");
        assert_eq!(clear["dryRun"], true);
        assert_eq!(clear["ended"], "completed");
        assert_eq!(
            outcomes(&record, "debug"),
            [pair("fresher-trial-cache", "would-move"), pair("fresher-trial-kept", "would-empty")]
        );
        assert_eq!(clear["groups"]["debug"][0]["module"], "trial");
        assert!(tree.root().join("sandbox/fresher-trial-cache/blob.bin").exists());
    }

    #[test]
    fn a_permanent_dry_run_is_recorded_as_would_delete() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::dry_run(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());
        record_scan(&shell, &runs);

        record_cleanup(&shell, &runs, &caches_only(), "permanent");

        let record = snapshot(folder.path());
        assert_eq!(record["clear"][0]["method"], "permanent");
        assert_eq!(outcomes(&record, "debug"), [pair("fresher-trial-cache", "would-delete")]);
        assert!(tree.root().join("sandbox/fresher-trial-cache/blob.bin").exists());
    }

    #[test]
    fn a_sandbox_cleanup_is_recorded_as_moved() {
        let (tree, folder) = (trial_home(true), tempfile::tempdir().unwrap());
        let shell = Shell::cleaning(tree.root());
        let runs = begin(&shell, folder.path(), tree.root());
        record_scan(&shell, &runs);

        record_cleanup(&shell, &runs, &caches_only(), "trash");

        let record = snapshot(folder.path());
        assert_eq!(outcomes(&record, "debug"), [pair("fresher-trial-cache", "moved")]);
        assert_eq!(record["clear"][0]["dryRun"], false);
        assert_eq!(record["scans"][0]["removed"].as_array().map(Vec::len), Some(1));
    }
}
