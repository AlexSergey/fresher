//! One node of the arena and the small values it is made of.

use fresher_core::{EntryKind, MarkerCheck, Sizes, Timestamp, Unreadable};

use crate::tree::access::NodeAccess;

/// The index of a node in the arena; 32 bits, so a node stays small.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeIndex(u32);

impl NodeIndex {
    /// No node: the end of a sibling list, the parent of a root.
    pub(crate) const NONE: Self = Self(u32::MAX);

    /// The index of the node at `position`; `None` past the last index a node can have.
    pub(crate) fn new(position: usize) -> Option<Self> {
        u32::try_from(position).ok().filter(|&raw| raw != u32::MAX).map(Self)
    }

    /// The position in the arena; `None` for [`NodeIndex::NONE`].
    pub(crate) fn get(self) -> Option<usize> {
        if self == Self::NONE { None } else { usize::try_from(self.0).ok() }
    }
}

/// The folded files of a folder, or the bytes of a subtree: a file count and three sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Sums {
    pub(crate) files: u32,
    pub(crate) logical: u64,
    pub(crate) allocated: u64,
    pub(crate) reclaimable: u64,
}

impl Sums {
    pub(crate) const ZERO: Self = Self { files: 0, logical: 0, allocated: 0, reclaimable: 0 };

    /// One file of `sizes`.
    pub(crate) const fn file(sizes: Sizes) -> Self {
        Self {
            files: 1,
            logical: sizes.logical.bytes(),
            allocated: sizes.allocated.bytes(),
            reclaimable: sizes.reclaimable.bytes(),
        }
    }

    /// The sum; `None` when a count or a size does not fit, so an overflow is never silent.
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            files: self.files.checked_add(other.files)?,
            logical: self.logical.checked_add(other.logical)?,
            allocated: self.allocated.checked_add(other.allocated)?,
            reclaimable: self.reclaimable.checked_add(other.reclaimable)?,
        })
    }

    /// The sum, each part stopping at its largest value.
    pub(crate) const fn saturating_add(self, other: Self) -> Self {
        Self {
            files: self.files.saturating_add(other.files),
            logical: self.logical.saturating_add(other.logical),
            allocated: self.allocated.saturating_add(other.allocated),
            reclaimable: self.reclaimable.saturating_add(other.reclaimable),
        }
    }
}

/// One named bit of a node's flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Flag {
    Folder,
    KeptFile,
    Link,
    /// A folder claimed whole: its totals are kept, nothing inside it is a node.
    Opaque,
    /// A subtree folded into its root to keep the cache within its cap.
    Collapsed,
    /// A folder the walk did not open: opening it would raise a system prompt.
    Avoided,
    /// A folder on another volume than its root: not entered.
    OtherVolume,
    /// Some entry of this folder could not be read; the rest was.
    Incomplete,
    /// Something in this subtree could not be read; set by `freeze`.
    Partial,
    /// A kept file whose hard link group's bytes are counted at another path; set by `freeze`.
    LinkCopy,
    /// A socket, a named pipe or a device ([`EntryKind::Other`]).
    Special,
    /// A folder of another user in a walked place: not entered; always with `Avoided`.
    OthersFolder,
    /// The system's own items (`TemporaryItems`): never opened; always with `Avoided`.
    SystemItems,
}

impl Flag {
    const fn bit(self) -> u16 {
        match self {
            Self::Folder => 1,
            Self::KeptFile => 1 << 1,
            Self::Link => 1 << 2,
            Self::Opaque => 1 << 3,
            Self::Collapsed => 1 << 4,
            Self::Avoided => 1 << 5,
            Self::OtherVolume => 1 << 6,
            Self::Incomplete => 1 << 7,
            Self::Partial => 1 << 8,
            Self::LinkCopy => 1 << 11,
            Self::Special => 1 << 12,
            Self::OthersFolder => 1 << 13,
            Self::SystemItems => 1 << 14,
        }
    }
}

/// The flags of a node in 16 bits: the [`Flag`]s and the kind of an unreadable folder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct NodeFlags(u16);

impl NodeFlags {
    const UNREADABLE_SHIFT: u32 = 9;
    const UNREADABLE_MASK: u16 = 0b11 << Self::UNREADABLE_SHIFT;
    /// Every bit a [`Flag`] or the unreadable kind uses.
    const KNOWN: u16 = 0x7fff;

    /// The flags as stored by the cache codec.
    pub(crate) const fn bits(self) -> u16 {
        self.0
    }

    /// The flags of `bits`; `None` when a bit no flag uses is set.
    pub(crate) const fn from_bits(bits: u16) -> Option<Self> {
        if bits & !Self::KNOWN == 0 { Some(Self(bits)) } else { None }
    }

    pub(crate) const fn of(kind: EntryKind) -> Self {
        Self(match kind {
            EntryKind::Folder => Flag::Folder.bit(),
            EntryKind::File => Flag::KeptFile.bit(),
            EntryKind::Symlink => Flag::Link.bit(),
            EntryKind::Other => Flag::Special.bit(),
        })
    }

    pub(crate) const fn has(self, flag: Flag) -> bool {
        self.0 & flag.bit() != 0
    }

    pub(crate) const fn set(&mut self, flag: Flag) {
        self.0 |= flag.bit();
    }

    pub(crate) const fn unset(&mut self, flag: Flag) {
        self.0 &= !flag.bit();
    }

    pub(crate) const fn kind(self) -> EntryKind {
        if self.has(Flag::Folder) {
            EntryKind::Folder
        } else if self.has(Flag::Link) {
            EntryKind::Symlink
        } else if self.has(Flag::Special) {
            EntryKind::Other
        } else {
            EntryKind::File
        }
    }

    /// Why this folder could not be read, if it could not.
    pub(crate) const fn unreadable(self) -> Option<Unreadable> {
        match (self.0 & Self::UNREADABLE_MASK) >> Self::UNREADABLE_SHIFT {
            1 => Some(Unreadable::NoPermission),
            2 => Some(Unreadable::Guarded),
            3 => Some(Unreadable::Failed),
            _ => None,
        }
    }

    pub(crate) const fn set_unreadable(&mut self, reason: Unreadable) {
        let code: u16 = match reason {
            Unreadable::NoPermission => 1,
            Unreadable::Guarded => 2,
            Unreadable::Failed => 3,
        };
        self.0 = (self.0 & !Self::UNREADABLE_MASK) | (code << Self::UNREADABLE_SHIFT);
    }

    /// Whether the node's own bytes are not all known: it, or an entry of it, was not read.
    pub(crate) const fn is_incomplete(self) -> bool {
        self.unreadable().is_some() || self.has(Flag::Avoided) || self.has(Flag::Incomplete)
    }
}

/// One interest that matched a node: which interest, the marker check, and which of its sibling names were present
/// (bit `i` for the interest's `i`-th sibling).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Hit {
    pub(crate) interest: u32,
    pub(crate) marker: Option<MarkerCheck>,
    pub(crate) siblings: u64,
}

/// A node: a folder or a kept entry. 72 bytes (its access takes a byte that was padding); lists and names are 32-bit
/// indices into the arena's buffers.
///
/// The three sizes and the file count hold the node's own bytes (a folder's folded files, a kept file's sizes) until
/// the tree is frozen, and the subtree's after: storing both would cost 28 bytes a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) logical: u64,
    pub(crate) allocated: u64,
    pub(crate) reclaimable: u64,
    /// Seconds since the Unix epoch; `i64::MIN` for none.
    pub(crate) modified: i64,
    pub(crate) name_offset: u32,
    pub(crate) parent: NodeIndex,
    pub(crate) first_child: NodeIndex,
    pub(crate) next_sibling: NodeIndex,
    pub(crate) files: u32,
    /// The folders below this one; before `freeze`, only those not kept as nodes (inside an opaque folder).
    pub(crate) folders: u32,
    pub(crate) hits_start: u32,
    /// An index into the claims table; `u32::MAX` for none.
    pub(crate) owner: u32,
    pub(crate) name_len: u16,
    pub(crate) hits_len: u16,
    pub(crate) flags: NodeFlags,
    /// Its entry's access for the scan's user; [`NodeAccess::NONE`] until it is read.
    pub(crate) access: NodeAccess,
}

impl Node {
    pub(crate) const NO_OWNER: u32 = u32::MAX;
    const NO_TIME: i64 = i64::MIN;

    pub(crate) const fn new(name_offset: u32, name_len: u16, parent: NodeIndex, flags: NodeFlags) -> Self {
        Self {
            logical: 0,
            allocated: 0,
            reclaimable: 0,
            modified: Self::NO_TIME,
            name_offset,
            parent,
            first_child: NodeIndex::NONE,
            next_sibling: NodeIndex::NONE,
            files: 0,
            folders: 0,
            hits_start: 0,
            owner: Self::NO_OWNER,
            name_len,
            hits_len: 0,
            flags,
            access: NodeAccess::NONE,
        }
    }

    pub(crate) const fn sums(&self) -> Sums {
        Sums { files: self.files, logical: self.logical, allocated: self.allocated, reclaimable: self.reclaimable }
    }

    pub(crate) const fn set_sums(&mut self, sums: Sums) {
        self.files = sums.files;
        self.logical = sums.logical;
        self.allocated = sums.allocated;
        self.reclaimable = sums.reclaimable;
    }

    /// Adds `sums` to the node's; a sum that does not fit stops at its largest value and makes the node `Partial`.
    pub(crate) fn add_sums(&mut self, sums: Sums) {
        if let Some(sum) = self.sums().checked_add(sums) {
            self.set_sums(sum);
        } else {
            self.set_sums(self.sums().saturating_add(sums));
            self.flags.set(Flag::Partial);
        }
    }

    pub(crate) fn modified(&self) -> Option<Timestamp> {
        time_of(self.modified)
    }

    pub(crate) fn set_modified(&mut self, modified: Option<Timestamp>) {
        self.modified = time_bits(modified);
    }
}

/// A time as a node and the cache codec keep it: seconds since the Unix epoch, `i64::MIN` for none.
pub(crate) fn time_bits(time: Option<Timestamp>) -> i64 {
    time.map_or(Node::NO_TIME, |time| i64::try_from(time.secs()).unwrap_or(i64::MAX))
}

/// The time of `bits`; `None` for no time (or any negative value).
pub(crate) fn time_of(bits: i64) -> Option<Timestamp> {
    u64::try_from(bits).ok().map(Timestamp::from_secs)
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::EntryKind;

        use crate::tree::node::{Flag, NodeFlags, NodeIndex, Sums};

        #[test]
        fn the_last_index_is_reserved_for_none() {
            assert_eq!(NodeIndex::new(u32::MAX as usize), None);
            assert_eq!(NodeIndex::NONE.get(), None);
        }

        #[test]
        fn an_overflowing_sum_is_refused() {
            let full = Sums { logical: u64::MAX, ..Sums::ZERO };
            assert_eq!(full.checked_add(Sums { logical: 1, ..Sums::ZERO }), None);
            assert_eq!(full.saturating_add(full).logical, u64::MAX);
        }

        #[test]
        fn a_readable_folder_has_no_unreadable_kind() {
            let flags = NodeFlags::of(EntryKind::Folder);
            assert_eq!(flags.unreadable(), None);
            assert!(!flags.is_incomplete());
            assert!(!flags.has(Flag::Partial));
        }
    }

    mod positive {
        use fresher_core::{EntryKind, Timestamp, Unreadable};

        use crate::tree::node::{Flag, Node, NodeFlags, NodeIndex};

        #[test]
        fn a_node_takes_72_bytes() {
            assert_eq!(size_of::<Node>(), 72);
        }

        #[test]
        fn flags_keep_the_kind_and_the_unreadable_reason_apart() {
            for kind in [EntryKind::File, EntryKind::Folder, EntryKind::Symlink] {
                assert_eq!(NodeFlags::of(kind).kind(), kind);
            }
            let mut flags = NodeFlags::of(EntryKind::Folder);
            for reason in [Unreadable::NoPermission, Unreadable::Guarded, Unreadable::Failed] {
                flags.set_unreadable(reason);
                assert_eq!(flags.unreadable(), Some(reason));
            }
            flags.set(Flag::Opaque);
            assert!(flags.has(Flag::Opaque) && flags.has(Flag::Folder) && flags.is_incomplete());
        }

        #[test]
        fn modified_round_trips_and_none_stays_none() {
            let mut node = Node::new(0, 1, NodeIndex::NONE, NodeFlags::of(EntryKind::File));
            assert_eq!(node.modified(), None);
            node.set_modified(Some(Timestamp::from_secs(1_735_689_600)));
            assert_eq!(node.modified(), Some(Timestamp::from_secs(1_735_689_600)));
            node.set_modified(None);
            assert_eq!(node.modified(), None);
        }
    }
}
