//! Volume facts: `statfs` and the volume attributes of `getattrlist`.

use std::ffi::CStr;
use std::io;
use std::path::Path;

use super::{c_path, check};

/// The file system statistics of the volume that holds `path` (a link in `path` is followed).
pub(crate) fn statfs(path: &Path) -> io::Result<libc::statfs> {
    let path = c_path(path)?;
    let mut stats = libc::statfs::default();
    // SAFETY: `path` is NUL-terminated and `stats` is a valid, writable `statfs`, both alive until the call returns.
    check(unsafe { libc::statfs(path.as_ptr(), &raw mut stats) })?;
    Ok(stats)
}

/// The attributes of `list` for the object at `path`, into `buffer`; a link is not followed.
pub(crate) fn attributes(path: &CStr, list: &libc::attrlist, buffer: &mut [u8]) -> io::Result<()> {
    let mut list = *list;
    // SAFETY: `path` is NUL-terminated, `list` is a valid `attrlist`, and the kernel writes at most `buffer.len()`
    // bytes into `buffer`; nothing is kept after the call.
    check(unsafe {
        libc::getattrlist(
            path.as_ptr(),
            (&raw mut list).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            libc::FSOPT_NOFOLLOW,
        )
    })
}

/// Every mounted volume's statistics, as the system has them cached (`MNT_NOWAIT`: no volume is asked, none blocks).
pub(crate) fn mounts() -> io::Result<Vec<libc::statfs>> {
    // SAFETY: a null buffer of size 0 asks only for the number of mounts.
    let count = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
    let count = usize::try_from(count).map_err(|_| io::Error::last_os_error())?;
    // Room for a few volumes mounted between the two calls.
    let mut mounts = vec![libc::statfs::default(); count + 4];
    let bytes = libc::c_int::try_from(size_of_val(mounts.as_slice()))
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    // SAFETY: `mounts` is writable for exactly `bytes` bytes, a whole number of `statfs`.
    let filled = unsafe { libc::getfsstat(mounts.as_mut_ptr(), bytes, libc::MNT_NOWAIT) };
    mounts.truncate(usize::try_from(filled).map_err(|_| io::Error::last_os_error())?);
    Ok(mounts)
}
