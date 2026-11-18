//! The stamp of a node with hits: its status change and its access as deltas from its modification time, then one
//! byte for its inside (bit 0 present, 1 special, 2 foreign, 3 complete, 4 a lock file's name) and, when present, its
//! newest change and its newest file access as deltas from the same time. A time is `i64::MIN` for none, as a node
//! keeps it.

use fresher_core::{Inside, Timestamp};

use crate::codec::CodecError;
use crate::codec::varint::{Reader, write_i64};
use crate::tree::node::{time_bits, time_of};
use crate::tree::times::{Stamp, Times};

const PRESENT: u8 = 1;
const SPECIAL: u8 = 1 << 1;
const FOREIGN: u8 = 1 << 2;
const COMPLETE: u8 = 1 << 3;
const LOCK_NAMED: u8 = 1 << 4;
const KNOWN: u8 = PRESENT | SPECIAL | FOREIGN | COMPLETE | LOCK_NAMED;

/// Writes `stamp` of a node whose modification time is `base` (as the node keeps it).
pub(super) fn write_stamp(out: &mut Vec<u8>, base: i64, stamp: &Stamp) {
    write_time(out, base, stamp.times.changed);
    write_time(out, base, stamp.times.accessed);
    let Some(inside) = stamp.inside else {
        out.push(0);
        return;
    };
    let mut bits = PRESENT;
    let flags = [
        (inside.special, SPECIAL),
        (inside.foreign, FOREIGN),
        (inside.complete, COMPLETE),
        (inside.lock_named, LOCK_NAMED),
    ];
    for (set, bit) in flags {
        if set {
            bits |= bit;
        }
    }
    out.push(bits);
    write_time(out, base, inside.newest_change);
    write_time(out, base, inside.newest_file_access);
}

/// Reads the stamp of a node whose modification time is `base`.
pub(super) fn read_stamp(reader: &mut Reader<'_>, base: i64) -> Result<Stamp, CodecError> {
    let times = Times { changed: read_time(reader, base)?, accessed: read_time(reader, base)? };
    let [bits] = reader.array::<1>()?;
    if bits & !KNOWN != 0 || bits & PRESENT == 0 && bits != 0 {
        return Err(CodecError::BadFlags);
    }
    let inside = if bits & PRESENT == 0 {
        None
    } else {
        Some(Inside {
            newest_change: read_time(reader, base)?,
            newest_file_access: read_time(reader, base)?,
            special: bits & SPECIAL != 0,
            foreign: bits & FOREIGN != 0,
            lock_named: bits & LOCK_NAMED != 0,
            complete: bits & COMPLETE != 0,
        })
    };
    Ok(Stamp { times, inside })
}

fn write_time(out: &mut Vec<u8>, base: i64, time: Option<Timestamp>) {
    write_i64(out, time_bits(time).wrapping_sub(base));
}

/// A time read back: none, or seconds since the epoch; anything else is out of range.
fn read_time(reader: &mut Reader<'_>, base: i64) -> Result<Option<Timestamp>, CodecError> {
    let bits = base.wrapping_add(reader.i64()?);
    if bits < 0 && bits != i64::MIN {
        return Err(CodecError::OutOfRange);
    }
    Ok(time_of(bits))
}

#[cfg(test)]
mod tests {
    use fresher_core::{Inside, Timestamp};

    use crate::tree::times::{Stamp, Times};

    fn stamp() -> Stamp {
        let at = |secs| Some(Timestamp::from_secs(secs));
        Stamp {
            times: Times { changed: at(1_700_000_100), accessed: None },
            inside: Some(Inside { newest_change: at(1_700_000_900), foreign: true, lock_named: true, ..Inside::EMPTY }),
        }
    }

    mod negative {
        use super::stamp;
        use crate::codec::CodecError;
        use crate::codec::stamp::{read_stamp, write_stamp};
        use crate::codec::varint::{Reader, write_i64};

        #[test]
        fn unknown_inside_bits_are_refused() {
            for bits in [0b10_0001, 0b0110, 0x80] {
                let mut out = Vec::new();
                write_i64(&mut out, 0);
                write_i64(&mut out, 0);
                out.push(bits);

                assert_eq!(read_stamp(&mut Reader::new(&out), 0), Err(CodecError::BadFlags), "{bits:#b}");
            }
        }

        #[test]
        fn a_negative_time_is_out_of_range() {
            let mut out = Vec::new();
            write_i64(&mut out, -5);

            assert_eq!(read_stamp(&mut Reader::new(&out), 0), Err(CodecError::OutOfRange));
        }

        #[test]
        fn a_stamp_cut_short_is_truncated() {
            let mut out = Vec::new();
            write_stamp(&mut out, 1_700_000_000, &stamp());
            out.pop();

            assert_eq!(read_stamp(&mut Reader::new(&out), 1_700_000_000), Err(CodecError::Truncated));
        }
    }

    mod positive {
        use super::stamp;
        use crate::codec::stamp::{read_stamp, write_stamp};
        use crate::codec::varint::Reader;
        use crate::tree::node::time_bits;
        use crate::tree::times::Stamp;

        #[test]
        fn a_stamp_round_trips_against_any_base() {
            for base in [time_bits(None), 0, 1_700_000_000] {
                for written in [stamp(), Stamp::default()] {
                    let mut out = Vec::new();
                    write_stamp(&mut out, base, &written);
                    let mut reader = Reader::new(&out);

                    assert_eq!(read_stamp(&mut reader, base), Ok(written), "{base}");
                    assert_eq!(reader.remaining(), 0);
                }
            }
        }
    }
}
