//! The scan roots, made ready for a walk: canonical, identified, on a known volume, none inside another.

use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

use fresher_core::{
    EntryAccess, FolderIdentity, FolderReader, ReadError, ScopeKey, VolumeFacts, Volumes, WalkError, stable_hash,
};

/// One root of the walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Root {
    /// The canonical path: links in the path above it resolved once, here and nowhere else.
    pub(crate) path: PathBuf,
    pub(crate) identity: FolderIdentity,
    pub(crate) volume: VolumeFacts,
    /// Who owns the root folder, its permission bits and its locks: the parent's access of its entries.
    pub(crate) access: EntryAccess,
}

/// The roots of one walk, in the order asked; a root inside another one is dropped (the outer one covers it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Roots {
    roots: Vec<Root>,
    dropped: usize,
}

impl Roots {
    /// Canonicalizes and identifies every root of `roots`. A root that is a link, not a folder, or cannot be read
    /// stops the walk with `RootUnreadable` and its index; nothing under a root is canonicalized.
    pub(crate) fn prepare(
        roots: &[PathBuf],
        volumes: &dyn Volumes,
        reader: &dyn FolderReader,
    ) -> Result<Self, WalkError> {
        Self::with_identities(roots, volumes, |path| Ok((reader.identify(path)?, reader.access(path)?)))
    }

    /// The roots as `prepare` makes them, without asking the reader anything: a root's identity is its volume's
    /// device and no file id, its access nothing known. What a restore needs, which must not touch the disk beyond
    /// the roots' metadata.
    pub(crate) fn locate(roots: &[PathBuf], volumes: &dyn Volumes) -> Result<Self, WalkError> {
        Self::with_identities(roots, volumes, |path| {
            let identity = volumes.identify(path).map(|volume| FolderIdentity { device: volume.device, file_id: 0 })?;
            Ok((identity, EntryAccess::default()))
        })
    }

    fn with_identities(
        roots: &[PathBuf],
        volumes: &dyn Volumes,
        identity_of: impl Fn(&std::path::Path) -> Result<(FolderIdentity, EntryAccess), ReadError>,
    ) -> Result<Self, WalkError> {
        let mut prepared: Vec<Root> = Vec::with_capacity(roots.len());
        for (index, root) in roots.iter().enumerate() {
            let unreadable = |kind| WalkError::RootUnreadable { index, kind };
            let metadata = fs::symlink_metadata(root).map_err(|error| unreadable(error.kind()))?;
            if !metadata.is_dir() {
                return Err(unreadable(io::ErrorKind::NotADirectory));
            }
            let path = fs::canonicalize(root).map_err(|error| unreadable(error.kind()))?;
            let (identity, access) = identity_of(&path).map_err(|error| unreadable(kind_of(error)))?;
            let volume = volumes.identify(&path).map_err(|error| unreadable(kind_of(error)))?;
            prepared.push(Root { path, identity, volume, access });
        }
        let mut kept: Vec<Root> = Vec::with_capacity(prepared.len());
        let mut dropped = 0;
        for (position, root) in prepared.iter().enumerate() {
            let covered = prepared.iter().enumerate().any(|(other, outer)| {
                other != position && root.path.starts_with(&outer.path) && (outer.path != root.path || other < position)
            });
            if covered {
                dropped += 1;
            } else {
                kept.push(root.clone());
            }
        }
        if dropped > 0 {
            tracing::info!(dropped, "scan roots inside another root are scanned through it");
        }
        Ok(Self { roots: kept, dropped })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Root> {
        self.roots.iter()
    }

    /// How many roots were dropped as inside another root.
    #[cfg_attr(not(test), expect(dead_code, reason = "only logged until the summary can carry it"))]
    pub(crate) const fn dropped(&self) -> usize {
        self.dropped
    }

    /// A stable hash of the canonical roots and their volume ids, whatever their order: what a cache is of.
    pub(crate) fn scope_key(&self) -> ScopeKey {
        let mut parts: Vec<(&[u8], u128)> =
            self.roots.iter().map(|root| (root.path.as_os_str().as_bytes(), root.volume.id.get())).collect();
        parts.sort_unstable();
        let mut bytes = Vec::new();
        for (path, volume) in parts {
            bytes.extend_from_slice(&(path.len() as u64).to_le_bytes());
            bytes.extend_from_slice(path);
            bytes.extend_from_slice(&volume.to_le_bytes());
        }
        ScopeKey::new(stable_hash(&bytes))
    }

    /// The root that holds `path`, by components.
    #[cfg(test)]
    pub(crate) fn holding(&self, path: &std::path::Path) -> Option<&Root> {
        self.roots.iter().find(|root| path.starts_with(&root.path))
    }
}

/// The kind of I/O error a refused read stands for, for `WalkError::RootUnreadable`.
pub(crate) const fn kind_of(error: ReadError) -> io::ErrorKind {
    match error {
        ReadError::NoPermission | ReadError::Guarded => io::ErrorKind::PermissionDenied,
        ReadError::Vanished => io::ErrorKind::NotFound,
        ReadError::Replaced => io::ErrorKind::NotADirectory,
        ReadError::Interrupted => io::ErrorKind::Interrupted,
        ReadError::Failed => io::ErrorKind::Other,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::{Path, PathBuf};

    use fresher_core::{ByteSize, VolumeFacts, VolumeId};
    use fresher_testing::FakeVolumes;

    pub(crate) fn facts(id: u128, device: u64) -> VolumeFacts {
        VolumeFacts { id: VolumeId::new(id), device, case_sensitive: false, used: ByteSize::new(1) }
    }

    /// Every path on one volume.
    pub(crate) fn one_volume() -> FakeVolumes {
        FakeVolumes::default().with("/", facts(1, 1))
    }

    pub(crate) fn canonical(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap()
    }

    mod negative {
        use std::io;
        use std::os::unix::fs::symlink;

        use fresher_core::WalkError;
        use fresher_testing::{FakeVolumes, FixtureTree};

        use super::{canonical, facts, one_volume};
        use crate::walk::roots::Roots;
        use crate::walk::std_reader::StdReader;

        #[test]
        fn a_root_inside_another_root_is_dropped_and_counted() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a/b\"\n[[dir]]\npath = \"c\"\n").unwrap();
            let root = tree.root();
            let asked = [root.join("a/b"), root.join("a"), root.join("c"), root.join("a")];

            let roots = Roots::prepare(&asked, &one_volume(), &StdReader::new()).unwrap();

            let kept: Vec<_> = roots.iter().map(|root| root.path.clone()).collect();
            assert_eq!(kept, [canonical(&root.join("a")), canonical(&root.join("c"))]);
            assert_eq!(roots.dropped(), 2);
        }

        #[test]
        fn a_root_that_is_a_file_is_refused() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"file\"\n").unwrap();
            let asked = [tree.root().to_path_buf(), tree.root().join("file")];

            let error = Roots::prepare(&asked, &one_volume(), &StdReader::new());

            assert_eq!(error, Err(WalkError::RootUnreadable { index: 1, kind: io::ErrorKind::NotADirectory }));
        }

        #[test]
        fn a_root_that_is_a_link_is_refused() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"real\"\n").unwrap();
            symlink(tree.root().join("real"), tree.root().join("link")).unwrap();

            let error = Roots::prepare(&[tree.root().join("link")], &one_volume(), &StdReader::new());

            assert_eq!(error, Err(WalkError::RootUnreadable { index: 0, kind: io::ErrorKind::NotADirectory }));
        }

        #[test]
        fn a_missing_root_or_an_unknown_volume_is_refused() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n").unwrap();

            let missing = Roots::prepare(&[tree.root().join("missing")], &one_volume(), &StdReader::new());
            let elsewhere = FakeVolumes::default().with("/nowhere", facts(1, 1));
            let unknown = Roots::prepare(&[tree.root().join("a")], &elsewhere, &StdReader::new());

            assert_eq!(missing, Err(WalkError::RootUnreadable { index: 0, kind: io::ErrorKind::NotFound }));
            assert_eq!(unknown, Err(WalkError::RootUnreadable { index: 0, kind: io::ErrorKind::NotFound }));
        }

        #[test]
        fn a_root_that_only_shares_a_prefix_is_not_inside() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n[[dir]]\npath = \"cache2\"\n").unwrap();
            let asked = [tree.root().join("cache"), tree.root().join("cache2")];

            let roots = Roots::prepare(&asked, &one_volume(), &StdReader::new()).unwrap();

            assert_eq!((roots.iter().count(), roots.dropped()), (2, 0));
            assert!(roots.holding(&canonical(&tree.root().join("cache2"))).is_some());
        }
    }

    mod positive {
        use fresher_testing::{FakeVolumes, FixtureTree};

        use super::{canonical, facts, one_volume};
        use crate::walk::roots::Roots;
        use crate::walk::std_reader::StdReader;

        #[test]
        fn a_root_is_canonical_and_identified() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n").unwrap();

            let roots = Roots::prepare(&[tree.root().join("a/../a")], &one_volume(), &StdReader::new()).unwrap();

            let root = roots.iter().next().unwrap();
            assert_eq!(root.path, canonical(&tree.root().join("a")));
            assert_eq!(root.volume, facts(1, 1));
            assert_eq!(
                root.identity.file_id,
                std::os::unix::fs::MetadataExt::ino(&std::fs::metadata(&root.path).unwrap())
            );
        }

        #[test]
        fn the_scope_key_does_not_depend_on_root_order() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n[[dir]]\npath = \"b\"\n").unwrap();
            let (a, b) = (tree.root().join("a"), tree.root().join("b"));
            let reader = StdReader::new();

            let forward = Roots::prepare(&[a.clone(), b.clone()], &one_volume(), &reader).unwrap();
            let backward = Roots::prepare(&[b, a], &one_volume(), &reader).unwrap();

            assert_eq!(forward.scope_key(), backward.scope_key());
        }

        #[test]
        fn the_scope_key_changes_with_the_roots() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n[[dir]]\npath = \"b\"\n").unwrap();
            let (a, b) = (tree.root().join("a"), tree.root().join("b"));
            let reader = StdReader::new();
            let other_volume = FakeVolumes::default().with("/", facts(2, 1));

            let one = Roots::prepare(std::slice::from_ref(&a), &one_volume(), &reader).unwrap().scope_key();
            let both = Roots::prepare(&[a.clone(), b], &one_volume(), &reader).unwrap().scope_key();
            let moved = Roots::prepare(&[a], &other_volume, &reader).unwrap().scope_key();

            assert_ne!(one, both);
            assert_ne!(one, moved);
        }
    }
}
