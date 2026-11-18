//! Reading a body back into a tree: the arena is rebuilt from the depth-first order and the child counts, the groups
//! are added again, and `freeze` sums the totals. Every count is checked against the bytes left; anything that does
//! not hold is a [`CodecError`], never a panic and never a partial tree.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use fresher_core::{FolderIdentity, MarkerCheck, Timestamp};

use crate::codec::CodecError;
use crate::codec::encode::GROUP_KINDS;
use crate::codec::header::Header;
use crate::codec::stamp::read_stamp;
use crate::codec::varint::Reader;
use crate::tree::access::NodeAccess;
use crate::tree::arena::{Shared, Tree};
use crate::tree::node::{Flag, Hit, Node, NodeFlags, NodeIndex, Sums};
use crate::tree::times::Stamp;

/// The fewest bytes a node's record takes: an empty suffix, flags, four sums, a time, no hits, no owner, the access,
/// and the name's prefix and suffix lengths.
const MIN_RECORD: usize = 11;
/// The fewest bytes a hit takes: interest, marker check, sibling bits.
const MIN_HIT: usize = 3;
/// The flags only a folder may carry.
const FOLDER_ONLY: [Flag; 7] = [
    Flag::Opaque,
    Flag::Collapsed,
    Flag::Avoided,
    Flag::OtherVolume,
    Flag::Incomplete,
    Flag::OthersFolder,
    Flag::SystemItems,
];

/// The tree and the header in `bytes`; the tree is frozen.
pub(crate) fn decode(bytes: &[u8]) -> Result<(Tree, Header), CodecError> {
    let mut reader = Reader::new(bytes);
    let header = Header::read(&mut reader)?;
    let roots = reader.count(MIN_RECORD)?;
    let nodes = reader.count(MIN_RECORD)?;
    let mut tree = Tree::with_capacity(nodes);
    let mut unclaimed = nodes.checked_sub(roots).ok_or(CodecError::CountTooLarge)?;
    let mut stack = vec![Frame { parent: NodeIndex::NONE, left: roots, last: NodeIndex::NONE, modified: 0 }];
    let mut name = Vec::new();
    let mut hits = Vec::new();
    while let Some(frame) = stack.last_mut() {
        if frame.left == 0 {
            stack.pop();
            continue;
        }
        frame.left -= 1;
        let Frame { parent, last, modified: base, .. } = *frame;
        let record = read_record(&mut reader, &tree, &header, Place { parent, last, base }, &mut name, &mut hits)?;
        if record.children > unclaimed {
            return Err(CodecError::CountTooLarge);
        }
        unclaimed -= record.children;
        let index = add(&mut tree, &record, parent, &name, &hits)?;
        if let Some(frame) = stack.last_mut() {
            frame.last = index;
        }
        if record.children > 0 {
            stack.push(Frame {
                parent: index,
                left: record.children,
                last: NodeIndex::NONE,
                modified: record.modified,
            });
        }
    }
    for kind in GROUP_KINDS {
        read_groups(&mut reader, &mut tree, kind, nodes)?;
    }
    if reader.remaining() != 0 {
        return Err(CodecError::TrailingBytes);
    }
    tree.freeze();
    Ok((tree, header))
}

/// A folder whose children are being read: how many are left, the last one read, its modification time.
#[derive(Clone, Copy, Debug)]
struct Frame {
    parent: NodeIndex,
    left: usize,
    last: NodeIndex,
    modified: i64,
}

/// Where a record lies: its parent, its previous sibling and the parent's modification time.
#[derive(Clone, Copy, Debug)]
struct Place {
    parent: NodeIndex,
    last: NodeIndex,
    base: i64,
}

/// One node's record, read; its name and hits are in the caller's buffers.
#[derive(Clone, Copy, Debug)]
struct Record {
    flags: NodeFlags,
    children: usize,
    hidden: u32,
    own: Sums,
    modified: i64,
    /// Its times and inside; only a node with hits has one.
    stamp: Option<Stamp>,
    owner: u32,
    access: NodeAccess,
    identity: Option<FolderIdentity>,
}

fn read_record(
    reader: &mut Reader<'_>,
    tree: &Tree,
    header: &Header,
    place: Place,
    name: &mut Vec<u8>,
    hits: &mut Vec<Hit>,
) -> Result<Record, CodecError> {
    let is_root = place.parent == NodeIndex::NONE;
    read_name(reader, tree.name(place.last), name)?;
    check_name(name, is_root, place.last != NodeIndex::NONE, tree.name(place.last))?;
    let flags = read_flags(reader, is_root)?;
    let (children, hidden) = if flags.has(Flag::Folder) { (reader.count(MIN_RECORD)?, reader.u32()?) } else { (0, 0) };
    if flags.has(Flag::Collapsed) && children > 0 {
        return Err(CodecError::BadFlags);
    }
    let own =
        Sums { files: reader.u32()?, logical: reader.u64()?, allocated: reader.u64()?, reclaimable: reader.u64()? };
    let modified = place.base.wrapping_add(reader.i64()?);
    if modified < 0 && modified != i64::MIN {
        return Err(CodecError::OutOfRange);
    }
    read_hits(reader, hits)?;
    let stamp = if hits.is_empty() { None } else { Some(read_stamp(reader, modified)?) };
    let owner = match reader.u64()? {
        0 => Node::NO_OWNER,
        claim => {
            let owner = u32::try_from(claim - 1).map_err(|_| CodecError::UnknownOwner)?;
            if owner as usize >= header.claims.len() {
                return Err(CodecError::UnknownOwner);
            }
            owner
        }
    };
    let [access] = reader.array::<1>()?;
    let access = NodeAccess::from_bits(access).ok_or(CodecError::BadFlags)?;
    let identity = if is_root { Some(FolderIdentity { device: reader.u64()?, file_id: reader.u64()? }) } else { None };
    Ok(Record { flags, children, hidden, own, modified, stamp, owner, access, identity })
}

/// Reads a front-coded name into `name`: the prefix it shares with `previous`, then the rest.
fn read_name(reader: &mut Reader<'_>, previous: &[u8], name: &mut Vec<u8>) -> Result<(), CodecError> {
    let shared = reader.count(0)?;
    let shared = previous.get(..shared).ok_or(CodecError::BadName)?;
    let len = reader.count(1)?;
    name.clear();
    name.extend_from_slice(shared);
    name.extend_from_slice(reader.bytes(len)?);
    Ok(())
}

/// A root is an absolute path; any other name is one plain name, after its previous sibling in byte order.
fn check_name(name: &[u8], is_root: bool, has_previous: bool, previous: &[u8]) -> Result<(), CodecError> {
    let valid = if is_root {
        Path::new(OsStr::from_bytes(name)).is_absolute()
    } else {
        !name.is_empty()
            && name != b"."
            && name != b".."
            && !name.contains(&b'/')
            && !name.contains(&0)
            && (!has_previous || name > previous)
    };
    if valid { Ok(()) } else { Err(CodecError::BadName) }
}

/// The flags of a node: known bits, exactly one kind, the folder flags on a folder only, a root a folder.
fn read_flags(reader: &mut Reader<'_>, is_root: bool) -> Result<NodeFlags, CodecError> {
    let flags = NodeFlags::from_bits(reader.u16()?).ok_or(CodecError::BadFlags)?;
    let kinds =
        [Flag::Folder, Flag::KeptFile, Flag::Link, Flag::Special].into_iter().filter(|&kind| flags.has(kind)).count();
    let folder = flags.has(Flag::Folder);
    let misplaced = !folder && (FOLDER_ONLY.into_iter().any(|flag| flags.has(flag)) || flags.unreadable().is_some())
        || folder && flags.has(Flag::LinkCopy);
    if kinds != 1 || misplaced || is_root && !folder {
        return Err(CodecError::BadFlags);
    }
    Ok(flags)
}

fn read_hits(reader: &mut Reader<'_>, hits: &mut Vec<Hit>) -> Result<(), CodecError> {
    let count = reader.count(MIN_HIT)?;
    hits.clear();
    for _ in 0..count {
        let interest = reader.u32()?;
        let marker = match reader.array::<1>()? {
            [0] => None,
            [1] => Some(MarkerCheck::Valid),
            [2] => Some(MarkerCheck::Invalid),
            _ => return Err(CodecError::OutOfRange),
        };
        hits.push(Hit { interest, marker, siblings: reader.u64()? });
    }
    Ok(())
}

/// Adds the node of `record` to the tree under `parent` (a root when there is none).
fn add(
    tree: &mut Tree,
    record: &Record,
    parent: NodeIndex,
    name: &[u8],
    hits: &[Hit],
) -> Result<NodeIndex, CodecError> {
    let index = match record.identity {
        Some(identity) => tree.add_root(Path::new(OsStr::from_bytes(name)), identity)?,
        None => tree.add_child(parent, name, record.flags.kind())?,
    };
    tree.set_flags(index, record.flags)?;
    tree.set_own(index, record.own)?;
    tree.add_hidden_folders(index, record.hidden)?;
    let modified = u64::try_from(record.modified).ok().map(Timestamp::from_secs);
    tree.set_modified(index, modified)?;
    tree.add_hits(index, hits)?;
    if let Some(stamp) = record.stamp {
        tree.set_times(index, stamp.times)?;
        tree.set_inside(index, stamp.inside)?;
    }
    tree.set_owner(index, record.owner)?;
    tree.set_access(index, record.access)?;
    Ok(index)
}

/// Reads the groups of `kind` and adds every member to the tree; a member's node is its depth-first position, which
/// is its index in the rebuilt arena.
fn read_groups(reader: &mut Reader<'_>, tree: &mut Tree, kind: Shared, nodes: usize) -> Result<(), CodecError> {
    let groups = reader.count(7)?;
    let (mut device, mut id) = (0_u64, 0_u64);
    for _ in 0..groups {
        let device_delta = reader.u64()?;
        let id_part = reader.u64()?;
        device = device.checked_add(device_delta).ok_or(CodecError::OutOfRange)?;
        id = if device_delta == 0 { id.checked_add(id_part).ok_or(CodecError::OutOfRange)? } else { id_part };
        let bytes =
            Sums { files: reader.u32()?, logical: reader.u64()?, allocated: reader.u64()?, reclaimable: reader.u64()? };
        let members = reader.count(1)?;
        let mut rank = 0_u64;
        for _ in 0..members {
            rank = rank.checked_add(reader.u64()?).ok_or(CodecError::OutOfRange)?;
            let holder = usize::try_from(rank)
                .ok()
                .filter(|&position| position < nodes)
                .and_then(NodeIndex::new)
                .ok_or(CodecError::OutOfRange)?;
            tree.add_shared(kind, device, id, holder, bytes)?;
        }
    }
    Ok(())
}
