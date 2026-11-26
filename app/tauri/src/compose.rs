//! The composition root: the real adapters, the modules, and the engine they make.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[cfg(debug_assertions)]
use fresher_cleaner::DryRunToken;
#[cfg(not(any(debug_assertions, test)))]
use fresher_cleaner::RealDiskToken;
use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
use fresher_core::{
    AccessProbe, Build, Clock, CommandRunner, DiskWalk, FinderSettings, FolderReader, HighLevel, InstallRecords,
    InstalledApps, KnownFolders, Module, ModuleGroup, ModuleId, Protection, ProtectionError, ReaderFiles, RunningApps,
    SchemaError, SystemPorts, Volumes,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, RegistryError};
#[cfg(not(debug_assertions))]
use fresher_macos::MacTrash;
use fresher_macos::{
    MacChangeJournal, MacFolderReader, MacOpenFiles, MacVolumes, current_user, io_policy, is_user_temp, protection,
    user_temp_dir,
};
use fresher_scanner::{Scanner, ScannerOptions, known_folders_anchor};
use fresher_store::{CachePolicy, DataFolder, DataFolderError, FileCleanupJournal, FileScanCache, FileSettingsStore};

use fresher_system::{
    AppSearch, BundleReader, COMMAND_TIMEOUT, INDEX_CHECK_TIMEOUT, MacAccess, MacFinderSettings, MacInstallRecords,
    MacInstalledApps, MacKnownFolders, MacMounts, MacRunningApps, MacSystemUi, ProcessCommandRunner, SpotlightSearch,
    SystemClock, macos_version,
};

/// Why the app could not start its engine.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ComposeError {
    #[error("the data folder cannot be used: {0}")]
    DataFolder(#[from] DataFolderError),
    /// Scanning without the policy could download cloud files: the app does not start.
    #[error("the process's IO policy cannot be set: {0}")]
    IoPolicy(String),
    #[error("a module was refused: {0}")]
    Registry(#[from] RegistryError),
    #[error("the settings schema is refused: {0}")]
    Schema(#[from] SchemaError),
    /// Without its own bundle in the protected places, the app does not start.
    #[error("the app's own executable cannot be found: {0}")]
    Executable(#[source] io::Error),
    #[error("the protected places cannot be listed: {0}")]
    Protection(#[from] ProtectionError),
    #[error("the jetbrains module cannot be built: {0}")]
    JetBrains(#[from] fresher_module_jetbrains::JetBrainsError),
    #[error("the temp module cannot be built: {0}")]
    Temp(#[from] fresher_module_temp::TempError),
    #[error("the dev-caches module cannot be built: {0}")]
    DevCaches(#[from] fresher_module_dev_caches::DevCachesError),
    #[error("the app-leftovers module cannot be built: {0}")]
    AppLeftovers(#[from] fresher_module_app_leftovers::AppLeftoversError),
    #[error("the app-caches module cannot be built: {0}")]
    AppCaches(#[from] fresher_module_app_caches::AppCachesError),
    #[error("the logs module cannot be built: {0}")]
    Logs(#[from] fresher_module_logs::LogsError),
    #[error("the downloads module cannot be built: {0}")]
    Downloads(#[from] fresher_module_downloads::DownloadsError),
    #[error("the stale-projects module cannot be built: {0}")]
    StaleProjects(#[from] fresher_module_stale_projects::StaleProjectsError),
    #[error("the local-ai-models module cannot be built: {0}")]
    LocalAiModels(#[from] fresher_module_local_ai_models::LocalAiModelsError),
    #[error("the unused-toolchains module cannot be built: {0}")]
    UnusedToolchains(#[from] fresher_module_unused_toolchains::UnusedToolchainsError),
    #[error("the xcode module cannot be built: {0}")]
    Xcode(#[from] fresher_module_xcode::XcodeError),
    #[error("the docker module cannot be built: {0}")]
    Docker(#[from] fresher_module_docker::DockerError),
    /// A development build started on a sandbox that is not one: it does not start rather than run elsewhere.
    #[cfg(debug_assertions)]
    #[error("sandbox mode is refused: {0}")]
    Sandbox(#[from] crate::sandbox::SandboxRefusal),
    /// A development build asked for a demo it cannot play: it does not start rather than play elsewhere.
    #[cfg(debug_assertions)]
    #[error("demo mode is refused: {0}")]
    DemoMode(#[from] crate::demo::DemoRefusal),
    #[cfg(debug_assertions)]
    #[error("the demo module cannot be built: {0}")]
    Demo(#[from] fresher_module_demo::DemoError),
    #[cfg(debug_assertions)]
    #[error("the trial module cannot be built: {0}")]
    Trial(#[from] fresher_module_trial::TrialError),
    /// A development build asked to record its run where it cannot: it does not start rather than run unrecorded.
    #[cfg(debug_assertions)]
    #[error("the development run record is refused: {0}")]
    DevRuns(#[from] crate::dev_runs::DevRunsRefusal),
}

/// The longest a rescan waits on the change journal before it scans in full: half the developer's warm full scan of
/// the home (22 s in plan 004/01), so a long replay never costs more than the scan it would save.
const JOURNAL_TIME_LIMIT: Duration = Duration::from_secs(11);

/// Sets one part of the process's IO policy.
type PolicySetter = fn() -> Result<(), String>;

/// The process's IO policy, set before the engine starts.
pub(crate) struct IoPolicy {
    pub(crate) never_download_cloud_files: PolicySetter,
    pub(crate) leave_access_times: PolicySetter,
}

/// The macOS IO policy of `fresher-macos`.
pub(crate) const MACOS_POLICY: IoPolicy = IoPolicy {
    never_download_cloud_files: || io_policy::never_download_cloud_files().map_err(|error| error.to_string()),
    leave_access_times: || io_policy::leave_access_times().map_err(|error| error.to_string()),
};

/// Where the app runs: its engine, the data folder it keeps its own data in, and the home of its scan roots.
#[derive(Debug)]
pub(crate) struct Start {
    pub(crate) engine: Engine,
    pub(crate) data_folder: PathBuf,
    pub(crate) home: PathBuf,
    pub(crate) build: Build,
    /// The development run record, when the launcher asked for one (`FRESHER_DEV_RUNS`).
    #[cfg(debug_assertions)]
    pub(crate) run: Option<crate::dev_runs::run::Run>,
    /// The scenario to play, when the launcher asked for one (`FRESHER_DEMO`, in sandbox mode only).
    #[cfg(debug_assertions)]
    pub(crate) demo: Option<crate::demo::DemoMode>,
}

/// The app's start: in a development build whose environment (read through `var`) names a sandbox in
/// `FRESHER_SANDBOX`, the sandbox's engine, data folder and home (`data_folder` is then only protected); otherwise
/// the engine of [`engine`] on `data_folder` and `home` as the disk spells it ([`disk_home`]). In a development build
/// whose environment names a folder in `FRESHER_DEV_RUNS`, the run's record begins there. A release build reads no
/// variable.
pub(crate) fn start(
    data_folder: PathBuf,
    home: PathBuf,
    #[cfg_attr(not(debug_assertions), expect(unused_variables, reason = "a release build has no sandbox mode"))]
    var: impl Fn(&str) -> Option<OsString>,
) -> Result<Start, ComposeError> {
    let home = disk_home(home);
    // A bad record folder stops the start before any engine is made.
    #[cfg(debug_assertions)]
    let launch = crate::dev_runs::Launch::from_env(&var)?;
    #[cfg(debug_assertions)]
    if let Some(mode) = crate::sandbox::SandboxMode::from_env(&var)? {
        apply(&MACOS_POLICY)?;
        // A demo shows the app as it ships: the dev-only modules' made-up findings stay out of the recording.
        // Its small world is walked in a second: the demo's scan is paced, so the recording shows it run.
        let mode = if crate::demo::DemoMode::asked(&var) { mode.shipped_modules().paced_scan() } else { mode };
        // The running apps come from the sandbox's world, never the Mac's: an app open on the Mac changes nothing.
        let (engine, sandbox) =
            crate::sandbox::compose(&mode, &home, &data_folder, crate::sandbox::SandboxPorts::real)?;
        let build = crate::sandbox::build(&sandbox, mode.trash())?;
        // The real home is still `home` here: the demo keeps it, and the checked sandbox folder, off the page.
        let demo = crate::demo::DemoMode::from_env(&var, Some(sandbox.path()), mode.trash(), &home)?;
        let home = sandbox.home();
        let run = begin_run(launch.as_ref(), &build, &home, &engine)?;
        return Ok(Start { engine, data_folder: sandbox.data_folder(), home, build, run, demo });
    }
    // Outside sandbox mode a demo is refused before any engine is made.
    #[cfg(debug_assertions)]
    let demo = crate::demo::DemoMode::from_env(&var, None, crate::sandbox::SandboxTrash::Sandbox, &home)?;
    let engine = engine(&data_folder, &home)?;
    let build = own_build();
    #[cfg(debug_assertions)]
    let run = begin_run(launch.as_ref(), &build, &home, &engine)?;
    Ok(Start {
        engine,
        data_folder,
        home,
        build,
        #[cfg(debug_assertions)]
        run,
        #[cfg(debug_assertions)]
        demo,
    })
}

/// The run's record of `build` on `engine`, when `launch` asks for one; the macOS version as `<major>.<minor>`.
#[cfg(debug_assertions)]
fn begin_run(
    launch: Option<&crate::dev_runs::Launch>,
    build: &Build,
    home: &Path,
    engine: &Engine,
) -> Result<Option<crate::dev_runs::run::Run>, ComposeError> {
    let Some(launch) = launch else {
        return Ok(None);
    };
    let macos = macos_version(Path::new("/")).map(|version| format!("{}.{}", version.major, version.minor));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    Ok(crate::dev_runs::run::Run::begin(launch, build, home, macos, clock, engine)?)
}

/// The home folder as the disk spells it: a home reached through a link (`/Users/<name>` linked to another volume) is
/// resolved once, so the known folders, the scan roots, the protection and the Trash all use the spelling the system
/// gives back for a held item (`F_GETPATH`), and put back finds every entry in the home's Trash. A home that cannot
/// be resolved is kept as given.
pub(crate) fn disk_home(home: PathBuf) -> PathBuf {
    std::fs::canonicalize(&home).unwrap_or(home)
}

/// The kind of this build: a release build is never a development one.
const fn own_build() -> Build {
    if cfg!(debug_assertions) { Build::Development } else { Build::Release }
}

/// The engine of the app, with its settings in `data_folder` (Tauri's `app_data_dir`) and `home` the user's home
/// folder (Tauri's `home_dir`): where the known folders are (`MacKnownFolders`), and `~` in the scan roots.
pub(crate) fn engine(data_folder: impl Into<PathBuf>, home: &Path) -> Result<Engine, ComposeError> {
    engine_with(data_folder, home, &MACOS_POLICY)
}

fn engine_with(data_folder: impl Into<PathBuf>, home: &Path, policy: &IoPolicy) -> Result<Engine, ComposeError> {
    assemble(normal(data_folder, home, policy)?)
}

/// What the normal engine is assembled from: the real ports of this Mac, after the process's IO policy is set.
fn normal(data_folder: impl Into<PathBuf>, home: &Path, policy: &IoPolicy) -> Result<Shared, ComposeError> {
    apply(policy)?;
    let folder = DataFolder::open(data_folder)?;
    // Before any write: the folder's own scratch files left by an interrupted write go.
    crate::own_data::clear_stale_temporaries(&folder);
    let volumes: Arc<dyn Volumes> = Arc::new(MacVolumes::new());
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let running: Arc<dyn RunningApps> = Arc::new(MacRunningApps::new());
    let commands: Arc<dyn CommandRunner> = Arc::new(ProcessCommandRunner::new(COMMAND_TIMEOUT));
    // The system's Trash goes to the cleaner and to nothing else; a development build's cleaner gets a Trash that
    // refuses.
    #[cfg(debug_assertions)]
    let trash = Arc::new(crate::dry_run::DryRunTrash);
    #[cfg(not(debug_assertions))]
    let trash = Arc::new(MacTrash::new());
    let cleaner = CleanerPorts {
        trash,
        open_files: Arc::new(MacOpenFiles::new()),
        running: Arc::clone(&running),
        commands: Arc::clone(&commands),
        clock: Arc::clone(&clock),
    };
    let user_temp = user_temp();
    let finder = Arc::new(MacFinderSettings::new(Arc::clone(&commands)));
    let cleanup = cleanup(home, user_temp.as_deref(), &folder, cleaner, Arc::clone(&volumes), finder, reach())?;
    let known_folders: Arc<dyn KnownFolders> = match user_temp {
        Some(temp) => Arc::new(MacKnownFolders::new(home).with_user_temp(temp)),
        None => Arc::new(MacKnownFolders::new(home)),
    };
    Ok(Shared {
        folder,
        reader: Arc::new(MacFolderReader::new()),
        // No standing list: the engine's request carries the guarded places the capability state keeps closed.
        avoid: Vec::new(),
        known_folders: Arc::clone(&known_folders),
        volumes,
        clock,
        running,
        apps: Arc::new(
            MacInstalledApps::new(
                known_folders,
                AppSearch::Spotlight(SpotlightSearch::new(
                    Arc::clone(&commands),
                    Arc::new(ProcessCommandRunner::new(INDEX_CHECK_TIMEOUT)),
                    Arc::new(MacMounts),
                )),
            )
            .with_bundle_reader(BundleReader::mac(Arc::clone(&commands)), current_user()),
        ),
        installs: Arc::new(MacInstallRecords::new("/")),
        commands,
        access: Arc::new(MacAccess::new(home.to_path_buf(), macos_version(Path::new("/")))),
        cleanup,
        modules: ModuleSet::Every,
        // The first release withholds High (plan 042); development builds too, so the walk sees what ships.
        high: HighLevel::Withheld,
        #[cfg(debug_assertions)]
        pace: crate::sandbox::ScanPace::AsWalked,
    })
}

/// Sets the process's IO policy before an engine starts.
pub(crate) fn apply(policy: &IoPolicy) -> Result<(), ComposeError> {
    (policy.never_download_cloud_files)().map_err(ComposeError::IoPolicy)?;
    if let Err(error) = (policy.leave_access_times)() {
        tracing::warn!(%error, "reading files may update their access times");
    }
    Ok(())
}

/// What the shared assembly takes, chosen apart by the normal engine and the sandbox's: the data `folder`, the reader
/// of the disk, the places the walk never opens, the known folders (whose home is `~` of the scan roots), the ports
/// read through, the cleanup, and the modules registered.
pub(crate) struct Shared {
    pub(crate) folder: DataFolder,
    /// The system's bulk reader; in sandbox mode seen through the sandbox's stated facts.
    pub(crate) reader: Arc<dyn FolderReader>,
    /// The places no walk opens whatever the capability state: empty in the normal app, sandbox mode's list there.
    pub(crate) avoid: Vec<PathBuf>,
    pub(crate) known_folders: Arc<dyn KnownFolders>,
    pub(crate) volumes: Arc<dyn Volumes>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) running: Arc<dyn RunningApps>,
    pub(crate) commands: Arc<dyn CommandRunner>,
    pub(crate) access: Arc<dyn AccessProbe>,
    pub(crate) cleanup: CleanupPorts,
    pub(crate) modules: ModuleSet,
    /// The installed apps: the apps folders and Spotlight on a Mac; in a sandbox its listed and declared apps, with the
    /// change times its world states.
    pub(crate) apps: Arc<dyn InstalledApps>,
    /// The installer receipts and the macOS version: the Mac's, or the sandbox's world's.
    pub(crate) installs: Arc<dyn InstallRecords>,
    /// Whether the engine offers High findings: withheld in the app, offered by the sandbox autotest.
    pub(crate) high: HighLevel,
    /// How the scan gives out its progress: as walked, but in a demo (debug builds only).
    #[cfg(debug_assertions)]
    pub(crate) pace: crate::sandbox::ScanPace,
}

/// The engine both paths share: the real walk of the disk over `shared`, the app's modules and its settings file.
pub(crate) fn assemble(shared: Shared) -> Result<Engine, ComposeError> {
    let Shared {
        folder,
        reader,
        avoid,
        known_folders,
        volumes,
        clock,
        running,
        commands,
        access,
        cleanup,
        modules,
        apps,
        installs,
        high,
        #[cfg(debug_assertions)]
        pace,
    } = shared;
    let options = ScannerOptions {
        standing_avoid: avoid,
        cache_body_limit: CachePolicy::DEFAULT.body_limit,
        ..ScannerOptions::default()
    };
    let journal = MacChangeJournal::new().with_time_limit(JOURNAL_TIME_LIMIT);
    let system = system_ports(Arc::clone(&known_folders), apps, running, installs, &reader);
    let walk = Scanner::new(
        reader,
        volumes,
        Arc::new(journal),
        Arc::new(FileScanCache::new(folder.clone())),
        known_folders_anchor(&*known_folders),
        options,
    )
    // One clock dates the scan's states and the journal's records, so the Trash view can compare them.
    .with_clock(Arc::clone(&clock));
    let walk: Arc<dyn DiskWalk> = Arc::new(walk);
    #[cfg(debug_assertions)]
    let walk = paced(walk, pace, &clock);
    let system_ui = Arc::new(MacSystemUi::new(Arc::clone(&commands)));
    let ports =
        Ports { walk, settings: Arc::new(FileSettingsStore::new(folder)), clock, commands, access, system_ui, system };
    Ok(Engine::with_cleanup(registry_of(modules)?, ports, cleanup, high)?)
}

/// `walk` as `pace` asks: a demo's is held to its minimum time, sleeping the scan's thread between its progresses.
#[cfg(debug_assertions)]
fn paced(walk: Arc<dyn DiskWalk>, pace: crate::sandbox::ScanPace, clock: &Arc<dyn Clock>) -> Arc<dyn DiskWalk> {
    match pace {
        crate::sandbox::ScanPace::AsWalked => walk,
        crate::sandbox::ScanPace::Demo { used } => Arc::new(fresher_sandbox::PacedWalk::new(
            walk,
            Arc::clone(clock),
            Arc::new(std::thread::sleep),
            fresher_sandbox::DEMO_SCAN_TIME,
            used,
        )),
    }
}

/// The system's ports, whose small files are read through `reader`, the walk's own (in sandbox mode the sandbox's
/// stated lens over it), so a module reads a file as the walk would.
fn system_ports(
    known_folders: Arc<dyn KnownFolders>,
    apps: Arc<dyn InstalledApps>,
    running: Arc<dyn RunningApps>,
    installs: Arc<dyn InstallRecords>,
    reader: &Arc<dyn FolderReader>,
) -> SystemPorts {
    SystemPorts { known_folders, apps, running, installs, files: Arc::new(ReaderFiles(Arc::clone(reader))) }
}

/// What a cleanup goes through: a cleaner with `reach`, acting as the user who runs the app, kept from the protected
/// places of `home` and `user_temp` (with the app's data `folder` and its bundle), over `ports`; the journal in
/// `folder`; the `volumes` whose space a cleanup measures; and Finder's settings, read through `finder`.
pub(crate) fn cleanup(
    home: &Path,
    user_temp: Option<&Path>,
    folder: &DataFolder,
    ports: CleanerPorts,
    volumes: Arc<dyn Volumes>,
    finder: Arc<dyn FinderSettings>,
    reach: Reach,
) -> Result<CleanupPorts, ComposeError> {
    let protection = real_protection(home, folder.path(), user_temp)?;
    let cleaner = DiskCleaner::new(reach, current_user(), home.to_path_buf(), protection, ports);
    let journal = Arc::new(FileCleanupJournal::open(folder));
    Ok(CleanupPorts { cleaner: Arc::new(cleaner), journal, volumes, finder, trashes: Vec::new() })
}

/// The protected places of this Mac for `home`, with the app's `data_folder`, in a built app its bundle, and the
/// user's own temporary folder when it is known ([`user_temp`]).
pub(crate) fn real_protection(
    home: &Path,
    data_folder: &Path,
    user_temp: Option<&Path>,
) -> Result<Protection, ComposeError> {
    let executable = std::env::current_exe().map_err(ComposeError::Executable)?;
    Ok(protection(home, data_folder, bundle_of(&executable), user_temp)?)
}

/// The user's own temporary folder, read once from the system's configuration and made canonical (`/var` is a link
/// to `/private/var`). None when it cannot be read: the folder then matches nothing and stays protected.
pub(crate) fn user_temp() -> Option<PathBuf> {
    checked_user_temp(user_temp_dir().and_then(std::fs::canonicalize))
}

/// The canonical `read` when it is shaped as a user's own temporary folder (`/private/var/folders/<xx>/<hash>/T`,
/// [`is_user_temp`]); any other answer is refused (logged).
fn checked_user_temp(read: io::Result<PathBuf>) -> Option<PathBuf> {
    match read {
        Ok(temp) if is_user_temp(&temp) => Some(temp),
        Ok(temp) => {
            tracing::warn!(folder = %temp.display(), "the user's temporary folder is not where macOS keeps it");
            None
        }
        Err(error) => {
            tracing::warn!(%error, "the user's temporary folder is unknown");
            None
        }
    }
}

/// How far the cleaner of a release build reaches: the real disk. The only place that makes the token.
#[cfg(not(any(debug_assertions, test)))]
const fn reach() -> Reach {
    Reach::RealDisk(RealDiskToken::new())
}

/// How far the cleaner of a development build reaches: the real disk, checked and never acted on
/// (`docs/architecture/removal.md`).
#[cfg(debug_assertions)]
pub(crate) const fn reach() -> Reach {
    Reach::DryRun(DryRunToken::new())
}

/// How far the cleaner of a release-mode test reaches: only below a sandbox folder, never the real disk.
#[cfg(all(test, not(debug_assertions)))]
const fn reach() -> Reach {
    Reach::Sandbox
}

/// The app's bundle around `executable` (`Fresher.app/Contents/MacOS/…`): the outermost folder named `*.app`, since a
/// bundle may hold helper apps. A development build runs from `target/` and has none.
fn bundle_of(executable: &Path) -> Option<&Path> {
    let is_bundle = |folder: &&Path| folder.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("app"));
    executable.ancestors().skip(1).filter(is_bundle).last()
}

/// Every module of the app, in a fixed order. The dev-only `demo` and `trial` modules run in debug builds only.
#[cfg(test)]
pub(crate) fn registry() -> Result<Registry, ComposeError> {
    registry_of(ModuleSet::Every)
}

/// Which of the app's modules an engine registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleSet {
    /// Every module of the build, the dev-only ones of a debug build included.
    Every,
    /// The modules a release ships: a debug build's dev-only ones (the `Debug` group) left out, as a demo plays.
    Shipped,
    /// Only the one with this id (none when the app has no such module), as the autotest of a case file runs.
    Only(ModuleId),
}

impl ModuleSet {
    /// Whether `module` is in the set.
    fn holds(self, module: &dyn Module) -> bool {
        let descriptor = module.descriptor();
        match self {
            Self::Every => true,
            Self::Shipped => descriptor.group() != ModuleGroup::Debug,
            Self::Only(id) => descriptor.id() == id,
        }
    }
}

/// The app's modules of `set`.
fn registry_of(set: ModuleSet) -> Result<Registry, ComposeError> {
    let mut registry = Registry::new()?;
    for module in modules()? {
        if set.holds(module.as_ref()) {
            registry.register(module)?;
        }
    }
    Ok(registry)
}

/// Every module of the app, in a fixed order: the feature modules of every build, then the dev-only ones. The scan
/// screen's "Looking at" line names them in this order, so the general ones (caches, temporary files, logs) come first.
fn modules() -> Result<Vec<Box<dyn Module>>, ComposeError> {
    let modules: Vec<Box<dyn Module>> = vec![
        Box::new(fresher_module_app_caches::AppCachesModule::new()?),
        Box::new(fresher_module_temp::TempModule::new()?),
        Box::new(fresher_module_logs::LogsModule::new()?),
        Box::new(fresher_module_dev_caches::DevCachesModule::new()?),
        Box::new(fresher_module_app_leftovers::AppLeftoversModule::new()?),
        Box::new(fresher_module_downloads::DownloadsModule::new()?),
        Box::new(fresher_module_jetbrains::JetBrainsModule::new()?),
        Box::new(fresher_module_stale_projects::StaleProjectsModule::new()?),
        Box::new(fresher_module_local_ai_models::LocalAiModelsModule::new()?),
        Box::new(fresher_module_unused_toolchains::UnusedToolchainsModule::new()?),
        Box::new(fresher_module_xcode::XcodeModule::new()?),
        Box::new(fresher_module_docker::DockerModule::new()?),
    ];
    #[cfg(debug_assertions)]
    let modules = {
        let mut modules = modules;
        modules.push(Box::new(fresher_module_demo::DemoModule::new()?));
        modules.push(Box::new(fresher_module_trial::TrialModule::new()?));
        modules
    };
    Ok(modules)
}

#[cfg(test)]
mod tests {
    /// The system's ports of a home at `home` whose files are read through the sandbox's stated lens, as the sandbox's
    /// engine reads them: the file `.nvmrc` stated changed at a fixed time.
    fn lens_system(home: &std::path::Path) -> fresher_core::SystemFacts {
        use std::sync::Arc;

        use fresher_core::{FolderReader, Timestamp};
        use fresher_macos::MacFolderReader;
        use fresher_testing::{
            FactsLens, FakeInstallRecords, FakeInstalledApps, FakeKnownFolders, FakeRunningApps, Stated, StatedFacts,
        };

        let mut stated = StatedFacts::new();
        stated.insert(home.join(".nvmrc"), Stated { changed: Some(Timestamp::from_secs(7)), ..Stated::default() });
        let reader: Arc<dyn FolderReader> = Arc::new(FactsLens::new(MacFolderReader::new(), stated));
        let ports = super::system_ports(
            Arc::new(FakeKnownFolders::home_layout(home)),
            Arc::new(FakeInstalledApps::default()),
            Arc::new(FakeRunningApps::default()),
            Arc::new(FakeInstallRecords::default()),
            &reader,
        );
        fresher_core::SystemFacts::new(&ports)
    }

    /// An I/O policy that sets nothing, for the tests that do not check the policy. The tests of `start` cannot take
    /// it (`start` applies the real policy in a sandbox run): they need a process of their own, which nextest gives.
    const NO_POLICY: super::IoPolicy =
        super::IoPolicy { never_download_cloud_files: || Ok(()), leave_access_times: || Ok(()) };

    mod negative {
        use std::path::Path;

        use super::NO_POLICY;
        use crate::compose::{ComposeError, IoPolicy, bundle_of, checked_user_temp, engine_with, normal};

        #[test]
        fn the_systems_small_files_never_follow_a_link_through_the_lens() {
            let home = tempfile::tempdir().unwrap();
            std::fs::write(home.path().join("target"), "20\n").unwrap();
            std::os::unix::fs::symlink(home.path().join("target"), home.path().join(".nvmrc")).unwrap();

            let facts = super::lens_system(home.path());

            assert_eq!(facts.small_file(home.path(), ".nvmrc"), fresher_core::SmallFile::NotAFile);
        }

        #[test]
        fn the_normal_engine_has_no_standing_avoid_list() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let shared = normal(folder.path(), home.path(), &NO_POLICY).unwrap();

            assert_eq!(shared.avoid, Vec::<std::path::PathBuf>::new());
        }

        /// What keeps the default scan out instead: the normal access names the guarded places under `~` (the default
        /// root, as the settings write it), and on a home where nothing is granted none of them is allowed, so the
        /// engine's request avoids every one. The home is a temporary tree with no Full Disk Access candidate; nothing
        /// is tried.
        #[test]
        fn the_normal_access_keeps_the_default_root_out_of_every_guarded_place() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            std::fs::create_dir(home.path().join("Desktop")).unwrap();

            let shared = normal(folder.path(), home.path(), &NO_POLICY).unwrap();

            let status = shared.access.status();
            let places = shared.access.guarded_places(&[std::path::PathBuf::from("~")]);
            assert_eq!(status.access(&fresher_core::Capability::AllFiles), fresher_core::Access::Unknown);
            assert!(places.iter().any(|place| place.path == home.path().join("Desktop")), "{places:?}");
            assert!(places.iter().any(|place| place.path == home.path().join("Library/Containers")), "{places:?}");
            assert!(places.iter().all(|place| !status.access(&place.capability).allows()), "{places:?}");
        }

        #[test]
        fn a_relative_data_folder_is_refused() {
            let home = tempfile::tempdir().unwrap();

            let error = engine_with("relative/folder", home.path(), &NO_POLICY);

            assert!(matches!(error, Err(ComposeError::DataFolder(_))), "{error:?}");
        }

        #[test]
        fn the_app_refuses_to_start_without_the_io_policy() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let refusing =
                IoPolicy { never_download_cloud_files: || Err("refused".into()), leave_access_times: || Ok(()) };

            let error = engine_with(folder.path(), home.path(), &refusing);

            assert!(matches!(error, Err(ComposeError::IoPolicy(_))), "{error:?}");
        }

        #[test]
        fn a_user_temp_outside_the_users_folders_or_unread_is_refused() {
            for path in [
                "/private/var/folders",
                "/private/var/folders/xx",
                "/private/var/folders/xx/abc",
                "/private/var/folders/xx/abc/C",
                "/private/var/folders/xx/abc/T/TemporaryItems",
                "/private/tmp/T",
                "/var/folders/xx/abc/T",
                "/Users/me/T",
            ] {
                assert_eq!(checked_user_temp(Ok(path.into())), None, "{path}");
            }
            assert_eq!(checked_user_temp(Err(std::io::Error::other("unread"))), None);
        }

        #[test]
        fn a_development_build_has_no_bundle() {
            assert_eq!(bundle_of(Path::new("/Users/me/fresher/target/debug/fresher-app")), None);
            assert_eq!(bundle_of(Path::new("/Users/me/Fresher.app")), None);
        }

        /// Run by `cargo test --release`: a release build has no sandbox mode, whatever the environment says.
        #[cfg(not(debug_assertions))]
        #[test]
        fn a_release_build_ignores_the_variable() {
            let (folder, home, sandbox) =
                (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let named = sandbox.path().as_os_str().to_owned();
            let var = |name: &str| match name {
                "FRESHER_SANDBOX" => Some(named.clone()),
                "FRESHER_SANDBOX_TRASH" => Some("not a choice".into()),
                _ => None,
            };

            let start = crate::compose::start(folder.path().into(), home.path().into(), var).unwrap();

            assert_eq!((start.data_folder.as_path(), start.home.as_path()), (folder.path(), home.path()));
            assert_eq!(start.build, fresher_core::Build::Release);
            assert_eq!(std::fs::read_dir(sandbox.path()).unwrap().count(), 0);
        }

        /// The real composition of a development build (the real scanner, the real open-files check, the refusing
        /// Trash, the dry-run reach) on the trial module's folders beside a valid marker, where the sandbox reach
        /// would move them: the cleanup runs every check and changes nothing.
        #[cfg(debug_assertions)]
        #[test]
        fn the_development_engine_moves_nothing_on_the_real_disk() {
            use fresher_core::{RecordedOutcome, SettingUpdate, SettingValue, SkipCode};

            use crate::testing::{
                caches, echo, is_cleanup_finished, is_scan_finished, listing, trial_home, until_finished,
            };

            let (folder, tree) = (tempfile::tempdir().unwrap(), trial_home(true));
            let engine = engine_with(folder.path(), tree.root(), &NO_POLICY).unwrap();
            engine.accept_agreement(fresher_core::AGREEMENT_VERSION).unwrap();
            let roots = SettingValue::Folders(vec![tree.root().to_path_buf()]);
            engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
            let events = engine.subscribe();
            engine.start_scan(fresher_engine::ScanMode::Full).unwrap();
            until_finished(&events, is_scan_finished);
            let before = listing(tree.root());
            let view = engine.prepare_cleanup(&caches()).unwrap();

            let id = engine.confirm_cleanup(&echo(&view)).unwrap();

            until_finished(&events, is_cleanup_finished);
            assert!(view.summary.count > 0);
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
            assert_eq!(listing(tree.root()), before);
            let items = engine.cleanup(id).unwrap().items;
            assert_eq!(items.len(), view.summary.count as usize);
            for item in &items {
                assert_eq!(item.outcome, RecordedOutcome::Skipped { reason: SkipCode::DryRun }, "{item:?}");
            }
        }

        #[cfg(debug_assertions)]
        #[test]
        fn a_development_build_without_the_dev_runs_variable_begins_no_run() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let start = crate::compose::start(folder.path().into(), home.path().into(), |_| None).unwrap();

            assert!(start.run.is_none());
        }

        #[cfg(debug_assertions)]
        #[test]
        fn a_relative_dev_runs_folder_stops_the_start() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let var = |name: &str| (name == "FRESHER_DEV_RUNS").then(|| "dev-runs".into());

            let start = crate::compose::start(folder.path().into(), home.path().into(), var);

            assert!(
                matches!(start, Err(ComposeError::DevRuns(crate::dev_runs::DevRunsRefusal::NotAbsolute { .. }))),
                "{start:?}"
            );
        }

        #[cfg(debug_assertions)]
        #[test]
        fn a_demo_without_a_sandbox_stops_the_start() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let var = |name: &str| (name == "FRESHER_DEMO").then(|| "/private/tmp/overview.toml".into());

            let start = crate::compose::start(folder.path().into(), home.path().into(), var);

            assert!(
                matches!(start, Err(ComposeError::DemoMode(crate::demo::DemoRefusal::WithoutSandbox))),
                "{start:?}"
            );
        }

        /// Run by `cargo test --release`: a release build records nothing, whatever the environment says.
        #[cfg(not(debug_assertions))]
        #[test]
        fn a_release_build_ignores_the_dev_runs_variable() {
            let (folder, home, records) =
                (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let named = records.path().as_os_str().to_owned();
            let var = |name: &str| match name {
                "FRESHER_DEV_RUNS" => Some(named.clone()),
                "FRESHER_BUILD_COMMIT" => Some("26e4cb0".into()),
                _ => None,
            };

            let start = crate::compose::start(folder.path().into(), home.path().into(), var).unwrap();

            assert_eq!(start.build, fresher_core::Build::Release);
            assert_eq!(std::fs::read_dir(records.path()).unwrap().count(), 0);
        }

        /// Nothing happens without the user (the user, 2026-11-25): the start reads no disk, so the engine it gives is
        /// idle and knows no last scan.
        #[test]
        fn the_start_begins_no_operation() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let start = crate::compose::start(folder.path().into(), home.path().into(), |_| None).unwrap();

            let overview = start.engine.overview().unwrap();
            assert_eq!(overview.status, fresher_core::EngineStatus::Idle);
            assert_eq!(overview.last_scan, None);
        }

        #[test]
        fn the_app_starts_when_access_times_cannot_be_left_alone() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let partial =
                IoPolicy { never_download_cloud_files: || Ok(()), leave_access_times: || Err("refused".into()) };

            assert!(engine_with(folder.path(), home.path(), &partial).is_ok());
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use super::NO_POLICY;
        use crate::compose::{ModuleSet, bundle_of, checked_user_temp, engine_with, modules, user_temp};

        #[test]
        fn a_home_reached_through_a_link_is_spelled_as_the_disk_spells_it() {
            let temp = tempfile::tempdir().unwrap();
            let real = std::fs::canonicalize(temp.path()).unwrap().join("Data/me");
            std::fs::create_dir_all(real.join(".Trash")).unwrap();
            std::os::unix::fs::symlink(&real, temp.path().join("me")).unwrap();
            let data = tempfile::tempdir().unwrap();

            let start = crate::compose::start(data.path().into(), temp.path().join("me"), |_| None).unwrap();

            assert_eq!(crate::compose::disk_home(temp.path().join("me")), real);
            assert_eq!(start.home, real);
            assert_eq!(crate::compose::disk_home(temp.path().join("missing")), temp.path().join("missing"));
        }

        #[test]
        fn the_systems_small_files_read_a_file_of_the_home_through_the_lens() {
            let home = tempfile::tempdir().unwrap();
            std::fs::write(home.path().join(".nvmrc"), "20\n").unwrap();

            let facts = super::lens_system(home.path());

            assert_eq!(facts.small_file(home.path(), ".nvmrc"), fresher_core::SmallFile::Text("20\n".into()));
        }

        #[cfg(debug_assertions)]
        #[test]
        fn a_development_build_reaches_the_real_disk_without_acting() {
            assert!(matches!(crate::compose::reach(), fresher_cleaner::Reach::DryRun(_)));
        }

        #[cfg(debug_assertions)]
        #[test]
        fn a_development_build_with_the_variable_begins_one_record() {
            let (folder, home, records) =
                (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let named = records.path().as_os_str().to_owned();
            let var = |name: &str| (name == "FRESHER_DEV_RUNS").then(|| named.clone());

            let start = crate::compose::start(folder.path().into(), home.path().into(), var).unwrap();

            assert!(start.run.is_some());
            let files: Vec<String> = std::fs::read_dir(records.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            assert_eq!(files.len(), 2, "{files:?}");
            assert_eq!(files.iter().filter(|name| name.ends_with("_mac.json")).count(), 1, "{files:?}");
            assert_eq!(files.iter().filter(|name| name.ends_with("_mac.events.jsonl")).count(), 1, "{files:?}");
        }

        #[test]
        fn the_user_temp_is_kept_inside_the_users_folders() {
            let temp = PathBuf::from("/private/var/folders/xx/abc/T");

            assert_eq!(checked_user_temp(Ok(temp.clone())), Some(temp));
        }

        #[test]
        fn this_macs_user_temp_is_canonical() {
            let temp = user_temp().unwrap();

            assert!(temp.starts_with("/private/var/folders"), "{}", temp.display());
            assert!(temp.ends_with("T"), "{}", temp.display());
        }

        #[test]
        fn the_bundle_is_the_outermost_app_around_the_executable() {
            let helper = Path::new("/Applications/Fresher.app/Contents/Helpers/Helper.app/Contents/MacOS/helper");

            assert_eq!(
                bundle_of(Path::new("/Users/me/Apps/Fresher.app/Contents/MacOS/fresher")),
                Some(Path::new("/Users/me/Apps/Fresher.app"))
            );
            assert_eq!(bundle_of(helper), Some(Path::new("/Applications/Fresher.app")));
            assert_eq!(
                bundle_of(Path::new("/Volumes/X/FRESHER.APP/Contents/MacOS/fresher")),
                Some(Path::new("/Volumes/X/FRESHER.APP"))
            );
        }

        /// `cargo test --release` builds without `debug_assertions`: there only the feature modules are registered.
        #[test]
        fn the_engine_runs_the_feature_modules_always_and_the_dev_only_ones_in_debug_builds_only() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let expected: &[&str] = if cfg!(debug_assertions) {
                &[
                    "app-caches",
                    "temp",
                    "logs",
                    "dev-caches",
                    "app-leftovers",
                    "downloads",
                    "jetbrains",
                    "stale-projects",
                    "local-ai-models",
                    "unused-toolchains",
                    "xcode",
                    "docker",
                    "demo",
                    "trial",
                ]
            } else {
                &[
                    "app-caches",
                    "temp",
                    "logs",
                    "dev-caches",
                    "app-leftovers",
                    "downloads",
                    "jetbrains",
                    "stale-projects",
                    "local-ai-models",
                    "unused-toolchains",
                    "xcode",
                    "docker",
                ]
            };

            let engine = engine_with(folder.path(), home.path(), &NO_POLICY).unwrap();

            let ids: Vec<&str> = engine.modules().iter().map(|module| module.id.as_str()).collect();
            assert_eq!(ids, expected);
        }

        #[test]
        fn the_shipped_set_leaves_out_the_dev_only_modules_in_every_build() {
            let modules = modules().unwrap();

            let ids: Vec<&str> = modules
                .iter()
                .filter(|module| ModuleSet::Shipped.holds(module.as_ref()))
                .map(|module| module.descriptor().id().as_str())
                .collect();
            assert_eq!(
                ids,
                [
                    "app-caches",
                    "temp",
                    "logs",
                    "dev-caches",
                    "app-leftovers",
                    "downloads",
                    "jetbrains",
                    "stale-projects",
                    "local-ai-models",
                    "unused-toolchains",
                    "xcode",
                    "docker",
                ]
            );
        }

        #[test]
        fn the_normal_engine_withholds_high() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let engine = engine_with(folder.path(), home.path(), &NO_POLICY).unwrap();

            assert_eq!(engine.high_level(), fresher_core::HighLevel::Withheld);
        }

        #[test]
        fn the_engine_records_its_cleanups_in_the_data_folder() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let engine = engine_with(folder.path(), home.path(), &NO_POLICY).unwrap();

            let page = engine.cleanups(0, 10);
            assert_eq!((page.total, page.blocked), (0, None));
        }
    }
}
