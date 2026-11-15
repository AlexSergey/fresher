//! The composition root: the real adapters, the modules, and the engine they make.

use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::SchemaError;
use fresher_engine::{Engine, Ports, Registry, RegistryError};
use fresher_store::{DataFolder, DataFolderError, FileSettingsStore};

use crate::adapters::{NoWalk, SystemClock, UnknownAccess};
use crate::process::{COMMAND_TIMEOUT, ProcessCommandRunner};

/// Why the app could not start its engine.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ComposeError {
    #[error("the data folder cannot be used: {0}")]
    DataFolder(#[from] DataFolderError),
    #[error("a module was refused: {0}")]
    Registry(#[from] RegistryError),
    #[error("the settings schema is refused: {0}")]
    Schema(#[from] SchemaError),
    #[cfg(debug_assertions)]
    #[error("the demo module cannot be built: {0}")]
    Demo(#[from] fresher_module_demo::DemoError),
}

/// The engine of the app, with its settings in `data_folder` (Tauri's `app_data_dir`).
pub(crate) fn engine(data_folder: impl Into<PathBuf>) -> Result<Engine, ComposeError> {
    let folder = DataFolder::open(data_folder)?;
    let ports = Ports {
        walk: Arc::new(NoWalk),
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
        use crate::compose::{ComposeError, engine};

        #[test]
        fn a_relative_data_folder_is_refused() {
            let error = engine("relative/folder");

            assert!(matches!(error, Err(ComposeError::DataFolder(_))), "{error:?}");
        }
    }

    mod positive {
        use crate::compose::engine;

        #[test]
        fn the_engine_runs_the_demo_module_in_debug_builds() {
            let folder = tempfile::tempdir().unwrap();

            let engine = engine(folder.path()).unwrap();

            let ids: Vec<&str> = engine.modules().iter().map(|module| module.id.as_str()).collect();
            assert_eq!(ids, ["demo"]);
        }
    }
}
