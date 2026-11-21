//! The app-caches module end to end: a fixture sandbox's home with an installed app's cache and its helper's, a removed
//! app's leftovers (`app-leftovers`' to offer), Apple's cache and two other modules' caches beside them under
//! `Library/Caches`; the real scanner (its reader seen through the facts lens), the modules the app registers for
//! them, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the sandbox,
//! with a fake Trash. The clock, the installed and open apps and the files held open are fakes a flow sets. The first
//! module on `EmptyFolder`: the emptying, the journal and put back are proven here. Flow 1's report is the golden
//! file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test app_caches` rewrites
//! `fixtures/golden/app-caches-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "app_caches_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    use fresher_core::{
        CleanupSelection, DangerLevel, Event, ItemOutcome, MeaningGroup, PutBackOutcome, ScanKind, SkipReason,
    };
    use fresher_testing::{FakeRunningApps, TreeReport, running_app};
    use serde_json::{Value, json};

    use crate::flow::{Flow, Setup, left_out};
    use crate::tree::contents;

    const CACHES: &str = "app-caches.library-caches";
    const EDITOR_ID: &str = "com.example.Editor";
    const EDITOR: &str = "home/Library/Caches/com.example.Editor";
    const HELPER: &str = "home/Library/Caches/com.example.Editor.Helper";
    /// The folders the module offers, as category and path.
    const OURS: [(&str, &str); 2] = [(CACHES, EDITOR), (CACHES, HELPER)];
    /// The removed app's cache, `app-leftovers`' to offer, and its saved state, the proof.
    const GONE: (&str, &str) = ("app-leftovers.caches", "home/Library/Caches/com.example.Gone");
    const GONE_STATE: (&str, &str) =
        ("app-leftovers.caches", "home/Library/Saved Application State/com.example.Gone.savedState");
    /// The other modules' answers under `Library/Caches`: Yarn's cache and `WebStorm`'s older version.
    const YARN: (&str, &str) = ("dev-caches.yarn", "home/Library/Caches/Yarn");
    const WEBSTORM: (&str, &str) = ("jetbrains.caches", "home/Library/Caches/JetBrains/WebStorm2024.3");
    /// The files no module claims: Apple's cache and the version of `WebStorm` in use.
    const KEPT: [&str; 2] = ["home/Library/Caches/com.apple.Safari", "home/Library/Caches/JetBrains/WebStorm2025.3"];

    fn found(cases: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            cases.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    /// Everything the four modules offer on the home.
    fn everything() -> Vec<(String, String)> {
        found(&[OURS[0], OURS[1], GONE, GONE_STATE, YARN, WEBSTORM])
    }

    /// Everything but the module's folders.
    fn without_ours() -> Vec<(String, String)> {
        found(&[GONE, GONE_STATE, YARN, WEBSTORM])
    }

    /// The findings of `module` only.
    fn of_module(flow: &Flow, module: &str) -> Vec<(String, String)> {
        let prefix = format!("{module}.");
        flow.findings().into_iter().filter(|(category, _)| category.starts_with(&prefix)).collect()
    }

    fn selection() -> CleanupSelection {
        serde_json::from_value(json!({ "findings": [], "categories": [{ "category": CACHES, "except": [] }] })).unwrap()
    }

    fn contents_of(flow: &Flow, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    fn bytes_of(flow: &Flow, paths: &[&str]) -> u64 {
        contents_of(flow, paths).iter().map(|files| files.values().map(|bytes| bytes.len() as u64).sum::<u64>()).sum()
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    fn editor_open() -> FakeRunningApps {
        FakeRunningApps::new(vec![running_app("Editor", &[EDITOR_ID]).unwrap()])
    }

    fn scan_kind(events: &[Event]) -> Option<&ScanKind> {
        events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        })
    }

    /// The bytes of every file under the fake Trash, sorted: what went there, whatever the numbering of its names.
    fn trashed_bytes(flow: &Flow) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = contents(&flow.sandbox.trash()).into_values().collect();
        bytes.sort();
        bytes
    }

    fn entries(flow: &Flow, folder: &str) -> usize {
        fs::read_dir(flow.path(folder)).unwrap().count()
    }

    /// Flow 1: the scan offers the two folders beside the other modules' answers; the cleanup of the category empties
    /// each folder and keeps it (the app writes there as it works), its top-level entries go to the Trash and the
    /// journal; put back returns them byte for byte; the removed app's, Apple's and the other modules' files stay.
    pub(crate) fn scan_confirm_empty_and_put_back() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
        let paths = [EDITOR, HELPER];
        let before = contents_of(&flow, &paths);
        assert!(before.iter().all(|files| !files.is_empty() && files.values().all(|bytes| !bytes.is_empty())));
        let others = [GONE.1, GONE_STATE.1, YARN.1, WEBSTORM.1, KEPT[0], KEPT[1]];
        let kept = contents_of(&flow, &others);

        let view = flow.engine.prepare_cleanup(&selection()).unwrap();

        assert_eq!((view.summary.count, view.summary.highest), (2, DangerLevel::Low));
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let expected = [
            (EDITOR.to_owned(), ItemOutcome::Emptied { moved: 2, skipped: 0 }),
            (HELPER.to_owned(), ItemOutcome::Emptied { moved: 1, skipped: 0 }),
        ];
        assert_eq!(outcomes, expected);
        assert!(paths.iter().all(|path| flow.path(path).is_dir() && entries(&flow, path) == 0), "emptied, not removed");
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 3);
        assert_eq!(trashed_bytes(&flow), bytes_under_before(&before), "every file is in the Trash");
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 2);
        assert_eq!(contents_of(&flow, &others), kept, "no other folder is touched");
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 2);
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(contents_of(&flow, &paths), before, "put back byte for byte");
        assert_eq!(contents_of(&flow, &others), kept);
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 0);
        let report = json!({
            "offered": everything(),
            "leftOut": left_out(&events),
            "plan": view,
            "outcomes": outcomes,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": tree(&flow),
        });
        flow.relative(&report)
    }

    fn bytes_under_before(before: &[BTreeMap<PathBuf, Vec<u8>>]) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = before.iter().flat_map(|files| files.values().cloned()).collect();
        bytes.sort();
        bytes
    }

    /// Flow 2: both modules run on one home that looks overlapping (the removed app's cache has the same shape as the
    /// installed app's): no path is offered twice, `app-caches` offers only the installed app's folders and
    /// `app-leftovers` only the removed app's, the other modules' answers are theirs, Apple's is no one's.
    pub(crate) fn no_folder_is_offered_by_two_modules() {
        let flow = Flow::new();

        flow.scan();

        let all = flow.findings();
        let paths: BTreeSet<&String> = all.iter().map(|(_, path)| path).collect();
        assert_eq!(paths.len(), all.len(), "a path is offered once: {all:?}");
        assert_eq!(of_module(&flow, "app-caches"), found(&OURS));
        assert_eq!(of_module(&flow, "app-leftovers"), found(&[GONE, GONE_STATE]));
        assert_eq!(of_module(&flow, "dev-caches"), found(&[YARN]));
        assert_eq!(of_module(&flow, "jetbrains"), found(&[WEBSTORM]));
        assert!(all.iter().all(|(_, path)| !path.contains("Safari") && !path.contains("WebStorm2025.3")), "{all:?}");
    }

    /// Flow 3: the app open: its folders are left out `app-open`, every other answer unchanged; it quits, and a rescan
    /// offers them.
    pub(crate) fn an_open_apps_cache_is_left_out_until_it_quits() {
        let running = editor_open().then(Vec::new());
        let flow = Flow::with(Setup { running, changed: vec![Vec::new()], ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), without_ours());
        assert_eq!(left_out(&events), [("library-caches".to_owned(), "app-open".to_owned(), 2)]);

        let events = flow.rescan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
    }

    /// Flow 4: a file held open when the cleanup acts: the app's folder is skipped `in use`, nothing of it moves, the
    /// Trash holds only the helper's emptied entry.
    pub(crate) fn an_entry_held_open_skips_the_item_in_use() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&selection()).unwrap();
        let before = contents_of(&flow, &[EDITOR]);
        let helper_before = contents_of(&flow, &[HELPER]);
        flow.open_files.hold(flow.path("home/Library/Caches/com.example.Editor/Cache.db"), "Editor");

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let skipped = ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Editor".into() } };
        let emptied = ItemOutcome::Emptied { moved: 1, skipped: 0 };
        assert_eq!(outcomes, [(EDITOR.to_owned(), skipped), (HELPER.to_owned(), emptied)]);
        assert_eq!(contents_of(&flow, &[EDITOR]), before, "nothing of the app's folder moved");
        assert_eq!(entries(&flow, HELPER), 0);
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 1, "only the helper's entry is in the Trash");
        assert_eq!(trashed_bytes(&flow), bytes_under_before(&helper_before), "the helper's file");
    }

    /// Flow 5: the app wrote a new file into its emptied folder before the user put the cleanup back: put back
    /// returns every entry, byte for byte, beside the new file, which stays.
    pub(crate) fn put_back_restores_every_entry_beside_what_the_app_wrote() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&selection()).unwrap();
        let before = contents_of(&flow, &[EDITOR, HELPER]);
        let (cleanup, _) = flow.clean(&view);
        let written = flow.path("home/Library/Caches/com.example.Editor/new.cache");
        fs::write(&written, b"written after the cleanup").unwrap();

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 2);
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let mut expected = before[0].clone();
        expected.insert(written.clone(), b"written after the cleanup".to_vec());
        assert_eq!(contents(&flow.path(EDITOR)), expected, "every entry is back and the new file stays");
        assert_eq!(contents(&flow.path(HELPER)), before[1]);
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 0);
    }

    /// Flow 6: a new engine on the same files restores the last scan at launch and offers the same folders (the
    /// scan cache holds their matches), with nothing left out.
    pub(crate) fn a_restored_scan_offers_the_same_caches() {
        let mut flow = Flow::new();
        flow.scan();
        let offered: Vec<_> = of_module(&flow, "app-caches");
        assert_eq!(offered, found(&OURS));
        flow.restart(Setup::default());

        let restored = flow.restore();

        assert!(matches!(scan_kind(&restored), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&restored));
        assert_eq!(of_module(&flow, "app-caches"), offered);
        assert_eq!(left_out(&restored), []);
    }

    /// Flow 6b: a restart with the app open since judges the cached matches again: the folders are left out
    /// `app-open`, and a scan after the app quits offers them.
    pub(crate) fn a_restored_scan_judges_the_open_app_again() {
        let mut flow = Flow::new();
        flow.scan();
        let offered: Vec<_> = of_module(&flow, "app-caches");
        flow.restart(Setup { running: editor_open().then(Vec::new()), ..Setup::default() });

        let restored = flow.restore();

        assert_eq!(of_module(&flow, "app-caches"), []);
        assert_eq!(left_out(&restored), [("library-caches".to_owned(), "app-open".to_owned(), 2)]);

        let events = flow.scan();

        assert_eq!(of_module(&flow, "app-caches"), offered);
        assert_eq!(left_out(&events), []);
    }

    /// Flow 7: the storage map counts both folders in the installed app's line, in the module's category.
    pub(crate) fn the_map_counts_both_folders_in_the_apps_line() {
        let flow = Flow::new();
        flow.scan();

        let groups = flow.engine.storage_meaning(0).unwrap();

        let (app, sizes, categories) = groups
            .iter()
            .find_map(|group| match group {
                MeaningGroup::App { app, sizes, categories, .. } if app.id.as_str() == EDITOR_ID => {
                    Some((app, sizes, categories))
                }
                _ => None,
            })
            .expect("the installed app has a line");
        assert_eq!((app.name.as_str(), app.installed), ("Editor", true));
        let shares: Vec<(&str, u64)> =
            categories.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        assert_eq!(shares, [(CACHES, bytes_of(&flow, &[EDITOR, HELPER]))]);
        assert_eq!(sizes.logical.bytes(), bytes_of(&flow, &[EDITOR, HELPER]));
    }
}

mod negative {
    use crate::flows::{
        a_restored_scan_judges_the_open_app_again, an_entry_held_open_skips_the_item_in_use,
        an_open_apps_cache_is_left_out_until_it_quits, no_folder_is_offered_by_two_modules,
    };

    #[test]
    fn no_folder_is_offered_by_both_app_caches_and_app_leftovers() {
        no_folder_is_offered_by_two_modules();
    }

    #[test]
    fn an_open_app_s_cache_is_left_out_and_offered_once_it_quits() {
        an_open_apps_cache_is_left_out_until_it_quits();
    }

    #[test]
    fn an_entry_held_open_skips_the_item_and_moves_nothing_of_it() {
        an_entry_held_open_skips_the_item_in_use();
    }

    #[test]
    fn a_restart_with_the_app_open_leaves_its_caches_out_until_a_scan_after_it_quits() {
        a_restored_scan_judges_the_open_app_again();
    }
}

mod positive {
    use crate::flows::{
        a_restored_scan_offers_the_same_caches, put_back_restores_every_entry_beside_what_the_app_wrote,
        scan_confirm_empty_and_put_back, the_map_counts_both_folders_in_the_apps_line,
    };
    use crate::support::check_golden;

    #[test]
    fn a_confirmed_cleanup_empties_the_folders_keeps_them_and_put_back_restores_every_entry() {
        check_golden("app-caches-flow", &scan_confirm_empty_and_put_back());
    }

    #[test]
    fn put_back_restores_every_entry_byte_for_byte_beside_a_file_the_app_wrote() {
        put_back_restores_every_entry_beside_what_the_app_wrote();
    }

    #[test]
    fn a_restored_scan_offers_the_same_caches_with_nothing_left_out() {
        a_restored_scan_offers_the_same_caches();
    }

    #[test]
    fn the_map_counts_both_folders_in_the_installed_apps_line() {
        the_map_counts_both_folders_in_the_apps_line();
    }
}
