//! `FakeKnownFolders`: the known folders of a fixture home.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use fresher_core::{KnownFolder, KnownFolders};

/// The known folders a test sets: `home_layout` puts the user's folders where macOS keeps them under a fixture home,
/// and the two temporary folders inside it too, so a fixture tree holds them; the apps folders of every user and of
/// the system are absent until `with` sets them.
#[derive(Clone, Debug, Default)]
pub struct FakeKnownFolders {
    folders: BTreeMap<KnownFolder, PathBuf>,
}

impl FakeKnownFolders {
    /// No known folder at all.
    pub fn new() -> Self {
        Self::default()
    }

    /// The user's folders under `home`, laid out as macOS lays them out (the macOS adapter's table, repeated so this
    /// crate depends on the core alone); the temporary folders as the system's are laid out from `/`, inside `home`
    /// (`private/var/folders/xx/fixture/T`, `private/tmp`), never the real ones.
    pub fn home_layout(home: impl AsRef<Path>) -> Self {
        let home = home.as_ref();
        let under = |names: &[&str]| names.iter().fold(home.to_path_buf(), |path, name| path.join(name));
        let folders = [
            (KnownFolder::Home, under(&[])),
            (KnownFolder::Library, under(&["Library"])),
            (KnownFolder::AppCaches, under(&["Library", "Caches"])),
            (KnownFolder::AppSupport, under(&["Library", "Application Support"])),
            (KnownFolder::Preferences, under(&["Library", "Preferences"])),
            (KnownFolder::Logs, under(&["Library", "Logs"])),
            (KnownFolder::Containers, under(&["Library", "Containers"])),
            (KnownFolder::GroupContainers, under(&["Library", "Group Containers"])),
            (KnownFolder::SavedState, under(&["Library", "Saved Application State"])),
            (KnownFolder::Trash, under(&[".Trash"])),
            (KnownFolder::Downloads, under(&["Downloads"])),
            (KnownFolder::UserApplications, under(&["Applications"])),
            (KnownFolder::UserTemp, under(&["private", "var", "folders", "xx", "fixture", "T"])),
            (KnownFolder::SharedTemp, under(&["private", "tmp"])),
        ];
        Self { folders: folders.into_iter().collect() }
    }

    /// Puts `folder` at `path`, replacing where it was.
    #[must_use]
    pub fn with(mut self, folder: KnownFolder, path: impl Into<PathBuf>) -> Self {
        self.folders.insert(folder, path.into());
        self
    }
}

impl KnownFolders for FakeKnownFolders {
    fn locate(&self, folder: KnownFolder) -> Option<PathBuf> {
        self.folders.get(&folder).cloned()
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{KnownFolder, KnownFolders};

        use crate::FakeKnownFolders;

        #[test]
        fn the_apps_folders_of_all_users_are_absent_until_set() {
            let folders = FakeKnownFolders::home_layout("/home");

            assert_eq!(folders.locate(KnownFolder::Applications), None);
            assert_eq!(folders.locate(KnownFolder::SystemApplications), None);
            assert_eq!(FakeKnownFolders::new().locate(KnownFolder::Home), None);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{KnownFolder, KnownFolders};

        use crate::FakeKnownFolders;

        #[test]
        fn the_home_layout_puts_the_user_folders_under_the_home() {
            let folders = FakeKnownFolders::home_layout("/home").with(KnownFolder::Applications, "/apps");

            assert_eq!(folders.locate(KnownFolder::Home), Some(PathBuf::from("/home")));
            assert_eq!(folders.locate(KnownFolder::AppCaches), Some(PathBuf::from("/home/Library/Caches")));
            assert_eq!(folders.locate(KnownFolder::Trash), Some(PathBuf::from("/home/.Trash")));
            assert_eq!(folders.locate(KnownFolder::Applications), Some(PathBuf::from("/apps")));
        }

        #[test]
        fn the_home_layout_puts_the_temporary_folders_inside_the_fixture() {
            let folders = FakeKnownFolders::home_layout("/home");

            assert_eq!(
                folders.locate(KnownFolder::UserTemp),
                Some(PathBuf::from("/home/private/var/folders/xx/fixture/T"))
            );
            assert_eq!(folders.locate(KnownFolder::SharedTemp), Some(PathBuf::from("/home/private/tmp")));
        }
    }
}
