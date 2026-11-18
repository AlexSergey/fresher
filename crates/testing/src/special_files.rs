//! Named pipes made in a test's own temporary folder: entries that are neither files, folders nor links. A socket
//! needs no helper: `std::os::unix::net::UnixListener::bind`.

use std::io;
use std::path::Path;
use std::process::Command;

/// Makes a named pipe (FIFO) at `path` with the system's `mkfifo`; nothing writes to it.
pub fn make_pipe(path: &Path) -> io::Result<()> {
    let status = Command::new("/usr/bin/mkfifo").arg(path).status()?;
    if status.success() { Ok(()) } else { Err(io::Error::other(format!("mkfifo failed: {status}"))) }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::special_files::make_pipe;

        #[test]
        fn a_pipe_is_not_made_in_a_missing_folder() {
            let root = tempfile::tempdir().unwrap();

            assert!(make_pipe(&root.path().join("missing/pipe")).is_err());
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::FileTypeExt as _;

        use crate::special_files::make_pipe;

        #[test]
        fn makes_a_pipe() {
            let root = tempfile::tempdir().unwrap();
            make_pipe(&root.path().join("pipe")).unwrap();

            assert!(fs::symlink_metadata(root.path().join("pipe")).unwrap().file_type().is_fifo());
        }
    }
}
