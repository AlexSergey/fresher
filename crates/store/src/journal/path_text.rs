//! A path as the journal stores it: a string when it is UTF-8, `{"bytes":"<hex>"}` otherwise, read back to the same
//! bytes.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A path of the user's own item, kept exactly as it is on the disk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "PathText", try_from = "PathText")]
pub(super) struct StoredPath(pub(super) PathBuf);

impl From<&Path> for StoredPath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

/// The two forms a path takes in the file.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PathText {
    Text(String),
    Bytes { bytes: String },
}

/// A `bytes` form that is not an even run of hexadecimal digits.
#[derive(Debug, thiserror::Error)]
#[error("a path's bytes are not hexadecimal")]
pub(super) struct NotHex;

impl From<StoredPath> for PathText {
    fn from(path: StoredPath) -> Self {
        match path.0.into_os_string().into_string() {
            Ok(text) => Self::Text(text),
            Err(raw) => Self::Bytes { bytes: hex(raw.as_bytes()) },
        }
    }
}

impl TryFrom<PathText> for StoredPath {
    type Error = NotHex;

    fn try_from(text: PathText) -> Result<Self, NotHex> {
        match text {
            PathText::Text(text) => Ok(Self(PathBuf::from(text))),
            PathText::Bytes { bytes } => Ok(Self(PathBuf::from(OsStr::from_bytes(&unhex(&bytes)?)))),
        }
    }
}

const DIGITS: &[u8; 16] = b"0123456789abcdef";

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn unhex(text: &str) -> Result<Vec<u8>, NotHex> {
    let digit = |byte: u8| char::from(byte).to_digit(16).and_then(|value| u8::try_from(value).ok()).ok_or(NotHex);
    text.as_bytes()
        .chunks(2)
        .map(|pair| match pair {
            [high, low] => Ok(digit(*high)? << 4 | digit(*low)?),
            _ => Err(NotHex),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        use crate::journal::path_text::StoredPath;

        #[test]
        fn bytes_that_are_not_hexadecimal_are_refused() {
            for text in [r#"{"bytes":"abc"}"#, r#"{"bytes":"zz"}"#, r#"{"bytes":"+1"}"#, "7", "null"] {
                assert!(serde_json::from_str::<StoredPath>(text).is_err(), "{text}");
            }
        }

        #[test]
        fn a_path_that_is_not_utf8_is_written_as_its_bytes() {
            let path = StoredPath(OsStr::from_bytes(b"/tmp/n\xe9e").into());

            assert_eq!(serde_json::to_string(&path).unwrap(), r#"{"bytes":"2f746d702f6ee965"}"#);
        }
    }

    mod positive {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        use proptest::prelude::*;

        use crate::journal::path_text::StoredPath;

        #[test]
        fn a_utf8_path_is_written_as_a_string() {
            let path = StoredPath(PathBuf::from("/home/Library/Caches/Élan"));

            assert_eq!(serde_json::to_string(&path).unwrap(), r#""/home/Library/Caches/Élan""#);
        }

        proptest! {
            #[test]
            fn a_non_utf8_path_round_trips(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
                let path = StoredPath(PathBuf::from(OsStr::from_bytes(&bytes)));

                let read: StoredPath = serde_json::from_str(&serde_json::to_string(&path).unwrap()).unwrap();

                prop_assert_eq!(read.0.as_os_str().as_bytes(), &bytes[..]);
            }
        }
    }
}
