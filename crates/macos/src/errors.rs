//! The system's error numbers in the core's words.

use std::io;

use fresher_core::ReadError;

/// What an error of the system means for a read: `EPERM` is the privacy protection, `EACCES` the POSIX permissions.
pub(crate) fn read_error(error: &io::Error) -> ReadError {
    error.raw_os_error().map_or(ReadError::Failed, from_errno)
}

pub(crate) const fn from_errno(errno: i32) -> ReadError {
    match errno {
        libc::EPERM => ReadError::Guarded,
        libc::EACCES => ReadError::NoPermission,
        libc::ENOENT | libc::ENOTDIR => ReadError::Vanished,
        libc::ELOOP => ReadError::Replaced,
        libc::EINTR => ReadError::Interrupted,
        _ => ReadError::Failed,
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::io;

        use fresher_core::ReadError;

        use crate::errors::read_error;

        #[test]
        fn an_error_with_no_number_or_an_unknown_one_is_a_failure() {
            assert_eq!(read_error(&io::Error::from(io::ErrorKind::InvalidData)), ReadError::Failed);
            assert_eq!(read_error(&io::Error::from_raw_os_error(libc::EIO)), ReadError::Failed);
        }
    }

    mod positive {
        use std::io;

        use fresher_core::ReadError;

        use crate::errors::read_error;

        #[test]
        fn each_known_number_has_its_meaning() {
            for (errno, meaning) in [
                (libc::EPERM, ReadError::Guarded),
                (libc::EACCES, ReadError::NoPermission),
                (libc::ENOENT, ReadError::Vanished),
                (libc::ENOTDIR, ReadError::Vanished),
                (libc::ELOOP, ReadError::Replaced),
                (libc::EINTR, ReadError::Interrupted),
            ] {
                assert_eq!(read_error(&io::Error::from_raw_os_error(errno)), meaning, "{errno}");
            }
        }
    }
}
