//! The jetbrains module end to end: the real scanner on fixture homes, the `jetbrains` module, the real settings file,
//! scan cache and journal in a temporary data folder, and the real `DiskCleaner` in the fixture's sandbox
//! (`Reach::Sandbox`) with a fake Trash. The installed and the open apps are fakes. Each flow is a function that
//! asserts and gives its report; the reports of all flows are the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test jetbrains` rewrites `fixtures/golden/jetbrains-flow.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "jetbrains_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use fresher_core::{
        DangerLevel, Event, HiddenBy, IgnoreTarget, IgnoredEntry, MeaningGroup, ModuleGroup, PartKind, ScanOutcome,
    };
    use fresher_testing::{FakeInstalledApps, TreeReport};
    use serde_json::{Value, json};

    use crate::flow::{Apps, Flow, app, because, contents, current_apps, left_out, open, select_all};

    const PLACES: [&str; 3] = ["jetbrains.caches", "jetbrains.logs", "jetbrains.settings"];
    const SETTINGS: &str = "home/Library/Application Support/JetBrains";
    const CACHES: &str = "home/Library/Caches/JetBrains";
    const LOGS: &str = "home/Library/Logs/JetBrains";
    const UNINSTALLED: &str = "jetbrains.uninstalled";

    /// The findings of the 2024.3 folders of `products` in the three places, sorted.
    fn older(products: &[&str]) -> Vec<(String, String)> {
        versions(products, "2024.3")
    }

    fn versions(products: &[&str], version: &str) -> Vec<(String, String)> {
        let mut found = Vec::new();
        for product in products {
            for (category, folder) in
                [("jetbrains.settings", SETTINGS), ("jetbrains.caches", CACHES), ("jetbrains.logs", LOGS)]
            {
                found.push((category.to_owned(), format!("{folder}/{product}{version}")));
            }
        }
        found.sort();
        found
    }

    /// The three folders of the removed `DataGrip` 2024.3 in `jetbrains-uninstalled`, as findings of their category.
    fn removed() -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> = [SETTINGS, CACHES, LOGS]
            .map(|folder| (UNINSTALLED.to_owned(), format!("{}/DataGrip2024.3", folder.trim_start_matches("home/"))))
            .into();
        found.sort();
        found
    }

    fn completed(events: &[Event]) -> bool {
        matches!(events.last(), Some(Event::ScanFinished { outcome: ScanOutcome::Completed, .. }))
    }

    /// The home a removed `DataGrip` leaves behind, scanned with nothing installed.
    fn removed_home(installed: FakeInstalledApps) -> Flow {
        Flow::new("jetbrains-uninstalled", Apps::with(installed))
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.fixture).unwrap()).unwrap()
    }

    fn explained(flow: &Flow) -> u64 {
        let map = flow.engine.storage_map().unwrap();
        let part = map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained).unwrap();
        part.sizes.logical.bytes()
    }

    fn hidden(flow: &Flow) -> Vec<(String, HiddenBy)> {
        let overview = flow.engine.overview().unwrap();
        overview.hidden_modules.iter().map(|one| (one.module.as_str().to_owned(), one.by)).collect()
    }

    /// Flow 6: the installed apps cannot be listed.
    pub(crate) fn failing_system() -> Value {
        let flow = Flow::new("jetbrains-flow", Apps::with(FakeInstalledApps::failing("no apps folder")));

        let events = flow.scan();

        assert_eq!(flow.findings(), []);
        let kept = left_out(&events);
        assert_eq!(because(&kept, "jetbrains.left-out.no-system"), PLACES.map(|place| (place, 4)));
        assert_eq!(kept.len(), 3, "{kept:?}");
        assert!(completed(&events), "{:?}", events.last());
        json!({ "findings": flow.findings(), "leftOut": kept })
    }

    /// Flow 2, first half: an open IDE is left out; the others are offered.
    pub(crate) fn open_ide() -> (Flow, Value) {
        let flow = Flow::new(
            "jetbrains-flow",
            Apps::installed(current_apps()).running(vec![open("GoLand", "com.jetbrains.goland")]),
        );

        let events = flow.scan();

        assert_eq!(flow.findings(), older(&["WebStorm"]));
        let kept = left_out(&events);
        assert_eq!(because(&kept, "jetbrains.left-out.running"), PLACES.map(|place| (place, 2)));
        let report = json!({ "findings": flow.findings(), "leftOut": kept });
        (flow, report)
    }

    /// Flow 2, second half: the IDE quits and a rescan offers its older versions.
    pub(crate) fn quit_and_rescan(mut flow: Flow) -> Value {
        flow.restart(Apps::installed(current_apps()));

        let events = flow.scan();

        assert_eq!(flow.findings(), older(&["GoLand", "WebStorm"]));
        let kept = left_out(&events);
        assert_eq!(because(&kept, "jetbrains.left-out.running"), []);
        json!({ "findings": flow.findings(), "leftOut": kept })
    }

    /// Flow 3: `WebStorm` is updated between two scans.
    pub(crate) fn update_between_scans() -> Value {
        let mut flow = Flow::new("jetbrains-flow", Apps::installed(current_apps()));
        flow.scan();
        assert_eq!(flow.findings(), older(&["GoLand", "WebStorm"]));

        let updated = vec![app("com.jetbrains.WebStorm", "WebStorm", "2026.1.1"), current_apps().remove(1)];
        flow.restart(Apps::installed(updated));
        let events = flow.scan();

        let mut expected = older(&["GoLand", "WebStorm"]);
        expected.extend(versions(&["WebStorm"], "2025.3"));
        expected.sort();
        assert_eq!(flow.findings(), expected);
        json!({ "findings": flow.findings(), "leftOut": left_out(&events) })
    }

    /// Flow 1: scan, review, confirm to the Trash, put back.
    pub(crate) fn clean_and_put_back() -> Value {
        let flow = Flow::new("jetbrains-flow", Apps::installed(current_apps()));
        flow.scan();
        let older_folders = [SETTINGS, CACHES, LOGS]
            .map(|base| ["WebStorm", "GoLand"].map(|product| flow.path(&format!("{base}/{product}2024.3"))));
        let before: Vec<_> = older_folders.iter().flatten().map(|folder| contents(folder)).collect();
        assert!(before.iter().all(|files| files.values().all(|bytes| !bytes.is_empty())));

        let view = flow.engine.prepare_cleanup(&select_all()).unwrap();

        assert_eq!(view.summary.count, 6);
        assert_eq!(view.summary.highest, DangerLevel::Medium);
        let (cleanup, events) = flow.clean(&view);
        assert!(older_folders.iter().flatten().all(|folder| !folder.exists()));
        assert!(flow.path(&format!("{SETTINGS}/WebStorm2025.3")).is_dir(), "the version in use stays");
        assert!(flow.path(&format!("{CACHES}/GoLand2025.2")).is_dir(), "the version in use stays");
        assert_eq!(flow.findings(), []);
        assert_eq!(flow.engine.cleanups(0, 10).total, 1);
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        let after: Vec<_> = older_folders.iter().flatten().map(|folder| contents(folder)).collect();
        assert_eq!(after, before, "put back byte for byte");
        let report = json!({
            "plan": view,
            "events": events,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": tree(&flow),
        });
        flow.relative(&report)
    }

    /// Flow 4: ignoring the module, then the group, hides everything; restoring brings it back without a rescan.
    pub(crate) fn ignoring() -> Value {
        let flow = Flow::new("jetbrains-flow", Apps::installed(current_apps()));
        flow.scan();
        let before = (flow.findings(), explained(&flow));
        assert_eq!(before.0, older(&["GoLand", "WebStorm"]));

        flow.engine.ignore(&IgnoreTarget::Module { module: "jetbrains".into() }).unwrap();

        assert_eq!(flow.findings(), []);
        assert_eq!(flow.engine.overview().unwrap().totals.len(), 0);
        assert_eq!(hidden(&flow), [("jetbrains".to_owned(), HiddenBy::Module)]);
        assert!(explained(&flow) < before.1, "the storage map still explains what is ignored");
        flow.engine.ignore(&IgnoreTarget::Group { group: ModuleGroup::Development }).unwrap();
        assert_eq!(flow.findings(), []);
        assert_eq!(hidden(&flow), [("jetbrains".to_owned(), HiddenBy::Group)]);
        assert!(
            flow.engine
                .prepare_cleanup(&select_all())
                .is_err_and(|error| { error == fresher_core::CommandError::NothingToClean })
        );

        flow.engine.restore_ignored(&IgnoredEntry::Group { group: ModuleGroup::Development }).unwrap();
        assert_eq!(hidden(&flow), [("jetbrains".to_owned(), HiddenBy::Module)]);
        flow.engine.restore_ignored(&IgnoredEntry::Module { module: "jetbrains".into() }).unwrap();

        assert_eq!((flow.findings(), explained(&flow)), before);
        assert_eq!(hidden(&flow), []);
        json!({ "findings": flow.findings() })
    }

    /// Flow 5: the storage map counts the offered folders in `WebStorm`'s line, by category.
    ///
    /// The map names an app's data by folders called after the app's id; the folders of a version in use are called
    /// `WebStorm2025.3`, so they are not in the line's data: they are the user's folders, never offered.
    pub(crate) fn storage_map() -> Value {
        let flow = Flow::new("jetbrains-flow", Apps::installed(current_apps()));
        flow.scan();

        let groups = flow.engine.storage_meaning(0).unwrap();

        let (sizes, categories, data) = groups
            .iter()
            .find_map(|group| match group {
                MeaningGroup::App { app, sizes, categories, data } if app.id.as_str() == "com.jetbrains.WebStorm" => {
                    Some((sizes, categories, data))
                }
                _ => None,
            })
            .expect("WebStorm has a line");
        let mut kept: Vec<(&str, u64)> =
            categories.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        kept.sort_unstable();
        assert_eq!(
            kept,
            [
                ("jetbrains.caches", 65_536 + 4096),
                ("jetbrains.logs", 8192 + 4096),
                ("jetbrains.settings", 2048 + 16_384)
            ]
        );
        assert_eq!(sizes.logical.bytes(), kept.iter().map(|(_, bytes)| bytes).sum::<u64>());
        assert_eq!(data, &[], "the version in use is not WebStorm's data folder by name");
        json!({ "webstormLine": kept })
    }

    /// Spotlight finds the removed IDE elsewhere: its data is kept, where it is offered otherwise.
    pub(crate) fn installed_elsewhere() -> Value {
        let offered = removed_home(FakeInstalledApps::default());
        offered.scan();
        assert_eq!(offered.findings(), removed());

        let mut far = app("com.jetbrains.datagrip", "DataGrip", "2025.3");
        far.path = "/Volumes/Drive/DataGrip.app".into();
        let flow = removed_home(FakeInstalledApps::default().with_elsewhere(vec![far]));

        let events = flow.scan();

        assert_eq!(flow.findings(), []);
        let kept = left_out(&events);
        assert_eq!(because(&kept, "jetbrains.left-out.installed-elsewhere"), [(UNINSTALLED, 3)]);
        json!({ "findings": flow.findings(), "leftOut": kept })
    }

    /// A restored scan reads no folder, so it knows no folder's age: it never offers a removed IDE's data.
    pub(crate) fn restored_scan() -> Value {
        let mut flow = removed_home(FakeInstalledApps::default());
        flow.scan();
        assert_eq!(flow.findings(), removed());

        flow.restart(Apps::installed(vec![]));
        let events = flow.restore();

        assert_eq!(flow.findings(), []);
        let kept = left_out(&events);
        assert_eq!(because(&kept, "jetbrains.left-out.used-recently"), [(UNINSTALLED, 3)]);
        json!({ "findings": flow.findings(), "leftOut": kept })
    }
}

mod negative {
    use crate::flows::{failing_system, installed_elsewhere, open_ide, restored_scan};

    #[test]
    fn apps_that_cannot_be_listed_leave_every_folder_out_and_the_scan_complete() {
        failing_system();
    }

    #[test]
    fn an_open_ide_is_left_out_and_the_others_are_offered() {
        open_ide();
    }

    #[test]
    fn a_removed_ide_spotlight_finds_elsewhere_keeps_its_data() {
        installed_elsewhere();
    }

    #[test]
    fn a_restored_scan_never_offers_a_removed_ides_data() {
        restored_scan();
    }
}

mod positive {
    use crate::flows::{
        clean_and_put_back, failing_system, ignoring, installed_elsewhere, open_ide, quit_and_rescan, restored_scan,
        storage_map, update_between_scans,
    };
    use crate::support::check_golden;

    #[test]
    fn an_ide_that_quit_has_its_older_versions_offered_after_a_rescan() {
        let (flow, _) = open_ide();

        quit_and_rescan(flow);
    }

    #[test]
    fn an_update_between_scans_offers_the_version_it_replaced() {
        update_between_scans();
    }

    #[test]
    fn the_older_versions_go_to_the_trash_and_come_back_byte_for_byte() {
        clean_and_put_back();
    }

    #[test]
    fn ignoring_the_module_and_the_group_hides_everything_and_restoring_brings_it_back() {
        ignoring();
    }

    #[test]
    fn the_storage_map_counts_the_offered_folders_in_the_ides_line() {
        storage_map();
    }

    #[test]
    fn the_reports_of_the_flows_match_the_golden_file() {
        let (open, first) = open_ide();
        let report = serde_json::json!({
            "failingSystem": failing_system(),
            "openIde": first,
            "quitAndRescan": quit_and_rescan(open),
            "updateBetweenScans": update_between_scans(),
            "cleanAndPutBack": clean_and_put_back(),
            "ignoring": ignoring(),
            "storageMap": storage_map(),
            "installedElsewhere": installed_elsewhere(),
            "restoredScan": restored_scan(),
        });

        check_golden("jetbrains-flow", &report);
    }
}
