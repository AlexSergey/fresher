//! The one reader of the sandbox marker, shared by the cleaner's reach and the sandbox generator, so both decide alike
//! whether a folder is a sandbox.

use std::fs::{self, OpenOptions};
use std::io::Read as _;
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::path::Path;

use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

/// Whether `folder` holds a valid marker of `user`: a regular file of that user, opened through no link in any
/// component (`O_NOFOLLOW_ANY`) and without waiting on a named pipe, which is the entry the folder lists (same volume
/// and file id as its `lstat`), and whose first line is the signature. Any failure answers `false`.
pub fn holds_marker(folder: &Path, user: u32) -> bool {
    let path = folder.join(SANDBOX_MARKER);
    let Ok(marker) = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW_ANY | libc::O_NONBLOCK).open(&path)
    else {
        return false;
    };
    let (Ok(opened), Ok(entry)) = (marker.metadata(), fs::symlink_metadata(&path)) else {
        return false;
    };
    if !opened.is_file() || opened.uid() != user || opened.dev() != entry.dev() || opened.ino() != entry.ino() {
        return false;
    }
    let signature = SANDBOX_SIGNATURE.as_bytes();
    let mut first = Vec::with_capacity(signature.len() + 1);
    if marker.take(signature.len() as u64 + 1).read_to_end(&mut first).is_err() {
        return false;
    }
    first.strip_suffix(b"\n").unwrap_or(&first) == signature
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::{Path, PathBuf};

    use fresher_core::SANDBOX_MARKER;
    use tempfile::TempDir;

    fn user(folder: &Path) -> u32 {
        fs::metadata(folder).unwrap().uid()
    }

    /// A temporary folder's canonical path: `/var` is a link of the system, and the reader follows no link.
    fn canonical(root: &TempDir) -> PathBuf {
        fs::canonicalize(root.path()).unwrap()
    }

    /// A temporary folder `sandbox/` holding a marker with `text`, and the folder's canonical path.
    fn marked(text: &str) -> (TempDir, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let folder = canonical(&root).join("sandbox");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join(SANDBOX_MARKER), text).unwrap();
        (root, folder)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::process::Command;

        use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

        use super::{canonical, marked, user};
        use crate::sandbox_marker::holds_marker;

        #[test]
        fn a_folder_without_a_marker_holds_none() {
            let root = tempfile::tempdir().unwrap();

            assert!(!holds_marker(&canonical(&root), user(root.path())));
        }

        #[test]
        fn a_marker_with_another_first_line_does_not_count() {
            for text in
                ["", "fresher-sandbox 0\n", &format!("{SANDBOX_SIGNATURE}x\n"), &format!("x{SANDBOX_SIGNATURE}")]
            {
                let (_root, folder) = marked(text);

                assert!(!holds_marker(&folder, user(&folder)), "{text:?}");
            }
        }

        #[test]
        fn a_marker_of_another_user_does_not_count() {
            let (_root, folder) = marked(SANDBOX_SIGNATURE);

            assert!(!holds_marker(&folder, user(&folder).wrapping_add(1)));
        }

        #[test]
        fn a_marker_that_is_a_link_does_not_count() {
            let (root, folder) = marked(SANDBOX_SIGNATURE);
            let linked = canonical(&root).join("linked");
            fs::create_dir(&linked).unwrap();
            symlink(folder.join(SANDBOX_MARKER), linked.join(SANDBOX_MARKER)).unwrap();

            assert!(!holds_marker(&linked, user(&linked)));
        }

        #[test]
        fn a_marker_reached_through_a_linked_folder_does_not_count() {
            let (root, folder) = marked(SANDBOX_SIGNATURE);
            let alias = canonical(&root).join("alias");
            symlink(&folder, &alias).unwrap();

            assert!(!holds_marker(&alias, user(&folder)));
        }

        #[test]
        fn a_marker_that_is_a_folder_or_a_pipe_does_not_count() {
            let root = tempfile::tempdir().unwrap();
            let folder = canonical(&root).join("folder");
            fs::create_dir_all(folder.join(SANDBOX_MARKER)).unwrap();
            let piped = canonical(&root).join("piped");
            fs::create_dir(&piped).unwrap();
            let made = Command::new("mkfifo").arg(piped.join(SANDBOX_MARKER)).status().unwrap();
            assert!(made.success());

            assert!(!holds_marker(&folder, user(&folder)));
            assert!(!holds_marker(&piped, user(&piped)));
        }
    }

    mod positive {
        use fresher_core::SANDBOX_SIGNATURE;

        use super::{marked, user};
        use crate::sandbox_marker::holds_marker;

        #[test]
        fn a_signed_marker_counts_with_or_without_a_line_end_and_more_lines() {
            for text in
                [SANDBOX_SIGNATURE.to_owned(), format!("{SANDBOX_SIGNATURE}\n"), format!("{SANDBOX_SIGNATURE}\nx\n")]
            {
                let (_root, folder) = marked(&text);

                assert!(holds_marker(&folder, user(&folder)), "{text:?}");
            }
        }
    }
}
