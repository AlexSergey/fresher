//! The composition root: the real adapters, the modules, and the engine they make.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(any(debug_assertions, test)))]
use fresher_cleaner::RealDiskToken;
use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
use fresher_core::{
    AccessProbe, Build, Clock, CommandRunner, FolderReader, InstallRecords, InstalledApps, KnownFolders, Module,
    ModuleId, Protection, ProtectionError, RunningApps, SchemaError, SystemPorts, Volumes,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, RegistryError};
use fresher_macos::{
    MacChangeJournal, MacFolderReader, MacOpenFiles, MacTrash, MacVolumes, current_user, io_policy, is_user_temp,
    protection, user_temp_dir,
};
use fresher_scanner::{Scanner, ScannerOptions, known_folders_anchor};
use fresher_store::{CachePolicy, DataFolder, DataFolderError, FileCleanupJournal, FileScanCache, FileSettingsStore};

use fresher_system::{
    AppSearch, COMMAND_TIMEOUT, INDEX_CHECK_TIMEOUT, MacAccess, MacInstallRecords, MacInstalledApps, MacKnownFolders,
    MacMounts, MacRunningApps, MacSystemUi, ProcessCommandRunner, SpotlightSearch, SystemClock, macos_version,
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
    #[error("the downloads module cannot be built: {0}")]
    Downloads(#[from] fresher_module_downloads::DownloadsError),
    /// A development build started on a sandbox that is not one: it does not start rather than run elsewhere.
    #[cfg(debug_assertions)]
    #[error("sandbox mode is refused: {0}")]
    Sandbox(#[from] crate::sandbox::SandboxRefusal),
    #[cfg(debug_assertions)]
    #[error("the demo module cannot be built: {0}")]
    Demo(#[from] fresher_module_demo::DemoError),
    #[cfg(debug_assertions)]
    #[error("the trial module cannot be built: {0}")]
    Trial(#[from] fresher_module_trial::TrialError),
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
}

/// The app's start: in a development build whose environment (read through `var`) names a sandbox in
/// `FRESHER_SANDBOX`, the sandbox's engine, data folder and home (`data_folder` is then only protected); otherwise
/// the engine of [`engine`] on `data_folder` and `home`. A release build reads no variable.
pub(crate) fn start(
    data_folder: PathBuf,
    home: PathBuf,
    #[cfg_attr(not(debug_assertions), expect(unused_variables, reason = "a release build has no sandbox mode"))]
    var: impl Fn(&str) -> Option<OsString>,
) -> Result<Start, ComposeError> {
    #[cfg(debug_assertions)]
    if let Some(mode) = crate::sandbox::SandboxMode::from_env(var)? {
        apply(&MACOS_POLICY)?;
        // The running apps come from the sandbox's world, never the Mac's: an app open on the Mac changes nothing.
        let (engine, sandbox) =
            crate::sandbox::compose(&mode, &home, &data_folder, crate::sandbox::SandboxPorts::real)?;
        let build = crate::sandbox::build(&sandbox)?;
        return Ok(Start { engine, data_folder: sandbox.data_folder(), home: sandbox.home(), build });
    }
    let engine = engine(&data_folder, &home)?;
    Ok(Start { engine, data_folder, home, build: own_build() })
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
    let volumes: Arc<dyn Volumes> = Arc::new(MacVolumes::new());
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let running: Arc<dyn RunningApps> = Arc::new(MacRunningApps::new());
    let commands: Arc<dyn CommandRunner> = Arc::new(ProcessCommandRunner::new(COMMAND_TIMEOUT));
    // The system's Trash goes to the cleaner and to nothing else.
    let cleaner = CleanerPorts {
        trash: Arc::new(MacTrash::new()),
        open_files: Arc::new(MacOpenFiles::new()),
        running: Arc::clone(&running),
        commands: Arc::clone(&commands),
        clock: Arc::clone(&clock),
    };
    let user_temp = user_temp();
    let cleanup = cleanup(home, user_temp.as_deref(), &folder, cleaner, Arc::clone(&volumes))?;
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
        apps: Arc::new(MacInstalledApps::new(
            known_folders,
            AppSearch::Spotlight(SpotlightSearch::new(
                Arc::clone(&commands),
                Arc::new(ProcessCommandRunner::new(INDEX_CHECK_TIMEOUT)),
                Arc::new(MacMounts),
            )),
        )),
        installs: Arc::new(MacInstallRecords::new("/")),
        commands,
        access: Arc::new(MacAccess::new(home.to_path_buf(), macos_version(Path::new("/")))),
        cleanup,
        only: None,
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
/// read through, the cleanup, and the one module registered (`None`: every module of the app).
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
    pub(crate) only: Option<ModuleId>,
    /// The installed apps: the apps folders and Spotlight on a Mac; in a sandbox its listed and declared apps, with the
    /// change times its world states.
    pub(crate) apps: Arc<dyn InstalledApps>,
    /// The installer receipts and the macOS version: the Mac's, or the sandbox's world's.
    pub(crate) installs: Arc<dyn InstallRecords>,
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
        only,
        apps,
        installs,
    } = shared;
    let options = ScannerOptions {
        standing_avoid: avoid,
        cache_body_limit: CachePolicy::DEFAULT.body_limit,
        ..ScannerOptions::default()
    };
    let journal = MacChangeJournal::new().with_time_limit(JOURNAL_TIME_LIMIT);
    let walk = Scanner::new(
        reader,
        volumes,
        Arc::new(journal),
        Arc::new(FileScanCache::new(folder.clone())),
        known_folders_anchor(&*known_folders),
        options,
    );
    let system_ui = Arc::new(MacSystemUi::new(Arc::clone(&commands)));
    let ports = Ports {
        walk: Arc::new(walk),
        settings: Arc::new(FileSettingsStore::new(folder)),
        clock,
        commands,
        access,
        system_ui,
        system: SystemPorts { known_folders: Arc::clone(&known_folders), apps, running, installs },
    };
    Ok(Engine::with_cleanup(registry_of(only)?, ports, cleanup)?)
}

/// What a cleanup goes through: this build's cleaner, acting as the user who runs the app, kept from the protected
/// places of `home` and `user_temp` (with the app's data `folder` and its bundle), over `ports`; the journal in
/// `folder`; and the `volumes` whose space a cleanup measures.
pub(crate) fn cleanup(
    home: &Path,
    user_temp: Option<&Path>,
    folder: &DataFolder,
    ports: CleanerPorts,
    volumes: Arc<dyn Volumes>,
) -> Result<CleanupPorts, ComposeError> {
    let protection = real_protection(home, folder.path(), user_temp)?;
    let cleaner = DiskCleaner::new(reach(), current_user(), home.to_path_buf(), protection, ports);
    Ok(CleanupPorts { cleaner: Arc::new(cleaner), journal: Arc::new(FileCleanupJournal::open(folder)), volumes })
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

/// How far the cleaner of a development build or a test reaches: only below a sandbox folder (`docs/hacks/`).
#[cfg(any(debug_assertions, test))]
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
    registry_of(None)
}

/// The app's modules, or only the one with the id `only` (none when the app has no such module).
fn registry_of(only: Option<ModuleId>) -> Result<Registry, ComposeError> {
    let mut registry = Registry::new()?;
    for module in modules()? {
        if only.is_none_or(|id| module.descriptor().id() == id) {
            registry.register(module)?;
        }
    }
    Ok(registry)
}

/// Every module of the app, in a fixed order: the feature modules of every build, then the dev-only ones.
fn modules() -> Result<Vec<Box<dyn Module>>, ComposeError> {
    let modules: Vec<Box<dyn Module>> = vec![
        Box::new(fresher_module_jetbrains::JetBrainsModule::new()?),
        Box::new(fresher_module_temp::TempModule::new()?),
        Box::new(fresher_module_dev_caches::DevCachesModule::new()?),
        Box::new(fresher_module_app_leftovers::AppLeftoversModule::new()?),
        Box::new(fresher_module_downloads::DownloadsModule::new()?),
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
    mod negative {
        use std::path::Path;

        use crate::compose::{
            ComposeError, IoPolicy, MACOS_POLICY, bundle_of, checked_user_temp, engine, engine_with, normal,
        };

        #[test]
        fn the_normal_engine_has_no_standing_avoid_list() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let shared = normal(folder.path(), home.path(), &MACOS_POLICY).unwrap();

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

            let shared = normal(folder.path(), home.path(), &MACOS_POLICY).unwrap();

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

            let error = engine("relative/folder", home.path());

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

        use crate::compose::{bundle_of, checked_user_temp, engine, user_temp};

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
                &["jetbrains", "temp", "dev-caches", "app-leftovers", "downloads", "demo", "trial"]
            } else {
                &["jetbrains", "temp", "dev-caches", "app-leftovers", "downloads"]
            };

            let engine = engine(folder.path(), home.path()).unwrap();

            let ids: Vec<&str> = engine.modules().iter().map(|module| module.id.as_str()).collect();
            assert_eq!(ids, expected);
        }

        #[test]
        fn the_engine_records_its_cleanups_in_the_data_folder() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let engine = engine(folder.path(), home.path()).unwrap();

            let page = engine.cleanups(0, 10);
            assert_eq!((page.total, page.blocked), (0, None));
        }
    }
}
