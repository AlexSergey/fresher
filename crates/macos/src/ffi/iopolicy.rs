//! The process's IO policy (`setiopolicy_np`): which side effects a read of the file system may have.

use std::io;

use libc::c_int;

use super::check;

// SAFETY: the signatures match `<sys/resource.h>`; `libc` does not declare them.
unsafe extern "C" {
    fn setiopolicy_np(iotype: c_int, scope: c_int, policy: c_int) -> c_int;
    #[cfg(test)]
    fn getiopolicy_np(iotype: c_int, scope: c_int) -> c_int;
}

const IOPOL_TYPE_VFS_ATIME_UPDATES: c_int = 2;
const IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES: c_int = 3;
const IOPOL_SCOPE_PROCESS: c_int = 0;
const IOPOL_ATIME_UPDATES_OFF: c_int = 1;
const IOPOL_MATERIALIZE_DATALESS_FILES_OFF: c_int = 1;

/// A policy this app turns off for the whole process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessPolicy {
    /// Reading a cloud file whose data is not on this Mac would download it.
    MaterializeDatalessFiles,
    /// Reading a file would update its access time.
    AccessTimeUpdates,
}

impl ProcessPolicy {
    const fn kind(self) -> c_int {
        match self {
            Self::MaterializeDatalessFiles => IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES,
            Self::AccessTimeUpdates => IOPOL_TYPE_VFS_ATIME_UPDATES,
        }
    }

    const fn off(self) -> c_int {
        match self {
            Self::MaterializeDatalessFiles => IOPOL_MATERIALIZE_DATALESS_FILES_OFF,
            Self::AccessTimeUpdates => IOPOL_ATIME_UPDATES_OFF,
        }
    }
}

/// Turns `policy` off for every thread of the process.
pub(crate) fn turn_off(policy: ProcessPolicy) -> io::Result<()> {
    // SAFETY: plain integers in, an integer out; the call touches no memory of ours.
    check(unsafe { setiopolicy_np(policy.kind(), IOPOL_SCOPE_PROCESS, policy.off()) })
}

/// Whether `policy` is off for the process.
#[cfg(test)]
pub(crate) fn is_off(policy: ProcessPolicy) -> io::Result<bool> {
    // SAFETY: plain integers in, an integer out; the call touches no memory of ours.
    let current = unsafe { getiopolicy_np(policy.kind(), IOPOL_SCOPE_PROCESS) };
    if current < 0 { Err(io::Error::last_os_error()) } else { Ok(current == policy.off()) }
}
