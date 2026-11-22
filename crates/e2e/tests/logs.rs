//! The logs module end to end: a fixture sandbox's home with old, fresh and ten-day-old logs, a state file and a crash
//! report under `Library/Logs`, beside a removed app's leftovers (`app-leftovers`' to offer) and an older IDE's
//! logs (`jetbrains`' to offer); the real scanner (its reader seen through the facts lens), the modules the app
//! registers for `Library/Logs`, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching
//! only inside the sandbox, with a fake Trash. The files held open are a fake a flow sets. The first scan's report is
//! the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test logs` rewrites `fixtures/golden/logs-flow.json`; the diff
//! is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "logs_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    use fresher_core::{
        DangerLevel, Event, FindingOrder, FindingsQuery, ItemOutcome, PutBackOutcome, ScanKind, SkipReason,
    };
    use serde_json::{Value, json};

    use crate::flow::{APP_LOGS, CRASH_REPORTS, Flow, left_out, select_logs};
    use crate::tree::contents;

    const UPDATER: &str = "home/Library/Logs/Vendor/Updater/updater.log";
    const TEN_DAYS: &str = "home/Library/Logs/notes-10d.log";
    /// The files the module offers at the default seven days, as category and path.
    const OURS: [(&str, &str); 4] = [
        (APP_LOGS, "home/Library/Logs/Vendor/Updater/updater.log"),
        (APP_LOGS, "home/Library/Logs/Writer/writer.1.log"),
        (APP_LOGS, TEN_DAYS),
        (CRASH_REPORTS, "home/Library/Logs/DiagnosticReports/Writer-2026-10-01-101010.ips"),
    ];
    /// The fresh log beside the old one.
    const FRESH: &str = "home/Library/Logs/Writer/writer.log";
    /// The vendor's state file, no one's.
    const STATE: &str = "home/Library/Logs/Vendor/Updater/state.plist";
    /// The folders of the offered files, which stay.
    const FOLDERS: [&str; 3] =
        ["home/Library/Logs/Writer", "home/Library/Logs/Vendor/Updater", "home/Library/Logs/DiagnosticReports"];
    /// The removed app's leftovers, `app-leftovers`' to offer.
    const GONE: [(&str, &str); 3] = [
        ("app-leftovers.caches", "home/Library/Caches/com.example.Gone"),
        ("app-leftovers.caches", "home/Library/Logs/com.example.Gone"),
        ("app-leftovers.caches", "home/Library/Saved Application State/com.example.Gone.savedState"),
    ];
    /// `WebStorm`'s older version's logs, `jetbrains`' to offer.
    const WEBSTORM: (&str, &str) = ("jetbrains.logs", "home/Library/Logs/JetBrains/WebStorm2024.3");

    fn found(cases: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            cases.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    fn ours() -> Vec<(String, String)> {
        found(&OURS)
    }

    fn ours_paths() -> Vec<&'static str> {
        OURS.iter().map(|(_, path)| *path).collect()
    }

    /// The findings of `module` only.
    fn of_module(flow: &Flow, module: &str) -> Vec<(String, String)> {
        let prefix = format!("{module}.");
        flow.findings().into_iter().filter(|(category, _)| category.starts_with(&prefix)).collect()
    }

    fn keep(category: &str, reason: &str, items: u32) -> (String, String, u32) {
        (category.to_owned(), format!("logs.left-out.{reason}"), items)
    }

    fn bytes_of(flow: &Flow, paths: &[&str]) -> Vec<Vec<u8>> {
        paths.iter().map(|path| fs::read(flow.path(path)).unwrap()).collect()
    }

    /// The bytes of every file under the fake Trash, sorted: what went there, whatever the numbering of its names.
    fn trashed_bytes(flow: &Flow) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = contents(&flow.sandbox.trash()).into_values().collect();
        bytes.sort();
        bytes
    }

    fn sorted(mut bytes: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        bytes.sort();
        bytes
    }

    /// The paths the journal's item records moved, relative to the sandbox, sorted.
    fn journaled(flow: &Flow) -> Vec<String> {
        let mut paths: Vec<String> = flow
            .journal()
            .iter()
            .filter(|record| record["type"] == "item")
            .flat_map(|record| record["trashed"].as_array().cloned().unwrap_or_default())
            .filter_map(|trashed| trashed["original"].as_str().map(|path| flow.relative_path(path.as_ref())))
            .collect();
        paths.sort();
        paths
    }

    /// The module's findings as the engine answers them (titles and reasons included), relative to the sandbox, by
    /// identity, without their ids (the order the walk found them in is not the report's).
    fn logs_findings(flow: &Flow) -> Value {
        let page = flow.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let mut ours: Vec<Value> = page
            .findings
            .iter()
            .filter(|finding| finding.finding().category().as_str().starts_with("logs."))
            .map(|finding| {
                let mut value = serde_json::to_value(finding).unwrap();
                value.as_object_mut().unwrap().remove("id");
                value
            })
            .collect();
        ours.sort_by_key(|value| value["key"]["identity"].as_str().unwrap_or_default().to_owned());
        flow.relative(&json!(ours))
    }

    /// The first scan's report: the three modules' findings and left-outs, and the module's findings as the engine
    /// answers them.
    fn first_scan(flow: &Flow, events: &[Event]) -> Value {
        let report = json!({
            "offered": flow.findings(),
            "leftOut": {
                "jetbrains": left_out(events, "jetbrains"),
                "app-leftovers": left_out(events, "app-leftovers"),
                "logs": left_out(events, "logs"),
            },
            "logsFindings": logs_findings(flow),
        });
        flow.relative(&report)
    }

    /// Flow 1: the three modules on one home: no path is offered twice; no `logs` finding lies inside the removed
    /// app's folder or `JetBrains`; those folders are their modules'; the state file is no one's.
    pub(crate) fn no_path_is_offered_by_two_modules() {
        let flow = Flow::new();

        flow.scan();

        let all = flow.findings();
        let paths: BTreeSet<&String> = all.iter().map(|(_, path)| path).collect();
        assert_eq!(paths.len(), all.len(), "a path is offered once: {all:?}");
        let logs = of_module(&flow, "logs");
        assert_eq!(logs, ours());
        assert!(
            logs.iter().all(|(_, path)| !path.starts_with("home/Library/Logs/com.example.Gone")
                && !path.starts_with("home/Library/Logs/JetBrains")),
            "{logs:?}"
        );
        assert_eq!(of_module(&flow, "app-leftovers"), found(&GONE));
        assert_eq!(of_module(&flow, "jetbrains"), found(&[WEBSTORM]));
        assert!(all.iter().all(|(_, path)| path != STATE && path != FRESH), "{all:?}");
    }

    /// Flow 2: the fresh log is left out `recent`, and a confirmed cleanup of everything offered leaves it byte for
    /// byte.
    pub(crate) fn a_fresh_log_stays() {
        let flow = Flow::new();
        let events = flow.scan();
        assert_eq!(left_out(&events, "logs"), [keep(APP_LOGS, "recent", 1)]);
        let before = bytes_of(&flow, &[FRESH, STATE]);
        let categories: BTreeSet<String> = flow.findings().into_iter().map(|(category, _)| category).collect();
        let selection = categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
        let selection = serde_json::from_value(json!({ "findings": [], "categories": Value::Array(selection) }));
        let view = flow.engine.prepare_cleanup(&selection.unwrap()).unwrap();

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        assert_eq!(outcomes.len(), OURS.len() + GONE.len() + 1);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert_eq!(bytes_of(&flow, &[FRESH, STATE]), before, "the fresh log and the state file stay");
    }

    /// Flow 3: a log held open when the cleanup acts is skipped in use and stays; the other files move.
    pub(crate) fn a_log_held_open_is_skipped() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select_logs()).unwrap();
        let before = bytes_of(&flow, &[UPDATER]);
        flow.open_files.hold(flow.path(UPDATER), "Updater");

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let in_use = ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Updater".into() } };
        let mut expected: Vec<(String, ItemOutcome)> = ours()
            .into_iter()
            .map(|(_, path)| {
                let outcome = if path == UPDATER { in_use.clone() } else { ItemOutcome::Moved };
                (path, outcome)
            })
            .collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(outcomes, expected);
        assert_eq!(bytes_of(&flow, &[UPDATER]), before, "the file held open stays");
        let moved: Vec<&str> = ours_paths().into_iter().filter(|path| *path != UPDATER).collect();
        assert!(moved.iter().all(|path| !flow.path(path).exists()), "the other files moved");
        assert_eq!(contents(&flow.sandbox.trash()).len(), moved.len());
    }

    /// Flow 4: a restart restores the scan: every match of the module is left out `scan-again`, nothing of it is
    /// offered; a scan offers them again.
    pub(crate) fn a_restored_scan_leaves_every_log_out() {
        let mut flow = Flow::new();
        flow.scan();
        flow.restart();

        let restored = flow.restore();

        let kind = restored.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        });
        assert!(matches!(kind, Some(ScanKind::Restored { .. })), "{kind:?}");
        assert_eq!(of_module(&flow, "logs"), []);
        assert_eq!(
            left_out(&restored, "logs"),
            [keep(APP_LOGS, "scan-again", 4), keep(CRASH_REPORTS, "scan-again", 1)]
        );

        let events = flow.scan();

        assert_eq!(of_module(&flow, "logs"), ours());
        assert_eq!(left_out(&events, "logs"), [keep(APP_LOGS, "recent", 1)]);
    }

    /// Flow 5: fourteen days in the settings file keep the log ten days old as recent; at the default it is offered.
    pub(crate) fn a_longer_setting_keeps_the_ten_day_old_log() {
        let flow = Flow::new();
        flow.scan();
        assert!(of_module(&flow, "logs").iter().any(|(_, path)| path == TEN_DAYS));
        flow.set_days(14);

        let events = flow.scan();

        let without: Vec<(String, String)> = ours().into_iter().filter(|(_, path)| path != TEN_DAYS).collect();
        assert_eq!(of_module(&flow, "logs"), without);
        assert_eq!(left_out(&events, "logs"), [keep(APP_LOGS, "recent", 2)]);
        let settings = fs::read_to_string(flow.sandbox.data_folder().join("settings.json")).unwrap();
        assert!(settings.contains("logs.older-than-days"), "{settings}");
    }

    /// Flow 6: a confirmed cleanup of the module's findings moves each file to the Trash and keeps its folder; the
    /// journal names each file; the fresh log, the state file and the other modules' files stay. Gives the first
    /// scan's report.
    pub(crate) fn clean_to_the_trash() -> Value {
        let flow = Flow::new();
        let events = flow.scan();
        assert_eq!(of_module(&flow, "logs"), ours());
        assert_eq!(left_out(&events, "logs"), [keep(APP_LOGS, "recent", 1)]);
        let report = first_scan(&flow, &events);
        let paths = ours_paths();
        let before = sorted(bytes_of(&flow, &paths));
        let kept_paths = [FRESH, STATE, "home/Library/Logs/com.example.Gone/app.log"];
        let kept = bytes_of(&flow, &kept_paths);

        let view = flow.engine.prepare_cleanup(&select_logs()).unwrap();

        assert_eq!((view.summary.count, view.summary.highest), (4, DangerLevel::Safe));
        let (_, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let mut expected: Vec<(String, ItemOutcome)> =
            ours().into_iter().map(|(_, path)| (path, ItemOutcome::Moved)).collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(outcomes, expected);
        assert!(paths.iter().all(|path| !flow.path(path).exists()));
        assert!(FOLDERS.iter().all(|folder| flow.path(folder).is_dir()), "the folders stay");
        assert_eq!(trashed_bytes(&flow), before, "every file is in the Trash");
        let mut names: Vec<String> = paths.iter().map(|path| (*path).to_owned()).collect();
        names.sort();
        assert_eq!(journaled(&flow), names);
        assert_eq!(bytes_of(&flow, &kept_paths), kept, "no other file is touched");
        assert!(flow.path(WEBSTORM.1).join("idea.log").is_file());
        report
    }

    /// Flow 7: put back returns every file to its place byte for byte and empties the Trash.
    pub(crate) fn put_back_restores_every_log() {
        let flow = Flow::new();
        flow.scan();
        let paths = ours_paths();
        let before: BTreeMap<PathBuf, Vec<u8>> =
            paths.iter().map(|path| (flow.path(path), fs::read(flow.path(path)).unwrap())).collect();
        let view = flow.engine.prepare_cleanup(&select_logs()).unwrap();
        let (cleanup, _) = flow.clean(&view);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), paths.len());
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after: BTreeMap<PathBuf, Vec<u8>> =
            paths.iter().map(|path| (flow.path(path), fs::read(flow.path(path)).unwrap())).collect();
        assert_eq!(after, before, "put back byte for byte");
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 0);
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn no_path_is_offered_by_two_modules() {
        flows::no_path_is_offered_by_two_modules();
    }

    #[test]
    fn a_fresh_log_is_left_out_and_stays_after_the_cleanup() {
        flows::a_fresh_log_stays();
    }

    #[test]
    fn a_log_held_open_is_skipped_and_moves_nothing() {
        flows::a_log_held_open_is_skipped();
    }

    #[test]
    fn a_restored_scan_leaves_every_log_out_until_a_rescan() {
        flows::a_restored_scan_leaves_every_log_out();
    }

    #[test]
    fn a_longer_setting_keeps_the_ten_day_old_log() {
        flows::a_longer_setting_keeps_the_ten_day_old_log();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn a_confirmed_cleanup_moves_the_old_logs_to_the_trash_and_keeps_their_folders() {
        check_golden("logs-flow", &flows::clean_to_the_trash());
    }

    #[test]
    fn put_back_restores_every_log_byte_for_byte() {
        flows::put_back_restores_every_log();
    }
}
