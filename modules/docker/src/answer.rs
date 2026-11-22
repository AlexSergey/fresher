//! The runtime's answer to `docker system df --format json`, read: one JSON object per line, one line per kind of
//! what Docker keeps. Anything but the documented shape is unreadable, so a card never shows a guessed number.

use fresher_core::ByteSize;
use serde::Deserialize;

/// The units Docker prints sizes in, as powers of 1000.
const UNITS: [(&str, u64); 7] = [
    ("B", 1),
    ("kB", 1_000),
    ("KB", 1_000),
    ("MB", 1_000_000),
    ("GB", 1_000_000_000),
    ("TB", 1_000_000_000_000),
    ("PB", 1_000_000_000_000_000),
];

/// The longest answer read, in bytes: a real one is a few hundred.
const MAX_ANSWER: usize = 64 * 1024;

/// The most digits a size's number may hold: enough for any real answer, and the arithmetic never overflows.
const MAX_DIGITS: usize = 20;

/// Why an answer could not be read: a short reason for the log, never shown to the user.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[error("the runtime's answer is unreadable: {0}")]
pub(crate) struct Unreadable(pub(crate) &'static str);

/// One kind of what Docker keeps, as the runtime counts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Kind {
    /// What all of the kind takes.
    pub(crate) size: ByteSize,
    /// What of it nothing uses (Docker's `Reclaimable`); never more than `size`.
    pub(crate) unused: ByteSize,
    /// How many there are.
    pub(crate) count: u64,
    /// How many are in use; never more than `count`.
    pub(crate) active: u64,
}

impl Kind {
    /// How many are not in use.
    pub(crate) const fn unused_count(&self) -> u64 {
        self.count.saturating_sub(self.active)
    }
}

/// The four kinds of what Docker keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Usage {
    pub(crate) build_cache: Kind,
    pub(crate) images: Kind,
    pub(crate) containers: Kind,
    pub(crate) volumes: Kind,
}

/// The type of one line, read first: a line of another type is ignored whatever its other fields.
#[derive(Debug, Deserialize)]
struct Typed {
    #[serde(rename = "Type")]
    kind: String,
}

/// One line of a known type; fields the module does not use are allowed and ignored.
#[derive(Debug, Deserialize)]
struct Line {
    #[serde(rename = "TotalCount")]
    total: Option<String>,
    #[serde(rename = "Active")]
    active: Option<String>,
    #[serde(rename = "Size")]
    size: Option<String>,
    #[serde(rename = "Reclaimable")]
    reclaimable: Option<String>,
}

/// The answer in `stdout`, at most 64 KiB: every non-empty line an object with the string fields `Type`, `TotalCount`,
/// `Active`, `Size` and `Reclaimable`; `Images`, `Containers`, `Local Volumes` and `Build Cache` each exactly once, any
/// other type ignored.
pub(crate) fn parse(stdout: &str) -> Result<Usage, Unreadable> {
    if stdout.len() > MAX_ANSWER {
        return Err(Unreadable("the answer is too long"));
    }
    let (mut build_cache, mut images, mut containers, mut volumes) = (None, None, None, None);
    for text in stdout.lines().map(str::trim).filter(|text| !text.is_empty()) {
        let typed: Typed = serde_json::from_str(text).map_err(|_| Unreadable("a line is not an object with a type"))?;
        let slot = match typed.kind.as_str() {
            "Build Cache" => &mut build_cache,
            "Images" => &mut images,
            "Containers" => &mut containers,
            "Local Volumes" => &mut volumes,
            _ => continue,
        };
        if slot.is_some() {
            return Err(Unreadable("a kind is listed twice"));
        }
        let line: Line = serde_json::from_str(text).map_err(|_| Unreadable("a line is not an object of strings"))?;
        *slot = Some(kind(&line)?);
    }
    match (build_cache, images, containers, volumes) {
        (Some(build_cache), Some(images), Some(containers), Some(volumes)) => {
            Ok(Usage { build_cache, images, containers, volumes })
        }
        _ => Err(Unreadable("a kind is missing")),
    }
}

/// The kind one line states.
fn kind(line: &Line) -> Result<Kind, Unreadable> {
    let field = |value: &Option<String>| value.clone().ok_or(Unreadable("a field is missing"));
    let count = whole(&field(&line.total)?)?;
    let active = whole(&field(&line.active)?)?;
    let size = size(&field(&line.size)?)?;
    let reclaimable = field(&line.reclaimable)?;
    let unused = size_of_reclaimable(&reclaimable)?;
    if active > count {
        return Err(Unreadable("more are active than there are"));
    }
    if unused > size {
        return Err(Unreadable("more is unused than the whole"));
    }
    Ok(Kind { size, unused, count, active })
}

/// A whole number: ASCII digits only.
fn whole(text: &str) -> Result<u64, Unreadable> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Unreadable("a count is not a whole number"));
    }
    text.parse().map_err(|_| Unreadable("a count is too large"))
}

/// `Reclaimable`: a size, optionally followed by ` (N%)`, which is dropped.
fn size_of_reclaimable(text: &str) -> Result<ByteSize, Unreadable> {
    let Some((before, percent)) = text.split_once(" (") else { return size(text) };
    let number = percent.strip_suffix("%)").ok_or(Unreadable("a percentage does not read"))?;
    decimal(number).ok_or(Unreadable("a percentage does not read"))?;
    size(before)
}

/// A size: a decimal number and one of Docker's units, rounded to whole bytes.
pub(crate) fn size(text: &str) -> Result<ByteSize, Unreadable> {
    let split = text.find(|char: char| !char.is_ascii_digit() && char != '.').unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let multiplier = UNITS
        .iter()
        .find(|(name, _)| *name == unit)
        .map(|(_, multiplier)| *multiplier)
        .ok_or(Unreadable("a size's unit is not one of Docker's"))?;
    let (digits, decimals) = decimal(number).ok_or(Unreadable("a size's number does not read"))?;
    let scale = 10_u128.pow(decimals);
    let bytes = (digits * u128::from(multiplier) + scale / 2) / scale;
    u64::try_from(bytes).map(ByteSize::new).map_err(|_| Unreadable("a size is too large"))
}

/// A decimal number (`12`, `3.034`) as its digits without the point and how many of them follow the point.
fn decimal(text: &str) -> Option<(u128, u32)> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let plain = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !plain(whole) || !plain(fraction) || whole.len() + fraction.len() > MAX_DIGITS {
        return None;
    }
    if text.contains('.') && fraction.is_empty() {
        return None;
    }
    let digits = format!("{whole}{fraction}").parse().ok()?;
    Some((digits, u32::try_from(fraction.len()).ok()?))
}

#[cfg(test)]
mod tests;
