//! The places under a home folder the system's privacy protection guards, each with what guards it.

use std::path::{Path, PathBuf};

use super::under;

/// What guards a place: the switch of the system's privacy settings that opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Guard {
    /// The Desktop folder (Files and Folders).
    Desktop,
    /// The Documents folder (Files and Folders).
    Documents,
    /// The Downloads folder (Files and Folders).
    Downloads,
    /// The data of other apps: their containers and group containers.
    OtherAppsData,
    /// Only Full Disk Access opens it; never tried, opened only with it.
    FullDiskAccess,
    /// A removable volume (Files and Folders → Removable Volumes).
    RemovableVolume,
    /// A network volume (Files and Folders → Network Volumes).
    NetworkVolume,
}

/// Relative to the home folder, one name per item, with its guard: the design's "Probe results", "Places", from
/// documentation and unverified until the user's run of plan 006/01, which may change this table and only it.
const PLACES: [(&[&str], Guard); 12] = [
    (&["Desktop"], Guard::Desktop),
    (&["Documents"], Guard::Documents),
    (&["Downloads"], Guard::Downloads),
    (&["Library", "Containers"], Guard::OtherAppsData),
    (&["Library", "Group Containers"], Guard::OtherAppsData),
    (&["Pictures"], Guard::FullDiskAccess),
    (&["Movies"], Guard::FullDiskAccess),
    (&["Music"], Guard::FullDiskAccess),
    (&["Library", "Mobile Documents"], Guard::FullDiskAccess),
    (&["Library", "CloudStorage"], Guard::FullDiskAccess),
    (&["Library", "Calendars"], Guard::FullDiskAccess),
    (&["Library", "Application Support", "AddressBook"], Guard::FullDiskAccess),
];

/// The guarded places under `home`, each with its guard. Pure: it builds paths and opens nothing.
pub fn guarded_places(home: &Path) -> Vec<(PathBuf, Guard)> {
    PLACES.iter().map(|(names, guard)| (under(home, names), *guard)).collect()
}

/// The places under `home` the walk must not open while their permission is not settled: the paths of
/// [`guarded_places`], passed as the walk's avoid list. Pure: it builds paths and opens nothing.
pub fn places_that_prompt(home: &Path) -> Vec<PathBuf> {
    guarded_places(home).into_iter().map(|(path, _)| path).collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::{guarded_places, places_that_prompt};

        #[test]
        fn places_that_prompt_never_contains_the_home_itself() {
            let home = Path::new("/Users/someone");

            let places = places_that_prompt(home);

            assert!(places.iter().all(|place| place != home && place.starts_with(home)));
        }

        #[test]
        fn places_that_prompt_opens_nothing() {
            let home = Path::new("/nowhere/home/that/does/not/exist");

            let places = places_that_prompt(home);

            assert_eq!(places.len(), 12);
            assert!(!home.exists());
        }

        #[test]
        fn guarded_places_never_leave_the_home_folder() {
            let home = Path::new("/Users/someone");

            let places = guarded_places(home);

            assert!(places.iter().all(|(place, _)| place != home && place.starts_with(home)));
            assert!(places.iter().all(|(place, _)| place.components().all(|part| part.as_os_str() != "..")));
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use crate::{Guard, guarded_places, places_that_prompt};

        #[test]
        fn the_places_are_absolute_under_the_given_home() {
            let places = places_that_prompt(Path::new("/Users/someone"));

            assert!(places.contains(&Path::new("/Users/someone/Documents").to_path_buf()));
            assert!(places.contains(&Path::new("/Users/someone/Library/Group Containers").to_path_buf()));
            assert!(
                places.contains(&Path::new("/Users/someone/Library/Application Support/AddressBook").to_path_buf())
            );
            assert!(places.iter().all(|place| place.is_absolute()));
        }

        #[test]
        fn guarded_places_of_a_fake_home_match_the_table() {
            let home = Path::new("/Users/someone");
            let expected: Vec<(PathBuf, Guard)> = [
                ("Desktop", Guard::Desktop),
                ("Documents", Guard::Documents),
                ("Downloads", Guard::Downloads),
                ("Library/Containers", Guard::OtherAppsData),
                ("Library/Group Containers", Guard::OtherAppsData),
                ("Pictures", Guard::FullDiskAccess),
                ("Movies", Guard::FullDiskAccess),
                ("Music", Guard::FullDiskAccess),
                ("Library/Mobile Documents", Guard::FullDiskAccess),
                ("Library/CloudStorage", Guard::FullDiskAccess),
                ("Library/Calendars", Guard::FullDiskAccess),
                ("Library/Application Support/AddressBook", Guard::FullDiskAccess),
            ]
            .into_iter()
            .map(|(relative, guard)| (home.join(relative), guard))
            .collect();

            assert_eq!(guarded_places(home), expected);
        }

        #[test]
        fn places_that_prompt_are_the_guarded_places() {
            let home = Path::new("/Users/someone");

            let paths: Vec<PathBuf> = guarded_places(home).into_iter().map(|(path, _)| path).collect();

            assert_eq!(places_that_prompt(home), paths);
        }
    }
}
