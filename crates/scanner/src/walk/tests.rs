//! Tests of the full walk through the `Scanner`, on fixture trees and generated trees.

use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError};

use fresher_core::{
    CancelToken, Claim, DiskWalk, FolderIdentity, FolderListing, FolderReader, Interest, InterestSet, Match, Pattern,
    ReadError, WalkError, WalkMode, WalkRequest, WalkSummary,
};
use fresher_testing::{LargeTree, TreeSpec};

use crate::matcher::testing::{interest, set_of};
use crate::scanner::{Scanner, ScannerOptions, home_anchor};
use crate::tree::arena::Tree;
use crate::walk::roots::tests::one_volume;

/// What one scan gave.
pub(crate) struct Scanned {
    pub(crate) matches: Vec<Match>,
    pub(crate) summary: Result<WalkSummary, WalkError>,
    pub(crate) scanner: Scanner,
}

impl Scanned {
    /// The frozen tree of the walk.
    pub(crate) fn tree(&self) -> Tree {
        self.scanner.last.lock().unwrap_or_else(PoisonError::into_inner).as_ref().unwrap().tree.clone()
    }
}

pub(crate) fn scanner(reader: Arc<dyn FolderReader>, home: &Path, threads: usize) -> Scanner {
    let options = ScannerOptions { threads: threads.try_into().unwrap(), ..ScannerOptions::default() };
    let journal = Arc::new(fresher_testing::FakeChangeJournal::default());
    let cache = Arc::new(fresher_testing::MemoryScanCache::with_cap(fresher_core::ByteSize::new(8 << 20)));
    Scanner::new(reader, Arc::new(one_volume()), journal, cache, home_anchor(home), options)
}

/// Scans `root` (also the home folder) for `interests`, avoiding `avoid`, with `threads` workers.
pub(crate) fn scan_with(
    reader: Arc<dyn FolderReader>,
    root: &Path,
    interests: &InterestSet,
    avoid: &[PathBuf],
    threads: usize,
    cancel: &CancelToken,
    on_progress: &mut dyn FnMut(),
) -> Scanned {
    let scanner = scanner(reader, root, threads);
    let roots = [root.to_path_buf()];
    let request = WalkRequest { roots: &roots, interests, avoid, mode: WalkMode::Full };
    let mut matches = Vec::new();
    let summary = scanner.walk(&request, cancel, &mut |found| matches.push(found), &mut |_| on_progress());
    Scanned { matches, summary, scanner }
}

pub(crate) fn scan(reader: Arc<dyn FolderReader>, root: &Path, interests: &InterestSet, threads: usize) -> Scanned {
    scan_with(reader, root, interests, &[], threads, &CancelToken::new(), &mut || {})
}

/// The interests of the walk checks: a whole folder name with siblings, an item extension, a valid tag, an anchor.
pub(crate) fn interests() -> InterestSet {
    set_of(&[
        interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
        interest(Pattern::Extension("json".into()), Claim::Item, &[]),
        interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
        interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, &[]),
    ])
}

/// The small generated tree of the tests, built once under `target/bench-trees/`.
pub(crate) fn small_tree() -> LargeTree {
    let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
    LargeTree::build(&TreeSpec::SMALL, &under).unwrap()
}

/// A generated tree of a few thousand entries in its own temporary folder.
pub(crate) fn tiny_tree(under: &Path) -> LargeTree {
    let spec = TreeSpec {
        name: "walk",
        seed: 11,
        entries: 3_000,
        depth: 4,
        fan_out: 4,
        large_file_share: 20,
        hard_links: 30,
        deep_dependency_folders: 4,
        cache_folders: 5,
    };
    LargeTree::build(&spec, under).unwrap()
}

pub(crate) fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap()
}

/// A reader that says the folder named `mount` is on another device.
struct OtherDevice<R> {
    inner: R,
    mount: &'static str,
}

impl<R: FolderReader> FolderReader for OtherDevice<R> {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        let mut listing = FolderListing::new();
        self.inner.read(path, expected, &mut listing)?;
        out.clear();
        for (name, entry) in listing.iter() {
            let mut entry = *entry;
            if name == self.mount.as_bytes() {
                entry.device += 1;
            }
            out.push(name, entry)?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

/// A reader that answers `Vanished` for the folder `gone`, as if it was removed between its listing and its read.
struct Vanishing<R> {
    inner: R,
    gone: PathBuf,
}

impl<R: FolderReader> FolderReader for Vanishing<R> {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        if path == self.gone { Err(ReadError::Vanished) } else { self.inner.read(path, expected, out) }
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

/// The interest of `Library/Caches/*` only.
fn caches() -> Vec<Interest> {
    vec![interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, &[])]
}

mod negative {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use fresher_core::{CancelToken, Claim, NotScanned, Pattern, WalkError};
    use fresher_testing::FixtureTree;

    use super::{OtherDevice, Vanishing, canonical, scan, scan_with, tiny_tree};
    use crate::matcher::testing::{interest, set_of};
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_cancelled_walk_stops_and_returns_cancelled() {
        let under = tempfile::tempdir().unwrap();
        let tree = tiny_tree(under.path());
        let reader = Arc::new(CountingReader::new(StdReader::new()));
        let cancel = CancelToken::new();
        let threads = 4;

        let scanned =
            scan_with(Arc::clone(&reader) as _, tree.root(), &super::interests(), &[], threads, &cancel, &mut || {
                cancel.cancel();
            });

        assert_eq!(scanned.summary, Err(WalkError::Cancelled));
        let folders = usize::try_from(tree.expected().folders).unwrap();
        assert!(reader.total() <= 1 + 2 * threads, "{} reads of {folders} folders", reader.total());
    }

    #[test]
    fn a_whole_claim_is_reported_only_when_its_subtree_is_read() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"app/node_modules/a/b/c/d.js\"\nsize = 10\n[[file]]\npath = \"app/node_modules/e/f.js\"\n\
             [[file]]\npath = \"app/package.json\"\n",
        )
        .unwrap();
        let reader = Arc::new(CountingReader::new(StdReader::new()));
        let root = canonical(tree.root());
        let modules = root.join("app/node_modules");
        let inside = ["", "a", "a/b", "a/b/c", "e"].map(|folder| modules.join(folder));
        let set = set_of(&[interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"])]);
        let read_all = AtomicBool::new(false);

        let scanner = super::scanner(Arc::clone(&reader) as _, &root, 3);
        let roots = [root.clone()];
        let request = fresher_core::WalkRequest {
            roots: &roots,
            interests: &set,
            avoid: &[],
            mode: fresher_core::WalkMode::Full,
        };
        let mut matches = Vec::new();
        fresher_core::DiskWalk::walk(
            &scanner,
            &request,
            &CancelToken::new(),
            &mut |found| {
                let log = reader.log();
                read_all.store(inside.iter().all(|folder| log.contains(folder)), Ordering::Relaxed);
                matches.push(found);
            },
            &mut |_| {},
        )
        .unwrap();

        assert_eq!(matches.len(), 1);
        assert!(read_all.load(Ordering::Relaxed), "the match came before its subtree was read");
        assert_eq!((matches[0].path.clone(), matches[0].sizes.logical.bytes()), (modules, 10));
        assert_eq!(matches[0].siblings, ["package.json"]);
    }

    #[test]
    fn an_avoided_folder_is_not_entered() {
        let tree =
            FixtureTree::from_toml_str("[[file]]\npath = \"Documents/a/b\"\n[[file]]\npath = \"Library/c\"\n").unwrap();
        let root = canonical(tree.root());
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let scanned = scan_with(
            Arc::clone(&reader) as _,
            &root,
            &set_of(&[]),
            &[root.join("Documents")],
            2,
            &CancelToken::new(),
            &mut || {},
        );

        let tree = scanned.tree();
        let summary = scanned.summary.unwrap();
        assert!(reader.log().iter().all(|folder| !folder.starts_with(root.join("Documents"))));
        assert_eq!(summary.not_scanned.get(&NotScanned::Avoided), Some(&1));
        assert_eq!(summary.entries, 3, "Documents, Library and Library/c");
        let documents = tree.lookup(&root.join("Documents")).unwrap();
        assert_eq!(tree.completeness(documents), Some(fresher_core::Completeness::Partial));
    }

    #[test]
    fn an_avoided_root_is_not_entered() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
        let root = canonical(tree.root());
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let scanned = scan_with(
            Arc::clone(&reader) as _,
            &root,
            &set_of(&[]),
            std::slice::from_ref(&root),
            2,
            &CancelToken::new(),
            &mut || {},
        );

        let summary = scanned.summary.unwrap();
        assert_eq!((reader.total(), summary.entries), (0, 0));
        assert_eq!(summary.not_scanned.get(&NotScanned::Avoided), Some(&1));
    }

    #[test]
    fn a_folder_on_another_device_is_not_entered() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"mnt/inside/x.json\"\n[[file]]\npath = \"y.json\"\n")
            .unwrap();
        let root = canonical(tree.root());
        let reader = Arc::new(CountingReader::new(OtherDevice { inner: StdReader::new(), mount: "mnt" }));
        let set = set_of(&[interest(Pattern::Extension("json".into()), Claim::Item, &[])]);

        let scanned = scan(Arc::clone(&reader) as _, &root, &set, 2);

        let summary = scanned.summary.unwrap();
        assert_eq!(reader.log(), std::slice::from_ref(&root));
        assert_eq!(summary.not_scanned.get(&NotScanned::OtherVolume), Some(&1));
        assert_eq!(summary.entries, 1, "only y.json");
        assert_eq!(scanned.matches.iter().map(|found| found.path.clone()).collect::<Vec<_>>(), [root.join("y.json")]);
        assert!(summary.is_complete());
    }

    #[test]
    fn an_unreadable_folder_is_reported_by_kind_and_makes_its_parents_partial() {
        use std::os::unix::fs::PermissionsExt;
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a/b/locked/f\"\n[[file]]\npath = \"c/g\"\n").unwrap();
        let root = canonical(tree.root());
        let locked = root.join("a/b/locked");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        let scanned = scan(Arc::new(StdReader::new()), &root, &set_of(&[]), 2);
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();

        let tree = scanned.tree();
        let summary = scanned.summary.unwrap();
        assert_eq!(summary.unreadable.get(&fresher_core::Unreadable::NoPermission), Some(&1));
        let node = tree.lookup(&locked).unwrap();
        assert_eq!(tree.node(node).unwrap().flags.unreadable(), Some(fresher_core::Unreadable::NoPermission));
        for partial in [locked.clone(), root.join("a/b"), root.join("a"), root.clone()] {
            assert_eq!(
                tree.completeness(tree.lookup(&partial).unwrap()),
                Some(fresher_core::Completeness::Partial),
                "{partial:?}"
            );
        }
        assert_eq!(
            tree.completeness(tree.lookup(&root.join("c")).unwrap()),
            Some(fresher_core::Completeness::Complete)
        );
    }

    #[test]
    fn a_folder_removed_during_the_walk_is_not_an_error_of_the_walk() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"gone/f\"\n[[file]]\npath = \"kept/g\"\n").unwrap();
        let root = canonical(tree.root());
        let reader = Vanishing { inner: StdReader::new(), gone: root.join("gone") };

        let scanned = scan(Arc::new(reader), &root, &set_of(&[]), 2);

        let summary = scanned.summary.unwrap();
        assert_eq!(summary.unreadable.get(&fresher_core::Unreadable::Failed), Some(&1));
        assert_eq!(summary.entries, 3, "gone, kept and kept/g");
    }

    #[test]
    fn a_restore_has_nothing_to_restore_yet() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
        let scanner = super::scanner(Arc::new(StdReader::new()), tree.root(), 1);
        let (roots, set) = ([tree.root().to_path_buf()], set_of(&[]));
        let request = fresher_core::WalkRequest {
            roots: &roots,
            interests: &set,
            avoid: &[],
            mode: fresher_core::WalkMode::Restore,
        };

        let result = fresher_core::DiskWalk::walk(&scanner, &request, &CancelToken::new(), &mut |_| {}, &mut |_| {});

        assert_eq!(result, Err(WalkError::NothingToRestore));
    }
}

mod positive {
    use std::collections::HashSet;
    use std::path::Path;
    use std::sync::Arc;

    use fresher_testing::FixtureTree;

    use super::{caches, canonical, scan, small_tree, tiny_tree};
    use crate::matcher::testing::set_of;
    use crate::tree::testing::canonical as shapes;
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn every_folder_is_read_exactly_once() {
        let tree = small_tree();
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let scanned = scan(Arc::clone(&reader) as _, tree.root(), &super::interests(), 8);

        scanned.summary.unwrap();
        let reads = reader.reads();
        assert!(reads.values().all(|&count| count == 1));
        assert_eq!(reads.len() as u64, tree.expected().folders + 1, "every folder and the root");
    }

    #[test]
    fn the_macos_reader_and_the_std_reader_build_the_same_tree() {
        let tree = small_tree();

        let fast = scan(Arc::new(fresher_macos::MacFolderReader::new()), tree.root(), &super::interests(), 8);
        let portable = scan(Arc::new(StdReader::new()), tree.root(), &super::interests(), 8);

        assert_eq!(fast.summary, portable.summary);
        assert_eq!(shapes(&fast.tree()), shapes(&portable.tree()));
    }

    #[test]
    fn a_clone_shares_its_blocks_once_in_the_tree() {
        let folder = tempfile::tempdir().unwrap();
        let root = canonical(folder.path());
        std::fs::create_dir(root.join("copies")).unwrap();
        std::fs::write(root.join("original"), vec![7_u8; 256 * 1024]).unwrap();
        fresher_macos::fixtures::clone_file(&root.join("original"), &root.join("copies/clone")).unwrap();
        let allocated =
            std::os::unix::fs::MetadataExt::blocks(&std::fs::metadata(root.join("original")).unwrap()) * 512;

        let fast = scan(Arc::new(fresher_macos::MacFolderReader::new()), &root, &set_of(&[]), 2);
        let portable = scan(Arc::new(StdReader::new()), &root, &set_of(&[]), 2);

        let (fast, portable) = (fast.tree(), portable.tree());
        let root_sums = |tree: &crate::tree::arena::Tree| tree.sums(tree.roots()[0].node).unwrap();
        assert_eq!(root_sums(&fast).allocated, allocated, "the shared blocks count once");
        assert_eq!(root_sums(&fast).reclaimable, 0, "removing either copy frees nothing");
        assert_eq!(root_sums(&portable).allocated, 2 * allocated, "the portable reader cannot see clones");
        assert_eq!(root_sums(&fast).files, 2);
    }

    #[test]
    fn a_root_written_with_a_tilde_is_under_the_home_folder() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Projects/x.json\"\n").unwrap();
        let home = canonical(tree.root());
        let scanner = super::scanner(Arc::new(StdReader::new()), &home, 1);
        let set = set_of(&[crate::matcher::testing::interest(
            fresher_core::Pattern::Extension("json".into()),
            fresher_core::Claim::Item,
            &[],
        )]);
        let mut found = Vec::new();

        for root in ["~", "~/Projects"] {
            let roots = [std::path::PathBuf::from(root)];
            let request = fresher_core::WalkRequest {
                roots: &roots,
                interests: &set,
                avoid: &[],
                mode: fresher_core::WalkMode::Auto,
            };
            fresher_core::DiskWalk::walk(
                &scanner,
                &request,
                &fresher_core::CancelToken::new(),
                &mut |matched| found.push(matched.path),
                &mut |_| {},
            )
            .unwrap();
        }

        assert_eq!(found, [home.join("Projects/x.json"), home.join("Projects/x.json")]);
    }

    #[test]
    fn a_bulk_read_per_folder() {
        let under = tempfile::tempdir().unwrap();
        let tree = tiny_tree(under.path());
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        scan(Arc::clone(&reader) as _, tree.root(), &super::interests(), 4).summary.unwrap();

        assert_eq!(reader.total() as u64, tree.expected().folders + 1);
    }

    #[test]
    fn interesting_folders_are_read_before_the_rest() {
        let tree = FixtureTree::from_toml_str(
            "[[dir]]\npath = \"Data/a/b/c/d\"\n[[dir]]\npath = \"Data/e/f/g\"\n[[dir]]\npath = \"Library/Caches/x/y\"\n\
             [[dir]]\npath = \"Library/Caches/z\"\n[[dir]]\npath = \"Library/Logs/w\"\n[[dir]]\npath = \"Other/h/i\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        scan(Arc::clone(&reader) as _, &root, &set_of(&caches()), 1).summary.unwrap();

        let interesting: HashSet<_> = ["Library", "Library/Caches", "Library/Caches/x", "Library/Caches/z"]
            .iter()
            .map(|folder| root.join(folder))
            .collect();
        let depth = |folder: &Path| folder.strip_prefix(&root).unwrap().components().count();
        let log = reader.log();
        for (position, folder) in log.iter().enumerate().filter(|(_, folder)| interesting.contains(*folder)) {
            let early = log[..position].iter().filter(|other| !interesting.contains(*other) && **other != root);
            for other in early {
                assert!(depth(other) < depth(folder), "{other:?} was read before {folder:?}");
            }
        }
    }

    #[test]
    fn one_thread_and_many_threads_build_the_same_tree() {
        let under = tempfile::tempdir().unwrap();
        let tree = tiny_tree(under.path());

        let one = scan(Arc::new(StdReader::new()), tree.root(), &super::interests(), 1);
        let many = scan(Arc::new(StdReader::new()), tree.root(), &super::interests(), 8);

        assert_eq!(one.summary, many.summary);
        assert_eq!(shapes(&one.tree()), shapes(&many.tree()));
        let mut one_matches = one.matches;
        let mut many_matches = many.matches;
        one_matches.sort_by(|left, right| (&left.path, left.interest).cmp(&(&right.path, right.interest)));
        many_matches.sort_by(|left, right| (&left.path, left.interest).cmp(&(&right.path, right.interest)));
        assert_eq!(one_matches, many_matches);
        assert_ne!(one_matches.len(), 0);
    }
}
