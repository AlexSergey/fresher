//! The composition root: the real adapters, the modules, and the engine they make.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fresher_core::SchemaError;
use fresher_engine::{Engine, Ports, Registry, RegistryError};
use fresher_macos::{MacChangeJournal, MacFolderReader, MacVolumes, io_policy, places_that_prompt};
use fresher_scanner::{Scanner, ScannerOptions, home_anchor};
use fresher_store::{CachePolicy, DataFolder, DataFolderError, FileScanCache, FileSettingsStore};

use crate::adapters::{SystemClock, UnknownAccess};
use crate::process::{COMMAND_TIMEOUT, ProcessCommandRunner};

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
    #[cfg(debug_assertions)]
    #[error("the demo module cannot be built: {0}")]
    Demo(#[from] fresher_module_demo::DemoError),
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
/// folder (Tauri's `home_dir`): the anchor of home-relative interests and of `~` in the scan roots.
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
    let journal = MacChangeJournal::new().with_time_limit(JOURNAL_TIME_LIMIT);
    let walk = Scanner::new(
        Arc::new(MacFolderReader::new()),
        Arc::new(MacVolumes::new()),
        Arc::new(journal),
        Arc::new(FileScanCache::new(folder.clone())),
        home_anchor(home),
        options,
    );
    let ports = Ports {
        walk: Arc::new(walk),
        settings: Arc::new(FileSettingsStore::new(folder)),
        clock: Arc::new(SystemClock),
        commands: Arc::new(ProcessCommandRunner::new(COMMAND_TIMEOUT)),
        access: Arc::new(UnknownAccess),
    };
    Ok(Engine::new(registry()?, ports)?)
}

/// Every module of the app, in a fixed order. The `demo` module runs in debug builds only.
pub(crate) fn registry() -> Result<Registry, ComposeError> {
    let registry = Registry::new()?;
    #[cfg(debug_assertions)]
    let registry = {
        let mut registry = registry;
        registry.register(Box::new(fresher_module_demo::DemoModule::new()?))?;
        registry
    };
    Ok(registry)
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::compose::{ComposeError, IoPolicy, engine, engine_with};

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
        fn the_app_starts_when_access_times_cannot_be_left_alone() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let partial =
                IoPolicy { never_download_cloud_files: || Ok(()), leave_access_times: || Err("refused".into()) };

            assert!(engine_with(folder.path(), home.path(), &partial).is_ok());
        }
    }

    mod positive {
        use crate::compose::engine;

        #[test]
        fn the_engine_runs_the_demo_module_in_debug_builds() {
            let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let engine = engine(folder.path(), home.path()).unwrap();

            let ids: Vec<&str> = engine.modules().iter().map(|module| module.id.as_str()).collect();
            assert_eq!(ids, ["demo"]);
        }
    }
}
