//! The read queries of a state: the largest files and search by name.

use std::path::Path;

use fresher_core::{EntryKind, FolderIdentity, StateId, Timestamp};

use crate::state::TreeState;
use crate::tree::arena::{Shared, Tree};
use crate::tree::node::{NodeIndex, Sums};

pub(super) const MIB: u64 = 1 << 20;

/// A tree built in memory, one entry at a time, then frozen into a state.
pub(super) struct Built {
    pub(super) tree: Tree,
    pub(super) root: NodeIndex,
}

impl Built {
    pub(super) fn new() -> Self {
        let mut tree = Tree::new();
        let root = tree.add_root(Path::new("/home"), FolderIdentity { device: 1, file_id: 2 }).unwrap();
        Self { tree, root }
    }

    pub(super) fn folder(&mut self, parent: NodeIndex, name: &str) -> NodeIndex {
        self.tree.add_child(parent, name.as_bytes(), EntryKind::Folder).unwrap()
    }

    /// A kept file of `allocated` bytes.
    pub(super) fn file(&mut self, parent: NodeIndex, name: &str, allocated: u64) -> NodeIndex {
        let file = self.tree.add_child(parent, name.as_bytes(), EntryKind::File).unwrap();
        self.tree.add_own(file, sums(allocated)).unwrap();
        file
    }

    /// A kept file of `allocated` bytes, a name of the hard link group `id`.
    pub(super) fn hard_link(&mut self, parent: NodeIndex, name: &str, id: u64, allocated: u64) -> NodeIndex {
        let file = self.tree.add_child(parent, name.as_bytes(), EntryKind::File).unwrap();
        self.tree.add_shared(Shared::HardLink, 1, id, file, sums(allocated)).unwrap();
        file
    }

    fn state(mut self) -> TreeState {
        self.tree.freeze();
        TreeState::new(StateId::new(1), Timestamp::from_secs(0), self.tree, Vec::new())
    }
}

pub(super) const fn sums(allocated: u64) -> Sums {
    Sums { files: 0, logical: allocated, allocated, reclaimable: 0 }
}

fn names(nodes: &[fresher_core::NodeInfo]) -> Vec<&str> {
    nodes.iter().map(|node| node.name.as_str()).collect()
}

mod negative {
    use fresher_core::DiskState;
    use fresher_testing::FixtureTree;

    use super::{Built, MIB, names};
    use crate::state::tests::committed;

    #[test]
    fn an_empty_query_finds_nothing() {
        let mut built = Built::new();
        let root = built.root;
        built.folder(root, " ");
        let state = built.state();
        let before = state.heap_bytes();

        assert_eq!(names(&state.search("", 10)), Vec::<&str>::new());
        assert_eq!(names(&state.search("  \t", 10)), Vec::<&str>::new());
        assert_eq!(state.heap_bytes(), before, "a query that finds nothing builds no copy");
    }

    #[test]
    fn a_query_matches_no_folded_small_file() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"docs/report.txt\"\nsize = 10\n").unwrap();

        let (_, state) = committed(&tree, &[], &[]);

        assert_eq!(names(&state.search("report", 10)), Vec::<&str>::new());
        assert_eq!(names(&state.search("docs", 10)), ["docs"]);
    }

    #[test]
    fn a_hard_link_without_bytes_is_not_among_the_largest() {
        let mut built = Built::new();
        let root = built.root;
        let (a, b) = (built.folder(root, "a"), built.folder(root, "b"));
        built.hard_link(a, "first", 7, 3 * MIB);
        built.hard_link(b, "second", 7, 3 * MIB);
        built.file(root, "plain", MIB);
        let state = built.state();

        assert_eq!(names(&state.largest_files(10)), ["first", "plain"]);
    }

    #[test]
    fn a_limit_of_zero_answers_nothing() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "big", MIB);
        let state = built.state();

        assert_eq!((state.largest_files(0), state.search("big", 0)), (vec![], vec![]));
    }

    #[test]
    fn a_query_past_the_longest_is_cut() {
        let mut built = Built::new();
        let root = built.root;
        built.folder(root, &"é".repeat(127));
        let state = built.state();

        let found = state.search(&format!("{}x", "É".repeat(200)), 10);

        assert_eq!(found.len(), 1, "cut to 254 bytes, the last character boundary within 255");
    }
}

mod positive {
    use fresher_core::{DiskState, NodeKind};
    use fresher_testing::FixtureTree;

    use super::{Built, MIB, names};
    use crate::state::tests::committed;

    #[test]
    fn search_ignores_case() {
        let mut built = Built::new();
        let root = built.root;
        built.folder(root, "Library");
        built.file(root, "ÉTÉ.MOV", MIB);
        let state = built.state();

        assert_eq!(names(&state.search("LIBR", 10)), ["Library"]);
        assert_eq!(names(&state.search("été", 10)), ["ÉTÉ.MOV"]);
    }

    #[test]
    fn search_ranks_by_size() {
        let mut built = Built::new();
        let root = built.root;
        let small = built.folder(root, "cache-small");
        built.file(small, "x", MIB);
        let large = built.folder(root, "cache-large");
        built.file(large, "y", 5 * MIB);
        built.file(root, "cache.bin", 3 * MIB);
        let same = built.folder(root, "b-cache");
        built.file(same, "z", MIB);
        let state = built.state();

        assert_eq!(names(&state.search("cache", 10)), ["cache-large", "cache.bin", "b-cache", "cache-small"]);
        assert_eq!(names(&state.search("cache", 2)), ["cache-large", "cache.bin"]);
    }

    #[test]
    fn search_finds_folders_and_kept_files() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"movies/movie.mov\"\nsize = 10\n").unwrap();
        std::fs::write(tree.root().join("movies/movie.mov"), vec![1_u8; 2 << 20]).unwrap();

        let (_, state) = committed(&tree, &[], &[]);

        let found = state.search("MOVIE", 10);
        let kinds: Vec<_> = found.iter().map(|node| (node.name.as_str(), node.kind)).collect();
        assert_eq!(kinds, [("movies", NodeKind::Folder), ("movie.mov", NodeKind::File)]);
    }

    #[test]
    fn largest_files_are_in_size_order() {
        let mut built = Built::new();
        let root = built.root;
        let deep = built.folder(root, "deep");
        built.file(deep, "two", 2 * MIB);
        built.file(root, "one", MIB);
        built.file(root, "four", 4 * MIB);
        built.file(deep, "also-two", 2 * MIB);
        let state = built.state();

        assert_eq!(names(&state.largest_files(10)), ["four", "also-two", "two", "one"]);
        assert_eq!(names(&state.largest_files(2)), ["four", "also-two"]);
        assert!(state.largest_files(u32::MAX).len() <= 500);
    }

    #[test]
    fn the_lowercase_copy_is_built_once() {
        let mut built = Built::new();
        let root = built.root;
        built.folder(root, "Library");
        let state = built.state();
        let before = state.heap_bytes();

        state.search("lib", 10);
        let after_first = state.heap_bytes();
        state.search("other", 10);

        assert!(after_first > before);
        assert_eq!(state.heap_bytes(), after_first);
    }
}
