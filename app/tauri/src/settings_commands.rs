//! The settings screen's commands that the engine does not answer: the system's folder picker, and the size of the
//! app's own data folder.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use std::path::{Path, PathBuf};

use fresher_core::{CommandError, OwnData, Refusal};
use fresher_store::DataFolder;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

/// The user's home folder, looked up once in the app's setup and managed as state.
pub(crate) struct HomeFolder(pub(crate) PathBuf);

/// The folder the user picks in the system's folder picker, `null` when the picker was cancelled. A folder in the
/// home folder is answered as `~/…`, as the settings write it. The engine checks it like any other value when the UI
/// sends the new list.
#[tauri::command(async)]
pub(crate) fn pick_folder<R: Runtime>(app: AppHandle<R>, home: State<'_, HomeFolder>) -> Option<PathBuf> {
    let picked = app.dialog().file().blocking_pick_folder()?.into_path().ok()?;
    Some(home_relative(&picked, &home.0))
}

/// What the app's own data folder takes on disk, against its budget.
#[tauri::command(async)]
pub(crate) fn get_own_data(folder: State<'_, DataFolder>) -> Result<OwnData, Refusal> {
    let footprint = folder.footprint().map_err(|error| {
        tracing::warn!(%error, "the data folder could not be measured");
        Refusal::from(CommandError::Internal)
    })?;
    Ok(OwnData { total: footprint.total, budget: footprint.budget })
}

/// `path` written from the home folder (`~`, `~/…`) when it is inside `home`; as it is otherwise.
fn home_relative(path: &Path, home: &Path) -> PathBuf {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => PathBuf::from("~"),
        Ok(rest) => Path::new("~").join(rest),
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::{Path, PathBuf};

        use super::super::home_relative;

        #[test]
        fn a_folder_outside_the_home_or_sharing_its_prefix_stays_as_it_is() {
            let home = Path::new("/Users/ann");

            assert_eq!(home_relative(Path::new("/Volumes/Work"), home), PathBuf::from("/Volumes/Work"));
            assert_eq!(home_relative(Path::new("/Users/anna/Projects"), home), PathBuf::from("/Users/anna/Projects"));
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use super::super::home_relative;
        use crate::settings_commands::get_own_data;
        use crate::testing::Shell;

        #[test]
        fn a_folder_in_the_home_is_written_from_it() {
            let home = Path::new("/Users/ann");

            assert_eq!(home_relative(Path::new("/Users/ann"), home), PathBuf::from("~"));
            assert_eq!(home_relative(Path::new("/Users/ann/Projects/app"), home), PathBuf::from("~/Projects/app"));
        }

        #[test]
        fn the_own_data_is_measured_against_its_budget() {
            let shell = Shell::new();

            let own = get_own_data(shell.data_folder()).unwrap();

            assert_eq!(own.budget, fresher_store::FOOTPRINT_BUDGET);
            assert!(own.total <= own.budget);
        }
    }
}
