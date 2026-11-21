//! The `app-caches` module checked through the module kit: the category contract and its catalogue file, the use of
//! its texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeInstalledApps, FakeRunningApps, ModuleRun};

use crate::support::{
    editor_system, kept, kept_system, kept_with, not_claimed, not_claimed_with, offered, offered_system, offered_with,
};

/// The run of every home with its default system, of the kept home with each port failing, and of the offered home
/// handed in reverse.
fn every_run() -> Vec<ModuleRun> {
    let runs = [
        not_claimed(),
        kept(),
        offered(),
        kept_with(kept_system().running(FakeRunningApps::failing("no process list"))),
        kept_with(kept_system().apps(FakeInstalledApps::failing("no app list"))),
        not_claimed_with(editor_system().apps(FakeInstalledApps::failing("no app list"))),
        offered_with(offered_system().reversed()),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::app_caches;

    #[test]
    fn the_app_caches_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&app_caches(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_app_caches_text_is_used() {
        assert_texts_used(&app_caches(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = app_caches();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 4);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }
}
