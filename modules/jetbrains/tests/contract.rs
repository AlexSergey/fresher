//! The `jetbrains` module checked through the module kit: the category contract and its catalogue files, the use of its
//! texts and left-out reasons, and its removals.

#[cfg(test)]
mod support;

use fresher_testing::{FakeInstalledApps, ModuleRun};

use crate::support::{
    System, app, current, open, running, scan, similar_names, stale, uninstalled, uninstalled_fresh, uninstalled_with,
    webstorm,
};

/// The run of every home with its default apps, and of the homes with the apps that give the other left-out reasons.
fn every_run() -> Vec<ModuleRun> {
    let mut no_version = webstorm();
    no_version.version = None;
    let mut elsewhere = app("com.jetbrains.datagrip", "DataGrip", "2025.3");
    elsewhere.path = "/Volumes/Drive/DataGrip.app".into();
    let longer = System::installed(Vec::new()).stored(r#"{"jetbrains.uninstalled-after-days": 60}"#);
    let runs = [
        current(),
        similar_names(),
        running(),
        uninstalled_fresh(),
        stale(),
        uninstalled(),
        scan("jetbrains-current", System::installed(vec![no_version])),
        scan("jetbrains-current", System::with(FakeInstalledApps::failing("no apps folder"))),
        scan("jetbrains-current", System::with(FakeInstalledApps::partial(vec![webstorm()], &["/Applications/X.app"]))),
        uninstalled_with(System::installed(Vec::new()).running(vec![open("Gateway", "com.jetbrains.gateway")])),
        uninstalled_with(System::with(FakeInstalledApps::new(Vec::new()).failing_search("timed out"))),
        uninstalled_with(System::installed(vec![app("com.jetbrains.NewIde", "NewIde", "2026.1")])),
        uninstalled_with(System::with(FakeInstalledApps::new(Vec::new()).with_elsewhere(vec![elsewhere]))),
        uninstalled_with(longer),
    ];
    runs.into_iter().map(|(_, run)| run).collect()
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::support::jetbrains;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(
            left_out_kinds(&jetbrains()).for_now,
            [
                "running",
                "unknown-version",
                "no-system",
                "used-recently",
                "vendor-app-open",
                "apps-unread",
                "spotlight-unread"
            ]
        );
    }
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::jetbrains;

    #[test]
    fn the_jetbrains_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");

        assert_module_contract(&jetbrains(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_jetbrains_text_is_used() {
        assert_texts_used(&jetbrains(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = jetbrains();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 11);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(
            fresher_testing::left_out_kinds(&jetbrains()).kept,
            ["current", "newer", "unknown-ide", "installed-elsewhere"]
        );
    }
}
