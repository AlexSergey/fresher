//! The module as built, and the module through `on_match` and `finish` on hand-built matches, with every system fact
//! and the command runner faked, in a home at `/home`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, CommandOutput, CommandRunError, CommandRunner, EntryFacts, EntryKind, Insight, InstalledApp,
    LeftOutItem, Match, Module as _, ModuleContext, ModuleError, NewFinding, PlaceSeen, RunningApp, SettingsSchema,
    Sizes, StoredSettings, SystemFacts, SystemPorts,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::DockerModule;
use crate::runtimes::Runtimes;

const DOCKER_DESKTOP: &str = "com.docker.docker";
const ORBSTACK: &str = "dev.kdrag0n.MacVirt";

/// The interests of the shipped table: per row its socket, then its disk file.
const DESKTOP_SOCKET: usize = 0;
const DESKTOP_DISK: usize = 1;
const ORBSTACK_SOCKET: usize = 2;
const ORBSTACK_DISK: usize = 3;

/// The only call the module may make for each runtime, as `FakeCommandRunner` keys it.
const ORBSTACK_READ: &str =
    "/Applications/OrbStack.app/Contents/MacOS/xbin/docker --context orbstack system df --format json";
const DESKTOP_READ: &str =
    "/Applications/Docker.app/Contents/Resources/bin/docker --context desktop-linux system df --format json";

/// An answer with something unused in every kind but the stopped containers.
const ANSWER: &str = r#"{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}"#;

/// Keeps everything the module reports, in order.
#[derive(Debug, Default)]
struct Recorded {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
    insights: Vec<Insight>,
}

impl Recorded {
    fn identities(&self) -> Vec<&str> {
        self.insights.iter().map(Insight::identity).collect()
    }

    fn summaries(&self) -> Vec<&str> {
        self.insights.iter().map(|insight| insight.summary().key().as_str()).collect()
    }
}

impl fresher_core::FindingSink for Recorded {
    fn push(&mut self, finding: NewFinding) {
        self.findings.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        self.left_out.push(left_out);
    }

    fn describe(&mut self, insight: Insight) {
        self.insights.push(insight);
    }
}

fn desktop() -> InstalledApp {
    fresher_testing::installed_app(DOCKER_DESKTOP, "Docker", "4.40").unwrap()
}

fn orbstack() -> InstalledApp {
    fresher_testing::installed_app(ORBSTACK, "OrbStack", "2.2.5").unwrap()
}

fn running(bundle_id: &str) -> RunningApp {
    fresher_testing::running_app("App", &[bundle_id]).unwrap()
}

fn entry(interest: usize, place: &str, kind: EntryKind, sizes: Sizes) -> Match {
    Match {
        interest,
        path: Path::new("/home").join(place),
        kind,
        sizes,
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

const DESKTOP_DISK_PLACE: &str = "Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw";

fn desktop_socket() -> Match {
    entry(DESKTOP_SOCKET, ".docker/run/docker.sock", EntryKind::Other, Sizes::ZERO)
}

fn orbstack_socket() -> Match {
    entry(ORBSTACK_SOCKET, ".orbstack/run/docker.sock", EntryKind::Other, Sizes::ZERO)
}

fn disk_sizes() -> Sizes {
    Sizes::complete(ByteSize::new(64_000_000_000), ByteSize::new(9_000_000_000))
}

fn desktop_disk() -> Match {
    entry(DESKTOP_DISK, DESKTOP_DISK_PLACE, EntryKind::File, disk_sizes())
}

fn orbstack_disk() -> Match {
    let place = "Library/Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img";
    entry(ORBSTACK_DISK, place, EntryKind::File, disk_sizes())
}

/// A command runner shared with the test, to read its calls after the scan.
struct Shared(Arc<FakeCommandRunner>);

impl CommandRunner for Shared {
    fn run(&self, program: &str, args: &[&str], cancel: &CancelToken) -> Result<CommandOutput, CommandRunError> {
        self.0.run(program, args, cancel)
    }
}

/// A command runner whose every run answers `Cancelled`, as a read stopped by the scan's cancel.
struct CancelledRead;

impl CommandRunner for CancelledRead {
    fn run(&self, program: &str, _: &[&str], _: &CancelToken) -> Result<CommandOutput, CommandRunError> {
        Err(CommandRunError::Cancelled { program: program.into() })
    }
}

/// The system and the commands a scan runs with; every place read unless a test says otherwise.
struct Scan {
    installed: Arc<FakeInstalledApps>,
    running: Arc<FakeRunningApps>,
    commands: Arc<FakeCommandRunner>,
    runner: Box<dyn CommandRunner>,
    places: Vec<PlaceSeen>,
    restored: bool,
}

impl Scan {
    fn new(installed: Vec<InstalledApp>, running: Vec<RunningApp>) -> Self {
        Self::with(FakeInstalledApps::new(installed), FakeRunningApps::new(running))
    }

    fn with(installed: FakeInstalledApps, running: FakeRunningApps) -> Self {
        Self::answering(installed, running, FakeCommandRunner::default())
    }

    fn answering(installed: FakeInstalledApps, running: FakeRunningApps, commands: FakeCommandRunner) -> Self {
        let commands = Arc::new(commands);
        Self {
            installed: Arc::new(installed),
            running: Arc::new(running),
            runner: Box::new(Shared(Arc::clone(&commands))),
            commands,
            places: vec![PlaceSeen::Read; 4],
            restored: false,
        }
    }

    /// The same scan with `commands` answering its calls.
    fn commands(self, commands: FakeCommandRunner) -> Self {
        let commands = Arc::new(commands);
        Self { runner: Box::new(Shared(Arc::clone(&commands))), commands, ..self }
    }

    fn runner(self, runner: impl CommandRunner + 'static) -> Self {
        Self { runner: Box::new(runner), ..self }
    }

    fn place(mut self, interest: usize, seen: PlaceSeen) -> Self {
        self.places[interest] = seen;
        self
    }

    fn restored(self) -> Self {
        Self { restored: true, ..self }
    }

    fn run(&self, matches: &[Match]) -> Recorded {
        let mut out = Recorded::default();
        self.finish(matches, &CancelToken::new(), &mut out).unwrap();
        out
    }

    /// Hands `matches` to a new module, then calls `finish` with the scan's places.
    fn finish(
        &self,
        matches: &[Match],
        cancel: &CancelToken,
        out: &mut dyn fresher_core::FindingSink,
    ) -> Result<(), ModuleError> {
        let module = DockerModule::new().unwrap();
        for found in matches {
            self.with_context(&module, cancel, |cx| module.on_match(found, &cx, out))?;
        }
        self.with_context(&module, cancel, |cx| module.finish(&cx.with_places(&self.places), out))
    }

    fn with_context<T>(
        &self,
        module: &DockerModule,
        cancel: &CancelToken,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::<FakeInstalledApps>::clone(&self.installed),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::new(fresher_testing::NoFiles),
        });
        let clock = FakeClock::at(1_767_225_600);
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        let cx = ModuleContext::new(&clock, self.runner.as_ref(), &system, &settings, cancel);
        call(if self.restored { cx.restored() } else { cx })
    }
}

/// `OrbStack` installed, running, its socket and disk file seen, answering `answer`.
fn answering(answer: &str) -> Recorded {
    Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
        .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, answer))
        .run(&[orbstack_socket(), orbstack_disk()])
}

/// Each part of the first insight as its label key, detail key and command line.
fn parts(recorded: &Recorded) -> Vec<(String, String, Option<String>)> {
    recorded.insights[0]
        .parts()
        .iter()
        .map(|part| {
            let line = part.command.as_ref().map(|command| command.line().to_owned());
            (part.label.key().to_string(), part.detail.key().to_string(), line)
        })
        .collect()
}

fn notes(insight: &Insight) -> Vec<&str> {
    insight.notes().iter().map(|note| note.key().as_str()).collect()
}

fn figures(insight: &Insight) -> Vec<(&str, u64)> {
    insight.figures().iter().map(|figure| (figure.label.key().as_str(), figure.size.bytes())).collect()
}

mod negative {
    use std::os::unix::ffi::OsStrExt as _;

    use fresher_core::{
        CancelToken, EntryKind, Insight, InstalledApp, LeftOutItem, ModuleError, NewFinding, PlaceSeen,
    };
    use fresher_testing::{FakeCommandRunner, FakeInstalledApps, FakeRunningApps};

    use super::{
        ANSWER, CancelledRead, DESKTOP_DISK, DESKTOP_DISK_PLACE, DOCKER_DESKTOP, ORBSTACK, ORBSTACK_DISK,
        ORBSTACK_READ, ORBSTACK_SOCKET, Recorded, Scan, desktop, desktop_disk, desktop_socket, disk_sizes, entry,
        notes, orbstack, orbstack_disk, orbstack_socket, running,
    };

    #[test]
    fn no_runtime_installed_runs_nothing_and_says_nothing() {
        let scan = Scan::new(Vec::new(), vec![running(DOCKER_DESKTOP), running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        let recorded = scan.run(&[desktop_socket(), orbstack_socket()]);

        assert_eq!(recorded.insights, Vec::<Insight>::new());
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
        assert_eq!(scan.running.calls(), 0);
    }

    #[test]
    fn nothing_is_said_when_the_installed_apps_cannot_be_listed() {
        let scan = Scan::with(FakeInstalledApps::failing("no access"), FakeRunningApps::new(vec![running(ORBSTACK)]));

        let recorded = scan.run(&[orbstack_socket(), orbstack_disk(), desktop_disk()]);

        assert_eq!(recorded.insights, Vec::<Insight>::new());
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
        assert_eq!(scan.installed.calls(), 1);
    }

    #[test]
    fn a_runtime_missed_by_a_partial_listing_is_not_described_as_left() {
        let installed = FakeInstalledApps::partial(vec![orbstack()], &["/Applications/Docker.app"]);
        let scan = Scan::with(installed, FakeRunningApps::new(vec![running(ORBSTACK)]))
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        let recorded = scan.run(&[desktop_disk(), orbstack_socket()]);

        assert_eq!(recorded.identities(), ["orbstack"]);
        assert_eq!(recorded.summaries(), ["docker.summary.answered"]);
    }

    #[test]
    fn a_stopped_runtime_runs_nothing() {
        let scan = Scan::new(vec![desktop(), orbstack()], vec![running(DOCKER_DESKTOP)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        let recorded = scan.run(&[orbstack_socket()]);

        assert_eq!(recorded.summaries(), ["docker.summary.stopped", "docker.summary.stopped"]);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
    }

    #[test]
    fn a_restored_scan_runs_nothing() {
        let scan = Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER))
            .restored();

        let recorded = scan.run(&[orbstack_socket(), orbstack_disk()]);

        assert_eq!(recorded.summaries(), ["docker.summary.restored"]);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
        assert_eq!(scan.running.calls(), 0);
    }

    #[test]
    fn nothing_is_ever_pushed_as_a_finding() {
        let scans = [
            Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
                .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER)),
            Scan::new(vec![orbstack()], vec![running(ORBSTACK)]),
            Scan::new(vec![desktop(), orbstack()], Vec::new()),
            Scan::new(vec![orbstack()], vec![running(ORBSTACK)]).restored(),
            Scan::new(Vec::new(), Vec::new()),
        ];
        for scan in &scans {
            let recorded = scan.run(&[desktop_socket(), desktop_disk(), orbstack_socket(), orbstack_disk()]);

            assert_eq!(recorded.findings, Vec::<NewFinding>::new());
            assert_eq!(recorded.left_out, Vec::<LeftOutItem>::new());
            assert_ne!(recorded.insights, Vec::<Insight>::new());
        }
    }

    /// Records the insights, and cancels the scan once the first is described.
    struct CancelsAtFirstInsight {
        cancel: CancelToken,
        insights: Vec<Insight>,
    }

    impl fresher_core::FindingSink for CancelsAtFirstInsight {
        fn push(&mut self, _: NewFinding) {}

        fn left_out(&mut self, _: LeftOutItem) {}

        fn describe(&mut self, insight: Insight) {
            self.insights.push(insight);
            self.cancel.cancel();
        }
    }

    #[test]
    fn a_cancelled_scan_stops_between_rows() {
        let cancel = CancelToken::new();
        let mut out = CancelsAtFirstInsight { cancel: cancel.clone(), insights: Vec::new() };
        let scan = Scan::new(vec![desktop(), orbstack()], vec![running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        let result = scan.finish(&[orbstack_socket()], &cancel, &mut out);

        assert_eq!(result, Err(ModuleError::Cancelled));
        assert_eq!(out.insights.len(), 1);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
    }

    #[test]
    fn a_cancelled_read_cancels_the_module() {
        let scan = Scan::new(vec![orbstack()], vec![running(ORBSTACK)]).runner(CancelledRead);
        let mut out = Recorded::default();

        let result = scan.finish(&[orbstack_socket()], &CancelToken::new(), &mut out);

        assert_eq!(result, Err(ModuleError::Cancelled));
        assert_eq!(out.insights, Vec::<Insight>::new());
    }

    #[test]
    fn a_link_at_the_disk_file_s_place_is_not_seen() {
        let link = entry(DESKTOP_DISK, DESKTOP_DISK_PLACE, EntryKind::Symlink, disk_sizes());
        let folder = entry(DESKTOP_DISK, DESKTOP_DISK_PLACE, EntryKind::Folder, disk_sizes());

        assert_eq!(Scan::new(Vec::new(), Vec::new()).run(&[link, folder]).insights, Vec::<Insight>::new());
    }

    #[test]
    fn a_file_at_the_socket_s_place_is_not_a_socket() {
        let file = entry(ORBSTACK_SOCKET, ".orbstack/run/docker.sock", EntryKind::File, disk_sizes());
        let scan = Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        assert_eq!(scan.run(&[file]).summaries(), ["docker.summary.stopped"]);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
    }

    #[test]
    fn a_match_of_no_interest_is_not_seen() {
        let stray = entry(99, ".orbstack/run/docker.sock", EntryKind::Other, disk_sizes());
        let scan = Scan::new(vec![orbstack()], vec![running(ORBSTACK)]);

        assert_eq!(scan.run(&[stray]).summaries(), ["docker.summary.stopped"]);
    }

    #[test]
    fn a_runtime_that_does_not_answer_says_so() {
        let failing = Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
            .commands(FakeCommandRunner::default().exiting(ORBSTACK_READ, 1));
        let unreadable = Scan::new(vec![orbstack()], vec![running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, "TYPE SIZE"));

        assert_eq!(failing.run(&[orbstack_socket()]).summaries(), ["docker.summary.no-answer"]);
        assert_eq!(unreadable.run(&[orbstack_socket()]).summaries(), ["docker.summary.unreadable"]);
    }

    #[test]
    fn it_never_asks_when_the_running_apps_cannot_be_listed() {
        let scan = Scan::with(FakeInstalledApps::new(vec![orbstack()]), FakeRunningApps::failing("no access"))
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        assert_eq!(scan.run(&[orbstack_socket()]).summaries(), ["docker.summary.unknown"]);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
    }

    #[test]
    fn a_program_path_that_is_not_utf_8_is_no_answer() {
        let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"/Applications/Orb\xffStack.app")).to_path_buf();
        let scan = Scan::new(vec![InstalledApp { path, ..orbstack() }], vec![running(ORBSTACK)]);

        assert_eq!(scan.run(&[orbstack_socket()]).summaries(), ["docker.summary.no-answer"]);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
    }

    #[test]
    fn a_place_not_known_adds_no_note() {
        let scan = Scan::new(vec![orbstack()], Vec::new()).place(ORBSTACK_DISK, PlaceSeen::Unknown);

        let recorded = scan.run(&[]);

        assert_eq!(notes(&recorded.insights[0]), ["docker.runtime.orbstack.gives-back"]);
    }
}

mod positive {
    use std::collections::BTreeSet;

    use fresher_core::{ByteSize, Module as _, ModuleGroup, PlaceSeen, Sizes};
    use fresher_testing::FakeCommandRunner;

    use super::{
        ANSWER, DESKTOP_READ, DOCKER_DESKTOP, DockerModule, ORBSTACK, ORBSTACK_DISK, ORBSTACK_READ, Runtimes, Scan,
        answering, desktop, desktop_disk, desktop_socket, figures, notes, orbstack, orbstack_disk, orbstack_socket,
        parts, running,
    };
    use crate::answer::parse;
    use crate::decide::{Disk, State};
    use crate::report;

    #[test]
    fn the_descriptor_has_no_category_no_setting_and_no_problem() {
        let module = DockerModule::new().unwrap();

        let descriptor = module.descriptor();
        assert_eq!(descriptor.id().as_str(), "docker");
        assert_eq!(descriptor.group(), ModuleGroup::Development);
        assert_eq!(descriptor.categories(), []);
        assert_eq!(descriptor.settings(), []);
        assert_eq!(descriptor.problems(), []);
        assert_eq!(module.interests().len(), 4);
    }

    #[test]
    fn every_row_has_its_gives_back_text() {
        let module = DockerModule::new().unwrap();
        let expected: BTreeSet<String> = Runtimes::shipped()
            .unwrap()
            .rows()
            .iter()
            .map(|row| format!("docker.runtime.{}.gives-back", row.id))
            .collect();
        let dictionary: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(module.descriptor().locales().en).unwrap();

        let present: BTreeSet<String> =
            dictionary.keys().filter(|key| key.starts_with("docker.runtime.")).cloned().collect();

        assert_eq!(present, expected);
    }

    #[test]
    fn the_only_call_is_the_read_of_its_runtime() {
        let scan = Scan::new(vec![desktop(), orbstack()], vec![running(DOCKER_DESKTOP), running(ORBSTACK)])
            .commands(FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER));

        scan.run(&[orbstack_socket()]);

        assert_eq!(
            scan.commands.calls(),
            ["/Applications/OrbStack.app/Contents/MacOS/xbin/docker --context orbstack system df --format json"]
        );
    }

    #[test]
    fn an_answered_runtime_has_four_parts_with_their_commands() {
        let recorded = answering(ANSWER);

        assert_eq!(recorded.summaries(), ["docker.summary.answered"]);
        let part = |label: &str, detail: &str, line: Option<&str>| {
            (format!("docker.part.{label}"), format!("docker.part.{detail}"), line.map(str::to_owned))
        };
        assert_eq!(
            parts(&recorded),
            [
                part("build-cache.label", "build-cache.detail", Some("docker --context orbstack builder prune --all")),
                part("images.label", "images.detail", Some("docker --context orbstack image prune --all")),
                part("containers.label", "containers.none", None),
                part("volumes.label", "volumes.detail", Some("docker --context orbstack volume prune")),
            ]
        );
        let insight = &recorded.insights[0];
        assert_eq!(
            figures(insight),
            [("docker.figure.on-disk", 9_000_000_000), ("docker.figure.inside", 3_587_201_090)]
        );
        assert_eq!(notes(insight), ["docker.runtime.orbstack.gives-back", "docker.note.never-runs"]);
        let dangers: Vec<_> = insight.parts().iter().map(|part| part.danger).collect();
        assert_eq!(
            dangers,
            [
                fresher_core::DangerLevel::Safe,
                fresher_core::DangerLevel::Low,
                fresher_core::DangerLevel::Medium,
                fresher_core::DangerLevel::High,
            ]
        );
    }

    #[test]
    fn a_part_with_nothing_unused_has_no_command() {
        let recorded = answering(&ANSWER.replace("1.071GB (35%)", "0B (0%)"));

        let images = &recorded.insights[0].parts()[1];
        assert_eq!(images.size, ByteSize::ZERO);
        assert_eq!(images.detail.key().as_str(), "docker.part.images.none");
        assert_eq!(images.command, None);
    }

    #[test]
    fn a_stopped_container_gives_the_containers_command() {
        let stopped =
            ANSWER.replace(r#""Active":"1","Reclaimable":"0B (0%)""#, r#""Active":"0","Reclaimable":"1.09kB""#);

        let recorded = answering(&stopped);

        let containers = &recorded.insights[0].parts()[2];
        let line = containers.command.as_ref().map(fresher_core::ShownCommand::line);
        assert_eq!(line, Some("docker --context orbstack container prune"));
    }

    #[test]
    fn nothing_unused_anywhere_gives_no_never_runs_note() {
        let none = ANSWER
            .replace("1.071GB (35%)", "0B")
            .replace("41.2MB (100%)", "0B")
            .replace(r#""Reclaimable":"512MB""#, r#""Reclaimable":"0B""#);

        let recorded = answering(&none);

        assert_eq!(notes(&recorded.insights[0]), ["docker.runtime.orbstack.gives-back"]);
    }

    #[test]
    fn the_maximum_is_shown_only_for_rows_that_show_it() {
        let recorded = Scan::new(vec![desktop(), orbstack()], Vec::new()).run(&[desktop_disk(), orbstack_disk()]);

        assert_eq!(
            figures(&recorded.insights[0]),
            [("docker.figure.maximum", 64_000_000_000), ("docker.figure.on-disk", 9_000_000_000)]
        );
        assert_eq!(figures(&recorded.insights[1]), [("docker.figure.on-disk", 9_000_000_000)]);
    }

    #[test]
    fn an_unread_disk_place_adds_the_note() {
        let unread = Scan::new(vec![orbstack()], Vec::new()).place(ORBSTACK_DISK, PlaceSeen::Unread);
        let not_scanned = Scan::new(vec![orbstack()], Vec::new()).place(ORBSTACK_DISK, PlaceSeen::NotScanned);

        let recorded = unread.run(&[]);

        assert_eq!(notes(&recorded.insights[0]), ["docker.note.disk-unseen", "docker.runtime.orbstack.gives-back"]);
        assert_eq!(notes(&not_scanned.run(&[]).insights[0])[0], "docker.note.disk-unseen");
    }

    #[test]
    fn a_left_disk_file_is_described_without_the_gives_back_note() {
        let recorded = Scan::new(Vec::new(), Vec::new()).run(&[desktop_disk()]);

        assert_eq!(recorded.identities(), ["docker-desktop"]);
        assert_eq!(recorded.summaries(), ["docker.summary.left"]);
        assert_eq!(notes(&recorded.insights[0]), Vec::<&str>::new());
    }

    #[test]
    fn two_runtimes_give_two_insights_in_table_order() {
        let commands = FakeCommandRunner::default().answering(ORBSTACK_READ, ANSWER).answering(DESKTOP_READ, ANSWER);
        let scan =
            Scan::new(vec![orbstack(), desktop()], vec![running(ORBSTACK), running(DOCKER_DESKTOP)]).commands(commands);

        let recorded = scan.run(&[orbstack_socket(), orbstack_disk(), desktop_socket(), desktop_disk()]);

        assert_eq!(recorded.identities(), ["docker-desktop", "orbstack"]);
        assert_eq!(recorded.summaries(), ["docker.summary.answered", "docker.summary.answered"]);
        assert_eq!(scan.commands.calls(), [DESKTOP_READ, ORBSTACK_READ]);
        assert_eq!(scan.running.calls(), 1);
        assert_eq!(scan.installed.calls(), 1);
        let first = recorded.insights[0].parts()[0].command.as_ref().map(|command| command.line().to_owned());
        assert_eq!(first.as_deref(), Some("docker --context desktop-linux builder prune --all"));
    }

    #[test]
    fn the_shipped_table_builds_every_state_without_an_error() {
        let usage = parse(ANSWER).unwrap();
        let states = [
            State::Left,
            State::Restored,
            State::Unknown,
            State::Stopped,
            State::NoAnswer,
            State::Unreadable,
            State::Answered(usage),
        ];
        let disks =
            [Disk::Seen(Sizes::complete(ByteSize::new(u64::MAX), ByteSize::new(u64::MAX))), Disk::Absent, Disk::Unseen];
        for row in Runtimes::shipped().unwrap().rows() {
            for state in &states {
                for disk in &disks {
                    let insight = report::insight(row, state, disk);

                    assert!(insight.is_ok(), "{} {state:?} {disk:?}: {insight:?}", row.id);
                }
            }
        }
    }
}
