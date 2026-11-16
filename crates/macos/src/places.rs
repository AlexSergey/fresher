//! The places in a home folder whose opening may raise a system prompt.

use std::path::{Path, PathBuf};

/// Relative to the home folder, one name per item (01/03, until plan 006's by-hand run says otherwise): the folders
/// behind "Files and Folders", the other apps' data, the cloud folders, and the calendar and address book data.
const PLACES: [&[&str]; 12] = [
    &["Desktop"],
    &["Documents"],
    &["Downloads"],
    &["Pictures"],
    &["Movies"],
    &["Music"],
    &["Library", "Containers"],
    &["Library", "Group Containers"],
    &["Library", "Mobile Documents"],
    &["Library", "CloudStorage"],
    &["Library", "Calendars"],
    &["Library", "Application Support", "AddressBook"],
];

/// The places under `home` the walk must not open while their permission is not settled: the stand-in for plan 006's
/// capability state, passed as the walk's avoid list. Pure: it builds paths and opens nothing.
pub fn places_that_prompt(home: &Path) -> Vec<PathBuf> {
    PLACES.iter().map(|names| names.iter().fold(home.to_path_buf(), |path, name| path.join(name))).collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::places_that_prompt;

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
    }

    mod positive {
        use std::path::Path;

        use crate::places_that_prompt;

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
    }
}
