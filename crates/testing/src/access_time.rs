//! Access times a walk does not move. The system moves a file's access time on a read only while it is older than
//! the modification time: a file written in two calls across a second keeps its creation second as its access time,
//! and the first walk that reads it (a `CACHEDIR.TAG`) would move it under the next walk's feet.

use std::fs::{self, File, FileTimes};
use std::io;
use std::path::Path;

/// Sets the access time of the file at `path` to its modification time, so reading it does not move it.
pub fn settle_access_time(path: &Path) -> io::Result<()> {
    let modified = fs::symlink_metadata(path)?.modified()?;
    File::open(path)?.set_times(FileTimes::new().set_accessed(modified))
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::access_time::settle_access_time;

        #[test]
        fn a_missing_file_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            assert!(settle_access_time(&root.path().join("missing")).is_err());
        }
    }

    mod positive {
        use std::fs::{self, File, FileTimes};
        use std::io::Read as _;
        use std::time::{Duration, SystemTime};

        use crate::access_time::settle_access_time;

        #[test]
        fn a_read_does_not_move_a_settled_access_time() {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("CACHEDIR.TAG");
            fs::write(&path, b"tag").unwrap();
            let past = SystemTime::UNIX_EPOCH + Duration::from_hours(400_000);
            let later = past + Duration::from_mins(1);
            File::options().write(true).open(&path).unwrap().set_times(FileTimes::new().set_accessed(past)).unwrap();
            File::options().write(true).open(&path).unwrap().set_modified(later).unwrap();

            settle_access_time(&path).unwrap();
            File::open(&path).unwrap().read_to_end(&mut Vec::new()).unwrap();

            assert_eq!(fs::metadata(&path).unwrap().accessed().unwrap(), later);
        }
    }
}
