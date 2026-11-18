//! The temp module on its fixture homes: what it must keep first, then what it must offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-temp` rewrites `fixtures/golden/temp.json`; the diff is
//! reviewed.

#[cfg(test)]
mod support;

use fresher_testing::ModuleRun;

const CATEGORY: &str = "temp.old-files";

/// The user's temporary folder of a fixture home, relative to it.
const USER_TEMP: &str = "private/var/folders/xx/fixture/T";

/// The reasons of the run's left-outs, each checked to be of the module's category, sorted.
fn left_out(run: &ModuleRun) -> Vec<&str> {
    let mut reasons: Vec<&str> = run
        .left_out
        .iter()
        .map(|item| {
            assert_eq!(item.category.as_str(), CATEGORY);
            item.reason.as_str()
        })
        .collect();
    reasons.sort_unstable();
    reasons
}

/// `count` times `reason`.
fn kept(reason: &str, count: usize) -> Vec<&str> {
    vec![reason; count]
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

/// The five items `temp-old` offers, by identity.
const OLD: [&str; 5] = [
    "shared-temp/old-build.log",
    "user-temp/com.example.Viewer",
    "user-temp/docs-link",
    "user-temp/node-compile-cache",
    "user-temp/tmp.IKKNWYIRnq",
];

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::FakeRunningApps;

    use crate::support::{System, old, recent, running, scan, special, system};
    use crate::{OLD, identities, kept, left_out};

    /// A fresh file, a folder unpacked with old modification times but a fresh status change, an old folder with one
    /// file changed yesterday, and an old file read yesterday.
    #[test]
    fn nothing_changed_or_read_recently_is_offered() {
        let (_tree, run) = recent();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept("temp.left-out.recent", 4));
    }

    #[test]
    fn the_systems_and_another_users_items_and_a_locked_one_are_never_offered() {
        let (_tree, run) = system();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        let mut expected = kept("temp.left-out.locked", 1);
        expected.extend(kept("temp.left-out.system", 5));
        assert_eq!(left_out(&run), expected);
    }

    /// The sockets, the pipe and the folder holding a socket are special; the lock names and the folder holding a
    /// serial line's lock are locks.
    #[test]
    fn sockets_pipes_and_locks_are_never_offered() {
        let (_tree, run) = special();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        let mut expected = kept("temp.left-out.lock", 4);
        expected.extend(kept("temp.left-out.special", 3));
        assert_eq!(left_out(&run), expected);
    }

    /// Its folder and its hidden working file, named after it with one leading dot.
    #[test]
    fn an_open_apps_items_are_never_offered() {
        let (_tree, run) = running();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept("temp.left-out.app-open", 2));
    }

    /// `C/`, `0/`, `X/`, `T/TemporaryItems` and a home file named like a temporary item are neither offered nor left
    /// out: the module never receives them.
    #[test]
    fn the_temporary_folders_neighbours_and_the_home_are_never_matched() {
        let (_tree, run) = old();

        assert_eq!(identities(&run), OLD);
        assert_eq!(left_out(&run), Vec::<&str>::new());
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn open_apps_that_cannot_be_listed_keep_everything() {
        let (_tree, run) = scan("temp-old", System::plain().running(FakeRunningApps::failing("no process list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept("temp.left-out.no-system", OLD.len()));
    }

    #[test]
    fn a_scan_restored_from_the_cache_offers_nothing() {
        let (_tree, run) = scan("temp-old", System::plain().restored());

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept("temp.left-out.scan-again", OLD.len()));
    }

    #[test]
    fn a_setting_of_14_days_keeps_the_ten_day_old_item() {
        let (_tree, run) = scan("temp-old", System::plain().stored(r#"{"temp.older-than-days": 14}"#));

        assert_eq!(identities(&run), OLD[1..]);
        assert_eq!(left_out(&run), kept("temp.left-out.recent", 1));
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, Removal, Subject, TextArg};
    use fresher_testing::MatchOrder;

    use crate::support::{System, fixtures, old, recent, running, scan, special, system};
    use crate::{CATEGORY, OLD, USER_TEMP, identities, left_out};

    /// Each item of `temp-old`: its finding's reason, its days and its app.
    #[test]
    fn five_old_items_are_offered_safe_with_their_places_reason_and_days() {
        let (tree, run) = old();

        assert_eq!(identities(&run), OLD);
        let expected = [
            ("shared-temp/old-build.log", "temp.finding.reason-shared", 10, None),
            ("user-temp/com.example.Viewer", "temp.finding.reason-user", 15, Some("com.example.Viewer")),
            ("user-temp/docs-link", "temp.finding.reason-user", 30, None),
            ("user-temp/node-compile-cache", "temp.finding.reason-user", 20, None),
            ("user-temp/tmp.IKKNWYIRnq", "temp.finding.reason-user", 30, None),
        ];
        for (identity, reason, days, app) in expected {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (CATEGORY, DangerLevel::Safe), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            assert_eq!(found.reason().args().get("days"), Some(&TextArg::Count(days)), "{identity}");
            assert_eq!(found.app().map(BundleId::as_str), app, "{identity}");
        }
        let link = tree.root().join(USER_TEMP).join("docs-link");
        let found = run.findings.iter().find(|finding| finding.key().identity() == "user-temp/docs-link").unwrap();
        assert_eq!(found.subject(), &Subject::Path(link.clone()));
        assert_eq!(found.removal(), &Removal::MoveToTrash(link));
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        for name in ["temp-recent", "temp-system", "temp-special", "temp-old"] {
            let (_tree, walked) = scan(name, System::plain());
            let (_tree, reversed) = scan(name, System { order: MatchOrder::Reversed, ..System::plain() });

            assert_eq!(identities(&walked), identities(&reversed), "{name}");
            assert_eq!(left_out(&walked), left_out(&reversed), "{name}");
        }
    }

    /// The report of every home with its default system, by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [
            ("recent", recent().1),
            ("system", system().1),
            ("special", special().1),
            ("running", running().1),
            ("old", old().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/temp.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
