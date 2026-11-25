//! Sandbox mode over a generated sandbox, with fakes for the ports that stay real in the app. The real home and the
//! development build's data folder are temporary folders too: no test touches the developer's own.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use fresher_core::{
    Access, ByteSize, Capability, CleanupPlanView, CleanupSelection, Confirmation, DangerLevel, Event, FindingOrder,
    FindingsQuery, Protection, SANDBOX_MARKER, SettingUpdate, SettingValue, Subject, VolumeFacts, VolumeId,
};
use fresher_engine::{Engine, ScanMode};
use fresher_sandbox::{CaseFile, Sandbox, generate};
use fresher_testing::{
    FIXTURE_NOW, FakeAccess, FakeClock, FakeCommandRunner, FakeInstallRecords, FakeOpenFiles, FakeRunningApps,
    FakeVolumes, StatedFacts,
};
use serde_json::json;
use tempfile::TempDir;

use super::{SandboxMode, SandboxPorts, SandboxTrash, compose};
use crate::compose::ComposeError;

/// A `trial` sandbox: a trial cache with no marker next to it, and one next to a marker of its own.
const TRIAL: &str = r#"
feature = "trial"
module = "trial"

[[case]]
name = "a trial folder with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/Sample app cache/blob.bin"
size = 4096

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/Sample app cache/blob.bin"
size = 4096
fill = "data"
[case.expect]
item = "home/work/Sample app cache"
category = "trial.caches"
danger = "safe"
after-cleanup = "in-trash"
"#;

/// A generated sandbox next to a stand-in for the real Mac: its home and the development build's data folder.
struct World {
    folder: TempDir,
    real: TempDir,
}

impl World {
    fn new() -> Self {
        let (folder, real) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        fs::create_dir(real.path().join("home")).unwrap();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
        let files = [CaseFile::parse(TRIAL).unwrap()];
        generate(&files, folder.path(), now, &Protection::default(), user(folder.path())).unwrap();
        Self { folder, real }
    }

    fn sandbox(&self) -> Sandbox {
        Sandbox::open(self.folder.path(), user(self.folder.path())).unwrap()
    }

    fn real_home(&self) -> PathBuf {
        self.real.path().join("home")
    }

    /// The development build's own data folder, which sandbox mode must leave alone.
    fn real_data(&self) -> PathBuf {
        self.real.path().join("data")
    }

    /// Sandbox mode on `path`, with the sandbox's own Trash.
    fn start_at(&self, path: &Path) -> Result<Engine, ComposeError> {
        self.start_with_home(path, &self.real_home())
    }

    /// Sandbox mode on `path` on a Mac whose real home is `real_home`.
    fn start_with_home(&self, path: &Path, real_home: &Path) -> Result<Engine, ComposeError> {
        let mode = SandboxMode::new(path, SandboxTrash::Sandbox);
        compose(&mode, real_home, &self.real_data(), |_| Ok(fakes())).map(|(engine, _)| engine)
    }

    /// Sandbox mode on the world's sandbox, the agreement accepted as before the first cleanup.
    fn start(&self) -> Engine {
        let engine = self.start_at(self.folder.path()).unwrap();
        engine.accept_agreement(fresher_core::AGREEMENT_VERSION).unwrap();
        engine
    }
}

fn user(folder: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt as _;
    fs::metadata(folder).unwrap().uid()
}

/// A sandbox generated from `cases`, an app that says it was installed 10 days ago and an installer receipt among them.
const INSTALLS: &str = r#"
feature = "installs"

[[case]]
name = "an app and a receipt with an age"
kind = "negative"
macos = "15.4"
[[case.app]]
path = "Applications/Tool.app"
bundle-id = "com.example.Tool"
name = "Tool"
changed = "10d"
[[case.receipt]]
package = "Tool-3.1.pkg"
installed = "10d"
"#;

/// The sandbox of [`INSTALLS`], generated at `FIXTURE_NOW`.
fn installs_sandbox() -> (TempDir, Sandbox) {
    let folder = tempfile::tempdir().unwrap();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
    generate(&[CaseFile::parse(INSTALLS).unwrap()], folder.path(), now, &Protection::default(), user(folder.path()))
        .unwrap();
    let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
    (folder, sandbox)
}

/// Fakes for every port sandbox mode keeps real: no clock, volume, open file, running app or tool of the machine.
fn fakes() -> SandboxPorts {
    let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
    SandboxPorts {
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        volumes: Arc::new(FakeVolumes::default().with("/", facts)),
        open_files: Arc::new(FakeOpenFiles::default()),
        running: Arc::new(FakeRunningApps::default()),
        commands: Arc::new(FakeCommandRunner::default()),
        access: Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)])),
        installs: Arc::new(FakeInstallRecords::default()),
        stated: StatedFacts::new(),
    }
}

/// Sandbox mode on the world's sandbox with its stated facts, as the real ports take them from its `world.toml`.
fn start_stated(world: &World) -> Engine {
    let mode = SandboxMode::new(world.folder.path(), SandboxTrash::Sandbox);
    let ports = |sandbox: &Sandbox| {
        let stated = fresher_sandbox::World::read(sandbox).map_err(super::SandboxRefusal::World)?;
        Ok(SandboxPorts { stated: stated.stated(sandbox.path()), ..fakes() })
    };
    compose(&mode, &world.real_home(), &world.real_data(), ports).unwrap().0
}

/// Writes `[access]` with `downloads = "granted"` into the world's `world.toml`, as a case stating it would.
fn grant_downloads(world: &World, access: &str) {
    let sandbox = world.sandbox();
    let text = fs::read_to_string(sandbox.world()).unwrap() + &format!("\n[access]\ndownloads = \"{access}\"\n");
    fs::write(sandbox.world(), text).unwrap();
}

/// Turns on the switch of Downloads in Settings → Permissions, as the user does before a scan reads it.
fn switch_on_downloads(engine: &Engine) {
    let update = SettingUpdate { key: "core.access.downloads".into(), value: SettingValue::Switch(true) };
    engine.update_setting(update).unwrap();
}

/// Sandbox mode on the world's sandbox with the access its `world.toml` states, as the real ports take it.
fn start_with_access(world: &World) -> Engine {
    let mode = SandboxMode::new(world.folder.path(), SandboxTrash::Sandbox);
    let ports = |sandbox: &Sandbox| {
        let read = fresher_sandbox::World::read(sandbox).map_err(super::SandboxRefusal::World)?;
        let access = super::access_of(&read, &sandbox.home(), fakes().access);
        Ok(SandboxPorts { access, ..fakes() })
    };
    compose(&mode, &world.real_home(), &world.real_data(), ports).unwrap().0
}

/// Writes `home/Downloads/x` into the sandbox, large enough to be its own node of the scan.
fn download(world: &World) -> PathBuf {
    let downloads = world.sandbox().home().join("Downloads");
    fs::create_dir_all(&downloads).unwrap();
    fs::write(downloads.join("x"), vec![1_u8; 2 << 20]).unwrap();
    downloads.join("x")
}

/// Whether the last scan's disk state has a node at `path`.
fn has_node(engine: &Engine, path: &Path) -> bool {
    engine.disk_state().unwrap().is_some_and(|state| state.find(path).is_some())
}

/// A full scan, waited for: its coverage.
fn scan_coverage(engine: &Engine) -> fresher_core::Coverage {
    let events = engine.subscribe();
    engine.start_scan(ScanMode::Full).unwrap();
    let mut coverage = None;
    until(&events, |event| {
        if let Event::ScanCoverage { coverage: seen, .. } = event {
            coverage = Some(seen.clone());
        }
        matches!(event, Event::ScanFinished { .. })
    });
    coverage.unwrap()
}

/// The removability of every `trial` finding, by path.
fn removability(engine: &Engine) -> Vec<(PathBuf, fresher_core::Removability)> {
    let query = FindingsQuery::new(Some("trial".into()), None, FindingOrder::Found, 0, 1000);
    let page = engine.findings(&query).unwrap();
    page.findings
        .iter()
        .filter_map(|finding| match finding.finding().subject() {
            Subject::Path(path) => Some((path.clone(), finding.removability())),
            Subject::External { .. } => None,
        })
        .collect()
}

/// A full scan, waited for.
fn scan(engine: &Engine) {
    let events = engine.subscribe();
    engine.start_scan(ScanMode::Full).unwrap();
    until(&events, |event| matches!(event, Event::ScanFinished { .. }));
}

/// The paths of every finding of the `trial` module (the `demo` module's are made up), sorted.
fn found(engine: &Engine) -> Vec<PathBuf> {
    let query = FindingsQuery::new(Some("trial".into()), None, FindingOrder::Found, 0, 1000);
    let page = engine.findings(&query).unwrap();
    let mut paths: Vec<PathBuf> = page
        .findings
        .iter()
        .filter_map(|finding| match finding.finding().subject() {
            Subject::Path(path) => Some(path.clone()),
            Subject::External { .. } => None,
        })
        .collect();
    paths.sort();
    paths
}

/// The cleanup of every trial cache, prepared.
fn prepare(engine: &Engine) -> CleanupPlanView {
    let selection: CleanupSelection =
        serde_json::from_value(json!({ "findings": [], "categories": [{ "category": "trial.caches", "except": [] }] }))
            .unwrap();
    engine.prepare_cleanup(&selection).unwrap()
}

/// Confirms `view` to the Trash, as the dialog sends it, and waits for the cleanup's end.
fn clean(engine: &Engine, view: &CleanupPlanView) {
    let summary = view.summary;
    let echo: Confirmation = serde_json::from_value(json!({ "plan": view.plan, "count": summary.count,
        "allocated": summary.sizes.allocated, "highest": summary.highest, "method": "trash",
        "highRiskUnderstood": summary.highest == DangerLevel::High }))
    .unwrap();
    let events = engine.subscribe();
    engine.confirm_cleanup(&echo).unwrap();
    until(&events, |event| matches!(event, Event::CleanupFinished { .. }));
}

/// Waits for the first event that `is_last`; fails after 30 s instead of hanging.
fn until(events: &Receiver<Event>, mut is_last: impl FnMut(&Event) -> bool) {
    while !is_last(&events.recv_timeout(Duration::from_secs(30)).unwrap()) {}
}

/// The Trash view of sandbox mode with `trash`, after a cleanup in the journal that moved a 5-byte file to the Mac's
/// own home Trash (the stand-in real home's `.Trash`), where it still is.
fn trash_after_a_move_to_the_macs_trash(world: &World, trash: SandboxTrash) -> fresher_core::TrashView {
    use std::os::unix::fs::MetadataExt as _;

    use fresher_core::{
        ActionKind, BeginRecord, CleanupJournal as _, DangerLevel, EndRecord, Ended, EntryKind, ItemIdentity,
        ItemRecord, JournalItem, Method, RecordedOutcome, Sizes, Text, TextKey, Timestamp, TrashedItem,
    };

    let landed = world.real_home().join(".Trash/old.log");
    fs::create_dir_all(landed.parent().unwrap()).unwrap();
    fs::write(&landed, b"hello").unwrap();
    let metadata = fs::symlink_metadata(&landed).unwrap();
    let identity = ItemIdentity::new(metadata.dev(), metadata.ino(), EntryKind::File);
    let original = world.sandbox().home().join("Library/Logs/old.log");
    let journal = fresher_store::FileCleanupJournal::open(
        &fresher_store::DataFolder::open(world.sandbox().data_folder()).unwrap(),
    );
    let item = JournalItem {
        index: 0,
        path: Some(original.clone()),
        identity: Some(identity),
        more: Vec::new(),
        action: ActionKind::Remove,
        category: "trial.caches".into(),
        key: "old.log".into(),
        reason: Text::new(TextKey::new("test.finding.reason").unwrap()),
        danger: DangerLevel::Safe,
        sizes: Sizes::complete(ByteSize::new(5), ByteSize::new(4_096)),
    };
    let at = Timestamp::from_secs(FIXTURE_NOW);
    let begin = BeginRecord { at, method: Method::Trash, roots: vec![], space: vec![], items: vec![item] };
    let id = journal.begin(&begin).unwrap();
    let trashed = vec![TrashedItem { original, landed, identity }];
    journal.item(&ItemRecord { cleanup: id, index: 0, outcome: RecordedOutcome::Moved, trashed, went: None }).unwrap();
    journal.end(&EndRecord { cleanup: id, at, ended: Ended::Completed, space: vec![] }).unwrap();
    let mode = SandboxMode::new(world.folder.path(), trash);
    let (engine, _) = compose(&mode, &world.real_home(), &world.real_data(), |_| Ok(fakes())).unwrap();
    engine.trash()
}

/// An environment of `vars`, as `std::env::var_os` reads it.
fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let vars: Vec<(String, OsString)> = vars.iter().map(|(name, value)| ((*name).into(), (*value).into())).collect();
    move |name| vars.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone())
}

mod negative {
    use std::fs;

    use fresher_core::{SANDBOX_MARKER, SettingUpdate, SettingValue, SkipReason};
    use fresher_testing::FixtureTree;

    use super::{World, env, found, prepare, scan};
    use crate::compose::ComposeError;
    use crate::sandbox::{SandboxMode, SandboxRefusal};

    /// The refusal of sandbox mode, if `result` is one.
    fn refused(result: Result<fresher_engine::Engine, ComposeError>) -> Option<SandboxRefusal> {
        match result {
            Err(ComposeError::Sandbox(refusal)) => Some(refusal),
            _ => None,
        }
    }

    #[test]
    fn the_sandboxs_own_trash_does_not_count_the_macs_home_trash() {
        let world = World::new();

        let view = super::trash_after_a_move_to_the_macs_trash(&world, crate::sandbox::SandboxTrash::Sandbox);

        assert_eq!(view.trashes, []);
    }

    #[test]
    fn the_real_trash_names_no_sandbox_path() {
        let world = World::new();

        let build = crate::sandbox::build(&world.sandbox(), crate::sandbox::SandboxTrash::Real).unwrap();

        assert!(
            matches!(
                &build,
                fresher_core::Build::Sandbox { trash, .. }
                    if trash.key().as_str() == "core.build.trash.system" && trash.args().is_empty()
            ),
            "{build:?}"
        );
    }

    #[test]
    fn a_folder_without_a_marker_does_not_start() {
        let world = World::new();
        fs::rename(world.folder.path().join(SANDBOX_MARKER), world.real.path().join(SANDBOX_MARKER)).unwrap();

        let refusal = refused(world.start_at(world.folder.path()));

        assert!(matches!(refusal, Some(SandboxRefusal::Folder(fresher_cleaner::SandboxError::NoMarker { .. }))));
    }

    #[test]
    fn a_folder_through_a_link_does_not_start() {
        let world = World::new();
        let link = world.real.path().join("alias");
        std::os::unix::fs::symlink(world.folder.path(), &link).unwrap();

        let refusal = refused(world.start_at(&link));

        assert!(matches!(refusal, Some(SandboxRefusal::Folder(fresher_cleaner::SandboxError::ThroughLink { .. }))));
    }

    #[test]
    fn a_sandbox_without_its_home_does_not_start() {
        let world = World::new();
        let home = world.sandbox().home();
        fs::rename(&home, world.real.path().join("moved-home")).unwrap();

        let refusal = refused(world.start_at(world.folder.path()));

        assert!(matches!(refusal, Some(SandboxRefusal::Layout(fresher_sandbox::SandboxError::MissingPart { .. }))));
    }

    #[test]
    fn the_real_home_does_not_start() {
        let world = World::new();
        let folder = world.folder.path();

        let home = refused(world.start_with_home(folder, folder));
        let above = refused(world.start_with_home(folder, &folder.join("home")));

        assert!(matches!(home, Some(SandboxRefusal::Folder(fresher_cleaner::SandboxError::Protected { .. }))));
        assert!(matches!(above, Some(SandboxRefusal::Folder(fresher_cleaner::SandboxError::Protected { .. }))));
    }

    #[test]
    fn an_unknown_trash_choice_does_not_start() {
        for value in ["", "Real", "fake", "real "] {
            let refusal =
                SandboxMode::from_env(env(&[("FRESHER_SANDBOX", "/tmp/s"), ("FRESHER_SANDBOX_TRASH", value)]));

            assert!(matches!(refusal, Err(SandboxRefusal::UnknownTrash { .. })), "{value:?}");
        }
        let alone = SandboxMode::from_env(env(&[("FRESHER_SANDBOX_TRASH", "real")]));
        assert!(matches!(alone, Err(SandboxRefusal::TrashWithoutSandbox)));
    }

    #[test]
    fn sandbox_mode_withholds_high_by_default() {
        let mode = SandboxMode::from_env(env(&[("FRESHER_SANDBOX", "/tmp/s")])).unwrap().unwrap();

        assert_eq!(mode.high, fresher_core::HighLevel::Withheld);
        assert_eq!(SandboxMode::new("/tmp/s", crate::sandbox::SandboxTrash::Sandbox).high, mode.high);
    }

    #[test]
    fn a_relative_sandbox_path_does_not_start() {
        let refusal = SandboxMode::from_env(env(&[("FRESHER_SANDBOX", "sandbox/trial")]));

        assert!(matches!(refusal, Err(SandboxRefusal::NotAbsolute { .. })));
    }

    #[test]
    fn the_sandbox_engine_never_writes_the_normal_data_folder() {
        let world = World::new();
        let engine = world.start();

        scan(&engine);
        let view = prepare(&engine);
        super::clean(&engine, &view);

        assert!(!world.real_data().exists());
        assert_eq!(fs::read_dir(world.real_home()).unwrap().count(), 0, "the real home is untouched");
    }

    /// The sandbox's scan root is `~`: the folder its world states is decided at the sandbox's home, so a denied
    /// Downloads, or a granted one whose switch is off, is not entered and the coverage says so.
    #[test]
    fn a_world_denying_downloads_or_granting_it_with_its_switch_off_keeps_the_walk_out() {
        let downloads = fresher_core::Capability::Folder(fresher_core::GuardedFolder::Downloads);
        for (access, switch) in [("denied", true), ("granted", false)] {
            let world = World::new();
            super::grant_downloads(&world, access);
            let x = super::download(&world);
            let engine = super::start_with_access(&world);
            if switch {
                super::switch_on_downloads(&engine);
            }

            let coverage = super::scan_coverage(&engine);

            assert!(!super::has_node(&engine, &x), "{access}: the walk entered home/Downloads");
            assert!(coverage.not_entered.iter().any(|not| not.capability == downloads), "{access}: {coverage:?}");
        }
    }

    /// A place of the real home that may raise a system prompt is never opened, even as a scan root.
    #[test]
    fn the_real_homes_places_that_prompt_are_not_walked() {
        let world = World::new();
        let documents = fs::canonicalize(world.real_home()).unwrap().join("Documents");
        fs::create_dir_all(documents.join("Sample app cache")).unwrap();
        fs::write(documents.join("Sample app cache/blob.bin"), [0; 64]).unwrap();
        fs::write(documents.join(SANDBOX_MARKER), format!("{}\n", fresher_core::SANDBOX_SIGNATURE)).unwrap();
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~".into(), documents.clone()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);

        assert!(found(&engine).iter().all(|path| !path.starts_with(&documents)), "{:?}", found(&engine));
    }

    /// Sandbox mode keeps its standing list: the guarded places of the sandbox's home and of the real home are never
    /// opened, though its access knows no guarded place.
    #[test]
    fn a_sandbox_engine_still_avoids_the_places_of_both_homes() {
        let world = World::new();
        let sandbox_home = fs::canonicalize(world.sandbox().home()).unwrap();
        let real_home = fs::canonicalize(world.real_home()).unwrap();
        let guarded = [sandbox_home.join("Library/Containers/com.example.app"), real_home.join("Desktop")];
        for place in &guarded {
            fs::create_dir_all(place.join("Sample app cache")).unwrap();
            fs::write(place.join("Sample app cache/blob.bin"), [0; 64]).unwrap();
            fs::write(place.join(SANDBOX_MARKER), format!("{}\n", fresher_core::SANDBOX_SIGNATURE)).unwrap();
        }
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~".into(), real_home.join("Desktop")]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);

        let found = found(&engine);
        assert!(!found.is_empty(), "the sandbox's own trial cache is found");
        assert!(found.iter().all(|path| guarded.iter().all(|place| !path.starts_with(place))), "{found:?}");
    }

    /// A world granting Downloads opens the sandbox home's, never the real home's: a scan root there finds nothing.
    #[test]
    fn the_real_homes_guarded_places_stay_avoided_whatever_the_world() {
        let world = World::new();
        super::grant_downloads(&world, "granted");
        let downloads = fs::canonicalize(world.real_home()).unwrap().join("Downloads");
        fs::create_dir_all(downloads.join("Sample app cache")).unwrap();
        fs::write(downloads.join("Sample app cache/blob.bin"), [0; 64]).unwrap();
        fs::write(downloads.join(SANDBOX_MARKER), format!("{}\n", fresher_core::SANDBOX_SIGNATURE)).unwrap();
        let engine = super::start_with_access(&world);
        let roots = SettingValue::Folders(vec!["~".into(), downloads.clone()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);

        assert!(!found(&engine).is_empty(), "the sandbox's own trial cache is found");
        assert!(found(&engine).iter().all(|path| !path.starts_with(&downloads)), "{:?}", found(&engine));
        assert!(!super::has_node(&engine, &downloads.join("Sample app cache")));
    }

    #[test]
    fn a_world_without_access_keeps_the_sandbox_homes_downloads_avoided() {
        let world = World::new();
        let x = super::download(&world);
        let engine = super::start_with_access(&world);

        scan(&engine);

        assert!(super::has_node(&engine, &world.sandbox().home().join("work")), "the walk ran");
        assert!(!super::has_node(&engine, &x));
    }

    #[test]
    fn the_sandbox_engine_names_its_refusal() {
        let world = World::new();
        let mode = SandboxMode::new(world.real.path(), crate::sandbox::SandboxTrash::Sandbox);

        let error =
            crate::sandbox::sandbox_engine(&mode, &world.real_home(), &world.real_data(), super::fakes()).unwrap_err();

        assert!(matches!(error.refusal(), Some(SandboxRefusal::Folder(_))), "{error:?}");
    }

    /// The Mac running the test has apps open; sandbox mode's ports list none of them, only the sandbox's (none here).
    #[test]
    fn the_real_ports_never_take_the_macs_running_apps() {
        let world = World::new();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        assert_eq!(fresher_sandbox::World::read(&world.sandbox()).unwrap().running(), []);
        assert_eq!(ports.running.list(), Ok(Vec::new()));
    }

    /// A program the world does not state is never run, even one that would harm nothing: it answers not installed.
    #[test]
    fn the_real_ports_run_no_program_of_the_mac_but_open() {
        let world = World::new();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        let answer = ports.commands.run("/bin/echo", &["hi"], &fresher_core::CancelToken::new());
        assert_eq!(answer, Err(fresher_core::CommandRunError::NotInstalled { program: "/bin/echo".into() }));
    }

    /// A sandbox with no generation moment (no `world.toml`, or one written before it held one) runs on the system's
    /// clock.
    #[test]
    fn the_real_ports_without_a_world_moment_take_the_systems_clock() {
        let world = World::new();
        fs::write(world.sandbox().world(), "").unwrap();
        let before = std::time::SystemTime::now();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        assert!(ports.clock.now() >= before);
    }

    #[test]
    fn the_real_ports_of_a_world_stating_no_volume_answer_the_macs() {
        let world = World::new();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        assert!(ports.volumes.space(world.sandbox().path()).is_ok());
    }

    #[test]
    fn a_world_that_is_not_valid_does_not_start() {
        let world = World::new();
        fs::write(world.sandbox().world(), "[[running]]\nname = \"CLion\"\nbundle-ids = [\"not an id\"]\n").unwrap();
        let mode = SandboxMode::new(world.folder.path(), crate::sandbox::SandboxTrash::Sandbox);

        let result =
            crate::sandbox::compose(&mode, &world.real_home(), &world.real_data(), crate::sandbox::SandboxPorts::real);

        let refusal = refused(result.map(|(engine, _)| engine));
        assert!(matches!(refusal, Some(SandboxRefusal::World(fresher_sandbox::SandboxError::World { .. }))));
    }

    /// Without a stated fact, the disk's facts stand: the user's own trial cache can be removed.
    #[test]
    fn a_path_with_no_stated_fact_reads_as_the_disk_has_it() {
        let world = World::new();
        let engine = super::start_stated(&world);

        scan(&engine);

        let home = world.sandbox().home();
        let found = super::removability(&engine);
        assert_eq!(found, [(home.join("work/Sample app cache"), fresher_core::Removability::Removable)]);
        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();
        assert!(ports.stated.is_empty());
    }

    /// A scan root outside the sandbox is scanned, but nothing in it can be removed.
    #[test]
    fn an_item_outside_the_sandbox_is_left_out() {
        let world = World::new();
        let outside = FixtureTree::from_toml_str("[[file]]\npath = \"Sample app cache/blob.bin\"\nsize = 4096\n")
            .unwrap()
            .with_sandbox()
            .unwrap();
        let root = fs::canonicalize(outside.root()).unwrap();
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~".into(), root.clone()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);
        let view = prepare(&engine);

        assert!(found(&engine).contains(&root.join("Sample app cache")));
        assert_eq!(view.left_out.len(), 1, "{view:?}");
        assert_eq!(view.left_out[0].reason(), &SkipReason::NotInSandbox);
        assert!(root.join("Sample app cache/blob.bin").exists());
    }
}

mod positive {
    use std::fs;

    use std::sync::Arc;

    use fresher_core::{SandboxTrashAt, SettingUpdate, SettingValue, Simulated};

    use super::{SANDBOX_MARKER, World, clean, env, found, prepare, scan};
    use crate::sandbox::{SandboxMode, SandboxTrash};

    #[test]
    fn offering_high_offers_it() {
        let mode = SandboxMode::new("/tmp/s", SandboxTrash::Sandbox).offering_high();

        assert_eq!(mode.high, fresher_core::HighLevel::Offered);
    }

    #[test]
    fn the_own_trash_names_its_folder() {
        let world = World::new();
        let sandbox = world.sandbox();

        let build = crate::sandbox::build(&sandbox, SandboxTrash::Sandbox).unwrap();

        // The folder the Trash port of `compose` is given, where `a_cleanup_lands_in_the_sandbox_trash` finds items.
        let folder = fresher_core::TextArg::Path(sandbox.trash());
        assert!(
            matches!(
                &build,
                fresher_core::Build::Sandbox { trash, .. }
                    if trash.key().as_str() == "core.build.trash.sandbox" && trash.args().get("path") == Some(&folder)
            ),
            "{build:?}"
        );
    }

    #[test]
    fn the_real_ports_take_the_sandboxs_declared_running_apps() {
        let world = World::new();
        let declared = "[[running]]\nname = \"CLion\"\nbundle-ids = [\"com.jetbrains.CLion\"]\n";
        fs::write(world.sandbox().world(), declared).unwrap();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        let running = ports.running.list().unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!((running[0].name.as_str(), running[0].bundle_ids[0].as_str()), ("CLion", "com.jetbrains.CLion"));
    }

    #[test]
    fn a_world_that_declares_nothing_of_spotlight_searches_the_listed_apps_only() {
        let world = World::new();
        let sandbox = world.sandbox();
        let declared = fresher_sandbox::World::read(&sandbox).unwrap();

        assert_eq!(crate::sandbox::app_search(&declared, sandbox.path()), fresher_system::DeclaredSearch::listed());
        assert!(matches!(
            crate::sandbox::build(&sandbox, SandboxTrash::Sandbox).unwrap(),
            fresher_core::Build::Sandbox { simulated, .. } if simulated == [Simulated::Running]
        ));
    }

    #[test]
    fn the_sandboxs_search_answers_its_apps_elsewhere_and_its_spotlight_off() {
        let world = World::new();
        let sandbox = world.sandbox();
        let declared = "spotlight = \"off\"\n[[elsewhere]]\nname = \"Far\"\nbundle-id = \"com.example.Far\"\n";
        fs::write(sandbox.world(), declared).unwrap();
        let read = fresher_sandbox::World::read(&sandbox).unwrap();

        let search = crate::sandbox::app_search(&read, sandbox.path());

        let far = fresher_core::BundleId::new("com.example.Far").unwrap();
        let answered = sandbox.path().join("Volumes/Elsewhere/Far.app");
        assert_eq!(search, fresher_system::DeclaredSearch { index_on: false, elsewhere: vec![(far, answered)] });
        assert!(matches!(
            crate::sandbox::build(&sandbox, SandboxTrash::Sandbox).unwrap(),
            fresher_core::Build::Sandbox { simulated, .. } if simulated == [Simulated::Running, Simulated::Spotlight]
        ));
    }

    #[test]
    fn the_banner_names_tools_when_the_world_states_them() {
        let world = World::new();
        let sandbox = world.sandbox();
        let tool = "[[tool]]\nprogram = \"Applications/OrbStack.app/Contents/MacOS/xbin/docker\"\nargs = [\"info\"]\n";
        fs::write(sandbox.world(), tool).unwrap();

        assert!(matches!(
            crate::sandbox::build(&sandbox, SandboxTrash::Sandbox).unwrap(),
            fresher_core::Build::Sandbox { simulated, .. } if simulated == [Simulated::Running, Simulated::Tools]
        ));
    }

    /// The switch is turned on first: as on a Mac without Full Disk Access, a granted folder is read only with it.
    #[test]
    fn a_world_granting_downloads_lets_the_walk_into_the_sandbox_downloads() {
        let world = World::new();
        super::grant_downloads(&world, "granted");
        let x = super::download(&world);
        let engine = super::start_with_access(&world);
        super::switch_on_downloads(&engine);

        let coverage = super::scan_coverage(&engine);

        assert!(super::has_node(&engine, &x), "the walk entered home/Downloads");
        let downloads = fresher_core::Capability::Folder(fresher_core::GuardedFolder::Downloads);
        assert!(coverage.not_entered.iter().all(|not| not.capability != downloads), "{coverage:?}");
    }

    #[test]
    fn the_banner_lists_access_when_the_world_states_it() {
        let world = World::new();
        super::grant_downloads(&world, "granted");

        let build = crate::sandbox::build(&world.sandbox(), SandboxTrash::Sandbox).unwrap();

        assert_eq!(
            build,
            fresher_core::Build::Sandbox {
                path: world.sandbox().path().to_path_buf(),
                simulated: vec![Simulated::Running, Simulated::Access],
                trash: SandboxTrashAt::Sandbox(&world.sandbox().trash()).text(),
            }
        );
        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();
        assert_eq!(
            ports.access.status().access(&fresher_core::Capability::Folder(fresher_core::GuardedFolder::Downloads)),
            fresher_core::Access::Granted
        );
    }

    #[test]
    fn the_sandboxs_apps_folder_opens_only_whole_bundles() {
        let (_folder, sandbox) = super::installs_sandbox();
        let apps = sandbox.path().join("Applications");

        let protection = crate::sandbox::apps_protection(&sandbox).unwrap();

        assert_eq!(protection.app_folders(), std::slice::from_ref(&apps));
        assert_eq!(protection.check(&apps.join("Tool.app")), Ok(()));
        assert_eq!(protection.check(&apps.join("Utilities/Tool.app")), Ok(()));
        for path in [apps.clone(), apps.join("Tool.app/Contents"), apps.join("Tool.app/Contents/Info.plist")] {
            assert_eq!(protection.check(&path), Err(fresher_core::Protected::Never), "{}", path.display());
        }
    }

    #[test]
    fn sandbox_mode_sees_an_apps_stated_change() {
        let (_folder, sandbox) = super::installs_sandbox();
        let world = fresher_sandbox::World::read(&sandbox).unwrap();
        let ports = crate::sandbox::SandboxPorts::real(&sandbox).unwrap();

        let apps =
            crate::sandbox::installed_apps(&world, sandbox.path(), Arc::new(sandbox.known_folders()), &ports.stated);

        let listed = apps.list().unwrap().apps;
        assert_eq!(listed.len(), 1);
        let ten_days = fresher_testing::FIXTURE_NOW - 10 * 24 * 3600;
        assert_eq!(listed[0].changed, Some(fresher_core::Timestamp::from_secs(ten_days)));
    }

    #[test]
    fn the_banner_lists_installs_when_the_world_states_them() {
        let (_folder, sandbox) = super::installs_sandbox();

        let build = crate::sandbox::build(&sandbox, SandboxTrash::Sandbox).unwrap();

        assert_eq!(
            build,
            fresher_core::Build::Sandbox {
                path: sandbox.path().to_path_buf(),
                simulated: vec![Simulated::Running, Simulated::Installs],
                trash: SandboxTrashAt::Sandbox(&sandbox.trash()).text(),
            }
        );
        let ports = crate::sandbox::SandboxPorts::real(&sandbox).unwrap();
        assert_eq!(ports.installs.system_version(), Ok(fresher_core::SystemVersion { major: 15, minor: 4 }));
        assert_eq!(ports.installs.receipts().unwrap().len(), 1);
    }

    #[test]
    fn a_world_stating_no_installs_never_asks_the_macs() {
        let world = World::new();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        assert!(ports.installs.system_version().is_err());
        assert_eq!(ports.installs.receipts().unwrap(), []);
    }

    #[test]
    fn the_real_ports_clock_starts_at_the_moment_the_sandbox_was_generated() {
        let world = World::new();
        fs::write(world.sandbox().world(), format!("now = {}\n", fresher_testing::FIXTURE_NOW)).unwrap();
        let start = std::time::UNIX_EPOCH + std::time::Duration::from_secs(fresher_testing::FIXTURE_NOW);

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        let now = ports.clock.now();
        assert!(now >= start && now < start + std::time::Duration::from_secs(60), "{now:?}");
    }

    #[test]
    fn the_real_ports_of_a_world_stating_a_volume_answer_its_space() {
        let world = World::new();
        let volume =
            "[volume]\nname = \"Macintosh HD\"\ncapacity = 994662584320\nused = 701202362368\nfree = 293460221952\n";
        fs::write(world.sandbox().world(), volume).unwrap();

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        let space = ports.volumes.space(world.sandbox().path()).unwrap();
        let bytes = fresher_core::ByteSize::new;
        assert_eq!(space.name, "Macintosh HD");
        assert_eq!(
            (space.capacity, space.used, space.free),
            (bytes(994_662_584_320), bytes(701_202_362_368), bytes(293_460_221_952))
        );
        assert!(ports.volumes.identify(world.sandbox().path()).is_ok());
    }

    #[test]
    fn sandbox_modes_reader_reports_a_stated_fact() {
        let world = World::new();
        let sandbox = world.sandbox();
        let entry =
            "\n[[entry]]\npath = \"home/work/Sample app cache\"\nowner = \"system\"\nflags = [\"system-lock\"]\n";
        let text = fs::read_to_string(sandbox.world()).unwrap() + entry;
        fs::write(sandbox.world(), text).unwrap();
        let engine = super::start_stated(&world);

        scan(&engine);

        let ports = crate::sandbox::SandboxPorts::real(&sandbox).unwrap();
        let cache = sandbox.home().join("work/Sample app cache");
        assert_eq!(
            ports.stated.get(&cache).and_then(|stated| stated.owner),
            Some(fresher_testing::StatedOwner::System)
        );
        let found = super::removability(&engine);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, cache);
        assert_ne!(found[0].1, fresher_core::Removability::Removable);
    }

    #[test]
    fn no_variable_is_the_normal_build() {
        assert_eq!(SandboxMode::from_env(env(&[])).unwrap(), None);
    }

    #[test]
    fn the_variables_choose_the_sandbox_and_its_trash() {
        let own = SandboxMode::from_env(env(&[("FRESHER_SANDBOX", "/tmp/s")])).unwrap();
        let real =
            SandboxMode::from_env(env(&[("FRESHER_SANDBOX", "/tmp/s"), ("FRESHER_SANDBOX_TRASH", "real")])).unwrap();

        assert_eq!(own, Some(SandboxMode::new("/tmp/s", SandboxTrash::Sandbox)));
        assert_eq!(real, Some(SandboxMode::new("/tmp/s", SandboxTrash::Real)));
    }

    #[test]
    fn a_development_start_without_the_variable_is_a_development_build() {
        let world = World::new();

        let start = crate::compose::start(world.real_data(), world.real_home(), env(&[])).unwrap();

        assert_eq!((start.data_folder, start.home), (world.real_data(), world.real_home()));
        assert_eq!(start.build, fresher_core::Build::Development);
    }

    /// The app's start, as its setup calls it: the sandbox's data folder and home replace the build's own, and the
    /// banner names the canonical sandbox folder, not the path as it was written.
    #[test]
    fn the_variable_moves_the_start_into_the_sandbox() {
        let world = World::new();
        let named = world.folder.path().join("home").join("..").to_str().unwrap().to_owned();

        let start =
            crate::compose::start(world.real_data(), world.real_home(), env(&[("FRESHER_SANDBOX", &named)])).unwrap();

        let sandbox = world.sandbox();
        assert_eq!((start.data_folder, start.home), (sandbox.data_folder(), sandbox.home()));
        let canonical = fs::canonicalize(world.folder.path()).unwrap();
        let trash = SandboxTrashAt::Sandbox(&sandbox.trash()).text();
        assert_eq!(
            start.build,
            fresher_core::Build::Sandbox { path: canonical, simulated: vec![Simulated::Running], trash }
        );
        assert!(!world.real_data().exists());
    }

    #[test]
    fn the_sandbox_home_is_the_scan_home() {
        let world = World::new();
        let engine = world.start();

        scan(&engine);

        let home = world.sandbox().home();
        assert_eq!(found(&engine), [home.join("work/Sample app cache")]);
    }

    #[test]
    fn the_settings_and_journal_live_in_the_sandbox() {
        let world = World::new();
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~/work".into()]);

        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
        scan(&engine);
        clean(&engine, &prepare(&engine));

        let data = world.sandbox().data_folder();
        assert!(data.join("settings.json").is_file());
        assert!(data.join("cleanups.jsonl").is_file());
        assert!(!world.real_data().exists());
    }

    #[test]
    fn a_cleanup_lands_in_the_sandbox_trash() {
        let world = World::new();
        let sandbox = world.sandbox();
        let engine = world.start();

        scan(&engine);
        clean(&engine, &prepare(&engine));

        let home = sandbox.home();
        assert!(!home.join("work/Sample app cache").exists());
        assert!(sandbox.trash().join("0-Sample app cache/blob.bin").is_file());
        assert!(home.join("bare/Sample app cache/blob.bin").is_file(), "the negative is untouched");
        assert!(home.join("work").join(SANDBOX_MARKER).is_file());
        assert!(fs::read_dir(world.real_home()).unwrap().next().is_none());
    }

    #[test]
    fn the_systems_trash_counts_the_macs_home_trash() {
        let world = World::new();

        let view = super::trash_after_a_move_to_the_macs_trash(&world, SandboxTrash::Real);

        assert_eq!(view.trashes.len(), 1, "{view:?}");
        assert_eq!(view.trashes[0].from_cleanups.allocated, fresher_core::ByteSize::new(4_096));
        assert_eq!(view.trashes[0].waiting, None, "the sandbox's scan never measures the Mac's Trash");
    }

    #[test]
    fn the_scan_and_the_journal_are_dated_by_one_clock() {
        let world = World::new();
        let engine = world.start();

        scan(&engine);
        clean(&engine, &prepare(&engine));

        // The fake clock of `fakes` stands still: the scan's state and the cleanup carry its moment, not the Mac's.
        let now = fresher_core::Timestamp::from_secs(fresher_testing::FIXTURE_NOW);
        assert_eq!(engine.disk_state().unwrap().unwrap().taken(), now);
        assert_eq!(engine.cleanups(0, 1).cleanups[0].started, now);
    }
}
