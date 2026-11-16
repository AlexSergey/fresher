//! The numbers of the body: unsigned LEB128 for `u64` (seven bits a byte, low bits first) and a zig-zag form for
//! signed values, so a small negative number stays short. A reader never reads past its slice and refuses a number
//! written with more bytes than it needs.

use crate::codec::CodecError;

/// The most bytes a `u64` takes.
const MAX_LEN: usize = 10;

pub(crate) fn write_u64(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push(low_byte(value) | 0x80);
        value >>= 7;
    }
    out.push(low_byte(value));
}

/// The low seven bits of `value`.
const fn low_byte(value: u64) -> u8 {
    (value & 0x7f).to_le_bytes()[0]
}

pub(crate) fn write_i64(out: &mut Vec<u8>, value: i64) {
    write_u64(out, zigzag(value));
}

/// The number of bytes `value` takes.
#[cfg(test)]
pub(crate) const fn len_u64(value: u64) -> usize {
    let bits = 64 - value.leading_zeros() as usize;
    if bits == 0 { 1 } else { bits.div_ceil(7) }
}

const fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)).cast_unsigned()
}

const fn unzigzag(value: u64) -> i64 {
    (value >> 1).cast_signed() ^ -((value & 1).cast_signed())
}

/// Reads the body from the front; every read checks the bytes left.
#[derive(Clone, Debug)]
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub(crate) const fn remaining(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn u64(&mut self) -> Result<u64, CodecError> {
        let mut value = 0_u64;
        let mut shift = 0_u32;
        for (position, &byte) in self.bytes.iter().enumerate().take(MAX_LEN) {
            let part = u64::from(byte & 0x7f);
            if position == MAX_LEN - 1 && part > 1 {
                return Err(CodecError::OverlongNumber);
            }
            value |= part << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if position > 0 && byte == 0 {
                    return Err(CodecError::OverlongNumber);
                }
                self.bytes = self.bytes.get(position + 1..).unwrap_or_default();
                return Ok(value);
            }
        }
        if self.bytes.len() >= MAX_LEN { Err(CodecError::OverlongNumber) } else { Err(CodecError::Truncated) }
    }

    pub(crate) fn i64(&mut self) -> Result<i64, CodecError> {
        self.u64().map(unzigzag)
    }

    pub(crate) fn u32(&mut self) -> Result<u32, CodecError> {
        u32::try_from(self.u64()?).map_err(|_| CodecError::OutOfRange)
    }

    pub(crate) fn u16(&mut self) -> Result<u16, CodecError> {
        u16::try_from(self.u64()?).map_err(|_| CodecError::OutOfRange)
    }

    /// A count of items that each take at least `min_bytes`: never more than the bytes left can hold, so a broken
    /// count cannot make a load allocate more than the body.
    pub(crate) fn count(&mut self, min_bytes: usize) -> Result<usize, CodecError> {
        let count = usize::try_from(self.u64()?).map_err(|_| CodecError::CountTooLarge)?;
        if count.saturating_mul(min_bytes) > self.remaining() {
            return Err(CodecError::CountTooLarge);
        }
        Ok(count)
    }

    pub(crate) fn bytes(&mut self, len: usize) -> Result<&'a [u8], CodecError> {
        let (taken, rest) = self.bytes.split_at_checked(len).ok_or(CodecError::Truncated)?;
        self.bytes = rest;
        Ok(taken)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        self.bytes(N)?.try_into().map_err(|_| CodecError::Truncated)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::codec::CodecError;
        use crate::codec::varint::Reader;

        #[test]
        fn an_overlong_varint_is_refused() {
            assert_eq!(Reader::new(&[0x80, 0x00]).u64(), Err(CodecError::OverlongNumber));
            assert_eq!(Reader::new(&[0xff; 11]).u64(), Err(CodecError::OverlongNumber));
            let mut past_64_bits = vec![0xff; 9];
            past_64_bits.push(0x02);
            assert_eq!(Reader::new(&past_64_bits).u64(), Err(CodecError::OverlongNumber));
        }

        #[test]
        fn a_cut_number_is_truncated() {
            assert_eq!(Reader::new(&[]).u64(), Err(CodecError::Truncated));
            assert_eq!(Reader::new(&[0x80, 0x80]).u64(), Err(CodecError::Truncated));
        }

        #[test]
        fn a_count_past_the_bytes_left_is_refused() {
            assert_eq!(Reader::new(&[3, 0, 0]).count(1), Err(CodecError::CountTooLarge));
            assert_eq!(Reader::new(&[0xff, 0xff, 0xff, 0xff, 0x0f]).u16(), Err(CodecError::OutOfRange));
        }
    }

    mod positive {
        use crate::codec::varint::{Reader, len_u64, write_i64, write_u64};

        #[test]
        fn every_width_round_trips() {
            for value in [0, 1, 127, 128, 16_383, 16_384, u64::from(u32::MAX), u64::MAX - 1, u64::MAX] {
                let mut out = Vec::new();
                write_u64(&mut out, value);
                assert_eq!(out.len(), len_u64(value));
                let mut reader = Reader::new(&out);
                assert_eq!(reader.u64(), Ok(value));
                assert_eq!(reader.remaining(), 0);
            }
        }

        #[test]
        fn small_signed_values_stay_short() {
            for value in [0, -1, 1, -64, 63, i64::MIN, i64::MAX] {
                let mut out = Vec::new();
                write_i64(&mut out, value);
                assert_eq!(Reader::new(&out).i64(), Ok(value));
            }
            let mut out = Vec::new();
            write_i64(&mut out, -64);
            assert_eq!(out.len(), 1);
        }
    }
}
