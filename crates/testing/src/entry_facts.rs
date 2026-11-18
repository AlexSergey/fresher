//! What the reference walk reads about an entry for its removability, with `std::fs` only. "The user" is the owner of
//! the walk's home folder (in a test, whoever built the fixture tree). Write access is read from the mode bits alone:
//! the owner's bit for the user's own folder, the others' bit for any other; a group is never assumed to include the
//! user. A fact the tree states (`StatedFacts`) is read instead of the disk's.

use std::fs::{self, Metadata};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use fresher_core::{EntryFacts, EntryKind, Timestamp};

use crate::facts_lens::{Stated, StatedFacts, StatedFlag, StatedOwner};

/// The restricted flag (`SF_RESTRICTED` in `sys/stat.h`): the system's integrity protection guards the entry.
#[cfg(target_os = "macos")]
const SF_RESTRICTED: u32 = 0x0008_0000;
/// The no-unlink flag (`SF_NOUNLINK`), which `libc` does not name: the item cannot be removed.
#[cfg(target_os = "macos")]
const SF_NOUNLINK: u32 = 0x0010_0000;

/// An entry as the reference walk sees it: its metadata, with the facts the tree states in place of the disk's.
#[derive(Debug)]
pub(crate) struct Seen<'a> {
    metadata: Metadata,
    stated: Option<&'a Stated>,
}

impl<'a> Seen<'a> {
    /// The entry at `path`, never following a link, with what `stated` says of it.
    pub(crate) fn read(path: &Path, stated: &'a StatedFacts) -> io::Result<Self> {
        Ok(Self { metadata: fs::symlink_metadata(path)?, stated: stated.get(path) })
    }

    /// The disk's own metadata: sizes, device, mode.
    pub(crate) const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub(crate) fn kind(&self) -> EntryKind {
        if self.stated.is_some_and(|stated| stated.kind.is_some()) {
            EntryKind::Other
        } else {
            disk_kind(&self.metadata)
        }
    }

    pub(crate) fn owner(&self) -> u32 {
        self.stated.and_then(|stated| stated.owner).map_or_else(|| self.metadata.uid(), StatedOwner::uid)
    }

    pub(crate) fn modified(&self) -> Option<Timestamp> {
        seconds(self.metadata.mtime())
    }

    pub(crate) fn changed(&self) -> Option<Timestamp> {
        self.stated.and_then(|stated| stated.changed).or_else(|| seconds(self.metadata.ctime()))
    }

    pub(crate) fn accessed(&self) -> Option<Timestamp> {
        self.stated.and_then(|stated| stated.accessed).or_else(|| seconds(self.metadata.atime()))
    }

    /// The user lock, the system lock and the restricted flag.
    pub(crate) fn flags(&self) -> (bool, bool, bool) {
        match self.stated.map(|stated| &stated.flags).filter(|flags| !flags.is_empty()) {
            Some(flags) => (
                flags.contains(&StatedFlag::UserLock),
                flags.contains(&StatedFlag::SystemLock) || flags.contains(&StatedFlag::NoUnlink),
                flags.contains(&StatedFlag::Restricted),
            ),
            None => flags(&self.metadata),
        }
    }

    /// Whether `user` may write to the entry, by its mode bits.
    fn writable(&self, user: Option<u32>) -> bool {
        let bit = if user == Some(self.owner()) { 0o200 } else { 0o002 };
        self.metadata.mode() & bit != 0
    }
}

/// The kind of an entry as the disk has it.
pub(crate) fn disk_kind(metadata: &Metadata) -> EntryKind {
    if metadata.is_symlink() {
        EntryKind::Symlink
    } else if metadata.is_dir() {
        EntryKind::Folder
    } else if metadata.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

/// Whole seconds since the epoch; a time before it is not known.
fn seconds(secs: i64) -> Option<Timestamp> {
    u64::try_from(secs).ok().map(Timestamp::from_secs)
}

/// The facts of the entry at `path`, seen as `seen`, for `user`; `None` (the owner of the home folder could not be
/// read) owns nothing. Its folder is read with what `stated` says of it.
pub(crate) fn entry_facts(
    path: &Path,
    seen: &Seen<'_>,
    kind: EntryKind,
    user: Option<u32>,
    stated: &StatedFacts,
) -> EntryFacts {
    let parent = path.parent().and_then(|parent| Seen::read(parent, stated).ok());
    let (user_immutable, system_immutable, restricted) = seen.flags();
    EntryFacts {
        owned_by_user: user == Some(seen.owner()),
        parent_writable: parent.is_some_and(|parent| parent.writable(user)),
        folder_writable: kind != EntryKind::Folder || seen.writable(user),
        user_immutable,
        system_immutable,
        restricted,
    }
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

    use super::{Seen, entry_facts};
    use crate::{FixtureTree, StatedFacts};

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str("[[file]]\npath = \"Caches/a.bin\"\n\n[[dir]]\npath = \"Caches/inner\"\n").unwrap()
    }

    fn facts_of(path: &Path, kind: EntryKind, user: Option<u32>) -> EntryFacts {
        let none = StatedFacts::new();
        entry_facts(path, &Seen::read(path, &none).unwrap(), kind, user, &none)
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
