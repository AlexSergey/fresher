//! The app-caches module on its fixture homes: what it must not claim or must keep first, then what it must offer, and
//! its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-app-caches` rewrites `fixtures/golden/app-caches.json`; the diff
//! is reviewed.

#[cfg(test)]
#[path = "support/plain.rs"]
mod plain;
#[cfg(test)]
mod support;

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
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use crate::left_out;
    use crate::plain::{
        editor_system, kept, kept_system, kept_with, not_claimed, not_claimed_with, offered_system, offered_with,
    };

    #[test]
    fn nothing_in_the_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), [("app-open", 2), ("not-yours", 1), ("special", 1)]);
    }

    /// The socket's and the lock's folders are kept for those reasons before the running apps are asked.
    #[test]
    fn running_apps_that_cannot_be_listed_keep_the_folders_that_reach_the_question() {
        let (_tree, run) = kept_with(kept_system().running(FakeRunningApps::failing("no process list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), [("no-system", 2), ("not-yours", 1), ("special", 1)]);
    }

    #[test]
    fn installed_apps_that_cannot_be_listed_claim_nothing_and_say_nothing() {
        for (home, run) in [
            ("not-claimed", not_claimed_with(editor_system().apps(FakeInstalledApps::failing("no app list"))).1),
            ("kept", kept_with(kept_system().apps(FakeInstalledApps::failing("no app list"))).1),
            ("offered", offered_with(offered_system().apps(FakeInstalledApps::failing("no app list"))).1),
        ] {
            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{home}");
            assert_eq!(left_out(&run), [], "{home}");
        }
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, Removal, Subject, TextArg};

    use crate::plain::{kept, not_claimed, offered, offered_system, offered_with};
    use crate::support::fixtures;
    use crate::{identities, left_out};

    /// Per finding: its folder, the title key, its app's id and name.
    const EXPECTED: [(&str, &str, &str, &str); 5] = [
        ("com.example.Editor", "own", "com.example.Editor", "Editor"),
        ("com.example.Editor.Helper", "part", "com.example.Editor", "Editor"),
        ("com.example.Editor.ShipIt", "part", "com.example.Editor", "Editor"),
        ("com.example.Suite", "own", "com.example.Suite", "Suite"),
        ("com.example.Suite.Pro", "own", "com.example.Suite.Pro", "Suite Pro"),
    ];

    fn find<'a>(run: &'a fresher_testing::ModuleRun, folder: &str) -> Option<&'a fresher_core::NewFinding> {
        let identity = format!("library-caches/{folder}");
        run.findings.iter().find(|finding| finding.key().identity() == identity)
    }

    #[test]
    fn the_offered_home_offers_every_cache_in_its_app_s_name() {
        let (tree, run) = offered();

        assert_eq!(left_out(&run), []);
        assert_eq!(identities(&run).len(), EXPECTED.len());
        for (folder, title, id, name) in EXPECTED {
            let found = find(&run, folder).unwrap();
            assert_eq!(found.category().as_str(), "app-caches.library-caches", "{folder}");
            assert_eq!(found.danger(), DangerLevel::Low, "{folder}");
            assert_eq!(found.title().key().as_str(), format!("app-caches.title.{title}"), "{folder}");
            assert_eq!(found.title().args().get("app"), Some(&TextArg::Text(name.into())), "{folder}");
            assert_eq!(found.reason().key().as_str(), "app-caches.reason", "{folder}");
            assert_eq!(found.app().map(BundleId::as_str), Some(id), "{folder}");
            assert_eq!(found.subject(), &Subject::Path(tree.root().join("Library/Caches").join(folder)), "{folder}");
        }
        assert_eq!(
            find(&run, "com.example.Editor.ShipIt").unwrap().title().args().get("folder"),
            Some(&TextArg::Text("com.example.Editor.ShipIt".into()))
        );
    }

    #[test]
    fn nested_ids_go_to_the_most_specific_app() {
        let (_tree, run) = offered();

        let app_of = |folder: &str| find(&run, folder).unwrap().app().map(BundleId::as_str).unwrap().to_owned();

        assert_eq!(app_of("com.example.Suite"), "com.example.Suite");
        assert_eq!(app_of("com.example.Suite.Pro"), "com.example.Suite.Pro");
        assert_eq!(app_of("com.example.Editor.ShipIt"), "com.example.Editor");
    }

    #[test]
    fn every_offer_empties_its_folder() {
        let (tree, run) = offered();

        for finding in &run.findings {
            let path = match finding.removal() {
                Removal::EmptyFolder(path) => path,
                other => unreachable!("not an EmptyFolder: {other:?}"),
            };
            assert!(path.starts_with(tree.root().join("Library/Caches")));
            assert_eq!(finding.subject(), &Subject::Path(path.clone()));
        }
        assert_eq!(run.findings.len(), EXPECTED.len());
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let (_tree, walked) = offered();
        let (_tree, reversed) = offered_with(offered_system().reversed());

        assert_eq!(reversed.golden().unwrap(), walked.golden().unwrap());
    }

    /// The report of every home with its default system, by home.
    #[test]
    fn the_golden_report_matches() {
        let runs = [("not-claimed", not_claimed().1), ("kept", kept().1), ("offered", offered().1)];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/app-caches.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
