//! The head of the body: what the state is of and when it was taken. A rescan compares it with the scan it is asked
//! for; any difference means a full scan.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use fresher_core::{JournalPosition, Timestamp, VolumeId};

use crate::codec::CodecError;
use crate::codec::varint::{Reader, write_u64};

/// The version of the tree format this build writes and reads; another version is refused (the cache is rebuilt).
/// 2: every node carries its access (plan 006/05/01).
pub(crate) const TREE_FORMAT: u16 = 2;

/// What a saved tree is of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Header {
    /// When the state was taken.
    pub(crate) taken: Timestamp,
    pub(crate) roots: Vec<RootRecord>,
    /// The folders the walk did not open.
    pub(crate) avoid: Vec<PathBuf>,
    /// The fingerprint of the compiled interest set.
    pub(crate) fingerprint: u64,
    /// The category ids of the claims; a node's owner is an index into it.
    pub(crate) claims: Vec<String>,
}

/// One scan root: its canonical path, its volume and where the volume's journal stood before the walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RootRecord {
    pub(crate) path: PathBuf,
    pub(crate) volume: VolumeId,
    /// `None` for a volume that keeps no journal.
    pub(crate) journal: Option<JournalPosition>,
}

impl Header {
    pub(crate) fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&TREE_FORMAT.to_le_bytes());
        write_u64(out, self.taken.secs());
        write_len(out, self.roots.len());
        for root in &self.roots {
            write_bytes(out, root.path.as_os_str().as_bytes());
            out.extend_from_slice(&root.volume.get().to_le_bytes());
            match root.journal {
                None => out.push(0),
                Some(position) => {
                    out.push(1);
                    out.extend_from_slice(&position.journal.to_le_bytes());
                    write_u64(out, position.cursor);
                }
            }
        }
        write_len(out, self.avoid.len());
        for path in &self.avoid {
            write_bytes(out, path.as_os_str().as_bytes());
        }
        out.extend_from_slice(&self.fingerprint.to_le_bytes());
        write_len(out, self.claims.len());
        for claim in &self.claims {
            write_bytes(out, claim.as_bytes());
        }
    }

    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, CodecError> {
        let format = u16::from_le_bytes(reader.array()?);
        if format != TREE_FORMAT {
            return Err(CodecError::OtherFormat { found: format });
        }
        let taken = Timestamp::from_secs(reader.u64()?);
        let count = reader.count(18)?;
        let mut roots = Vec::with_capacity(count);
        for _ in 0..count {
            let path = read_path(reader)?;
            let volume = VolumeId::new(u128::from_le_bytes(reader.array()?));
            let journal = match reader.array::<1>()? {
                [0] => None,
                [1] => {
                    let journal = u128::from_le_bytes(reader.array()?);
                    Some(JournalPosition { journal, cursor: reader.u64()? })
                }
                _ => return Err(CodecError::OutOfRange),
            };
            roots.push(RootRecord { path, volume, journal });
        }
        let count = reader.count(1)?;
        let mut avoid = Vec::with_capacity(count);
        for _ in 0..count {
            avoid.push(read_path(reader)?);
        }
        let fingerprint = u64::from_le_bytes(reader.array()?);
        let count = reader.count(1)?;
        let mut claims = Vec::with_capacity(count);
        for _ in 0..count {
            let bytes = read_bytes(reader)?;
            claims.push(String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::BadName)?);
        }
        Ok(Self { taken, roots, avoid, fingerprint, claims })
    }
}

pub(crate) fn write_len(out: &mut Vec<u8>, len: usize) {
    write_u64(out, u64::try_from(len).unwrap_or(u64::MAX));
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    write_len(out, bytes.len());
    out.extend_from_slice(bytes);
}

fn read_bytes<'a>(reader: &mut Reader<'a>) -> Result<&'a [u8], CodecError> {
    let len = reader.count(1)?;
    reader.bytes(len)
}

/// A path of the header: always absolute.
fn read_path(reader: &mut Reader<'_>) -> Result<PathBuf, CodecError> {
    let path = Path::new(OsStr::from_bytes(read_bytes(reader)?));
    if path.is_absolute() { Ok(path.to_path_buf()) } else { Err(CodecError::BadName) }
}

#[cfg(test)]
pub(crate) mod testing {
    use std::path::PathBuf;

    use fresher_core::{JournalPosition, Timestamp, VolumeId};

    use crate::codec::header::{Header, RootRecord};

    /// A header with one root of each kind of journal and three claims.
    pub(crate) fn header() -> Header {
        Header {
            taken: Timestamp::from_secs(1_763_251_200),
            roots: vec![
                RootRecord {
                    path: PathBuf::from("/home/user"),
                    volume: VolumeId::new(0x1234_5678_9abc_def0_1234_5678_9abc_def0),
                    journal: Some(JournalPosition { journal: 7, cursor: 1_000_000 }),
                },
                RootRecord { path: PathBuf::from("/Volumes/Data"), volume: VolumeId::new(9), journal: None },
            ],
            avoid: vec![PathBuf::from("/home/user/Documents")],
            fingerprint: 0xfeed_beef_dead_cafe,
            claims: vec!["node-modules.stale".to_owned(), "xcode.derived-data".to_owned(), "rust.target".to_owned()],
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::codec::CodecError;
        use crate::codec::header::testing::header;
        use crate::codec::header::{Header, TREE_FORMAT};
        use crate::codec::varint::Reader;

        #[test]
        fn another_tree_format_is_refused() {
            let mut out = Vec::new();
            header().write(&mut out);
            out[0] = u8::try_from(TREE_FORMAT + 1).unwrap();
            assert_eq!(Header::read(&mut Reader::new(&out)), Err(CodecError::OtherFormat { found: TREE_FORMAT + 1 }));
        }

        #[test]
        fn a_relative_root_is_refused() {
            let mut bad = header();
            bad.roots[0].path = "home".into();
            let mut out = Vec::new();
            bad.write(&mut out);
            assert_eq!(Header::read(&mut Reader::new(&out)), Err(CodecError::BadName));
        }

        #[test]
        fn a_header_cut_anywhere_is_refused() {
            let mut out = Vec::new();
            header().write(&mut out);
            for len in 0..out.len() {
                assert!(Header::read(&mut Reader::new(&out[..len])).is_err(), "{len}");
            }
        }
    }

    mod positive {
        use crate::codec::header::Header;
        use crate::codec::header::testing::header;
        use crate::codec::varint::Reader;

        #[test]
        fn the_header_round_trips() {
            let mut out = Vec::new();
            header().write(&mut out);
            let mut reader = Reader::new(&out);
            assert_eq!(Header::read(&mut reader), Ok(header()));
            assert_eq!(reader.remaining(), 0);
        }
    }
}
