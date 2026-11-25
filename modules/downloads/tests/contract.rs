//! The `downloads` module checked through the module kit: the category contract and its catalogue files, the use of its
//! texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeInstallRecords, FakeInstalledApps, FakeRunningApps, ModuleRun};

use crate::support::{System, kept, kept_apps, kept_with, not_claimed, offered, offered_with};

/// The run of every home with its own system, of the kept home with each port failing, and of the
/// offered home handed in reverse, with Downloads avoided and with a macOS installer not read in full.
fn every_run() -> Vec<ModuleRun> {
    let runs = [
        not_claimed(),
        kept(),
        kept_with(System::kept().running(FakeRunningApps::failing("no process list"))),
        kept_with(System::kept().apps(FakeInstalledApps::partial(kept_apps(), &["/Applications/Broken.app"]))),
        kept_with(System::kept().apps(FakeInstalledApps::failing("no app list"))),
        kept_with(System::kept().installs(FakeInstallRecords::default().failing_version("no version"))),
        offered(),
        offered_with(System::offered().reversed()),
        offered_with(System::offered().avoiding("Downloads")),
        offered_with(System::offered().avoiding("Downloads/Install macOS Sonoma.app/Contents")),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::downloads;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(
            left_out_kinds(&downloads()).for_now,
            ["no-system", "apps-unread", "browser-open", "recent", "not-older", "scan-again"]
        );
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::downloads;

    #[test]
    fn the_downloads_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&downloads(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_downloads_text_is_used() {
        assert_texts_used(&downloads(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = downloads();
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
        assert_eq!(fresher_testing::left_out_kinds(&downloads()).kept, ["newer-macos", "special", "not-yours"]);
    }
}
