//! What the reference walk reads about an entry for its removability, with `std::fs` only. "The user" is the owner of
//! the walk's home folder (in a test, whoever built the fixture tree). Write access is read from the mode bits alone:
//! the owner's bit for the user's own folder, the others' bit for any other; a group is never assumed to include the
//! user.

use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use fresher_core::{EntryFacts, EntryKind};

/// The restricted flag (`SF_RESTRICTED` in `sys/stat.h`): the system's integrity protection guards the entry.
#[cfg(target_os = "macos")]
const SF_RESTRICTED: u32 = 0x0008_0000;
/// The no-unlink flag (`SF_NOUNLINK`), which `libc` does not name: the item cannot be removed.
#[cfg(target_os = "macos")]
const SF_NOUNLINK: u32 = 0x0010_0000;

/// The facts of the entry at `path` with `metadata`, for `user`; `None` (the owner of the home folder could not be
/// read) owns nothing.
pub(crate) fn entry_facts(path: &Path, metadata: &Metadata, kind: EntryKind, user: Option<u32>) -> EntryFacts {
    let parent = path.parent().and_then(|parent| fs::symlink_metadata(parent).ok());
    let (user_immutable, system_immutable, restricted) = flags(metadata);
    EntryFacts {
        owned_by_user: user == Some(metadata.uid()),
        parent_writable: parent.is_some_and(|parent| writable(&parent, user)),
        folder_writable: kind != EntryKind::Folder || writable(metadata, user),
        user_immutable,
        system_immutable,
        restricted,
    }
}

/// Whether `user` may write to the entry, by its mode bits.
fn writable(metadata: &Metadata, user: Option<u32>) -> bool {
    let bit = if user == Some(metadata.uid()) { 0o200 } else { 0o002 };
    metadata.mode() & bit != 0
}

/// The user lock (immutable or append-only), the system lock (immutable, append-only or no-unlink) and the restricted
/// flag.
#[cfg(target_os = "macos")]
fn flags(metadata: &Metadata) -> (bool, bool, bool) {
    use std::os::macos::fs::MetadataExt as _;

    let flags = metadata.st_flags();
    (
        flags & (libc::UF_IMMUTABLE | libc::UF_APPEND) != 0,
        flags & (libc::SF_IMMUTABLE | libc::SF_APPEND | SF_NOUNLINK) != 0,
        flags & SF_RESTRICTED != 0,
    )
}

/// No flags off macOS.
#[cfg(not(target_os = "macos"))]
fn flags(_metadata: &Metadata) -> (bool, bool, bool) {
    (false, false, false)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, Permissions};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::Path;

    use fresher_core::{EntryFacts, EntryKind};

    use super::entry_facts;
    use crate::FixtureTree;

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str("[[file]]\npath = \"Caches/a.bin\"\n\n[[dir]]\npath = \"Caches/inner\"\n").unwrap()
    }

    fn facts_of(path: &Path, kind: EntryKind, user: Option<u32>) -> EntryFacts {
        entry_facts(path, &fs::symlink_metadata(path).unwrap(), kind, user)
    }

    fn me(tree: &FixtureTree) -> Option<u32> {
        fs::metadata(tree.root()).ok().map(|root| root.uid())
    }

    /// Sets the mode of `path`, and gives it back when dropped, so the temporary tree can be removed.
    struct Mode<'a>(&'a Path, u32);

    impl<'a> Mode<'a> {
        fn set(path: &'a Path, mode: u32) -> Self {
            let old = fs::metadata(path).unwrap().mode() & 0o7777;
            fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
            Self(path, old)
        }
    }

    impl Drop for Mode<'_> {
        fn drop(&mut self) {
            let _ = fs::set_permissions(self.0, Permissions::from_mode(self.1));
        }
    }

    mod negative {
        use fresher_core::{EntryKind, Removability};

        use super::{Mode, facts_of, me, tree};

        #[test]
        fn an_entry_of_another_user_is_not_owned() {
            let tree = tree();
            let other = me(&tree).map(|uid| uid.wrapping_add(1));

            let facts = facts_of(&tree.root().join("Caches/a.bin"), EntryKind::File, other);

            assert!(!facts.owned_by_user);
            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn an_entry_in_a_read_only_folder_needs_an_administrator() {
            let tree = tree();
            let caches = tree.root().join("Caches");
            let _mode = Mode::set(&caches, 0o555);

            let facts = facts_of(&caches.join("a.bin"), EntryKind::File, me(&tree));

            assert!(facts.owned_by_user && !facts.parent_writable);
            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn a_read_only_folder_needs_an_administrator() {
            let tree = tree();
            let inner = tree.root().join("Caches/inner");
            let _mode = Mode::set(&inner, 0o555);

            let facts = facts_of(&inner, EntryKind::Folder, me(&tree));

            assert!(!facts.folder_writable);
            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn without_a_user_nothing_is_owned() {
            let tree = tree();

            let facts = facts_of(&tree.root().join("Caches/a.bin"), EntryKind::File, None);

            assert!(!facts.owned_by_user);
        }
    }

    mod positive {
        use fresher_core::{EntryFacts, EntryKind, Removability};

        use super::{facts_of, me, tree};

        #[test]
        fn the_users_own_entries_in_writable_folders_are_removable() {
            let tree = tree();

            let file = facts_of(&tree.root().join("Caches/a.bin"), EntryKind::File, me(&tree));
            let folder = facts_of(&tree.root().join("Caches/inner"), EntryKind::Folder, me(&tree));

            assert_eq!(file, EntryFacts::REMOVABLE);
            assert_eq!(Removability::of(folder), Removability::Removable);
        }
    }
}
