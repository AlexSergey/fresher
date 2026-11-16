//! The process's resource usage (`getrusage`).

use std::io;
use std::mem::MaybeUninit;

use super::check;

/// The largest resident memory of the process so far, in bytes (`ru_maxrss` is in bytes on macOS).
pub(crate) fn peak_resident_bytes() -> io::Result<i64> {
    let mut usage = MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: `usage` is a valid, writable `rusage` for the duration of the call.
    check(unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) })?;
    // SAFETY: zeroed memory is a valid `rusage` (integers only), and the call above filled it.
    let usage = unsafe { usage.assume_init() };
    Ok(usage.ru_maxrss)
}
