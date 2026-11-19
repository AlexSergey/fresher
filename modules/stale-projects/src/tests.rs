//! The module through `on_match` and `finish`, on a recording sink: the projects set by hand on the system's facts,
//! the matches built by hand, in a home at `/home`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    Artifact, ByteSize, CancelToken, EntryFacts, EntryKind, LastChange, LeftOutItem, Match, Module as _, ModuleContext,
    ModuleError, NewFinding, Project, ProjectKind, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts,
    Timestamp,
};
use fresher_testing::{
    FakeClock, FakeCommandRunner, FakeInstallRecords, FakeInstalledApps, FakeKnownFolders, FakeRunningApps,
};

use crate::StaleProjectsModule;

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;

/// A time `days` days before the scan.
fn ago(days: u64) -> Timestamp {
    Timestamp::from_secs(NOW - days * DAY)
}

/// Keeps everything the module reports, in order.
#[derive(Debug, Default, PartialEq, Eq)]
struct Recorded {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
}

impl Recorded {
    fn identities(&self) -> Vec<&str> {
        self.findings.iter().map(|finding| finding.key().identity()).collect()
    }

    /// Each left-out as its category, reason and logical bytes.
    fn left_out(&self) -> Vec<(&str, &str, u64)> {
        self.left_out
            .iter()
            .map(|item| (item.category.as_str(), item.reason.as_str(), item.sizes.logical.bytes()))
            .collect()
    }
}

impl fresher_core::FindingSink for Recorded {
    fn push(&mut self, finding: NewFinding) {
        self.findings.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        self.left_out.push(left_out);
    }
}

/// Records like `Recorded`, and cancels the scan on the first report.
struct CancelOnReport<'a> {
    recorded: &'a mut Recorded,
    cancel: &'a CancelToken,
}

impl fresher_core::FindingSink for CancelOnReport<'_> {
    fn push(&mut self, finding: NewFinding) {
        self.recorded.push(finding);
        self.cancel.cancel();
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        fresher_core::FindingSink::left_out(self.recorded, left_out);
        self.cancel.cancel();
    }
}

/// A folder under the home of `bytes` bytes, read in full, last changed `days` days ago.
fn folder(path: &str, bytes: u64, days: u64) -> Artifact {
    let size = ByteSize::new(bytes);
    Artifact {
        path: Path::new("/home").join(path),
        sizes: Sizes::complete(size, size),
        last_change: LastChange::Known(ago(days)),
    }
}

/// A project at `root` under the home, of `kinds`, nothing in it changed for `days` days.
fn project(root: &str, kinds: &[ProjectKind], artifacts: Vec<Artifact>, caches: Vec<Artifact>, days: u64) -> Project {
    Project {
        root: Path::new("/home").join(root),
        kinds: kinds.to_vec(),
        artifacts,
        caches,
        last_activity: LastChange::Known(ago(days)),
    }
}

/// The module's match of `folder`, as the walk gives it.
fn matched(folder: &Artifact) -> Match {
    Match {
        interest: 0,
        path: folder.path.clone(),
        kind: EntryKind::Folder,
        sizes: Sizes::ZERO,
        modified: None,
        changed: None,
        accessed: None,
        inside: None,
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// Every artifact and cache of `projects`, matched.
fn every_folder(projects: &[Project]) -> Vec<Match> {
    projects.iter().flat_map(|project| project.artifacts.iter().chain(&project.caches)).map(matched).collect()
}

/// One scan: the projects the walk found (`None`: not known yet), the stored days, restored or not.
#[derive(Default)]
struct Scan {
    projects: Option<Vec<Project>>,
    days: Option<u32>,
    restored: bool,
}

impl Scan {
    fn of(projects: Vec<Project>) -> Self {
        Self { projects: Some(projects), ..Self::default() }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let (recorded, finished) = self.run_until(matches, false);
        finished.unwrap();
        recorded
    }

    /// As `run`; with `cancel_on_report`, the scan is cancelled when `finish` reports its first item.
    fn run_until(&self, matches: &[Match], cancel_on_report: bool) -> (Recorded, Result<(), ModuleError>) {
        let cancel = &CancelToken::new();
        let module = StaleProjectsModule::new().unwrap();
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::default()),
            running: Arc::new(FakeRunningApps::default()),
            installs: Arc::new(FakeInstallRecords::default()),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let values = self
            .days
            .map(|days| ("stale-projects.unused-for-days".to_owned(), serde_json::json!(days)))
            .into_iter()
            .collect();
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings { values })
            .module(module.descriptor().id());
        let context = || {
            let cx = ModuleContext::new(&clock, &commands, &system, &settings, cancel).complete_walk();
            if self.restored { cx.restored() } else { cx }
        };
        let mut out = Recorded::default();
        for found in matches {
            module.on_match(found, &context(), &mut out).unwrap();
        }
        if let Some(projects) = &self.projects {
            system.set_projects(projects.clone());
        }
        let finished = if cancel_on_report {
            module.finish(&context(), &mut CancelOnReport { recorded: &mut out, cancel })
        } else {
            module.finish(&context(), &mut out)
        };
        (out, finished)
    }
}

/// A stale monorepo `shop` with a package, a stale Rust project, an active Rust project with an old cache, an active
/// Node project.
fn home() -> Vec<Project> {
    vec![
        project("Code/shop", &[ProjectKind::Node], vec![folder("Code/shop/node_modules", 500, 400)], Vec::new(), 400),
        project(
            "Code/shop/packages/web",
            &[ProjectKind::Node],
            vec![folder("Code/shop/packages/web/node_modules", 300, 420)],
            Vec::new(),
            420,
        ),
        project(
            "Code/old-tool",
            &[ProjectKind::Rust],
            vec![folder("Code/old-tool/target", 900, 500)],
            vec![folder("Code/old-tool/target/debug/incremental", 400, 500)],
            500,
        ),
        project(
            "Code/fresher",
            &[ProjectKind::Rust],
            vec![folder("Code/fresher/target", 1000, 1)],
            vec![
                folder("Code/fresher/target/debug/incremental", 300, 10),
                folder("Code/fresher/target/llvm-cov-target", 200, 2),
            ],
            1,
        ),
        project("Code/web", &[ProjectKind::Node], vec![folder("Code/web/node_modules", 700, 5)], Vec::new(), 5),
    ]
}

mod negative {
    use fresher_core::{LastChange, ModuleError, ProjectKind};

    use super::{Recorded, Scan, ago, every_folder, folder, home, project};

    #[test]
    fn nothing_is_claimed_before_the_projects_are_known() {
        let recorded = Scan::default().run(&every_folder(&home()));

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn framework_output_is_never_claimed() {
        let web = project(
            "Code/site",
            &[ProjectKind::Node],
            vec![folder("Code/site/dist", 800, 900), folder("Code/site/.next", 600, 900)],
            Vec::new(),
            900,
        );
        let matches = every_folder(std::slice::from_ref(&web));
        assert_eq!(matches.len(), 2);

        let recorded = Scan::of(vec![web]).run(&matches);

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn a_folder_the_module_did_not_match_is_not_claimed() {
        let recorded = Scan::of(home()).run(&[]);

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn the_offered_target_s_caches_are_not_offered_again() {
        let projects = home();
        let tool: Vec<_> = projects.into_iter().filter(|project| project.root.ends_with("old-tool")).collect();

        let recorded = Scan::of(tool.clone()).run(&every_folder(&tool));

        assert_eq!(recorded.identities(), ["build-output/Code/old-tool/target"]);
        assert_eq!(recorded.left_out, []);
    }

    #[test]
    fn a_restored_scan_offers_nothing() {
        let projects = home();
        let scan = Scan { restored: true, ..Scan::of(projects.clone()) };

        let recorded = scan.run(&every_folder(&projects));

        assert_eq!(recorded.findings, []);
        assert!(recorded.left_out().iter().all(|(_, reason, _)| *reason == "stale-projects.left-out.scan-again"));
        assert_eq!(recorded.left_out.len(), 8);
    }

    #[test]
    fn a_commit_in_a_nested_repository_keeps_the_whole_group() {
        let mut projects: Vec<_> =
            home().into_iter().filter(|project| project.root.starts_with("/home/Code/shop")).collect();
        projects[1].last_activity = LastChange::Known(ago(3));

        let recorded = Scan::of(projects.clone()).run(&every_folder(&projects));

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("stale-projects.dependencies", "stale-projects.left-out.recent", 500),
                ("stale-projects.dependencies", "stale-projects.left-out.recent", 300),
            ]
        );
    }

    #[test]
    fn a_cancelled_scan_stops_between_groups() {
        let projects = home();

        let (recorded, finished) = Scan::of(projects.clone()).run_until(&every_folder(&projects), true);

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(recorded.identities(), ["rust-build-cache/Code/fresher/target/debug/incremental"]);
        assert_eq!(
            recorded.left_out(),
            [
                ("stale-projects.rust-build-cache", "stale-projects.left-out.cache-recent", 200),
                ("stale-projects.build-output", "stale-projects.left-out.recent", 700),
            ]
        );
    }

    #[test]
    fn an_artifact_not_right_inside_its_project_is_not_claimed() {
        let site = project(
            "Code/site",
            &[ProjectKind::Node],
            vec![folder("Code/site/sub/node_modules", 800, 900)],
            Vec::new(),
            900,
        );
        let matches = every_folder(std::slice::from_ref(&site));

        let recorded = Scan::of(vec![site]).run(&matches);

        assert_eq!(recorded, Recorded::default());
    }
}

mod positive {
    use fresher_core::{ProjectKind, Removal, TextArg};

    use super::{Scan, ago, every_folder, folder, home, project};

    #[test]
    fn a_stale_monorepo_offers_every_package_under_its_name() {
        let projects: Vec<_> =
            home().into_iter().filter(|project| project.root.starts_with("/home/Code/shop")).collect();

        let recorded = Scan::of(projects.clone()).run(&every_folder(&projects));

        assert_eq!(
            recorded.identities(),
            ["dependencies/Code/shop/node_modules", "dependencies/Code/shop/packages/web/node_modules"]
        );
        let titles: Vec<(&TextArg, &TextArg)> = recorded
            .findings
            .iter()
            .map(|finding| (&finding.title().args()["project"], &finding.title().args()["folder"]))
            .collect();
        assert_eq!(
            titles,
            [
                (&TextArg::Text("shop".into()), &TextArg::Text("node_modules".into())),
                (&TextArg::Text("shop".into()), &TextArg::Text("packages/web/node_modules".into())),
            ]
        );
        for finding in &recorded.findings {
            assert_eq!(finding.reason().key().as_str(), "stale-projects.reason.dependencies");
            assert_eq!(finding.reason().args()["date"], TextArg::Date(ago(400)));
            assert_eq!(finding.reason().args()["command"], TextArg::Text("npm install".into()));
        }
        let path = std::path::PathBuf::from("/home/Code/shop/node_modules");
        assert_eq!(recorded.findings[0].removal(), &Removal::MoveToTrash(path));
        assert_eq!(recorded.left_out, []);
    }

    #[test]
    fn an_active_rust_project_offers_its_old_cache_and_keeps_the_rest() {
        let projects: Vec<_> = home().into_iter().filter(|project| project.root.ends_with("fresher")).collect();

        let recorded = Scan::of(projects.clone()).run(&every_folder(&projects));

        assert_eq!(recorded.identities(), ["rust-build-cache/Code/fresher/target/debug/incremental"]);
        let reason = recorded.findings[0].reason();
        assert_eq!(reason.key().as_str(), "stale-projects.reason.rust-build-cache");
        assert_eq!(reason.args()["days"], TextArg::Count(7));
        assert_eq!(reason.args()["command"], TextArg::Text("cargo build".into()));
        assert_eq!(recorded.findings[0].title().args()["folder"], TextArg::Text("target/debug/incremental".into()));
        assert_eq!(
            recorded.left_out(),
            [
                ("stale-projects.rust-build-cache", "stale-projects.left-out.cache-recent", 200),
                ("stale-projects.build-output", "stale-projects.left-out.recent", 700),
            ]
        );
    }

    #[test]
    fn the_matches_in_two_orders_give_the_same_answers() {
        let projects = home();
        let mut reversed = every_folder(&projects);
        reversed.reverse();

        let forward = Scan::of(projects.clone()).run(&every_folder(&projects));

        assert_eq!(forward, Scan::of(projects).run(&reversed));
        assert_eq!(forward.findings.len(), 4);
    }

    #[test]
    fn the_setting_moves_the_days() {
        let app =
            project("Code/app", &[ProjectKind::Node], vec![folder("Code/app/node_modules", 500, 300)], Vec::new(), 300);
        let matches = every_folder(std::slice::from_ref(&app));

        let by_default = Scan::of(vec![app.clone()]).run(&matches);
        let shorter = Scan { days: Some(200), ..Scan::of(vec![app]) }.run(&matches);

        assert_eq!(by_default.findings, []);
        assert_eq!(by_default.left_out(), [("stale-projects.dependencies", "stale-projects.left-out.recent", 500)]);
        assert_eq!(shorter.identities(), ["dependencies/Code/app/node_modules"]);
        assert_eq!(shorter.left_out, []);
    }
}
