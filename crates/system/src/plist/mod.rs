//! A reader of the string values of the top-level dictionary of a property list, in the XML form and the binary
//! form (`bplist00`).
//!
//! Only string values are returned; every other value kind is skipped. A string, key or value, that cannot be
//! decoded (non-ASCII bytes in a binary ASCII string, invalid UTF-16) drops its entry instead of failing the read.

mod binary;
mod xml;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

/// The largest property list read: 1 MiB.
pub(crate) const MAX_PLIST: usize = 1024 * 1024;

const BINARY_MAGIC: &[u8] = b"bplist00";
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const XML_STARTS: [&[u8]; 3] = [b"<?xml", b"<plist", b"<!DOCTYPE"];

/// Why a property list could not be read.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum PlistError {
    /// The input is larger than [`MAX_PLIST`].
    #[error("the property list is larger than {MAX_PLIST} bytes")]
    TooLarge,
    /// The input is neither an XML nor a binary property list.
    #[error("the input is not a property list")]
    NotAPlist,
    /// The input claims to be a property list but breaks its format.
    #[error("the property list is malformed: {0}")]
    Malformed(&'static str),
    /// The top-level object is not a dictionary.
    #[error("the top-level object of the property list is not a dictionary")]
    NotADictionary,
}

/// Reads the entries of the top-level dictionary of a property list whose value is a string.
///
/// The form is detected from the first bytes: `bplist00` is the binary form; `<?xml`, `<plist` or `<!DOCTYPE`, after
/// an optional UTF-8 byte order mark and whitespace, is the XML form. When a key repeats, the last string wins.
pub(crate) fn read_strings(bytes: &[u8]) -> Result<BTreeMap<String, String>, PlistError> {
    if bytes.len() > MAX_PLIST {
        return Err(PlistError::TooLarge);
    }
    if bytes.starts_with(BINARY_MAGIC) {
        return binary::read_strings(bytes);
    }
    let text = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let start = text.iter().position(|byte| !byte.is_ascii_whitespace()).unwrap_or(text.len());
    let rest = text.get(start..).unwrap_or_default();
    if XML_STARTS.iter().any(|prefix| rest.starts_with(prefix)) {
        xml::read_strings(rest)
    } else {
        Err(PlistError::NotAPlist)
    }
}
