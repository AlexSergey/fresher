//! The docker module on its fixture homes: what it must not say or run first, then what it must describe, and its
//! golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-docker` rewrites `fixtures/golden/docker.json`; the diff is
//! reviewed.

#[cfg(test)]
mod support;

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::support::{Scan, closed, every_scan, no_socket, restored, silent, silent_with_desktop, unknown};

    /// The identity and the summary's key of each insight of `scan`, in order.
    fn summaries(scan: &Scan) -> Vec<(&str, &str)> {
        scan.run.insights.iter().map(|insight| (insight.identity(), insight.summary().key().as_str())).collect()
    }

    #[test]
    fn nothing_is_said_or_run_without_an_installed_runtime() {
        let scan = silent();

        assert_eq!(scan.run.insights, []);
        assert_eq!(scan.calls, Vec::<String>::new());
        assert_eq!(scan.run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn a_stale_socket_alone_runs_nothing() {
        let scan = silent_with_desktop();

        let summaries: Vec<&str> = scan.run.insights.iter().map(|insight| insight.summary().key().as_str()).collect();
        assert_eq!(summaries, ["docker.summary.stopped"]);
        assert_eq!(scan.run.insights[0].figures(), []);
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn a_folder_at_the_disk_file_s_place_is_not_a_disk_file() {
        // The silent home has a folder at Docker Desktop's disk file's place; with the app gone, a disk file would say
        // `left`.
        assert_eq!(silent().run.insights, []);
        assert_eq!(silent_with_desktop().run.insights[0].figures(), []);
    }

    #[test]
    fn a_closed_runtime_runs_nothing() {
        let scan = closed();

        assert_eq!(scan.run.insights.len(), 1);
        assert_eq!(scan.run.insights[0].summary().key().as_str(), "docker.summary.stopped");
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn an_open_runtime_without_its_socket_runs_nothing() {
        let scan = no_socket();

        // Docker Desktop's disk file is there with no app installed: its card is `left`.
        assert_eq!(
            summaries(&scan),
            [("docker-desktop", "docker.summary.left"), ("orbstack", "docker.summary.stopped")]
        );
        assert_eq!(scan.run.insights[1].figures(), []);
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn a_restored_scan_runs_nothing() {
        let scan = restored();

        assert_eq!(
            summaries(&scan),
            [("docker-desktop", "docker.summary.left"), ("orbstack", "docker.summary.restored")]
        );
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn running_apps_that_cannot_be_listed_run_nothing() {
        let scan = unknown();

        assert_eq!(
            summaries(&scan),
            [("docker-desktop", "docker.summary.left"), ("orbstack", "docker.summary.unknown")]
        );
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn no_run_pushes_a_finding_or_a_left_out() {
        for scan in every_scan() {
            assert_eq!(scan.run.findings, Vec::<NewFinding>::new());
            assert_eq!(scan.run.left_out.len(), 0);
        }
    }
}

mod positive {
    use fresher_core::DangerLevel;

    use crate::support::{answered, fixtures, left, unseen};

    fn label(figure: &fresher_core::Figure) -> &str {
        figure.label.key().as_str()
    }

    #[test]
    fn an_open_runtime_is_asked_once_through_its_socket_with_no_config_of_the_user() {
        let scan = answered();

        assert_eq!(scan.calls, std::slice::from_ref(&scan.read));
        assert!(scan.read.contains("/OrbStack.app/Contents/MacOS/xbin/docker --host unix:///"));
        assert!(scan.read.ends_with("/.orbstack/run/docker.sock --config /private/var/empty system df --format json"));
        assert!(!scan.read.contains("--context"));
        assert_eq!(scan.run.insights.len(), 1);
        assert_eq!(scan.run.insights[0].summary().key().as_str(), "docker.summary.answered");
    }

    #[test]
    fn the_answer_gives_four_parts_and_a_command_for_each_unused_one() {
        let insight = &answered().run.insights[0];

        let figures: Vec<&str> = insight.figures().iter().map(label).collect();
        assert_eq!(figures, ["docker.figure.on-disk", "docker.figure.inside"]);
        let parts: Vec<(DangerLevel, Option<&str>)> = insight
            .parts()
            .iter()
            .map(|part| (part.danger, part.command.as_ref().map(fresher_core::ShownCommand::line)))
            .collect();
        assert_eq!(
            parts,
            [
                (DangerLevel::Safe, Some("docker --context orbstack builder prune --all")),
                (DangerLevel::Low, Some("docker --context orbstack image prune --all")),
                (DangerLevel::Medium, None),
                (DangerLevel::High, Some("docker --context orbstack volume prune")),
            ]
        );
    }

    #[test]
    fn a_left_disk_file_is_described() {
        let scan = left();

        let insight = &scan.run.insights[0];
        assert_eq!(insight.summary().key().as_str(), "docker.summary.left");
        let figures: Vec<&str> = insight.figures().iter().map(label).collect();
        assert_eq!(figures, ["docker.figure.maximum", "docker.figure.on-disk"]);
        assert_eq!(insight.notes(), []);
        assert_eq!(scan.calls, Vec::<String>::new());
    }

    #[test]
    fn an_unseen_disk_place_says_how_to_see_it() {
        let insight = &unseen().run.insights[0];

        let notes: Vec<&str> = insight.notes().iter().map(|note| note.key().as_str()).collect();
        assert_eq!(notes, ["docker.note.disk-unseen", "docker.runtime.orbstack.gives-back", "docker.note.never-runs"]);
        let figures: Vec<&str> = insight.figures().iter().map(label).collect();
        assert_eq!(figures, ["docker.figure.inside"]);
    }

    /// The report of the answered run.
    #[test]
    fn the_golden_report_matches() {
        let report = answered().run.golden().unwrap();
        let golden = fixtures().join("golden/docker.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
