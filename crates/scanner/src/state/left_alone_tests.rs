//! The left-alone query over trees built in memory.

use std::cell::RefCell;

use fresher_core::{ByteSize, CategoryId, DiskState, ItemRule, ItemVisit, ModuleId, NodeKind, StateId, Timestamp};

use crate::state::TreeState;
use crate::state::query_tests::{Built, MIB, sums};
use crate::tree::node::NodeIndex;

/// "Before" in every test: whatever is older than this is left alone.
const BEFORE: u64 = 100;

/// A rule that answers by name (anything not named is entered) and records the names it was asked about.
struct Named {
    answers: Vec<(&'static str, ItemVisit)>,
    asked: RefCell<Vec<String>>,
}

impl Named {
    fn new(answers: &[(&'static str, ItemVisit)]) -> Self {
        Self { answers: answers.to_vec(), asked: RefCell::new(Vec::new()) }
    }
}

impl ItemRule for Named {
    fn visit(&self, name: &str, _: NodeKind, _: u32) -> ItemVisit {
        self.asked.borrow_mut().push(name.to_owned());
        self.answers.iter().find(|(named, _)| *named == name).map_or(ItemVisit::Enter, |(_, visit)| *visit)
    }
}

/// A kept file of `allocated` bytes whose newest change is `newest` seconds.
fn file(built: &mut Built, parent: NodeIndex, name: &str, allocated: u64, newest: u64) -> NodeIndex {
    let file = built.file(parent, name, allocated);
    built.tree.join_newest(file, i64::try_from(newest).unwrap()).unwrap();
    file
}

/// A folder holding one file `data` of `allocated` bytes changed at `newest`.
fn folder(built: &mut Built, parent: NodeIndex, name: &str, allocated: u64, newest: u64) -> NodeIndex {
    let folder = built.folder(parent, name);
    file(built, folder, "data", allocated, newest);
    folder
}

fn state(mut built: Built) -> TreeState {
    built.tree.freeze();
    let owner = CategoryId::new(ModuleId::new("demo").unwrap(), "demo.claimed").unwrap();
    TreeState::new(StateId::new(1), Timestamp::from_secs(0), built.tree, vec![owner])
}

/// The names and newest seconds `left_alone` answers from the root, at least `min` MiB, before [`BEFORE`].
fn answered(state: &TreeState, rule: &dyn ItemRule, min: u64) -> Vec<(String, u64)> {
    let found =
        state.left_alone(state.roots()[0], rule, ByteSize::new(min * MIB), Timestamp::from_secs(BEFORE)).unwrap();
    found.into_iter().map(|one| (state.node(one.folder).unwrap().name, one.newest.secs())).collect()
}

mod negative {
    use fresher_core::{ByteSize, DiskState, EntryKind, ItemVisit, NodeId, StateId, Timestamp};

    use super::{BEFORE, Built, MIB, Named, answered, file, folder, state, sums};
    use crate::tree::node::Flag;

    #[test]
    fn the_root_itself_is_never_answered() {
        let mut built = Built::new();
        let root = built.root;
        file(&mut built, root, "old.bin", 5 * MIB, 10);
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), Vec::<(String, u64)>::new());
    }

    #[test]
    fn a_folder_below_the_minimum_is_neither_answered_nor_entered() {
        let mut built = Built::new();
        let root = built.root;
        let small = folder(&mut built, root, "small", MIB, 10);
        built.folder(small, "inner");
        folder(&mut built, root, "large", 4 * MIB, 10);
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 3), [("large".to_owned(), 10)]);
        assert_eq!(*rule.asked.borrow(), ["large"]);
    }

    #[test]
    fn a_folder_with_a_recent_file_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let project = folder(&mut built, root, "project", 4 * MIB, 10);
        file(&mut built, project, "touched", MIB, BEFORE);
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), Vec::<(String, u64)>::new());
    }

    #[test]
    fn a_partial_folder_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let unread = folder(&mut built, root, "unread", 4 * MIB, 10);
        built.tree.mark(unread, Flag::Partial).unwrap();
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), Vec::<(String, u64)>::new());
    }

    #[test]
    fn a_folder_without_a_newest_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let empty = built.folder(root, "bare");
        built.file(empty, "data", 4 * MIB);
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), Vec::<(String, u64)>::new());
    }

    #[test]
    fn a_claimed_folder_is_neither_answered_nor_entered() {
        let mut built = Built::new();
        let root = built.root;
        let caches = folder(&mut built, root, "Caches", 4 * MIB, 10);
        built.tree.set_owner(caches, 0).unwrap();
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 1), Vec::<(String, u64)>::new());
        assert_eq!(*rule.asked.borrow(), Vec::<String>::new());
    }

    #[test]
    fn a_file_a_link_or_a_collapsed_subtree_is_never_answered() {
        let mut built = Built::new();
        let root = built.root;
        file(&mut built, root, "old.bin", 5 * MIB, 10);
        let link = built.tree.add_child(root, b"shortcut", EntryKind::Symlink).unwrap();
        built.tree.add_own(link, sums(5 * MIB)).unwrap();
        let folded = folder(&mut built, root, "folded", 5 * MIB, 10);
        built.tree.mark(folded, Flag::Collapsed).unwrap();
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 1), Vec::<(String, u64)>::new());
        assert_eq!(*rule.asked.borrow(), Vec::<String>::new());
    }

    #[test]
    fn what_the_rule_skips_or_calls_an_item_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let skipped = built.folder(root, ".hidden");
        folder(&mut built, skipped, "inner", 4 * MIB, 10);
        folder(&mut built, root, "App.app", 4 * MIB, 10);
        let state = state(built);
        let rule = Named::new(&[(".hidden", ItemVisit::Skip), ("App.app", ItemVisit::Item)]);

        assert_eq!(answered(&state, &rule, 1), Vec::<(String, u64)>::new());
        assert_eq!(*rule.asked.borrow(), [".hidden", "App.app"]);
    }

    #[test]
    fn a_folder_inside_an_answered_one_is_not_answered() {
        let mut built = Built::new();
        let root = built.root;
        let outer = folder(&mut built, root, "outer", 4 * MIB, 10);
        folder(&mut built, outer, "inner", 3 * MIB, 10);
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), [("outer".to_owned(), 10)]);
    }

    #[test]
    fn a_node_of_another_state_answers_none() {
        let state = state(Built::new());
        let before = Timestamp::from_secs(BEFORE);

        assert_eq!(state.left_alone(NodeId::new(StateId::new(2), 0), &Named::new(&[]), ByteSize::new(0), before), None);
    }
}

mod positive {
    use super::{BEFORE, Built, MIB, Named, answered, file, folder, state};

    #[test]
    fn a_stale_folder_is_answered_with_its_newest() {
        let mut built = Built::new();
        let root = built.root;
        let project = folder(&mut built, root, "project", 4 * MIB, 10);
        file(&mut built, project, "later", MIB, 40);
        let state = state(built);

        assert_eq!(answered(&state, &Named::new(&[]), 1), [("project".to_owned(), 40)]);
    }

    #[test]
    fn the_stale_children_of_a_recent_folder_are_answered() {
        let mut built = Built::new();
        let root = built.root;
        let work = built.folder(root, "work");
        folder(&mut built, work, "old", 4 * MIB, 10);
        folder(&mut built, work, "fresh", 4 * MIB, BEFORE + 5);
        let state = state(built);
        let rule = Named::new(&[]);

        assert_eq!(answered(&state, &rule, 1), [("old".to_owned(), 10)]);
        assert_eq!(*rule.asked.borrow(), ["work", "fresh", "old"]);
    }

    #[test]
    fn answers_come_largest_first_ties_in_path_order() {
        let mut built = Built::new();
        let root = built.root;
        folder(&mut built, root, "b", 4 * MIB, 10);
        folder(&mut built, root, "a", 4 * MIB, 20);
        folder(&mut built, root, "big", 9 * MIB, 30);
        folder(&mut built, root, "c", 2 * MIB, 40);
        let state = state(built);

        let names: Vec<_> = answered(&state, &Named::new(&[]), 1).into_iter().map(|(name, _)| name).collect();

        assert_eq!(names, ["big", "a", "b", "c"]);
    }
}
