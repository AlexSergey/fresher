//! The `app-caches` module checked through the module kit: the category contract and its catalogue file, the use of
//! its texts and left-out reasons, and its removals.

#[cfg(test)]
#[path = "support/plain.rs"]
mod plain;
#[cfg(test)]
mod support;
#[cfg(test)]
#[path = "support/tables.rs"]
mod tables;

use fresher_testing::{FakeInstalledApps, FakeRunningApps, ModuleRun};

use crate::plain::{
    editor_system, kept, kept_system, kept_with, not_claimed, not_claimed_with, offered, offered_system, offered_with,
};
use crate::tables::{
    tables_kept, tables_kept_system, tables_kept_with, tables_not_claimed, tables_offered, tables_offered_system,
    tables_offered_with,
};

/// The run of every home with its default system, of the kept homes with each port failing, of the tables' offered home
/// with the installed apps failing, and of the offered homes handed in reverse.
fn every_run() -> Vec<ModuleRun> {
    let runs = [
        not_claimed(),
        kept(),
        offered(),
        kept_with(kept_system().running(FakeRunningApps::failing("no process list"))),
        kept_with(kept_system().apps(FakeInstalledApps::failing("no app list"))),
        not_claimed_with(editor_system().apps(FakeInstalledApps::failing("no app list"))),
        offered_with(offered_system().reversed()),
        tables_not_claimed(),
        tables_kept(),
        tables_offered(),
        tables_kept_with(tables_kept_system().running(FakeRunningApps::failing("no process list"))),
        tables_kept_with(tables_kept_system().apps(FakeInstalledApps::failing("no app list"))),
        tables_offered_with(tables_offered_system().apps(FakeInstalledApps::failing("no app list"))),
        tables_offered_with(tables_offered_system().reversed()),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::app_caches;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&app_caches()).for_now, ["no-system", "app-open"]);
    }
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

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&app_caches()).kept, ["special", "not-yours"]);
    }
}
