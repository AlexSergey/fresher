//! The items query over a frozen tree.

use std::cell::RefCell;

use fresher_core::{
    ByteSize, CategoryId, DiskState, EntryKind, ItemRule, ItemVisit, ModuleId, NodeId, NodeKind, StateId, Timestamp,
};

use super::query_tests::{Built, MIB, sums};
use crate::state::TreeState;
use crate::tree::node::NodeIndex;

/// A rule that answers by name (anything not named is entered) and records what it was asked.
struct Named {
    answers: Vec<(&'static str, ItemVisit)>,
    asked: RefCell<Vec<(String, NodeKind, u32)>>,
}

impl Named {
    fn new(answers: &[(&'static str, ItemVisit)]) -> Self {
        Self { answers: answers.to_vec(), asked: RefCell::new(Vec::new()) }
    }

    fn asked_names(&self) -> Vec<String> {
        self.asked.borrow().iter().map(|(name, _, _)| name.clone()).collect()
    }
}

impl ItemRule for Named {
    fn visit(&self, name: &str, kind: NodeKind, depth: u32) -> ItemVisit {
        self.asked.borrow_mut().push((name.to_owned(), kind, depth));
        self.answers.iter().find(|(named, _)| *named == name).map_or(ItemVisit::Enter, |(_, visit)| *visit)
    }
}

/// A rule that calls every folder entered and every file an item.
struct Everything;

impl ItemRule for Everything {
    fn visit(&self, _: &str, _: NodeKind, _: u32) -> ItemVisit {
        ItemVisit::Enter
    }
}

/// Adds an entry of `kind` with `allocated` bytes of its own.
fn entry(built: &mut Built, parent: NodeIndex, name: &str, kind: EntryKind, allocated: u64) -> NodeIndex {
    let node = built.tree.add_child(parent, name.as_bytes(), kind).unwrap();
    built.tree.add_own(node, sums(allocated)).unwrap();
    node
}

/// Marks `node` claimed by the state's only category.
fn claim(built: &mut Built, node: NodeIndex) {
    built.tree.set_owner(node, 0).unwrap();
}

/// The frozen state, its only category `demo.claimed`.
fn state(mut built: Built) -> TreeState {
    built.tree.freeze();
    let owner = CategoryId::new(ModuleId::new("demo").unwrap(), "demo.claimed").unwrap();
    TreeState::new(StateId::new(1), Timestamp::from_secs(0), built.tree, vec![owner])
}

fn root_of(state: &TreeState) -> NodeId {
    state.roots()[0]
}

/// The names `items` answers from the root, at least `min` MiB, at most `limit`.
fn answered(state: &TreeState, rule: &dyn ItemRule, min: u64, limit: u32) -> Vec<String> {
    let found = state.items(root_of(state), rule, ByteSize::new(min * MIB), limit).unwrap();
    found.into_iter().map(|id| state.node(id).unwrap().name).collect()
}

mod negative {
    use fresher_core::{ByteSize, DiskState, EntryKind, ItemVisit, NodeId, StateId};

    use super::{Built, Everything, MIB, Named, answered, claim, entry, root_of, state};

    #[test]
    fn a_node_of_another_state_answers_none() {
        let state = state(Built::new());

        assert_eq!(state.items(NodeId::new(StateId::new(2), 0), &Everything, ByteSize::new(0), 10), None);
        assert_eq!(state.items(NodeId::new(StateId::new(1), 99), &Everything, ByteSize::new(0), 10), None);
    }

    #[test]
    fn the_root_itself_is_never_an_item() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "movie.mov", 3 * MIB);
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 1, 10), ["movie.mov"]);
        assert_eq!(rule.asked_names(), ["movie.mov"]);
    }

    #[test]
    fn a_claimed_folder_is_neither_entered_nor_answered() {
        let mut built = Built::new();
        let root = built.root;
        let caches = built.folder(root, "Caches");
        built.file(caches, "blob", 5 * MIB);
        claim(&mut built, caches);
        let state = state(built);
        let rule = Named::new(&[("Caches", ItemVisit::Item)]);

        assert_eq!(answered(&state, &rule, 1, 10), Vec::<String>::new());
        assert_eq!(rule.asked_names(), Vec::<String>::new());
    }

    #[test]
    fn a_claimed_file_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let log = built.file(root, "big.log", 5 * MIB);
        built.file(root, "movie.mov", 2 * MIB);
        claim(&mut built, log);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 1, 10), ["movie.mov"]);
    }

    #[test]
    fn a_link_is_never_asked_about() {
        let mut built = Built::new();
        let root = built.root;
        entry(&mut built, root, "shortcut", EntryKind::Symlink, 5 * MIB);
        let state = state(built);
        let rule = Named::new(&[("shortcut", ItemVisit::Item)]);

        assert_eq!(answered(&state, &rule, 0, 10), Vec::<String>::new());
        assert_eq!(rule.asked_names(), Vec::<String>::new());
    }

    #[test]
    fn a_special_entry_is_never_asked_about() {
        let mut built = Built::new();
        let root = built.root;
        entry(&mut built, root, "socket", EntryKind::Other, 5 * MIB);
        let state = state(built);
        let rule = Named::new(&[("socket", ItemVisit::Item)]);

        assert_eq!(answered(&state, &rule, 0, 10), Vec::<String>::new());
        assert_eq!(rule.asked_names(), Vec::<String>::new());
    }

    #[test]
    fn a_hard_link_counted_elsewhere_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let (a, b) = (built.folder(root, "a"), built.folder(root, "b"));
        built.hard_link(a, "first", 7, 3 * MIB);
        built.hard_link(b, "second", 7, 3 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 10), ["first"]);
    }

    #[test]
    fn a_skipped_folder_is_not_entered() {
        let mut built = Built::new();
        let root = built.root;
        let hidden = built.folder(root, ".hidden");
        built.file(hidden, "secret.bin", 5 * MIB);
        let state = state(built);
        let rule = Named::new(&[(".hidden", ItemVisit::Skip)]);

        assert_eq!(answered(&state, &rule, 1, 10), Vec::<String>::new());
        assert_eq!(rule.asked_names(), [".hidden"]);
    }

    #[test]
    fn an_item_below_the_minimum_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "small.bin", 2 * MIB);
        built.file(root, "large.bin", 4 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 3, 10), ["large.bin"]);
    }

    #[test]
    fn a_folder_below_the_minimum_is_not_entered() {
        let mut built = Built::new();
        let root = built.root;
        let small = built.folder(root, "small");
        built.file(small, "a.bin", MIB);
        built.file(small, "b.bin", MIB);
        built.file(root, "large.bin", 4 * MIB);
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 3, 10), ["large.bin"]);
        assert_eq!(rule.asked_names(), ["large.bin"]);
    }

    #[test]
    fn a_claimed_root_answers_nothing() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "movie.mov", 3 * MIB);
        claim(&mut built, root);
        let state = state(built);

        assert_eq!(state.items(root_of(&state), &Everything, ByteSize::new(0), 10), Some(Vec::new()));
    }
}

mod positive {
    use fresher_core::{ByteSize, DiskState, ItemRule, ItemVisit, NodeKind};
    use proptest::prelude::*;

    use super::{Built, Everything, MIB, Named, answered, claim, root_of, state};
    use crate::tree::node::Flag;

    #[test]
    fn items_come_largest_first() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "a.bin", 2 * MIB);
        built.file(root, "b.bin", 5 * MIB);
        built.file(root, "c.bin", 3 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 10), ["b.bin", "c.bin", "a.bin"]);
    }

    #[test]
    fn ties_come_in_path_order() {
        let mut built = Built::new();
        let root = built.root;
        let (z, a) = (built.folder(root, "z"), built.folder(root, "a"));
        built.file(z, "first.bin", 2 * MIB);
        built.file(a, "second.bin", 2 * MIB);
        built.file(root, "m.bin", 2 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 10), ["second.bin", "m.bin", "first.bin"]);
    }

    #[test]
    fn a_folder_item_counts_its_whole_size_and_is_not_entered() {
        let mut built = Built::new();
        let root = built.root;
        let bundle = built.folder(root, "Photos.photoslibrary");
        built.file(bundle, "one.jpg", 2 * MIB);
        built.file(bundle, "two.jpg", 3 * MIB);
        let state = state(built);
        let rule = Named::new(&[("Photos.photoslibrary", ItemVisit::Item)]);

        let found = state.items(root_of(&state), &rule, ByteSize::new(0), 10).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(state.node(found[0]).unwrap().sizes.allocated, ByteSize::new(5 * MIB));
        assert_eq!(rule.asked_names(), ["Photos.photoslibrary"]);
    }

    #[test]
    fn an_entered_folder_is_not_itself_an_item() {
        let mut built = Built::new();
        let root = built.root;
        let downloads = built.folder(root, "Downloads");
        built.file(downloads, "disk.dmg", 4 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 10), ["disk.dmg"]);
    }

    #[test]
    fn a_file_the_rule_would_enter_is_an_item() {
        let mut built = Built::new();
        let root = built.root;
        built.file(root, "movie.mov", 4 * MIB);
        let state = state(built);
        let rule = Named::new(&[("movie.mov", ItemVisit::Enter)]);

        assert_eq!(answered(&state, &rule, 0, 10), ["movie.mov"]);
    }

    #[test]
    fn the_depth_counts_from_the_root() {
        let mut built = Built::new();
        let root = built.root;
        let outer = built.folder(root, "outer");
        let inner = built.folder(outer, "inner");
        built.file(inner, "deep.bin", 4 * MIB);
        let state = state(built);
        let (from_root, from_outer) = (Named::new(&[]), Named::new(&[]));

        state.items(root_of(&state), &from_root, ByteSize::new(0), 10).unwrap();
        state.items(child(&state, "outer"), &from_outer, ByteSize::new(0), 10).unwrap();

        let depths = |rule: &Named| -> Vec<(String, u32)> {
            rule.asked.borrow().iter().map(|(name, _, depth)| (name.clone(), *depth)).collect()
        };
        assert_eq!(depths(&from_root), [("outer".to_owned(), 1), ("inner".to_owned(), 2), ("deep.bin".to_owned(), 3)]);
        assert_eq!(depths(&from_outer), [("inner".to_owned(), 1), ("deep.bin".to_owned(), 2)]);
    }

    #[test]
    fn the_limit_keeps_the_largest() {
        let mut built = Built::new();
        let root = built.root;
        for (name, size) in [("a.bin", 1), ("b.bin", 4), ("c.bin", 2), ("d.bin", 3)] {
            built.file(root, name, size * MIB);
        }
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 2), ["b.bin", "d.bin"]);
    }

    #[test]
    fn the_limit_reaches_past_a_page_of_findings() {
        let mut built = Built::new();
        let root = built.root;
        for position in 0..600 {
            built.file(root, &format!("{position:03}.bin"), MIB);
        }
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, fresher_core::MAX_ITEMS + 1).len(), 600);
    }

    #[test]
    fn the_cap_keeps_one_more_than_the_items_a_caller_keeps() {
        let mut built = Built::new();
        let root = built.root;
        let cap = fresher_core::MAX_ITEMS + 1;
        for position in 0..=cap {
            built.file(root, &format!("{position:05}.bin"), MIB);
        }
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, u32::MAX).len(), cap as usize);
    }

    #[test]
    fn a_collapsed_subtree_is_offered_to_the_rule_as_collapsed() {
        let mut built = Built::new();
        let root = built.root;
        let folded = built.folder(root, "folded");
        built.tree.add_own(folded, super::sums(6 * MIB)).unwrap();
        built.tree.mark(folded, Flag::Collapsed).unwrap();
        let state = state(built);
        let rule = Named::new(&[("folded", ItemVisit::Item)]);

        assert_eq!(answered(&state, &rule, 0, 10), ["folded"]);
        assert_eq!(rule.asked.borrow()[0].1, NodeKind::Collapsed);
    }

    #[test]
    fn a_collapsed_subtree_entered_is_one_item() {
        let mut built = Built::new();
        let root = built.root;
        let folded = built.folder(root, "folded");
        built.tree.add_own(folded, super::sums(6 * MIB)).unwrap();
        built.tree.mark(folded, Flag::Collapsed).unwrap();
        let state = state(built);

        assert_eq!(answered(&state, &Everything, 0, 10), ["folded"]);
    }

    /// The child of the root named `name`.
    fn child(state: &super::TreeState, name: &str) -> fresher_core::NodeId {
        let page = state.children(root_of(state), fresher_core::ChildOrder::ByName, 0, 10).unwrap();
        page.nodes.into_iter().find(|node| node.name == name).unwrap().id
    }

    /// One random entry: its parent among the folders made before it, whether it is a folder, its size in MiB,
    /// whether it is claimed and how the rule answers it.
    fn raw() -> impl Strategy<Value = (usize, bool, u64, bool, u8)> {
        (0..32_usize, any::<bool>(), 0..8_u64, prop::bool::weighted(0.15), 0..3_u8)
    }

    /// Answers by the last byte of the name: the rule of a random tree.
    struct Random;

    impl ItemRule for Random {
        fn visit(&self, name: &str, _: NodeKind, _: u32) -> ItemVisit {
            match name.as_bytes().last().map_or(0, |byte| byte % 3) {
                0 => ItemVisit::Skip,
                1 => ItemVisit::Item,
                _ => ItemVisit::Enter,
            }
        }
    }

    proptest! {
        #[test]
        fn the_answer_is_sorted_unclaimed_large_enough_and_never_nested(
            entries in prop::collection::vec(raw(), 0..40),
            min in 0..6_u64,
            limit in 0..20_u32,
        ) {
            let mut built = Built::new();
            let mut folders = vec![built.root];
            for (position, (parent, folder, size, claimed, answer)) in entries.into_iter().enumerate() {
                let parent = folders[parent % folders.len()];
                let name = format!("n{position}-{answer}");
                let node = if folder { built.folder(parent, &name) } else { built.file(parent, &name, size * MIB) };
                if folder {
                    folders.push(node);
                }
                if claimed {
                    claim(&mut built, node);
                }
            }
            let state = state(built);
            let root = root_of(&state);
            let root_path = state.path(root).unwrap();

            let found = state.items(root, &Random, ByteSize::new(min * MIB), limit).unwrap();

            prop_assert!(found.len() <= limit as usize);
            let infos: Vec<_> = found.iter().map(|&id| state.node(id).unwrap()).collect();
            let paths: Vec<_> = found.iter().map(|&id| state.path(id).unwrap()).collect();
            for (info, path) in infos.iter().zip(&paths) {
                prop_assert!(path.starts_with(&root_path) && *path != root_path);
                prop_assert_eq!(info.owner, None);
                prop_assert!(info.sizes.allocated >= ByteSize::new(min * MIB));
            }
            for pair in infos.windows(2) {
                prop_assert!(pair[0].sizes.allocated >= pair[1].sizes.allocated);
            }
            for (left, outer) in paths.iter().enumerate() {
                for (right, inner) in paths.iter().enumerate() {
                    prop_assert!(left == right || !inner.starts_with(outer));
                }
            }
        }
    }
}
