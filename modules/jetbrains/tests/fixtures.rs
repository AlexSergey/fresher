//! The jetbrains module on its fixture homes: what it must keep first, then what it must offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-jetbrains` rewrites `fixtures/golden/jetbrains.json`; the diff
//! is reviewed.

#[cfg(test)]
mod support;

use fresher_core::NewFinding;
use fresher_testing::ModuleRun;

/// Each left-out of the run as its category and reason, in the order reported.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str)> {
    run.left_out.iter().map(|item| (item.category.as_str(), item.reason.as_str())).collect()
}

/// The left-outs `reason` gives, one per category of `categories`, in that order.
fn kept<'a>(categories: &[&'a str], reason: &'a str) -> Vec<(&'a str, &'a str)> {
    categories.iter().map(|category| (*category, reason)).collect()
}

/// The finding whose identity is `identity` (`caches/WebStorm2024.3`), if any.
fn finding<'a>(run: &'a ModuleRun, identity: &str) -> Option<&'a NewFinding> {
    run.findings.iter().find(|finding| finding.key().identity() == identity)
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

const PLACES: [&str; 3] = ["jetbrains.settings", "jetbrains.caches", "jetbrains.logs"];
const UNINSTALLED: [&str; 3] = ["jetbrains.uninstalled"; 3];

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::FakeInstalledApps;

    use crate::support::{
        System, app, current, open, running, scan, similar_names, uninstalled_fresh, uninstalled_with, webstorm,
    };
    use crate::{PLACES, UNINSTALLED, kept, left_out};

    #[test]
    fn the_version_in_use_is_never_offered() {
        let (_tree, run) = current();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&PLACES, "jetbrains.left-out.current"));
    }

    #[test]
    fn names_that_only_look_like_version_folders_are_never_claimed() {
        let (_tree, run) = similar_names();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn an_open_ide_keeps_every_version_even_from_no_apps_folder() {
        let (_tree, run) = running();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        let mut reported = left_out(&run);
        reported.sort_unstable();
        let running = "jetbrains.left-out.running";
        assert_eq!(
            reported,
            kept(&["jetbrains.caches", "jetbrains.caches", "jetbrains.caches"], running)
                .into_iter()
                .chain(kept(&["jetbrains.settings", "jetbrains.settings"], running))
                .collect::<Vec<_>>()
        );
    }

    /// `PhpStorm`'s 2025.1 folder changed 5 days ago keeps its 2024.2 folder too; the Community edition of `PyCharm` is a
    /// product of its own, not installed while the full edition is.
    #[test]
    fn a_removed_ide_whose_data_changed_recently_keeps_every_version() {
        let (_tree, run) = uninstalled_fresh();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&UNINSTALLED, "jetbrains.left-out.used-recently"));
    }

    #[test]
    fn an_app_with_no_version_keeps_every_version() {
        let mut no_version = webstorm();
        no_version.version = None;

        let (_tree, run) = scan("jetbrains-current", System::installed(vec![no_version]));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&PLACES, "jetbrains.left-out.unknown-version"));
    }

    #[test]
    fn apps_that_cannot_be_listed_or_only_in_part_keep_every_version() {
        let failing = FakeInstalledApps::failing("no apps folder");
        let partial = FakeInstalledApps::partial(vec![webstorm()], &["/Applications/Broken.app"]);

        for (installed, reason) in
            [(failing, "jetbrains.left-out.no-system"), (partial, "jetbrains.left-out.apps-unread")]
        {
            let (_tree, run) = scan("jetbrains-current", System::with(installed));

            assert_eq!(run.findings, Vec::<NewFinding>::new());
            assert_eq!(left_out(&run), kept(&PLACES, reason));
        }
    }

    #[test]
    fn an_installed_ide_of_the_folders_version_keeps_them_as_the_version_in_use() {
        let installed = app("com.jetbrains.datagrip", "DataGrip", "2024.3.5");

        let (_tree, run) = uninstalled_with(System::installed(vec![installed]));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&PLACES, "jetbrains.left-out.current"));
    }

    #[test]
    fn an_unknown_app_of_the_vendor_open_keeps_a_removed_ides_data() {
        let system = System::installed(Vec::new()).running(vec![open("Gateway", "com.jetbrains.gateway")]);

        let (_tree, run) = uninstalled_with(system);

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&PLACES, "jetbrains.left-out.vendor-app-open"));
    }

    #[test]
    fn an_unknown_app_of_the_vendor_installed_keeps_a_removed_ides_data() {
        let unknown = app("com.jetbrains.NewIde", "NewIde", "2026.1");

        let (_tree, run) = uninstalled_with(System::installed(vec![unknown]));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&UNINSTALLED, "jetbrains.left-out.unknown-ide"));
    }

    #[test]
    fn a_removed_ide_spotlight_finds_elsewhere_keeps_its_data() {
        let mut far = app("com.jetbrains.datagrip", "DataGrip", "2025.3");
        far.path = "/Volumes/Drive/DataGrip.app".into();
        let installed = FakeInstalledApps::new(Vec::new()).with_elsewhere(vec![far]);

        let (_tree, run) = uninstalled_with(System::with(installed));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&UNINSTALLED, "jetbrains.left-out.installed-elsewhere"));
    }

    #[test]
    fn a_spotlight_that_cannot_be_asked_keeps_a_removed_ides_data() {
        let installed = FakeInstalledApps::new(Vec::new()).failing_search("timed out");

        let (_tree, run) = uninstalled_with(System::with(installed));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&UNINSTALLED, "jetbrains.left-out.spotlight-unread"));
    }

    #[test]
    fn a_setting_longer_than_the_folders_age_keeps_a_removed_ides_data() {
        let system = System::installed(Vec::new()).stored(r#"{"jetbrains.uninstalled-after-days": 60}"#);

        let (_tree, run) = uninstalled_with(system);

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), kept(&UNINSTALLED, "jetbrains.left-out.used-recently"));
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, Removal, Subject};

    use crate::support::{System, fixtures, open, stale, uninstalled, uninstalled_with};
    use crate::{PLACES, finding, identities, kept, left_out};

    #[test]
    fn an_older_version_is_offered_in_its_places_category_for_the_ide_in_use() {
        let (tree, run) = stale();

        assert_eq!(
            identities(&run),
            [
                "caches/IntelliJIdea2024.3",
                "caches/WebStorm2024.3",
                "logs/IntelliJIdea2024.3",
                "logs/WebStorm2024.3",
                "settings/IntelliJIdea2024.3",
                "settings/WebStorm2024.3",
            ]
        );
        let dangers = [DangerLevel::Medium, DangerLevel::Safe, DangerLevel::Safe];
        for (place, (category, danger)) in
            ["settings", "caches", "logs"].into_iter().zip(PLACES.into_iter().zip(dangers))
        {
            let found = finding(&run, &format!("{place}/WebStorm2024.3")).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{place}");
            assert_eq!(found.app().map(BundleId::as_str), Some("com.jetbrains.WebStorm"), "{place}");
        }
        let caches = tree.root().join("Library/Caches/JetBrains/WebStorm2024.3");
        let found = finding(&run, "caches/WebStorm2024.3").unwrap();
        assert_eq!(found.subject(), &Subject::Path(caches.clone()));
        assert_eq!(found.removal(), &Removal::MoveToTrash(caches));
    }

    /// `IntelliJ IDEA` 2025.2 and its early access build 2025.3 are both in use; 2024.3 is older than both, and its
    /// findings name the newest app; 2026.1 is newer than both.
    #[test]
    fn two_versions_side_by_side_are_both_kept_and_an_older_one_is_offered() {
        let (_tree, run) = stale();

        let found = finding(&run, "caches/IntelliJIdea2024.3").unwrap();
        assert_eq!(found.app().map(BundleId::as_str), Some("com.jetbrains.intellij-EAP"));
        let mut reported = left_out(&run);
        reported.sort_unstable();
        let current = "jetbrains.left-out.current";
        let mut expected = kept(&["jetbrains.caches", "jetbrains.caches", "jetbrains.caches"], current);
        expected.push(("jetbrains.caches", "jetbrains.left-out.newer"));
        expected.extend(kept(&["jetbrains.logs"], current));
        expected.extend(kept(&["jetbrains.settings", "jetbrains.settings", "jetbrains.settings"], current));
        assert_eq!(reported, expected);
    }

    #[test]
    fn the_data_of_a_removed_ide_is_offered_folder_by_folder() {
        let (tree, run) = uninstalled();

        assert_eq!(identities(&run), ["caches/DataGrip2024.3", "logs/DataGrip2024.3", "settings/DataGrip2024.3"]);
        for found in &run.findings {
            assert_eq!((found.category().as_str(), found.danger()), ("jetbrains.uninstalled", DangerLevel::Medium));
            assert_eq!(found.app().map(BundleId::as_str), Some("com.jetbrains.datagrip"));
        }
        let logs = tree.root().join("Library/Logs/JetBrains/DataGrip2024.3");
        assert_eq!(finding(&run, "logs/DataGrip2024.3").unwrap().removal(), &Removal::MoveToTrash(logs));
        assert_eq!(left_out(&run), []);
    }

    #[test]
    fn the_toolbox_app_open_does_not_keep_a_removed_ides_data() {
        let system = System::installed(Vec::new()).running(vec![open("JetBrains Toolbox", "com.jetbrains.toolbox")]);

        let (_tree, run) = uninstalled_with(system);

        assert_eq!(identities(&run), ["caches/DataGrip2024.3", "logs/DataGrip2024.3", "settings/DataGrip2024.3"]);
    }

    /// The report of every home with its default apps, by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [
            ("current", crate::support::current().1),
            ("similar-names", crate::support::similar_names().1),
            ("running", crate::support::running().1),
            ("uninstalled-fresh", crate::support::uninstalled_fresh().1),
            ("stale", stale().1),
            ("uninstalled", uninstalled().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/jetbrains.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
