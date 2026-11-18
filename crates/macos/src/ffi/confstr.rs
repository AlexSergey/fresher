//! The system's configuration strings (`confstr`).

use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

/// The longest answer the first call takes; a longer one is read again into a buffer of its length.
const FIRST_LENGTH: usize = 1024;

/// The user's own temporary folder as the system's configuration names it (`_CS_DARWIN_USER_TEMP_DIR`), never the
/// `TMPDIR` variable, which a shell or a launch agent may change. As the system writes it: through the link `/var`,
/// with a trailing `/`; the caller makes it canonical.
pub fn user_temp_dir() -> io::Result<PathBuf> {
    let mut buffer = vec![0_u8; FIRST_LENGTH];
    let mut needed = read(libc::_CS_DARWIN_USER_TEMP_DIR, &mut buffer)?;
    if needed > buffer.len() {
        buffer = vec![0_u8; needed];
        needed = read(libc::_CS_DARWIN_USER_TEMP_DIR, &mut buffer)?;
        if needed > buffer.len() {
            return Err(io::Error::other("the configuration string grew between two reads"));
        }
    }
    // `needed` counts the terminating NUL.
    let value = buffer.get(..needed.saturating_sub(1)).unwrap_or_default();
    if value.is_empty() || value.contains(&0) {
        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }
    Ok(PathBuf::from(OsStr::from_bytes(value)))
}

/// One `confstr` call into `buffer`: the length the whole value needs, its NUL included. A name with no value, or
/// one the system does not know, is an error of its own: `errno` is not read, since it may be left from before.
fn read(name: libc::c_int, buffer: &mut [u8]) -> io::Result<usize> {
    // SAFETY: `buffer` is valid and writable for `buffer.len()` bytes, the length passed; `confstr` writes at most
    // that many, NUL included.
    let needed = unsafe { libc::confstr(name, buffer.as_mut_ptr().cast(), buffer.len()) };
    if needed == 0 { Err(io::Error::other("the configuration string has no value")) } else { Ok(needed) }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::ffi::confstr::read;

        #[test]
        fn an_unknown_name_is_an_error() {
            let mut buffer = [0_u8; 16];

            assert!(read(-1, &mut buffer).is_err());
        }
    }

    mod positive {
        use std::os::unix::ffi::OsStrExt as _;

        use crate::ffi::confstr::user_temp_dir;

        #[test]
        fn the_users_temporary_folder_is_an_absolute_path_ending_in_t() {
            let folder = user_temp_dir().unwrap();

            assert!(folder.is_absolute(), "{}", folder.display());
            assert!(folder.as_os_str().as_bytes().ends_with(b"/T/"), "{}", folder.display());
        }
    }
}
