//! The facts a test tree or a sandbox states instead of making them: a status-change time no tool can set, an owner
//! or a system flag only root can set, a socket or a pipe whose path would be too long. [`FactsLens`] reports them
//! over a real reader, for exactly the stated paths; nothing else changes, and Finder never shows them.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use fresher_core::{
    EntryAccess, EntryKind, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError, Timestamp,
};
use serde::{Deserialize, Serialize};

/// The user id a stated `other-user` entry reads as: `nobody` (-2), an account no one logs in as, so it never equals
/// the user that builds trees and runs the tests (a second account of the Mac, such as 502, could).
pub const OTHER_USER: u32 = 4_294_967_294;

/// Who a stated entry belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatedOwner {
    /// Root: the user id 0.
    System,
    /// Another user of the Mac: [`OTHER_USER`].
    OtherUser,
}

impl StatedOwner {
    /// The user id the reader reports.
    pub const fn uid(self) -> u32 {
        match self {
            Self::System => 0,
            Self::OtherUser => OTHER_USER,
        }
    }
}

/// A flag of a stated entry, as `chflags` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatedFlag {
    /// Locked by the user (`uchg`).
    UserLock,
    /// Locked by the system (`schg`).
    SystemLock,
    /// Not removable (`sunlnk`): read as the system's lock.
    NoUnlink,
    /// Guarded by the system's integrity protection.
    Restricted,
}

/// What a stated file is instead of a regular file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatedKind {
    Socket,
    Pipe,
}

/// The facts stated for one entry; a fact not stated stays as the disk has it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stated {
    pub changed: Option<Timestamp>,
    pub accessed: Option<Timestamp>,
    pub owner: Option<StatedOwner>,
    /// When not empty, the entry's flags: every flag the disk has is replaced.
    pub flags: BTreeSet<StatedFlag>,
    pub kind: Option<StatedKind>,
}

impl Stated {
    /// Whether nothing is stated.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// `entry` with the stated facts in place of its own.
    pub fn apply(&self, entry: RawEntry) -> RawEntry {
        RawEntry {
            kind: if self.kind.is_some() { EntryKind::Other } else { entry.kind },
            changed: self.changed.or(entry.changed),
            accessed: self.accessed.or(entry.accessed),
            access: self.apply_access(entry.access),
            ..entry
        }
    }

    /// `access` with the stated owner and flags in place of its own.
    pub fn apply_access(&self, access: EntryAccess) -> EntryAccess {
        let owner = self.owner.map_or(access.owner, |owner| Some(owner.uid()));
        if self.flags.is_empty() {
            return EntryAccess { owner, ..access };
        }
        let has = |flag| self.flags.contains(&flag);
        EntryAccess {
            owner,
            user_immutable: has(StatedFlag::UserLock),
            system_immutable: has(StatedFlag::SystemLock) || has(StatedFlag::NoUnlink),
            restricted: has(StatedFlag::Restricted),
            ..access
        }
    }
}

/// Stated facts by path. A lookup is exact: an entry is found only at the path it was stated at (the caller states it
/// at every form of the path a walk reaches, such as the canonical one).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatedFacts {
    by_path: HashMap<PathBuf, Stated>,
    folders: HashSet<PathBuf>,
}

impl StatedFacts {
    /// No stated fact.
    pub fn new() -> Self {
        Self::default()
    }

    /// The facts of `entries`, each path joined to `root`; an entry that states nothing is left out.
    pub fn under<'a>(root: &Path, entries: impl IntoIterator<Item = (&'a Path, &'a Stated)>) -> Self {
        let mut facts = Self::new();
        for (path, stated) in entries {
            facts.insert(root.join(path), stated.clone());
        }
        facts
    }

    /// States `stated` for the entry at `path`; nothing when it states nothing.
    pub fn insert(&mut self, path: PathBuf, stated: Stated) {
        if stated.is_empty() {
            return;
        }
        if let Some(folder) = path.parent() {
            self.folders.insert(folder.to_path_buf());
        }
        self.by_path.insert(path, stated);
    }

    /// Both sets of facts in one; a path stated by both takes `other`'s.
    #[must_use]
    pub fn joined(mut self, other: Self) -> Self {
        for (path, stated) in other.by_path {
            self.insert(path, stated);
        }
        self
    }

    /// The facts stated for the entry at exactly `path`.
    pub fn get(&self, path: &Path) -> Option<&Stated> {
        self.by_path.get(path)
    }

    /// Whether an entry of the folder at exactly `folder` is stated.
    pub fn holds_in(&self, folder: &Path) -> bool {
        self.folders.contains(folder)
    }

    pub fn is_empty(&self) -> bool {
        self.by_path.is_empty()
    }

    pub fn len(&self) -> usize {
        self.by_path.len()
    }
}

/// A [`FolderReader`] that answers every call with the wrapped reader, then reports, for a stated entry only, its
/// stated status change, access, owner, flags and kind instead of the disk's.
#[derive(Debug)]
pub struct FactsLens<R> {
    inner: R,
    facts: StatedFacts,
}

impl<R: FolderReader> FactsLens<R> {
    pub const fn new(inner: R, facts: StatedFacts) -> Self {
        Self { inner, facts }
    }
}

impl<R: FolderReader> FolderReader for FactsLens<R> {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        let access = self.inner.access(path)?;
        Ok(self.facts.get(path).map_or(access, |stated| stated.apply_access(access)))
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.inner.read(path, expected, out)?;
        if !self.facts.holds_in(path) {
            return Ok(());
        }
        let entries: Vec<(Vec<u8>, RawEntry)> = out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        out.clear();
        for (name, entry) in entries {
            let stated = std::str::from_utf8(&name).ok().and_then(|text| self.facts.get(&path.join(text)));
            out.push(&name, stated.map_or(entry, |stated| stated.apply(entry)))?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use fresher_core::{
        EntryAccess, EntryKind, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError, Sizes, Timestamp,
    };

    use super::{FactsLens, Stated, StatedFacts, StatedFlag, StatedKind, StatedOwner};

    /// A reader of fixed folders: each listed with a plain file per name, owned by 501.
    #[derive(Debug, Default)]
    struct Listed {
        folders: BTreeMap<PathBuf, Vec<&'static str>>,
    }

    fn plain() -> RawEntry {
        RawEntry {
            kind: EntryKind::File,
            device: 1,
            file_id: 2,
            links: 1,
            sizes: Sizes::ZERO,
            clone: None,
            modified: Some(Timestamp::from_secs(10)),
            changed: Some(Timestamp::from_secs(20)),
            accessed: Some(Timestamp::from_secs(30)),
            cloud_only: false,
            access: EntryAccess { owner: Some(501), mode: 0o644, ..EntryAccess::default() },
            error: None,
        }
    }

    impl FolderReader for Listed {
        fn identify(&self, _path: &Path) -> Result<FolderIdentity, ReadError> {
            Ok(FolderIdentity { device: 1, file_id: 1 })
        }

        fn access(&self, _path: &Path) -> Result<EntryAccess, ReadError> {
            Ok(plain().access)
        }

        fn read(&self, path: &Path, _expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
            out.clear();
            for name in self.folders.get(path).ok_or(ReadError::Vanished)? {
                out.push(name.as_bytes(), plain())?;
            }
            Ok(())
        }

        fn head(&self, _folder: &Path, _name: &str, _limit: usize) -> Result<Vec<u8>, ReadError> {
            Ok(b"head".to_vec())
        }
    }

    fn reader() -> Listed {
        let mut folders = BTreeMap::new();
        folders.insert(PathBuf::from("/tree/a"), vec!["socket", "old", "plain"]);
        folders.insert(PathBuf::from("/other/a"), vec!["socket", "old", "plain"]);
        Listed { folders }
    }

    fn facts() -> StatedFacts {
        let mut facts = StatedFacts::new();
        facts.insert("/tree/a/socket".into(), Stated { kind: Some(StatedKind::Socket), ..Stated::default() });
        facts.insert(
            "/tree/a/old".into(),
            Stated {
                changed: Some(Timestamp::from_secs(5)),
                accessed: Some(Timestamp::from_secs(6)),
                owner: Some(StatedOwner::System),
                flags: [StatedFlag::NoUnlink, StatedFlag::Restricted].into(),
                kind: None,
            },
        );
        facts
    }

    fn listing(reader: &dyn FolderReader, path: &str) -> Vec<(String, RawEntry)> {
        let mut out = FolderListing::new();
        reader.read(Path::new(path), FolderIdentity { device: 1, file_id: 1 }, &mut out).unwrap();
        out.iter().map(|(name, entry)| (String::from_utf8_lossy(name).into_owned(), *entry)).collect()
    }

    mod negative {
        use std::path::Path;

        use fresher_core::FolderReader as _;

        use super::{FactsLens, Stated, StatedFacts, facts, listing, reader};

        #[test]
        fn an_entry_not_stated_reads_as_the_reader_reads_it() {
            let lens = FactsLens::new(reader(), facts());

            let read = listing(&lens, "/tree/a");

            assert_eq!(read[2], listing(&reader(), "/tree/a")[2]);
        }

        #[test]
        fn the_lens_never_answers_for_a_path_outside_the_tree() {
            let lens = FactsLens::new(reader(), facts());

            assert_eq!(listing(&lens, "/other/a"), listing(&reader(), "/other/a"));
            assert_eq!(lens.access(Path::new("/other/a/old")), reader().access(Path::new("/other/a/old")));
        }

        #[test]
        fn the_lens_with_no_facts_is_the_reader() {
            let lens = FactsLens::new(reader(), StatedFacts::new());

            assert_eq!(listing(&lens, "/tree/a"), listing(&reader(), "/tree/a"));
            assert_eq!(lens.head(Path::new("/tree/a"), "plain", 4), Ok(b"head".to_vec()));
        }

        #[test]
        fn an_entry_that_states_nothing_is_not_kept() {
            let mut facts = StatedFacts::new();
            facts.insert("/tree/a/plain".into(), Stated::default());

            assert!(facts.is_empty() && !facts.holds_in(Path::new("/tree/a")));
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{EntryKind, FolderReader as _, Timestamp};

        use super::{FactsLens, facts, listing, reader};

        #[test]
        fn a_stated_socket_is_another_kind_with_its_own_times() {
            let lens = FactsLens::new(reader(), facts());

            let (name, socket) = listing(&lens, "/tree/a")[0].clone();

            assert_eq!(name, "socket");
            assert_eq!(socket.kind, EntryKind::Other);
            assert_eq!(socket.changed, Some(Timestamp::from_secs(20)));
        }

        #[test]
        fn stated_times_owner_and_flags_replace_the_disks() {
            let lens = FactsLens::new(reader(), facts());

            let (_, old) = listing(&lens, "/tree/a")[1].clone();

            assert_eq!(
                (old.modified, old.changed, old.accessed),
                (Some(Timestamp::from_secs(10)), Some(Timestamp::from_secs(5)), Some(Timestamp::from_secs(6)))
            );
            assert_eq!(old.access.owner, Some(0));
            assert!(!old.access.user_immutable && old.access.system_immutable && old.access.restricted);
            assert_eq!(old.access.mode, 0o644);
        }

        #[test]
        fn a_stated_folder_access_is_reported() {
            let lens = FactsLens::new(reader(), facts());

            let access = lens.access(Path::new("/tree/a/old")).unwrap();

            assert_eq!(access.owner, Some(0));
            assert!(access.system_immutable);
        }
    }
}
