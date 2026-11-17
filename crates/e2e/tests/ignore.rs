//! The ignore list end to end: the real scanner on a fixture home, the real settings file in a temporary data folder.
//! Ignored findings leave every answer at once and come back on restore, without a rescan. A module and a group are
//! ignored on the cleanup home (the `trial` module of the group Debug, a build module of Development).

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "cleaning/mod.rs"]
#[expect(dead_code, reason = "the cleanup home is shared; the ignore flows use its scan and its selection only")]
mod cleaning;

#[cfg(test)]
#[path = "app/mod.rs"]
mod app;

#[cfg(test)]
mod hidden {
    use fresher_core::{HiddenBy, HiddenModule, PartKind, Sizes};

    use crate::cleaning::Home;

    /// The categories of the findings the engine answers, in order, without repeats.
    pub(crate) fn categories(home: &Home) -> Vec<String> {
        let mut found: Vec<String> =
            home.findings().iter().map(|finding| finding["category"].as_str().unwrap().to_owned()).collect();
        found.dedup();
        found
    }

    /// The categories of the overview's totals.
    pub(crate) fn totals(home: &Home) -> Vec<String> {
        home.engine.overview().unwrap().totals.iter().map(|total| total.category.as_str().to_owned()).collect()
    }

    /// What the storage map of the first volume explains with the findings.
    pub(crate) fn explained(home: &Home) -> Sizes {
        let map = home.engine.storage_map().unwrap();
        map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained).unwrap().sizes
    }

    /// The modules the overview says it hides, with why.
    pub(crate) fn hidden_modules(home: &Home) -> Vec<(String, HiddenBy)> {
        let hidden: Vec<HiddenModule> = home.engine.overview().unwrap().hidden_modules;
        hidden.iter().map(|one| (one.module.as_str().to_owned(), one.by)).collect()
    }
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{CommandError, FindingId, IgnoreTarget, ModuleGroup, Subject};
    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, relative, scan};
    use crate::cleaning::{Setup, home as cleanup_home};
    use crate::hidden::categories;

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
    #[test]
    fn a_module_nobody_registered_is_refused_and_the_settings_file_keeps_no_ignore_list() {
        let home = cleanup_home(Setup::DEFAULT);
        home.scan();

        let error = home.engine.ignore(&IgnoreTarget::Module { module: "ghost".into() });

        assert_eq!(error, Err(CommandError::UnknownModule { id: "ghost".into() }));
        let file = std::fs::read_to_string(home.data.path().join("settings.json")).unwrap();
        assert!(!file.contains("core.ignore"), "{file}");
        assert_eq!(home.engine.overview().unwrap().ignored.modules, 0);
    }

    #[test]
    fn a_cleanup_cannot_select_what_an_ignored_module_or_group_found() {
        for target in
            [IgnoreTarget::Module { module: "trial".into() }, IgnoreTarget::Group { group: ModuleGroup::Debug }]
        {
            let home = cleanup_home(Setup::DEFAULT);
            home.scan();
            let selection = home.selection();
            home.engine.ignore(&target).unwrap();

            let view = home.engine.prepare_cleanup(&selection).unwrap();

            let planned: Vec<&str> = view.groups.iter().map(|group| group.category.as_str()).collect();
            assert_eq!(planned, ["build.folders"], "{view:?}");
            assert_eq!(categories(&home), ["build.folders"]);
        }
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::{HiddenBy, IgnoreTarget, IgnoredEntry, ModuleGroup};
    use fresher_testing::FakeChangeJournal;

    use crate::app::{home, probe_findings, report, scan};
    use crate::cleaning::{Setup, home as cleanup_home, home_sharing};
    use crate::hidden::{categories, explained, hidden_modules, totals};
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

    #[test]
    fn ignoring_a_module_leaves_every_answer_and_restore_brings_it_back_without_a_rescan() {
        let home = cleanup_home(Setup::DEFAULT);
        home.scan();
        let before = (home.findings(), totals(&home), explained(&home));

        home.engine.ignore(&IgnoreTarget::Module { module: "trial".into() }).unwrap();

        assert!(explained(&home).logical < before.2.logical, "the storage map still explains what is ignored");
        assert_eq!(categories(&home), ["build.folders"]);
        assert_eq!(totals(&home), ["build.folders"]);
        assert_eq!(hidden_modules(&home), [("trial".to_owned(), HiddenBy::Module)]);
        assert_eq!(home.engine.overview().unwrap().ignored.modules, 1);
        let file = std::fs::read_to_string(home.data.path().join("settings.json")).unwrap();
        assert!(file.contains("core.ignore.modules"), "{file}");

        home.engine.restore_ignored(&IgnoredEntry::Module { module: "trial".into() }).unwrap();

        assert_eq!((home.findings(), totals(&home), explained(&home)), before);
        assert_eq!(hidden_modules(&home), []);
    }

    #[test]
    fn ignoring_a_group_hides_its_modules_and_a_new_engine_on_the_same_data_still_ignores_it() {
        let home = cleanup_home(Setup::DEFAULT);
        home.scan();
        let before = home.findings();

        home.engine.ignore(&IgnoreTarget::Group { group: ModuleGroup::Debug }).unwrap();
        home.engine.ignore(&IgnoreTarget::Module { module: "trial".into() }).unwrap();

        assert_eq!(categories(&home), ["build.folders"]);
        assert_eq!(hidden_modules(&home), [("trial".to_owned(), HiddenBy::Group)]);
        let overview = home.engine.overview().unwrap();
        assert_eq!((overview.ignored.groups, overview.ignored.modules), (1, 1));
        let again = home_sharing(Setup::DEFAULT, Arc::clone(&home.data));
        again.scan();
        assert_eq!(categories(&again), ["build.folders"]);
        assert_eq!(hidden_modules(&again), [("trial".to_owned(), HiddenBy::Group)]);

        again.engine.restore_ignored(&IgnoredEntry::Group { group: ModuleGroup::Debug }).unwrap();
        assert_eq!(hidden_modules(&again), [("trial".to_owned(), HiddenBy::Module)]);
        again.engine.restore_ignored(&IgnoredEntry::Module { module: "trial".into() }).unwrap();

        assert_eq!(again.findings().len(), before.len());
        assert_eq!(hidden_modules(&again), []);
    }
}
