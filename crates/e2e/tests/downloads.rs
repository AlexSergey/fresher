//! The downloads module end to end: a fixture sandbox's home with `Downloads` holding what the module offers in
//! each category and what it keeps for each reason, the user's own folder `Project` with a `node_modules` inside, and
//! a file outside Downloads; the real scanner (its reader seen through the facts lens), the modules the app
//! registers, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the
//! sandbox, with a fake Trash. The clock, the installed and open apps, the receipts, the system's permissions and the
//! change journal's answers are fakes a flow sets. The flows of the design's section 4, "Core e2e", and one for Full
//! Disk Access; flow 2's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test downloads` rewrites
//! `fixtures/golden/downloads-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "downloads_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    use fresher_core::{
        Access, AccessStatus, ByteSize, Coverage, DangerLevel, Descent, Event, IgnoreTarget, IgnoredEntry, ItemOutcome,
        MeaningGroup, ModuleGroup, ModuleOutcome, NotEntered, PlaceView, PutBackOutcome,
    };
    use fresher_testing::{FakeInstalledApps, FakeRunningApps, running_app};
    use serde_json::{Value, json};

    use crate::flow::{DAY, DOWNLOADS, Example, Home, MODULE, Setup, installed_apps, left_out, outcome};
    use crate::tree::contents;

    const UNFINISHED: &str = "downloads.unfinished";
    const SAFARI: &str = "downloads.safari-resumable";
    const INSTALLERS: &str = "downloads.installers";
    const MACOS: &str = "downloads.macos-installers";
    const LARGE: &str = "downloads.large-untouched";
    const FIREFOX: &str = "org.mozilla.firefox";
    const EXAMPLE: &str = "com.example.Example";

    /// What the first scan after the opt-in offers, in the order `Flow::findings` sorts them: the installer of
    /// Firefox and the one a receipt proves, the big file, the macOS installer, Safari's bundle, Chrome's file and
    /// Firefox's `.part` with its empty placeholder.
    const OFFERED: [(&str, &str); 8] = [
        (INSTALLERS, "home/Downloads/Firefox 133.0.dmg"),
        (INSTALLERS, "home/Downloads/Tool-3.1.pkg"),
        (LARGE, "home/Downloads/video.mov"),
        (MACOS, "home/Downloads/Install macOS Sonoma.app"),
        (SAFARI, "home/Downloads/trip.zip.download"),
        (UNFINISHED, "home/Downloads/movie.mp4.crdownload"),
        (UNFINISHED, "home/Downloads/video.mkv"),
        (UNFINISHED, "home/Downloads/video.mkv.part"),
    ];
    /// What is kept in the scan: Opera's file within the days, and the installer of the `Example` not yet installed.
    const KEPT: [&str; 2] = ["home/Downloads/Draft.iso.opdownload", "home/Downloads/Example-2.0.dmg"];
    /// What is the user's and never touched: the folder with its `node_modules`, and a file outside Downloads.
    const THE_USERS: [&str; 2] = ["home/Downloads/Project", "home/Documents/keep.txt"];
    fn found(offered: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            offered.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    fn everything() -> Vec<(String, String)> {
        found(&OFFERED)
    }

    /// The findings of the module only.
    fn ours(home: &Home) -> Vec<(String, String)> {
        home.flow.findings().into_iter().filter(|(category, _)| category.starts_with("downloads.")).collect()
    }

    fn access_changes(events: &[Event]) -> Vec<&AccessStatus> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::AccessChanged { status } => Some(status),
                _ => None,
            })
            .collect()
    }

    fn coverage(events: &[Event]) -> Option<&Coverage> {
        events.iter().find_map(|event| match event {
            Event::ScanCoverage { coverage, .. } => Some(coverage),
            _ => None,
        })
    }

    fn position(events: &[Event], is: impl Fn(&Event) -> bool) -> usize {
        events.iter().position(is).expect("the event is there")
    }

    fn module_finished(event: &Event) -> bool {
        matches!(event, Event::ModuleFinished { module, .. } if module.as_str() == MODULE)
    }

    fn contents_of(home: &Home, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&home.flow.path(path))).collect()
    }

    fn total_bytes(home: &Home, paths: &[&str]) -> u64 {
        contents_of(home, paths).iter().flat_map(BTreeMap::values).map(|bytes| bytes.len() as u64).sum()
    }

    /// The categories of the overview's totals, sorted.
    fn totals(home: &Home) -> Vec<String> {
        let mut totals: Vec<String> = home
            .flow
            .engine
            .overview()
            .unwrap()
            .totals
            .iter()
            .map(|total| total.category.as_str().to_owned())
            .collect();
        totals.sort();
        totals
    }

    /// The place `names` below the scan root, each step a folder the user opened.
    fn place(home: &Home, names: &[&str]) -> PlaceView {
        let engine = &home.flow.engine;
        let root = engine.storage_map().unwrap().volumes[0].roots()[0];
        let mut view = engine.storage_place(root, Descent::Stay, 0, 100).unwrap();
        for name in names {
            let row = view.rows.iter().find(|row| row.node.name == *name).expect("the place has the row");
            view = engine.storage_place(row.node.id, Descent::Stay, 0, 100).unwrap();
        }
        view
    }

    /// Flow 1: Downloads not asked and the switch off: the module is unavailable for the folder, nothing is tried or
    /// read in it, and the coverage names the folder not entered.
    pub(crate) fn nothing_is_read_in_downloads_before_the_opt_in() {
        let home = Home::new();

        let events = home.flow.scan();

        assert_eq!(outcome(&events), Some(ModuleOutcome::Unavailable { missing: vec![DOWNLOADS] }));
        assert_eq!(home.access.tried(), []);
        assert!(access_changes(&events).is_empty(), "{events:?}");
        assert_eq!(ours(&home), []);
        assert_eq!(left_out(&events), []);
        let not_entered = vec![NotEntered { capability: DOWNLOADS, access: Access::NotAsked }];
        assert_eq!(coverage(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
        let module = home.flow.engine.modules().into_iter().find(|module| module.id.as_str() == MODULE).unwrap();
        assert_eq!(module.group, ModuleGroup::Files);
    }

    /// Flow 2: the switch turned on in Settings, the folder answers `Granted` when tried: in that same scan the
    /// engine learns it (one `AccessChanged`, before the module finishes) and the module offers; the report is the
    /// golden file.
    pub(crate) fn the_first_scan_after_the_opt_in_offers() -> Value {
        let home = Home::new();
        home.opt_in();

        let events = home.flow.scan();

        assert_eq!(home.access.tried().iter().map(|place| place.capability).collect::<Vec<_>>(), [DOWNLOADS]);
        let changes = access_changes(&events);
        assert_eq!(changes.len(), 1, "{events:?}");
        assert_eq!(changes[0].access(&DOWNLOADS), Access::Granted);
        let learnt = position(&events, |event| matches!(event, Event::AccessChanged { .. }));
        assert!(learnt < position(&events, module_finished), "the access is learnt before the module finishes");
        assert!(learnt < position(&events, |event| matches!(event, Event::FindingsAdded { .. })));
        assert_eq!(outcome(&events), Some(ModuleOutcome::Completed));
        assert_eq!(home.flow.engine.access_status().access(&DOWNLOADS), Access::Granted);
        assert_eq!(coverage(&events), Some(&Coverage::default()));
        assert_eq!(home.flow.findings(), everything());
        assert_eq!(
            left_out(&events),
            [("installers".to_owned(), "not-older".to_owned(), 1), ("unfinished".to_owned(), "recent".to_owned(), 1)]
        );
        json!({ "offered": home.flow.findings(), "leftOut": left_out(&events) })
    }

    /// Flow 3: the folder answers `Denied` when tried: the module stays unavailable, the coverage names the folder
    /// denied and nothing of Downloads is offered.
    pub(crate) fn a_denied_folder_keeps_the_module_unavailable() {
        let home = Home::with(Setup { answer: Access::Denied, ..Setup::default() });
        home.opt_in();

        let events = home.flow.scan();

        assert_eq!(home.access.tried().len(), 1);
        assert_eq!(home.flow.engine.access_status().access(&DOWNLOADS), Access::Denied);
        assert_eq!(outcome(&events), Some(ModuleOutcome::Unavailable { missing: vec![DOWNLOADS] }));
        assert_eq!(ours(&home), []);
        assert_eq!(left_out(&events), []);
        let not_entered = vec![NotEntered { capability: DOWNLOADS, access: Access::Denied }];
        assert_eq!(coverage(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
        let learnt = position(&events, |event| matches!(event, Event::AccessChanged { .. }));
        assert!(learnt < position(&events, module_finished), "the answer is learnt before the module is skipped");
    }

    /// Flow 4: every category selected and the plan names High (the big file); the items go to the Trash and the
    /// journal, the placeholder with its `.part`, and come back byte for byte; the user's folder, its `node_modules`,
    /// the kept items and the file outside Downloads stay untouched.
    pub(crate) fn scan_confirm_trash_and_put_back() {
        let home = Home::new();
        home.opt_in();
        let events = home.flow.scan();
        assert_eq!(home.flow.findings(), everything());
        assert_eq!(outcome(&events), Some(ModuleOutcome::Completed));
        let offered: Vec<&str> = OFFERED.iter().map(|(_, path)| *path).collect();
        let before = contents_of(&home, &offered);
        assert!(before.iter().all(|files| !files.is_empty()));
        let untouched: Vec<&str> = THE_USERS.iter().chain(&KEPT).copied().collect();
        let kept = contents_of(&home, &untouched);

        let view = home.flow.engine.prepare_cleanup(&home.everything()).unwrap();

        assert_eq!(view.summary.count, 8);
        assert_eq!(view.summary.highest, DangerLevel::High);
        let (cleanup, cleaned) = home.flow.clean(&view);
        let outcomes = home.flow.outcomes(&view, &cleaned);
        let mut planned = offered.clone();
        planned.sort_unstable();
        assert_eq!(outcomes.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(), planned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(planned.iter().all(|path| !home.flow.path(path).exists()));
        assert_eq!(fs::read_dir(home.flow.sandbox.trash()).unwrap().count(), 8);
        assert_eq!(home.flow.journal().iter().filter(|record| record["type"] == "item").count(), 8);
        assert_eq!(contents_of(&home, &untouched), kept, "the user's folder and the kept items stay");
        assert!(home.flow.path("home/Downloads/Project/node_modules/left-pad/index.js").is_file());

        let back = home.flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(contents_of(&home, &offered), before, "put back byte for byte");
        assert_eq!(contents_of(&home, &untouched), kept);
    }

    /// Flow 5: Chrome open: its unfinished download is left out `browser-open`; Chrome quits, and a rescan (nothing
    /// changed on disk) offers it.
    pub(crate) fn an_unfinished_download_goes_once_the_browser_quits() {
        // One listing of the open apps per scan: the first scan sees Chrome open, the rescan sees it quit.
        let running =
            FakeRunningApps::new(vec![running_app("Chrome", &["com.google.Chrome"]).unwrap()]).then(Vec::new());
        let home = Home::with(Setup { running, changed: vec![Vec::new()], ..Setup::default() });
        home.opt_in();

        let events = home.flow.scan();

        let unfinished = (UNFINISHED, "home/Downloads/movie.mp4.crdownload");
        assert!(!ours(&home).contains(&(unfinished.0.to_owned(), unfinished.1.to_owned())));
        assert!(
            left_out(&events).contains(&("unfinished".to_owned(), "browser-open".to_owned(), 1)),
            "{:?}",
            left_out(&events)
        );

        let events = home.flow.rescan();

        assert_eq!(home.flow.findings(), everything());
        assert_eq!(left_out(&events).iter().filter(|(category, ..)| category == "unfinished").count(), 1);
        assert!(!left_out(&events).iter().any(|(_, reason, _)| reason == "browser-open"));
    }

    /// Flow 6: an unfinished download 20 days old, within the setting's 30: left out `recent`; 12 days on, a rescan
    /// that reads nothing again offers it.
    pub(crate) fn an_unfinished_download_goes_once_the_days_pass() {
        let setup = Setup { changed: vec![Vec::new()], ..Setup::default() };
        let clock = std::sync::Arc::clone(&setup.clock);
        let home = Home::with(setup);
        home.opt_in();

        let events = home.flow.scan();

        let draft = ("downloads.unfinished".to_owned(), "home/Downloads/Draft.iso.opdownload".to_owned());
        assert!(!ours(&home).contains(&draft));
        assert!(left_out(&events).contains(&("unfinished".to_owned(), "recent".to_owned(), 1)));

        clock.advance(Duration::from_secs(12 * DAY));
        let events = home.flow.rescan();

        assert!(ours(&home).contains(&draft), "{:?}", ours(&home));
        assert!(!left_out(&events).iter().any(|(_, reason, _)| reason == "recent"), "{:?}", left_out(&events));
    }

    /// Flow 7: the installer of `Example` 2.0 is newer than the `Example` installed (1.0): left out `not-older`; the
    /// app is updated after it, and a rescan offers it, in the line of the app.
    pub(crate) fn an_installer_goes_once_the_app_is_updated() {
        let installed = FakeInstalledApps::new(installed_apps(Example::Old)).then(installed_apps(Example::Updated));
        let home = Home::with(Setup { installed, changed: vec![Vec::new()], ..Setup::default() });
        home.opt_in();

        let events = home.flow.scan();

        let offered_installer = ("downloads.installers".to_owned(), "home/Downloads/Example-2.0.dmg".to_owned());
        assert!(!ours(&home).contains(&offered_installer));
        assert!(left_out(&events).contains(&("installers".to_owned(), "not-older".to_owned(), 1)));

        let events = home.flow.rescan();

        assert!(ours(&home).contains(&offered_installer), "{:?}", ours(&home));
        assert!(!left_out(&events).iter().any(|(_, reason, _)| reason == "not-older"), "{:?}", left_out(&events));
        let groups = home.flow.engine.storage_meaning(0).unwrap();
        let shares = groups.iter().find_map(|group| match group {
            MeaningGroup::App { app, categories, .. } if app.id.as_str() == EXAMPLE => Some(categories),
            _ => None,
        });
        let shares: Vec<(&str, u64)> = shares
            .expect("the app has a line")
            .iter()
            .map(|share| (share.category.as_str(), share.sizes.logical.bytes()))
            .collect();
        assert_eq!(shares, [(INSTALLERS, total_bytes(&home, &["home/Downloads/Example-2.0.dmg"]))]);
    }

    /// Flow 8: the installer of Firefox is counted in Firefox's line of the by-meaning view; the browse view opens the
    /// user's folder `Downloads/Project` and lists its children, `node_modules` among them.
    pub(crate) fn the_storage_map_counts_an_installer_in_its_apps_line() {
        let home = Home::new();
        home.opt_in();
        home.flow.scan();

        let groups = home.flow.engine.storage_meaning(0).unwrap();

        let (app, shares) = groups
            .iter()
            .find_map(|group| match group {
                MeaningGroup::App { app, categories, .. } if app.id.as_str() == FIREFOX => Some((app, categories)),
                _ => None,
            })
            .expect("Firefox has a line");
        assert_eq!((app.name.as_str(), app.installed), ("Firefox", true));
        let shares: Vec<(&str, u64)> =
            shares.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        assert_eq!(shares, [(INSTALLERS, total_bytes(&home, &["home/Downloads/Firefox 133.0.dmg"]))]);

        let project = place(&home, &["Downloads", "Project"]);

        let mut names: Vec<&str> = project.rows.iter().map(|row| row.node.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, ["node_modules"]);
        assert!(project.rows.iter().all(|row| row.node.owner.is_none() && row.node.not_scanned.is_none()));
        let downloads = place(&home, &["Downloads"]);
        let owner = |name: &str| downloads.rows.iter().find(|row| row.node.name == name).and_then(|row| row.node.owner);
        assert_eq!(owner("Firefox 133.0.dmg").map(|id| id.as_str().to_owned()), Some(INSTALLERS.to_owned()));
        assert_eq!(owner("Project"), None);
    }

    /// Flow 9: the big files ignored: their finding is in no total while the others stay; restoring brings it back
    /// without a scan.
    pub(crate) fn ignoring_the_big_files_keeps_them_out_of_every_total() {
        let home = Home::new();
        home.opt_in();
        home.flow.scan();
        let all_totals = totals(&home);
        assert!(all_totals.iter().any(|category| category == LARGE));
        let found_bytes = |home: &Home| home.flow.engine.overview().unwrap().found.logical.bytes();
        let all_found = found_bytes(&home);

        home.flow.engine.ignore(&IgnoreTarget::Category { category: LARGE.into() }).unwrap();

        let without: Vec<(&str, &str)> = OFFERED.iter().copied().filter(|(of, _)| *of != LARGE).collect();
        assert_eq!(home.flow.findings(), found(&without));
        assert_eq!(totals(&home), all_totals.iter().filter(|category| *category != LARGE).cloned().collect::<Vec<_>>());
        assert_eq!(found_bytes(&home), all_found - total_bytes(&home, &["home/Downloads/video.mov"]));

        home.flow.engine.restore_ignored(&IgnoredEntry::Category { category: LARGE.into() }).unwrap();

        assert_eq!(home.flow.findings(), everything());
        assert_eq!(totals(&home), all_totals);
        assert_eq!(found_bytes(&home), all_found);
    }

    /// Flow 10: Full Disk Access counts as allowing Downloads: with it granted the module is available and offers
    /// without the Downloads switch, and the folder is never tried.
    pub(crate) fn full_disk_access_needs_no_try() {
        let home = Home::with(Setup { full_disk: Access::Granted, folder: Access::Granted, ..Setup::default() });

        let events = home.flow.scan();

        assert_eq!(home.access.tried(), []);
        assert!(access_changes(&events).is_empty(), "{events:?}");
        assert_eq!(outcome(&events), Some(ModuleOutcome::Completed));
        assert_eq!(home.flow.findings(), everything());
        assert_eq!(coverage(&events), Some(&Coverage::default()));
    }

    /// Flow 11: without Full Disk Access the switch decides, even when macOS remembers a grant: the switch turned on,
    /// a scan that tries the folder (granted), the switch turned off, and the next scan neither tries nor reads
    /// Downloads; the module is unavailable and the coverage names the folder not entered.
    pub(crate) fn a_remembered_grant_without_the_switch_reads_nothing() {
        let home = Home::new();
        home.opt_in();
        home.flow.scan();
        assert_eq!(home.flow.engine.access_status().access(&DOWNLOADS), Access::Granted);
        home.opt_out();

        let events = home.flow.scan();

        assert_eq!(home.access.tried().len(), 1, "only the first scan tried the folder");
        assert_eq!(home.flow.engine.access_status().access(&DOWNLOADS), Access::Granted);
        assert_eq!(outcome(&events), Some(ModuleOutcome::Unavailable { missing: vec![DOWNLOADS] }));
        assert_eq!(ours(&home), []);
        assert_eq!(left_out(&events), []);
        let not_entered = vec![NotEntered { capability: DOWNLOADS, access: Access::NotAsked }];
        assert_eq!(coverage(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
        let downloads = place(&home, &[]).rows.into_iter().find(|row| row.node.name == "Downloads");
        assert_eq!(downloads.map(|row| row.node.sizes.logical), Some(ByteSize::new(0)), "Downloads was not read");
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn nothing_is_read_in_downloads_before_the_opt_in() {
        flows::nothing_is_read_in_downloads_before_the_opt_in();
    }

    #[test]
    fn a_denied_folder_keeps_the_module_unavailable() {
        flows::a_denied_folder_keeps_the_module_unavailable();
    }

    #[test]
    fn an_unfinished_download_goes_once_the_browser_quits() {
        flows::an_unfinished_download_goes_once_the_browser_quits();
    }

    #[test]
    fn an_unfinished_download_goes_once_the_days_pass() {
        flows::an_unfinished_download_goes_once_the_days_pass();
    }

    #[test]
    fn an_installer_goes_once_the_app_is_updated() {
        flows::an_installer_goes_once_the_app_is_updated();
    }

    #[test]
    fn ignoring_the_big_files_keeps_them_out_of_every_total() {
        flows::ignoring_the_big_files_keeps_them_out_of_every_total();
    }

    #[test]
    fn a_remembered_grant_without_the_switch_reads_nothing() {
        flows::a_remembered_grant_without_the_switch_reads_nothing();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn the_first_scan_after_the_opt_in_offers() {
        check_golden("downloads-flow", &flows::the_first_scan_after_the_opt_in_offers());
    }

    #[test]
    fn scan_confirm_trash_and_put_back() {
        flows::scan_confirm_trash_and_put_back();
    }

    #[test]
    fn the_storage_map_counts_an_installer_in_its_apps_line() {
        flows::the_storage_map_counts_an_installer_in_its_apps_line();
    }

    #[test]
    fn full_disk_access_needs_no_try() {
        flows::full_disk_access_needs_no_try();
    }
}
