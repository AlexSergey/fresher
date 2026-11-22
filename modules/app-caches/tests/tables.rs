//! The app-caches module's browser and web-engine tables on their fixture homes: what they must not claim or must keep
//! first, then what they must offer, and their golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-app-caches` rewrites `fixtures/golden/app-caches-tables.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;
#[cfg(test)]
#[path = "support/tables.rs"]
mod tables;

use std::collections::BTreeMap;

use fresher_testing::ModuleRun;

/// How many items the run left out, by reason.
fn left_out(run: &ModuleRun) -> Vec<(&str, usize)> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for item in &run.left_out {
        *counts.entry(item.reason.as_str().trim_start_matches("app-caches.left-out.")).or_default() += 1;
    }
    counts.into_iter().collect()
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding, Subject};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use crate::left_out;
    use crate::tables::{
        tables_kept, tables_kept_system, tables_kept_with, tables_not_claimed, tables_not_claimed_with, tables_offered,
        tables_offered_system, tables_offered_with, tables_system,
    };

    #[test]
    fn nothing_in_the_tables_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = tables_not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn the_tables_kept_home_leaves_out_each_cache_with_its_reason() {
        let (_tree, run) = tables_kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), [("app-open", 2), ("not-yours", 1), ("special", 1)]);
    }

    #[test]
    fn running_apps_that_cannot_be_listed_keep_the_table_caches_that_reach_the_question() {
        let (_tree, run) = tables_kept_with(tables_kept_system().running(FakeRunningApps::failing("no process list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), [("no-system", 2), ("not-yours", 1), ("special", 1)]);
    }

    #[test]
    fn installed_apps_that_cannot_be_listed_claim_no_table_cache_and_say_nothing() {
        for (home, run) in [
            ("not-claimed", tables_not_claimed_with(tables_system().apps(FakeInstalledApps::failing("none"))).1),
            ("kept", tables_kept_with(tables_kept_system().apps(FakeInstalledApps::failing("none"))).1),
            ("offered", tables_offered_with(tables_offered_system().apps(FakeInstalledApps::failing("none"))).1),
        ] {
            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{home}");
            assert_eq!(left_out(&run), [], "{home}");
        }
    }

    #[test]
    fn a_profile_s_and_an_app_s_data_are_never_in_a_finding() {
        let (_tree, run) = tables_offered();

        for finding in &run.findings {
            let Subject::Path(path) = finding.subject() else { unreachable!("not a path: {finding:?}") };
            let held: Vec<&str> = path.components().filter_map(|part| part.as_os_str().to_str()).collect();
            for data in ["image_cache", "Cookies", "Local Storage", "config.json"] {
                assert!(!held.contains(&data), "{path:?}");
            }
        }
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, NewFinding, Removal, Subject, TextArg};
    use fresher_testing::ModuleRun;

    use crate::support::fixtures;
    use crate::tables::{tables_kept, tables_not_claimed, tables_offered, tables_offered_system, tables_offered_with};
    use crate::{identities, left_out};

    const CHROME: &str = "com.google.Chrome";
    const FIREFOX: &str = "org.mozilla.firefox";
    const REDIS: &str = "org.RedisLabs.RedisInsight-V2";

    fn find<'a>(run: &'a ModuleRun, identity: &str) -> &'a NewFinding {
        let found = run.findings.iter().find(|finding| finding.key().identity() == identity);
        found.unwrap_or_else(|| unreachable!("no finding {identity}"))
    }

    #[test]
    fn the_tables_offered_home_offers_each_cache_folder_once() {
        let (_tree, run) = tables_offered();

        assert_eq!(left_out(&run), []);
        assert_eq!(
            identities(&run),
            [
                "chrome/Default/Cache",
                "chrome/Default/Code Cache",
                "chrome/Profile 1/Cache",
                "firefox/abcd1234.default-release/cache2",
                "firefox/abcd1234.default-release/startupCache",
                "redis-insight/Cache",
                "redis-insight/Code Cache",
                "redis-insight/DawnGraphiteCache",
                "redis-insight/DawnWebGPUCache",
                "redis-insight/GPUCache",
            ]
        );
        for finding in &run.findings {
            assert_eq!(finding.danger(), DangerLevel::Low, "{finding:?}");
            assert_eq!(finding.reason().key().as_str(), "app-caches.reason", "{finding:?}");
        }
    }

    #[test]
    fn browser_findings_name_the_browser_and_the_profile() {
        let (_tree, run) = tables_offered();

        for (identity, app, profile, folder) in [
            ("chrome/Default/Cache", "Google Chrome", "Default", "Cache"),
            ("chrome/Default/Code Cache", "Google Chrome", "Default", "Code Cache"),
            ("chrome/Profile 1/Cache", "Google Chrome", "Profile 1", "Cache"),
            ("firefox/abcd1234.default-release/cache2", "Firefox", "abcd1234.default-release", "cache2"),
            ("firefox/abcd1234.default-release/startupCache", "Firefox", "abcd1234.default-release", "startupCache"),
        ] {
            let found = find(&run, identity);
            let args = found.title().args();
            assert_eq!(found.category().as_str(), "app-caches.browsers", "{identity}");
            assert_eq!(found.title().key().as_str(), "app-caches.title.profile", "{identity}");
            assert_eq!(args.get("app"), Some(&TextArg::Text(app.into())), "{identity}");
            assert_eq!(args.get("profile"), Some(&TextArg::Text(profile.into())), "{identity}");
            assert_eq!(args.get("folder"), Some(&TextArg::Text(folder.into())), "{identity}");
        }
    }

    #[test]
    fn web_engine_findings_name_the_app_and_the_folder() {
        let (_tree, run) = tables_offered();

        for folder in ["Cache", "Code Cache", "GPUCache", "DawnGraphiteCache", "DawnWebGPUCache"] {
            let identity = format!("redis-insight/{folder}");
            let found = find(&run, &identity);
            let args = found.title().args();
            assert_eq!(found.category().as_str(), "app-caches.web-engine", "{identity}");
            assert_eq!(found.title().key().as_str(), "app-caches.title.part", "{identity}");
            assert_eq!(args.get("app"), Some(&TextArg::Text("Redis Insight".into())), "{identity}");
            assert_eq!(args.get("folder"), Some(&TextArg::Text(folder.into())), "{identity}");
            assert_eq!(args.get("profile"), None, "{identity}");
        }
    }

    #[test]
    fn every_table_offer_empties_its_folder() {
        let (tree, run) = tables_offered();

        for finding in &run.findings {
            let Removal::EmptyFolder(path) = finding.removal() else { unreachable!("not an EmptyFolder: {finding:?}") };
            assert!(path.starts_with(tree.root().join("Library")), "{path:?}");
            assert!(path.is_dir(), "{path:?}");
            assert_eq!(finding.subject(), &Subject::Path(path.clone()));
        }
        assert_eq!(run.findings.len(), 10);
    }

    #[test]
    fn table_findings_are_counted_in_their_app_s_line() {
        let (_tree, run) = tables_offered();

        let count =
            |id: &str| run.findings.iter().filter(|finding| finding.app().map(BundleId::as_str) == Some(id)).count();

        assert_eq!((count(CHROME), count(FIREFOX), count(REDIS)), (3, 2, 5));
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let (_tree, walked) = tables_offered();
        let (_tree, reversed) = tables_offered_with(tables_offered_system().reversed());

        assert_eq!(reversed.golden().unwrap(), walked.golden().unwrap());
    }

    /// The report of every tables' home with its default system, by home.
    #[test]
    fn the_tables_golden_report_matches() {
        let runs =
            [("not-claimed", tables_not_claimed().1), ("kept", tables_kept().1), ("offered", tables_offered().1)];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/app-caches-tables.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
