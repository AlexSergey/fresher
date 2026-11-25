//! The unused-toolchains module end to end: a fixture sandbox's home with rustup's settings (its default, a moving
//! channel, an old toolchain, a dated nightly, a toolchain a project's `rust-toolchain.toml` names) and nvm's (its
//! default alias, the newest Node version, an old one), the real scanner, the `unused-toolchains` module with its Node
//! switch on, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the
//! sandbox, with a fake Trash. The flows of the design's section 9; the cleanup flow's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test unused_toolchains` rewrites
//! `fixtures/golden/unused-toolchains-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "unused_toolchains_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use fresher_core::{DangerLevel, Event, ItemOutcome, PutBackOutcome, ScanKind, SkipReason};
    use fresher_testing::TreeReport;
    use serde_json::{Value, json};

    use crate::flow::{Flow, left_out, select};
    use crate::tree::contents;

    const NODE: (&str, &str) = ("unused-toolchains.node", "home/.nvm/versions/node/v14.21.3");
    const OLD_RUST: (&str, &str) = ("unused-toolchains.rust", "home/.rustup/toolchains/1.60.0-aarch64-apple-darwin");
    const NIGHTLY: (&str, &str) =
        ("unused-toolchains.rust", "home/.rustup/toolchains/nightly-2023-06-01-aarch64-apple-darwin");
    /// The three versions of the flow home, in the order `Flow::findings` sorts them.
    const OFFERED: [(&str, &str); 3] = [NODE, OLD_RUST, NIGHTLY];
    /// What is never offered: the default, the moving channel, the pinned toolchain, nvm's default and newest.
    const KEPT: [&str; 5] = [
        "home/.rustup/toolchains/stable-aarch64-apple-darwin",
        "home/.rustup/toolchains/nightly-aarch64-apple-darwin",
        "home/.rustup/toolchains/1.79.0-aarch64-apple-darwin",
        "home/.nvm/versions/node/v20.11.0",
        "home/.nvm/versions/node/v22.1.0",
    ];
    const PROJECT: &str = "home/Documents/dev/app";
    const NVM_DEFAULT: &str = "home/.nvm/alias/default";
    const SETTINGS: &str = "home/.rustup/settings.toml";
    const NODE_BINARY: &str = "home/.nvm/versions/node/v14.21.3/bin/node";

    fn offered(versions: &[(&str, &str)]) -> Vec<(String, String)> {
        versions.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect()
    }

    fn categories() -> Vec<&'static str> {
        let mut categories: Vec<&str> = OFFERED.iter().map(|(category, _)| *category).collect();
        categories.dedup();
        categories
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    fn scan_kind(events: &[Event]) -> Option<&ScanKind> {
        events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        })
    }

    fn kept_files(flow: &Flow) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        KEPT.iter().chain(&[SETTINGS, NVM_DEFAULT, PROJECT]).map(|path| contents(&flow.path(path))).collect()
    }

    /// The default, the moving channel, nvm's default and newest and the pinned toolchain are never offered.
    pub(crate) fn the_default_the_channel_and_the_pinned_toolchain_are_never_offered() {
        let flow = Flow::new();

        let events = flow.scan();

        let found = flow.findings();
        assert_eq!(found, offered(&OFFERED));
        assert!(found.iter().all(|(_, path)| !KEPT.contains(&path.as_str())), "{found:?}");
        assert_eq!(left_out(&events), []);
    }

    /// A `.nvmrc` written in the project after the first scan: the rescan keeps the version it names.
    pub(crate) fn a_pin_written_between_two_scans_keeps_its_version() {
        let flow = Flow::with(&[vec![PROJECT]]);
        flow.scan();
        assert_eq!(flow.findings(), offered(&OFFERED));
        fs::write(flow.path(PROJECT).join(".nvmrc"), "14\n").unwrap();

        let events = flow.rescan();

        assert!(matches!(scan_kind(&events), Some(ScanKind::Rescan { .. })));
        assert_eq!(flow.findings(), offered(&[OLD_RUST, NIGHTLY]));
    }

    /// The version's `bin/node` held open: the version is skipped in use, nothing of it moves, the others go.
    pub(crate) fn a_version_with_a_file_open_is_skipped_in_use() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();
        flow.open_files.hold(flow.path(NODE_BINARY), "node");
        let before = contents(&flow.path(NODE.1));

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let mut expected: Vec<(String, ItemOutcome)> = OFFERED
            .iter()
            .map(|(_, path)| {
                let outcome = if *path == NODE.1 {
                    ItemOutcome::Skipped { reason: SkipReason::InUse { app: "node".into() } }
                } else {
                    ItemOutcome::Moved
                };
                ((*path).to_owned(), outcome)
            })
            .collect();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(outcomes, expected);
        assert_eq!(contents(&flow.path(NODE.1)), before, "nothing of the version moved");
        assert!(!flow.path(OLD_RUST.1).exists() && !flow.path(NIGHTLY.1).exists());
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 2);
        let moved: Vec<String> = flow
            .journal()
            .iter()
            .filter(|record| record["type"] == "item")
            .filter_map(|record| Some(record["trashed"][0]["original"].as_str()?.to_owned()))
            .collect();
        assert_eq!(moved.len(), 2, "{moved:?}");
        assert!(moved.iter().all(|path| !path.ends_with("v14.21.3")), "{moved:?}");
    }

    /// Scan, review, confirm to the Trash, put back. The settings, the pins and the kept versions stay.
    pub(crate) fn a_confirmed_cleanup_moves_the_toolchain_folder_to_the_trash() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&OFFERED));
        assert_eq!(left_out(&events), []);
        let before: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert!(before.iter().all(|files| !files.is_empty()));
        let kept = kept_files(&flow);

        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();

        assert_eq!(view.summary.count, 3);
        assert_eq!(view.summary.highest, DangerLevel::Medium);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(OFFERED.iter().all(|(_, path)| !flow.path(path).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 3);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 3);
        assert_eq!(kept_files(&flow), kept, "the settings, the pins and the kept versions untouched");
        assert_eq!(flow.findings(), []);
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert_eq!(after, before, "put back byte for byte");
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

    /// The versions are offered per category with their sizes: the folder's files.
    pub(crate) fn the_scan_offers_each_unused_version_in_its_category_with_its_sizes() {
        let flow = Flow::new();
        flow.scan();

        let overview = flow.engine.overview().unwrap();

        let mut totals: Vec<(String, u64)> = overview
            .totals
            .iter()
            .map(|total| (total.category.as_str().to_owned(), total.sizes.logical.bytes()))
            .collect();
        totals.sort();
        // Node 14.21.3's binary (4000 bytes); the rust toolchains' compilers (1000 and 2000 bytes).
        let expected = [(NODE.0, 4000), (OLD_RUST.0, 3000)];
        assert_eq!(totals, expected.map(|(category, bytes)| (category.to_owned(), bytes)));
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn the_default_the_channel_and_the_pinned_toolchain_are_never_offered() {
        flows::the_default_the_channel_and_the_pinned_toolchain_are_never_offered();
    }

    #[test]
    fn a_pin_written_between_two_scans_keeps_its_version() {
        flows::a_pin_written_between_two_scans_keeps_its_version();
    }

    #[test]
    fn a_version_with_a_file_open_is_skipped_in_use() {
        flows::a_version_with_a_file_open_is_skipped_in_use();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn the_scan_offers_each_unused_version_in_its_category_with_its_sizes() {
        flows::the_scan_offers_each_unused_version_in_its_category_with_its_sizes();
    }

    #[test]
    fn a_confirmed_cleanup_moves_the_toolchain_folder_to_the_trash() {
        check_golden("unused-toolchains-flow", &flows::a_confirmed_cleanup_moves_the_toolchain_folder_to_the_trash());
    }
}
