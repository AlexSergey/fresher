//! The Largest screen's commands: the Files list, the Folders view and a row picked by the user. They take ids, never
//! paths; a pick is judged by the engine, which alone decides what may become an item.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use fresher_core::{Finding, FoldersPage, FoldersQuery, LargestPage, LargestQuery, NodeId, OwnPlace, Refusal};
use fresher_engine::Engine;
use tauri::State;

/// One page of the home's largest items of the last scan.
#[tauri::command(async)]
pub(crate) fn get_largest(engine: State<'_, Engine>, query: LargestQuery) -> Result<LargestPage, Refusal> {
    engine.largest(&query).map_err(Refusal::from)
}

/// One page of the folder `folder` of the home (the home itself when `None`), largest first.
#[tauri::command(async)]
pub(crate) fn get_largest_place(
    engine: State<'_, Engine>,
    folder: Option<NodeId>,
    offset: u32,
    limit: u32,
) -> Result<OwnPlace, Refusal> {
    engine.largest_place(folder, offset, limit).map_err(Refusal::from)
}

/// The row `node` the user ticked, as a finding of the core's own category; refused unless the engine offers it.
#[tauri::command(async)]
pub(crate) fn pick_own_item(engine: State<'_, Engine>, node: NodeId) -> Result<Finding, Refusal> {
    engine.pick_own_item(node).map_err(Refusal::from)
}

/// One page of the big folders of the home left alone, largest first.
#[tauri::command(async)]
pub(crate) fn get_largest_folders(engine: State<'_, Engine>, query: FoldersQuery) -> Result<FoldersPage, Refusal> {
    engine.largest_folders(query).map_err(Refusal::from)
}

/// Shows the row `node` in Finder, selected.
#[tauri::command(async)]
pub(crate) fn reveal_node(engine: State<'_, Engine>, node: NodeId) -> Result<(), Refusal> {
    engine.reveal_node(node).map_err(Refusal::from)
}

#[cfg(test)]
mod tests {
    use fresher_core::{FoldersQuery, LargestQuery, Unchanged};

    const fn query() -> LargestQuery {
        LargestQuery { kind: None, place: None, unchanged: Unchanged::Any, offset: 0, limit: 100 }
    }

    fn folders_query() -> FoldersQuery {
        serde_json::from_value(serde_json::json!({ "offset": 0, "limit": 10 })).unwrap()
    }

    mod negative {
        use fresher_core::{CommandError, LargestQuery, NodeId, Offer, Refusal, StateId};

        use super::{folders_query, query};
        use crate::largest_commands::{
            get_largest, get_largest_folders, get_largest_place, pick_own_item, reveal_node,
        };
        use crate::testing::{Shell, scan, trial_home};

        #[test]
        fn no_scan_is_refused() {
            let shell = Shell::new();
            let no_scan = Refusal::from(CommandError::NoScan);

            assert_eq!(get_largest(shell.engine(), query()).unwrap_err(), no_scan);
            assert_eq!(get_largest_place(shell.engine(), None, 0, 10).unwrap_err(), no_scan);
            assert_eq!(pick_own_item(shell.engine(), NodeId::new(StateId::new(0), 0)).unwrap_err(), no_scan);
            assert_eq!(get_largest_folders(shell.engine(), folders_query()).unwrap_err(), no_scan);
            assert_eq!(reveal_node(shell.engine(), NodeId::new(StateId::new(0), 0)).unwrap_err(), no_scan);
        }

        #[test]
        fn a_plain_folder_is_not_picked() {
            let home = trial_home(true);
            let project = home.root().join("Projects");
            std::fs::create_dir(&project).unwrap();
            std::fs::write(project.join("clip.mov"), [1_u8; 4_096]).unwrap();
            let shell = Shell::cleaning(home.root());
            scan(&shell);
            let rows = get_largest_place(shell.engine(), None, 0, 10).unwrap().rows;
            let projects = rows.iter().find(|row| row.node.name == "Projects").unwrap();
            assert_eq!(projects.offer, Offer::Offered);

            let refused = pick_own_item(shell.engine(), projects.node.id);

            assert_eq!(refused.map(|_| ()), Err(Refusal::from(CommandError::NotOwnItem)));
        }

        #[test]
        fn a_stale_node_is_refused() {
            let shell = Shell::new();
            scan(&shell);
            let older = NodeId::new(StateId::new(u32::MAX), 0);
            let stale = Refusal::from(CommandError::StaleNode);
            let in_older = LargestQuery { place: Some(older), ..query() };

            assert_eq!(get_largest(shell.engine(), in_older).unwrap_err(), stale);
            assert_eq!(get_largest_place(shell.engine(), Some(older), 0, 10).unwrap_err(), stale);
            assert_eq!(pick_own_item(shell.engine(), older).unwrap_err(), stale);
            assert_eq!(reveal_node(shell.engine(), older).unwrap_err(), stale);
        }
    }

    mod positive {
        use fresher_core::{ByteSize, DangerLevel, OWN_FILE_ID, Offer};

        use super::{folders_query, query};
        use crate::largest_commands::{get_largest, get_largest_folders, get_largest_place, pick_own_item};
        use crate::testing::{Shell, scan, trial_home};

        #[test]
        fn the_files_list_answers_at_the_default_minimum() {
            let home = trial_home(true);
            let shell = Shell::cleaning(home.root());
            scan(&shell);

            let page = get_largest(shell.engine(), query()).unwrap();

            assert_eq!(page.min_size, ByteSize::new(100_000_000));
            assert_eq!(page.total, 0);
        }

        #[test]
        fn the_folders_list_answers_at_the_default_size() {
            let home = trial_home(true);
            let shell = Shell::cleaning(home.root());
            scan(&shell);

            let page = get_largest_folders(shell.engine(), folders_query()).unwrap();

            assert_eq!(page.min_size, ByteSize::new(1_000_000_000));
            assert_eq!(page.days, 365);
            assert_eq!(page.total, 0);
        }

        #[test]
        fn the_folders_view_starts_at_the_home() {
            let home = trial_home(true);
            let shell = Shell::cleaning(home.root());
            scan(&shell);

            let place = get_largest_place(shell.engine(), None, 0, 10).unwrap();

            assert_eq!(place.path.len(), 1);
            assert_eq!(place.rows.iter().map(|row| row.node.name.as_str()).collect::<Vec<_>>(), ["sandbox"]);
        }

        #[test]
        fn an_offered_package_is_picked_at_high() {
            let home = trial_home(true);
            let project = home.root().join("Projects").join("Clip.photoslibrary");
            std::fs::create_dir_all(&project).unwrap();
            std::fs::write(project.join("clip.mov"), [1_u8; 4_096]).unwrap();
            let shell = Shell::cleaning(home.root());
            scan(&shell);
            let rows = get_largest_place(shell.engine(), None, 0, 10).unwrap().rows;
            let projects = rows.iter().find(|row| row.node.name == "Projects").unwrap();
            let inside = get_largest_place(shell.engine(), Some(projects.node.id), 0, 10).unwrap().rows;
            let package = inside.iter().find(|row| row.node.name == "Clip.photoslibrary").unwrap();
            assert_eq!(package.offer, Offer::Offered);

            let finding = pick_own_item(shell.engine(), package.node.id).unwrap();

            assert_eq!(finding.finding().category().as_str(), OWN_FILE_ID);
            assert_eq!(finding.finding().danger(), DangerLevel::High);
        }
    }
}
