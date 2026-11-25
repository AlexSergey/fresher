//! The `unused-toolchains` module checked through the module kit: the category contract and its catalogue files, and
//! the texts and left-out reasons its fixture runs give.

#[cfg(test)]
mod support;

use fresher_testing::{MatchOrder, ModuleRun};

use crate::support::{Variant, kept, kept_unknown, not_claimed, offered, scan_as, switched_off};

/// The run of every home, and of the offered home handed in reverse.
fn every_run() -> Vec<ModuleRun> {
    let offered_as = |variant| scan_as("unused-toolchains-offered", variant).1;
    vec![
        not_claimed().1,
        kept().1,
        kept_unknown().1,
        offered().1,
        switched_off().1,
        offered_as(Variant { order: MatchOrder::Reversed, ..Variant::DEFAULT }),
    ]
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::unused_toolchains;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&unused_toolchains()).for_now, ["partial-scan", "default-unknown", "pin-unknown"]);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::unused_toolchains;

    #[test]
    fn the_unused_toolchains_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");
        let module = unused_toolchains();
        assert_eq!(module.descriptor().categories().len(), 2);

        assert_module_contract(&module, &Catalogue::Required(docs));
    }

    #[test]
    fn every_unused_toolchains_text_is_used() {
        assert_texts_used(&unused_toolchains(), &every_run());
    }

    /// `partial-scan` needs a folder the walk cannot read, which a fixture home cannot state: the module's unit tests
    /// give it.
    #[test]
    fn every_declared_left_out_reason_but_partial_scan_is_given_by_a_fixture_run() {
        let module = unused_toolchains();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        let unread: BTreeSet<&str> = BTreeSet::from(["unused-toolchains.left-out.partial-scan"]);
        assert_eq!(given, declared.difference(&unread).copied().collect());
        assert_eq!(declared.len(), 5);
    }

    #[test]
    fn both_finding_reasons_and_titles_are_given_by_a_fixture_run() {
        let runs = every_run();

        let findings = || runs.iter().flat_map(|run| &run.findings);
        let reasons: BTreeSet<&str> = findings().map(|finding| finding.reason().key().as_str()).collect();
        let titles: BTreeSet<&str> = findings().map(|finding| finding.title().key().as_str()).collect();

        assert_eq!(reasons, BTreeSet::from(["unused-toolchains.reason.node", "unused-toolchains.reason.rust"]));
        assert_eq!(titles, BTreeSet::from(["unused-toolchains.title.node", "unused-toolchains.title.rust"]));
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&unused_toolchains()).kept, ["special", "not-yours"]);
    }
}
