use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use fresher_core::{
    ByteSize, CategoryId, ChildOrder, DiskState, NodeId, NodeInfo, NodeKind, NodePage, Sizes, StateId, Timestamp,
    UnreadablePlace,
};
use fresher_testing::FixedDiskState;

use crate::Engine;
use crate::testing::{ScriptedWalk, engine, scan};

/// `/home` with two folders, a large file and a locked folder holding one more, as state `id`.
fn home(id: u32) -> FixedDiskState {
    let nodes = [
        ("/home", NodeKind::Folder, 900),
        ("/home/Library", NodeKind::Folder, 300),
        ("/home/Library/Caches", NodeKind::Folder, 300),
        ("/home/movie.mov", NodeKind::File, 500),
        ("/home/locked", NodeKind::Folder, 100),
        ("/home/locked/inner", NodeKind::Folder, 100),
    ];
    let nodes = nodes.map(|(path, kind, size)| {
        (PathBuf::from(path), kind, Sizes::complete(ByteSize::new(size), ByteSize::new(size)))
    });
    FixedDiskState::new(StateId::new(id), Timestamp::from_secs(50), nodes)
}

/// An engine that has scanned once and holds `state`; the next scans commit `next`.
fn scanned(state: Arc<dyn DiskState>, next: Vec<Arc<dyn DiskState>>) -> Engine {
    let mut states = vec![state];
    states.extend(next);
    let engine = engine(ScriptedWalk::new().committing(states));
    scan(&engine);
    engine
}

fn names(nodes: &[NodeInfo]) -> Vec<&str> {
    nodes.iter().map(|node| node.name.as_str()).collect()
}

/// A state whose `search` waits, once entered, until it is released: a long query.
#[derive(Debug)]
struct SlowState {
    inner: FixedDiskState,
    entered: AtomicBool,
    released: AtomicBool,
}

impl SlowState {
    fn wait_until_entered(&self) {
        while !self.entered.load(Ordering::Acquire) {
            thread::yield_now();
        }
    }
}

impl DiskState for SlowState {
    fn id(&self) -> StateId {
        self.inner.id()
    }

    fn taken(&self) -> Timestamp {
        self.inner.taken()
    }

    fn roots(&self) -> Vec<NodeId> {
        self.inner.roots()
    }

    fn node(&self, id: NodeId) -> Option<NodeInfo> {
        self.inner.node(id)
    }

    fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage> {
        self.inner.children(id, order, offset, limit)
    }

    fn largest_files(&self, limit: u32) -> Vec<NodeInfo> {
        self.inner.largest_files(limit)
    }

    fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo> {
        self.entered.store(true, Ordering::Release);
        while !self.released.load(Ordering::Acquire) {
            thread::yield_now();
        }
        self.inner.search(query, limit)
    }

    fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace> {
        self.inner.unreadable(limit)
    }

    fn path(&self, id: NodeId) -> Option<PathBuf> {
        self.inner.path(id)
    }

    fn owned(&self) -> Vec<(NodeId, CategoryId)> {
        self.inner.owned()
    }

    fn find(&self, path: &Path) -> Option<NodeId> {
        self.inner.find(path)
    }
}

/// Releases a [`SlowState`] when dropped, so a failing test ends instead of hanging on its reader.
struct Release<'a>(&'a AtomicBool);

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

mod negative {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    use fresher_core::{ChildOrder, CommandError, Event, NodeId, StateId};

    use super::{Release, SlowState, home, names, scanned};
    use crate::testing::{ScriptedWalk, engine, wait_for_finish};

    #[test]
    fn queries_before_any_scan_answer_no_scan() {
        let engine = engine(ScriptedWalk::new());
        let node = NodeId::new(StateId::new(1), 0);

        assert_eq!(engine.disk_roots(), Err(CommandError::NoScan));
        assert_eq!(engine.disk_node(node), Err(CommandError::NoScan));
        assert_eq!(engine.disk_children(node, ChildOrder::BySize, 0, 10), Err(CommandError::NoScan));
        assert_eq!(engine.largest_files(10), Err(CommandError::NoScan));
        assert_eq!(engine.search_names("cache", 10), Err(CommandError::NoScan));
        assert_eq!(engine.unreadable_places(10), Err(CommandError::NoScan));
    }

    #[test]
    fn a_node_of_an_older_state_is_stale() {
        let engine = scanned(Arc::new(home(1)), vec![Arc::new(home(2))]);
        let old = engine.disk_roots().unwrap()[0].id;
        super::scan(&engine);

        assert_eq!(engine.disk_node(old), Err(CommandError::StaleNode));
        assert_eq!(engine.disk_children(old, ChildOrder::ByName, 0, 10), Err(CommandError::StaleNode));
        assert_eq!(engine.disk_node(NodeId::new(StateId::new(2), 99)), Err(CommandError::StaleNode));
    }

    #[test]
    fn a_query_does_not_block_a_new_scan() {
        let slow =
            Arc::new(SlowState { inner: home(1), entered: AtomicBool::new(false), released: AtomicBool::new(false) });
        let engine = scanned(Arc::clone(&slow) as _, vec![Arc::new(home(2))]);

        thread::scope(|scope| {
            let _release_on_failure = Release(&slow.released);
            let query = scope.spawn(|| engine.search_names("cache", 10));
            slow.wait_until_entered();

            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();
            let finished = wait_for_finish(&events);

            assert!(matches!(finished.last(), Some(Event::ScanFinished { .. })));
            assert_eq!(engine.disk_roots().unwrap()[0].id.state(), StateId::new(2), "committed during the query");
            slow.released.store(true, Ordering::Release);
            let found = query.join().unwrap().unwrap();
            assert_eq!(names(&found), ["Caches"], "the old state answers its reader to the end");
        });
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{ByteSize, ChildOrder, NodeKind, NotScanned, Sizes, StateId, Timestamp, Unreadable};
    use fresher_testing::FixedDiskState;

    use super::{home, names, scanned};

    #[test]
    fn children_page_by_size() {
        let engine = scanned(Arc::new(home(1)), vec![]);
        let root = engine.disk_roots().unwrap()[0].id;

        let page = engine.disk_children(root, ChildOrder::BySize, 0, 2).unwrap();
        let next = engine.disk_children(root, ChildOrder::BySize, 2, 2).unwrap();

        assert_eq!((names(&page.nodes), page.total), (vec!["movie.mov", "Library"], 3));
        assert_eq!((names(&next.nodes), next.offset), (vec!["locked"], 2));
        assert_eq!(engine.disk_node(root).unwrap().name, "home");
    }

    #[test]
    fn a_page_holds_at_most_five_hundred_nodes() {
        let size = Sizes::complete(ByteSize::new(1), ByteSize::new(1));
        let mut nodes = vec![(PathBuf::from("/many"), NodeKind::Folder, size)];
        nodes.extend((0..501).map(|index| (PathBuf::from(format!("/many/f{index}")), NodeKind::File, size)));
        let engine = scanned(Arc::new(FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)), vec![]);
        let root = engine.disk_roots().unwrap()[0].id;

        let page = engine.disk_children(root, ChildOrder::ByName, 0, u32::MAX).unwrap();

        assert_eq!((page.nodes.len(), page.total), (500, 501));
        assert_eq!(engine.largest_files(u32::MAX).unwrap().len(), 500);
    }

    #[test]
    fn search_names_answers_from_the_last_state() {
        let engine = scanned(Arc::new(home(1)), vec![]);

        assert_eq!(names(&engine.search_names("CACHE", 10).unwrap()), ["Caches"]);
        assert_eq!(names(&engine.largest_files(10).unwrap()), ["movie.mov"]);
    }

    #[test]
    fn unreadable_places_lists_the_top_folders_with_their_reason() {
        let reason = NotScanned::Unreadable(Unreadable::NoPermission);
        let state = home(1)
            .with_not_scanned(std::path::Path::new("/home/locked"), reason)
            .with_not_scanned(std::path::Path::new("/home/locked/inner"), reason);
        let engine = scanned(Arc::new(state), vec![]);

        let places = engine.unreadable_places(10).unwrap();

        assert_eq!(places.len(), 1);
        assert_eq!((places[0].node.name.as_str(), places[0].reason, places[0].hidden_folders), ("locked", reason, 1));
    }

    #[test]
    fn a_new_scan_replaces_the_state() {
        let engine = scanned(Arc::new(home(1)), vec![Arc::new(home(2))]);
        assert_eq!(engine.disk_roots().unwrap()[0].id.state(), StateId::new(1));

        super::scan(&engine);

        let root = engine.disk_roots().unwrap()[0].id;
        assert_eq!(root.state(), StateId::new(2));
        assert_eq!(engine.disk_node(root).unwrap().name, "home");
    }
}
