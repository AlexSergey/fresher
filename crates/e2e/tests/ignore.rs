//! The ignore list end to end: the real scanner on a fixture home, the real settings file in a temporary data folder.
//! Ignored findings leave every answer at once and come back on restore, without a rescan.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "app/mod.rs"]
mod app;

mod negative {
    use std::sync::Arc;

    use fresher_core::{FindingId, IgnoreTarget, Subject};
    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, relative, scan};

    #[test]
    fn an_ignored_item_and_what_is_inside_it_leave_the_results_and_the_file_keeps_them() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));
        scan(&home.engine);
        let before = probe_findings(&home);
        let first = home.engine.finding(FindingId::new(1)).unwrap();
        let ignored = match first.finding().subject() {
            Subject::Path(path) => relative(&home.root, path),
            Subject::External { .. } => String::new(),
        };
        assert!(!ignored.is_empty(), "a probe finding has a path");

        home.engine.ignore(&IgnoreTarget::Finding { finding: first.id() }).unwrap();

        let after = probe_findings(&home);
        assert!(after.len() < before.len());
        assert!(after.iter().all(|(path, _)| !std::path::Path::new(path).starts_with(&ignored)), "{after:?}");
        let file = std::fs::read_to_string(home.data.path().join("settings.json")).unwrap();
        assert!(file.contains("core.ignore.paths"), "{file}");
    }

    #[test]
    fn an_ignored_category_leaves_the_overview() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));
        scan(&home.engine);

        home.engine.ignore(&IgnoreTarget::Category { category: "probe.found".into() }).unwrap();

        let overview = home.engine.overview().unwrap();
        assert_eq!(probe_findings(&home), []);
        assert!(overview.totals.iter().all(|total| total.category.as_str() != "probe.found"));
        assert_eq!(overview.ignored.categories, 1);
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::{IgnoreTarget, IgnoredEntry};
    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, report, scan};
    use crate::support::check_golden;

    #[test]
    fn a_fixture_home_with_an_ignored_category_gives_its_golden_report() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));
        let events = scan(&home.engine);

        home.engine.ignore(&IgnoreTarget::Category { category: "demo.caches".into() }).unwrap();

        check_golden("ignore", &report(&home, &events));
        assert!(home.tree.root().exists());
    }

    #[test]
    fn restore_brings_the_findings_back_without_a_rescan() {
        let home = home("scan-basic.toml", Arc::new(FakeChangeJournal::default()));
        scan(&home.engine);
        let before = probe_findings(&home);
        home.engine.ignore(&IgnoreTarget::Category { category: "probe.found".into() }).unwrap();

        home.engine.restore_ignored(&IgnoredEntry::Category { category: "probe.found".into() }).unwrap();

        assert_eq!(probe_findings(&home), before);
    }
}
