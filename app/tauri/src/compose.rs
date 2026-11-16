//! The composition root: the real adapters, the modules, and the engine they make.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(any(debug_assertions, test)))]
use fresher_cleaner::RealDiskToken;
use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
use fresher_core::{
    Clock, CommandRunner, KnownFolders, ProtectionError, RunningApps, SchemaError, SystemPorts, Volumes,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, RegistryError};
use fresher_macos::{
    MacChangeJournal, MacFolderReader, MacOpenFiles, MacTrash, MacVolumes, current_user, io_policy, places_that_prompt,
    protection,
};
use fresher_scanner::{Scanner, ScannerOptions, known_folders_anchor};
use fresher_store::{CachePolicy, DataFolder, DataFolderError, FileCleanupJournal, FileScanCache, FileSettingsStore};

use fresher_system::{
    COMMAND_TIMEOUT, MacInstalledApps, MacKnownFolders, MacRunningApps, ProcessCommandRunner, SystemClock,
};

use crate::adapters::{NoSystemUi, UnknownAccess};

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
    Executable(#[source] std::io::Error),
    #[error("the protected places cannot be listed: {0}")]
    Protection(#[from] ProtectionError),
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
const MACOS_POLICY: IoPolicy = IoPolicy {
    never_download_cloud_files: || io_policy::never_download_cloud_files().map_err(|error| error.to_string()),
    leave_access_times: || io_policy::leave_access_times().map_err(|error| error.to_string()),
};

/// The engine of the app, with its settings in `data_folder` (Tauri's `app_data_dir`) and `home` the user's home
/// folder (Tauri's `home_dir`): where the known folders are (`MacKnownFolders`), and `~` in the scan roots.
pub(crate) fn engine(data_folder: impl Into<PathBuf>, home: &Path) -> Result<Engine, ComposeError> {
    engine_with(data_folder, home, &MACOS_POLICY)
}

fn engine_with(data_folder: impl Into<PathBuf>, home: &Path, policy: &IoPolicy) -> Result<Engine, ComposeError> {
    (policy.never_download_cloud_files)().map_err(ComposeError::IoPolicy)?;
    if let Err(error) = (policy.leave_access_times)() {
        tracing::warn!(%error, "reading files may update their access times");
    }
    let folder = DataFolder::open(data_folder)?;
    // Until plan 006 decides with the capability state, the places that prompt are never opened (docs/hacks/).
    let options = ScannerOptions {
        standing_avoid: places_that_prompt(home),
        cache_body_limit: CachePolicy::DEFAULT.body_limit,
        ..ScannerOptions::default()
    };
    let known_folders: Arc<dyn KnownFolders> = Arc::new(MacKnownFolders::new(home));
    let journal = MacChangeJournal::new().with_time_limit(JOURNAL_TIME_LIMIT);
    let volumes: Arc<dyn Volumes> = Arc::new(MacVolumes::new());
    let walk = Scanner::new(
        Arc::new(MacFolderReader::new()),
        Arc::clone(&volumes),
        Arc::new(journal),
        Arc::new(FileScanCache::new(folder.clone())),
        known_folders_anchor(&*known_folders),
        options,
    );
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
    let cleanup = cleanup(home, &folder, cleaner, volumes)?;
    let ports = Ports {
        walk: Arc::new(walk),
        settings: Arc::new(FileSettingsStore::new(folder)),
        clock,
        commands,
        access: Arc::new(UnknownAccess),
        system_ui: Arc::new(NoSystemUi),
        system: SystemPorts {
            known_folders: Arc::clone(&known_folders),
            apps: Arc::new(MacInstalledApps::new(known_folders)),
            running,
        },
    };
    Ok(Engine::with_cleanup(registry()?, ports, cleanup)?)
}

/// What a cleanup goes through: this build's cleaner, acting as the user who runs the app, kept from the protected
/// places of `home` (with the app's data `folder` and its bundle), over `ports`; the journal in `folder`; and the
/// `volumes` whose space a cleanup measures.
pub(crate) fn cleanup(
    home: &Path,
    folder: &DataFolder,
    ports: CleanerPorts,
    volumes: Arc<dyn Volumes>,
) -> Result<CleanupPorts, ComposeError> {
    let executable = std::env::current_exe().map_err(ComposeError::Executable)?;
    let protection = protection(home, folder.path(), bundle_of(&executable))?;
    let cleaner = DiskCleaner::new(reach(), current_user(), home.to_path_buf(), protection, ports);
    Ok(CleanupPorts { cleaner: Arc::new(cleaner), journal: Arc::new(FileCleanupJournal::open(folder)), volumes })
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
pub(crate) fn registry() -> Result<Registry, ComposeError> {
    let registry = Registry::new()?;
    #[cfg(debug_assertions)]
    let registry = {
        let mut registry = registry;
        registry.register(Box::new(fresher_module_demo::DemoModule::new()?))?;
        registry.register(Box::new(fresher_module_trial::TrialModule::new()?))?;
        registry
    };
    Ok(registry)
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::compose::{ComposeError, IoPolicy, bundle_of, engine, engine_with};

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
        fn a_development_build_has_no_bundle() {
            assert_eq!(bundle_of(Path::new("/Users/me/fresher/target/debug/fresher-app")), None);
            assert_eq!(bundle_of(Path::new("/Users/me/Fresher.app")), None);
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
        use std::path::Path;

        use crate::compose::{bundle_of, engine};

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

        /// `cargo test --release` builds without `debug_assertions`: there neither dev-only module is registered.
        #[test]
        fn the_engine_runs_the_dev_only_modules_in_debug_builds_only() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let dev_only: &[&str] = if cfg!(debug_assertions) { &["demo", "trial"] } else { &[] };

            let engine = engine(folder.path(), home.path()).unwrap();

            let ids: Vec<&str> = engine.modules().iter().map(|module| module.id.as_str()).collect();
            assert_eq!(ids, dev_only);
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
