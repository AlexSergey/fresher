//! The records of a bulk read, parsed by safe code: every length and offset is checked, nothing panics.
//!
//! A record is its length (`u32`), the mask of the attributes it holds (`attribute_set_t`), then those attributes in
//! group order (common, folder, file, extended), each group in bit order, packed on 4 bytes (8-byte values may be
//! unaligned); `ATTR_CMN_ERROR` comes first when present. The next record starts `length` bytes after this one.

use fresher_core::{ByteSize, EntryAccess, EntryKind, RawEntry, ReadError, Sizes, Timestamp};

use crate::errors::from_errno;

const ATTR_CMN_NAME: u32 = 0x0000_0001;
const ATTR_CMN_DEVID: u32 = 0x0000_0002;
const ATTR_CMN_OBJTYPE: u32 = 0x0000_0008;
const ATTR_CMN_MODTIME: u32 = 0x0000_0400;
const ATTR_CMN_OWNERID: u32 = 0x0000_8000;
const ATTR_CMN_ACCESSMASK: u32 = 0x0002_0000;
const ATTR_CMN_FLAGS: u32 = 0x0004_0000;
const ATTR_CMN_FILEID: u32 = 0x0200_0000;
const ATTR_CMN_ERROR: u32 = 0x2000_0000;
const ATTR_CMN_RETURNED_ATTRS: u32 = 0x8000_0000;
const ATTR_FILE_LINKCOUNT: u32 = 0x0000_0001;
const ATTR_FILE_TOTALSIZE: u32 = 0x0000_0002;
const ATTR_FILE_ALLOCSIZE: u32 = 0x0000_0004;
const ATTR_CMNEXT_CLONEID: u32 = 0x0000_0100;
const ATTR_CMNEXT_EXT_FLAGS: u32 = 0x0000_0200;

/// The common attributes the bulk read asks for.
pub(super) const COMMON: u32 = ATTR_CMN_RETURNED_ATTRS
    | ATTR_CMN_NAME
    | ATTR_CMN_DEVID
    | ATTR_CMN_OBJTYPE
    | ATTR_CMN_MODTIME
    | ATTR_CMN_OWNERID
    | ATTR_CMN_ACCESSMASK
    | ATTR_CMN_FLAGS
    | ATTR_CMN_FILEID
    | ATTR_CMN_ERROR;
/// The file attributes the bulk read asks for (returned for files and links, not for folders).
pub(super) const FILE: u32 = ATTR_FILE_LINKCOUNT | ATTR_FILE_TOTALSIZE | ATTR_FILE_ALLOCSIZE;
/// The extended attributes the bulk read asks for: free to read (01). The private size is not among them: asking it
/// of every file doubled the walk, so it is read per clone.
pub(super) const EXTENDED: u32 = ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS;

/// `VREG`, `VDIR` and `VLNK` of `<sys/vnode.h>`.
const VREG: u32 = 1;
const VDIR: u32 = 2;
const VLNK: u32 = 5;
/// `SF_DATALESS` of `<sys/stat.h>`: the file's data is not on this Mac (a cloud file).
const SF_DATALESS: u32 = 0x4000_0000;
/// `UF_IMMUTABLE`, `SF_IMMUTABLE` and `SF_RESTRICTED` of `<sys/stat.h>`: locked by the user, locked by the system,
/// guarded by the system's integrity protection.
const UF_IMMUTABLE: u32 = 0x0000_0002;
const SF_IMMUTABLE: u32 = 0x0002_0000;
const SF_RESTRICTED: u32 = 0x0008_0000;
/// `UF_APPEND`, `SF_APPEND` and `SF_NOUNLINK`: the item cannot be removed either, so they count as the user's or the
/// system's lock (fail closed).
const UF_APPEND: u32 = 0x0000_0004;
const SF_APPEND: u32 = 0x0004_0000;
const SF_NOUNLINK: u32 = 0x0010_0000;
/// The permission bits of a mode.
const PERMISSIONS: u32 = 0o7777;
/// `EF_MAY_SHARE_BLOCKS` of `<sys/attr.h>`: the file may share its blocks with a clone.
const EF_MAY_SHARE_BLOCKS: u64 = 0x1;

/// The fixed part of a record: its length and the mask of the attributes it holds.
const HEADER: usize = 4 + 5 * 4;

/// One entry of a bulk read, as the kernel packed it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Record<'a> {
    pub(super) name: &'a [u8],
    error: u32,
    kind: Option<u32>,
    device: Option<i32>,
    modified: Option<i64>,
    owner: Option<u32>,
    mode: Option<u32>,
    flags: u32,
    file_id: Option<u64>,
    links: Option<u32>,
    logical: Option<u64>,
    allocated: Option<u64>,
    clone_id: Option<u64>,
    extended_flags: u64,
}

impl Record<'_> {
    /// A regular file that may share its blocks with a clone: its private size is worth one more call.
    pub(super) fn may_share_blocks(&self) -> bool {
        self.kind == Some(VREG) && self.extended_flags & EF_MAY_SHARE_BLOCKS != 0
    }

    fn kind(&self) -> EntryKind {
        match self.kind {
            Some(VDIR) => EntryKind::Folder,
            Some(VLNK) => EntryKind::Symlink,
            _ => EntryKind::File,
        }
    }

    /// The entry in the core's words; `private` is the private size of a file that may share its blocks.
    pub(super) fn entry(&self, private: Option<u64>) -> RawEntry {
        let kind = self.kind();
        let cloud_only = self.flags & SF_DATALESS != 0;
        let sizes = if kind == EntryKind::Folder {
            Sizes::ZERO
        } else if cloud_only {
            Sizes::complete(ByteSize::new(self.logical.unwrap_or(0)), ByteSize::ZERO)
        } else {
            let allocated = self.allocated.unwrap_or(0);
            let reclaimable = private.map_or(allocated, |private| private.min(allocated));
            Sizes::new(ByteSize::new(self.logical.unwrap_or(0)), ByteSize::new(allocated), ByteSize::new(reclaimable))
        };
        RawEntry {
            kind,
            device: self.device.map_or(0, device_id),
            file_id: self.file_id.unwrap_or(0),
            links: self.links.unwrap_or(1),
            sizes,
            clone: self.clone_id.filter(|_| self.may_share_blocks()),
            modified: self.modified.map(|secs| Timestamp::from_secs(u64::try_from(secs).unwrap_or(0))),
            cloud_only,
            access: access(self.owner, self.mode.unwrap_or(0), self.flags),
            error: (self.error != 0).then(|| from_errno(i32::try_from(self.error).unwrap_or(i32::MAX))),
        }
    }
}

/// The access of an entry owned by `owner`, of mode `mode` and with the flags `flags`; a mode not read allows nothing.
pub(crate) fn access(owner: Option<u32>, mode: u32, flags: u32) -> EntryAccess {
    EntryAccess {
        owner,
        mode: u16::try_from(mode & PERMISSIONS).unwrap_or(0),
        user_immutable: flags & (UF_IMMUTABLE | UF_APPEND) != 0,
        system_immutable: flags & (SF_IMMUTABLE | SF_APPEND | SF_NOUNLINK) != 0,
        restricted: flags & SF_RESTRICTED != 0,
    }
}

/// The device number as `std` reports it (`st_dev`, an `i32`, widened with its sign), so both readers agree.
pub(crate) fn device_id(raw: i32) -> u64 {
    i64::from(raw).cast_unsigned()
}

/// The `count` records at the start of `buffer`. A record that does not fit, or whose name is not one valid name,
/// is an error, after which nothing more is read.
pub(super) fn records(buffer: &[u8], count: usize) -> Records<'_> {
    Records { buffer, at: 0, left: count }
}

pub(super) struct Records<'a> {
    buffer: &'a [u8],
    at: usize,
    left: usize,
}

impl<'a> Iterator for Records<'a> {
    type Item = Result<Record<'a>, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left == 0 {
            return None;
        }
        self.left -= 1;
        let parsed = self.record();
        if parsed.is_err() {
            self.left = 0;
        }
        Some(parsed)
    }
}

impl<'a> Records<'a> {
    fn record(&mut self) -> Result<Record<'a>, ReadError> {
        let rest = self.buffer.get(self.at..).ok_or(ReadError::Failed)?;
        let length = Fields { bytes: rest, at: 0 }.u32()?;
        let length = usize::try_from(length).map_err(|_| ReadError::Failed)?;
        if length < HEADER {
            return Err(ReadError::Failed);
        }
        let bytes = rest.get(..length).ok_or(ReadError::Failed)?;
        self.at += length;
        parse(bytes)
    }
}

fn parse(bytes: &[u8]) -> Result<Record<'_>, ReadError> {
    let mut fields = Fields { bytes, at: 4 };
    let [common, _volume, _folder, file, extended] =
        [fields.u32()?, fields.u32()?, fields.u32()?, fields.u32()?, fields.u32()?];
    let mut record = Record::default();
    if common & ATTR_CMN_ERROR != 0 {
        record.error = fields.u32()?;
    }
    if common & ATTR_CMN_NAME == 0 {
        return Err(ReadError::Failed);
    }
    record.name = fields.name()?;
    if common & ATTR_CMN_DEVID != 0 {
        record.device = Some(fields.i32()?);
    }
    if common & ATTR_CMN_OBJTYPE != 0 {
        record.kind = Some(fields.u32()?);
    }
    if common & ATTR_CMN_MODTIME != 0 {
        record.modified = Some(fields.i64()?);
        fields.i64()?;
    }
    if common & ATTR_CMN_OWNERID != 0 {
        record.owner = Some(fields.u32()?);
    }
    if common & ATTR_CMN_ACCESSMASK != 0 {
        record.mode = Some(fields.u32()?);
    }
    if common & ATTR_CMN_FLAGS != 0 {
        record.flags = fields.u32()?;
    }
    if common & ATTR_CMN_FILEID != 0 {
        record.file_id = Some(fields.u64()?);
    }
    if file & ATTR_FILE_LINKCOUNT != 0 {
        record.links = Some(fields.u32()?);
    }
    if file & ATTR_FILE_TOTALSIZE != 0 {
        record.logical = Some(fields.u64()?);
    }
    if file & ATTR_FILE_ALLOCSIZE != 0 {
        record.allocated = Some(fields.u64()?);
    }
    if extended & ATTR_CMNEXT_CLONEID != 0 {
        record.clone_id = Some(fields.u64()?);
    }
    if extended & ATTR_CMNEXT_EXT_FLAGS != 0 {
        record.extended_flags = fields.u64()?;
    }
    Ok(record)
}

/// A cursor over one record.
struct Fields<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Fields<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
        let end = self.at.checked_add(N).ok_or(ReadError::Failed)?;
        let field = self.bytes.get(self.at..end).ok_or(ReadError::Failed)?;
        self.at = end;
        field.try_into().map_err(|_| ReadError::Failed)
    }

    fn u32(&mut self) -> Result<u32, ReadError> {
        self.take().map(u32::from_ne_bytes)
    }

    fn i32(&mut self) -> Result<i32, ReadError> {
        self.take().map(i32::from_ne_bytes)
    }

    fn u64(&mut self) -> Result<u64, ReadError> {
        self.take().map(u64::from_ne_bytes)
    }

    fn i64(&mut self) -> Result<i64, ReadError> {
        self.take().map(i64::from_ne_bytes)
    }

    /// The name of an `attrreference`: an offset from the reference itself and a length that counts the final NUL.
    /// The name must lie inside the record and be one name: not empty, no `/`, no NUL before the end.
    fn name(&mut self) -> Result<&'a [u8], ReadError> {
        let reference = self.at;
        let offset = isize::try_from(self.i32()?).map_err(|_| ReadError::Failed)?;
        let length = usize::try_from(self.u32()?).map_err(|_| ReadError::Failed)?;
        let start = reference.checked_add_signed(offset).ok_or(ReadError::Failed)?;
        let end = start.checked_add(length).ok_or(ReadError::Failed)?;
        let raw = self.bytes.get(start..end).ok_or(ReadError::Failed)?;
        let (&last, name) = raw.split_last().ok_or(ReadError::Failed)?;
        if last != 0 || name.is_empty() || name.iter().any(|&byte| byte == 0 || byte == b'/') {
            return Err(ReadError::Failed);
        }
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{ATTR_CMN_ERROR, ATTR_CMN_NAME, ATTR_CMN_RETURNED_ATTRS, COMMON, EXTENDED, FILE};

    /// A record as the kernel packs it: `fields` are the attributes after the name reference, in order.
    pub(super) struct Packed {
        pub(super) masks: [u32; 5],
        pub(super) error: Option<u32>,
        pub(super) fields: Vec<u8>,
        pub(super) name: Vec<u8>,
        /// Added to the name's offset, to point it elsewhere.
        pub(super) shift: i32,
        /// Added to the record's length.
        pub(super) extra_length: i64,
    }

    impl Packed {
        pub(super) fn named(name: &[u8]) -> Self {
            let mut stored = name.to_vec();
            stored.push(0);
            Self {
                masks: [ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_NAME, 0, 0, 0, 0],
                error: None,
                fields: Vec::new(),
                name: stored,
                shift: 0,
                extra_length: 0,
            }
        }

        /// A regular file with every attribute the reader asks for.
        pub(super) fn file(name: &[u8], extended_flags: u64) -> Self {
            let mut fields = Vec::new();
            fields.extend_from_slice(&0x0100_000d_i32.to_ne_bytes());
            fields.extend_from_slice(&1_u32.to_ne_bytes());
            fields.extend_from_slice(&1_700_000_000_i64.to_ne_bytes());
            fields.extend_from_slice(&0_i64.to_ne_bytes());
            fields.extend_from_slice(&501_u32.to_ne_bytes());
            fields.extend_from_slice(&0o100_644_u32.to_ne_bytes());
            fields.extend_from_slice(&0_u32.to_ne_bytes());
            fields.extend_from_slice(&42_u64.to_ne_bytes());
            fields.extend_from_slice(&2_u32.to_ne_bytes());
            fields.extend_from_slice(&5_000_u64.to_ne_bytes());
            fields.extend_from_slice(&8_192_u64.to_ne_bytes());
            fields.extend_from_slice(&77_u64.to_ne_bytes());
            fields.extend_from_slice(&extended_flags.to_ne_bytes());
            Self { masks: [COMMON & !ATTR_CMN_ERROR, 0, 0, FILE, EXTENDED], fields, ..Self::named(name) }
        }

        pub(super) fn bytes(&self) -> Vec<u8> {
            let mut masks = self.masks;
            let mut out = vec![0; 4];
            if self.error.is_some() {
                masks[0] |= ATTR_CMN_ERROR;
            }
            for mask in masks {
                out.extend_from_slice(&mask.to_ne_bytes());
            }
            if let Some(error) = self.error {
                out.extend_from_slice(&error.to_ne_bytes());
            }
            let reference = out.len();
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&self.fields);
            let offset = i32::try_from(out.len() - reference).unwrap().wrapping_add(self.shift);
            out[reference..reference + 4].copy_from_slice(&offset.to_ne_bytes());
            let length = u32::try_from(self.name.len()).unwrap();
            out[reference + 4..reference + 8].copy_from_slice(&length.to_ne_bytes());
            out.extend_from_slice(&self.name);
            while !out.len().is_multiple_of(8) {
                out.push(0);
            }
            let total = i64::try_from(out.len()).unwrap() + self.extra_length;
            out[..4].copy_from_slice(&u32::try_from(total).unwrap().to_ne_bytes());
            out
        }
    }

    mod negative {
        use fresher_core::ReadError;

        use super::Packed;
        use crate::bulk::parse::records;

        fn parse(buffer: &[u8], count: usize) -> Vec<Result<Vec<u8>, ReadError>> {
            records(buffer, count).map(|record| record.map(|record| record.name.to_vec())).collect()
        }

        #[test]
        fn a_record_longer_than_the_buffer_fails_the_folder() {
            let buffer = Packed { extra_length: 8, ..Packed::named(b"a") }.bytes();

            assert_eq!(parse(&buffer, 1), [Err(ReadError::Failed)]);
        }

        #[test]
        fn a_record_shorter_than_its_header_fails_the_folder() {
            let mut buffer = Packed::named(b"a").bytes();
            buffer[..4].copy_from_slice(&8_u32.to_ne_bytes());

            assert_eq!(parse(&buffer, 1), [Err(ReadError::Failed)]);
        }

        #[test]
        fn more_records_than_the_buffer_holds_fail_the_folder_and_stop() {
            let buffer = Packed::named(b"a").bytes();

            assert_eq!(parse(&buffer, 3), [Ok(b"a".to_vec()), Err(ReadError::Failed)]);
        }

        #[test]
        fn a_name_reference_outside_the_record_fails_the_folder() {
            for shift in [64, -64, i32::MAX, i32::MIN] {
                let buffer = Packed { shift, ..Packed::named(b"a") }.bytes();

                assert_eq!(parse(&buffer, 1), [Err(ReadError::Failed)], "{shift}");
            }
        }

        #[test]
        fn a_name_with_a_slash_fails_the_folder() {
            let buffer = Packed::named(b"a/b").bytes();

            assert_eq!(parse(&buffer, 1), [Err(ReadError::Failed)]);
        }

        #[test]
        fn a_name_with_a_nul_inside_or_none_at_the_end_fails_the_folder() {
            let inside = Packed::named(b"a\0b").bytes();
            let unterminated = Packed { name: b"ab".to_vec(), ..Packed::named(b"") }.bytes();
            let empty = Packed::named(b"").bytes();

            for buffer in [inside, unterminated, empty] {
                assert_eq!(parse(&buffer, 1), [Err(ReadError::Failed)]);
            }
        }

        #[test]
        fn a_record_without_a_name_fails_the_folder() {
            let mut packed = Packed::named(b"a");
            packed.masks[0] &= !super::ATTR_CMN_NAME;

            assert_eq!(parse(&packed.bytes(), 1), [Err(ReadError::Failed)]);
        }

        #[test]
        fn an_entry_error_marks_the_entry_not_the_folder() {
            let buffer =
                Packed { error: Some(u32::try_from(libc::EACCES).unwrap()), ..Packed::named(b"locked") }.bytes();

            let parsed: Vec<_> = records(&buffer, 1).collect();

            let entry = parsed[0].unwrap().entry(None);
            assert_eq!(parsed[0].unwrap().name, b"locked");
            assert_eq!(entry.error, Some(ReadError::NoPermission));
        }

        #[test]
        fn a_record_without_its_owner_or_mode_is_owned_by_nobody_and_allows_nothing() {
            let buffer = Packed::named(b"bare").bytes();

            let entry = records(&buffer, 1).next().unwrap().unwrap().entry(None);

            assert_eq!(entry.access, fresher_core::EntryAccess::default());
            assert_eq!((entry.access.owner, entry.access.mode), (None, 0));
        }
    }

    mod positive {
        use std::ops::Range;

        use fresher_core::{ByteSize, EntryAccess, EntryKind, Sizes, Timestamp};

        use super::Packed;
        use crate::bulk::parse::{
            EF_MAY_SHARE_BLOCKS, SF_APPEND, SF_DATALESS, SF_IMMUTABLE, SF_NOUNLINK, SF_RESTRICTED, UF_APPEND,
            UF_IMMUTABLE, device_id, records,
        };

        /// Where the flags lie in the fields of `Packed::file`.
        const FLAGS: Range<usize> = 32..36;

        #[test]
        fn a_file_record_gives_every_attribute() {
            let buffer = Packed::file(b"report.txt", 0).bytes();

            let record = records(&buffer, 1).next().unwrap().unwrap();
            let entry = record.entry(None);

            assert_eq!(record.name, b"report.txt");
            assert!(!record.may_share_blocks());
            assert_eq!(entry.kind, EntryKind::File);
            assert_eq!((entry.device, entry.file_id, entry.links), (0x0100_000d, 42, 2));
            assert_eq!(entry.sizes, Sizes::complete(ByteSize::new(5_000), ByteSize::new(8_192)));
            assert_eq!(entry.modified, Some(Timestamp::from_secs(1_700_000_000)));
            assert_eq!((entry.clone, entry.cloud_only, entry.error), (None, false, None));
            assert_eq!(entry.access, EntryAccess { owner: Some(501), mode: 0o644, ..EntryAccess::default() });
        }

        #[test]
        fn the_flags_give_the_locks() {
            for (flags, locks) in [
                (UF_IMMUTABLE, (true, false, false)),
                (SF_IMMUTABLE, (false, true, false)),
                (SF_RESTRICTED, (false, false, true)),
                (UF_APPEND, (true, false, false)),
                (SF_APPEND, (false, true, false)),
                (SF_NOUNLINK, (false, true, false)),
                (UF_IMMUTABLE | SF_IMMUTABLE | SF_RESTRICTED | SF_DATALESS, (true, true, true)),
            ] {
                let mut packed = Packed::file(b"locked", 0);
                packed.fields[FLAGS].copy_from_slice(&flags.to_ne_bytes());

                let access = records(&packed.bytes(), 1).next().unwrap().unwrap().entry(None).access;

                assert_eq!((access.user_immutable, access.system_immutable, access.restricted), locks, "{flags:#x}");
            }
        }

        #[test]
        fn a_clone_gives_its_clone_id_and_its_private_size() {
            let buffer = Packed::file(b"copy", EF_MAY_SHARE_BLOCKS).bytes();

            let record = records(&buffer, 1).next().unwrap().unwrap();
            let entry = record.entry(Some(4_096));

            assert!(record.may_share_blocks());
            assert_eq!(entry.clone, Some(77));
            assert_eq!(entry.sizes, Sizes::new(ByteSize::new(5_000), ByteSize::new(8_192), ByteSize::new(4_096)));
        }

        #[test]
        fn a_cloud_file_takes_no_allocated_space() {
            let mut packed = Packed::file(b"in-the-cloud", 0);
            packed.fields[FLAGS].copy_from_slice(&SF_DATALESS.to_ne_bytes());

            let entry = records(&packed.bytes(), 1).next().unwrap().unwrap().entry(None);

            assert!(entry.cloud_only);
            assert_eq!(entry.sizes, Sizes::complete(ByteSize::new(5_000), ByteSize::ZERO));
        }

        #[test]
        fn several_records_are_read_in_order() {
            let mut buffer = Packed::file(b"one", 0).bytes();
            buffer.extend(Packed::named(b"two").bytes());

            let names: Vec<_> = records(&buffer, 2).map(|record| record.unwrap().name.to_vec()).collect();

            assert_eq!(names, [b"one".to_vec(), b"two".to_vec()]);
        }

        #[test]
        fn a_device_number_is_widened_like_std_does() {
            assert_eq!(device_id(0x0100_0011), 0x0100_0011);
            assert_eq!(device_id(-1), u64::MAX);
        }
    }

    mod property {
        use proptest::prelude::*;

        use crate::bulk::parse::records;

        proptest! {
            #[test]
            fn the_parser_never_panics_on_any_bytes(
                buffer in proptest::collection::vec(any::<u8>(), 0..512),
                count in 0_usize..64,
            ) {
                for record in records(&buffer, count).flatten() {
                    let _ = record.entry(Some(1));
                }
            }

            #[test]
            fn the_parser_never_panics_on_any_masks(
                masks in proptest::array::uniform5(any::<u32>()),
                tail in proptest::collection::vec(any::<u8>(), 0..256),
            ) {
                let mut buffer = vec![0_u8; 4];
                for mask in masks {
                    buffer.extend_from_slice(&mask.to_ne_bytes());
                }
                buffer.extend_from_slice(&tail);
                let length = u32::try_from(buffer.len()).unwrap();
                buffer[..4].copy_from_slice(&length.to_ne_bytes());

                for record in records(&buffer, 2).flatten() {
                    let _ = record.entry(None);
                }
            }
        }
    }
}
