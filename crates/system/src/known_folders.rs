//! `MacKnownFolders`: where macOS keeps each known folder.

use std::path::PathBuf;

use fresher_core::{KnownFolder, KnownFolders};

/// The temporary folder every user shares, canonical (`/tmp` is a link to it).
const SHARED_TEMP: &str = "/private/tmp";

/// The known folders of macOS for the user whose home folder it is built with. Pure: it builds paths and opens
/// nothing, so a test gives it a fixture home, and the user's temporary folder is read by the caller.
#[derive(Clone, Debug)]
pub struct MacKnownFolders {
    home: Option<PathBuf>,
    user_temp: Option<PathBuf>,
}

impl MacKnownFolders {
    /// The folders of `home`; a home that is not an absolute path gives no folder of the user.
    pub fn new(home: impl Into<PathBuf>) -> Self {
        let home = home.into();
        Self { home: home.is_absolute().then_some(home), user_temp: None }
    }

    /// The same folders, with the user's own temporary folder at `path`, which the caller made canonical; a path
    /// that is not absolute gives none.
    #[must_use]
    pub fn with_user_temp(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        self.user_temp = path.is_absolute().then_some(path);
        self
    }

    fn under_home(&self, names: &[&str]) -> Option<PathBuf> {
        self.home.as_ref().map(|home| names.iter().fold(home.clone(), |path, name| path.join(name)))
    }
}

impl KnownFolders for MacKnownFolders {
    fn locate(&self, folder: KnownFolder) -> Option<PathBuf> {
        match folder {
            KnownFolder::Home => self.under_home(&[]),
            KnownFolder::Library => self.under_home(&["Library"]),
            KnownFolder::AppCaches => self.under_home(&["Library", "Caches"]),
            KnownFolder::AppSupport => self.under_home(&["Library", "Application Support"]),
            KnownFolder::Preferences => self.under_home(&["Library", "Preferences"]),
            KnownFolder::Logs => self.under_home(&["Library", "Logs"]),
            KnownFolder::Containers => self.under_home(&["Library", "Containers"]),
            KnownFolder::GroupContainers => self.under_home(&["Library", "Group Containers"]),
            KnownFolder::SavedState => self.under_home(&["Library", "Saved Application State"]),
            KnownFolder::Trash => self.under_home(&[".Trash"]),
            KnownFolder::Downloads => self.under_home(&["Downloads"]),
            KnownFolder::UserApplications => self.under_home(&["Applications"]),
            KnownFolder::Applications => Some(PathBuf::from("/Applications")),
            KnownFolder::SystemApplications => Some(PathBuf::from("/System/Applications")),
            KnownFolder::UserTemp => self.user_temp.clone(),
            KnownFolder::SharedTemp => Some(PathBuf::from(SHARED_TEMP)),
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{KnownFolder, KnownFolders};

        use crate::MacKnownFolders;

        fn of_all_users(folder: KnownFolder) -> bool {
            matches!(folder, KnownFolder::Applications | KnownFolder::SystemApplications | KnownFolder::SharedTemp)
        }

        #[test]
        fn without_a_user_temp_there_is_no_user_temp() {
            let folders = MacKnownFolders::new("/Users/someone");

            assert_eq!(folders.locate(KnownFolder::UserTemp), None);
            assert_eq!(folders.with_user_temp("relative/T").locate(KnownFolder::UserTemp), None);
        }

        #[test]
        fn a_relative_home_gives_no_folder_of_the_user() {
            let folders = MacKnownFolders::new("relative/home");

            for folder in KnownFolder::ALL {
                assert_eq!(folders.locate(folder).is_some(), of_all_users(folder), "{folder}");
            }
        }

        #[test]
        fn no_folder_but_those_of_all_users_and_the_temporary_one_is_outside_the_home() {
            let folders = MacKnownFolders::new("/Users/someone").with_user_temp("/private/var/folders/xx/abc/T");

            for folder in KnownFolder::ALL {
                let outside = !folders.locate(folder).unwrap().starts_with("/Users/someone");
                assert_eq!(outside, of_all_users(folder) || folder == KnownFolder::UserTemp, "{folder}");
            }
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{KnownFolder, KnownFolders};

        use crate::MacKnownFolders;

        #[test]
        fn every_known_folder_has_its_macos_place() {
            let folders = MacKnownFolders::new("/Users/someone");
            let at = |folder| folders.locate(folder).unwrap();

            assert_eq!(at(KnownFolder::Home), PathBuf::from("/Users/someone"));
            assert_eq!(at(KnownFolder::AppCaches), PathBuf::from("/Users/someone/Library/Caches"));
            assert_eq!(at(KnownFolder::AppSupport), PathBuf::from("/Users/someone/Library/Application Support"));
            assert_eq!(at(KnownFolder::SavedState), PathBuf::from("/Users/someone/Library/Saved Application State"));
            assert_eq!(at(KnownFolder::Trash), PathBuf::from("/Users/someone/.Trash"));
            assert_eq!(at(KnownFolder::SystemApplications), PathBuf::from("/System/Applications"));
            assert_eq!(at(KnownFolder::SharedTemp), PathBuf::from("/private/tmp"));
        }

        #[test]
        fn the_user_temp_is_the_given_path() {
            let folders = MacKnownFolders::new("/Users/someone").with_user_temp("/private/var/folders/xx/abc/T");

            assert_eq!(folders.locate(KnownFolder::UserTemp), Some(PathBuf::from("/private/var/folders/xx/abc/T")));
            assert_eq!(folders.locate(KnownFolder::SharedTemp), Some(PathBuf::from("/private/tmp")));
        }
    }
}
