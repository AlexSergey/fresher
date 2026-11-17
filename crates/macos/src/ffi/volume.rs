//! Volume facts: `statfs`, the volume attributes of `getattrlist` and two resource values of Foundation.

use std::ffi::CStr;
use std::io;
use std::path::Path;

use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2_foundation::{
    NSCocoaErrorDomain, NSNumber, NSPOSIXErrorDomain, NSString, NSURL, NSURLResourceKey,
    NSURLVolumeAvailableCapacityForImportantUsageKey, NSURLVolumeIsInternalKey,
};

use super::{c_path, check};

/// `NSFileNoSuchFileError` and `NSFileReadNoSuchFileError` of `<Foundation/FoundationErrors.h>`.
const COCOA_NO_SUCH_FILE: [isize; 2] = [4, 260];

/// What Foundation knows of the volume that holds a path and `statfs` does not tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VolumeResources {
    /// The capacity available for important usage: the free space plus what the system can purge.
    pub(crate) important_available: Option<u64>,
    /// Whether the volume is on an internal disk.
    pub(crate) internal: Option<bool>,
}

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

/// The resources of the volume that holds `path`; each is `None` when the system gives no value for it.
pub(crate) fn resources(path: &Path) -> io::Result<VolumeResources> {
    autoreleasepool(|_| {
        // `None` for an empty path or one holding a NUL byte, as `c_path`.
        let url = NSURL::from_path(path, false, None).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        // SAFETY: reading an immutable Foundation constant, set when the framework is loaded.
        let important_key = unsafe { NSURLVolumeAvailableCapacityForImportantUsageKey };
        // SAFETY: reading an immutable Foundation constant, set when the framework is loaded.
        let internal_key = unsafe { NSURLVolumeIsInternalKey };
        let important_available = number(&url, important_key)?.map(|number| number.unsignedLongLongValue());
        let internal = number(&url, internal_key)?.map(|number| number.boolValue());
        Ok(VolumeResources { important_available, internal })
    })
}

/// The resource value of `key` for `url`, when it is an `NSNumber`.
fn number(url: &NSURL, key: &NSURLResourceKey) -> io::Result<Option<Retained<NSNumber>>> {
    let mut value: Option<Retained<AnyObject>> = None;
    // SAFETY: `value` is typed `AnyObject`, which every object the call may store is; it is checked as an `NSNumber`
    // below before any use.
    unsafe { url.getResourceValue_forKey_error(&mut value, key) }
        .map_err(|error| io_error(&error.domain(), error.code()))?;
    Ok(value.and_then(|value| value.downcast::<NSNumber>().ok()))
}

/// The `io::Error` an `NSError` of the resource call (its domain and code) stands for.
fn io_error(domain: &NSString, code: isize) -> io::Error {
    // SAFETY: reading an immutable Foundation constant, set when the framework is loaded.
    let posix = unsafe { NSPOSIXErrorDomain };
    // SAFETY: reading an immutable Foundation constant, set when the framework is loaded.
    let cocoa = unsafe { NSCocoaErrorDomain };
    if *domain == *posix {
        if let Ok(code) = i32::try_from(code) {
            return io::Error::from_raw_os_error(code);
        }
    } else if *domain == *cocoa && COCOA_NO_SUCH_FILE.contains(&code) {
        return io::Error::from(io::ErrorKind::NotFound);
    }
    io::Error::other(format!("{domain} error {code}"))
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::ffi::OsStr;
        use std::io;
        use std::os::unix::ffi::OsStrExt as _;
        use std::path::Path;

        use objc2_foundation::NSString;

        use crate::ffi::volume::{io_error, resources};

        #[test]
        fn an_unknown_error_domain_is_another_error() {
            let error = io_error(&NSString::from_str("SomeOtherDomain"), 7);

            assert_eq!(error.kind(), io::ErrorKind::Other);
            assert_eq!(error.raw_os_error(), None);
        }

        #[test]
        fn a_cocoa_error_that_is_not_a_missing_file_is_another_error() {
            let error = io_error(&NSString::from_str("NSCocoaErrorDomain"), 257);

            assert_eq!(error.kind(), io::ErrorKind::Other);
        }

        #[test]
        fn a_path_that_does_not_exist_is_an_error() {
            let folder = tempfile::tempdir().unwrap();

            let error = resources(&folder.path().join("missing")).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::NotFound);
        }

        #[test]
        fn a_path_with_a_nul_is_refused() {
            let path = Path::new(OsStr::from_bytes(b"/tmp/a\0b"));

            let error = resources(path).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
    }

    mod positive {
        use std::io;

        use objc2_foundation::NSString;

        use crate::disk_image::{DiskImage, Format};
        use crate::ffi::volume::{io_error, resources, statfs};

        #[test]
        fn a_posix_error_keeps_its_errno() {
            let error = io_error(&NSString::from_str("NSPOSIXErrorDomain"), isize::try_from(libc::EACCES).unwrap());

            assert_eq!(error.raw_os_error(), Some(libc::EACCES));
        }

        #[test]
        fn a_missing_file_of_cocoa_is_not_found() {
            let error = io_error(&NSString::from_str("NSCocoaErrorDomain"), 260);

            assert_eq!(error.kind(), io::ErrorKind::NotFound);
        }

        #[test]
        fn the_important_capacity_is_at_least_the_free_space() {
            let folder = tempfile::tempdir().unwrap();
            let free = || {
                let stats = statfs(folder.path()).unwrap();
                stats.f_bavail * u64::from(stats.f_bsize)
            };

            let before = free();
            let important = resources(folder.path()).unwrap().important_available.unwrap();
            let after = free();

            // Other programs write while the test runs: only the smaller reading is a floor the answer must clear.
            let free = before.min(after);
            assert!(important >= free, "important capacity {important} is below the free space {free}");
        }

        #[test]
        fn the_volume_of_a_temporary_folder_is_internal() {
            let folder = tempfile::tempdir().unwrap();

            let internal = resources(folder.path()).unwrap().internal;

            if internal == Some(false) {
                eprintln!("skipped: the temporary folder {} is not on an internal disk", folder.path().display());
                return;
            }
            assert_eq!(internal, Some(true));
        }

        #[test]
        fn a_disk_image_is_not_internal() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };

            let internal = resources(image.mount_point()).unwrap().internal;

            // The system gives no value for a disk image (`None`); it never calls one internal.
            assert!(matches!(internal, None | Some(false)), "a disk image reported internal: {internal:?}");
        }
    }
}
