//! The checks of a guarded place: what an open answers, told without reading a byte.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use fresher_core::ReadError;

use super::under;
use crate::errors::read_error;

/// What an open of a guarded place answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Probe {
    /// It opened.
    Allowed,
    /// The system's privacy protection refused it (`EPERM`).
    Refused,
    /// It is not there.
    Missing,
    /// Anything else: the POSIX permissions, a link, an error the check cannot tell.
    Failed,
}

/// What an error of an open means for a check: `EPERM` is the privacy protection; `EACCES` is the POSIX permissions,
/// not privacy, so it fails like any other error.
pub fn classify(error: &io::Error) -> Probe {
    match read_error(error) {
        ReadError::Guarded => Probe::Refused,
        ReadError::Vanished => Probe::Missing,
        _ => Probe::Failed,
    }
}

/// Relative to the home folder, in order: the files only Full Disk Access opens, the design's "Probe results",
/// "Checks", from documentation and unverified until the user's run of plan 006/01.
const FDA_CANDIDATES: [&[&str]; 2] =
    [&["Library", "Application Support", "com.apple.TCC", "TCC.db"], &["Library", "Safari"]];

/// The candidates [`full_disk_access`] tries under `home`, in order. Pure: it builds paths and opens nothing.
pub fn fda_candidates(home: &Path) -> Vec<PathBuf> {
    FDA_CANDIDATES.iter().map(|names| under(home, names)).collect()
}

/// Whether the app has Full Disk Access, without a prompt: each candidate in order is opened and closed at once, not
/// a byte read; the first whose open is not `Missing` decides; every one `Missing` → `Missing`. Nothing is asked
/// before the open: inside a guarded folder `exists()` and `metadata` answer `EPERM` too.
pub fn full_disk_access(candidates: &[PathBuf]) -> Probe {
    candidates
        .iter()
        .map(|candidate| try_place(candidate))
        .find(|probe| *probe != Probe::Missing)
        .unwrap_or(Probe::Missing)
}

/// Tries one place: opens it read-only through no link at its end, lists nothing, closes it. MAY raise a system
/// prompt for a place that prompts.
pub fn try_place(path: &Path) -> Probe {
    match open(path) {
        Ok(_) => Probe::Allowed,
        Err(error) => classify(&error),
    }
}

/// Tries a folder whose own opening raises no prompt, but whose sub-folders may (the other apps' data): lists its
/// names, then opens its first sub-folder that is not a link as [`try_place`] does; that open is the one that may
/// prompt. An empty folder, or one with no sub-folder, is `Allowed`: nothing guarded to see.
pub fn try_inside(path: &Path) -> Probe {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => return classify(&error),
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return classify(&error),
        };
        // The listing's own type: a link is a link, never followed.
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            return try_place(&entry.path());
        }
    }
    Probe::Allowed
}

fn open(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt as _, symlink};
    use std::path::Path;

    fn set_mode(path: &Path, mode: u32) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    fn unreadable_file(path: &Path) {
        fs::write(path, b"x").unwrap();
        set_mode(path, 0o000);
    }

    mod negative {
        use std::fs;
        use std::io;
        use std::os::unix::fs::symlink;

        use super::{set_mode, unreadable_file};
        use crate::{Probe, classify, full_disk_access, try_inside, try_place};

        #[test]
        fn classify_tells_privacy_from_posix_permissions() {
            assert_eq!(classify(&io::Error::from_raw_os_error(libc::EPERM)), Probe::Refused);
            assert_eq!(classify(&io::Error::from_raw_os_error(libc::EACCES)), Probe::Failed);
            assert_eq!(classify(&io::Error::from_raw_os_error(libc::ENOENT)), Probe::Missing);
            assert_eq!(classify(&io::Error::other("no number")), Probe::Failed);
        }

        #[test]
        fn full_disk_access_is_missing_when_no_candidate_exists() {
            let home = tempfile::tempdir().unwrap();
            let candidates = [home.path().join("absent.db"), home.path().join("absent").join("folder")];

            assert_eq!(full_disk_access(&candidates), Probe::Missing);
            assert_eq!(full_disk_access(&[]), Probe::Missing);
        }

        #[test]
        fn a_symlinked_candidate_is_not_followed() {
            let home = tempfile::tempdir().unwrap();
            let target = home.path().join("readable.db");
            fs::write(&target, b"x").unwrap();
            let link = home.path().join("link.db");
            symlink(&target, &link).unwrap();

            assert_eq!(full_disk_access(&[link]), Probe::Failed);
        }

        #[test]
        fn an_unreadable_candidate_is_not_allowed() {
            let home = tempfile::tempdir().unwrap();
            let candidate = home.path().join("unreadable.db");
            unreadable_file(&candidate);

            assert_eq!(full_disk_access(std::slice::from_ref(&candidate)), Probe::Failed);
        }

        #[test]
        fn the_first_candidate_that_is_not_missing_decides_even_when_it_fails() {
            let home = tempfile::tempdir().unwrap();
            let unreadable = home.path().join("unreadable.db");
            unreadable_file(&unreadable);
            let readable = home.path().join("readable.db");
            fs::write(&readable, b"x").unwrap();

            assert_eq!(full_disk_access(&[unreadable, readable]), Probe::Failed);
        }

        #[test]
        fn try_place_on_a_missing_folder_is_missing() {
            let home = tempfile::tempdir().unwrap();

            assert_eq!(try_place(&home.path().join("absent")), Probe::Missing);
        }

        #[test]
        fn try_inside_skips_a_link() {
            let home = tempfile::tempdir().unwrap();
            let outside = home.path().join("outside");
            fs::create_dir(&outside).unwrap();
            set_mode(&outside, 0o000);
            let containers = home.path().join("Containers");
            fs::create_dir(&containers).unwrap();
            symlink(&outside, containers.join("link")).unwrap();

            let probe = try_inside(&containers);

            set_mode(&outside, 0o755);
            assert_eq!(probe, Probe::Allowed);
        }

        #[test]
        fn try_inside_of_a_missing_folder_is_missing() {
            let home = tempfile::tempdir().unwrap();

            assert_eq!(try_inside(&home.path().join("absent")), Probe::Missing);
        }
    }

    mod positive {
        use std::fs;

        use super::{set_mode, symlink, unreadable_file};
        use crate::{Probe, fda_candidates, full_disk_access, try_inside, try_place};

        #[test]
        fn full_disk_access_is_allowed_when_the_first_candidate_opens() {
            let home = tempfile::tempdir().unwrap();
            let readable = home.path().join("readable.db");
            fs::write(&readable, b"x").unwrap();
            let unreadable = home.path().join("unreadable.db");
            unreadable_file(&unreadable);

            assert_eq!(full_disk_access(&[readable, unreadable]), Probe::Allowed);
        }

        #[test]
        fn the_first_candidate_that_is_not_missing_decides() {
            let home = tempfile::tempdir().unwrap();
            let readable = home.path().join("Safari");
            fs::create_dir(&readable).unwrap();

            assert_eq!(full_disk_access(&[home.path().join("absent.db"), readable]), Probe::Allowed);
        }

        #[test]
        fn the_candidates_are_the_permission_database_then_the_browser_data() {
            let home = std::path::Path::new("/Users/someone");

            assert_eq!(
                fda_candidates(home),
                [home.join("Library/Application Support/com.apple.TCC/TCC.db"), home.join("Library/Safari"),]
            );
        }

        #[test]
        fn try_place_on_a_readable_folder_is_allowed() {
            let home = tempfile::tempdir().unwrap();

            assert_eq!(try_place(home.path()), Probe::Allowed);
        }

        #[test]
        fn try_inside_opens_the_first_sub_folder() {
            let home = tempfile::tempdir().unwrap();
            let containers = home.path().join("Containers");
            fs::create_dir(&containers).unwrap();

            assert_eq!(try_inside(&containers), Probe::Allowed, "an empty folder");

            fs::write(containers.join("a file"), b"x").unwrap();
            symlink(home.path(), containers.join("a link")).unwrap();
            let app = containers.join("com.example.app");
            fs::create_dir(&app).unwrap();
            set_mode(&app, 0o000);

            let probe = try_inside(&containers);

            set_mode(&app, 0o755);
            assert_eq!(probe, Probe::Failed, "the sub-folder was opened, and its POSIX refusal is not privacy");
        }
    }
}
