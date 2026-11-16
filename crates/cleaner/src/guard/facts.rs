//! Check 8: the item can be removed now. Its facts are read again from its handle and its parent's, the way plan
//! 006's reference walk reads them: the owner against the user, write access from the mode bits alone (the owner's
//! bit for the user's own entry, the others' bit for any other), and the file flags.

use std::fs::Metadata;
use std::os::unix::fs::MetadataExt as _;

use fresher_core::{EntryFacts, EntryKind, Removability, SkipReason};

use crate::guard::identity;
use crate::paths::{Handle, LastLink, Placed, open_failure, open_no_link};

/// The user's immutable and append-only flags (`UF_IMMUTABLE`, `UF_APPEND` in `sys/stat.h`).
const USER_LOCKS: u32 = 0x0000_0002 | 0x0000_0004;
/// The system's immutable, append-only and no-unlink flags (`SF_IMMUTABLE`, `SF_APPEND`, `SF_NOUNLINK`).
const SYSTEM_LOCKS: u32 = 0x0002_0000 | 0x0004_0000 | 0x0010_0000;
/// The restricted flag (`SF_RESTRICTED`): the system's integrity protection guards the entry.
const RESTRICTED: u32 = 0x0008_0000;

/// Check 8: `NotRemovable` unless the facts read now make the item `Removable` for `user`. The parent is opened
/// through no link, like the item.
pub(super) fn check(handle: &Handle, placed: &Placed, user: u32) -> Result<(), SkipReason> {
    let Some(parent) = placed.physical.parent() else {
        return Err(SkipReason::CheckFailed);
    };
    let parent = open_no_link(parent, LastLink::Refuse).map_err(|error| open_failure(&error))?;
    let parent = parent.metadata().map_err(|_| SkipReason::CheckFailed)?;
    match Removability::of(facts(handle.metadata(), &parent, user)) {
        Removability::Removable => Ok(()),
        removability => Err(SkipReason::NotRemovable { removability }),
    }
}

fn facts(item: &Metadata, parent: &Metadata, user: u32) -> EntryFacts {
    let flags = flags(item);
    EntryFacts {
        owned_by_user: item.uid() == user,
        parent_writable: writable(parent, user),
        folder_writable: identity::kind(item) != EntryKind::Folder || writable(item, user),
        user_immutable: flags & USER_LOCKS != 0,
        system_immutable: flags & SYSTEM_LOCKS != 0,
        restricted: flags & RESTRICTED != 0,
    }
}

/// Whether `user` may write to the entry, by its mode bits; a group is never assumed to include the user.
fn writable(metadata: &Metadata, user: u32) -> bool {
    let bit = if metadata.uid() == user { 0o200 } else { 0o002 };
    metadata.mode() & bit != 0
}

#[cfg(target_os = "macos")]
fn flags(metadata: &Metadata) -> u32 {
    std::os::macos::fs::MetadataExt::st_flags(metadata)
}

/// No flags off macOS.
#[cfg(not(target_os = "macos"))]
const fn flags(_metadata: &Metadata) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use fresher_core::SkipReason;
    use fresher_testing::FixtureTree;

    use crate::paths::{Handle, Roots};
    use crate::test_support::user_of;

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str("[[file]]\npath = \"cache/a.bin\"\n\n[[dir]]\npath = \"cache/inner\"\n").unwrap()
    }

    fn check(tree: &FixtureTree, relative: &str, user: u32) -> Result<(), SkipReason> {
        let roots = Roots::new(&[tree.root().to_path_buf()], &[tree.root().to_path_buf()]);
        let placed = roots.place(&tree.root().join(relative)).unwrap();
        super::check(&Handle::open(&placed).unwrap(), &placed, user)
    }

    mod negative {
        use fresher_core::{Removability, SkipReason};

        use super::{check, tree};
        use crate::test_support::{Flags, Mode, user_of};

        #[test]
        fn an_item_of_another_owner_is_not_removable() {
            let tree = tree();
            let other = user_of(tree.root()).wrapping_add(1);

            let answer = check(&tree, "cache/a.bin", other);

            assert_eq!(answer, Err(SkipReason::NotRemovable { removability: Removability::NeedsAdministrator }));
        }

        #[test]
        fn an_item_in_a_read_only_folder_is_not_removable() {
            let tree = tree();
            let _mode = Mode::set(&tree.root().join("cache"), 0o555);

            let answer = check(&tree, "cache/a.bin", user_of(tree.root()));

            assert_eq!(answer, Err(SkipReason::NotRemovable { removability: Removability::NeedsAdministrator }));
        }

        #[test]
        fn a_read_only_folder_is_not_removable() {
            let tree = tree();
            let _mode = Mode::set(&tree.root().join("cache/inner"), 0o555);

            let answer = check(&tree, "cache/inner", user_of(tree.root()));

            assert_eq!(answer, Err(SkipReason::NotRemovable { removability: Removability::NeedsAdministrator }));
        }

        #[test]
        fn a_locked_item_is_not_removable() {
            let tree = tree();
            let _flags = Flags::lock(&tree.root().join("cache/a.bin"));

            let answer = check(&tree, "cache/a.bin", user_of(tree.root()));

            assert_eq!(answer, Err(SkipReason::NotRemovable { removability: Removability::Locked }));
        }
    }

    mod positive {
        use super::{check, tree, user_of};

        #[test]
        fn the_users_own_items_in_writable_folders_are_removable() {
            let tree = tree();

            assert_eq!(check(&tree, "cache/a.bin", user_of(tree.root())), Ok(()));
            assert_eq!(check(&tree, "cache/inner", user_of(tree.root())), Ok(()));
        }
    }
}
