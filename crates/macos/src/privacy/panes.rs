//! The System Settings panes a permission is granted in, their addresses and the fixed argument lists of
//! `/usr/bin/open`. The addresses are from documentation, unverified until the user's run of plan 006/01; they live in
//! `ADDRESSES` only. Nothing here opens anything: the caller runs the program with these arguments.

use std::ffi::OsStr;
use std::path::Path;

use super::Guard;

/// A pane of the system's privacy settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pane {
    /// Privacy & Security → Full Disk Access.
    FullDiskAccess,
    /// Privacy & Security → Files and Folders.
    FilesAndFolders,
    /// Privacy & Security, the top of the section.
    PrivacyAndSecurity,
    /// Privacy & Security → App Management: moving other developers' apps.
    AppManagement,
}

/// The program that opens an address or reveals a path.
pub const OPEN: &str = "/usr/bin/open";

/// One address per pane, the same for every supported version (macOS 13 on).
const ADDRESSES: [(Pane, &str); 4] = [
    (Pane::FullDiskAccess, "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles"),
    (Pane::FilesAndFolders, "x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders"),
    (Pane::PrivacyAndSecurity, "x-apple.systempreferences:com.apple.preference.security?Privacy"),
    (Pane::AppManagement, "x-apple.systempreferences:com.apple.preference.security?Privacy_AppBundles"),
];

/// The address of `pane`.
pub fn pane_address(pane: Pane) -> &'static str {
    ADDRESSES.iter().find(|(known, _)| *known == pane).map_or(ADDRESSES[2].1, |(_, address)| address)
}

/// The pane where the permission behind `guard` is granted. The data of other apps has no pane of its own, so it
/// gets the top of Privacy & Security.
pub fn pane_of(guard: Guard) -> Pane {
    match guard {
        Guard::FullDiskAccess => Pane::FullDiskAccess,
        Guard::Desktop | Guard::Documents | Guard::Downloads | Guard::RemovableVolume | Guard::NetworkVolume => {
            Pane::FilesAndFolders
        }
        Guard::OtherAppsData => Pane::PrivacyAndSecurity,
    }
}

/// The arguments of [`OPEN`] that open `pane`: the address alone.
pub fn open_args(pane: Pane) -> [&'static str; 1] {
    [pane_address(pane)]
}

/// The arguments of [`OPEN`] that reveal `path` in Finder: `-R` and the path, kept as one `OsStr`, never formatted.
pub fn reveal_args(path: &Path) -> [&OsStr; 2] {
    [OsStr::new("-R"), path.as_os_str()]
}

/// The arguments of [`OPEN`] that open the folder `path` in Finder: `-a Finder` and the path, kept as one `OsStr`,
/// never formatted. Finder only shows what it is given: an app or a program at the path is never launched.
pub fn open_folder_args(path: &Path) -> [&OsStr; 3] {
    [OsStr::new("-a"), OsStr::new("Finder"), path.as_os_str()]
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::ffi::OsStr;
        use std::path::Path;

        use crate::privacy::panes::ADDRESSES;
        use crate::privacy::{Guard, Pane, open_folder_args, pane_of, reveal_args};

        #[test]
        fn reveal_keeps_a_path_with_spaces_and_quotes_as_one_argument() {
            let path = Path::new("/Users/a b/it's \"x\"/file name");
            let args = reveal_args(path);
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], OsStr::new("-R"));
            assert_eq!(args[1], path.as_os_str());
        }

        #[test]
        fn open_folder_keeps_a_path_with_spaces_and_quotes_as_one_argument() {
            let path = Path::new("/Users/a b/it's \"x\"/.Trash");

            let args = open_folder_args(path);

            assert_eq!(args, [OsStr::new("-a"), OsStr::new("Finder"), path.as_os_str()]);
        }

        #[test]
        fn no_address_points_outside_system_settings() {
            for (_, address) in ADDRESSES {
                assert!(address.starts_with("x-apple.systempreferences:"), "{address}");
            }
        }

        #[test]
        fn data_from_other_apps_falls_back_to_privacy_and_security() {
            assert_eq!(pane_of(Guard::OtherAppsData), Pane::PrivacyAndSecurity);
        }
    }

    mod positive {
        use std::collections::HashSet;

        use crate::privacy::{Guard, Pane, open_args, pane_address, pane_of};

        #[test]
        fn each_pane_has_an_address() {
            let panes = [Pane::FullDiskAccess, Pane::FilesAndFolders, Pane::PrivacyAndSecurity, Pane::AppManagement];
            let addresses: HashSet<_> = panes.into_iter().map(pane_address).collect();
            assert_eq!(addresses.len(), 4);
            assert!(pane_address(Pane::FullDiskAccess).ends_with("Privacy_AllFiles"));
        }

        #[test]
        fn app_management_opens_its_own_pane() {
            assert_eq!(
                open_args(Pane::AppManagement),
                ["x-apple.systempreferences:com.apple.preference.security?Privacy_AppBundles"]
            );
        }

        #[test]
        fn open_takes_exactly_the_address() {
            assert_eq!(open_args(Pane::FilesAndFolders), [pane_address(Pane::FilesAndFolders)]);
        }

        #[test]
        fn each_guard_has_its_pane() {
            assert_eq!(pane_of(Guard::FullDiskAccess), Pane::FullDiskAccess);
            for guard in
                [Guard::Desktop, Guard::Documents, Guard::Downloads, Guard::RemovableVolume, Guard::NetworkVolume]
            {
                assert_eq!(pane_of(guard), Pane::FilesAndFolders);
            }
        }
    }
}
