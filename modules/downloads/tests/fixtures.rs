//! The downloads module on its fixture homes: what it must not claim or must keep first, then what it must offer, and
//! its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-downloads` rewrites `fixtures/golden/downloads.json`; the diff
//! is reviewed.

#[cfg(test)]
mod support;

use std::collections::BTreeMap;

use fresher_testing::ModuleRun;

/// How many items the run left out, by category and reason.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str, usize)> {
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for item in &run.left_out {
        let reason = item.reason.as_str().trim_start_matches("downloads.left-out.");
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

const UNFINISHED: &str = "downloads.unfinished";
const SAFARI: &str = "downloads.safari-resumable";
const INSTALLERS: &str = "downloads.installers";
const MACOS: &str = "downloads.macos-installers";
const LARGE: &str = "downloads.large-untouched";

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::{FakeInstallRecords, FakeInstalledApps, FakeRunningApps};

    use crate::support::{System, kept, kept_apps, kept_with, not_claimed, offered_with};
    use crate::{INSTALLERS, LARGE, MACOS, SAFARI, UNFINISHED, left_out};

    #[test]
    fn nothing_in_the_not_claimed_home_is_claimed_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn each_kept_item_is_left_out_with_its_reason() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (INSTALLERS, "not-older", 2),
                (LARGE, "recent", 1),
                (MACOS, "newer-macos", 1),
                (SAFARI, "not-yours", 1),
                (SAFARI, "special", 1),
                (UNFINISHED, "browser-open", 1),
                (UNFINISHED, "recent", 1),
            ]
        );
    }

    #[test]
    fn failing_running_apps_keep_the_unfinished_as_no_system() {
        let (_tree, run) = kept_with(System::kept().running(FakeRunningApps::failing("no process list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (INSTALLERS, "not-older", 2),
                (LARGE, "recent", 1),
                (MACOS, "newer-macos", 1),
                (SAFARI, "no-system", 2),
                (UNFINISHED, "no-system", 2),
            ]
        );
    }

    #[test]
    fn a_partial_app_list_keeps_installers_as_apps_unread() {
        let apps = FakeInstalledApps::partial(kept_apps(), &["/Applications/Broken.app"]);

        let (_tree, run) = kept_with(System::kept().apps(apps));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (INSTALLERS, "apps-unread", 2),
                (LARGE, "recent", 1),
                (MACOS, "newer-macos", 1),
                (SAFARI, "not-yours", 1),
                (SAFARI, "special", 1),
                (UNFINISHED, "browser-open", 1),
                (UNFINISHED, "recent", 1),
            ]
        );
    }

    #[test]
    fn a_failing_app_list_keeps_installers_as_no_system() {
        let (_tree, run) = kept_with(System::kept().apps(FakeInstalledApps::failing("no app list")));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run).iter().filter(|(category, ..)| *category == INSTALLERS).collect::<Vec<_>>(),
            [&(INSTALLERS, "no-system", 2)]
        );
    }

    #[test]
    fn a_failing_version_keeps_macos_installers_as_no_system() {
        let installs = FakeInstallRecords::default().failing_version("no version");

        let (_tree, run) = kept_with(System::kept().installs(installs));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run).iter().filter(|(category, ..)| *category == MACOS).collect::<Vec<_>>(),
            [&(MACOS, "no-system", 1)]
        );
    }

    #[test]
    fn a_restored_run_keeps_everything_to_scan_again() {
        let (_tree, run) = kept_with(System::kept().restored());

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (INSTALLERS, "scan-again", 2),
                (LARGE, "scan-again", 1),
                (MACOS, "scan-again", 1),
                (SAFARI, "scan-again", 2),
                (UNFINISHED, "scan-again", 2),
            ]
        );
    }

    #[test]
    fn downloads_not_scanned_offers_nothing() {
        let (_tree, run) = offered_with(System::offered().avoiding("Downloads"));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
    }
}

mod positive {
    use std::collections::BTreeMap;

    use fresher_core::{BundleId, DangerLevel, Removal, StoredSettings, Subject};

    use crate::support::{System, fixtures, kept, not_claimed, offered, offered_with, scan_with};
    use crate::{INSTALLERS, LARGE, MACOS, SAFARI, UNFINISHED, identities, left_out};

    /// Per finding: its identity, its category, its danger, the key of its reason and its file under `Downloads`.
    const EXPECTED: [(&str, &str, DangerLevel, &str, &str); 11] = [
        (
            "installers/Example-2.0.pkg",
            INSTALLERS,
            DangerLevel::Low,
            "downloads.reason.installer-receipt",
            "Example-2.0.pkg",
        ),
        (
            "installers/Firefox 133.0.dmg",
            INSTALLERS,
            DangerLevel::Low,
            "downloads.reason.installer",
            "Firefox 133.0.dmg",
        ),
        ("installers/Tool.dmg", INSTALLERS, DangerLevel::Low, "downloads.reason.installer-no-version", "Tool.dmg"),
        ("large-untouched/video.mov", LARGE, DangerLevel::High, "downloads.reason.large-untouched", "video.mov"),
        (
            "macos-installers/Install macOS Sonoma.app",
            MACOS,
            DangerLevel::Low,
            "downloads.reason.macos-installer",
            "Install macOS Sonoma.app",
        ),
        (
            "safari-resumable/trip.zip.download",
            SAFARI,
            DangerLevel::Medium,
            "downloads.reason.safari-resumable",
            "trip.zip.download",
        ),
        ("unfinished/doc.pdf.partial", UNFINISHED, DangerLevel::Low, "downloads.reason.unfinished", "doc.pdf.partial"),
        (
            "unfinished/movie.mp4.crdownload",
            UNFINISHED,
            DangerLevel::Low,
            "downloads.reason.unfinished",
            "movie.mp4.crdownload",
        ),
        (
            "unfinished/song.flac.opdownload",
            UNFINISHED,
            DangerLevel::Low,
            "downloads.reason.unfinished",
            "song.flac.opdownload",
        ),
        ("unfinished/video.mkv", UNFINISHED, DangerLevel::Low, "downloads.reason.placeholder", "video.mkv"),
        ("unfinished/video.mkv.part", UNFINISHED, DangerLevel::Low, "downloads.reason.unfinished", "video.mkv.part"),
    ];

    /// The offers come through the real walk: the module keeps only what lies directly in the home's Downloads, so a
    /// mismatch between the walk's path and the system's would show here as nothing offered.
    #[test]
    fn every_category_is_offered_at_its_danger() {
        let (tree, run) = offered();

        let mut expected: Vec<&str> = EXPECTED.iter().map(|(identity, ..)| *identity).collect();
        expected.sort_unstable();
        assert_eq!(identities(&run), expected);
        assert_eq!(left_out(&run), []);
        for (identity, category, danger, reason, file) in EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            let path = tree.root().join("Downloads").join(file);
            assert_eq!(found.subject(), &Subject::Path(path.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(path), "{identity}");
        }
    }

    #[test]
    fn an_installer_proven_by_name_names_its_app() {
        let (_tree, run) = offered();

        let apps: BTreeMap<&str, Option<&str>> = run
            .findings
            .iter()
            .filter(|finding| finding.category().as_str() == INSTALLERS)
            .map(|finding| (finding.key().identity(), finding.app().map(BundleId::as_str)))
            .collect();

        assert_eq!(
            apps,
            BTreeMap::from([
                ("installers/Example-2.0.pkg", None),
                ("installers/Firefox 133.0.dmg", Some("org.mozilla.firefox")),
                ("installers/Tool.dmg", Some("com.example.Tool")),
            ])
        );
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let (_tree, walked) = offered();
        let (_tree, reversed) = offered_with(System::offered().reversed());

        assert_eq!(reversed.golden().unwrap(), walked.golden().unwrap());
    }

    #[test]
    fn the_setting_moves_the_days() {
        let settings = StoredSettings { values: BTreeMap::from([("downloads.older-than-days".into(), 60.into())]) };

        let (_tree, run) = scan_with("downloads-offered", System::offered(), settings);

        assert_eq!(left_out(&run), [(SAFARI, "recent", 1), (UNFINISHED, "recent", 4)]);
        assert_eq!(
            identities(&run),
            [
                "installers/Example-2.0.pkg",
                "installers/Firefox 133.0.dmg",
                "installers/Tool.dmg",
                "large-untouched/video.mov",
                "macos-installers/Install macOS Sonoma.app",
            ]
        );
    }

    /// The report of every home with its own system, by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [("not-claimed", not_claimed().1), ("kept", kept().1), ("offered", offered().1)];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/downloads.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
