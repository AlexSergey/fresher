//! The logs module on its fixture homes: what it must keep first, then what it must offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-logs` rewrites `fixtures/golden/logs.json`; the diff is
//! reviewed.

#[cfg(test)]
mod support;

use fresher_testing::ModuleRun;

/// The reasons of the run's left-outs, sorted.
fn left_out(run: &ModuleRun) -> Vec<&str> {
    let mut reasons: Vec<&str> = run.left_out.iter().map(|item| item.reason.as_str()).collect();
    reasons.sort_unstable();
    reasons
}

/// `count` times `reason`.
fn times(reason: &str, count: usize) -> Vec<&str> {
    vec![reason; count]
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

/// The eight files `logs-offered` offers, by identity.
const OFFERED: [&str; 8] = [
    "app-logs/Vendor/Updater/1.0/updater.log",
    "app-logs/Writer/writer.1.log",
    "app-logs/Writer/writer.10d.log",
    "app-logs/a/b/c/d/e/six.log",
    "app-logs/com.example.Tool.log",
    "app-logs/old-tool.log",
    "crash-reports/Retired/Old-2026-09-01-000000.ips",
    "crash-reports/Writer-2026-10-01-101010.ips",
];

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::support::{System, kept, not_claimed, offered, scan, subject_path};
    use crate::{OFFERED, identities, left_out, times};

    /// Another module's places, links, folders, state and lock names, a file too deep and folders outside the Library.
    #[test]
    fn nothing_in_the_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), Vec::<&str>::new());
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        let mut expected = times("logs.left-out.locked", 1);
        expected.extend(times("logs.left-out.not-yours", 2));
        expected.extend(times("logs.left-out.recent", 4));
        expected.extend(times("logs.left-out.special", 1));
        assert_eq!(left_out(&run), expected);
    }

    #[test]
    fn a_longer_setting_keeps_the_ten_day_old_log() {
        let (_tree, run) = scan("logs-offered", System::plain().stored(r#"{"logs.older-than-days": 14}"#));

        let expected: Vec<&str> =
            OFFERED.into_iter().filter(|identity| !identity.ends_with("writer.10d.log")).collect();
        assert_eq!(identities(&run), expected);
        assert_eq!(left_out(&run), times("logs.left-out.recent", 2));
    }

    #[test]
    fn no_folder_is_ever_offered() {
        let longer = scan("logs-offered", System::plain().stored(r#"{"logs.older-than-days": 14}"#));
        let runs = [not_claimed(), kept(), offered(), longer];

        for (_tree, run) in &runs {
            for finding in &run.findings {
                let path = subject_path(finding);
                let kind = std::fs::symlink_metadata(path).unwrap().file_type();
                assert!(kind.is_file(), "{path:?} is not a file");
            }
        }
    }
}

mod positive {
    use fresher_core::{DangerLevel, Removal, TextArg};

    use crate::support::{fixtures, offered, subject_path};
    use crate::{OFFERED, identities, left_out, times};

    #[test]
    fn the_offered_home_offers_every_old_file_once() {
        let (_tree, run) = offered();

        assert_eq!(identities(&run), OFFERED);
        assert_eq!(left_out(&run), times("logs.left-out.recent", 1));
    }

    /// Each file: its category, its reason with the days and the folder it states.
    #[test]
    fn crash_reports_and_app_logs_are_apart() {
        let (_tree, run) = offered();

        let expected = [
            ("app-logs/old-tool.log", "logs.app-logs", "logs.reason.top", 30, None),
            ("app-logs/com.example.Tool.log", "logs.app-logs", "logs.reason.top", 30, None),
            ("app-logs/Writer/writer.1.log", "logs.app-logs", "logs.reason.folder", 30, Some("Writer")),
            ("app-logs/Writer/writer.10d.log", "logs.app-logs", "logs.reason.folder", 10, Some("Writer")),
            (
                "app-logs/Vendor/Updater/1.0/updater.log",
                "logs.app-logs",
                "logs.reason.folder",
                20,
                Some("Vendor/Updater/1.0"),
            ),
            ("app-logs/a/b/c/d/e/six.log", "logs.app-logs", "logs.reason.folder", 30, Some("a/b/c/d/e")),
            ("crash-reports/Writer-2026-10-01-101010.ips", "logs.crash-reports", "logs.reason.report", 30, None),
            ("crash-reports/Retired/Old-2026-09-01-000000.ips", "logs.crash-reports", "logs.reason.report", 60, None),
        ];
        for (identity, category, reason, days, folder) in expected {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, DangerLevel::Safe), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            assert_eq!(found.reason().args().get("days"), Some(&TextArg::Count(days)), "{identity}");
            let shown = found.reason().args().get("folder");
            assert_eq!(shown, folder.map(|name| TextArg::Text(name.into())).as_ref(), "{identity}");
        }
    }

    #[test]
    fn every_offer_moves_its_file_to_the_trash() {
        let (tree, run) = offered();

        for finding in &run.findings {
            let path = subject_path(finding);
            assert_eq!(finding.removal(), &Removal::MoveToTrash(path.to_path_buf()));
            assert!(path.starts_with(tree.root().join("Library/Logs")), "{path:?}");
        }
    }

    /// The offered home's report: its findings and its left-out.
    #[test]
    fn the_golden_report_matches() {
        let report = offered().1.golden().unwrap();
        let golden = fixtures().join("golden/logs.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
