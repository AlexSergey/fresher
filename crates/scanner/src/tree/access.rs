//! What a node keeps of its entry's access, for the user the scan is for: five bits, from which the facts of a match
//! are worked out with its parent's.

use fresher_core::{EntryAccess, EntryFacts, EntryKind};

/// The access of a node for the scan's user, in one byte. Nothing set is what nothing is known of: owned by
/// somebody else, not writable, with no lock, so a node whose access was never read is not removable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct NodeAccess(u8);

impl NodeAccess {
    const OWNED: u8 = 1;
    const WRITABLE: u8 = 1 << 1;
    const USER_IMMUTABLE: u8 = 1 << 2;
    const SYSTEM_IMMUTABLE: u8 = 1 << 3;
    const RESTRICTED: u8 = 1 << 4;
    const KNOWN: u8 = 0b1_1111;

    /// The access of a node whose entry was not read.
    pub(crate) const NONE: Self = Self(0);

    /// The access of `access` for `user` (`None`: no user is known, so nothing is owned). Write access comes from
    /// the permission bits alone: the owner's bit for the user's own entry, the others' bit for any other; a group or
    /// an access list never counts, so an entry is writable by the user only when the bits plainly say so.
    pub(crate) const fn of(access: EntryAccess, user: Option<u32>) -> Self {
        let owned = match (access.owner, user) {
            (Some(owner), Some(user)) => owner == user,
            _ => false,
        };
        let bit = if owned { 0o200 } else { 0o002 };
        let mut bits = 0;
        if owned {
            bits |= Self::OWNED;
        }
        if access.mode & bit != 0 {
            bits |= Self::WRITABLE;
        }
        if access.user_immutable {
            bits |= Self::USER_IMMUTABLE;
        }
        if access.system_immutable {
            bits |= Self::SYSTEM_IMMUTABLE;
        }
        if access.restricted {
            bits |= Self::RESTRICTED;
        }
        Self(bits)
    }

    /// The bits as the cache codec stores them.
    pub(crate) const fn bits(self) -> u8 {
        self.0
    }

    /// The access of `bits`; `None` when a bit no fact uses is set.
    pub(crate) const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::KNOWN == 0 { Some(Self(bits)) } else { None }
    }

    const fn has(self, bit: u8) -> bool {
        self.0 & bit != 0
    }

    /// The facts of an entry of `kind` with this access, in a folder with the access `parent`.
    pub(crate) const fn facts(self, kind: EntryKind, parent: Self) -> EntryFacts {
        EntryFacts {
            owned_by_user: self.has(Self::OWNED),
            parent_writable: parent.has(Self::WRITABLE),
            folder_writable: !matches!(kind, EntryKind::Folder) || self.has(Self::WRITABLE),
            user_immutable: self.has(Self::USER_IMMUTABLE),
            system_immutable: self.has(Self::SYSTEM_IMMUTABLE),
            restricted: self.has(Self::RESTRICTED),
        }
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::EntryAccess;

    const ME: Option<u32> = Some(501);

    fn mine(mode: u16) -> EntryAccess {
        EntryAccess { owner: ME, mode, ..EntryAccess::default() }
    }

    mod negative {
        use fresher_core::{EntryAccess, EntryFacts, EntryKind, Removability};

        use super::{ME, mine};
        use crate::tree::access::NodeAccess;

        #[test]
        fn an_access_never_read_is_not_removable() {
            for kind in [EntryKind::File, EntryKind::Folder, EntryKind::Symlink] {
                let facts = NodeAccess::NONE.facts(kind, NodeAccess::NONE);

                assert_eq!(Removability::of(facts), Removability::NeedsAdministrator, "{kind:?}");
            }
            assert_eq!(NodeAccess::of(EntryAccess::default(), ME), NodeAccess::NONE);
        }

        #[test]
        fn an_entry_of_another_user_is_not_owned_and_writable_only_by_the_others_bit() {
            let theirs = EntryAccess { owner: Some(0), mode: 0o755, ..EntryAccess::default() };
            let open = EntryAccess { mode: 0o777, ..theirs };

            let facts = NodeAccess::of(theirs, ME).facts(EntryKind::Folder, NodeAccess::of(mine(0o755), ME));

            assert!(!facts.owned_by_user && !facts.folder_writable);
            assert!(NodeAccess::of(open, ME).facts(EntryKind::Folder, NodeAccess::NONE).folder_writable);
        }

        #[test]
        fn without_a_user_or_an_owner_nothing_is_owned() {
            let unknown = EntryAccess { owner: None, ..mine(0o755) };

            assert!(!NodeAccess::of(mine(0o755), None).facts(EntryKind::File, NodeAccess::NONE).owned_by_user);
            assert!(!NodeAccess::of(unknown, ME).facts(EntryKind::File, NodeAccess::NONE).owned_by_user);
        }

        #[test]
        fn a_read_only_parent_or_folder_needs_an_administrator() {
            let parent = NodeAccess::of(mine(0o555), ME);

            let file = NodeAccess::of(mine(0o644), ME).facts(EntryKind::File, parent);
            let folder = NodeAccess::of(mine(0o555), ME).facts(EntryKind::Folder, NodeAccess::of(mine(0o755), ME));

            assert_eq!(file, EntryFacts { parent_writable: false, ..EntryFacts::REMOVABLE });
            assert_eq!(folder, EntryFacts { folder_writable: false, ..EntryFacts::REMOVABLE });
        }

        #[test]
        fn the_locks_are_kept() {
            let locked = EntryAccess { user_immutable: true, system_immutable: true, restricted: true, ..mine(0o644) };
            let parent = NodeAccess::of(mine(0o755), ME);

            let facts = NodeAccess::of(locked, ME).facts(EntryKind::File, parent);

            assert!(facts.user_immutable && facts.system_immutable && facts.restricted);
            assert_eq!(Removability::of(facts), Removability::ProtectedBySystem);
        }

        #[test]
        fn unknown_bits_are_refused() {
            assert_eq!(NodeAccess::from_bits(0b10_0000), None);
            assert_eq!(NodeAccess::from_bits(u8::MAX), None);
        }
    }

    mod positive {
        use fresher_core::{EntryAccess, EntryFacts, EntryKind};

        use super::{ME, mine};
        use crate::tree::access::NodeAccess;

        #[test]
        fn the_users_own_writable_entries_are_removable() {
            let parent = NodeAccess::of(mine(0o755), ME);

            for kind in [EntryKind::File, EntryKind::Folder, EntryKind::Symlink] {
                assert_eq!(NodeAccess::of(mine(0o755), ME).facts(kind, parent), EntryFacts::REMOVABLE, "{kind:?}");
            }
            let read_only_file = NodeAccess::of(mine(0o444), ME).facts(EntryKind::File, parent);
            assert_eq!(read_only_file, EntryFacts::REMOVABLE);
        }

        #[test]
        fn the_bits_round_trip() {
            let all = EntryAccess { user_immutable: true, system_immutable: true, restricted: true, ..mine(0o700) };
            for access in [NodeAccess::NONE, NodeAccess::of(mine(0o644), ME), NodeAccess::of(all, ME)] {
                assert_eq!(NodeAccess::from_bits(access.bits()), Some(access));
            }
        }
    }
}
