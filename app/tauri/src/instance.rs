//! One running app per user and identifier. A bundled app opened again is brought forward by macOS itself (the system
//! activates the running copy, the app sees a reopen); a second process started any other way (`open -n`, the binary
//! from a terminal, a second `npm run dev`) finds the lock held and does not start. The lock is an advisory lock on the
//! data folder itself: the folder is private to the user, so no other account can hold it, nothing is written into it,
//! and the system drops the lock when the process ends.

use std::fs::{File, TryLockError};
use std::io;
use std::path::Path;
use std::sync::Mutex;

/// Why this process may not run.
#[derive(Debug, thiserror::Error)]
pub(crate) enum InstanceError {
    #[error("Fresher is already running with this identifier: quit it first")]
    Running,
    #[error("the data folder could not be locked: {kind}")]
    Io { kind: io::ErrorKind },
}

/// The lock of the running app, held until the process ends or [`InstanceLock::release`].
#[derive(Debug)]
pub(crate) struct InstanceLock(Mutex<Option<File>>);

impl InstanceLock {
    /// Takes the lock of `folder` (the app's data folder, which exists), or says another process holds it.
    pub(crate) fn take(folder: &Path) -> Result<Self, InstanceError> {
        let file = File::open(folder).map_err(|error| InstanceError::Io { kind: error.kind() })?;
        match file.try_lock() {
            Ok(()) => Ok(Self(Mutex::new(Some(file)))),
            Err(TryLockError::WouldBlock) => Err(InstanceError::Running),
            Err(TryLockError::Error(error)) => Err(InstanceError::Io { kind: error.kind() }),
        }
    }

    /// Lets the lock go before the process ends, so the copy a relaunch starts can take it.
    pub(crate) fn release(&self) {
        let mut held = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        held.take();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::instance::{InstanceError, InstanceLock};

        #[test]
        fn a_second_take_of_a_held_folder_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            let _held = InstanceLock::take(folder.path()).unwrap();

            assert!(matches!(InstanceLock::take(folder.path()), Err(InstanceError::Running)));
        }

        #[test]
        fn a_missing_folder_is_an_error() {
            let folder = tempfile::tempdir().unwrap();

            let taken = InstanceLock::take(&folder.path().join("missing"));

            assert!(matches!(taken, Err(InstanceError::Io { kind: std::io::ErrorKind::NotFound })));
        }
    }

    mod positive {
        use crate::instance::InstanceLock;

        #[test]
        fn a_released_lock_can_be_taken_again() {
            let folder = tempfile::tempdir().unwrap();
            let held = InstanceLock::take(folder.path()).unwrap();

            held.release();

            assert!(InstanceLock::take(folder.path()).is_ok());
        }

        #[test]
        fn a_dropped_lock_can_be_taken_again() {
            let folder = tempfile::tempdir().unwrap();
            drop(InstanceLock::take(folder.path()).unwrap());

            assert!(InstanceLock::take(folder.path()).is_ok());
        }

        #[test]
        fn the_lock_writes_nothing_into_the_folder() {
            let folder = tempfile::tempdir().unwrap();
            let _held = InstanceLock::take(folder.path()).unwrap();

            assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
        }
    }
}
