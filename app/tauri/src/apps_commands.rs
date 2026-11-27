//! The Apps screen's commands: the list, an app's page, Show in Finder and the uninstall plan. They take ids, never
//! paths: an app's id names a bundle the engine listed itself.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use fresher_core::{AppDetail, AppId, AppsQuery, AppsView, CleanupPlanView, Refusal, UninstallChoice};
use fresher_engine::Engine;
use tauri::State;

/// The installed apps of the last scan, largest first, filtered by `query`.
#[tauri::command(async)]
pub(crate) fn get_apps(engine: State<'_, Engine>, query: AppsQuery) -> Result<AppsView, Refusal> {
    engine.apps(query).map_err(Refusal::from)
}

/// The page of app `app`: its files, the uninstall choices and its categories on Scan.
#[tauri::command(async)]
pub(crate) fn get_app(engine: State<'_, Engine>, app: AppId) -> Result<AppDetail, Refusal> {
    engine.app(app).map_err(Refusal::from)
}

/// Shows app `app`'s bundle in Finder, selected.
#[tauri::command(async)]
pub(crate) fn reveal_app(engine: State<'_, Engine>, app: AppId) -> Result<(), Refusal> {
    engine.reveal_app(app).map_err(Refusal::from)
}

/// The uninstall of app `app` with `choice`, made the pending plan; confirmed with `confirm_cleanup`.
#[tauri::command(async)]
pub(crate) fn prepare_uninstall(
    engine: State<'_, Engine>,
    app: AppId,
    choice: UninstallChoice,
) -> Result<CleanupPlanView, Refusal> {
    engine.prepare_uninstall(app, choice).map_err(Refusal::from)
}

#[cfg(test)]
mod tests {
    use fresher_core::{AppId, StateId};

    const APP: AppId = AppId::new(StateId::new(1), 0);

    mod negative {
        use std::path::Path;

        use fresher_core::{AppsQuery, CommandError, Refusal, SkipReason, UninstallChoice};
        use serde_json::json;

        use super::APP;
        use crate::apps_commands::{get_app, get_apps, prepare_uninstall, reveal_app};
        use crate::largest_commands::{get_largest_place, pick_own_item};
        use crate::testing::{Shell, scan, trial_home};

        #[test]
        fn the_apps_before_any_scan_are_refused_as_no_scan() {
            let shell = Shell::new();

            assert_eq!(get_apps(shell.engine(), AppsQuery::default()), Err(Refusal::from(CommandError::NoScan)));
            assert_eq!(get_app(shell.engine(), APP), Err(Refusal::from(CommandError::NoScan)));
            assert_eq!(reveal_app(shell.engine(), APP), Err(Refusal::from(CommandError::NoScan)));
            let refused = prepare_uninstall(shell.engine(), APP, UninstallChoice::KeepData);
            assert_eq!(refused, Err(Refusal::from(CommandError::NoScan)));
        }

        #[test]
        fn an_app_the_list_does_not_hold_is_refused() {
            let shell = Shell::new();
            scan(&shell);
            let state = shell.engine().disk_state().unwrap().unwrap().id();
            let app = fresher_core::AppId::new(state, 0);

            assert_eq!(get_app(shell.engine(), app), Err(Refusal::from(CommandError::UnknownApp { id: app })));
            assert_eq!(reveal_app(shell.engine(), app), Err(Refusal::from(CommandError::UnknownApp { id: app })));
            let refused = prepare_uninstall(shell.engine(), app, UninstallChoice::Everything);
            assert_eq!(refused, Err(Refusal::from(CommandError::UnknownApp { id: app })));
        }

        /// The second wall: the composition's cleaner, given an ordinary plan that holds a package in the home's
        /// apps folder (a pick of the Largest screen, which never offers a bundle itself, in a sandbox the cleaner
        /// reaches), skips it as protected and leaves it in place.
        #[test]
        fn an_ordinary_cleanup_never_moves_anything_in_an_apps_folder() {
            let home = trial_home(true);
            // The home's apps folder is a standard folder: a marker counts only in a folder inside it.
            home.mark_sandbox(Path::new("Applications/Installers")).unwrap();
            let old = home.root().join("Applications").join("Installers").join("Old.utm");
            std::fs::create_dir(&old).unwrap();
            let file = old.join("Editor.dmg");
            std::fs::write(&file, [1_u8; 4_096]).unwrap();
            let shell = Shell::cleaning(home.root());
            scan(&shell);
            let mut place = None;
            for name in ["Applications", "Installers", "Old.utm"] {
                let rows = get_largest_place(shell.engine(), place, 0, 10).unwrap().rows;
                place = Some(rows.iter().find(|row| row.node.name == name).unwrap().node.id);
            }
            let pick = pick_own_item(shell.engine(), place.unwrap()).unwrap();

            let selection = serde_json::from_value(json!({ "findings": [pick.id()], "categories": [] })).unwrap();
            let view = shell.engine().prepare_cleanup(&selection).unwrap();

            assert_eq!(view.groups, []);
            assert_eq!(view.left_out.len(), 1);
            assert_eq!(view.left_out[0].reason(), &SkipReason::Protected);
            assert!(file.is_file());
        }

        /// A plain folder of the apps folder is not a pick at all.
        #[test]
        fn a_plain_folder_in_an_apps_folder_is_not_picked() {
            let home = trial_home(true);
            home.mark_sandbox(Path::new("Applications/Installers")).unwrap();
            let old = home.root().join("Applications").join("Installers").join("Old");
            std::fs::create_dir(&old).unwrap();
            std::fs::write(old.join("Editor.dmg"), [1_u8; 4_096]).unwrap();
            let shell = Shell::cleaning(home.root());
            scan(&shell);
            let mut place = None;
            for name in ["Applications", "Installers", "Old"] {
                let rows = get_largest_place(shell.engine(), place, 0, 10).unwrap().rows;
                place = Some(rows.iter().find(|row| row.node.name == name).unwrap().node.id);
            }

            let refused = pick_own_item(shell.engine(), place.unwrap());

            assert_eq!(refused.map(|_| ()), Err(Refusal::from(CommandError::NotOwnItem)));
        }
    }

    mod positive {
        use fresher_core::{AppsQuery, AppsView};

        use crate::apps_commands::get_apps;
        use crate::testing::{Shell, scan};

        #[test]
        fn the_apps_after_a_scan_are_listed() {
            let shell = Shell::new();
            scan(&shell);

            assert_eq!(get_apps(shell.engine(), AppsQuery::default()), Ok(AppsView::listed(Vec::new())));
        }
    }
}
