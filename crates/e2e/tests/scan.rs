//! The engine with the real scanner on fixture homes: the walk, a module with interests of every pattern, the claims,
//! the committed state. The portable reader runs anywhere; every root is a temporary fixture tree.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test scan` rewrites `fixtures/golden/scan.json`.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "app/mod.rs"]
mod app;

mod negative {
    use std::sync::Arc;

    use fresher_core::{Event, NotScanned};
    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, scan};

    #[test]
    fn an_avoided_folder_is_reported_as_not_scanned() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));

        scan(&home.engine);

        let state = home.engine.disk_state().unwrap().unwrap();
        let places = state.unreadable(10);
        assert_eq!(places.len(), 1);
        assert_eq!((places[0].node.name.as_str(), places[0].reason), ("Documents", NotScanned::Avoided));
        assert!(probe_findings(&home).iter().all(|(path, _)| !path.starts_with("Documents")));
    }

    #[test]
    fn the_scan_never_leaves_its_root() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));

        let events = scan(&home.engine);

        let found = probe_findings(&home);
        let link = found.iter().find(|(path, _)| path == "Library/Caches/system").unwrap();
        assert_eq!(link.1, 0, "a link is listed with no size and not followed");
        assert!(found.iter().all(|(path, _)| !path.starts_with('/')), "{found:?}");
        assert!(events.iter().any(|event| matches!(event, Event::ScanFinished { .. })));
        assert!(home.tree.root().exists());
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, report, scan};
    use crate::support::check_golden;

    #[test]
    fn a_fixture_home_scans_to_its_golden_report() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));

        let events = scan(&home.engine);

        check_golden("scan", &report(&home, &events));
        assert!(home.data.path().join("scan-1.bin").exists(), "the committed scan is cached");
    }

    #[test]
    fn the_overview_counts_nested_claims_once() {
        let home = home("scan-nested-claims.toml", Arc::new(FakeChangeJournal::default()));

        scan(&home.engine);

        let found = probe_findings(&home);
        let size_of = |wanted: &str| found.iter().find(|(path, _)| path == wanted).map(|(_, size)| *size).unwrap();
        assert_eq!(size_of("Library/Caches"), 0, "an item claim of a folder has no size of its own");
        assert_eq!(size_of("Library/Caches/app"), 5_000_000 + 3);
        let totals = home.engine.overview().unwrap().totals;
        let probe = totals.iter().find(|total| total.category.as_str() == "probe.found").unwrap();
        let all: u64 = found.iter().map(|(_, size)| size).sum();
        assert_eq!(probe.sizes.logical.bytes(), all, "every claimed path once, the outer folder adding nothing");
        let state = home.engine.disk_state().unwrap().unwrap();
        let library = state.children(state.roots()[0], fresher_core::ChildOrder::ByName, 0, 10).unwrap();
        let library = library.nodes.iter().find(|node| node.name == "Library").unwrap();
        let caches = &state.children(library.id, fresher_core::ChildOrder::ByName, 0, 10).unwrap().nodes[0];
        let app = &state.children(caches.id, fresher_core::ChildOrder::ByName, 0, 10).unwrap().nodes[0];
        assert_eq!((app.name.as_str(), app.owner.map(fresher_core::CategoryId::as_str)), ("app", Some("probe.found")));
        assert_eq!(caches.owner.map(fresher_core::CategoryId::as_str), Some("probe.found"));
    }
}
