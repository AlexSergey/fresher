use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fresher_core::{
    ByteSize, CategoryId, ChildOrder, Completeness, DiskState, ItemRule, ItemVisit, LeftAlone, NodeId, NodeInfo,
    NodeKind, NodePage, NotScanned, Sizes, StateId, Timestamp, UnreadablePlace,
};

/// A [`DiskState`] built from a list of paths with their kinds and sizes, for the engine's tests and the reference
/// walk. A node's parent is the node of its parent path; a node without one is a root. Sizes are taken as given,
/// never summed.
#[derive(Clone, Debug)]
pub struct FixedDiskState {
    id: StateId,
    taken: Timestamp,
    nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
struct Node {
    path: PathBuf,
    kind: NodeKind,
    sizes: Sizes,
    owner: Option<CategoryId>,
    not_scanned: Option<NotScanned>,
    modified: Option<Timestamp>,
    newest: Option<Timestamp>,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl FixedDiskState {
    /// The state `id`, taken at `taken`, with `nodes` in the order given (the roots keep that order).
    pub fn new(id: StateId, taken: Timestamp, nodes: impl IntoIterator<Item = (PathBuf, NodeKind, Sizes)>) -> Self {
        let mut list: Vec<Node> = nodes
            .into_iter()
            .map(|(path, kind, sizes)| Node {
                path,
                kind,
                sizes,
                owner: None,
                not_scanned: None,
                modified: None,
                newest: None,
                parent: None,
                children: Vec::new(),
            })
            .collect();
        let index: HashMap<PathBuf, usize> =
            list.iter().enumerate().map(|(position, node)| (node.path.clone(), position)).collect();
        for child in 0..list.len() {
            let parent = list[child].path.parent().and_then(|parent| index.get(parent).copied());
            if let Some(parent) = parent {
                list[child].parent = Some(parent);
                list[parent].children.push(child);
            }
        }
        Self { id, taken, nodes: list }
    }

    /// Records `owner` as the category that claimed `path`; a path not in the state is ignored.
    #[must_use]
    pub fn with_owner(mut self, path: &Path, owner: CategoryId) -> Self {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.path == path) {
            node.owner = Some(owner);
        }
        self
    }

    /// Records why `path` was not scanned; a path not in the state is ignored.
    #[must_use]
    pub fn with_not_scanned(mut self, path: &Path, reason: NotScanned) -> Self {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.path == path) {
            node.not_scanned = Some(reason);
        }
        self
    }

    /// Records when `path` was last changed; a path not in the state is ignored.
    #[must_use]
    pub fn with_modified(mut self, path: &Path, modified: Timestamp) -> Self {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.path == path) {
            node.modified = Some(modified);
        }
        self
    }

    /// Records the newest change of anything inside `path`; a path not in the state is ignored.
    #[must_use]
    pub fn with_newest(mut self, path: &Path, newest: Timestamp) -> Self {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.path == path) {
            node.newest = Some(newest);
        }
        self
    }

    /// The position of `id` in this state; `None` for an id of another state or no node.
    fn position(&self, id: NodeId) -> Option<usize> {
        let position = usize::try_from(id.index()).ok()?;
        (id.state() == self.id && position < self.nodes.len()).then_some(position)
    }

    fn id_of(&self, position: usize) -> NodeId {
        NodeId::new(self.id, u32::try_from(position).unwrap_or(u32::MAX))
    }

    /// Every node under `position`, at every depth.
    fn descendants(&self, position: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut stack = self.nodes[position].children.clone();
        while let Some(next) = stack.pop() {
            found.push(next);
            stack.extend_from_slice(&self.nodes[next].children);
        }
        found
    }

    fn count(&self, position: usize, kind: NodeKind) -> u64 {
        self.descendants(position).iter().filter(|&&inner| self.nodes[inner].kind == kind).count() as u64
    }

    fn name(&self, position: usize) -> String {
        let path = &self.nodes[position].path;
        path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned()
    }

    fn info(&self, position: usize) -> NodeInfo {
        let node = &self.nodes[position];
        NodeInfo {
            id: self.id_of(position),
            name: self.name(position),
            kind: node.kind,
            sizes: node.sizes,
            files: self.count(position, NodeKind::File),
            folders: self.count(position, NodeKind::Folder),
            owner: node.owner,
            modified: node.modified,
            not_scanned: node.not_scanned,
        }
    }

    /// `positions` as nodes, the largest first (then by path), at most `limit` of them.
    fn largest(&self, mut positions: Vec<usize>, limit: u32) -> Vec<NodeInfo> {
        positions.sort_by(|&a, &b| {
            let (a, b) = (&self.nodes[a], &self.nodes[b]);
            b.sizes.allocated.cmp(&a.sizes.allocated).then_with(|| a.path.cmp(&b.path))
        });
        positions.into_iter().take(limit as usize).map(|position| self.info(position)).collect()
    }

    /// Adds to `found` the items below `position`, children by name, as [`DiskState::items`] says.
    fn collect_items(&self, position: usize, rule: &dyn ItemRule, min: ByteSize, depth: u32, found: &mut Vec<usize>) {
        let mut children = self.nodes[position].children.clone();
        children.sort_by_key(|&child| self.name(child));
        for child in children {
            let node = &self.nodes[child];
            if node.owner.is_some() || node.kind == NodeKind::Link || node.sizes.allocated < min {
                continue;
            }
            match rule.visit(&self.name(child), node.kind, depth) {
                ItemVisit::Skip => {}
                ItemVisit::Enter if node.kind == NodeKind::Folder => {
                    self.collect_items(child, rule, min, depth + 1, found);
                }
                ItemVisit::Item | ItemVisit::Enter => found.push(child),
            }
        }
    }

    /// Adds to `found` the folders below `position` left alone before `before`, children by name, as
    /// [`DiskState::left_alone`] says.
    fn collect_left_alone(
        &self,
        position: usize,
        rule: &dyn ItemRule,
        min: ByteSize,
        before: Timestamp,
        depth: u32,
        found: &mut Vec<(usize, Timestamp)>,
    ) {
        let mut children = self.nodes[position].children.clone();
        children.sort_by_key(|&child| self.name(child));
        for child in children {
            let node = &self.nodes[child];
            if node.owner.is_some() || node.kind != NodeKind::Folder || node.sizes.allocated < min {
                continue;
            }
            if rule.visit(&self.name(child), node.kind, depth) != ItemVisit::Enter {
                continue;
            }
            match node.newest {
                Some(newest) if node.sizes.completeness == Completeness::Complete && newest < before => {
                    found.push((child, newest));
                }
                _ => self.collect_left_alone(child, rule, min, before, depth + 1, found),
            }
        }
    }

    fn is_hidden(&self, position: usize) -> bool {
        let mut parent = self.nodes[position].parent;
        while let Some(above) = parent {
            if self.nodes[above].not_scanned.is_some() {
                return true;
            }
            parent = self.nodes[above].parent;
        }
        false
    }
}

impl DiskState for FixedDiskState {
    fn id(&self) -> StateId {
        self.id
    }

    fn taken(&self) -> Timestamp {
        self.taken
    }

    fn roots(&self) -> Vec<NodeId> {
        (0..self.nodes.len()).filter(|&position| self.nodes[position].parent.is_none()).map(|p| self.id_of(p)).collect()
    }

    fn node(&self, id: NodeId) -> Option<NodeInfo> {
        self.position(id).map(|position| self.info(position))
    }

    fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage> {
        let mut children = self.nodes[self.position(id)?].children.clone();
        children.sort_by(|&a, &b| {
            let by_name = self.name(a).cmp(&self.name(b));
            match order {
                ChildOrder::ByName => by_name,
                ChildOrder::BySize => self.nodes[b].sizes.allocated.cmp(&self.nodes[a].sizes.allocated).then(by_name),
            }
        });
        let total = u32::try_from(children.len()).unwrap_or(u32::MAX);
        let nodes = children
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|position| self.info(position))
            .collect();
        Some(NodePage { nodes, offset, total })
    }

    fn largest_files(&self, limit: u32) -> Vec<NodeInfo> {
        let files = (0..self.nodes.len()).filter(|&position| self.nodes[position].kind == NodeKind::File).collect();
        self.largest(files, limit)
    }

    fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo> {
        let query = query.to_lowercase();
        let found = (0..self.nodes.len()).filter(|&position| self.name(position).to_lowercase().contains(&query));
        self.largest(found.collect(), limit)
    }

    fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace> {
        (0..self.nodes.len())
            .filter(|&position| !self.is_hidden(position))
            .filter_map(|position| {
                let reason = self.nodes[position].not_scanned?;
                let hidden_folders = self.count(position, NodeKind::Folder);
                Some(UnreadablePlace { node: self.info(position), reason, hidden_folders })
            })
            .take(limit as usize)
            .collect()
    }

    fn path(&self, id: NodeId) -> Option<PathBuf> {
        self.position(id).map(|position| self.nodes[position].path.clone())
    }

    /// Depth first from the roots, each folder's children in the order given.
    fn owned(&self) -> Vec<(NodeId, CategoryId)> {
        let mut owned = Vec::new();
        let mut stack: Vec<usize> = self.roots().iter().rev().filter_map(|&root| self.position(root)).collect();
        while let Some(position) = stack.pop() {
            let node = &self.nodes[position];
            if let Some(owner) = node.owner {
                owned.push((self.id_of(position), owner));
            }
            stack.extend(node.children.iter().rev());
        }
        owned
    }

    fn find(&self, path: &Path) -> Option<NodeId> {
        self.nodes.iter().position(|node| node.path == path).map(|position| self.id_of(position))
    }

    /// A recursive walk from `root`; a link is never offered; there are no special entries or hard links here.
    fn items(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, limit: u32) -> Option<Vec<NodeId>> {
        let root = self.position(root)?;
        let node = &self.nodes[root];
        let mut found = Vec::new();
        if node.owner.is_none() && node.kind != NodeKind::Link && node.sizes.allocated >= min {
            self.collect_items(root, rule, min, 1, &mut found);
        }
        let limit = limit.min(fresher_core::MAX_ITEMS.saturating_add(1));
        Some(self.largest(found, limit).into_iter().map(|info| info.id).collect())
    }

    /// A recursive walk from `root`, sizes and newest as recorded; a `Partial` folder is unknown.
    fn left_alone(
        &self,
        root: NodeId,
        rule: &dyn ItemRule,
        min: ByteSize,
        before: Timestamp,
    ) -> Option<Vec<LeftAlone>> {
        let root = self.position(root)?;
        let node = &self.nodes[root];
        let mut found = Vec::new();
        if node.owner.is_none() && node.kind != NodeKind::Link && node.sizes.allocated >= min {
            self.collect_left_alone(root, rule, min, before, 1, &mut found);
        }
        let mut found: Vec<(usize, Timestamp)> = found;
        found.sort_by(|&(a, _), &(b, _)| {
            let (a, b) = (&self.nodes[a], &self.nodes[b]);
            b.sizes.allocated.cmp(&a.sizes.allocated).then_with(|| a.path.cmp(&b.path))
        });
        Some(found.into_iter().map(|(position, newest)| LeftAlone { folder: self.id_of(position), newest }).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use fresher_core::{
        ByteSize, CategoryId, DiskState, ItemRule, ItemVisit, ModuleId, NodeKind, Sizes, StateId, Timestamp,
    };

    use crate::FixedDiskState;

    fn category(id: &'static str) -> CategoryId {
        CategoryId::new(ModuleId::new("demo").unwrap(), id).unwrap()
    }

    fn bytes(size: u64) -> Sizes {
        Sizes::complete(ByteSize::new(size), ByteSize::new(size))
    }

    /// `/home` with `Library/Caches` (two folders), a large file and a locked folder holding one more.
    fn state() -> FixedDiskState {
        let nodes = [
            ("/home", NodeKind::Folder, 900),
            ("/home/Library", NodeKind::Folder, 300),
            ("/home/Library/Caches", NodeKind::Folder, 300),
            ("/home/movie.mov", NodeKind::File, 500),
            ("/home/locked", NodeKind::Folder, 100),
            ("/home/locked/inner", NodeKind::Folder, 100),
            ("/home/Library/big.cache", NodeKind::File, 200),
        ];
        let nodes = nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, bytes(size)));
        FixedDiskState::new(StateId::new(1), Timestamp::from_secs(50), nodes)
    }

    /// A rule that skips, calls an item or enters by name (anything not named is entered) and records the names
    /// and depths it was asked about.
    struct Named {
        answers: &'static [(&'static str, ItemVisit)],
        asked: RefCell<Vec<(String, u32)>>,
    }

    impl Named {
        fn new(answers: &'static [(&'static str, ItemVisit)]) -> Self {
            Self { answers, asked: RefCell::new(Vec::new()) }
        }
    }

    impl ItemRule for Named {
        fn visit(&self, name: &str, _: NodeKind, depth: u32) -> ItemVisit {
            self.asked.borrow_mut().push((name.to_owned(), depth));
            self.answers.iter().find(|(named, _)| *named == name).map_or(ItemVisit::Enter, |(_, visit)| *visit)
        }
    }

    /// The names `items` answers from the first root.
    fn items(state: &FixedDiskState, rule: &Named, min: u64, limit: u32) -> Vec<String> {
        let found = state.items(state.roots()[0], rule, ByteSize::new(min), limit).unwrap();
        found.into_iter().map(|id| state.node(id).unwrap().name).collect()
    }

    mod negative {
        use std::path::Path;

        use crate::FixedDiskState;
        use fresher_core::{
            ByteSize, ChildOrder, Completeness, DiskState, ItemVisit, NodeId, NodeKind, NotScanned, Sizes, StateId,
            Timestamp, Unreadable,
        };
        use std::path::PathBuf;

        use super::{Named, items, state};

        #[test]
        fn items_of_a_node_of_another_state_answer_none() {
            let other = NodeId::new(StateId::new(2), 0);

            assert_eq!(state().items(other, &Named::new(&[]), ByteSize::new(0), 10), None);
        }

        #[test]
        fn items_never_offer_a_claimed_node_or_a_link() {
            let state = state()
                .with_owner(Path::new("/home/Library"), super::category("demo.caches"))
                .with_owner(Path::new("/home/movie.mov"), super::category("demo.files"));
            let rule = Named::new(&[]);

            assert_eq!(items(&state, &rule, 0, 10), Vec::<String>::new());
            let asked: Vec<_> = rule.asked.borrow().iter().map(|(name, _)| name.clone()).collect();
            assert_eq!(asked, ["locked", "inner"]);
        }

        #[test]
        fn left_alone_of_a_node_of_another_state_answers_none() {
            let other = NodeId::new(StateId::new(2), 0);

            assert_eq!(state().left_alone(other, &Named::new(&[]), ByteSize::new(0), Timestamp::from_secs(100)), None);
        }

        #[test]
        fn left_alone_skips_a_claimed_a_partial_or_a_recent_folder() {
            let partial = Sizes { completeness: Completeness::Partial, ..super::bytes(100) };
            let nodes = [
                ("/home", NodeKind::Folder, super::bytes(900)),
                ("/home/claimed", NodeKind::Folder, super::bytes(100)),
                ("/home/partial", NodeKind::Folder, partial),
                ("/home/recent", NodeKind::Folder, super::bytes(100)),
                ("/home/unknown", NodeKind::Folder, super::bytes(100)),
                ("/home/file", NodeKind::File, super::bytes(100)),
            ];
            let nodes = nodes.map(|(path, kind, sizes)| (PathBuf::from(path), kind, sizes));
            let old = Timestamp::from_secs(10);
            let state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(500), nodes)
                .with_owner(Path::new("/home/claimed"), super::category("demo.claimed"))
                .with_newest(Path::new("/home/claimed"), old)
                .with_newest(Path::new("/home/partial"), old)
                .with_newest(Path::new("/home/recent"), Timestamp::from_secs(100))
                .with_newest(Path::new("/home/file"), old);

            let found =
                state.left_alone(state.roots()[0], &Named::new(&[]), ByteSize::new(0), Timestamp::from_secs(100));

            assert_eq!(found, Some(Vec::new()));
        }

        #[test]
        fn items_skip_what_the_rule_skips_and_what_is_below_the_minimum() {
            let rule = Named::new(&[("Library", ItemVisit::Skip)]);

            assert_eq!(items(&state(), &rule, 150, 10), ["movie.mov"]);
            let asked: Vec<_> = rule.asked.borrow().iter().map(|(name, _)| name.clone()).collect();
            assert_eq!(asked, ["Library", "movie.mov"]);
        }

        #[test]
        fn a_node_id_of_another_state_is_refused() {
            let state = state();
            let other = NodeId::new(StateId::new(2), 0);

            assert_eq!(state.node(other), None);
            assert_eq!(state.children(other, ChildOrder::ByName, 0, 10), None);
            assert_eq!(state.path(other), None);
            assert_eq!(state.node(NodeId::new(StateId::new(1), 99)), None);
        }

        #[test]
        fn a_folder_inside_a_not_scanned_one_is_not_listed_on_its_own() {
            let reason = NotScanned::Unreadable(Unreadable::NoPermission);
            let state = state()
                .with_not_scanned(Path::new("/home/locked"), reason)
                .with_not_scanned(Path::new("/home/locked/inner"), reason)
                .with_not_scanned(Path::new("/elsewhere"), NotScanned::Avoided);

            let places = state.unreadable(10);

            assert_eq!(places.len(), 1);
            assert_eq!(
                (places[0].node.name.as_str(), places[0].reason, places[0].hidden_folders),
                ("locked", reason, 1)
            );
        }

        #[test]
        fn a_search_with_no_hit_finds_nothing() {
            assert_eq!(state().search("nothing-like-it", 10), Vec::new());
        }

        #[test]
        fn a_state_without_claims_owns_nothing() {
            assert_eq!(state().owned(), Vec::new());
        }

        #[test]
        fn a_claim_on_no_node_is_not_owned() {
            let state = state().with_owner(Path::new("/home/missing"), super::category("demo.caches"));

            assert_eq!(state.owned(), Vec::new());
        }

        #[test]
        fn find_outside_every_root_is_none() {
            let state = state();

            assert_eq!(state.find(Path::new("/elsewhere/Library")), None);
            assert_eq!(state.find(Path::new("/home/Library/missing")), None);
        }

        #[test]
        fn find_does_not_match_a_prefix_of_a_name() {
            assert_eq!(state().find(Path::new("/home/Lib")), None);
            assert_eq!(state().find(Path::new("/home/Library/CachesX")), None);
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use fresher_core::{ByteSize, ChildOrder, DiskState, ItemVisit, NodeKind, StateId, Timestamp};

        use super::{Named, category, items, state};

        #[test]
        fn left_alone_answers_the_highest_stale_folder_of_each_branch() {
            let nodes = [
                ("/home", NodeKind::Folder, 900),
                ("/home/work", NodeKind::Folder, 500),
                ("/home/work/old", NodeKind::Folder, 300),
                ("/home/work/old/inner", NodeKind::Folder, 200),
                ("/home/work/fresh", NodeKind::Folder, 200),
                ("/home/big", NodeKind::Folder, 400),
            ];
            let nodes = nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, super::bytes(size)));
            let (old, new) = (Timestamp::from_secs(10), Timestamp::from_secs(200));
            let state = crate::FixedDiskState::new(StateId::new(1), Timestamp::from_secs(500), nodes)
                .with_newest(Path::new("/home/work"), new)
                .with_newest(Path::new("/home/work/old"), old)
                .with_newest(Path::new("/home/work/old/inner"), old)
                .with_newest(Path::new("/home/work/fresh"), new)
                .with_newest(Path::new("/home/big"), Timestamp::from_secs(20));

            let found =
                state.left_alone(state.roots()[0], &Named::new(&[]), ByteSize::new(0), Timestamp::from_secs(100));

            let named: Vec<_> =
                found.unwrap().into_iter().map(|one| (state.node(one.folder).unwrap().name, one.newest)).collect();
            assert_eq!(named, [("big".to_owned(), Timestamp::from_secs(20)), ("old".to_owned(), old)]);
        }

        #[test]
        fn items_come_largest_first_with_their_depth() {
            let rule = Named::new(&[("Caches", ItemVisit::Item), ("movie.mov", ItemVisit::Enter)]);

            assert_eq!(items(&state(), &rule, 0, 10), ["movie.mov", "Caches", "big.cache"]);
            assert_eq!(items(&state(), &Named::new(&[]), 0, 1), ["movie.mov"]);
            let depths = rule.asked.borrow().clone();
            assert_eq!(
                depths,
                [
                    ("Library".to_owned(), 1),
                    ("Caches".to_owned(), 2),
                    ("big.cache".to_owned(), 2),
                    ("locked".to_owned(), 1),
                    ("inner".to_owned(), 2),
                    ("movie.mov".to_owned(), 1),
                ]
            );
        }

        #[test]
        fn items_answer_a_collapsed_subtree_entered_as_one_item() {
            let nodes = [
                ("/home", NodeKind::Folder, 900),
                ("/home/folded", NodeKind::Collapsed, 600),
                ("/home/movie.mov", NodeKind::File, 300),
            ];
            let nodes = nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, super::bytes(size)));
            let state = crate::FixedDiskState::new(StateId::new(1), Timestamp::from_secs(50), nodes);

            assert_eq!(items(&state, &Named::new(&[]), 0, 10), ["folded", "movie.mov"]);
        }

        #[test]
        fn the_roots_and_their_children_page_by_size_or_name() {
            let state = state();
            let roots = state.roots();
            assert_eq!(roots.len(), 1);
            assert_eq!(state.taken(), Timestamp::from_secs(50));
            assert_eq!(state.id(), roots[0].state());

            let by_size = state.children(roots[0], ChildOrder::BySize, 0, 10).unwrap();
            let by_name = state.children(roots[0], ChildOrder::ByName, 1, 1).unwrap();

            let names =
                |page: &fresher_core::NodePage| page.nodes.iter().map(|node| node.name.clone()).collect::<Vec<_>>();
            assert_eq!(names(&by_size), ["movie.mov", "Library", "locked"]);
            assert_eq!((names(&by_name), by_name.offset, by_name.total), (vec!["locked".to_owned()], 1, 3));
        }

        #[test]
        fn a_node_keeps_the_time_it_was_changed() {
            let state = state().with_modified(Path::new("/home/movie.mov"), Timestamp::from_secs(7));
            let movie = state.find(Path::new("/home/movie.mov")).unwrap();
            let library = state.find(Path::new("/home/Library")).unwrap();

            assert_eq!(state.node(movie).unwrap().modified, Some(Timestamp::from_secs(7)));
            assert_eq!(state.node(library).unwrap().modified, None);
        }

        #[test]
        fn a_folder_counts_what_is_inside_it() {
            let state = state();
            let root = state.node(state.roots()[0]).unwrap();

            assert_eq!((root.files, root.folders, root.kind), (2, 4, NodeKind::Folder));
            assert_eq!(state.path(root.id), Some(PathBuf::from("/home")));
        }

        #[test]
        fn largest_files_and_search_put_the_largest_first() {
            let state = state();

            let files: Vec<_> = state.largest_files(10).into_iter().map(|node| node.name).collect();
            let found: Vec<_> = state.search("CACHE", 1).into_iter().map(|node| node.name).collect();

            assert_eq!(files, ["movie.mov", "big.cache"]);
            assert_eq!(found, ["Caches"]);
        }

        #[test]
        fn the_owner_is_recorded_on_its_node() {
            let category = category("demo.caches");
            let state = state().with_owner(Path::new("/home/Library/Caches"), category);

            let found = state.search("caches", 1);

            assert_eq!(found[0].owner, Some(category));
        }

        #[test]
        fn owned_lists_every_claimed_node_ancestors_first() {
            let (caches, files) = (category("demo.caches"), category("demo.files"));
            let state = state()
                .with_owner(Path::new("/home/Library/big.cache"), files)
                .with_owner(Path::new("/home/Library/Caches"), caches)
                .with_owner(Path::new("/home/Library"), caches)
                .with_owner(Path::new("/home/locked/inner"), files);

            let owned: Vec<_> = state.owned().into_iter().map(|(id, owner)| (state.path(id).unwrap(), owner)).collect();

            assert_eq!(
                owned,
                [
                    (PathBuf::from("/home/Library"), caches),
                    (PathBuf::from("/home/Library/Caches"), caches),
                    (PathBuf::from("/home/Library/big.cache"), files),
                    (PathBuf::from("/home/locked/inner"), files),
                ]
            );
        }

        #[test]
        fn find_answers_the_node_of_every_path() {
            let state = state();
            let mut every = state.roots();
            let mut position = 0;
            while let Some(&id) = every.get(position) {
                every.extend(state.children(id, ChildOrder::ByName, 0, u32::MAX).unwrap().nodes.iter().map(|n| n.id));
                position += 1;
            }

            assert_eq!(every.len(), 7);
            for id in every {
                assert_eq!(state.find(&state.path(id).unwrap()), Some(id));
            }
        }
    }
}
