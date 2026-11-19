//! The `app-leftovers` module checked through the module kit: the category contract and its catalogue files, the use
//! of its texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeInstalledApps, FakeRunningApps, ModuleRun};

use crate::support::{
    System, apple, elsewhere, installed, kept, kept_with, leftovers, leftovers_with, not_ids, proof_absent, unproven,
};

/// The run of every home with its default system, of the kept home with each port failing, restored and with its
/// logs avoided, and of the leftovers handed in reverse and with the containers avoided.
fn every_run() -> Vec<ModuleRun> {
    let spotlight_off = FakeInstalledApps::default().with_elsewhere(elsewhere()).failing_search("index off");
    let runs = [
        not_ids(),
        apple(),
        unproven(),
        proof_absent(),
        installed(),
        kept(),
        kept_with(System::kept().running(FakeRunningApps::failing("no process list"))),
        kept_with(System::kept().apps(FakeInstalledApps::partial(Vec::new(), &["/Applications/Broken.app"]))),
        kept_with(System::kept().apps(FakeInstalledApps::failing("no app list"))),
        kept_with(System::kept().apps(spotlight_off)),
        kept_with(System::kept().restored()),
        kept_with(System::kept().avoiding("Library/Logs")),
        leftovers(),
        leftovers_with(System::plain().reversed()),
        leftovers_with(System::plain().avoiding("Library/Containers")),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::app_leftovers;

    #[test]
    fn the_app_leftovers_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&app_leftovers(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_app_leftovers_text_is_used() {
        assert_texts_used(&app_leftovers(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = app_leftovers();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 9);
    }

    #[test]
    fn the_finding_reason_and_every_title_are_given_by_a_fixture_run() {
        let runs = every_run();

        let findings = || runs.iter().flat_map(|run| &run.findings);
        let reasons: BTreeSet<&str> = findings().map(|finding| finding.reason().key().as_str()).collect();
        let titles: BTreeSet<&str> = findings().map(|finding| finding.title().key().as_str()).collect();

        assert_eq!(reasons, BTreeSet::from(["app-leftovers.finding.reason"]));
        assert_eq!(titles.len(), 9);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }
}
