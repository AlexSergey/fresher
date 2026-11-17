use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fresher_core::{
    CategoryId, ChildOrder, DiskState, NodeId, NodeInfo, NodeKind, NodePage, NotScanned, Sizes, StateId, Timestamp,
    UnreadablePlace,
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
            modified: None,
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
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{ByteSize, CategoryId, ModuleId, NodeKind, Sizes, StateId, Timestamp};

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

    mod negative {
        use std::path::Path;

        use fresher_core::{ChildOrder, DiskState, NodeId, NotScanned, StateId, Unreadable};

        use super::state;

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

        use fresher_core::{ChildOrder, DiskState, NodeKind, Timestamp};

        use super::{category, state};

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
