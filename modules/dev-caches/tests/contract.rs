//! The `dev-caches` module checked through the module kit: the category contract and its catalogue files, the use of
//! its texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeRunningApps, ModuleRun};

use crate::support::{System, caches, kept, kept_with, not_folders, scan, similar_names, unsigned};

/// The run of every home with its default system, of the kept home with the open apps failing, and of the caches
/// restored from the cache and handed in reverse.
fn every_run() -> Vec<ModuleRun> {
    let runs = [
        similar_names(),
        unsigned(),
        not_folders(),
        kept(),
        kept_with(FakeRunningApps::failing("no process list")),
        caches(),
        scan("dev-caches-caches", System::plain().restored()),
        scan("dev-caches-caches", System::plain().reversed()),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::dev_caches;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&dev_caches()).for_now, ["app-open", "no-system"]);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::dev_caches;

    #[test]
    fn the_dev_caches_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&dev_caches(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_dev_caches_text_is_used() {
        assert_texts_used(&dev_caches(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = dev_caches();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 4);
    }

    #[test]
    fn both_finding_reasons_and_every_title_are_given_by_a_fixture_run() {
        let runs = every_run();

        let findings = || runs.iter().flat_map(|run| &run.findings);
        let reasons: BTreeSet<&str> = findings().map(|finding| finding.reason().key().as_str()).collect();
        let titles: BTreeSet<&str> = findings().map(|finding| finding.title().key().as_str()).collect();

        assert_eq!(reasons, BTreeSet::from(["dev-caches.finding.reason-build", "dev-caches.finding.reason-download"]));
        assert_eq!(titles.len(), 10);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&dev_caches()).kept, ["not-yours", "special"]);
    }
}
