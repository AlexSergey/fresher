//! The binary form (`bplist00`): a header, the objects, an offset table and a 32-byte trailer.
//!
//! Only the top-level dictionary is decoded, so nested containers are never followed and cycles cannot occur.

use std::collections::BTreeMap;

use super::{BINARY_MAGIC, PlistError};

const TRAILER_LEN: usize = 32;
const DICT: u8 = 0xD;
const INT: u8 = 0x1;
const ASCII_STRING: u8 = 0x5;
const UTF16_STRING: u8 = 0x6;
/// The length nibble that says the length follows as an integer object.
const LENGTH_FOLLOWS: u8 = 0xF;
/// The most bytes of string the keys and values of one dictionary may decode to, together. Many keys may refer to one
/// shared string object, so without a bound a 1 MiB file could decode to gigabytes; a real property list is far below.
const MAX_DECODED: usize = 4 * super::MAX_PLIST;

pub(super) fn read_strings(bytes: &[u8]) -> Result<BTreeMap<String, String>, PlistError> {
    let document = Document::parse(bytes)?;
    let top = document.object_offset(document.top)?;
    let marker = document.object_byte(top)?;
    if marker >> 4 != DICT {
        return Err(PlistError::NotADictionary);
    }
    let (count, refs) = document.length(marker, top.saturating_add(1))?;
    let refs_len = count
        .checked_mul(2)
        .and_then(|refs| refs.checked_mul(document.ref_size))
        .ok_or(malformed("the dictionary is larger than the data"))?;
    document.objects(refs, refs_len).map_err(|_| malformed("the dictionary is larger than the data"))?;

    let mut map = BTreeMap::new();
    let mut budget = MAX_DECODED;
    for index in 0..count {
        let key_ref = document.object_ref(refs, index)?;
        let Some(key) = document.string(key_ref, &mut budget)? else { continue };
        let value_ref = document.object_ref(refs, count.saturating_add(index))?;
        if let Some(value) = document.string(value_ref, &mut budget)? {
            map.insert(key, value);
        }
    }
    Ok(map)
}

#[derive(Debug)]
struct Document<'a> {
    bytes: &'a [u8],
    offset_size: usize,
    ref_size: usize,
    num_objects: usize,
    top: usize,
    /// Where the offset table starts; every object lies before it.
    table: usize,
}

impl<'a> Document<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Self, PlistError> {
        let trailer_start = bytes
            .len()
            .checked_sub(TRAILER_LEN)
            .filter(|&start| start > BINARY_MAGIC.len())
            .ok_or(malformed("the data is too short for the trailer"))?;
        let trailer = bytes.get(trailer_start..).unwrap_or_default();
        let field = |start: usize, len: usize| trailer.get(start..start.saturating_add(len)).map(be_uint);
        let offset_size = field(6, 1).ok_or(malformed("a truncated trailer"))?;
        let ref_size = field(7, 1).ok_or(malformed("a truncated trailer"))?;
        let num_objects = field(8, 8).ok_or(malformed("a truncated trailer"))?;
        let top = field(16, 8).ok_or(malformed("a truncated trailer"))?;
        let table = field(24, 8).ok_or(malformed("a truncated trailer"))?;

        if !(1..=8).contains(&offset_size) || !(1..=8).contains(&ref_size) {
            return Err(malformed("an offset or reference size outside 1 to 8"));
        }
        let num_objects = to_usize(num_objects)?;
        if num_objects == 0 || num_objects > trailer_start {
            return Err(malformed("an object count that does not fit the data"));
        }
        let top = to_usize(top)?;
        if top >= num_objects {
            return Err(malformed("the top object is beyond the object count"));
        }
        let table = to_usize(table)?;
        let table_end = num_objects
            .checked_mul(to_usize(offset_size)?)
            .and_then(|len| len.checked_add(table))
            .ok_or(malformed("the offset table is outside the data"))?;
        if table <= BINARY_MAGIC.len() || table_end > trailer_start {
            return Err(malformed("the offset table is outside the data"));
        }
        Ok(Self { bytes, offset_size: to_usize(offset_size)?, ref_size: to_usize(ref_size)?, num_objects, top, table })
    }

    /// The position of an object, checked to lie between the header and the offset table.
    fn object_offset(&self, object: usize) -> Result<usize, PlistError> {
        let entry = object
            .checked_mul(self.offset_size)
            .and_then(|start| start.checked_add(self.table))
            .ok_or(malformed("an object is beyond the offset table"))?;
        let entry_end = entry.checked_add(self.offset_size).ok_or(malformed("an object is beyond the offset table"))?;
        let offset = self.bytes.get(entry..entry_end).map(be_uint).ok_or(malformed("an object is beyond the table"))?;
        let offset = to_usize(offset)?;
        if offset < BINARY_MAGIC.len() || offset >= self.table {
            return Err(malformed("an object offset is outside the data"));
        }
        Ok(offset)
    }

    /// The `index`-th object reference of a list starting at `refs`.
    fn object_ref(&self, refs: usize, index: usize) -> Result<usize, PlistError> {
        let start = index
            .checked_mul(self.ref_size)
            .and_then(|offset| offset.checked_add(refs))
            .ok_or(malformed("a reference is outside the data"))?;
        let object = to_usize(be_uint(self.objects(start, self.ref_size)?))?;
        if object >= self.num_objects {
            return Err(malformed("a reference is beyond the object count"));
        }
        Ok(object)
    }

    /// The length of the object whose marker is `marker`, and where its content starts.
    fn length(&self, marker: u8, after_marker: usize) -> Result<(usize, usize), PlistError> {
        let nibble = marker & 0x0F;
        if nibble != LENGTH_FOLLOWS {
            return Ok((usize::from(nibble), after_marker));
        }
        let int_marker = self.object_byte(after_marker)?;
        if int_marker >> 4 != INT {
            return Err(malformed("a length that is not an integer"));
        }
        let size = 1_usize << (int_marker & 0x0F);
        if size > 8 {
            return Err(malformed("a length wider than 8 bytes"));
        }
        let start = after_marker.saturating_add(1);
        let length = to_usize(be_uint(self.objects(start, size)?))?;
        Ok((length, start.saturating_add(size)))
    }

    /// The text of a string object; `None` for any other kind, or for a string that cannot be decoded. Its bytes are
    /// taken from `budget` before it is decoded; a string past what is left fails the read.
    fn string(&self, object: usize, budget: &mut usize) -> Result<Option<String>, PlistError> {
        let offset = self.object_offset(object)?;
        let marker = self.object_byte(offset)?;
        let kind = marker >> 4;
        if kind != ASCII_STRING && kind != UTF16_STRING {
            return Ok(None);
        }
        let (length, start) = self.length(marker, offset.saturating_add(1))?;
        let len = if kind == ASCII_STRING { Some(length) } else { length.checked_mul(2) };
        let len = len.ok_or(malformed("a string is larger than the data"))?;
        *budget = budget.checked_sub(len).ok_or(malformed("the strings decode to far more than the data"))?;
        if kind == ASCII_STRING {
            let ascii = self.objects(start, len)?;
            return Ok(ascii.is_ascii().then(|| ascii.iter().copied().map(char::from).collect()));
        }
        let units: Vec<u16> =
            self.objects(start, len)?.as_chunks::<2>().0.iter().copied().map(u16::from_be_bytes).collect();
        Ok(String::from_utf16(&units).ok())
    }

    fn object_byte(&self, at: usize) -> Result<u8, PlistError> {
        self.objects(at, 1)?.first().copied().ok_or(malformed("an object is outside the data"))
    }

    /// `len` bytes at `start`, checked to lie before the offset table.
    fn objects(&self, start: usize, len: usize) -> Result<&'a [u8], PlistError> {
        let end = start
            .checked_add(len)
            .filter(|&end| end <= self.table)
            .ok_or(malformed("an object is outside the data"))?;
        self.bytes.get(start..end).ok_or(malformed("an object is outside the data"))
    }
}

/// A big-endian unsigned integer of at most 8 bytes.
fn be_uint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |value, &byte| (value << 8) | u64::from(byte))
}

fn to_usize(value: u64) -> Result<usize, PlistError> {
    usize::try_from(value).map_err(|_| malformed("a number does not fit in memory"))
}

fn malformed(reason: &'static str) -> PlistError {
    PlistError::Malformed(reason)
}
