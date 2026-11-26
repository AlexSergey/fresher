//! Sandbox mode of a development build: started with `FRESHER_SANDBOX=<folder>`, the app lives entirely inside that
//! sandbox folder — its home and known folders, its data folder, its Trash — and its cleaner reaches nothing outside it
//! (docs/plans/007-safe-development/00-design.md, "Sandbox mode of the app"). Compiled in debug builds only.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
use fresher_core::{
    AccessProbe, Build, ByteSize, Clock, CommandRunner, HighLevel, InstallRecords, InstalledApps, KnownFolders,
    ModuleId, OpenFiles, Protection, ProtectionError, RunningApps, SandboxTrashAt, Simulated, Trash, Volumes,
};
use fresher_engine::{CleanupPorts, Engine};
use fresher_macos::{
    MacFolderReader, MacOpenFiles, MacTrash, MacVolumes, current_user, places_that_prompt, protection,
};
use fresher_sandbox::{Sandbox, StatedSpace, World};
use fresher_store::{DataFolder, FileCleanupJournal};
use fresher_system::{
    AppSearch, BundleReader, COMMAND_TIMEOUT, DeclaredSearch, MacFinderSettings, MacInstalledApps,
    ProcessCommandRunner, StartedClock, SystemClock,
};
use fresher_testing::{FactsLens, FakeTrash, StatedFacts, StatedInstalledApps};

use crate::adapters::UnknownAccess;
use crate::compose::{ComposeError, ModuleSet, Shared, assemble, real_protection, user_temp};

mod commands;

use commands::SandboxCommands;

/// The variable naming the sandbox folder, an absolute path.
const SANDBOX_VARIABLE: &str = "FRESHER_SANDBOX";
/// The variable choosing the Trash: absent for the sandbox's own, `real` for the system's.
const TRASH_VARIABLE: &str = "FRESHER_SANDBOX_TRASH";
/// The one value of [`TRASH_VARIABLE`].
const REAL_TRASH: &str = "real";
/// The user's Trash, in the home.
const HOME_TRASH: &str = ".Trash";
/// The sandbox's apps folder, at its top: the stand-in of `/Applications`.
const APPS_FOLDER: &str = "Applications";

/// Where a cleanup in sandbox mode moves items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTrash {
    /// The sandbox's own Trash, `home/.Trash`: nothing leaves the sandbox.
    Sandbox,
    /// The system's Trash, for the sandbox's own items only (`--real-trash`).
    Real,
}

/// How the scan of sandbox mode gives out its progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScanPace {
    /// As the walk goes: the app's, and every autotest's.
    AsWalked,
    /// A demo's: held to a minimum time, its progress numbers made up for the recording and rising evenly over it, the
    /// bytes toward `used`, the demo volume's used space, which the bar measures against (`fresher_sandbox::PacedWalk`).
    /// `used` is the world's, filled in when the engine is made ([`ScanPace::of_world`]); the findings stay real.
    Demo { used: Option<ByteSize> },
}

impl ScanPace {
    /// The pace with a demo's `used` taken from `world`'s stated volume; `None` when it states none.
    pub(crate) fn of_world(self, world: &World) -> Self {
        match self {
            Self::AsWalked => Self::AsWalked,
            Self::Demo { .. } => Self::Demo { used: world.volume().map(|volume| ByteSize::new(volume.used())) },
        }
    }
}

/// A development build's sandbox mode: the sandbox folder, the Trash its cleanups use, and the modules it registers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxMode {
    path: PathBuf,
    trash: SandboxTrash,
    /// The modules registered: every module of the build (the app's), the shipped ones (a demo's), or the one of an
    /// autotest's case file.
    modules: ModuleSet,
    /// Whether its engine offers High findings: withheld, as in the app, unless the autotest offers it.
    high: HighLevel,
    /// How its scan gives out its progress: as walked, unless a demo plays.
    pace: ScanPace,
}

/// Why sandbox mode does not start: the app then stops instead of running anywhere else.
#[derive(Debug, thiserror::Error)]
pub enum SandboxRefusal {
    #[error("{SANDBOX_VARIABLE} must be an absolute path, not {path:?}")]
    NotAbsolute { path: PathBuf },
    #[error("{TRASH_VARIABLE} must be `{REAL_TRASH}` or unset, not {value:?}")]
    UnknownTrash { value: OsString },
    #[error("{TRASH_VARIABLE} is set without {SANDBOX_VARIABLE}")]
    TrashWithoutSandbox,
    /// A link, a protected place of this Mac, or a folder without a valid marker of the user.
    #[error(transparent)]
    Folder(#[from] fresher_cleaner::SandboxError),
    /// A part of the sandbox's layout, such as `home/`, is missing.
    #[error(transparent)]
    Layout(#[from] fresher_sandbox::SandboxError),
    /// The sandbox's `world.toml` cannot be read or is not valid.
    #[error("the sandbox's world: {0}")]
    World(fresher_sandbox::SandboxError),
}

/// Why the sandbox's engine could not be made.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct SandboxEngineError(#[from] ComposeError);

impl SandboxEngineError {
    /// Why sandbox mode refused the folder or the environment, when that is the error.
    pub const fn refusal(&self) -> Option<&SandboxRefusal> {
        match &self.0 {
            ComposeError::Sandbox(refusal) => Some(refusal),
            _ => None,
        }
    }
}

impl SandboxMode {
    /// The sandbox at `path`, its cleanups going to `trash`.
    pub fn new(path: impl Into<PathBuf>, trash: SandboxTrash) -> Self {
        Self {
            path: path.into(),
            trash,
            modules: ModuleSet::Every,
            high: HighLevel::Withheld,
            pace: ScanPace::AsWalked,
        }
    }

    /// The same mode with only `module` registered, as the autotest of a case file naming it runs; no module when the
    /// app has none with that id.
    #[must_use]
    pub fn only(self, module: ModuleId) -> Self {
        Self { modules: ModuleSet::Only(module), ..self }
    }

    /// The same mode with only the modules a release ships registered: a demo shows the app as it ships, never the
    /// made-up findings of the dev-only modules.
    #[must_use]
    pub fn shipped_modules(self) -> Self {
        Self { modules: ModuleSet::Shipped, ..self }
    }

    /// The same mode with a demo's scan: a fast walk is held to `fresher_sandbox::DEMO_SCAN_TIME`, its progress rising
    /// evenly. For demo mode only; the demo worlds' scan test and every autotest walk as the app does.
    #[must_use]
    pub fn paced_scan(self) -> Self {
        Self { pace: ScanPace::Demo { used: None }, ..self }
    }

    /// The same mode with High findings offered. For the sandbox autotest only, which runs a case file's High
    /// positives; the app's sandbox mode withholds High, as the app does.
    #[must_use]
    pub fn offering_high(self) -> Self {
        Self { high: HighLevel::Offered, ..self }
    }

    /// The mode the environment asks for, read through `var`: `FRESHER_SANDBOX` (an absolute path) and
    /// `FRESHER_SANDBOX_TRASH` (`real` or absent); `None` when neither is set. Anything else is refused.
    pub fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Option<Self>, SandboxRefusal> {
        let trash = match var(TRASH_VARIABLE) {
            None => SandboxTrash::Sandbox,
            Some(value) if value == REAL_TRASH => SandboxTrash::Real,
            Some(value) => return Err(SandboxRefusal::UnknownTrash { value }),
        };
        let Some(path) = var(SANDBOX_VARIABLE).map(PathBuf::from) else {
            return match trash {
                SandboxTrash::Sandbox => Ok(None),
                SandboxTrash::Real => Err(SandboxRefusal::TrashWithoutSandbox),
            };
        };
        if !path.is_absolute() {
            return Err(SandboxRefusal::NotAbsolute { path });
        }
        Ok(Some(Self { path, trash, modules: ModuleSet::Every, high: HighLevel::Withheld, pace: ScanPace::AsWalked }))
    }

    /// The sandbox folder as it was named.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Where its cleanups move items.
    pub const fn trash(&self) -> SandboxTrash {
        self.trash
    }
}

/// What sandbox mode keeps real in the app, and the autotests replace by fakes: all of it is only read. The running
/// apps are never the Mac's, the clock starts at the sandbox's own moment, the access is the one its world states, the
/// receipts and the macOS version are its world's, the facts the sandbox's entries state are read instead of the
/// disk's, and the commands are the world's (no program of the Mac runs but `/usr/bin/open`): the app takes all six
/// from the sandbox's world.toml.
#[derive(Clone)]
pub struct SandboxPorts {
    pub clock: Arc<dyn Clock>,
    pub volumes: Arc<dyn Volumes>,
    pub open_files: Arc<dyn OpenFiles>,
    pub running: Arc<dyn RunningApps>,
    pub commands: Arc<dyn CommandRunner>,
    pub access: Arc<dyn AccessProbe>,
    /// The installer receipts and the macOS version its world states (`World::install_records`): never the Mac's.
    pub installs: Arc<dyn InstallRecords>,
    /// What the walk's reader reports instead of the disk's, for the stated entries only (`World::stated`).
    pub stated: StatedFacts,
}

impl std::fmt::Debug for SandboxPorts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("SandboxPorts").finish_non_exhaustive()
    }
}

impl SandboxPorts {
    /// The app's real ports, as a normal development build has them, but the running apps, the clock, the access, the
    /// installs and the stated facts: the sandbox's declared running apps (none without a `world.toml`), so an app open
    /// on the Mac never changes what a sandbox shows, a clock that starts at the moment the sandbox was generated (the
    /// system's without one), so its ages are the case files' however long ago it was generated, the access its world
    /// states (a stand-in that knows nothing without one), the receipts and the macOS version its world states (none,
    /// and a version that fails, without them: never the Mac's), the facts its entries state, and the commands its
    /// world answers: a program it does not state answers not installed and is never run, with or without stated
    /// tools; only `/usr/bin/open`, the system's interface (Show in Finder, System Settings), runs for real. A demo
    /// world that states its volume has that volume's space answered for every path (`StatedSpace`), so the app never
    /// shows the developer's disk; which volume a path is on stays the Mac's.
    pub(crate) fn real(sandbox: &Sandbox) -> Result<Self, SandboxRefusal> {
        let world = World::read(sandbox).map_err(SandboxRefusal::World)?;
        let clock: Arc<dyn Clock> = match world.now() {
            Some(start) => Arc::new(StartedClock::at(start)),
            None => Arc::new(SystemClock),
        };
        // The Mac's access is never asked: the sandbox's world is not the Mac's (docs/hacks/).
        let access = access_of(&world, &sandbox.home(), Arc::new(UnknownAccess));
        let volumes: Arc<dyn Volumes> = match world.volume() {
            Some(volume) => Arc::new(StatedSpace::new(Arc::new(MacVolumes::new()), volume)),
            None => Arc::new(MacVolumes::new()),
        };
        Ok(Self {
            clock,
            volumes,
            open_files: Arc::new(MacOpenFiles::new()),
            running: Arc::new(world.running_apps()),
            commands: Arc::new(SandboxCommands {
                world: Arc::new(world.commands(sandbox.path())),
                open: Arc::new(ProcessCommandRunner::new(COMMAND_TIMEOUT)),
            }),
            access,
            installs: Arc::new(world.install_records()),
            stated: world.stated(sandbox.path()),
        })
    }
}

/// The access sandbox mode answers in the sandbox whose home is `home`: the one `world` states, else `otherwise`.
pub fn access_of(world: &World, home: &Path, otherwise: Arc<dyn AccessProbe>) -> Arc<dyn AccessProbe> {
    match world.access(home) {
        Some(stated) => Arc::new(stated),
        None => otherwise,
    }
}

/// The engine of sandbox mode over `ports`: the sandbox's home and known folders, its data folder
/// `<sandbox>/.fresher-data`, its Trash (`home/.Trash`, or the system's for [`SandboxTrash::Real`]), a cleaner that
/// reaches only inside the sandbox folder, the system's reader seen through the ports' stated facts, and the
/// protection lists — the real Mac's for `real_home` (with the development build's own `real_data_folder`), the
/// sandbox home's and its temporary places'. Refused when the folder is a link, is
/// protected on the real Mac, holds no valid marker of the user, or misses a part of its layout. It leaves the
/// process's IO policy alone: the app's start sets it.
pub fn sandbox_engine(
    sandbox: &SandboxMode,
    real_home: &Path,
    real_data_folder: &Path,
    ports: SandboxPorts,
) -> Result<Engine, SandboxEngineError> {
    Ok(compose(sandbox, real_home, real_data_folder, |_| Ok(ports))?.0)
}

/// The sandbox's engine and its checked layout, its ports made by `ports` once the sandbox is checked.
pub(crate) fn compose(
    mode: &SandboxMode,
    real_home: &Path,
    real_data_folder: &Path,
    ports: impl FnOnce(&Sandbox) -> Result<SandboxPorts, SandboxRefusal>,
) -> Result<(Engine, Sandbox), ComposeError> {
    let user = current_user();
    let real_temp = user_temp();
    let real = real_protection(real_home, real_data_folder, real_temp.as_deref())?;
    let folder = SandboxFolder::new(mode.path(), user, &real).map_err(SandboxRefusal::from)?;
    let sandbox = Sandbox::open(folder.path(), user).map_err(SandboxRefusal::from)?;
    let ports = ports(&sandbox)?;
    let world = World::read(&sandbox).map_err(SandboxRefusal::World)?;
    let home = sandbox.home();
    let known_folders: Arc<dyn KnownFolders> = Arc::new(sandbox.known_folders());
    let data = DataFolder::open(sandbox.data_folder())?;
    // Each list keeps its own carve-outs: the real Mac stays safe whatever a bug does, and the sandbox behaves like a
    // Mac (its `Documents` whole, its `.Trash` never touched, its own `T` carved out of its users' folders). The
    // Mac's lists carve the real user's `T` out of every user's temporary folders, so a sandbox generated there stays
    // reachable; the sandbox's `T` is no user's of the Mac, so it is carved by a list of its own. The sandbox's apps
    // folder is a list of its own too, opened like `/Applications`: a whole bundle may go, nothing inside one.
    let joined = real
        .joined(&protection(&home, data.path(), None, real_temp.as_deref())?)?
        .joined(&sandbox.places_protection()?)?
        .joined(&apps_protection(&sandbox)?)?;
    // With the system's Trash, the sandbox's items land in the Mac's own home Trash: the Trash view counts it too.
    let (trash, trashes): (Arc<dyn Trash>, Vec<PathBuf>) = match mode.trash() {
        SandboxTrash::Sandbox => (Arc::new(FakeTrash::new(sandbox.trash())), Vec::new()),
        SandboxTrash::Real => (Arc::new(MacTrash::new()), vec![real_home.join(HOME_TRASH)]),
    };
    let cleaner = CleanerPorts {
        trash,
        open_files: ports.open_files,
        running: Arc::clone(&ports.running),
        commands: Arc::clone(&ports.commands),
        clock: Arc::clone(&ports.clock),
    };
    let cleaner = DiskCleaner::new(Reach::SandboxFolder(folder), user, home.clone(), joined, cleaner);
    let cleanup = CleanupPorts {
        cleaner: Arc::new(cleaner),
        journal: Arc::new(FileCleanupJournal::open(&data)),
        volumes: Arc::clone(&ports.volumes),
        finder: Arc::new(MacFinderSettings::new(Arc::clone(&ports.commands))),
        trashes,
    };
    // Sandbox mode keeps a standing avoid list, the guarded places of both homes: its access is a stand-in, since the
    // sandbox's world is not the Mac's, and a scan root outside the sandbox, such as the real home, must raise no
    // system prompt (docs/hacks/, "Sandbox mode avoids the places that prompt"). The sandbox home's places of the
    // folders and of other apps' data its world states are left to the engine's coverage, as on a Mac; the real
    // home's stay avoided whatever the world says.
    let stated = world.stated_places(&home);
    let mut avoid: Vec<PathBuf> =
        places_that_prompt(&home).into_iter().filter(|place| !stated.contains(place)).collect();
    avoid.extend(places_that_prompt(real_home));
    let apps = installed_apps(&world, sandbox.path(), Arc::clone(&known_folders), &ports.stated);
    let engine = assemble(Shared {
        folder: data,
        reader: Arc::new(FactsLens::new(MacFolderReader::new(), ports.stated)),
        avoid,
        known_folders,
        volumes: ports.volumes,
        clock: ports.clock,
        running: ports.running,
        commands: ports.commands,
        access: ports.access,
        cleanup,
        modules: mode.modules,
        apps,
        installs: ports.installs,
        high: mode.high,
        pace: mode.pace.of_world(&world),
    })?;
    Ok((engine, sandbox))
}

/// The sandbox's apps folder, the stand-in of `/Applications`, as the Mac's list has it: a never place that lets
/// through exactly a whole app bundle at its top or one folder below.
fn apps_protection(sandbox: &Sandbox) -> Result<Protection, ProtectionError> {
    let apps = sandbox.path().join(APPS_FOLDER);
    Protection::new(vec![apps.clone()], Vec::new(), Vec::new())?.with_app_folders(&[apps])
}

/// What the app's search answers in the sandbox at `path`: its listed apps, the apps its world declares elsewhere,
/// and whether its Spotlight is on.
fn app_search(world: &World, path: &Path) -> DeclaredSearch {
    DeclaredSearch { index_on: world.spotlight_on(), elsewhere: world.elsewhere_apps(path) }
}

/// The apps of the sandbox at `path`: the ones in its apps folders and its world's declared ones (Spotlight would
/// answer the real Mac's), each with the change time the world states for its bundle. Its bundles are measured on its
/// own disk; the Mac's Spotlight and Caskrooms are never asked.
fn installed_apps(
    world: &World,
    path: &Path,
    known_folders: Arc<dyn KnownFolders>,
    stated: &StatedFacts,
) -> Arc<dyn InstalledApps> {
    let listed = MacInstalledApps::new(known_folders, AppSearch::Declared(app_search(world, path)))
        .with_bundle_reader(BundleReader::stated(), current_user());
    Arc::new(StatedInstalledApps::new(Arc::new(listed), stated.clone()))
}

/// The build the banner shows for `sandbox`: its folder, what it simulates and where its cleanups go (`trash`). It
/// simulates its open apps always, Spotlight when its world declares any of it (an app found elsewhere, or Spotlight
/// off), the access when its world states any, the installs when it states the macOS version or a receipt, and the
/// tools when it states what a program answers.
pub(crate) fn build(sandbox: &Sandbox, trash: SandboxTrash) -> Result<Build, SandboxRefusal> {
    let world = World::read(sandbox).map_err(SandboxRefusal::World)?;
    let mut simulated = vec![Simulated::Running];
    if world.declares_spotlight() {
        simulated.push(Simulated::Spotlight);
    }
    if world.declares_access() {
        simulated.push(Simulated::Access);
    }
    if world.declares_installs() {
        simulated.push(Simulated::Installs);
    }
    if world.declares_tools() {
        simulated.push(Simulated::Tools);
    }
    // The same folder the sandbox's Trash port is given in `compose`.
    let own_trash = sandbox.trash();
    let trash = match trash {
        SandboxTrash::Sandbox => SandboxTrashAt::Sandbox(&own_trash),
        SandboxTrash::Real => SandboxTrashAt::System,
    };
    Ok(Build::Sandbox { path: sandbox.path().to_path_buf(), simulated, trash: trash.text() })
}

#[cfg(test)]
mod tests;
