//! The `logs` module checked through the module kit: the category contract and its catalogue files, the use of its
//! texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FixtureTree, ModuleRun};

use crate::support::{System, kept, not_claimed, offered, scan};

/// The trees and runs of every home with its default system, and of the offered home with a longer setting.
fn every_scan() -> Vec<(FixtureTree, ModuleRun)> {
    vec![
        not_claimed(),
        kept(),
        offered(),
        scan("logs-offered", System::plain().stored(r#"{"logs.older-than-days": 14}"#)),
    ]
}

fn every_run() -> Vec<ModuleRun> {
    every_scan().into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::logs;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&logs()).for_now, ["locked", "recent"]);
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, Removal, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::support::{logs, subject_path};
    use crate::{every_run, every_scan};

    #[test]
    fn the_logs_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&logs(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_logs_text_is_used() {
        assert_texts_used(&logs(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = logs();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 4);
    }

    #[test]
    fn all_three_finding_reasons_are_given_by_a_fixture_run() {
        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.findings).map(|finding| finding.reason().key().as_str()).collect();

        assert_eq!(given, BTreeSet::from(["logs.reason.top", "logs.reason.folder", "logs.reason.report"]));
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    /// The subject's path is a regular file of the fixture tree, which the module can only have matched as a file.
    #[test]
    fn every_removal_moves_a_file_to_the_trash() {
        for (_tree, run) in every_scan() {
            for finding in &run.findings {
                let path = subject_path(finding);
                assert_eq!(finding.removal(), &Removal::MoveToTrash(path.to_path_buf()));
                assert!(std::fs::symlink_metadata(path).unwrap().file_type().is_file(), "{path:?}");
            }
        }
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&logs()).kept, ["special", "not-yours"]);
    }
}
