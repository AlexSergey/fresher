//! The places of a Mac the cleaner keeps away from, for one home folder: the data of the core's `Protection`. The
//! checks themselves are the core's (by path components); this file only names places.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use fresher_core::{Protection, ProtectionError};

use crate::ffi::volume;
use crate::volume::c_string;

/// The system's own places: what macOS needs to run. Nothing in them is ever the user's to clean.
const SYSTEM: [&str; 9] = [
    "/System",
    "/usr",
    "/bin",
    "/sbin",
    "/private/var/db",
    "/private/etc",
    "/cores",
    "/Library/Apple",
    // Apps are removed only by an uninstaller, which is not planned yet.
    "/Applications",
];
/// Carved out of `/usr`: what the user installed there may go, item by item.
const USR_LOCAL: &str = "/usr/local";
/// The folder of every home; the user's own and the shared one are carved out of it, so every other home is never.
const USERS: &str = "/Users";
const SHARED: &str = "/Users/Shared";
/// The folder volumes mount in.
const VOLUMES: &str = "/Volumes";
/// A volume's Trash folder, at its root: what the user put there is theirs.
const TRASHES: &str = ".Trashes";

/// Relative to the home: the user's Trash, and the synced places whose removal also removes the copy in the cloud.
const HOME_NEVER: [&[&str]; 3] = [&[".Trash"], &["Library", "Mobile Documents"], &["Library", "CloudStorage"]];
/// Relative to the home: the user's own folders and the Library's standard ones, never removed as a whole.
const HOME_WHOLE: [&[&str]; 23] = [
    &["Desktop"],
    &["Documents"],
    &["Downloads"],
    &["Movies"],
    &["Music"],
    &["Pictures"],
    &["Public"],
    &["Applications"],
    &["Library"],
    &["Library", "Caches"],
    &["Library", "Application Support"],
    &["Library", "Preferences"],
    &["Library", "Containers"],
    &["Library", "Group Containers"],
    &["Library", "Logs"],
    &["Library", "Saved Application State"],
    &["Library", "Cookies"],
    &["Library", "HTTPStorages"],
    &["Library", "WebKit"],
    &["Library", "LaunchAgents"],
    &["Library", "Mail"],
    &["Library", "Messages"],
    &["Library", "Keychains"],
];

/// The protection of this Mac for the user whose home is `home`, with the app's own data folder and, in a built app,
/// its bundle. The given paths are made canonical (`/private/var`, not `/var`); the mounted volumes are read now.
pub fn protection(home: &Path, data_folder: &Path, app_bundle: Option<&Path>) -> Result<Protection, ProtectionError> {
    let places =
        Places::of(&canonical(home), &canonical(data_folder), app_bundle.map(canonical).as_deref(), &mounted_volumes());
    Protection::new(places.never, places.whole, places.except)
}

/// The three lists of the core's `Protection`.
#[derive(Debug)]
struct Places {
    never: Vec<PathBuf>,
    whole: Vec<PathBuf>,
    except: Vec<PathBuf>,
}

impl Places {
    /// Pure: builds paths from the given ones and reads nothing.
    fn of(home: &Path, data_folder: &Path, app_bundle: Option<&Path>, volumes: &[PathBuf]) -> Self {
        let under_home = |names: &&[&str]| names.iter().fold(home.to_path_buf(), |path, name| path.join(name));
        let users = Path::new(USERS);

        let mut never: Vec<PathBuf> = SYSTEM.iter().map(PathBuf::from).collect();
        never.push(users.to_path_buf());
        never.extend(HOME_NEVER.iter().map(under_home));
        never.push(data_folder.to_path_buf());
        never.extend(app_bundle.map(Path::to_path_buf));
        never.push(Path::new("/").join(TRASHES));
        never.extend(volumes.iter().map(|volume| volume.join(TRASHES)));

        let mut except = vec![PathBuf::from(USR_LOCAL), PathBuf::from(SHARED)];
        // The user's own home is carved out of `/Users`, when it is there (a home elsewhere needs no carving).
        if home.starts_with(users) && home != users {
            except.push(home.to_path_buf());
        }

        let mut whole = vec![home.to_path_buf(), PathBuf::from(VOLUMES)];
        whole.extend(HOME_WHOLE.iter().map(under_home));
        whole.extend(volumes.iter().cloned());

        Self { never, whole, except }
    }
}

/// The mount points of the mounted volumes, as the system names them (canonical). None when they cannot be read:
/// each volume's own Trash then rests on the guard's same-volume check.
fn mounted_volumes() -> Vec<PathBuf> {
    match volume::mounts() {
        Ok(mounts) => mounts
            .iter()
            .filter_map(|stats| c_string(&stats.f_mntonname))
            .map(|point| PathBuf::from(OsStr::from_bytes(point.as_bytes())))
            .collect(),
        Err(error) => {
            tracing::warn!(%error, "the mounted volumes are unknown");
            Vec::new()
        }
    }
}

/// `path` with its links resolved; a part that does not exist yet (a data folder not made) is kept as given after
/// its longest existing ancestor. A path that cannot be resolved is kept as given, and the core refuses it if it is
/// not plain.
fn canonical(path: &Path) -> PathBuf {
    let mut missing = Vec::new();
    let mut existing = path;
    loop {
        if let Ok(found) = fs::canonicalize(existing) {
            return missing.iter().rev().fold(found, |path, name| path.join(name));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                missing.push(name);
                existing = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use fresher_core::Protection;
    use tempfile::TempDir;

    use crate::protection;

    /// A temporary folder standing in for the home, and its canonical path, which the checks use.
    fn home() -> (TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let home = fs::canonicalize(folder.path()).unwrap();
        (folder, home)
    }

    /// The protection for the temporary home given as the system names it (through the link `/var`), with a data
    /// folder that does not exist yet and an app bundle.
    fn protect(folder: &TempDir) -> Protection {
        let data = folder.path().join("Library").join("Application Support").join("Fresher");
        protection(folder.path(), &data, Some(Path::new("/Applications/Fresher.app"))).unwrap()
    }

    mod negative {
        use std::path::Path;

        use fresher_core::Protected;

        use super::{home, protect};
        use crate::protection;
        use crate::protection::Places;

        #[test]
        fn the_home_and_its_standard_folders_are_whole() {
            let (folder, home) = home();
            let protection = protect(&folder);

            // The home holds `never` places (its Trash, iCloud Drive), so it is refused as their ancestor first.
            assert_eq!(protection.check(&home), Err(Protected::Ancestor));
            for folder in ["Desktop", "Documents", "Downloads", "Movies", "Music", "Pictures", "Public"] {
                assert_eq!(protection.check(&home.join(folder)), Err(Protected::WholeFolder), "{folder}");
            }
            for folder in ["Caches", "Preferences", "Containers", "Group Containers", "Logs", "Keychains"] {
                let path = home.join("Library").join(folder);
                assert_eq!(protection.check(&path), Err(Protected::WholeFolder), "{folder}");
            }
            // These hold `never` places (iCloud Drive; the app's own data folder).
            assert_eq!(protection.check(&home.join("Library")), Err(Protected::Ancestor));
            assert_eq!(protection.check(&home.join("Library/Application Support")), Err(Protected::Ancestor));
        }

        #[test]
        fn icloud_and_cloud_storage_are_never() {
            let (folder, home) = home();
            let protection = protect(&folder);

            for place in ["Library/Mobile Documents", "Library/Mobile Documents/com~apple~Notes/a.txt"] {
                assert_eq!(protection.check(&home.join(place)), Err(Protected::Never), "{place}");
            }
            for place in ["Library/CloudStorage", "Library/CloudStorage/Drive-ann/big.mov"] {
                assert_eq!(protection.check(&home.join(place)), Err(Protected::Never), "{place}");
            }
        }

        #[test]
        fn the_trash_is_never() {
            let (folder, home) = home();
            let protection = protect(&folder);

            assert_eq!(protection.check(&home.join(".Trash")), Err(Protected::Never));
            assert_eq!(protection.check(&home.join(".Trash/old.zip")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/.Trashes/501/old.zip")), Err(Protected::Never));
        }

        #[test]
        fn every_mounted_volume_is_whole_and_its_trash_never() {
            let volume = Path::new("/Volumes/Backup");
            let places = Places::of(Path::new("/Users/ann"), Path::new("/data"), None, &[volume.to_path_buf()]);
            let protection = fresher_core::Protection::new(places.never, places.whole, places.except).unwrap();

            assert_eq!(protection.check(volume), Err(Protected::Ancestor));
            assert_eq!(protection.check(&volume.join(".Trashes/501/a.txt")), Err(Protected::Never));
            assert_eq!(protection.check(&volume.join("old builds")), Ok(()));
        }

        #[test]
        fn the_apps_own_data_folder_is_never() {
            let (folder, home) = home();
            let data = home.join("Library/Application Support/Fresher");
            let protection = protect(&folder);

            assert_eq!(protection.check(&data), Err(Protected::Never));
            assert_eq!(protection.check(&data.join("cleanups.jsonl")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Applications/Fresher.app/Contents")), Err(Protected::Never));
        }

        #[test]
        fn another_users_home_is_never() {
            let protection = protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None).unwrap();

            assert_eq!(protection.check(Path::new("/Users/bob")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Users/bob/Library/Caches/x")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Users/annie/Library/Caches/x")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Users/a-file-at-the-top")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Users")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Users/Shared")), Err(Protected::WholeFolder));
        }

        #[test]
        fn usr_is_never_but_usr_local() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            for place in ["/usr", "/usr/bin/ls", "/usr/lib", "/System/Library", "/bin/sh", "/private/etc/hosts"] {
                assert_eq!(protection.check(Path::new(place)), Err(Protected::Never), "{place}");
            }
            assert_eq!(protection.check(Path::new("/usr/local")), Err(Protected::WholeFolder));
            assert_eq!(protection.check(Path::new("/private/var/db/receipts")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Applications/Notes.app")), Err(Protected::Never));
        }

        #[test]
        fn a_path_that_cannot_be_made_canonical_is_refused() {
            let made = protection(Path::new("relative/home"), Path::new("/data"), None);

            assert!(made.is_err());
        }
    }

    mod positive {
        use std::path::{Component, Path};

        use super::{home, protect};
        use crate::protection;
        use crate::protection::{Places, canonical, mounted_volumes};

        #[test]
        fn an_item_inside_the_caches_folder_may_go() {
            let (folder, home) = home();
            let protection = protect(&folder);

            assert_eq!(protection.check(&home.join("Library/Caches/com.example.app")), Ok(()));
            assert_eq!(protection.check(&home.join("Downloads/installer.dmg")), Ok(()));
        }

        #[test]
        fn an_item_inside_usr_local_may_go() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            assert_eq!(protection.check(Path::new("/usr/local/Cellar/old-tool")), Ok(()));
        }

        #[test]
        fn the_users_own_home_and_shared_items_may_go() {
            let protection = protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None).unwrap();

            assert_eq!(protection.check(Path::new("/Users/ann/Library/Caches/x")), Ok(()));
            assert_eq!(protection.check(Path::new("/Users/Shared/old installer.pkg")), Ok(()));
        }

        #[test]
        fn every_path_of_the_list_is_canonical() {
            let folder = tempfile::tempdir().unwrap();
            let tmp = std::env::temp_dir();
            let places = Places::of(
                &canonical(folder.path()),
                &canonical(&tmp.join("not made yet")),
                Some(&canonical(&tmp)),
                &mounted_volumes(),
            );
            let links = ["var", "tmp", "etc"];

            let all = places.never.iter().chain(&places.whole).chain(&places.except);

            for path in all {
                let first = path.components().nth(1);
                let through_a_link = links.iter().any(|link| first == Some(Component::Normal(link.as_ref())));
                assert!(!through_a_link, "{}", path.display());
                assert!(path.is_absolute(), "{}", path.display());
            }
            assert!(tmp.starts_with("/var"), "the test needs a temporary folder behind a link");
        }
    }
}
