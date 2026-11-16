use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    CancelToken, CategoryId, ClaimedPath, DiskState, DiskWalk, InterestSet, ModuleId, WalkMode, WalkRequest,
};
use fresher_testing::{FakeClock, FixtureTree};

use crate::matcher::testing::set_of;
use crate::walk::std_reader::StdReader;
use crate::walk::tests::{canonical, scanner};

/// A clock at the fixtures' usual start.
const TAKEN: u64 = 1_735_689_600;

/// Scans `tree` (avoiding `avoid`) and commits `claims`.
pub(super) fn committed(
    tree: &FixtureTree,
    avoid: &[&str],
    claims: &[(&str, &'static str)],
) -> (PathBuf, Arc<dyn DiskState>) {
    let root = canonical(tree.root());
    let scanner = scanner(Arc::new(StdReader::new()), &root, 2).with_clock(Arc::new(FakeClock::at(TAKEN)));
    let (roots, interests): ([PathBuf; 1], InterestSet) = ([root.clone()], set_of(&[]));
    let avoid: Vec<PathBuf> = avoid.iter().map(|place| root.join(place)).collect();
    let request = WalkRequest { roots: &roots, interests: &interests, avoid: &avoid, mode: WalkMode::Full };
    scanner.walk(&request, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();
    let claims: Vec<ClaimedPath> = claims
        .iter()
        .map(|&(path, category)| ClaimedPath { path: root.join(path), category: category_id(category) })
        .collect();
    let state = scanner.commit(&claims).unwrap();
    (root, state)
}

fn category_id(id: &'static str) -> CategoryId {
    CategoryId::new(ModuleId::new("demo").unwrap(), id).unwrap()
}

/// The node at `relative` under the first root, by walking the children by name.
fn find(state: &dyn DiskState, relative: &Path) -> fresher_core::NodeInfo {
    let mut node = state.node(state.roots()[0]).unwrap();
    for name in relative {
        let page = state.children(node.id, fresher_core::ChildOrder::ByName, 0, u32::MAX).unwrap();
        node = page.nodes.into_iter().find(|child| child.name == name.to_string_lossy()).unwrap();
    }
    node
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{ChildOrder, ClaimedPath, DiskWalk, NodeId, StateId, WalkError};
    use fresher_testing::FixtureTree;

    use super::{category_id, committed};
    use crate::walk::std_reader::StdReader;
    use crate::walk::tests::scanner;

    #[test]
    fn a_node_of_another_state_is_refused() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a/f\"\n").unwrap();
        let (_, state) = committed(&tree, &[], &[]);
        let root = state.roots()[0];
        let stranger = NodeId::new(StateId::new(state.id().get() + 1), root.index());
        let past_the_end = NodeId::new(state.id(), 1_000_000);

        assert_eq!(state.node(stranger), None);
        assert_eq!(state.children(stranger, ChildOrder::BySize, 0, 10), None);
        assert_eq!(state.path(stranger), None);
        assert_eq!(state.node(past_the_end), None);
        assert_eq!((state.largest_files(10), state.search("no-such-name", 10)), (vec![], vec![]));
    }

    #[test]
    fn a_commit_without_a_walk_is_refused() {
        let scanner = scanner(Arc::new(StdReader::new()), std::path::Path::new("/nowhere"), 1);
        let claim = ClaimedPath { path: "/nowhere".into(), category: category_id("demo.caches") };

        let error = scanner.commit(&[claim]).err();

        assert_eq!(error, Some(WalkError::Commit { kind: std::io::ErrorKind::NotFound }));
    }

    #[test]
    fn a_claim_on_a_path_that_is_no_node_is_left_out() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a/f\"\n").unwrap();

        let (root, state) = committed(&tree, &[], &[("a/f", "demo.caches"), ("missing", "demo.caches")]);

        let a = super::find(&*state, std::path::Path::new("a"));
        assert_eq!(a.owner, None);
        assert_eq!(state.path(a.id), Some(root.join("a")));
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{ChildOrder, NodeKind, NotScanned, Timestamp};
    use fresher_testing::FixtureTree;

    use super::{TAKEN, category_id, committed, find};

    #[test]
    fn the_committed_state_answers_children_by_size() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"b/small\"\nsize = 10\n[[file]]\npath = \"a/big\"\nsize = 9000000\n[[dir]]\npath = \"c\"\n",
        )
        .unwrap();
        std::fs::write(tree.root().join("b/small"), vec![1_u8; 10_000]).unwrap();

        let (root, state) = committed(&tree, &[], &[]);

        let top = state.node(state.roots()[0]).unwrap();
        assert_eq!((top.name, top.kind, top.folders), (root.to_string_lossy().into_owned(), NodeKind::Folder, 3));
        assert_eq!(state.taken(), Timestamp::from_secs(TAKEN));
        let by_size = state.children(top.id, ChildOrder::BySize, 0, 10).unwrap();
        let names: Vec<_> = by_size.nodes.iter().map(|node| node.name.as_str()).collect();
        assert_eq!(names, ["b", "a", "c"], "b holds real blocks; a and c none (sparse, empty)");
        let by_name = state.children(top.id, ChildOrder::ByName, 1, 1).unwrap();
        assert_eq!((by_name.total, by_name.offset, by_name.nodes[0].name.as_str()), (3, 1, "b"));
        assert_eq!(find(&*state, Path::new("b")).files, 1);
    }

    #[test]
    fn a_claimed_node_knows_its_category() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Library/Caches/app/x\"\n[[dir]]\npath = \"Logs\"\n")
            .unwrap();

        let (_, state) = committed(&tree, &[], &[("Library/Caches/app", "demo.caches"), ("Logs", "demo.logs")]);

        assert_eq!(find(&*state, Path::new("Library/Caches/app")).owner, Some(category_id("demo.caches")));
        assert_eq!(find(&*state, Path::new("Logs")).owner, Some(category_id("demo.logs")));
        assert_eq!(find(&*state, Path::new("Library")).owner, None);
    }

    #[test]
    fn the_state_lists_the_places_not_scanned() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Documents/a/b/f\"\n[[file]]\npath = \"Library/g\"\n")
            .unwrap();

        let (_, state) = committed(&tree, &["Documents"], &[]);

        let places = state.unreadable(10);
        assert_eq!(places.len(), 1);
        assert_eq!((places[0].node.name.as_str(), places[0].reason), ("Documents", NotScanned::Avoided));
        assert_eq!(state.unreadable(0), []);
    }

    #[test]
    fn each_commit_of_a_scanner_is_a_new_state() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"big\"\nsize = 5000000\n").unwrap();
        std::fs::write(tree.root().join("big"), vec![1_u8; 2 << 20]).unwrap();
        let root = super::canonical(tree.root());
        let scanner = crate::walk::tests::scanner(std::sync::Arc::new(crate::StdReader::new()), &root, 1);
        let (roots, interests) = ([root.clone()], crate::matcher::testing::set_of(&[]));
        let request = fresher_core::WalkRequest {
            roots: &roots,
            interests: &interests,
            avoid: &[],
            mode: fresher_core::WalkMode::Full,
        };
        let mut ids = Vec::new();

        for _ in 0..2 {
            fresher_core::DiskWalk::walk(
                &scanner,
                &request,
                &fresher_core::CancelToken::new(),
                &mut |_| {},
                &mut |_| {},
            )
            .unwrap();
            ids.push(fresher_core::DiskWalk::commit(&scanner, &[]).unwrap());
        }

        assert_ne!(ids[0].id(), ids[1].id());
        assert_eq!(find(&*ids[1], Path::new("big")).kind, NodeKind::File, "a file of 1 MiB and up is kept");
    }
}
