//! The app-caches module's browser and web-engine rows end to end: a fixture sandbox's home with Google Chrome's caches
//! of two profiles under `Library/Caches` beside a profile folder that is not a table cache and its profile data in
//! its support folder, Redis Insight's caches in its support folder beside its cookies and local storage, and the
//! other modules' answers beside them (`jetbrains`, `dev-caches`, `app-leftovers`, the module's own id-named cache);
//! the real scanner (its reader seen through the facts lens), the modules the app registers, the real settings file,
//! scan cache and journal, and the real `DiskCleaner` reaching only inside the sandbox, with a fake Trash. The clock,
//! the installed and open apps and the files held open are fakes a flow sets. Flow 1's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test app_caches_browsers` rewrites
//! `fixtures/golden/app-caches-browsers-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "app_caches_browsers_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    use fresher_core::{CleanupSelection, DangerLevel, ItemOutcome, MeaningGroup, PutBackOutcome, SkipReason};
    use fresher_testing::{FakeRunningApps, TreeReport, running_app};
    use serde_json::{Value, json};

    use crate::flow::{CHROME_ID, Flow, REDIS_INSIGHT_ID, Setup, left_out};
    use crate::tree::contents;

    const BROWSERS: &str = "app-caches.browsers";
    const WEB_ENGINE: &str = "app-caches.web-engine";
    const LIBRARY_CACHES: &str = "app-caches.library-caches";
    /// The module's three categories, in its order.
    const CATEGORIES: [&str; 3] = [LIBRARY_CACHES, BROWSERS, WEB_ENGINE];

    const CHROME_CACHE: &str = "home/Library/Caches/Google/Chrome/Default/Cache";
    const CHROME_CODE_CACHE: &str = "home/Library/Caches/Google/Chrome/Default/Code Cache";
    const CHROME_PROFILE_CACHE: &str = "home/Library/Caches/Google/Chrome/Profile 1/Cache";
    const REDIS_INSIGHT_CACHE: &str = "home/Library/Application Support/RedisInsight/Cache";
    const REDIS_INSIGHT_GPU_CACHE: &str = "home/Library/Application Support/RedisInsight/GPUCache";
    const EDITOR: &str = "home/Library/Caches/com.example.Editor";
    /// Google Chrome's three caches.
    const CHROME: [(&str, &str); 3] =
        [(BROWSERS, CHROME_CACHE), (BROWSERS, CHROME_CODE_CACHE), (BROWSERS, CHROME_PROFILE_CACHE)];
    /// Redis Insight's two caches.
    const REDIS_INSIGHT: [(&str, &str); 2] = [(WEB_ENGINE, REDIS_INSIGHT_CACHE), (WEB_ENGINE, REDIS_INSIGHT_GPU_CACHE)];
    /// The installed `com.example.Editor`'s id-named cache.
    const EDITOR_CACHE: (&str, &str) = (LIBRARY_CACHES, EDITOR);
    /// Every folder the module offers.
    const OURS: [&str; 6] =
        [CHROME_CACHE, CHROME_CODE_CACHE, CHROME_PROFILE_CACHE, REDIS_INSIGHT_CACHE, REDIS_INSIGHT_GPU_CACHE, EDITOR];
    /// The removed app's cache and saved state, `app-leftovers`' to offer.
    const GONE: (&str, &str) = ("app-leftovers.caches", "home/Library/Caches/com.example.Gone");
    const GONE_STATE: (&str, &str) =
        ("app-leftovers.caches", "home/Library/Saved Application State/com.example.Gone.savedState");
    /// The other modules' answers: Yarn's cache and `WebStorm`'s older version.
    const YARN: (&str, &str) = ("dev-caches.yarn", "home/Library/Caches/Yarn");
    const WEBSTORM: (&str, &str) = ("jetbrains.caches", "home/Library/Caches/JetBrains/WebStorm2024.3");
    /// What a profile and an app keep beside their caches: never offered, never touched.
    const DATA: [&str; 7] = [
        "home/Library/Application Support/Google/Chrome/Default/History",
        "home/Library/Application Support/Google/Chrome/Default/Cookies",
        "home/Library/Application Support/Google/Chrome/Default/Login Data",
        "home/Library/Application Support/Google/Chrome/Default/Preferences",
        "home/Library/Caches/Google/Chrome/Default/image_cache",
        "home/Library/Application Support/RedisInsight/Cookies",
        "home/Library/Application Support/RedisInsight/Local Storage",
    ];

    fn found(cases: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            cases.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    /// Everything the four modules offer on the home.
    fn everything() -> Vec<(String, String)> {
        let mut all: Vec<(&str, &str)> = CHROME.to_vec();
        all.extend(REDIS_INSIGHT);
        all.extend([EDITOR_CACHE, GONE, GONE_STATE, YARN, WEBSTORM]);
        found(&all)
    }

    /// Everything but Google Chrome's caches.
    fn without_chrome() -> Vec<(String, String)> {
        let chrome: BTreeSet<(String, String)> = found(&CHROME).into_iter().collect();
        everything().into_iter().filter(|finding| !chrome.contains(finding)).collect()
    }

    /// The findings of `module` only.
    fn of_module(flow: &Flow, module: &str) -> Vec<(String, String)> {
        let prefix = format!("{module}.");
        flow.findings().into_iter().filter(|(category, _)| category.starts_with(&prefix)).collect()
    }

    /// The whole of each of `categories`.
    fn selection(categories: &[&str]) -> CleanupSelection {
        let categories: Vec<Value> =
            categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
        serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
    }

    fn contents_of(flow: &Flow, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    fn bytes_of(flow: &Flow, paths: &[&str]) -> u64 {
        contents_of(flow, paths).iter().map(|files| files.values().map(|bytes| bytes.len() as u64).sum::<u64>()).sum()
    }

    fn sorted_bytes(files: &[BTreeMap<PathBuf, Vec<u8>>]) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = files.iter().flat_map(|files| files.values().cloned()).collect();
        bytes.sort();
        bytes
    }

    /// The bytes of every file under the fake Trash, sorted: what went there, whatever the numbering of its names.
    fn trashed_bytes(flow: &Flow) -> Vec<Vec<u8>> {
        sorted_bytes(&[contents(&flow.sandbox.trash())])
    }

    fn trashed(flow: &Flow) -> usize {
        fs::read_dir(flow.sandbox.trash()).unwrap().count()
    }

    fn entries(flow: &Flow, folder: &str) -> usize {
        fs::read_dir(flow.path(folder)).unwrap().count()
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    fn chrome_open() -> FakeRunningApps {
        FakeRunningApps::new(vec![running_app("Google Chrome", &[CHROME_ID]).unwrap()])
    }

    /// Whether `path` is `other`, holds it or lies inside it.
    fn overlaps(path: &Path, other: &Path) -> bool {
        path.starts_with(other) || other.starts_with(path)
    }

    /// Flow 1: the scan offers each table cache folder beside the other modules' answers; the cleanup of the module's
    /// categories empties each folder and keeps it, its top-level entries go to the Trash and the journal; put back
    /// returns them byte for byte; the data beside the caches and the other modules' files stay.
    pub(crate) fn scan_confirm_empty_and_put_back() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
        let before = contents_of(&flow, &OURS);
        assert!(before.iter().all(|files| !files.is_empty() && files.values().all(|bytes| !bytes.is_empty())));
        let mut others = DATA.to_vec();
        others.extend([GONE.1, GONE_STATE.1, YARN.1, WEBSTORM.1]);
        let kept = contents_of(&flow, &others);

        let view = flow.engine.prepare_cleanup(&selection(&CATEGORIES)).unwrap();

        assert_eq!((view.summary.count, view.summary.highest), (6, DangerLevel::Low));
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let mut expected: Vec<(String, ItemOutcome)> =
            OURS.iter().map(|path| ((*path).to_owned(), ItemOutcome::Emptied { moved: 1, skipped: 0 })).collect();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(outcomes, expected);
        assert!(OURS.iter().all(|path| flow.path(path).is_dir() && entries(&flow, path) == 0), "emptied, not removed");
        assert_eq!(trashed(&flow), 6, "each cache's one top-level entry");
        assert_eq!(trashed_bytes(&flow), sorted_bytes(&before), "every file is in the Trash");
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 6);
        assert_eq!(contents_of(&flow, &others), kept, "no other file is touched");
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 6);
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(contents_of(&flow, &OURS), before, "put back byte for byte");
        assert_eq!(contents_of(&flow, &others), kept);
        assert_eq!(trashed(&flow), 0);
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

    /// Flow 2: the four modules on one home: no path is offered twice; Chrome's caches are the browsers category's,
    /// Redis Insight's the web-engine category's, Editor's id-named cache `library-caches`', and the removed app's,
    /// Yarn's and the IDE's their modules'.
    pub(crate) fn no_folder_is_offered_by_two_modules() {
        let flow = Flow::new();

        flow.scan();

        let all = flow.findings();
        let paths: BTreeSet<&String> = all.iter().map(|(_, path)| path).collect();
        assert_eq!(paths.len(), all.len(), "a path is offered once: {all:?}");
        let mut ours: Vec<(&str, &str)> = CHROME.to_vec();
        ours.extend(REDIS_INSIGHT);
        ours.push(EDITOR_CACHE);
        assert_eq!(of_module(&flow, "app-caches"), found(&ours));
        assert_eq!(of_module(&flow, "app-leftovers"), found(&[GONE, GONE_STATE]));
        assert_eq!(of_module(&flow, "dev-caches"), found(&[YARN]));
        assert_eq!(of_module(&flow, "jetbrains"), found(&[WEBSTORM]));
    }

    /// Flow 3: no finding's subject is, holds or lies inside what a profile or an app keeps beside its caches; a
    /// cleanup of every finding leaves those files byte for byte as they were.
    pub(crate) fn a_profile_s_and_an_app_s_data_are_never_offered() {
        let flow = Flow::new();
        flow.scan();
        let all = flow.findings();
        for (_, path) in &all {
            for data in DATA {
                assert!(!overlaps(Path::new(path), Path::new(data)), "{path} overlaps {data}");
            }
        }
        let kept = contents_of(&flow, &DATA);
        assert!(kept.iter().all(|files| !files.is_empty()));
        let categories: BTreeSet<&str> = all.iter().map(|(category, _)| category.as_str()).collect();
        let categories: Vec<&str> = categories.into_iter().collect();
        let view = flow.engine.prepare_cleanup(&selection(&categories)).unwrap();
        assert_eq!(usize::try_from(view.summary.count).unwrap(), all.len());

        flow.clean(&view);

        assert_eq!(contents_of(&flow, &DATA), kept, "the data beside the caches is untouched");
        assert!(trashed(&flow) > 0, "the cleanup moved the findings");
    }

    /// Flow 4: Chrome open: its three caches are left out `app-open`, Redis Insight's are still offered and every
    /// other answer is unchanged; it quits, and a rescan offers them.
    pub(crate) fn an_open_browser_s_caches_are_left_out_until_it_quits() {
        let running = chrome_open().then(Vec::new());
        let flow = Flow::with(Setup { running, changed: vec![Vec::new()], ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), without_chrome());
        assert_eq!(left_out(&events), [("browsers".to_owned(), "app-open".to_owned(), 3)]);

        let events = flow.rescan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
    }

    /// Flow 5: a file of a cache held open when the cleanup acts: that cache is skipped `in use`, nothing of it moves,
    /// the folder is unchanged; every other cache is emptied.
    pub(crate) fn an_entry_held_open_skips_the_cache() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&selection(&CATEGORIES)).unwrap();
        let held_before = contents_of(&flow, &[CHROME_CACHE]);
        let rest: Vec<&str> = OURS.iter().copied().filter(|path| *path != CHROME_CACHE).collect();
        let rest_before = contents_of(&flow, &rest);
        flow.open_files.hold(flow.path("home/Library/Caches/Google/Chrome/Default/Cache/Cache_Data/index"), "Chrome");

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let skipped = ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Chrome".into() } };
        let mut expected: Vec<(String, ItemOutcome)> = rest
            .iter()
            .map(|path| ((*path).to_owned(), ItemOutcome::Emptied { moved: 1, skipped: 0 }))
            .chain([(CHROME_CACHE.to_owned(), skipped)])
            .collect();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(outcomes, expected);
        assert_eq!(contents_of(&flow, &[CHROME_CACHE]), held_before, "nothing of the held cache moved");
        assert!(rest.iter().all(|path| entries(&flow, path) == 0), "the other caches are emptied");
        assert_eq!(trashed(&flow), rest.len());
        assert_eq!(trashed_bytes(&flow), sorted_bytes(&rest_before), "only the other caches' files");
    }

    /// Flow 6: the storage map counts Chrome's three caches in Chrome's line and Redis Insight's two in its line, each
    /// in its category.
    pub(crate) fn the_map_counts_the_caches_in_their_app_s_line() {
        let flow = Flow::new();
        flow.scan();

        let groups = flow.engine.storage_meaning(0).unwrap();

        let line = |bundle_id: &str| {
            groups
                .iter()
                .find_map(|group| match group {
                    MeaningGroup::App { app, sizes, categories, .. } if app.id.as_str() == bundle_id => {
                        let shares: Vec<(String, u64)> = categories
                            .iter()
                            .map(|share| (share.category.as_str().to_owned(), share.sizes.logical.bytes()))
                            .collect();
                        Some((app.name.clone(), app.installed, shares, sizes.logical.bytes()))
                    }
                    _ => None,
                })
                .expect("the installed app has a line")
        };
        let chrome = bytes_of(&flow, &[CHROME_CACHE, CHROME_CODE_CACHE, CHROME_PROFILE_CACHE]);
        let redis_insight = bytes_of(&flow, &[REDIS_INSIGHT_CACHE, REDIS_INSIGHT_GPU_CACHE]);
        assert_eq!(line(CHROME_ID), ("Google Chrome".into(), true, vec![(BROWSERS.to_owned(), chrome)], chrome));
        assert_eq!(
            line(REDIS_INSIGHT_ID),
            ("Redis Insight".into(), true, vec![(WEB_ENGINE.to_owned(), redis_insight)], redis_insight)
        );
    }
}

mod negative {
    use crate::flows::{
        a_profile_s_and_an_app_s_data_are_never_offered as data_never_offered, an_entry_held_open_skips_the_cache,
        an_open_browser_s_caches_are_left_out_until_it_quits, no_folder_is_offered_by_two_modules as offered_once,
    };

    #[test]
    fn no_folder_is_offered_by_two_modules() {
        offered_once();
    }

    #[test]
    fn a_profile_s_and_an_app_s_data_are_never_offered() {
        data_never_offered();
    }

    #[test]
    fn an_open_browser_s_caches_are_left_out_and_offered_once_it_quits() {
        an_open_browser_s_caches_are_left_out_until_it_quits();
    }

    #[test]
    fn an_entry_held_open_skips_the_cache_and_moves_nothing_of_it() {
        an_entry_held_open_skips_the_cache();
    }
}

mod positive {
    use crate::flows::{scan_confirm_empty_and_put_back, the_map_counts_the_caches_in_their_app_s_line as map_lines};
    use crate::support::check_golden;

    #[test]
    fn a_confirmed_cleanup_empties_each_cache_keeps_it_and_put_back_restores_every_entry() {
        check_golden("app-caches-browsers-flow", &scan_confirm_empty_and_put_back());
    }

    #[test]
    fn the_map_counts_the_caches_in_their_app_s_line() {
        map_lines();
    }
}
