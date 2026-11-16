//! Test helpers that write, for the tests of other crates (the feature `fixtures`); never used by the app.

use std::io;
use std::path::Path;

use crate::ffi;

/// Creates `to` as an APFS clone of `from` (a new file sharing its blocks); `to` must not exist. It creates a file and
/// never removes one.
pub fn clone_file(from: &Path, to: &Path) -> io::Result<()> {
    ffi::clone::clone_file(from, to)
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::fixtures::clone_file;

        #[test]
        fn cloning_onto_an_existing_file_fails_and_keeps_it() {
            let folder = tempfile::tempdir().unwrap();
            let (from, to) = (folder.path().join("from"), folder.path().join("to"));
            fs::write(&from, b"source").unwrap();
            fs::write(&to, b"kept").unwrap();

            assert!(clone_file(&from, &to).is_err());
            assert_eq!(fs::read(&to).unwrap(), b"kept");
        }
    }

    mod positive {
        use std::fs;

        use crate::ffi::clone::clone_id_and_flags;
        use crate::fixtures::clone_file;

        /// `EF_MAY_SHARE_BLOCKS` of `<sys/attr.h>`.
        const EF_MAY_SHARE_BLOCKS: u64 = 0x1;

        #[test]
        fn a_cloned_file_has_a_clone_id() {
            let folder = tempfile::tempdir().unwrap();
            let (from, to) = (folder.path().join("from"), folder.path().join("to"));
            fs::write(&from, vec![7_u8; 64 * 1024]).unwrap();

            clone_file(&from, &to).unwrap();

            let (clone, flags) = clone_id_and_flags(&to).unwrap();
            assert_eq!(fs::read(&to).unwrap(), fs::read(&from).unwrap());
            assert_ne!(clone, 0);
            assert_eq!(clone_id_and_flags(&from).unwrap().0, clone);
            assert_ne!(flags & EF_MAY_SHARE_BLOCKS, 0);
        }
    }
}
