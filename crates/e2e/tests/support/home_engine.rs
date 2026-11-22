//! The engine of a flow home, assembled as sandbox mode composes it from the parts a home gives: the real scanner, its
//! reader seen through the facts lens; the real settings file, scan cache and journal in the sandbox's data folder;
//! the real `DiskCleaner` reaching only inside the sandbox folder, with the sandbox's places protection and a fake
//! Trash at `home/.Trash` (or at a second Trash a flow names in its `Extras`). Pulled in with `#[path]`, beside
//! `home.rs`, only by the homes that use it.

use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
use fresher_core::{Access, AutoEmpty, ByteSize, Capability, Module, Protection, SystemPorts, VolumeFacts, VolumeId};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry};
use fresher_sandbox::Sandbox;
use fresher_scanner::{Scanner, ScannerOptions, StdReader, known_folders_anchor};
use fresher_store::{DataFolder, FileCleanupJournal, FileScanCache, FileSettingsStore};
use fresher_testing::{
    FactsLens, FakeAccess, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeFinderSettings, FakeInstallRecords,
    FakeInstalledApps, FakeOpenFiles, FakeRunningApps, FakeSystemUi, FakeTrash, FakeVolumes, StatedFacts,
};

use super::home::user;

/// The one volume of a flow home, `/`.
pub(crate) const VOLUME: VolumeId = VolumeId::new(1);

/// What a home gives its engine besides the files.
pub(crate) struct Parts {
    /// The modules registered, in order.
    pub(crate) modules: Vec<Box<dyn Module>>,
    pub(crate) clock: Arc<FakeClock>,
    /// The change journal of the volume `VOLUME`.
    pub(crate) journal: FakeChangeJournal,
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    /// What the system says of its permissions; a flow keeps its handle to read what was tried.
    pub(crate) access: Arc<FakeAccess>,
    /// The install receipts and the macOS version.
    pub(crate) installs: Arc<FakeInstallRecords>,
    /// The scanner's worker threads.
    pub(crate) threads: NonZeroUsize,
    /// The places no walk opens (the scanner's standing avoid).
    pub(crate) avoid: Vec<PathBuf>,
}

/// What only some flows set besides the parts: the system's interface they read back, Finder's Trash setting, a
/// second Trash, the external commands' answers, and what the cleaner refuses besides the sandbox's places.
pub(crate) struct Extras {
    pub(crate) system_ui: Arc<FakeSystemUi>,
    pub(crate) finder: AutoEmpty,
    /// A second Trash outside the sandbox: the fake Trash moves there instead of `home/.Trash`, and the composition
    /// names it among the Trash folders. `None`: the home's Trash only.
    pub(crate) other_trash: Option<PathBuf>,
    /// The external commands' answers, shared by the modules and the cleaner; a flow keeps its handle to read the
    /// calls.
    pub(crate) commands: Arc<FakeCommandRunner>,
    /// Joined to the sandbox's places protection (as sandbox mode joins a Mac's lists for the sandbox's home, its
    /// standard folders whole); nothing more by default.
    pub(crate) protection: Protection,
}

impl Default for Extras {
    /// A fresh system interface, Finder's setting unknown, the home's Trash only, no command answered, the sandbox's
    /// places protection alone.
    fn default() -> Self {
        Self {
            system_ui: Arc::new(FakeSystemUi::default()),
            finder: AutoEmpty::Unknown,
            other_trash: None,
            commands: Arc::new(FakeCommandRunner::default()),
            protection: Protection::default(),
        }
    }
}

/// The access of a home that needs none: Full Disk Access `Unknown`, as sandbox mode stands in for it.
pub(crate) fn unknown_access() -> Arc<FakeAccess> {
    Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)]))
}

/// The installs of a home that needs none: no receipt, no version stated.
pub(crate) fn no_installs() -> Arc<FakeInstallRecords> {
    Arc::new(FakeInstallRecords::default())
}

/// The engine of `given` on `sandbox`, its reader stating `stated`, its cleaner meeting `open_files`.
pub(crate) fn engine(sandbox: &Sandbox, stated: &StatedFacts, open_files: &Arc<FakeOpenFiles>, given: Parts) -> Engine {
    engine_with(sandbox, stated, open_files, given, Extras::default())
}

/// The engine of `engine` with `extras` in place of the defaults.
pub(crate) fn engine_with(
    sandbox: &Sandbox,
    stated: &StatedFacts,
    open_files: &Arc<FakeOpenFiles>,
    given: Parts,
    extras: Extras,
) -> Engine {
    let data = sandbox.data_folder();
    let folder = DataFolder::open(&data).unwrap();
    let clock = given.clock;
    let facts = VolumeFacts { id: VOLUME, device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let volumes = Arc::new(FakeVolumes::default().with("/", facts));
    let folders = Arc::new(sandbox.known_folders());
    let options = ScannerOptions { threads: given.threads, standing_avoid: given.avoid, ..ScannerOptions::default() };
    let cache = Arc::new(FileScanCache::new(DataFolder::open(&data).unwrap()));
    let scanner = Scanner::new(
        Arc::new(FactsLens::new(StdReader::new(), stated.clone())),
        Arc::clone(&volumes) as _,
        Arc::new(given.journal),
        cache,
        known_folders_anchor(&*folders),
        options,
    )
    .with_clock(Arc::clone(&clock) as _);
    let running = Arc::new(given.running);
    let system = SystemPorts {
        known_folders: Arc::clone(&folders) as _,
        apps: Arc::new(given.installed),
        running: Arc::clone(&running) as _,
        installs: given.installs,
        files: Arc::new(fresher_testing::DiskFiles),
    };
    let commands = extras.commands;
    let ports = Ports {
        walk: Arc::new(scanner),
        settings: Arc::new(FileSettingsStore::new(DataFolder::open(&data).unwrap())),
        clock: Arc::clone(&clock) as _,
        commands: Arc::clone(&commands) as _,
        access: given.access,
        system_ui: extras.system_ui,
        system,
    };
    let cleaner_ports = CleanerPorts {
        trash: Arc::new(FakeTrash::new(extras.other_trash.clone().unwrap_or_else(|| sandbox.trash()))),
        open_files: Arc::clone(open_files) as _,
        running,
        commands,
        clock,
    };
    let user = user(sandbox.path());
    let reach = Reach::SandboxFolder(SandboxFolder::new(sandbox.path(), user, &Protection::default()).unwrap());
    let protection = sandbox.places_protection().unwrap().joined(&extras.protection).unwrap();
    let disk = Arc::new(DiskCleaner::new(reach, user, sandbox.home(), protection, cleaner_ports));
    let cleanup = CleanupPorts {
        cleaner: disk,
        journal: Arc::new(FileCleanupJournal::open(&folder)),
        volumes,
        finder: Arc::new(FakeFinderSettings::new(extras.finder)),
        trashes: extras.other_trash.into_iter().collect(),
    };
    let mut registry = Registry::new().unwrap();
    for module in given.modules {
        registry.register(module).unwrap();
    }
    Engine::with_cleanup(registry, ports, cleanup).unwrap()
}
