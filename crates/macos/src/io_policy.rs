//! The process's IO policy: a scan must never download a cloud file, and should leave access times alone.

use std::io;
use std::sync::Once;

use crate::ffi::iopolicy::{self, ProcessPolicy};

/// The system refused to set the policy.
#[derive(Debug, thiserror::Error)]
#[error("the system refused the process's IO policy: {0}")]
pub struct IoPolicyError(io::Error);

/// From now on, no thread of the process downloads the data of a cloud file by reading it: such a read fails
/// instead. Idempotent; logged once.
pub fn never_download_cloud_files() -> Result<(), IoPolicyError> {
    static LOGGED: Once = Once::new();
    turn_off(ProcessPolicy::MaterializeDatalessFiles, &LOGGED)
}

/// From now on, reading a file through the process does not update its access time. Idempotent; logged once.
pub fn leave_access_times() -> Result<(), IoPolicyError> {
    static LOGGED: Once = Once::new();
    turn_off(ProcessPolicy::AccessTimeUpdates, &LOGGED)
}

fn turn_off(policy: ProcessPolicy, logged: &Once) -> Result<(), IoPolicyError> {
    iopolicy::turn_off(policy).map_err(IoPolicyError)?;
    logged.call_once(|| tracing::info!(?policy, "turned off for the process"));
    Ok(())
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::ffi::iopolicy::{ProcessPolicy, is_off};
        use crate::io_policy::{leave_access_times, never_download_cloud_files};

        #[test]
        fn setting_the_policy_twice_is_not_an_error() {
            never_download_cloud_files().unwrap();
            never_download_cloud_files().unwrap();
            leave_access_times().unwrap();
            leave_access_times().unwrap();

            assert!(is_off(ProcessPolicy::MaterializeDatalessFiles).unwrap());
            assert!(is_off(ProcessPolicy::AccessTimeUpdates).unwrap());
        }
    }

    mod positive {
        use crate::ffi::iopolicy::{ProcessPolicy, is_off};
        use crate::io_policy::{leave_access_times, never_download_cloud_files};

        #[test]
        fn the_policy_is_set_for_the_process() {
            never_download_cloud_files().unwrap();
            leave_access_times().unwrap();

            let from_another_thread = std::thread::spawn(|| {
                (
                    is_off(ProcessPolicy::MaterializeDatalessFiles).unwrap(),
                    is_off(ProcessPolicy::AccessTimeUpdates).unwrap(),
                )
            });

            assert_eq!(from_another_thread.join().unwrap(), (true, true));
        }
    }
}
