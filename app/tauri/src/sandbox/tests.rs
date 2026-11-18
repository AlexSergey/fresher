//! Sandbox mode over a generated sandbox, with fakes for the ports that stay real in the app. The real home and the
//! development build's data folder are temporary folders too: no test touches the developer's own.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use fresher_core::{
    Access, ByteSize, Capability, CleanupPlanView, CleanupSelection, Confirmation, Event, FindingOrder, FindingsQuery,
    Protection, SANDBOX_MARKER, Subject, VolumeFacts, VolumeId,
};
use fresher_engine::{Engine, ScanMode};
use fresher_sandbox::{CaseFile, Sandbox, generate};
use fresher_testing::{
    FIXTURE_NOW, FakeAccess, FakeClock, FakeCommandRunner, FakeOpenFiles, FakeRunningApps, FakeVolumes, StatedFacts,
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
path = "home/bare/fresher-trial-cache/blob.bin"
size = 4096

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
size = 4096
fill = "data"
[case.expect]
item = "home/work/fresher-trial-cache"
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

    fn start(&self) -> Engine {
        self.start_at(self.folder.path()).unwrap()
    }
}

fn user(folder: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt as _;
    fs::metadata(folder).unwrap().uid()
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
        "allocated": summary.sizes.allocated, "highest": summary.highest, "method": "trash" }))
    .unwrap();
    let events = engine.subscribe();
    engine.confirm_cleanup(&echo).unwrap();
    until(&events, |event| matches!(event, Event::CleanupFinished { .. }));
}

/// Waits for the first event that `is_last`; fails after 30 s instead of hanging.
fn until(events: &Receiver<Event>, is_last: impl Fn(&Event) -> bool) {
    while !is_last(&events.recv_timeout(Duration::from_secs(30)).unwrap()) {}
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

    /// A place of the real home that may raise a system prompt is never opened, even as a scan root.
    #[test]
    fn the_real_homes_places_that_prompt_are_not_walked() {
        let world = World::new();
        let documents = fs::canonicalize(world.real_home()).unwrap().join("Documents");
        fs::create_dir_all(documents.join("fresher-trial-cache")).unwrap();
        fs::write(documents.join("fresher-trial-cache/blob.bin"), [0; 64]).unwrap();
        fs::write(documents.join(SANDBOX_MARKER), format!("{}\n", fresher_core::SANDBOX_SIGNATURE)).unwrap();
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~".into(), documents.clone()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);

        assert!(found(&engine).iter().all(|path| !path.starts_with(&documents)), "{:?}", found(&engine));
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
        assert_eq!(found, [(home.join("work/fresher-trial-cache"), fresher_core::Removability::Removable)]);
        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();
        assert!(ports.stated.is_empty());
    }

    /// A scan root outside the sandbox is scanned, but nothing in it can be removed.
    #[test]
    fn an_item_outside_the_sandbox_is_left_out() {
        let world = World::new();
        let outside = FixtureTree::from_toml_str("[[file]]\npath = \"fresher-trial-cache/blob.bin\"\nsize = 4096\n")
            .unwrap()
            .with_sandbox()
            .unwrap();
        let root = fs::canonicalize(outside.root()).unwrap();
        let engine = world.start();
        let roots = SettingValue::Folders(vec!["~".into(), root.clone()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();

        scan(&engine);
        let view = prepare(&engine);

        assert!(found(&engine).contains(&root.join("fresher-trial-cache")));
        assert_eq!(view.left_out.len(), 1, "{view:?}");
        assert_eq!(view.left_out[0].reason(), &SkipReason::NotInSandbox);
        assert!(root.join("fresher-trial-cache/blob.bin").exists());
    }
}

mod positive {
    use std::fs;

    use fresher_core::{SettingUpdate, SettingValue};

    use super::{SANDBOX_MARKER, World, clean, env, found, prepare, scan};
    use crate::sandbox::{SandboxMode, SandboxTrash};

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
    fn the_real_ports_clock_starts_at_the_moment_the_sandbox_was_generated() {
        let world = World::new();
        fs::write(world.sandbox().world(), format!("now = {}\n", fresher_testing::FIXTURE_NOW)).unwrap();
        let start = std::time::UNIX_EPOCH + std::time::Duration::from_secs(fresher_testing::FIXTURE_NOW);

        let ports = crate::sandbox::SandboxPorts::real(&world.sandbox()).unwrap();

        let now = ports.clock.now();
        assert!(now >= start && now < start + std::time::Duration::from_secs(60), "{now:?}");
    }

    #[test]
    fn sandbox_modes_reader_reports_a_stated_fact() {
        let world = World::new();
        let sandbox = world.sandbox();
        let entry =
            "\n[[entry]]\npath = \"home/work/fresher-trial-cache\"\nowner = \"system\"\nflags = [\"system-lock\"]\n";
        let text = fs::read_to_string(sandbox.world()).unwrap() + entry;
        fs::write(sandbox.world(), text).unwrap();
        let engine = super::start_stated(&world);

        scan(&engine);

        let ports = crate::sandbox::SandboxPorts::real(&sandbox).unwrap();
        let cache = sandbox.home().join("work/fresher-trial-cache");
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
        assert_eq!(start.build, fresher_core::Build::Sandbox { path: canonical, simulated_running: true });
        assert!(!world.real_data().exists());
    }

    #[test]
    fn the_sandbox_home_is_the_scan_home() {
        let world = World::new();
        let engine = world.start();

        scan(&engine);

        let home = world.sandbox().home();
        assert_eq!(found(&engine), [home.join("work/fresher-trial-cache")]);
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
        assert!(!home.join("work/fresher-trial-cache").exists());
        assert!(sandbox.trash().join("0-fresher-trial-cache/blob.bin").is_file());
        assert!(home.join("bare/fresher-trial-cache/blob.bin").is_file(), "the negative is untouched");
        assert!(home.join("work").join(SANDBOX_MARKER).is_file());
        assert!(fs::read_dir(world.real_home()).unwrap().next().is_none());
    }
}
