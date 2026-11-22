//! What can go wrong inside the engine, and how it reaches the UI.

use std::io;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CommandError, JournalError, PlanError, RemovalError, StorageMapError, SystemUiError};

/// A failure inside the engine. It is a bug or a broken host, never the user's input: the UI sees `Internal`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum EngineError {
    #[error("a lock of the engine was poisoned by a panic")]
    Poisoned,
    #[error("the thread of an operation could not be started: {kind}")]
    Spawn { kind: io::ErrorKind },
    #[error("the scan ids ran out")]
    ScanIdsExhausted,
    #[error("the plan ids ran out")]
    PlanIdsExhausted,
    #[error("the finding ids ran out")]
    FindingIdsExhausted,
    #[error("the plan could not be made: {0}")]
    Plan(PlanError),
    #[error("the rehearsal answered {answers} times for {items} items")]
    Rehearsal { items: usize, answers: usize },
    #[error("the cleanup journal could not be read: {0}")]
    Journal(JournalError),
    #[error("the system could not show the item: {0}")]
    SystemUi(SystemUiError),
    #[error("the storage map could not be drawn: {0}")]
    StorageMap(StorageMapError),
    #[error("an uninstall's paths could not be made one item: {0}")]
    Parts(RemovalError),
}

impl From<EngineError> for CommandError {
    fn from(error: EngineError) -> Self {
        tracing::error!(%error, "the engine refused a command");
        Self::Internal
    }
}

/// Locks `mutex`; a poisoned lock is an [`EngineError`], never a panic.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, EngineError> {
    mutex.lock().map_err(|_| EngineError::Poisoned)
}

/// Locks `mutex` on a path that must finish (ending a scan, quitting): the data is taken as it was left.
pub(crate) fn lock_anyway<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::io;

        use fresher_core::CommandError;

        use crate::error::EngineError;

        #[test]
        fn every_engine_error_is_internal_for_the_ui() {
            let errors = [
                EngineError::Poisoned,
                EngineError::Spawn { kind: io::ErrorKind::OutOfMemory },
                EngineError::ScanIdsExhausted,
                EngineError::PlanIdsExhausted,
                EngineError::FindingIdsExhausted,
                EngineError::Plan(fresher_core::PlanError::Overflow),
                EngineError::Rehearsal { items: 2, answers: 1 },
                EngineError::Journal(fresher_core::JournalError::ReadOnly),
                EngineError::SystemUi(fresher_core::SystemUiError::Unsupported),
                EngineError::StorageMap(fresher_core::StorageMapError::Overflow),
                EngineError::Parts(fresher_core::RemovalError::TooFewParts),
            ];

            for error in errors {
                assert_eq!(CommandError::from(error), CommandError::Internal);
            }
        }
    }

    mod positive {
        use std::sync::Mutex;

        use crate::error::{EngineError, lock, lock_anyway};
        use crate::testing::poison;

        #[test]
        fn a_poisoned_lock_is_an_error_or_taken_as_it_was() {
            let mutex = Mutex::new(7);
            poison(&mutex);

            assert_eq!(lock(&mutex).err(), Some(EngineError::Poisoned));
            assert_eq!(*lock_anyway(&mutex), 7);
        }
    }
}
