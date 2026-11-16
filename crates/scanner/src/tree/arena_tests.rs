mod negative {
    use std::path::Path;

    use fresher_core::{Completeness, EntryKind, FolderIdentity, Unreadable};

    use crate::tree::TreeError;
    use crate::tree::arena::Tree;
    use crate::tree::node::{Flag, NodeIndex};
    use crate::tree::testing::{rooted, sums};

    #[test]
    fn a_name_too_long_is_refused() {
        let (mut tree, root) = rooted("/home");
        let long = vec![b'a'; usize::from(u16::MAX) + 1];
        assert_eq!(tree.add_child(root, &long, EntryKind::File), Err(TreeError::NameTooLong { len: long.len() }));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.children(root).count(), 0);
    }

    #[test]
    fn a_file_or_a_missing_node_takes_no_child() {
        let (mut tree, root) = rooted("/home");
        let file = tree.add_child(root, b"a.bin", EntryKind::File).unwrap();
        assert_eq!(tree.add_child(file, b"x", EntryKind::File), Err(TreeError::NotAFolder));
        assert_eq!(tree.add_child(NodeIndex::NONE, b"x", EntryKind::File), Err(TreeError::NotAFolder));
        assert_eq!(tree.add_own(NodeIndex::NONE, sums(1, 1, 1, 1)), Err(TreeError::NoSuchNode));
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn a_frozen_tree_is_not_built_any_further() {
        let (mut tree, root) = rooted("/home");
        tree.freeze();
        assert_eq!(tree.add_child(root, b"late", EntryKind::Folder), Err(TreeError::Frozen));
        assert_eq!(tree.add_own(root, sums(1, 1, 1, 1)), Err(TreeError::Frozen));
        assert_eq!(tree.mark_unreadable(root, Unreadable::Failed), Err(TreeError::Frozen));
        let mut other = Tree::new();
        other.freeze();
        assert_eq!(other.add_root(Path::new("/x"), FolderIdentity { device: 1, file_id: 1 }), Err(TreeError::Frozen));
    }

    #[test]
    fn lookup_of_a_path_outside_every_root_is_none() {
        let (mut tree, root) = rooted("/home/user");
        tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        tree.freeze();
        assert_eq!(tree.lookup(Path::new("/home/other/Library")), None);
        assert_eq!(tree.lookup(Path::new("/home")), None);
        assert_eq!(tree.lookup(Path::new("/home/user/Missing")), None);
        assert_eq!(tree.lookup(Path::new("/home/user/Library/../Library")), None);
    }

    #[test]
    fn lookup_does_not_match_a_prefix_of_a_name() {
        let (mut tree, root) = rooted("/home/user");
        tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        tree.freeze();
        assert_eq!(tree.lookup(Path::new("/home/user/Lib")), None);
        assert_eq!(tree.lookup(Path::new("/home/us/Library")), None);
    }

    #[test]
    fn an_unreadable_node_makes_every_ancestor_partial() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let mail = tree.add_child(library, b"Mail", EntryKind::Folder).unwrap();
        let sibling = tree.add_child(root, b"Projects", EntryKind::Folder).unwrap();
        tree.mark_unreadable(mail, Unreadable::Guarded).unwrap();
        tree.freeze();
        for index in [mail, library, root] {
            assert_eq!(tree.completeness(index), Some(Completeness::Partial));
        }
        assert_eq!(tree.completeness(sibling), Some(Completeness::Complete));
        assert_eq!(tree.node(mail).unwrap().flags.unreadable(), Some(Unreadable::Guarded));
        assert!(tree.node(root).unwrap().flags.has(Flag::Partial));
    }

    #[test]
    fn an_avoided_or_incomplete_folder_and_an_overflow_are_partial() {
        let (mut tree, root) = rooted("/home");
        let avoided = tree.add_child(root, b"Documents", EntryKind::Folder).unwrap();
        let incomplete = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let full = tree.add_child(root, b"Big", EntryKind::Folder).unwrap();
        tree.mark(avoided, Flag::Avoided).unwrap();
        tree.mark(incomplete, Flag::Incomplete).unwrap();
        tree.add_own(full, sums(0, u64::MAX, 0, 0)).unwrap();
        tree.add_own(full, sums(0, 1, 0, 0)).unwrap();
        tree.freeze();
        for index in [avoided, incomplete, full, root] {
            assert_eq!(tree.completeness(index), Some(Completeness::Partial));
        }
        assert_eq!(tree.sums(full).unwrap().logical, u64::MAX);
    }
}

mod positive {
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    use fresher_core::{EntryKind, FolderIdentity, MarkerCheck, Timestamp};

    use crate::tree::arena::Tree;
    use crate::tree::node::{Hit, NodeIndex};
    use crate::tree::testing::{canonical, rooted, sums};

    fn names(tree: &Tree, folder: NodeIndex) -> Vec<Vec<u8>> {
        tree.children(folder).map(|child| tree.name(child).to_vec()).collect()
    }

    #[test]
    fn freeze_sorts_children_by_name() {
        let (mut tree, root) = rooted("/home");
        for name in [&b"b"[..], b"a", b"\xff", b"B", b"ab"] {
            tree.add_child(root, name, EntryKind::Folder).unwrap();
        }
        assert_eq!(names(&tree, root), [&b"ab"[..], b"B", b"\xff", b"a", b"b"]);
        tree.freeze();
        assert_eq!(names(&tree, root), [&b"B"[..], b"a", b"ab", b"b", b"\xff"]);
    }

    #[test]
    fn freeze_sums_every_subtree() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let caches = tree.add_child(library, b"Caches", EntryKind::Folder).unwrap();
        let big = tree.add_child(caches, b"big.bin", EntryKind::File).unwrap();
        let link = tree.add_child(root, b"link", EntryKind::Symlink).unwrap();
        tree.add_own(root, sums(3, 30, 40, 40)).unwrap();
        tree.add_own(caches, sums(2, 200, 400, 300)).unwrap();
        tree.add_own(big, sums(0, 5_000_000, 5_242_880, 5_242_880)).unwrap();
        tree.add_hidden_folders(caches, 7).unwrap();
        tree.freeze();

        assert_eq!(tree.sums(big), Some(sums(1, 5_000_000, 5_242_880, 5_242_880)));
        assert_eq!(tree.sums(link), Some(sums(1, 0, 0, 0)));
        assert_eq!(tree.sums(caches), Some(sums(3, 5_000_200, 5_243_280, 5_243_180)));
        assert_eq!(tree.sums(library), tree.sums(caches));
        assert_eq!(tree.sums(root), Some(sums(7, 5_000_230, 5_243_320, 5_243_220)));
        let folders = |index| tree.node(index).unwrap().folders;
        assert_eq!((folders(root), folders(library), folders(caches), folders(big)), (9, 8, 7, 0));
    }

    #[test]
    fn path_rebuilds_the_full_path() {
        let (mut tree, root) = rooted("/home/user");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let odd = tree.add_child(library, b"\xe9t\xe9", EntryKind::File).unwrap();
        tree.freeze();
        assert_eq!(tree.path(root), Some(PathBuf::from("/home/user")));
        assert_eq!(tree.path(library), Some(PathBuf::from("/home/user/Library")));
        assert_eq!(tree.path(odd).unwrap().as_os_str().as_bytes(), b"/home/user/Library/\xe9t\xe9");
        assert_eq!(tree.path(NodeIndex::NONE), None);
    }

    #[test]
    fn lookup_finds_every_node_by_its_path() {
        let mut tree = Tree::new();
        let first = tree.add_root(Path::new("/a"), FolderIdentity { device: 1, file_id: 1 }).unwrap();
        let second = tree.add_root(Path::new("/b/c"), FolderIdentity { device: 1, file_id: 2 }).unwrap();
        let inner = tree.add_child(second, b"d", EntryKind::Folder).unwrap();
        let file = tree.add_child(inner, b"e.bin", EntryKind::File).unwrap();
        tree.freeze();
        for index in [first, second, inner, file] {
            assert_eq!(tree.lookup(&tree.path(index).unwrap()), Some(index));
        }
        assert_eq!(tree.roots().len(), 2);
    }

    #[test]
    fn children_added_in_any_order_freeze_to_the_same_tree() {
        let build = |order: &[&[u8]]| {
            let (mut tree, root) = rooted("/home");
            for &name in order {
                let folder = tree.add_child(root, name, EntryKind::Folder).unwrap();
                tree.add_own(folder, sums(1, name.len() as u64, 4096, 4096)).unwrap();
                tree.add_child(folder, b"inner", EntryKind::Folder).unwrap();
            }
            tree.freeze();
            tree
        };
        let forward = build(&[b"a", b"bb", b"ccc"]);
        let backward = build(&[b"ccc", b"a", b"bb"]);
        assert_eq!(canonical(&forward), canonical(&backward));
    }

    #[test]
    fn hits_and_times_stay_with_their_node() {
        let (mut tree, root) = rooted("/home");
        let first = tree.add_child(root, b"node_modules", EntryKind::Folder).unwrap();
        let second = tree.add_child(root, b"target", EntryKind::Folder).unwrap();
        let hit = |interest| Hit { interest, marker: None, siblings: 0b1 };
        tree.add_hits(first, &[hit(0)]).unwrap();
        tree.add_hits(second, &[hit(1), hit(2)]).unwrap();
        tree.add_hits(second, &[hit(3)]).unwrap();
        tree.add_hits(first, &[Hit { interest: 4, marker: Some(MarkerCheck::Valid), siblings: 0 }]).unwrap();
        tree.add_hits(first, &[]).unwrap();
        tree.set_modified(first, Some(Timestamp::from_secs(7))).unwrap();
        tree.freeze();
        let interests = |index| tree.hits(index).iter().map(|hit| hit.interest).collect::<Vec<_>>();
        assert_eq!(interests(first), [0, 4]);
        assert_eq!(interests(second), [1, 2, 3]);
        assert_eq!(tree.node(first).unwrap().modified(), Some(Timestamp::from_secs(7)));
    }

    #[test]
    fn freeze_twice_changes_nothing() {
        let (mut tree, root) = rooted("/home");
        let folder = tree.add_child(root, b"a", EntryKind::Folder).unwrap();
        tree.add_own(folder, sums(2, 10, 20, 20)).unwrap();
        tree.freeze();
        let once = canonical(&tree);
        tree.freeze();
        assert_eq!(canonical(&tree), once);
    }

    #[test]
    fn heap_bytes_per_node_stays_under_96() {
        const NODES: usize = 100_000;
        let (mut tree, root) = rooted("/home/user");
        let mut folders = vec![root];
        let mut next_folder = 0;
        let mut name = Vec::new();
        while tree.len() < NODES {
            let parent = folders[next_folder / 8];
            next_folder += 1;
            name.clear();
            // Names of 4 to 20 bytes, 12 on average.
            name.resize(4 + tree.len() % 17, b'n');
            let kind = if tree.len() % 20 == 0 { EntryKind::File } else { EntryKind::Folder };
            let child = tree.add_child(parent, &name, kind).unwrap();
            tree.add_own(child, sums(3, 1000, 4096, 4096)).unwrap();
            if kind == EntryKind::Folder {
                folders.push(child);
            }
        }
        tree.freeze();
        let per_node = tree.heap_bytes() / tree.len();
        assert!(per_node < 96, "{per_node} bytes a node");
    }
}

mod property {
    use fresher_core::EntryKind;
    use proptest::prelude::*;

    use crate::tree::arena::Tree;
    use crate::tree::node::{NodeIndex, Sums};
    use crate::tree::testing::{canonical, rooted, sums};

    /// One node of a random tree: which earlier folder is its parent, whether it is a folder, and its own bytes.
    type Spec = (usize, bool, Sums);

    fn specs() -> impl Strategy<Value = Vec<Spec>> {
        let own = (0..1000_u32, 0..1_u64 << 40, 0..1_u64 << 40, 0..1_u64 << 40)
            .prop_map(|(files, logical, allocated, reclaimable)| sums(files, logical, allocated, reclaimable));
        prop::collection::vec((any::<usize>(), any::<bool>(), own), 0..200)
    }

    /// Builds the tree of `specs`; in `reversed`, every folder's children are added in the opposite order.
    fn build(specs: &[Spec], reversed: bool) -> Tree {
        let (mut tree, root) = rooted("/r");
        let mut parents = vec![None; specs.len()];
        let mut folders: Vec<Option<usize>> = vec![None];
        for (position, &(choice, folder, _)) in specs.iter().enumerate() {
            parents[position] = folders[choice % folders.len()];
            if folder {
                folders.push(Some(position));
            }
        }
        let mut indices: Vec<NodeIndex> = vec![NodeIndex::NONE; specs.len()];
        let mut queue = vec![None];
        while let Some(parent) = queue.pop() {
            let mut children: Vec<usize> = (0..specs.len()).filter(|&child| parents[child] == parent).collect();
            if reversed {
                children.reverse();
            }
            let parent_index = parent.map_or(root, |parent| indices[parent]);
            for child in children {
                let (_, folder, own) = specs[child];
                let kind = if folder { EntryKind::Folder } else { EntryKind::File };
                let index = tree.add_child(parent_index, format!("n{child}").as_bytes(), kind).unwrap();
                let own = if folder { own } else { Sums { files: 0, ..own } };
                tree.add_own(index, own).unwrap();
                indices[child] = index;
                if folder {
                    queue.push(Some(child));
                }
            }
        }
        tree.freeze();
        tree
    }

    proptest! {
        #[test]
        fn totals_equal_the_sum_of_the_leaves(specs in specs()) {
            let tree = build(&specs, false);
            let expected = specs.iter().fold(Sums::ZERO, |total, &(_, folder, own)| {
                let own = if folder { own } else { Sums { files: 1, ..own } };
                total.checked_add(own).unwrap()
            });
            let root = tree.roots()[0].node;
            prop_assert_eq!(tree.sums(root), Some(expected));
            let folders = specs.iter().filter(|&&(_, folder, _)| folder).count();
            prop_assert_eq!(tree.node(root).unwrap().folders as usize, folders);
        }

        #[test]
        fn freeze_is_idempotent(specs in specs()) {
            let mut tree = build(&specs, false);
            let once = canonical(&tree);
            tree.freeze();
            prop_assert_eq!(canonical(&tree), once);
        }

        #[test]
        fn build_order_does_not_change_the_frozen_tree(specs in specs()) {
            prop_assert_eq!(canonical(&build(&specs, false)), canonical(&build(&specs, true)));
        }
    }
}
