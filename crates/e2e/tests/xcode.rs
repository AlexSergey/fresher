//! The xcode module end to end, beside `dev-caches` (the other module of `Library/Developer`): a fixture sandbox's home
//! with three iOS device support folders, Xcode's previews, the simulators' caches, an archive and a derived data
//! project; the real scanner (its reader seen through the facts lens), both modules as the app registers them, the
//! real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the sandbox, with a fake
//! Trash. Xcode is installed; the open apps at each scan, the files held open and the change journal's answers are
//! fakes a flow sets. The storage map's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test xcode` rewrites `fixtures/golden/xcode-flow.json`; the
//! diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "xcode_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    use fresher_core::{
        DangerLevel, Event, ItemOutcome, MeaningGroup, PutBackOutcome, RunningApp, ScanKind, SkipReason,
    };
    use serde_json::json;

    use crate::flow::{Flow, Setup, XCODE, left_out, select, simulator_open, xcode_open};
    use crate::support::check_golden;
    use crate::tree::contents;

    const DEVICE_SUPPORT: &str = "xcode.device-support";
    const PREVIEWS: &str = "xcode.previews";
    const SIMULATOR_CACHES: &str = "xcode.simulator-caches";
    const ARCHIVES: &str = "xcode.archives";
    /// The module's categories.
    const CATEGORIES: [&str; 4] = [DEVICE_SUPPORT, PREVIEWS, SIMULATOR_CACHES, ARCHIVES];

    const IOS: &str = "home/Library/Developer/Xcode/iOS DeviceSupport";
    const IOS_15: &str = "home/Library/Developer/Xcode/iOS DeviceSupport/15.0 (19A346)";
    const IOS_16: &str = "home/Library/Developer/Xcode/iOS DeviceSupport/16.4.1 (20E252)";
    const IOS_17: &str = "home/Library/Developer/Xcode/iOS DeviceSupport/17.2 (21C62)";
    const PREVIEWS_FOLDER: &str = "home/Library/Developer/Xcode/UserData/Previews";
    const CACHES: &str = "home/Library/Developer/CoreSimulator/Caches";
    const ARCHIVE: &str = "home/Library/Developer/Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive";
    /// The file of the simulators' caches a flow holds open.
    const CACHE_FILE: &str = "home/Library/Developer/CoreSimulator/Caches/dyld/24D70/runtime/cache";
    const DERIVED_DATA: &str = "home/Library/Developer/Xcode/DerivedData";
    const PROJECT: (&str, &str) =
        ("dev-caches.xcode-derived-data", "home/Library/Developer/Xcode/DerivedData/App-abcdefghijklmnopqrstuvwxyzab");

    /// The module's six folders, as category and path, in the order `Flow::findings` sorts them.
    const OURS: [(&str, &str); 6] = [
        (ARCHIVES, ARCHIVE),
        (DEVICE_SUPPORT, IOS_15),
        (DEVICE_SUPPORT, IOS_16),
        (DEVICE_SUPPORT, IOS_17),
        (PREVIEWS, PREVIEWS_FOLDER),
        (SIMULATOR_CACHES, CACHES),
    ];
    /// The folders that go to the Trash whole.
    const MOVED: [&str; 4] = [ARCHIVE, IOS_15, IOS_16, IOS_17];
    /// The folders emptied and kept.
    const EMPTIED: [&str; 2] = [PREVIEWS_FOLDER, CACHES];

    fn found(cases: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            cases.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    /// The findings of `module` only.
    fn of_module(flow: &Flow, module: &str) -> Vec<(String, String)> {
        let prefix = format!("{module}.");
        flow.findings().into_iter().filter(|(category, _)| category.starts_with(&prefix)).collect()
    }

    /// Everything both modules offer on the home.
    fn everything() -> Vec<(String, String)> {
        found(&everything_refs())
    }

    /// The module's six folders left out `app-open`, by category.
    fn all_left_out_open() -> Vec<(String, String, u32)> {
        let reason = "xcode.left-out.app-open";
        let mut counts: Vec<(String, String, u32)> =
            [(ARCHIVES, 1), (DEVICE_SUPPORT, 3), (PREVIEWS, 1), (SIMULATOR_CACHES, 1)]
                .into_iter()
                .map(|(category, items)| (category.to_owned(), reason.to_owned(), items))
                .collect();
        counts.sort();
        counts
    }

    fn scan_kind(events: &[Event]) -> Option<&ScanKind> {
        events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        })
    }

    fn contents_of(flow: &Flow, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    fn bytes_of(flow: &Flow, path: &str) -> u64 {
        contents(&flow.path(path)).values().map(|bytes| bytes.len() as u64).sum()
    }

    fn entries(flow: &Flow, folder: &str) -> usize {
        fs::read_dir(flow.path(folder)).unwrap().count()
    }

    /// The names in the fake Trash without its order number (`3-Name` is `Name`), sorted.
    fn trashed_names(flow: &Flow) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(flow.sandbox.trash())
            .unwrap()
            .map(|entry| {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                name.split_once('-').map_or(name.clone(), |(_, rest)| rest.to_owned())
            })
            .collect();
        names.sort();
        names
    }

    /// Both modules on one home: the derived data project is `dev-caches`' only, and no `xcode` finding lies under
    /// `Xcode/DerivedData`; no path is offered twice.
    pub(crate) fn no_folder_is_offered_by_both_modules() {
        let flow = Flow::new();

        flow.scan();

        let all = flow.findings();
        assert_eq!(all, everything());
        let paths: BTreeSet<&String> = all.iter().map(|(_, path)| path).collect();
        assert_eq!(paths.len(), all.len(), "a path is offered once: {all:?}");
        assert_eq!(of_module(&flow, "dev-caches"), found(&[PROJECT]));
        assert_eq!(of_module(&flow, "xcode"), found(&OURS));
        assert!(of_module(&flow, "xcode").iter().all(|(_, path)| !path.starts_with(DERIVED_DATA)));
    }

    /// `open` running at the first scan: no `xcode` finding, the six folders left out `app-open`; it quits, and a
    /// rescan offers them all.
    pub(crate) fn everything_waits_while_open(open: Vec<RunningApp>) {
        let flow = Flow::with(Setup { running: vec![open, Vec::new()], changed: vec![Vec::new()] });

        let events = flow.scan();

        assert_eq!(of_module(&flow, "xcode"), []);
        assert_eq!(left_out(&events, "xcode"), all_left_out_open());

        let events = flow.rescan();

        assert_eq!(of_module(&flow, "xcode"), found(&OURS));
        assert_eq!(left_out(&events, "xcode"), []);
    }

    pub(crate) fn waits_while_xcode_is_open() {
        everything_waits_while_open(xcode_open());
    }

    pub(crate) fn waits_while_the_simulator_is_open() {
        everything_waits_while_open(simulator_open());
    }

    /// A file inside the simulators' caches held open at the act: the cleaner's in-use check skips that item whole,
    /// nothing of it moves; every other item goes.
    pub(crate) fn a_file_held_open_skips_the_simulator_caches_in_use() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select(&CATEGORIES)).unwrap();
        let before = contents_of(&flow, &[CACHES]);
        flow.open_files.hold(flow.path(CACHE_FILE), "Simulator");

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let in_use = ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Simulator".into() } };
        let expected = [
            (CACHES.to_owned(), in_use),
            (ARCHIVE.to_owned(), ItemOutcome::Moved),
            (PREVIEWS_FOLDER.to_owned(), ItemOutcome::Emptied { moved: 1, skipped: 0 }),
            (IOS_15.to_owned(), ItemOutcome::Moved),
            (IOS_16.to_owned(), ItemOutcome::Moved),
            (IOS_17.to_owned(), ItemOutcome::Moved),
        ];
        assert_eq!(outcomes, expected);
        assert_eq!(contents_of(&flow, &[CACHES]), before, "nothing of the simulators' caches moved");
        assert!(!trashed_names(&flow).contains(&"dyld".to_owned()), "{:?}", trashed_names(&flow));
        let expected = [
            "15.0 (19A346)",
            "16.4.1 (20E252)",
            "17.2 (21C62)",
            "Shop 01-03-2026, 10.15.xcarchive",
            "Simulator Devices",
        ];
        assert_eq!(trashed_names(&flow), expected);
    }

    /// Every `xcode` finding selected and confirmed to the Trash: the device support folders and the archive are in
    /// the Trash, the previews and the simulators' caches emptied and kept; the derived data project stays.
    pub(crate) fn a_cleanup_moves_and_empties() {
        let flow = Flow::new();
        flow.scan();
        let project = contents_of(&flow, &[PROJECT.1]);

        let view = flow.engine.prepare_cleanup(&select(&CATEGORIES)).unwrap();

        assert_eq!((view.summary.count, view.summary.highest), (6, DangerLevel::High));
        let (_, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let moved: Vec<&str> = outcomes
            .iter()
            .filter(|(_, outcome)| *outcome == ItemOutcome::Moved)
            .map(|(path, _)| path.as_str())
            .collect();
        assert_eq!(moved, MOVED);
        let emptied = ItemOutcome::Emptied { moved: 1, skipped: 0 };
        assert_eq!(outcomes.iter().filter(|(_, outcome)| *outcome == emptied).count(), 2, "{outcomes:?}");
        assert!(MOVED.iter().all(|path| !flow.path(path).exists()));
        assert!(EMPTIED.iter().all(|path| flow.path(path).is_dir() && entries(&flow, path) == 0), "emptied, kept");
        assert!(flow.path(IOS).is_dir(), "the platform's folder stays");
        let expected = [
            "15.0 (19A346)",
            "16.4.1 (20E252)",
            "17.2 (21C62)",
            "Shop 01-03-2026, 10.15.xcarchive",
            "Simulator Devices",
            "dyld",
        ];
        assert_eq!(trashed_names(&flow), expected);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 6);
        assert_eq!(contents_of(&flow, &[PROJECT.1]), project, "the derived data project stays");
        assert_eq!(of_module(&flow, "xcode"), []);
    }

    /// After the cleanup, put back of the archive alone: every file of it as it was.
    pub(crate) fn put_back_restores_the_archive() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select(&CATEGORIES)).unwrap();
        let before = contents_of(&flow, &[ARCHIVE]);
        let (cleanup, _) = flow.clean(&view);
        assert!(!flow.path(ARCHIVE).exists());
        let detail = flow.engine.cleanup(cleanup).unwrap();
        let archive = detail.items.iter().find(|item| item.path.as_deref() == Some(flow.path(ARCHIVE).as_path()));
        let archive = archive.unwrap().index;

        let back = flow.engine.put_back(cleanup, Some(&[archive])).unwrap();

        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].outcome, PutBackOutcome::PutBack, "{back:?}");
        assert_eq!(contents_of(&flow, &[ARCHIVE]), before, "put back byte for byte");
        assert!(MOVED[1..].iter().all(|path| !flow.path(path).exists()), "the rest stays in the Trash");
    }

    /// A file written into the emptied simulators' caches after the cleanup stays; put back brings every entry back
    /// beside it, and every other item as it was.
    pub(crate) fn put_back_restores_the_emptied_caches_beside_a_new_file() {
        let flow = Flow::new();
        let all: Vec<&str> = OURS.iter().map(|(_, path)| *path).collect();
        flow.scan();
        let before = contents_of(&flow, &all);
        let view = flow.engine.prepare_cleanup(&select(&CATEGORIES)).unwrap();
        let (cleanup, _) = flow.clean(&view);
        let written = flow.path(CACHES).join("new.cache");
        fs::write(&written, b"written after the cleanup").unwrap();

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 6);
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let mut expected = before.clone();
        let caches = OURS.iter().position(|(_, path)| *path == CACHES).unwrap();
        expected[caches].insert(written, b"written after the cleanup".to_vec());
        assert_eq!(contents_of(&flow, &all), expected, "every entry is back and the new file stays");
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 0);
    }

    /// A new engine on the same files restores the last scan: with Xcode closed it offers the same; restarted with
    /// Xcode open, it leaves the six folders out `app-open`.
    pub(crate) fn a_restored_scan_judges_the_open_apps_again() {
        let mut flow = Flow::new();
        flow.scan();
        let offered = of_module(&flow, "xcode");
        assert_eq!(offered, found(&OURS));
        flow.restart(Setup::default());

        let restored = flow.restore();

        assert!(matches!(scan_kind(&restored), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&restored));
        assert_eq!(of_module(&flow, "xcode"), offered);
        assert_eq!(left_out(&restored, "xcode"), []);

        flow.restart(Setup { running: vec![xcode_open()], changed: Vec::new() });

        let restored = flow.restore();

        assert!(matches!(scan_kind(&restored), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&restored));
        assert_eq!(of_module(&flow, "xcode"), []);
        assert_eq!(left_out(&restored, "xcode"), all_left_out_open());
    }

    /// The storage map counts every finding of both modules in Xcode's line, by category; the report is the golden.
    pub(crate) fn the_storage_map_counts_every_finding_in_xcodes_line() {
        let flow = Flow::new();
        flow.scan();

        let groups = flow.engine.storage_meaning(0).unwrap();

        let line = groups
            .iter()
            .find(|group| matches!(group, MeaningGroup::App { app, .. } if app.id.as_str() == XCODE))
            .expect("Xcode has a line");
        let MeaningGroup::App { app, sizes, categories, .. } = line else { unreachable!() };
        assert_eq!((app.name.as_str(), app.installed), ("Xcode", true));
        let mut by_category: BTreeMap<&str, u64> = BTreeMap::new();
        for (category, path) in everything_refs() {
            *by_category.entry(category).or_default() += bytes_of(&flow, path);
        }
        let shares: BTreeMap<&str, u64> =
            categories.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        assert_eq!(shares, by_category);
        assert_eq!(sizes.logical.bytes(), by_category.values().sum::<u64>());
        let report = json!({ "offered": flow.findings(), "xcodeLine": line });
        check_golden("xcode-flow", &flow.relative(&report));
    }

    /// Every offered folder, as category and path.
    fn everything_refs() -> Vec<(&'static str, &'static str)> {
        OURS.into_iter().chain([PROJECT]).collect()
    }
}

mod negative {
    use crate::flows::{
        a_file_held_open_skips_the_simulator_caches_in_use, no_folder_is_offered_by_both_modules,
        waits_while_the_simulator_is_open, waits_while_xcode_is_open,
    };

    #[test]
    fn no_folder_is_offered_by_both_xcode_and_dev_caches() {
        no_folder_is_offered_by_both_modules();
    }

    #[test]
    fn everything_waits_while_xcode_is_open_until_a_rescan_after_it_quits() {
        waits_while_xcode_is_open();
    }

    #[test]
    fn everything_waits_while_the_simulator_is_open() {
        waits_while_the_simulator_is_open();
    }

    #[test]
    fn a_file_held_open_skips_the_simulator_caches_in_use_and_the_rest_go() {
        a_file_held_open_skips_the_simulator_caches_in_use();
    }
}

mod positive {
    use crate::flows::{
        a_cleanup_moves_and_empties, a_restored_scan_judges_the_open_apps_again, put_back_restores_the_archive,
        put_back_restores_the_emptied_caches_beside_a_new_file, the_storage_map_counts_every_finding_in_xcodes_line,
    };

    #[test]
    fn a_cleanup_moves_device_support_and_the_archive_and_empties_the_caches() {
        a_cleanup_moves_and_empties();
    }

    #[test]
    fn put_back_restores_the_archive_byte_for_byte() {
        put_back_restores_the_archive();
    }

    #[test]
    fn put_back_restores_the_emptied_caches_beside_a_file_written_since() {
        put_back_restores_the_emptied_caches_beside_a_new_file();
    }

    #[test]
    fn a_restored_scan_offers_the_same_while_xcode_is_closed_and_leaves_out_while_it_is_open() {
        a_restored_scan_judges_the_open_apps_again();
    }

    #[test]
    fn the_storage_map_counts_every_finding_in_xcode_s_line() {
        the_storage_map_counts_every_finding_in_xcodes_line();
    }
}
