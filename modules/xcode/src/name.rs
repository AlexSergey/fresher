//! The names the module reads (design, section 3), on plain values: a device support folder's
//! `[<model> ]<version> (<build>)` and an archive's `<date>/<name>.xcarchive`. A name of another shape reads as
//! nothing, and its folder is not claimed.

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

/// The suffix of an archive folder, ASCII case ignored.
pub(crate) const ARCHIVE_SUFFIX: &str = ".xcarchive";

/// A system version of two or three numbers, compared by its numbers with trailing zeros ignored: `17.10` is above
/// `17.9`, `17.2.1` above `17.2`, and `17.2.0` is the same version as `17.2`. It shows as it was written, in numbers.
#[derive(Clone, Debug)]
pub(crate) struct Version(Vec<u32>);

impl Version {
    /// The numbers without their trailing zeros.
    fn significant(&self) -> &[u32] {
        let end = self.0.iter().rposition(|&number| number != 0).map_or(0, |last| last + 1);
        &self.0[..end]
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.significant() == other.significant()
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.significant().cmp(other.significant())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, number) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(".")?;
            }
            write!(f, "{number}")?;
        }
        Ok(())
    }
}

/// The name of a device support folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeviceSupportName {
    /// The device model the folder was copied from (`iPhone15,2`), when the name starts with one.
    pub(crate) model: Option<String>,
    pub(crate) version: Version,
    pub(crate) build: String,
}

/// The name of an archive and the date of the folder it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveName {
    /// The folder's name without `.xcarchive`.
    pub(crate) name: String,
    /// The parent folder's name, `YYYY-MM-DD`.
    pub(crate) date: String,
}

/// `name` read as `[<model> ]<version> (<build>)`: valid UTF-8; `<version>` two or three dot-separated runs of ASCII
/// digits, each fitting a `u32`; `<build>` one or more ASCII letters or digits; `<model>` one word of ASCII letters,
/// digits and commas followed by exactly one space; nothing before or after.
pub(crate) fn device_support(name: &OsStr) -> Option<DeviceSupportName> {
    let name = name.to_str()?;
    let (head, build) = name.strip_suffix(')')?.rsplit_once(" (")?;
    if build.is_empty() || !build.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return None;
    }
    let (model, version) = match head.split_once(' ') {
        Some((model, version)) => (Some(model), version),
        None => (None, head),
    };
    if let Some(model) = model
        && (model.is_empty() || !model.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b','))
    {
        return None;
    }
    Some(DeviceSupportName { model: model.map(str::to_owned), version: version_of(version)?, build: build.to_owned() })
}

/// `path` read as an archive: its parent's name is a date `YYYY-MM-DD` (month 01 to 12, day 01 to 31), and its own
/// name ends with `.xcarchive`, ASCII case ignored, with at least one character before it.
pub(crate) fn archive(path: &Path) -> Option<ArchiveName> {
    let name = path.file_name()?.to_str()?;
    let date = path.parent()?.file_name()?.to_str()?;
    if !is_date(date) {
        return None;
    }
    let (stem, suffix) = name.split_at_checked(name.len().checked_sub(ARCHIVE_SUFFIX.len())?)?;
    if stem.is_empty() || !suffix.eq_ignore_ascii_case(ARCHIVE_SUFFIX) {
        return None;
    }
    Some(ArchiveName { name: stem.to_owned(), date: date.to_owned() })
}

/// Two or three dot-separated runs of ASCII digits, each fitting a `u32`.
fn version_of(text: &str) -> Option<Version> {
    let numbers = text
        .split('.')
        .map(|part| {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            part.parse::<u32>().ok()
        })
        .collect::<Option<Vec<u32>>>()?;
    (2..=3).contains(&numbers.len()).then_some(Version(numbers))
}

/// `YYYY-MM-DD`: four, two and two ASCII digits, the month 01 to 12 and the day 01 to 31.
fn is_date(text: &str) -> bool {
    let &[y1, y2, y3, y4, b'-', m1, m2, b'-', d1, d2] = text.as_bytes() else { return false };
    if ![y1, y2, y3, y4, m1, m2, d1, d2].iter().all(u8::is_ascii_digit) {
        return false;
    }
    let number = |tens: u8, ones: u8| (tens - b'0') * 10 + (ones - b'0');
    (1..=12).contains(&number(m1, m2)) && (1..=31).contains(&number(d1, d2))
}

#[cfg(test)]
mod tests;
