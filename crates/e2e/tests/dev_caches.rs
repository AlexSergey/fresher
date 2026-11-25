//! The dev-caches module end to end: a fixture sandbox's home with npm's cache, Cargo's registry, Gradle's caches and
//! a derived data project, the real scanner (its reader seen through the facts lens), the `dev-caches` module, the
//! real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the sandbox, with a fake
//! Trash. Xcode is installed; the open apps, the files held open and the change journal's answers are fakes a flow
//! sets. The flows of the design's section 6, "Core e2e"; flow 1's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test dev_caches` rewrites `fixtures/golden/dev-caches-flow.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "dev_caches_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::{Path, PathBuf};

    use fresher_core::{
        Completeness, DangerLevel, IgnoreTarget, IgnoredEntry, ItemOutcome, MeaningGroup, ModuleGroup, PartKind,
        PutBackOutcome, SkipReason,
    };
    use fresher_testing::TreeReport;
    use serde_json::{Value, json};

    use crate::flow::{Flow, Setup, XCODE, left_out, select, xcode_open};
    use crate::tree::contents;

    const NPM: (&str, &str) = ("dev-caches.npm", "home/.npm/_cacache");
    const CARGO_CACHE: (&str, &str) = ("dev-caches.cargo", "home/.cargo/registry/cache");
    const CARGO_SRC: (&str, &str) = ("dev-caches.cargo", "home/.cargo/registry/src");
    const GRADLE: (&str, &str) = ("dev-caches.gradle", "home/.gradle/caches");
    const PROJECT: (&str, &str) =
        ("dev-caches.xcode-derived-data", "home/Library/Developer/Xcode/DerivedData/Demo-abcdefghijklmnopqrstuvwxyzab");
    const HOMEBREW: (&str, &str) = ("dev-caches.homebrew", "home/Library/Caches/Homebrew/downloads");
    /// The five caches of the flow home, in the order `Flow::findings` sorts them.
    const OFFERED: [(&str, &str); 5] = [CARGO_CACHE, CARGO_SRC, GRADLE, NPM, PROJECT];
    /// What is Cargo's and never a cache: its index, its installed programs and the user's credentials, and its tag.
    const CARGO_KEPT: [&str; 4] = [
        "home/.cargo/registry/index",
        "home/.cargo/registry/CACHEDIR.TAG",
        "home/.cargo/bin",
        "home/.cargo/credentials.toml",
    ];
    const GRADLE_TAG: &str = "home/.gradle/caches/CACHEDIR.TAG";
    /// The folder inside npm's cache that flow 5 makes unreadable.
    const NPM_INNER: &str = "home/.npm/_cacache/content-v2/sha512/ab";

    fn offered(caches: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            caches.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    /// The offered caches without `left`.
    fn offered_but(left: (&str, &str)) -> Vec<(String, String)> {
        offered(&OFFERED.into_iter().filter(|cache| *cache != left).collect::<Vec<_>>())
    }

    /// The categories of the flow home's caches, once each.
    fn categories() -> Vec<&'static str> {
        let mut categories: Vec<&str> = OFFERED.iter().map(|(category, _)| *category).collect();
        categories.sort_unstable();
        categories.dedup();
        categories
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    fn total_bytes(flow: &Flow, relative: &str) -> u64 {
        contents(&flow.path(relative)).values().map(|bytes| bytes.len() as u64).sum()
    }

    /// A folder made unreadable (mode 000) until the guard is dropped, whatever the flow's outcome.
    struct Locked {
        path: PathBuf,
    }

    impl Locked {
        fn new(path: &Path) -> Self {
            fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
            Self { path: path.to_path_buf() }
        }
    }

    impl Drop for Locked {
        fn drop(&mut self) {
            // Restored even while a failed assertion unwinds, so the temporary folder can be removed after.
            let _restored = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o755));
        }
    }

    /// Flow 1: scan, review, confirm to the Trash, put back. Cargo's own files stay.
    pub(crate) fn scan_confirm_trash_and_put_back() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&OFFERED));
        assert_eq!(left_out(&events), []);
        let module = flow.engine.modules().into_iter().find(|module| module.id.as_str() == "dev-caches").unwrap();
        assert_eq!(module.group, ModuleGroup::Development);
        let before: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert!(before.iter().all(|files| !files.is_empty() && files.values().all(|bytes| !bytes.is_empty())));
        let kept: Vec<_> = CARGO_KEPT.iter().map(|path| contents(&flow.path(path))).collect();

        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();

        assert_eq!(view.summary.count, 5);
        assert_eq!(view.summary.highest, DangerLevel::Low);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert_eq!(outcomes.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(), OFFERED.map(|(_, path)| path));
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(OFFERED.iter().all(|(_, path)| !flow.path(path).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 5);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 5);
        assert_eq!(CARGO_KEPT.map(|path| contents(&flow.path(path))).to_vec(), kept, "Cargo's own files stay");
        assert_eq!(flow.findings(), []);
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert_eq!(after, before, "put back byte for byte");
        assert_eq!(CARGO_KEPT.map(|path| contents(&flow.path(path))).to_vec(), kept, "Cargo's own files stay");
        let report = json!({
            "findings": offered(&OFFERED),
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

    /// Flow 2: a file of Gradle's caches held open when the cleanup acts: Gradle's caches are skipped, the rest go.
    pub(crate) fn a_cache_held_open_is_skipped_in_use() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();
        let held = flow.path("home/.gradle/caches/modules-2/modules-2.lock");
        flow.open_files.hold(&held, "java");

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let expected: Vec<(String, ItemOutcome)> = OFFERED
            .iter()
            .map(|cache| {
                let outcome = if *cache == GRADLE {
                    ItemOutcome::Skipped { reason: SkipReason::InUse { app: "java".into() } }
                } else {
                    ItemOutcome::Moved
                };
                (cache.1.to_owned(), outcome)
            })
            .collect();
        assert_eq!(outcomes, expected);
        assert!(held.is_file(), "Gradle's caches stay");
        assert!(OFFERED.iter().filter(|cache| **cache != GRADLE).all(|(_, path)| !flow.path(path).exists()));
    }

    /// Flow 3: with Xcode open the project is left out `app-open`; Xcode quits, and a rescan offers it. The caches
    /// whose sign is in a parent folder (Cargo's) are still offered after the rescan.
    pub(crate) fn derived_data_waits_while_xcode_is_open() {
        let flow = Flow::with(Setup { running: vec![xcode_open(), Vec::new()], changed: vec![Vec::new()] });

        let events = flow.scan();

        assert_eq!(flow.findings(), offered_but(PROJECT));
        let app_open = (PROJECT.0.to_owned(), "dev-caches.left-out.app-open".to_owned(), 1);
        assert_eq!(left_out(&events), [app_open]);

        let events = flow.rescan();

        assert_eq!(flow.findings(), offered(&OFFERED));
        assert_eq!(left_out(&events), []);
    }

    /// Flow 4: Gradle's tag is read again by every rescan, never kept from the cache: invalid, it is not claimed;
    /// rewritten valid, a rescan offers Gradle's caches; removed, a rescan claims nothing of Gradle's. Homebrew's
    /// downloads and Cargo's two folders, whose sign is in a parent folder, stay offered through each rescan, whether
    /// the journal names the parent or the folder: the scanner hands each sign place once.
    pub(crate) fn the_sign_is_read_again_by_a_rescan() {
        let flow = Flow::with(Setup {
            running: Vec::new(),
            changed: vec![
                vec!["home/.gradle/caches", "home/.cargo/registry", "home/Library/Caches/Homebrew"],
                vec!["home/.gradle/caches", "home/.cargo/registry/cache", HOMEBREW.1],
            ],
        });
        fs::create_dir_all(flow.path("home/Library/Caches/Homebrew/api")).unwrap();
        fs::write(flow.path("home/Library/Caches/Homebrew/api/formula.jws.json"), [1_u8; 64]).unwrap();
        fs::create_dir_all(flow.path(HOMEBREW.1)).unwrap();
        fs::write(flow.path(HOMEBREW.1).join("0123abcd--x-1.0.bottle.tar.gz"), [2_u8; 512]).unwrap();
        let tag = flow.path(GRADLE_TAG);
        let valid = fs::read(&tag).unwrap();
        fs::write(&tag, [0_u8; 43]).unwrap();
        let with_homebrew = |caches: &[(&'static str, &'static str)]| {
            offered(&caches.iter().copied().chain([HOMEBREW]).collect::<Vec<_>>())
        };
        let without_gradle: Vec<_> = OFFERED.into_iter().filter(|cache| *cache != GRADLE).collect();

        let events = flow.scan();

        assert_eq!(flow.findings(), with_homebrew(&without_gradle));
        assert_eq!(left_out(&events), []);

        fs::write(&tag, &valid).unwrap();
        let events = flow.rescan();

        assert_eq!(flow.findings(), with_homebrew(&OFFERED));
        assert_eq!(left_out(&events), []);

        // Moved out of the home: only the cleaner deletes.
        fs::rename(&tag, flow.path("gradle-tag-moved-away")).unwrap();
        let events = flow.rescan();

        assert_eq!(flow.findings(), with_homebrew(&without_gradle));
        assert_eq!(left_out(&events), []);
    }

    /// Flow 5: a folder inside npm's cache that cannot be read: npm's cache is read in part, so it is not claimed;
    /// every other answer is the same. The guard gives the folder its mode back, whatever the outcome.
    pub(crate) fn a_cache_read_in_part_is_not_claimed() {
        let flow = Flow::new();
        let inner = flow.path(NPM_INNER);
        let unreadable = Locked::new(&inner);

        let events = flow.scan();

        assert_eq!(flow.findings(), offered_but(NPM));
        assert_eq!(left_out(&events), []);
        let state = flow.engine.disk_state().unwrap().unwrap();
        // A gap, a known issue: the scanner does not count an unreadable folder inside a whole-claimed match as not
        // read (the coverage stays empty). Assert it here once the scanner counts it.
        let npm = state.find(&flow.path(NPM.1)).and_then(|id| state.node(id)).unwrap();
        assert_eq!((npm.owner, npm.sizes.completeness), (None, Completeness::Partial), "{npm:?}");
        drop(unreadable);
        assert!(fs::read_dir(&inner).is_ok(), "the folder's mode is given back");
    }

    /// Flow 7: npm's category ignored leaves npm's finding out of every total while the others stay; the module
    /// ignored leaves nothing; restoring brings each back.
    pub(crate) fn ignoring_a_category_then_the_module() {
        let flow = Flow::new();
        flow.scan();
        let totals = |flow: &Flow| -> Vec<String> {
            let mut totals: Vec<String> =
                flow.engine.overview().unwrap().totals.iter().map(|total| total.category.as_str().to_owned()).collect();
            totals.sort();
            totals
        };
        let found = |flow: &Flow| flow.engine.overview().unwrap().found.logical.bytes();
        let all_found = found(&flow);
        assert_eq!(totals(&flow), categories());

        flow.engine.ignore(&IgnoreTarget::Category { category: NPM.0.into() }).unwrap();

        assert_eq!(flow.findings(), offered_but(NPM));
        assert_eq!(totals(&flow), categories().into_iter().filter(|category| *category != NPM.0).collect::<Vec<_>>());
        assert_eq!(found(&flow), all_found - total_bytes(&flow, NPM.1));
        flow.engine.restore_ignored(&IgnoredEntry::Category { category: NPM.0.into() }).unwrap();
        assert_eq!(flow.findings(), offered(&OFFERED));

        flow.engine.ignore(&IgnoreTarget::Module { module: "dev-caches".into() }).unwrap();

        assert_eq!(flow.findings(), []);
        assert_eq!(totals(&flow), Vec::<String>::new());
        assert_eq!(found(&flow), 0);
        flow.engine.restore_ignored(&IgnoredEntry::Module { module: "dev-caches".into() }).unwrap();
        assert_eq!(flow.findings(), offered(&OFFERED));
        assert_eq!(found(&flow), all_found);
    }

    /// Flow 8: the storage map explains the offered caches, and the project counts in Xcode's line.
    pub(crate) fn the_map_explains_the_caches() {
        let flow = Flow::new();
        flow.scan();
        let offered_bytes: u64 = OFFERED.iter().map(|(_, path)| total_bytes(&flow, path)).sum();

        let map = flow.engine.storage_map().unwrap();
        let groups = flow.engine.storage_meaning(0).unwrap();

        let explained = map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained).unwrap();
        assert_eq!(explained.sizes.logical.bytes(), offered_bytes);
        let xcode = groups
            .iter()
            .find_map(|group| match group {
                MeaningGroup::App { app, categories, .. } if app.id.as_str() == XCODE => Some(categories),
                _ => None,
            })
            .unwrap();
        let shares: Vec<(&str, u64)> =
            xcode.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        assert_eq!(shares, [(PROJECT.0, total_bytes(&flow, PROJECT.1))]);
    }
}

mod negative {
    use crate::flows::{
        a_cache_held_open_is_skipped_in_use, a_cache_read_in_part_is_not_claimed,
        derived_data_waits_while_xcode_is_open, ignoring_a_category_then_the_module,
        the_sign_is_read_again_by_a_rescan,
    };

    #[test]
    fn a_cache_held_open_is_skipped_in_use_and_the_rest_go() {
        a_cache_held_open_is_skipped_in_use();
    }

    #[test]
    fn derived_data_is_left_out_while_xcode_is_open_and_offered_by_a_rescan_once_it_quits() {
        derived_data_waits_while_xcode_is_open();
    }

    #[test]
    fn an_invalid_or_removed_tag_is_read_again_by_each_rescan() {
        the_sign_is_read_again_by_a_rescan();
    }

    #[test]
    fn a_cache_read_in_part_is_not_claimed_and_its_node_is_partial() {
        a_cache_read_in_part_is_not_claimed();
    }

    #[test]
    fn an_ignored_category_or_module_is_in_no_total_until_restored() {
        ignoring_a_category_then_the_module();
    }
}

mod positive {
    use crate::flows::{scan_confirm_trash_and_put_back, the_map_explains_the_caches};
    use crate::support::check_golden;

    #[test]
    fn the_caches_go_to_the_trash_and_come_back_byte_for_byte() {
        check_golden("dev-caches-flow", &scan_confirm_trash_and_put_back());
    }

    #[test]
    fn the_map_explains_the_caches_and_the_project_counts_in_xcodes_line() {
        the_map_explains_the_caches();
    }
}
