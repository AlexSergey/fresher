//! APFS clones (`clonefile`), for test fixtures only.

use std::io;
use std::path::Path;

use super::{c_path, check};

/// `CLONE_NOFOLLOW` of `<sys/clonefile.h>`: a link is cloned as a link, never followed.
const CLONE_NOFOLLOW: u32 = 0x1;

/// Creates `to` as a clone of `from`; `to` must not exist.
pub(crate) fn clone_file(from: &Path, to: &Path) -> io::Result<()> {
    let (from, to) = (c_path(from)?, c_path(to)?);
    // SAFETY: both pointers are NUL-terminated strings that live until the call returns.
    check(unsafe { libc::clonefile(from.as_ptr(), to.as_ptr(), CLONE_NOFOLLOW) })
}

/// The clone id and the extended flags of the file at `path` (`ATTR_CMNEXT_CLONEID`, `ATTR_CMNEXT_EXT_FLAGS`).
#[cfg(test)]
pub(crate) fn clone_id_and_flags(path: &Path) -> io::Result<(u64, u64)> {
    let path = c_path(path)?;
    let mut list = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: 0,
        volattr: 0,
        dirattr: 0,
        fileattr: 0,
        forkattr: libc::ATTR_CMNEXT_CLONEID | libc::ATTR_CMNEXT_EXT_FLAGS,
    };
    let mut buffer = [0_u8; 64];
    // SAFETY: `path` is NUL-terminated, `list` is a valid `attrlist` and `buffer` is writable for its exact length,
    // all alive until the call returns.
    check(unsafe {
        libc::getattrlist(
            path.as_ptr(),
            (&raw mut list).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            libc::FSOPT_NOFOLLOW | libc::FSOPT_ATTR_CMN_EXTENDED,
        )
    })?;
    let field = |at: usize| buffer.get(at..at + 8).and_then(|bytes| bytes.try_into().ok()).map(u64::from_ne_bytes);
    field(4).zip(field(12)).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
}
