//! Which folders are this user's Trash: the home's, the ones the composition names, a volume's `.Trashes/<uid>` at
//! the root of that volume. The Trash view counts only what lies in one, and the history trusts a journal entry only
//! when it landed in one: the journal file is plain text any program of the user can write.

use std::path::{Component, Path, PathBuf};

use fresher_core::{KnownFolder, Volumes};

use crate::engine::{CleanupPorts, Shared};

/// The folder at a volume's root that holds each user's Trash of that volume, by user id.
const VOLUME_TRASHES: &str = ".Trashes";

/// The Trash folders known so far, and every folder already asked about.
#[derive(Debug, Default)]
pub(crate) struct TrashFolders {
    /// The home's Trash, as the known folders spell it.
    home: Option<PathBuf>,
    /// The other Trash folders the composition names.
    named: Vec<PathBuf>,
    /// The folders asked about, and whether each is a Trash.
    checked: Vec<(PathBuf, bool)>,
}

impl TrashFolders {
    pub(crate) fn of(shared: &Shared, ports: &CleanupPorts) -> Self {
        Self::new(shared.ports.system.known_folders.locate(KnownFolder::Trash), ports.trashes.clone())
    }

    pub(crate) fn new(home: Option<PathBuf>, named: Vec<PathBuf>) -> Self {
        Self { home, named, checked: Vec::new() }
    }

    /// Whether `folder` is a Trash: the home's, one the composition names, or a volume's `.Trashes/<uid>` at the root
    /// of that volume.
    pub(crate) fn is_trash(&mut self, volumes: &dyn Volumes, folder: &Path) -> bool {
        if self.home.as_deref() == Some(folder) || self.named.iter().any(|trash| trash == folder) {
            return true;
        }
        if let Some(&(_, known)) = self.checked.iter().find(|(path, _)| path == folder) {
            return known;
        }
        let uid = folder.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.parse::<u32>().is_ok());
        let trash = uid
            && volumes
                .space(folder)
                .is_ok_and(|space| folder.parent() == Some(space.mount_point.join(VOLUME_TRASHES).as_path()));
        self.checked.push((folder.to_path_buf(), trash));
        trash
    }

    /// Whether `landed` is an entry right inside a Trash: an absolute path of plain names only (no `.` nor `..`),
    /// whose folder [`Self::is_trash`].
    pub(crate) fn holds(&mut self, volumes: &dyn Volumes, landed: &Path) -> bool {
        let mut components = landed.components();
        let plain = components.next() == Some(Component::RootDir)
            && components.all(|component| matches!(component, Component::Normal(_)));
        plain && landed.parent().is_some_and(|folder| folder != Path::new("/") && self.is_trash(volumes, folder))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{ByteSize, DiskSpace};
    use fresher_testing::FakeVolumes;

    use super::TrashFolders;

    fn volumes() -> FakeVolumes {
        let disk = |name: &str| FakeVolumes::disk(name, ByteSize::new(1_000), ByteSize::new(400), ByteSize::new(600));
        let backup = DiskSpace { mount_point: PathBuf::from("/Volumes/Backup"), ..disk("Backup") };
        FakeVolumes::default().with_space("/", disk("Macintosh HD")).with_space("/Volumes/Backup", backup)
    }

    fn folders() -> TrashFolders {
        TrashFolders::new(Some(PathBuf::from("/home/.Trash")), vec![PathBuf::from("/other/.Trash")])
    }

    mod negative {
        use std::path::Path;

        use super::{folders, volumes};

        #[test]
        fn an_entry_outside_every_trash_is_not_held() {
            let mut folders = folders();

            for landed in ["/home/Documents/secret", "/home/.Trash", "/secret", "/", "relative/.Trash/a"] {
                assert!(!folders.holds(&volumes(), Path::new(landed)), "{landed}");
            }
        }

        #[test]
        fn an_entry_below_a_folder_of_a_trash_is_not_held() {
            assert!(!folders().holds(&volumes(), Path::new("/home/.Trash/folder/a")));
        }

        #[test]
        fn an_entry_through_a_dotted_name_is_not_held() {
            let mut folders = folders();

            for landed in ["/home/.Trash/../Documents", "/home/Documents/../.Trash/a"] {
                assert!(!folders.holds(&volumes(), Path::new(landed)), "{landed}");
            }
        }

        #[test]
        fn a_folder_named_like_a_trash_inside_a_volume_is_no_trash() {
            let mut folders = folders();

            for landed in ["/home/app/.Trash/a", "/Volumes/Backup/app/.Trashes/501/a", "/Volumes/Backup/.Trashes/me/a"]
            {
                assert!(!folders.holds(&volumes(), Path::new(landed)), "{landed}");
            }
        }
    }

    mod positive {
        use std::path::Path;

        use super::{folders, volumes};

        #[test]
        fn an_entry_of_the_home_trash_is_held() {
            assert!(folders().holds(&volumes(), Path::new("/home/.Trash/a")));
        }

        #[test]
        fn an_entry_of_a_named_trash_is_held() {
            assert!(folders().holds(&volumes(), Path::new("/other/.Trash/a")));
        }

        #[test]
        fn an_entry_of_a_volume_trash_is_held() {
            let mut folders = folders();

            assert!(folders.holds(&volumes(), Path::new("/Volumes/Backup/.Trashes/501/a")));
            assert!(folders.is_trash(&volumes(), Path::new("/Volumes/Backup/.Trashes/501")));
        }
    }
}
