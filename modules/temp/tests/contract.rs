//! The `temp` module checked through the module kit: the category contract and its catalogue file, the use of its
//! texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeRunningApps, ModuleRun};

use crate::support::{System, old, recent, running, scan, special, system};

/// The run of every home with its default system, and of the old home with the systems that give the other left-out
/// reasons, and with a longer setting.
fn every_run() -> Vec<ModuleRun> {
    let runs = [
        recent(),
        system(),
        special(),
        running(),
        old(),
        scan("temp-old", System::plain().running(FakeRunningApps::failing("no process list"))),
        scan("temp-old", System::plain().stored(r#"{"temp.older-than-days": 14}"#)),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::temp;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&temp()).for_now, ["locked", "app-open", "recent", "no-system"]);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::temp;

    #[test]
    fn the_temp_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&temp(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_temp_text_is_used() {
        assert_texts_used(&temp(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = temp();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 7);
    }

    #[test]
    fn both_finding_reasons_are_given_by_a_fixture_run() {
        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.findings).map(|finding| finding.reason().key().as_str()).collect();

        assert_eq!(given, BTreeSet::from(["temp.finding.reason-shared", "temp.finding.reason-user"]));
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&temp()).kept, ["system", "special", "lock"]);
    }
}
