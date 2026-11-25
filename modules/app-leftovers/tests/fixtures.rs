//! The app-leftovers module on its fixture homes: what it must not claim or must keep first, then what it must offer,
//! and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-app-leftovers` rewrites `fixtures/golden/app-leftovers.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;

use std::collections::BTreeMap;

use fresher_testing::ModuleRun;

/// How many items the run left out, by category and reason.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str, usize)> {
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for item in &run.left_out {
        let reason = item.reason.as_str().trim_start_matches("app-leftovers.left-out.");
        *counts.entry((item.category.as_str(), reason)).or_default() += 1;
    }
    counts.into_iter().map(|((category, reason), count)| (category, reason, count)).collect()
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

const CACHES: &str = "app-leftovers.caches";
const SETTINGS: &str = "app-leftovers.settings";
const DATA: &str = "app-leftovers.data";

/// The kept home's eleven leftovers, all left out with `reason`: nine of the caches, one of the settings, one of the
/// data.
fn every_leftover(reason: &str) -> Vec<(&'static str, &str, usize)> {
    vec![(CACHES, reason, 9), (DATA, reason, 1), (SETTINGS, reason, 1)]
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use crate::support::{System, apple, elsewhere, installed, kept, kept_with, not_ids, proof_absent, unproven};
    use crate::{CACHES, DATA, SETTINGS, every_leftover, identities, left_out};

    /// Each home is named in the assertion that fails for it.
    #[test]
    fn names_that_are_not_ids_apples_ids_and_ids_without_a_proof_are_never_claimed() {
        for (home, (_tree, run)) in
            [("not-ids", not_ids()), ("apple", apple()), ("unproven", unproven()), ("proof-absent", proof_absent())]
        {
            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{home}");
            assert_eq!(left_out(&run), [], "{home}");
            assert_eq!(run.outcome, ModuleOutcome::Completed, "{home}");
        }
    }

    #[test]
    fn an_installed_apps_its_helpers_and_its_related_ids_files_are_never_claimed() {
        let (_tree, run) = installed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
    }

    /// The socket's and the system's file's ids are offered but for that one item: their saved states are offered.
    #[test]
    fn each_kept_id_is_left_out_with_its_reason_and_the_link_is_not_claimed() {
        let (_tree, run) = kept();

        assert_eq!(
            left_out(&run),
            [
                (CACHES, "installed-elsewhere", 1),
                (CACHES, "not-yours", 1),
                (CACHES, "running", 2),
                (CACHES, "used-recently", 3),
                (DATA, "special", 1),
                (SETTINGS, "installed-elsewhere", 1),
            ]
        );
        assert_eq!(
            identities(&run),
            ["saved-state/com.example.Foreign.savedState", "saved-state/com.example.Socket.savedState"]
        );
    }

    #[test]
    fn running_apps_that_cannot_be_listed_keep_every_leftover_of_every_proven_id() {
        let (_tree, run) = kept_with(System::kept().running(FakeRunningApps::failing("no process list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), every_leftover("no-system"));
    }

    #[test]
    fn a_partial_list_of_apps_keeps_every_leftover_of_every_proven_id() {
        let apps = FakeInstalledApps::partial(Vec::new(), &["/Applications/Broken.app"]);

        let (_tree, run) = kept_with(System::kept().apps(apps));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), every_leftover("apps-unread"));
    }

    #[test]
    fn installed_apps_that_cannot_be_listed_keep_every_leftover_of_every_proven_id() {
        let (_tree, run) = kept_with(System::kept().apps(FakeInstalledApps::failing("no app list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), every_leftover("no-system"));
    }

    /// The open app and the ids changed lately are kept for those reasons first; the three ids that reach Spotlight
    /// are kept unread.
    #[test]
    fn spotlight_that_cannot_tell_keeps_by_precedence_the_ids_that_reach_it() {
        let apps = FakeInstalledApps::default().with_elsewhere(elsewhere()).failing_search("index off");

        let (_tree, run) = kept_with(System::kept().apps(apps));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (CACHES, "running", 2),
                (CACHES, "spotlight-unread", 4),
                (CACHES, "used-recently", 3),
                (DATA, "spotlight-unread", 1),
                (SETTINGS, "spotlight-unread", 1),
            ]
        );
    }

    /// The open app is kept for that first; every other id is kept to scan again, the ones changed lately included.
    #[test]
    fn an_unseen_place_keeps_by_precedence_every_id_to_scan_again() {
        let (_tree, run) = kept_with(System::kept().avoiding("Library/Logs"));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [(CACHES, "running", 2), (CACHES, "scan-again", 7), (DATA, "scan-again", 1), (SETTINGS, "scan-again", 1)]
        );
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, Removal, Subject, TextArg};

    use crate::support::{
        System, apple, fixtures, installed, kept, leftovers, leftovers_with, not_ids, proof_absent, unproven,
    };
    use crate::{CACHES, DATA, SETTINGS, identities, left_out};

    /// Per finding: its identity, its category, its danger, its app's id and its path under `Library`. A container is
    /// not read: it proves nothing, so `com.example.Boxed`, with only a container and its caches, is not claimed.
    const EXPECTED: [(&str, &str, DangerLevel, &str, &str); 11] = [
        ("caches", CACHES, DangerLevel::Safe, "com.example.Appx", "Caches/com.example.Appx"),
        ("caches", CACHES, DangerLevel::Safe, "com.example.Gone", "Caches/com.example.Gone"),
        ("cookies", SETTINGS, DangerLevel::Medium, "com.example.Gone", "Cookies/com.example.Gone.binarycookies"),
        (
            "http-cookies",
            SETTINGS,
            DangerLevel::Medium,
            "com.example.Gone",
            "HTTPStorages/com.example.Gone.binarycookies",
        ),
        ("http-storage", SETTINGS, DangerLevel::Medium, "com.example.Gone", "HTTPStorages/com.example.Gone"),
        ("logs", CACHES, DangerLevel::Safe, "com.example.Gone", "Logs/com.example.Gone"),
        ("preferences", SETTINGS, DangerLevel::Medium, "com.example.Gone", "Preferences/com.example.Gone.plist"),
        (
            "saved-state",
            CACHES,
            DangerLevel::Safe,
            "com.example.Appx",
            "Saved Application State/com.example.Appx.savedState",
        ),
        (
            "saved-state",
            CACHES,
            DangerLevel::Safe,
            "com.example.Gone",
            "Saved Application State/com.example.Gone.savedState",
        ),
        ("support", DATA, DangerLevel::High, "com.example.Gone", "Application Support/com.example.Gone"),
        ("web-data", SETTINGS, DangerLevel::Medium, "com.example.Gone", "WebKit/com.example.Gone"),
    ];

    /// The identity of the finding of `row` at `place`: the row id and the place's last name.
    fn identity(row: &str, place: &str) -> String {
        format!("{row}/{}", place.rsplit_once('/').map_or(place, |(_, name)| name))
    }

    /// The identity of each expected finding, sorted.
    fn expected_identities() -> Vec<String> {
        let mut expected: Vec<String> = EXPECTED.iter().map(|(row, _, _, _, place)| identity(row, place)).collect();
        expected.sort_unstable();
        expected
    }

    #[test]
    fn every_leftover_of_an_app_that_is_gone_is_offered_in_its_category_at_its_danger() {
        let (tree, run) = leftovers();

        assert_eq!(identities(&run), expected_identities());
        assert_eq!(left_out(&run), []);
        for (row, category, danger, id, place) in EXPECTED {
            let identity = identity(row, place);
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.title().key().as_str(), format!("app-leftovers.title.{row}"), "{identity}");
            assert_eq!(found.title().args().get("id"), Some(&TextArg::Text(id.into())), "{identity}");
            assert_eq!(found.reason().key().as_str(), "app-leftovers.finding.reason", "{identity}");
            assert_eq!(found.reason().args().get("days"), Some(&TextArg::Count(30)), "{identity}");
            assert_eq!(found.app().map(BundleId::as_str), Some(id), "{identity}");
            let path = tree.root().join("Library").join(place);
            assert_eq!(found.subject(), &Subject::Path(path.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(path), "{identity}");
        }
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let (_tree, walked) = leftovers();
        let (_tree, reversed) = leftovers_with(System::plain().reversed());

        assert_eq!(reversed.golden().unwrap(), walked.golden().unwrap());
    }

    /// The module reads no container, so a walk that avoids them gives the same answers.
    #[test]
    fn with_the_containers_avoided_the_answers_are_the_same() {
        let (_tree, run) = leftovers_with(System::plain().avoiding("Library/Containers"));

        assert_eq!(identities(&run), expected_identities());
        assert_eq!(left_out(&run), []);
    }

    /// The report of every home with its default system, by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [
            ("not-ids", not_ids().1),
            ("apple", apple().1),
            ("unproven", unproven().1),
            ("proof-absent", proof_absent().1),
            ("installed", installed().1),
            ("kept", kept().1),
            ("leftovers", leftovers().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/app-leftovers.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
