//! The stale-projects module end to end: a fixture sandbox's home with `Code/` holding active, committed, stale and
//! recently built projects of several kinds and `Documents/` outside every project, every folder dated; the real
//! scanner (its reader seen through the facts lens, the module asking its tree after the walk), the modules the app
//! registers, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching only inside the
//! sandbox, with a fake Trash. The change journal's answers are a fake a flow sets. The flows of the design's
//! section 4, "Core e2e"; flow 3's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test stale_projects` rewrites
//! `fixtures/golden/stale-projects-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "stale_projects_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use fresher_core::{
        Descent, Event, IgnoreTarget, IgnoredEntry, ItemOutcome, MeaningGroup, PartKind, PlaceView, PutBackOutcome,
        ScanKind,
    };
    use serde_json::{Value, json};

    use crate::flow::{Flow, MODULE, Setup, left_out};
    use crate::tree::contents;

    const DEPENDENCIES: &str = "stale-projects.dependencies";
    const BUILD_OUTPUT: &str = "stale-projects.build-output";
    const PYTHON: &str = "stale-projects.python-environments";
    const CACHE: &str = "stale-projects.rust-build-cache";
    const DAILY_CACHE: &str = "home/Code/daily/target/debug/incremental";

    /// What the first scan offers, in the order `Flow::findings` sorts them: `tool`'s `target` (its incremental
    /// folder not offered again), the three stale `node_modules` (`shop`'s two, `site`'s), `py`'s environment and
    /// `daily`'s incremental cache.
    const OFFERED: [(&str, &str); 6] = [
        (BUILD_OUTPUT, "home/Code/tool/target"),
        (DEPENDENCIES, "home/Code/shop/node_modules"),
        (DEPENDENCIES, "home/Code/shop/packages/web/node_modules"),
        (DEPENDENCIES, "home/Code/site/node_modules"),
        (PYTHON, "home/Code/py/.venv"),
        (CACHE, DAILY_CACHE),
    ];
    /// What is the user's and never offered: source, git folders, framework output, marker files, the documents, and
    /// the rest of `daily`'s `target` (only its incremental cache is offered, never the whole `target`).
    const THE_USERS: [&str; 17] = [
        "home/Code/active",
        "home/Code/committed",
        "home/Code/older",
        "home/Code/daily/src",
        "home/Code/daily/Cargo.toml",
        "home/Code/daily/target/debug/deps",
        "home/Code/daily/target/llvm-cov-target",
        "home/Code/py/pyproject.toml",
        "home/Code/shop/package.json",
        "home/Code/shop/packages/web/package.json",
        "home/Code/site/dist",
        "home/Code/site/src",
        "home/Code/site/package.json",
        "home/Code/tool/.git",
        "home/Code/tool/src",
        "home/Code/tool/Cargo.toml",
        "home/Documents",
    ];

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
    fn ours(flow: &Flow) -> Vec<(String, String)> {
        flow.findings().into_iter().filter(|(category, _)| category.starts_with("stale-projects.")).collect()
    }

    fn contents_of(flow: &Flow, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    fn total_bytes(flow: &Flow, paths: &[&str]) -> u64 {
        contents_of(flow, paths).iter().flat_map(BTreeMap::values).map(|bytes| bytes.len() as u64).sum()
    }

    /// The categories of the overview's totals, sorted.
    fn totals(flow: &Flow) -> Vec<String> {
        let mut totals: Vec<String> =
            flow.engine.overview().unwrap().totals.iter().map(|total| total.category.as_str().to_owned()).collect();
        totals.sort();
        totals
    }

    /// The place `names` below the scan root, each step a folder the user opened.
    fn place(flow: &Flow, names: &[&str]) -> PlaceView {
        let engine = &flow.engine;
        let root = engine.storage_map().unwrap().volumes[0].roots()[0];
        let mut view = engine.storage_place(root, Descent::Stay, 0, 100).unwrap();
        for name in names {
            let row = view.rows.iter().find(|row| row.node.name == *name).expect("the place has the row");
            view = engine.storage_place(row.node.id, Descent::Stay, 0, 100).unwrap();
        }
        view
    }

    /// The names of the children `place` lists, sorted.
    fn children(view: &PlaceView) -> Vec<&str> {
        let mut names: Vec<&str> = view.rows.iter().map(|row| row.node.name.as_str()).collect();
        names.sort_unstable();
        names
    }

    fn scan_kind(events: &[Event]) -> Option<&ScanKind> {
        events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        })
    }

    /// Whether `left_out` holds `category` left out for `reason`: its items and logical bytes.
    fn kept(events: &[Event], category: &str, reason: &str) -> Option<(u32, u64)> {
        left_out(events)
            .into_iter()
            .find(|(of, why, ..)| of == category && why == reason)
            .map(|(_, _, items, bytes)| (items, bytes))
    }

    /// Flow 1: `active/` changed 3 days ago: its `node_modules` is left out `recent`, nothing of it is offered, and a
    /// cleanup of everything offered leaves its source, its `.git` and its `node_modules` as they were.
    pub(crate) fn nothing_of_an_active_project_is_offered() {
        let flow = Flow::new();

        let events = flow.scan();

        assert!(ours(&flow).iter().all(|(_, path)| !path.starts_with("home/Code/active")), "{:?}", ours(&flow));
        // `active` (4096 bytes), `committed` (8192) and `older` (16384): the three kept `node_modules`.
        assert_eq!(kept(&events, "dependencies", "recent"), Some((3, 28672)));
        let active = contents_of(&flow, &["home/Code/active"]);
        let view = flow.engine.prepare_cleanup(&flow.everything()).unwrap();
        let (_, cleaned) = flow.clean(&view);
        assert_eq!(flow.outcomes(&view, &cleaned).len(), 6);
        assert_eq!(contents_of(&flow, &["home/Code/active"]), active, "src, .git and node_modules stay");
        assert!(flow.path("home/Code/active/node_modules/x/index.js").is_file());
    }

    /// Flow 2: `committed/`'s files and folders are 400 days old, its branch moved 3 days ago: its `node_modules` is
    /// left out `recent` and stays on disk.
    pub(crate) fn a_recent_commit_keeps_a_project_active() {
        let flow = Flow::new();

        let events = flow.scan();

        assert!(ours(&flow).iter().all(|(_, path)| !path.starts_with("home/Code/committed")));
        // 8192 bytes of `committed` among the three kept.
        assert_eq!(kept(&events, "dependencies", "recent"), Some((3, 28672)));
        assert!(flow.path("home/Code/committed/node_modules/x/index.js").is_file());
    }

    /// Flow 3: the first scan offers every category; the report is the golden file.
    pub(crate) fn the_first_scan_offers_every_category() -> Value {
        let flow = Flow::new();

        let events = flow.scan();

        assert_eq!(ours(&flow), everything());
        // `daily`: `deps` (16384) and the fresh coverage build (8192) are left in its `target`, less the offered
        // cache's 4096; the coverage build alone is kept `cache-recent`.
        assert_eq!(kept(&events, "build-output", "recent"), Some((1, 24576)));
        assert_eq!(kept(&events, "rust-build-cache", "cache-recent"), Some((1, 8192)));
        assert_eq!(
            left_out(&events)
                .iter()
                .map(|(category, reason, ..)| (category.as_str(), reason.as_str()))
                .collect::<Vec<_>>(),
            [("build-output", "recent"), ("dependencies", "recent"), ("rust-build-cache", "cache-recent")]
        );
        let page = flow
            .engine
            .findings(&fresher_core::FindingsQuery::new(None, None, fresher_core::FindingOrder::Found, 0, 1000))
            .unwrap();
        let mut entries: Vec<Value> = page
            .findings
            .iter()
            .filter(|finding| finding.module().as_str() == MODULE)
            .map(|finding| {
                let value = serde_json::to_value(finding).unwrap();
                json!({
                    "category": value["category"],
                    "path": value["removal"]["value"].as_str().map(|path| path.replace(&format!("{}/", flow.fixture.display()), "")),
                    "danger": value["danger"],
                    "title": value["title"],
                })
            })
            .collect();
        entries.sort_by_key(Value::to_string);
        json!({ "offered": entries, "leftOut": left_out(&events) })
    }

    /// Flow 4: every offered item selected and the plan names Medium (the Python environment); the items go to the
    /// Trash and the journal and come back byte for byte; the user's source, `.git`, `dist`, marker files and the
    /// documents stay untouched.
    pub(crate) fn scan_confirm_trash_and_put_back() {
        let flow = Flow::new();
        flow.scan();
        assert_eq!(ours(&flow), everything());
        let offered: Vec<&str> = OFFERED.iter().map(|(_, path)| *path).collect();
        let before = contents_of(&flow, &offered);
        assert!(before.iter().all(|files| !files.is_empty()));
        let kept = contents_of(&flow, &THE_USERS);

        let view = flow.engine.prepare_cleanup(&flow.everything()).unwrap();

        assert_eq!(view.summary.count, 6);
        assert_eq!(view.summary.highest, fresher_core::DangerLevel::Medium);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let mut planned = offered.clone();
        planned.sort_unstable();
        assert_eq!(outcomes.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(), planned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(planned.iter().all(|path| !flow.path(path).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 6);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 6);
        assert_eq!(contents_of(&flow, &THE_USERS), kept, "the user's files stay");
        assert!(flow.path("home/Code/daily/target/debug").is_dir(), "the kept target stays");

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(contents_of(&flow, &offered), before, "put back byte for byte");
        assert_eq!(contents_of(&flow, &THE_USERS), kept);
    }

    /// Flow 5: `site/` is offered; a file added to `site/src` (its folder's time now) and the journal naming it: the
    /// rescan keeps `site`'s `node_modules` as `recent`.
    pub(crate) fn a_change_in_a_stale_project_keeps_it() {
        let flow = Flow::with(&Setup { changed: vec![vec!["home/Code/site/src"]] });
        flow.scan();
        assert!(ours(&flow).contains(&(DEPENDENCIES.to_owned(), "home/Code/site/node_modules".to_owned())));
        fs::write(flow.path("home/Code/site/src/new.ts"), "export {};\n").unwrap();

        let events = flow.rescan();

        assert!(matches!(scan_kind(&events), Some(ScanKind::Rescan { .. })));
        let without: Vec<(&str, &str)> =
            OFFERED.iter().copied().filter(|(_, path)| *path != "home/Code/site/node_modules").collect();
        assert_eq!(ours(&flow), found(&without));
        // `site` (4096) joins the three kept before.
        assert_eq!(kept(&events, "dependencies", "recent"), Some((4, 32768)));
    }

    /// Flow 6: `older/` (300 days) is `recent` at the default 365 days; the setting moved to 200, the next scan offers
    /// it.
    pub(crate) fn the_setting_moves_the_days() {
        let flow = Flow::new();
        let older = (DEPENDENCIES.to_owned(), "home/Code/older/node_modules".to_owned());

        flow.scan();

        assert!(!ours(&flow).contains(&older));

        flow.set_days("stale-projects.unused-for-days", 200);
        let events = flow.scan();

        assert!(ours(&flow).contains(&older), "{:?}", ours(&flow));
        // `active` and `committed` stay.
        assert_eq!(kept(&events, "dependencies", "recent"), Some((2, 12288)));
    }

    /// Flow 7: after a scan, a restore (`ScanKind::Restored`) offers nothing: the facts can be older than the disk,
    /// so every item is left out `scan-again`.
    pub(crate) fn a_restore_at_launch_offers_nothing() {
        let flow = Flow::new();
        flow.scan();
        assert_eq!(ours(&flow), everything());

        let events = flow.restore();

        assert!(matches!(scan_kind(&events), Some(ScanKind::Restored { .. })));
        assert_eq!(ours(&flow), []);
        let reasons: Vec<String> = left_out(&events).into_iter().map(|(_, reason, ..)| reason).collect();
        assert!(!reasons.is_empty() && reasons.iter().all(|reason| reason == "scan-again"), "{reasons:?}");
        let items: u32 = left_out(&events).iter().map(|(.., items, _)| items).sum();
        assert!(items >= 6, "every item the scan offered is kept: {:?}", left_out(&events));
    }

    /// Flow 8: an offered `node_modules` is in the explained part; `tool/src` is among the user's folders; the browse
    /// view opens `tool/`, `tool/target/debug/incremental`, `daily/target/` and `Documents/build`, listing their
    /// children: nothing of the module is claimed whole.
    pub(crate) fn the_storage_map_explains_the_artifacts_and_keeps_the_rest_open() {
        let flow = Flow::new();
        flow.scan();

        let map = flow.engine.storage_map().unwrap();
        let groups = flow.engine.storage_meaning(0).unwrap();

        let offered: Vec<&str> = OFFERED.iter().map(|(_, path)| *path).collect();
        let explained = map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained).unwrap();
        assert!(explained.sizes.logical.bytes() >= total_bytes(&flow, &["home/Code/shop/node_modules"]));
        assert!(explained.sizes.logical.bytes() <= total_bytes(&flow, &offered));
        let dependencies = groups.iter().find_map(|group| match group {
            MeaningGroup::Category(share) if share.category.as_str() == DEPENDENCIES => Some(share),
            _ => None,
        });
        assert_eq!(
            dependencies.map(|share| share.sizes.logical.bytes()),
            Some(total_bytes(
                &flow,
                &[
                    "home/Code/shop/node_modules",
                    "home/Code/shop/packages/web/node_modules",
                    "home/Code/site/node_modules"
                ]
            ))
        );

        let tool = place(&flow, &["Code", "tool"]);
        assert_eq!(children(&tool), [".git", "src", "target"]);
        let owner = |view: &PlaceView, name: &str| {
            view.rows
                .iter()
                .find(|row| row.node.name == name)
                .and_then(|row| row.node.owner)
                .map(|id| id.as_str().to_owned())
        };
        assert_eq!(owner(&tool, "src"), None);
        assert_eq!(owner(&tool, "target"), Some(BUILD_OUTPUT.to_owned()));
        let incremental = place(&flow, &["Code", "tool", "target", "debug", "incremental"]);
        assert_eq!(children(&incremental), ["x-1"]);
        let daily = place(&flow, &["Code", "daily", "target"]);
        assert_eq!(children(&daily), ["debug", "llvm-cov-target"]);
        let build = place(&flow, &["Documents", "build"]);
        assert_eq!(children(&build), ["drafts"]);
        assert_eq!(owner(&place(&flow, &["Code", "daily"]), "target"), None);
    }

    /// Flow 9: the build output ignored: its finding is in no total while the others stay; restoring brings it back
    /// without a scan.
    pub(crate) fn ignoring_the_build_output_keeps_it_out_of_every_total() {
        let flow = Flow::new();
        flow.scan();
        let all_totals = totals(&flow);
        assert!(all_totals.iter().any(|category| category == BUILD_OUTPUT));
        let found_bytes = |flow: &Flow| flow.engine.overview().unwrap().found.logical.bytes();
        let all_found = found_bytes(&flow);

        flow.engine.ignore(&IgnoreTarget::Category { category: BUILD_OUTPUT.into() }).unwrap();

        let without: Vec<(&str, &str)> = OFFERED.iter().copied().filter(|(of, _)| *of != BUILD_OUTPUT).collect();
        assert_eq!(ours(&flow), found(&without));
        assert_eq!(
            totals(&flow),
            all_totals.iter().filter(|category| *category != BUILD_OUTPUT).cloned().collect::<Vec<_>>()
        );
        assert!(found_bytes(&flow) < all_found);

        flow.engine.restore_ignored(&IgnoredEntry::Category { category: BUILD_OUTPUT.into() }).unwrap();

        assert_eq!(ours(&flow), everything());
        assert_eq!(totals(&flow), all_totals);
        assert_eq!(found_bytes(&flow), all_found);
    }

    /// Flow 10: `shop`'s `node_modules` alone to the Trash moves `shop`'s folder time: the next scan (a rescan, the
    /// journal reporting `shop`) keeps `packages/web/node_modules` as `recent`. The design's known risk, asserted so
    /// it stays known.
    pub(crate) fn a_stale_project_cleaned_once_looks_active_after() {
        let flow = Flow::with(&Setup { changed: vec![vec!["home/Code/shop"]] });
        flow.scan();
        let page = flow
            .engine
            .findings(&fresher_core::FindingsQuery::new(None, None, fresher_core::FindingOrder::Found, 0, 1000))
            .unwrap();
        let shop = page
            .findings
            .iter()
            .find(|finding| match finding.finding().subject() {
                fresher_core::Subject::Path(path) => path == &flow.path("home/Code/shop/node_modules"),
                fresher_core::Subject::External { .. } => false,
            })
            .expect("shop's node_modules is offered");
        let selection = serde_json::from_value(json!({ "findings": [shop.id()], "categories": [] })).unwrap();

        let view = flow.engine.prepare_cleanup(&selection).unwrap();
        let (_, cleaned) = flow.clean(&view);
        assert!(flow.outcomes(&view, &cleaned).iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved));
        assert!(!flow.path("home/Code/shop/node_modules").exists());

        let events = flow.rescan();

        let without: Vec<(&str, &str)> =
            OFFERED.iter().copied().filter(|(_, path)| !path.starts_with("home/Code/shop")).collect();
        assert_eq!(ours(&flow), found(&without));
        assert_eq!(kept(&events, "dependencies", "recent"), Some((4, 28672 + 4096)));
        assert!(flow.path("home/Code/shop/packages/web/node_modules").is_dir());
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn nothing_of_an_active_project_is_offered() {
        flows::nothing_of_an_active_project_is_offered();
    }

    #[test]
    fn a_recent_commit_keeps_a_project_active() {
        flows::a_recent_commit_keeps_a_project_active();
    }

    #[test]
    fn a_change_in_a_stale_project_keeps_it() {
        flows::a_change_in_a_stale_project_keeps_it();
    }

    #[test]
    fn a_restore_at_launch_offers_nothing() {
        flows::a_restore_at_launch_offers_nothing();
    }

    #[test]
    fn ignoring_the_build_output_keeps_it_out_of_every_total() {
        flows::ignoring_the_build_output_keeps_it_out_of_every_total();
    }

    #[test]
    fn a_stale_project_cleaned_once_looks_active_after() {
        flows::a_stale_project_cleaned_once_looks_active_after();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn the_first_scan_offers_every_category() {
        check_golden("stale-projects-flow", &flows::the_first_scan_offers_every_category());
    }

    #[test]
    fn scan_confirm_trash_and_put_back() {
        flows::scan_confirm_trash_and_put_back();
    }

    #[test]
    fn the_setting_moves_the_days() {
        flows::the_setting_moves_the_days();
    }

    #[test]
    fn the_storage_map_explains_the_artifacts_and_keeps_the_rest_open() {
        flows::the_storage_map_explains_the_artifacts_and_keeps_the_rest_open();
    }
}
