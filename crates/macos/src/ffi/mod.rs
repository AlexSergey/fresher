//! Every FFI call of Fresher. Each unsafe block holds one operation and a `// SAFETY:` comment stating why it is
//! sound; the functions here are safe to call, so nothing outside this module needs `unsafe`.

pub(crate) mod beneath;
pub(crate) mod bulk;
#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod clone;
pub(crate) mod fsevents;
pub(crate) mod iopolicy;
pub(crate) mod libproc;
pub(crate) mod rename;
pub(crate) mod rusage;
pub(crate) mod trash;
pub(crate) mod volume;

use std::ffi::{CString, OsStr};
use std::io;
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

/// The path as a C string; a path holding a NUL byte cannot name a file.
pub(crate) fn c_path(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
}

/// One plain name of a folder's entry as a C string: never empty, `.` or `..`, and holding no `/` or NUL, so a call
/// relative to a folder resolves nothing through a path.
pub(crate) fn c_name(name: &OsStr) -> io::Result<CString> {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    CString::new(bytes).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
}

/// The result of a call that returns 0 on success and -1 with `errno` on failure.
fn check(result: libc::c_int) -> io::Result<()> {
    if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}
