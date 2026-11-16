//! A rename that never replaces anything and follows no link (`renamex_np`).

use std::io;
use std::path::Path;

use super::{c_path, check};

/// `RENAME_NOFOLLOW_ANY` of the SDK's `<sys/stdio.h>`: a link in any component of either path fails with `ELOOP`.
/// `libc` 0.2.191 lacks it.
const RENAME_NOFOLLOW_ANY: libc::c_uint = 0x10;

/// Renames `from` to `to`; fails with `EEXIST` when `to` exists and with `ELOOP` when either path holds a link. The
/// error keeps the system's number.
pub(crate) fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
    let (from, to) = (c_path(from)?, c_path(to)?);
    // SAFETY: both pointers are NUL-terminated strings that live until the call returns.
    check(unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL | RENAME_NOFOLLOW_ANY) })
}
