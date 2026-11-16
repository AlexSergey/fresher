//! Facts about the running process.

use fresher_core::ByteSize;

use crate::ffi::{libproc, rusage};

/// The user this process runs as (`getuid`): the owner the cleaner compares an item's with.
pub fn current_user() -> u32 {
    libproc::user_id()
}

/// The largest resident memory the process has used so far; zero if the system does not tell.
pub fn peak_memory() -> ByteSize {
    match rusage::peak_resident_bytes() {
        Ok(bytes) => ByteSize::new(u64::try_from(bytes).unwrap_or(0)),
        Err(error) => {
            tracing::warn!(%error, "the peak memory is unknown");
            ByteSize::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::hint::black_box;
        use std::os::unix::fs::MetadataExt as _;

        use crate::{current_user, peak_memory};

        #[test]
        fn the_current_user_owns_a_new_temporary_file() {
            let file = tempfile::NamedTempFile::new().unwrap();

            assert_eq!(file.as_file().metadata().unwrap().uid(), current_user());
        }

        #[test]
        fn peak_memory_grows_after_a_large_allocation() {
            const SIZE: usize = 256 * 1024 * 1024;
            let before = peak_memory();

            let touched = black_box(vec![1_u8; SIZE]);
            let after = peak_memory();
            drop(touched);

            assert!(before.bytes() > 0);
            assert!(after.bytes() >= before.bytes() + (SIZE as u64) / 2, "{before} -> {after}");
        }
    }
}
