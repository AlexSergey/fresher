//! The bulk folder read (`getattrlistbulk`) and the calls around it that `std` lacks (`openat`, `getattrlistat`).

use std::ffi::CStr;
use std::io;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

use super::check;

/// Reads the next entries of the open folder `folder` into `buffer`, with the attributes of `list`; answers how many
/// records were written, 0 at the end of the folder.
pub(crate) fn read_entries(
    folder: BorrowedFd<'_>,
    list: &libc::attrlist,
    buffer: &mut [u8],
    options: u64,
) -> io::Result<usize> {
    let mut list = *list;
    // SAFETY: the descriptor is open for the borrow, `list` is a valid `attrlist`, and the kernel writes at most
    // `buffer.len()` bytes into `buffer`, which is writable for that length; nothing is kept after the call.
    let count = unsafe {
        libc::getattrlistbulk(
            folder.as_raw_fd(),
            (&raw mut list).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            options,
        )
    };
    usize::try_from(count).map_err(|_| io::Error::last_os_error())
}

/// The attributes of `list` for the entry `name` of the open folder `folder`, into `buffer`.
pub(crate) fn attributes_at(
    folder: BorrowedFd<'_>,
    name: &CStr,
    list: &libc::attrlist,
    buffer: &mut [u8],
    options: u32,
) -> io::Result<()> {
    let mut list = *list;
    // SAFETY: the descriptor is open for the borrow, `name` is NUL-terminated, `list` is a valid `attrlist`, and the
    // kernel writes at most `buffer.len()` bytes into `buffer`; nothing is kept after the call.
    check(unsafe {
        libc::getattrlistat(
            folder.as_raw_fd(),
            name.as_ptr(),
            (&raw mut list).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            libc::c_ulong::from(options),
        )
    })
}

/// Opens the entry `name` of the open folder `folder` with `flags` (`O_CLOEXEC` is always added).
pub(crate) fn open_at(folder: BorrowedFd<'_>, name: &CStr, flags: libc::c_int) -> io::Result<OwnedFd> {
    // SAFETY: the descriptor is open for the borrow and `name` is NUL-terminated; the call keeps neither.
    let descriptor = unsafe { libc::openat(folder.as_raw_fd(), name.as_ptr(), flags | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    Ok(unsafe { OwnedFd::from_raw_fd(descriptor) })
}
