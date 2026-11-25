//! The `stale-projects` module checked through the module kit: the category contract and its catalogue files, the use
//! of its texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::ModuleRun;

use crate::support::{System, kept, not_claimed, offered, offered_with};

/// The run of every home, and of the offered home handed in reverse.
fn every_run() -> Vec<ModuleRun> {
    let runs = [not_claimed(), kept(), offered(), offered_with(System::plain().reversed())];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::stale_projects;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&stale_projects()).for_now, ["unread", "recent", "cache-recent"]);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::stale_projects;

    #[test]
    fn the_stale_projects_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");
        let module = stale_projects();

        assert_module_contract(&module, &Catalogue::Required(docs));
        assert_eq!(module.descriptor().categories().len(), 4);
    }

    #[test]
    fn every_stale_projects_text_is_used() {
        assert_texts_used(&stale_projects(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = stale_projects();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&stale_projects()).kept, Vec::<String>::new());
    }
}
