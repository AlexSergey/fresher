//! The docker module end to end: a fixture sandbox's home with `OrbStack`'s socket and disk file, Docker Desktop's
//! stale socket and npm's cache (`dev-caches`' to offer); the real scanner (its reader seen through the facts lens),
//! the modules the app registers for them, the real settings file, scan cache and journal, and the real `DiskCleaner`
//! reaching only inside the sandbox, with a fake Trash. `OrbStack` is installed; whether it is open and what its CLI
//! answers are fakes a flow sets. Docker is described, never cleaned: nothing of it is offered, counted or cleaned, and
//! the only program run is the read, never in a restored scan or with the runtime closed.

#[cfg(test)]
#[expect(dead_code, reason = "the docker flows write no golden file")]
mod support;

#[cfg(test)]
#[path = "docker_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;

    use fresher_core::{DangerLevel, Event, FindingOrder, FindingsQuery, Insight, ModuleOutcome, ScanKind};
    use serde_json::{Value, json};

    use crate::flow::{DISK, Home, NPM, SOCKET, Setup, finished, outcome};

    /// The design's four parts, in order: the label's key, the danger level and the command shown.
    pub(crate) const PARTS: [(&str, DangerLevel, Option<&str>); 4] = [
        ("docker.part.build-cache.label", DangerLevel::Safe, Some("docker --context orbstack builder prune --all")),
        ("docker.part.images.label", DangerLevel::Low, Some("docker --context orbstack image prune --all")),
        ("docker.part.containers.label", DangerLevel::Medium, None),
        ("docker.part.volumes.label", DangerLevel::High, Some("docker --context orbstack volume prune")),
    ];

    /// The flow home scanned in full in `setup`, with the scan's events.
    pub(crate) fn scanned(setup: Setup) -> (Home, Vec<Event>) {
        let home = Home::with(setup);
        let events = home.flow.scan();
        (home, events)
    }

    /// The summary's key of each insight.
    pub(crate) fn summaries(insights: &[Insight]) -> Vec<&str> {
        insights.iter().map(|insight| insight.summary().key().as_str()).collect()
    }

    /// The size, the allocated blocks and the kind of the file at `relative`.
    fn state(home: &Home, relative: &str) -> (u64, u64, bool) {
        let metadata = fs::symlink_metadata(home.flow.path(relative)).unwrap();
        (metadata.len(), metadata.blocks(), metadata.is_file())
    }

    pub(crate) fn nothing_of_docker_s_is_found_or_counted() {
        let (home, _) = scanned(Setup::answered());

        assert_eq!(home.flow.findings(), [(NPM.0.to_owned(), NPM.1.to_owned())]);
        let overview = home.flow.engine.overview().unwrap();
        let categories: Vec<&str> = overview.totals.iter().map(|total| total.category.as_str()).collect();
        assert_eq!(categories, [NPM.0]);
        assert!(categories.iter().all(|category| !category.starts_with("docker.")));
        assert_eq!(overview.found, overview.totals[0].sizes, "found is the other module's findings only");
    }

    pub(crate) fn a_cleanup_of_everything_leaves_docker_untouched() {
        let (home, _) = scanned(Setup::answered());
        let before = [state(&home, SOCKET), state(&home, DISK)];
        let page = home.flow.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let findings: Vec<Value> =
            page.findings.iter().map(|finding| serde_json::to_value(finding.id()).unwrap()).collect();
        let overview = home.flow.engine.overview().unwrap();
        let categories: Vec<Value> =
            overview.totals.iter().map(|total| json!({ "category": total.category, "except": [] })).collect();
        let selection = serde_json::from_value(json!({ "findings": findings, "categories": categories })).unwrap();

        let view = home.flow.engine.prepare_cleanup(&selection).unwrap();

        let paths: Vec<String> = view
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .map(|item| home.flow.relative_path(item.path.as_ref().unwrap()))
            .collect();
        assert_eq!(paths, [NPM.1]);
        assert!(view.groups.iter().all(|group| group.items.iter().all(|item| item.more.is_empty())));
        home.flow.clean(&view);
        assert!(!home.flow.path(NPM.1).exists() || fs::read_dir(home.flow.path(NPM.1)).unwrap().next().is_none());
        assert_eq!([state(&home, SOCKET), state(&home, DISK)], before, "OrbStack's files are untouched");
        assert_eq!(home.commands.calls(), std::slice::from_ref(&home.read), "the cleanup runs nothing");
    }

    pub(crate) fn closed_runtime_is_not_asked() {
        let (home, events) = scanned(Setup::closed());

        assert_eq!(home.commands.calls(), Vec::<String>::new());
        let (_, insights) = finished(&events);
        assert_eq!(summaries(&insights), ["docker.summary.stopped"]);
    }

    pub(crate) fn restored_scan_is_not_asked() {
        let (mut home, _) = scanned(Setup::answered());
        home.restart(Setup::answered());

        let events = home.flow.restore();

        let kind = events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        });
        assert!(matches!(kind, Some(ScanKind::Restored { .. })), "{kind:?}");
        let (_, insights) = finished(&events);
        assert_eq!(summaries(&insights), ["docker.summary.restored"]);
        assert_eq!(home.commands.calls(), Vec::<String>::new());
    }

    pub(crate) fn cancelled_read_cancels_the_module() {
        let (home, events) = scanned(Setup::cancelled());

        assert_eq!(finished(&events).0, ModuleOutcome::Cancelled);
        assert_eq!(outcome(&events, "dev-caches"), ModuleOutcome::Completed);
        assert_eq!(home.flow.findings(), [(NPM.0.to_owned(), NPM.1.to_owned())], "the scan goes on");
    }
}

mod negative {
    use crate::flows::{
        a_cleanup_of_everything_leaves_docker_untouched, cancelled_read_cancels_the_module,
        closed_runtime_is_not_asked, nothing_of_docker_s_is_found_or_counted, restored_scan_is_not_asked,
    };

    #[test]
    fn nothing_of_docker_s_is_a_finding_a_total_or_a_category() {
        nothing_of_docker_s_is_found_or_counted();
    }

    #[test]
    fn a_cleanup_of_everything_holds_nothing_of_docker_s() {
        a_cleanup_of_everything_leaves_docker_untouched();
    }

    #[test]
    fn a_closed_runtime_runs_nothing() {
        closed_runtime_is_not_asked();
    }

    #[test]
    fn a_restored_scan_runs_nothing() {
        restored_scan_is_not_asked();
    }

    #[test]
    fn a_cancelled_read_ends_the_module_cancelled() {
        cancelled_read_cancels_the_module();
    }
}

mod positive {
    use fresher_core::{DangerLevel, ModuleOutcome, ShownCommand};

    use crate::flow::{Setup, finished};
    use crate::flows::{PARTS, scanned, summaries};

    #[test]
    fn the_only_call_is_the_read_through_the_runtime_s_socket() {
        let (home, _) = scanned(Setup::answered());

        assert_eq!(home.commands.calls(), std::slice::from_ref(&home.read));
        assert!(home.read.contains("/OrbStack.app/Contents/MacOS/xbin/docker --host unix:///"));
        assert!(
            home.read.ends_with("/home/.orbstack/run/docker.sock --config /private/var/empty system df --format json")
        );
    }

    #[test]
    fn the_module_finished_event_carries_the_insight() {
        let (_, events) = scanned(Setup::answered());

        let (outcome, insights) = finished(&events);
        assert_eq!(outcome, ModuleOutcome::Completed);
        let insight = &insights[0];
        assert_eq!(insight.identity(), "orbstack");
        assert_eq!(insight.summary().key().as_str(), "docker.summary.answered");
        let figures: Vec<&str> = insight.figures().iter().map(|figure| figure.label.key().as_str()).collect();
        assert_eq!(figures, ["docker.figure.on-disk", "docker.figure.inside"], "OrbStack hides the maximum");
        let parts: Vec<(&str, DangerLevel, Option<&str>)> = insight
            .parts()
            .iter()
            .map(|part| (part.label.key().as_str(), part.danger, part.command.as_ref().map(ShownCommand::line)))
            .collect();
        assert_eq!(parts, PARTS);
        let notes: Vec<&str> = insight.notes().iter().map(|note| note.key().as_str()).collect();
        assert_eq!(notes, ["docker.runtime.orbstack.gives-back", "docker.note.never-runs"]);
    }

    #[test]
    fn docker_desktop_s_stale_socket_adds_no_insight() {
        let (_, events) = scanned(Setup::answered());

        let (_, insights) = finished(&events);
        assert_eq!(summaries(&insights), ["docker.summary.answered"]);
        assert_eq!(insights[0].identity(), "orbstack");
    }
}
