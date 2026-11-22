//! The places of a Mac the cleaner keeps away from, for one home folder: the data of the core's `Protection`. The
//! checks themselves are the core's (by path components); this file only names places.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Component, Path, PathBuf};

use fresher_core::{Protection, ProtectionError};

use crate::ffi::volume;
use crate::volume::c_string;

/// The system's own places: what macOS needs to run. Nothing in them is ever the user's to clean.
const SYSTEM: [&str; 8] =
    ["/System", "/usr", "/bin", "/sbin", "/private/var/db", "/private/etc", "/cores", "/Library/Apple"];
/// The apps of every user: a whole app bundle may go, for an uninstall the user confirmed; nothing inside a bundle,
/// and not the folder.
const APP_FOLDERS: [&str; 1] = ["/Applications"];
/// The system's places of temporary and transient data: every user's caches and temporary folders (the user's own
/// `T` is carved out), the system's temporary folder, its swap and the versions of documents.
const TRANSIENT: [&str; 4] =
    ["/private/var/folders", "/private/var/tmp", "/private/var/vm", "/.DocumentRevisions-V100"];
/// The folder of every user's temporary and cache folders; the user's own `T` is carved out of it.
const USER_FOLDERS: &str = "/private/var/folders";
/// The name of a user's own temporary folder inside its triplet.
const TEMP: &str = "T";
/// Inside the user's own `T`: the system's own temporary items (a document being saved), never removed.
const TEMPORARY_ITEMS: &str = "TemporaryItems";
/// The temporary folder every user shares (`/tmp` is a link to it): what is inside may go, item by item.
const SHARED_TEMP: &str = "/private/tmp";
/// Carved out of `/usr`: what the user installed there may go, item by item.
const USR_LOCAL: &str = "/usr/local";
/// The folder of every home; the user's own and the shared one are carved out of it, so every other home is never.
const USERS: &str = "/Users";
const SHARED: &str = "/Users/Shared";
/// The folder volumes mount in.
const VOLUMES: &str = "/Volumes";
/// A volume's Trash folder, at its root: what the user put there is theirs.
const TRASHES: &str = ".Trashes";

/// Relative to the home: the user's Trash, the synced places whose removal also removes the copy in the cloud, and
/// the documents the system keeps until they are saved.
const HOME_NEVER: [&[&str]; 4] =
    [&[".Trash"], &["Library", "Mobile Documents"], &["Library", "CloudStorage"], &["Library", "Autosave Information"]];
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

/// The protection of this Mac for the user whose home is `home`, with the app's own data folder, in a built app its
/// bundle, and the user's own temporary folder when it is known (`user_temp_dir`). The given paths are made canonical
/// (`/private/var`, not `/var`); the mounted volumes are read now.
pub fn protection(
    home: &Path,
    data_folder: &Path,
    app_bundle: Option<&Path>,
    user_temp: Option<&Path>,
) -> Result<Protection, ProtectionError> {
    let places = Places::of(
        &canonical(home),
        &canonical(data_folder),
        app_bundle.map(canonical).as_deref(),
        user_temp.map(canonical).as_deref(),
        &mounted_volumes(),
    );
    places.protection()
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
    /// A `user_temp` outside `/private/var/folders` is no user's `T`: nothing is carved out for it.
    fn of(
        home: &Path,
        data_folder: &Path,
        app_bundle: Option<&Path>,
        user_temp: Option<&Path>,
        volumes: &[PathBuf],
    ) -> Self {
        let under_home = |names: &&[&str]| names.iter().fold(home.to_path_buf(), |path, name| path.join(name));
        let users = Path::new(USERS);
        let user_temp = user_temp.filter(|temp| is_user_temp(temp));

        let mut never: Vec<PathBuf> = SYSTEM.iter().chain(&APP_FOLDERS).chain(&TRANSIENT).map(PathBuf::from).collect();
        never.push(users.to_path_buf());
        never.extend(HOME_NEVER.iter().map(under_home));
        never.push(data_folder.to_path_buf());
        never.extend(app_bundle.map(Path::to_path_buf));
        never.push(Path::new("/").join(TRASHES));
        never.extend(volumes.iter().map(|volume| volume.join(TRASHES)));
        never.extend(user_temp.map(|temp| temp.join(TEMPORARY_ITEMS)));

        let mut except = vec![PathBuf::from(USR_LOCAL), PathBuf::from(SHARED)];
        // The user's own home is carved out of `/Users`, when it is there (a home elsewhere needs no carving).
        if is_strictly_inside(home, users) {
            except.push(home.to_path_buf());
        }
        except.extend(user_temp.map(Path::to_path_buf));

        let mut whole = vec![home.to_path_buf(), PathBuf::from(VOLUMES), PathBuf::from(SHARED_TEMP)];
        whole.extend(HOME_WHOLE.iter().map(under_home));
        whole.extend(volumes.iter().cloned());
        whole.extend(user_temp.map(Path::to_path_buf));

        Self { never, whole, except }
    }

    /// The core's `Protection` of these places, with the app folders opening their bundles.
    fn protection(self) -> Result<Protection, ProtectionError> {
        let app_folders: Vec<PathBuf> = APP_FOLDERS.iter().map(PathBuf::from).collect();
        Protection::new(self.never, self.whole, self.except)?.with_app_folders(&app_folders)
    }
}

/// `true` when canonical `path` has the shape of a user's own temporary folder: exactly
/// `/private/var/folders/<xx>/<hash>/T`, three names below every user's folders, the last `T`. Anything else is no
/// user's `T`: nothing is carved out for it and it is no known folder.
pub fn is_user_temp(path: &Path) -> bool {
    let Ok(below) = path.strip_prefix(USER_FOLDERS) else { return false };
    let names: Vec<Component<'_>> = below.components().collect();
    names.len() == 3
        && names.iter().all(|name| matches!(name, Component::Normal(_)))
        && names.last() == Some(&Component::Normal(TEMP.as_ref()))
}

/// `inner` is below `outer`, not `outer` itself, by components.
fn is_strictly_inside(inner: &Path, outer: &Path) -> bool {
    inner.starts_with(outer) && inner != outer
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
    /// folder that does not exist yet, an app bundle, and the temporary folder that holds the home as the user's own.
    fn protect(folder: &TempDir) -> Protection {
        let data = folder.path().join("Library").join("Application Support").join("Fresher");
        let bundle = Path::new("/Applications/Fresher.app");
        protection(folder.path(), &data, Some(bundle), Some(&std::env::temp_dir())).unwrap()
    }

    /// The user's own temporary folder of [`with_temp`], as the system names it (through the link `/var`).
    const USER_TEMP: &str = "/var/folders/xx/abc/T/";

    /// The protection of the home `/Users/ann` with [`USER_TEMP`] as the user's own temporary folder.
    fn with_temp() -> Protection {
        protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, Some(Path::new(USER_TEMP))).unwrap()
    }

    /// `path` as the cleaner checks it: the physical form, through no link (`/var` and `/tmp` resolved).
    fn physical(path: &str) -> PathBuf {
        super::canonical(Path::new(path))
    }

    mod negative {
        use std::path::Path;

        use fresher_core::{Protected, ProtectionError};

        use super::{USER_TEMP, home, physical, protect, with_temp};
        use crate::protection;
        use crate::protection::{Places, is_user_temp};

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
            let places = Places::of(Path::new("/Users/ann"), Path::new("/data"), None, None, &[volume.to_path_buf()]);
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
            assert_eq!(protection.check(Path::new("/Applications/Fresher.app")), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/Applications/Fresher.app/Contents")), Err(Protected::Never));
        }

        #[test]
        fn nothing_inside_an_app_bundle_passes() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            for path in [
                "/Applications/Notes.app/Contents",
                "/Applications/Notes.app/Contents/MacOS/Notes",
                "/Applications/Utilities/Terminal.app/Contents/Info.plist",
                "/Applications/Notes.app/Contents/Library/LoginItems/Helper.app",
                "/Applications/Notes.app/Helper.app",
            ] {
                assert_eq!(protection.check(Path::new(path)), Err(Protected::Never), "{path}");
            }
        }

        #[test]
        fn the_apps_folder_and_what_is_not_a_bundle_in_it_stay_never() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            for path in [
                "/Applications",
                "/Applications/Utilities",
                "/Applications/readme.txt",
                "/Applications/Notes.APP",
                "/Applications/Notes.app.zip",
                "/Applications/.app",
                "/Applications/a/b/C.app",
                "/Applications/./Notes.app",
                "/Applications/x/../Notes.app",
            ] {
                assert_eq!(protection.check(Path::new(path)), Err(Protected::Never), "{path}");
            }
        }

        #[test]
        fn the_system_apps_folder_stays_never() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            for path in
                ["/System/Applications", "/System/Applications/Chess.app", "/System/Applications/Utilities/A.app"]
            {
                assert_eq!(protection.check(Path::new(path)), Err(Protected::Never), "{path}");
            }
        }

        #[test]
        fn the_users_own_apps_folder_opens_no_bundle() {
            let places = Places::of(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, None, &[]);
            let closed =
                fresher_core::Protection::new(places.never.clone(), places.whole.clone(), places.except.clone())
                    .unwrap();
            let open = places.protection().unwrap();
            let user_apps = Path::new("/Users/ann/Applications");

            assert_eq!(open.check(user_apps), Err(Protected::WholeFolder));
            for path in ["A.app", "A.app/Contents/MacOS/a", "Games/B.app", "a/b/C.app"] {
                assert_eq!(open.check(&user_apps.join(path)), closed.check(&user_apps.join(path)), "{path}");
            }
            let error = open.with_app_folders(&[user_apps.to_path_buf()]);
            assert_eq!(error, Err(ProtectionError::LooseAppFolder { path: user_apps.to_path_buf() }));
        }

        #[test]
        fn another_users_home_is_never() {
            let protection = protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, None).unwrap();

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
        }

        #[test]
        fn the_system_parts_of_the_users_temporary_folders_are_never() {
            let protection = with_temp();

            for place in ["C", "0", "X", "T/TemporaryItems"] {
                for form in [format!("/var/folders/xx/abc/{place}/x"), format!("/private/var/folders/xx/abc/{place}/x")]
                {
                    assert_eq!(protection.check(&physical(&form)), Err(Protected::Never), "{form}");
                }
            }
            assert_eq!(protection.check(&physical("/var/folders/xx/abc/T/TemporaryItems")), Err(Protected::Never));
        }

        #[test]
        fn another_users_temporary_folder_is_never() {
            let protection = with_temp();

            for form in ["/var/folders/yy/other/T/x", "/private/var/folders/yy/other/T/x", "/var/folders/xx/abd/T/x"] {
                assert_eq!(protection.check(&physical(form)), Err(Protected::Never), "{form}");
            }
            assert_eq!(protection.check(&physical("/var/folders")), Err(Protected::Never));
        }

        #[test]
        fn without_a_user_temp_every_temporary_folder_is_never() {
            let protection = protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, None).unwrap();

            assert_eq!(protection.check(&physical(&format!("{USER_TEMP}x"))), Err(Protected::Never));
            assert_eq!(protection.check(Path::new("/private/tmp/x")), Ok(()));
        }

        #[test]
        fn a_user_temp_outside_the_users_folders_carves_nothing() {
            let protection =
                protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, Some(Path::new("/opt/T")))
                    .unwrap();

            assert_eq!(protection.check(Path::new("/opt/T/TemporaryItems/x")), Ok(()));
            assert_eq!(protection.check(Path::new("/opt/T")), Ok(()));
            assert_eq!(protection.check(&physical(&format!("{USER_TEMP}x"))), Err(Protected::Never));
        }

        #[test]
        fn a_path_not_shaped_as_a_users_temp_is_no_users_temp() {
            for path in [
                "/private/var/folders",
                "/private/var/folders/xx",
                "/private/var/folders/xx/abc",
                "/private/var/folders/xx/abc/C",
                "/private/var/folders/xx/abc/T/TemporaryItems",
                "/private/var/folders/xx/abc/T/deeper",
                "/private/var/folders/xx/abc/t",
                "/var/folders/xx/abc/T",
                "/private/tmp/xx/abc/T",
            ] {
                assert!(!is_user_temp(Path::new(path)), "{path}");
            }
        }

        #[test]
        fn a_user_temp_of_the_wrong_shape_carves_nothing() {
            for temp in ["/private/var/folders/xx/abc", "/private/var/folders/xx/abc/T/TemporaryItems"] {
                let places = Places::of(Path::new("/Users/ann"), Path::new("/data"), None, Some(Path::new(temp)), &[]);
                let protection = fresher_core::Protection::new(places.never, places.whole, places.except).unwrap();

                assert_eq!(protection.check(Path::new("/private/var/folders/xx/abc/T/x")), Err(Protected::Never));
                assert_eq!(protection.check(&Path::new(temp).join("x")), Err(Protected::Never), "{temp}");
            }
        }

        #[test]
        fn the_systems_transient_places_and_autosaved_documents_are_never() {
            let protection = with_temp();

            for form in [
                "/var/tmp/x",
                "/private/var/tmp/x",
                "/var/vm/swapfile0",
                "/private/var/vm",
                "/.DocumentRevisions-V100/PerUID/501/x",
                "/Users/ann/Library/Autosave Information/Untitled.rtf",
            ] {
                assert_eq!(protection.check(&physical(form)), Err(Protected::Never), "{form}");
            }
        }

        #[test]
        fn the_temporary_folders_are_refused_as_a_whole() {
            let protection = with_temp();

            // The user's `T` holds a `never` place (`TemporaryItems`), so it is refused as its ancestor first.
            for form in [USER_TEMP, "/private/var/folders/xx/abc/T"] {
                assert_eq!(protection.check(&physical(form)), Err(Protected::Ancestor), "{form}");
            }
            for form in ["/tmp", "/private/tmp"] {
                assert_eq!(protection.check(&physical(form)), Err(Protected::WholeFolder), "{form}");
            }
        }

        #[test]
        fn a_path_that_cannot_be_made_canonical_is_refused() {
            let made = protection(Path::new("relative/home"), Path::new("/data"), None, None);

            assert!(made.is_err());
        }
    }

    mod positive {
        use std::path::{Component, Path, PathBuf};

        use fresher_core::Protected;

        use super::{USER_TEMP, home, physical, protect, with_temp};
        use crate::protection;
        use crate::protection::{
            APP_FOLDERS, HOME_NEVER, HOME_WHOLE, Places, SYSTEM, TRANSIENT, canonical, is_user_temp, mounted_volumes,
        };

        #[test]
        fn a_whole_app_bundle_in_the_apps_folder_passes() {
            let folder = tempfile::tempdir().unwrap();
            let protection = protect(&folder);

            for path in ["/Applications/Notes.app", "/Applications/Utilities/Terminal.app", "/Applications/My App.app"]
            {
                assert_eq!(protection.check(Path::new(path)), Ok(()), "{path}");
            }
        }

        #[test]
        fn every_path_but_a_whole_app_bundle_answers_as_before() {
            let home = Path::new("/Users/ann");
            let volume = PathBuf::from("/Volumes/Backup");
            let places = Places::of(
                home,
                Path::new("/Users/ann/Library/Application Support/Fresher"),
                Some(Path::new("/Applications/Fresher.app")),
                Some(Path::new("/private/var/folders/xx/abc/T")),
                std::slice::from_ref(&volume),
            );
            // The list as it was: `/Applications` a plain `never` place, opening nothing.
            let before =
                fresher_core::Protection::new(places.never.clone(), places.whole.clone(), places.except.clone())
                    .unwrap();
            let under_home = |names: &&[&str]| names.iter().fold(home.to_path_buf(), |path, name| path.join(name));
            let mut places_named: Vec<PathBuf> =
                SYSTEM.iter().chain(&APP_FOLDERS).chain(&TRANSIENT).map(PathBuf::from).collect();
            places_named.extend(HOME_NEVER.iter().chain(&HOME_WHOLE).map(under_home));
            places_named.extend(places.never.iter().chain(&places.whole).chain(&places.except).cloned());
            places_named.extend(["/", "/Users/bob", "/opt/homebrew", "/System/Applications"].map(PathBuf::from));
            let tails = ["", "x", "A.app", "A.app/Contents", "Utilities/A.app", "a/b/C.app", "A.APP", ".app"];
            let opened = ["/Applications/A.app", "/Applications/Utilities/A.app"].map(PathBuf::from);

            let after = places.protection().unwrap();

            for place in &places_named {
                for tail in tails {
                    let path = if tail.is_empty() { place.clone() } else { place.join(tail) };
                    if opened.contains(&path) {
                        assert_eq!(before.check(&path), Err(Protected::Never), "{}", path.display());
                        assert_eq!(after.check(&path), Ok(()), "{}", path.display());
                    } else {
                        assert_eq!(after.check(&path), before.check(&path), "{}", path.display());
                    }
                }
            }
        }

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
            let protection = protection(Path::new("/Users/ann"), Path::new("/Users/ann/data"), None, None).unwrap();

            assert_eq!(protection.check(Path::new("/Users/ann/Library/Caches/x")), Ok(()));
            assert_eq!(protection.check(Path::new("/Users/Shared/old installer.pkg")), Ok(()));
        }

        #[test]
        fn a_users_temp_is_three_names_below_the_users_folders_ending_in_t() {
            assert!(is_user_temp(Path::new("/private/var/folders/xx/abc/T")));
            assert!(is_user_temp(Path::new("/private/var/folders/xx/abc/T/")));
        }

        #[test]
        fn an_item_in_the_users_temporary_folder_or_the_shared_one_may_go() {
            let protection = with_temp();

            for form in
                [&format!("{USER_TEMP}old.log"), "/private/var/folders/xx/abc/T/build-1", "/tmp/x", "/private/tmp/x"]
            {
                assert_eq!(protection.check(&physical(form)), Ok(()), "{form}");
            }
            assert_eq!(protection.check(&physical(&format!("{USER_TEMP}TemporaryItemsOld"))), Ok(()));
        }

        #[test]
        fn every_path_of_the_list_is_canonical() {
            let folder = tempfile::tempdir().unwrap();
            let tmp = std::env::temp_dir();
            let places = Places::of(
                &canonical(folder.path()),
                &canonical(&tmp.join("not made yet")),
                Some(&canonical(&tmp)),
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
