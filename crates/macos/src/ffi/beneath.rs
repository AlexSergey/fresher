//! The calls relative to a held folder that `std` lacks: the status of one of its entries (`fstatat`) and the one
//! deleting call of the workspace (`unlinkat`), which only the cleaner's deletion point may call.

use std::ffi::OsStr;
use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd as _, BorrowedFd};

use fresher_core::EntryKind;

use super::{c_name, check};

/// The status of the entry `name` of the open folder `folder`, a link as itself (`AT_SYMLINK_NOFOLLOW`).
pub(crate) fn stat_at(folder: BorrowedFd<'_>, name: &OsStr) -> io::Result<libc::stat> {
    let name = c_name(name)?;
    let mut status = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: the descriptor is open for the borrow, `name` is NUL-terminated and `status` is writable for a whole
    // `stat`, all alive until the call returns; the call keeps none of them.
    check(unsafe { libc::fstatat(folder.as_raw_fd(), name.as_ptr(), status.as_mut_ptr(), libc::AT_SYMLINK_NOFOLLOW) })?;
    // SAFETY: `fstatat` returned 0, so it wrote the whole `stat`.
    Ok(unsafe { status.assume_init() })
}

/// Removes the entry `name` of the open folder `folder`: a file or a link (as the link) with `unlinkat`, an empty
/// folder with `AT_REMOVEDIR`. `name` must be one plain name (`InvalidInput` otherwise), so nothing is resolved
/// through a path. The error keeps the system's number.
///
/// The only deleting function of the workspace besides `std::fs`'s: Clippy's `disallowed-methods` forbids calling it
/// anywhere but the deletion point, `crates/cleaner/src/remove/permanent.rs` (docs/architecture/removal.md).
pub fn remove_at(folder: BorrowedFd<'_>, name: &OsStr, kind: EntryKind) -> io::Result<()> {
    let name = c_name(name)?;
    let flags = match kind {
        EntryKind::Folder => libc::AT_REMOVEDIR,
        EntryKind::File | EntryKind::Symlink => 0,
    };
    // SAFETY: the descriptor is open for the borrow and `name` is NUL-terminated, alive until the call returns; the
    // call keeps neither.
    check(unsafe { libc::unlinkat(folder.as_raw_fd(), name.as_ptr(), flags) })
}
