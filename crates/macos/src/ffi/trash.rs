//! The system's Trash call (`NSFileManager`'s `trashItemAtURL:resultingItemURL:error:`) and where a held item is
//! now (`fcntl(F_GETPATH)`).

use std::ffi::{CStr, OsStr};
use std::io;
use std::os::fd::{AsRawFd as _, BorrowedFd};
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::{NSFileManager, NSURL};

/// The longest path the system names (`MAXPATHLEN` of `<sys/param.h>`), its NUL included.
const MAX_PATH: usize = 1024;

/// Why the Trash call moved nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TrashFailure {
    /// The path cannot be made into a file URL (it is empty or holds a NUL byte).
    BadPath,
    /// The call's `NSError`: its domain and code.
    Refused { domain: String, code: isize },
}

/// Moves the item at `path` to the Trash of its volume. `Ok` holds the landed path the call returned, which it may
/// leave out (a race, 01/01). The URL is made from the path's own bytes; nothing of the item is followed.
pub(crate) fn trash_item(path: &Path, is_directory: bool) -> Result<Option<PathBuf>, TrashFailure> {
    autoreleasepool(|_| {
        let url = NSURL::from_path(path, is_directory, None).ok_or(TrashFailure::BadPath)?;
        let manager = NSFileManager::defaultManager();
        let mut landed: Option<Retained<NSURL>> = None;
        match manager.trashItemAtURL_resultingItemURL_error(&url, Some(&mut landed)) {
            Ok(()) => Ok(landed.and_then(|url| url.to_file_path())),
            Err(error) => Err(TrashFailure::Refused { domain: error.domain().to_string(), code: error.code() }),
        }
    })
}

/// Where the item behind `fd` is now, whatever moved it since it was opened.
pub(crate) fn path_of(fd: BorrowedFd<'_>) -> io::Result<PathBuf> {
    let mut buffer = [0_u8; MAX_PATH];
    // SAFETY: `F_GETPATH` writes at most `MAXPATHLEN` bytes, NUL included, into the buffer, which is that long and
    // alive until the call returns; `fd` is an open descriptor for the call's duration.
    let result = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETPATH, buffer.as_mut_ptr()) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    let path = CStr::from_bytes_until_nul(&buffer).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
    Ok(PathBuf::from(OsStr::from_bytes(path.to_bytes())))
}
