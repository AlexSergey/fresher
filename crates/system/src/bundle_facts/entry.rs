//! The bundle's entry facts for the user, from its `lstat` and its folder's.

use std::fs::Metadata;
use std::os::macos::fs::MetadataExt as _;

use fresher_core::EntryFacts;

/// `UF_IMMUTABLE` and `UF_APPEND` of `<sys/stat.h>`: locked by the user (an append-only entry cannot be removed
/// either, so it counts as locked: fail closed).
const USER_LOCKS: u32 = 0x0000_0002 | 0x0000_0004;
/// `SF_IMMUTABLE`, `SF_APPEND` and `SF_NOUNLINK`: locked by the system.
const SYSTEM_LOCKS: u32 = 0x0002_0000 | 0x0004_0000 | 0x0010_0000;
/// `SF_RESTRICTED`: guarded by the system's integrity protection.
const RESTRICTED: u32 = 0x0008_0000;

/// The write bits of a mode for its owner, its group and the others.
const OWNER_WRITE: u32 = 0o200;
const GROUP_WRITE: u32 = 0o020;
const OTHERS_WRITE: u32 = 0o002;

/// What the facts are read from: an entry's owner, group, mode and flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Access {
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    pub(crate) mode: u32,
    pub(crate) flags: u32,
}

impl Access {
    /// The access of an entry as `lstat` read it.
    pub(crate) fn of(metadata: &Metadata) -> Self {
        Self { uid: metadata.st_uid(), gid: metadata.st_gid(), mode: metadata.st_mode(), flags: metadata.st_flags() }
    }

    /// Whether `user`, in `groups`, may write to it by its mode bits: the owner's bit when the user owns it, the
    /// group's when the user is in its group, else the others'. An access list never counts.
    fn writable_by(self, user: u32, groups: &[u32]) -> bool {
        let bit = if self.uid == user {
            OWNER_WRITE
        } else if groups.contains(&self.gid) {
            GROUP_WRITE
        } else {
            OTHERS_WRITE
        };
        self.mode & bit != 0
    }
}

/// The facts of a bundle folder with the access `bundle`, in a folder with the access `parent`, for `user` in
/// `groups`. A folder that is locked (by the user or the system) or restricted lets no entry out, whatever its mode
/// says, so its lock makes it not writable. Nothing is opened to find out.
pub(crate) fn entry_facts(bundle: Access, parent: Access, user: u32, groups: &[u32]) -> EntryFacts {
    let parent_locked = parent.flags & (USER_LOCKS | SYSTEM_LOCKS | RESTRICTED) != 0;
    EntryFacts {
        owned_by_user: bundle.uid == user,
        parent_writable: !parent_locked && parent.writable_by(user, groups),
        folder_writable: bundle.writable_by(user, groups),
        user_immutable: bundle.flags & USER_LOCKS != 0,
        system_immutable: bundle.flags & SYSTEM_LOCKS != 0,
        restricted: bundle.flags & RESTRICTED != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::Access;

    const ME: u32 = 501;
    const STAFF: u32 = 20;
    const ADMIN: u32 = 80;

    /// A folder of the user's, in the group `staff`, with `mode` and no flag.
    fn mine(mode: u32) -> Access {
        Access { uid: ME, gid: STAFF, mode, flags: 0 }
    }

    /// A folder of root's, in the group `admin`, with `mode`.
    fn roots(mode: u32) -> Access {
        Access { uid: 0, gid: ADMIN, mode, flags: 0 }
    }

    mod negative {
        use fresher_core::{EntryFacts, Removability};

        use super::{ADMIN, ME, STAFF, mine, roots};
        use crate::bundle_facts::entry::{Access, entry_facts};

        #[test]
        fn a_bundle_of_another_owner_is_not_owned() {
            let facts = entry_facts(roots(0o775), mine(0o755), ME, &[STAFF, ADMIN]);

            assert!(!facts.owned_by_user);
            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn a_folder_the_user_cannot_write_to_is_not_writable() {
            // The owner's bit for the user's own folder, whatever the group's and the others' say.
            assert!(!entry_facts(mine(0o755), mine(0o577), ME, &[STAFF]).parent_writable);
            // The group's bit when the user is in the folder's group, whatever the others' say.
            assert!(!entry_facts(mine(0o755), roots(0o757), ME, &[STAFF, ADMIN]).parent_writable);
            // The others' bit when the user is not in it, whatever the group's says.
            assert!(!entry_facts(mine(0o755), roots(0o775), ME, &[STAFF]).parent_writable);
            // The bundle's own bits decide whether it may move.
            let facts = entry_facts(mine(0o555), mine(0o755), ME, &[STAFF]);
            assert_eq!(facts, EntryFacts { folder_writable: false, ..EntryFacts::REMOVABLE });
            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn a_bundle_in_a_locked_folder_is_not_removable() {
            for flags in [0x0000_0002, 0x0000_0004, 0x0002_0000, 0x0004_0000, 0x0010_0000, 0x0008_0000] {
                let parent = Access { flags, ..mine(0o777) };

                let facts = entry_facts(mine(0o755), parent, ME, &[STAFF]);

                assert_eq!(facts, EntryFacts { parent_writable: false, ..EntryFacts::REMOVABLE }, "{flags:#x}");
                assert_ne!(Removability::of(facts), Removability::Removable, "{flags:#x}");
            }
            // A flag no lock uses leaves the folder writable.
            let hidden = Access { flags: 0x0000_8000, ..mine(0o755) };
            assert!(entry_facts(mine(0o755), hidden, ME, &[STAFF]).parent_writable);
        }

        #[test]
        fn each_flag_is_read() {
            let cases = [
                (0x0000_0002, (true, false, false)),
                (0x0000_0004, (true, false, false)),
                (0x0002_0000, (false, true, false)),
                (0x0004_0000, (false, true, false)),
                (0x0010_0000, (false, true, false)),
                (0x0008_0000, (false, false, true)),
                (0x0000_0001, (false, false, false)),
            ];
            for (flags, (user, system, restricted)) in cases {
                let bundle = Access { flags, ..mine(0o755) };

                let facts = entry_facts(bundle, mine(0o755), ME, &[STAFF]);

                assert_eq!(
                    (facts.user_immutable, facts.system_immutable, facts.restricted),
                    (user, system, restricted),
                    "{flags:#x}"
                );
            }
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;

        use fresher_core::{EntryFacts, Removability};

        use super::{ADMIN, ME, STAFF, mine, roots};
        use crate::bundle_facts::entry::{Access, entry_facts};

        #[test]
        fn the_user_s_bundle_in_a_writable_folder_is_removable() {
            // The apps folder as macOS makes it: root's, writable by the group `admin` the user is in.
            let facts = entry_facts(mine(0o755), roots(0o775), ME, &[STAFF, ADMIN]);

            assert_eq!(facts, EntryFacts::REMOVABLE);
            assert_eq!(Removability::of(facts), Removability::Removable);
            // A folder writable by everyone, and the user's own.
            assert_eq!(entry_facts(mine(0o700), roots(0o777), ME, &[]), EntryFacts::REMOVABLE);
            assert_eq!(entry_facts(mine(0o700), mine(0o700), ME, &[]), EntryFacts::REMOVABLE);
        }

        #[test]
        fn the_access_is_read_from_the_entry_itself() {
            let root = tempfile::tempdir().unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let metadata = fs::symlink_metadata(root.path()).unwrap();

            let access = Access::of(&metadata);

            assert_eq!(access.uid, fresher_macos::current_user());
            assert_eq!(access.mode & 0o7777, 0o700);
            assert_eq!(access.flags, 0);
        }
    }
}
