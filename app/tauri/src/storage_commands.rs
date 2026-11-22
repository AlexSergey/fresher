//! The storage map's commands: each one asks the engine for one view of the map and returns it. They take ids, never
//! paths.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use fresher_core::{MeaningGroup, Refusal, StorageMap};
use fresher_engine::Engine;
use tauri::State;

/// The map of every volume that holds a scan root: parts that add up to each volume's capacity.
#[tauri::command(async)]
pub(crate) fn get_storage_map(engine: State<'_, Engine>) -> Result<StorageMap, Refusal> {
    engine.storage_map().map_err(Refusal::from)
}

/// The by-meaning groups of volume `volume` (its index in the map).
#[tauri::command(async)]
pub(crate) fn get_storage_meaning(engine: State<'_, Engine>, volume: u32) -> Result<Vec<MeaningGroup>, Refusal> {
    let volume = usize::try_from(volume).unwrap_or(usize::MAX);
    engine.storage_meaning(volume).map_err(Refusal::from)
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{CommandError, Refusal};

        use crate::storage_commands::get_storage_meaning;
        use crate::testing::{Shell, scan};

        #[test]
        fn an_unknown_volume_is_refused() {
            let shell = Shell::new();
            scan(&shell);

            assert_eq!(get_storage_meaning(shell.engine(), 99), Err(Refusal::from(CommandError::UnknownVolume)));
        }

        #[test]
        fn the_meaning_before_any_scan_is_refused_as_no_scan() {
            let shell = Shell::new();

            assert_eq!(get_storage_meaning(shell.engine(), 0), Err(Refusal::from(CommandError::NoScan)));
        }
    }

    mod positive {
        use fresher_core::MapPhase;

        use crate::storage_commands::get_storage_map;
        use crate::testing::{Shell, scan, trial_home};

        #[test]
        fn the_map_before_a_scan_is_no_scan() {
            let shell = Shell::new();

            let map = get_storage_map(shell.engine()).unwrap();

            assert_eq!(map.phase, MapPhase::NoScan);
        }

        #[test]
        fn the_map_after_a_scan_adds_up() {
            let home = trial_home(true);
            let shell = Shell::cleaning(home.root());
            scan(&shell);

            let map = get_storage_map(shell.engine()).unwrap();

            assert!(matches!(map.phase, MapPhase::Complete { .. }));
            assert_eq!(map.volumes.len(), 1);
            for volume in &map.volumes {
                let sum: u64 = volume.parts().iter().map(|part| part.sizes.allocated.bytes()).sum();
                assert_eq!(sum, volume.capacity().bytes());
            }
        }
    }
}
