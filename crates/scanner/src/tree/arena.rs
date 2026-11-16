//! The arena: every node in one `Vec`, every name in one buffer, linked by 32-bit indices.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use fresher_core::{Completeness, EntryFacts, EntryKind, FolderIdentity, Timestamp, Unreadable};

use crate::tree::TreeError;
use crate::tree::access::NodeAccess;
use crate::tree::links::{LinkSet, Member};
use crate::tree::names::{NameRef, Names};
use crate::tree::node::{Flag, Hit, Node, NodeFlags, NodeIndex, Sums};

/// A scan root: its node and the identity its folder had when the walk began.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Root {
    pub(crate) node: NodeIndex,
    pub(crate) identity: FolderIdentity,
}

/// The tree of one scan. Built by one owner (the aggregator), then frozen: children sorted by name, totals summed.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tree {
    nodes: Vec<Node>,
    names: Names,
    hits: Vec<Hit>,
    roots: Vec<Root>,
    links: LinkSet,
    clones: LinkSet,
    frozen: bool,
}

/// The kind of a group of paths that share bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shared {
    /// One file under several names: keyed by its file id.
    HardLink,
    /// Files that share blocks: keyed by their clone id.
    Clone,
}

impl Tree {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// An empty tree with room for `nodes` nodes: the cache codec allocates the arena once.
    pub(crate) fn with_capacity(nodes: usize) -> Self {
        Self { nodes: Vec::with_capacity(nodes), ..Self::default() }
    }

    pub(crate) const fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Adds a scan root; its name is its whole path.
    pub(crate) fn add_root(&mut self, path: &Path, identity: FolderIdentity) -> Result<NodeIndex, TreeError> {
        let node = self.push(path.as_os_str().as_bytes(), NodeIndex::NONE, NodeFlags::of(EntryKind::Folder))?;
        self.roots.push(Root { node, identity });
        Ok(node)
    }

    /// Adds an entry named `name` to the folder `parent`, first in its list (O(1); `freeze` sorts the list).
    pub(crate) fn add_child(
        &mut self,
        parent: NodeIndex,
        name: &[u8],
        kind: EntryKind,
    ) -> Result<NodeIndex, TreeError> {
        let first = self.folder(parent)?.first_child;
        let child = self.push(name, parent, NodeFlags::of(kind))?;
        let node = self.node_mut(child)?;
        node.next_sibling = first;
        if kind != EntryKind::Folder {
            node.files = 1;
        }
        self.folder_mut(parent)?.first_child = child;
        Ok(child)
    }

    /// Adds `sums` to the node's own bytes. A sum that does not fit stops at its largest value and makes the node
    /// `Partial`.
    pub(crate) fn add_own(&mut self, index: NodeIndex, sums: Sums) -> Result<(), TreeError> {
        self.check_open()?;
        self.node_mut(index)?.add_sums(sums);
        Ok(())
    }

    /// Adds `folders` folders below `index` that are not kept as nodes (inside an opaque folder).
    pub(crate) fn add_hidden_folders(&mut self, index: NodeIndex, folders: u32) -> Result<(), TreeError> {
        self.check_open()?;
        let node = self.node_mut(index)?;
        node.folders = node.folders.saturating_add(folders);
        Ok(())
    }

    pub(crate) fn mark(&mut self, index: NodeIndex, flag: Flag) -> Result<(), TreeError> {
        self.node_mut(index)?.flags.set(flag);
        Ok(())
    }

    pub(crate) fn mark_unreadable(&mut self, index: NodeIndex, reason: Unreadable) -> Result<(), TreeError> {
        self.check_open()?;
        self.node_mut(index)?.flags.set_unreadable(reason);
        Ok(())
    }

    pub(crate) fn set_modified(&mut self, index: NodeIndex, modified: Option<Timestamp>) -> Result<(), TreeError> {
        self.node_mut(index)?.set_modified(modified);
        Ok(())
    }

    /// Records the access of the node's entry; allowed on a frozen tree, which a rescan copies into a new one.
    pub(crate) fn set_access(&mut self, index: NodeIndex, access: NodeAccess) -> Result<(), TreeError> {
        self.node_mut(index)?.access = access;
        Ok(())
    }

    /// The facts of the node's entry, from its access and its folder's; nothing known (not removable) for a node that
    /// is not in the tree.
    pub(crate) fn facts(&self, index: NodeIndex) -> EntryFacts {
        let Some(node) = self.node(index) else { return EntryFacts::default() };
        let parent = self.node(node.parent).map_or(NodeAccess::NONE, |parent| parent.access);
        node.access.facts(node.flags.kind(), parent)
    }

    /// Records the claim that owns the node: an index into the committed state's table of categories (the cache's
    /// claims table), or [`Node::NO_OWNER`]. Allowed on a frozen tree: the claims are resolved after the walk.
    pub(crate) fn set_owner(&mut self, index: NodeIndex, owner: u32) -> Result<(), TreeError> {
        self.node_mut(index)?.owner = owner;
        Ok(())
    }

    /// Adds `hits` to the node's hits. The hits of a node lie side by side: a node that already has some gets a fresh
    /// run at the end (rare: a marker found after the folder's name matched).
    pub(crate) fn add_hits(&mut self, index: NodeIndex, hits: &[Hit]) -> Result<(), TreeError> {
        if hits.is_empty() {
            return Ok(());
        }
        let node = *self.node(index).ok_or(TreeError::NoSuchNode)?;
        let old = node.hits_start as usize..node.hits_start as usize + usize::from(node.hits_len);
        let len = u16::try_from(old.len() + hits.len()).map_err(|_| TreeError::Full)?;
        let start = if node.hits_len > 0 && old.end == self.hits.len() {
            node.hits_start
        } else {
            let start = u32::try_from(self.hits.len()).map_err(|_| TreeError::Full)?;
            self.hits.extend_from_within(old);
            start
        };
        self.hits.extend_from_slice(hits);
        let node = self.node_mut(index)?;
        node.hits_start = start;
        node.hits_len = len;
        Ok(())
    }

    /// Records that `holder` holds a member of the group `(device, id)` of `kind`, which may count `bytes` once.
    pub(crate) fn add_shared(
        &mut self,
        kind: Shared,
        device: u64,
        id: u64,
        holder: NodeIndex,
        bytes: Sums,
    ) -> Result<(), TreeError> {
        self.check_open()?;
        self.node(holder).ok_or(TreeError::NoSuchNode)?;
        match kind {
            Shared::HardLink => self.links.add(device, id, holder, bytes),
            Shared::Clone => self.clones.add(device, id, holder, bytes),
        }
        Ok(())
    }

    /// Replaces the node's own bytes; the cache codec restores them as they were saved.
    pub(crate) fn set_own(&mut self, index: NodeIndex, sums: Sums) -> Result<(), TreeError> {
        self.check_open()?;
        self.node_mut(index)?.set_sums(sums);
        Ok(())
    }

    /// Replaces the node's flags; the cache codec restores them as they were saved.
    pub(crate) fn set_flags(&mut self, index: NodeIndex, flags: NodeFlags) -> Result<(), TreeError> {
        self.check_open()?;
        self.node_mut(index)?.flags = flags;
        Ok(())
    }

    /// The members of every group of `kind`.
    pub(crate) fn shared(&self, kind: Shared) -> &[Member] {
        match kind {
            Shared::HardLink => self.links.members(),
            Shared::Clone => self.clones.members(),
        }
    }

    /// Ends the build: sorts every folder's children by name bytes, then sums every subtree bottom-up and marks
    /// `Partial` every node above one that was not read in full. Frees the spare capacity of every buffer. A frozen
    /// tree is not changed again: a second call does nothing.
    pub(crate) fn freeze(&mut self) {
        if self.frozen {
            return;
        }
        self.sort_children();
        self.attribute_shared();
        self.sum_bottom_up();
        self.nodes.shrink_to_fit();
        self.names.shrink_to_fit();
        self.hits.shrink_to_fit();
        self.roots.shrink_to_fit();
        self.links.shrink_to_fit();
        self.clones.shrink_to_fit();
        self.frozen = true;
    }

    pub(crate) fn node(&self, index: NodeIndex) -> Option<&Node> {
        self.nodes.get(index.get()?)
    }

    pub(crate) fn name(&self, index: NodeIndex) -> &[u8] {
        self.node(index).map_or(&[], |node| self.names.get(NameRef { offset: node.name_offset, len: node.name_len }))
    }

    pub(crate) fn hits(&self, index: NodeIndex) -> &[Hit] {
        self.node(index)
            .and_then(|node| {
                let start = node.hits_start as usize;
                self.hits.get(start..start + usize::from(node.hits_len))
            })
            .unwrap_or_default()
    }

    /// The node's bytes: its own before `freeze`, its subtree's after.
    pub(crate) fn sums(&self, index: NodeIndex) -> Option<Sums> {
        self.node(index).map(Node::sums)
    }

    pub(crate) fn completeness(&self, index: NodeIndex) -> Option<Completeness> {
        self.node(index).map(
            |node| {
                if node.flags.has(Flag::Partial) { Completeness::Partial } else { Completeness::Complete }
            },
        )
    }

    pub(crate) fn children(&self, index: NodeIndex) -> Children<'_> {
        Children { tree: self, next: self.node(index).map_or(NodeIndex::NONE, |node| node.first_child) }
    }

    pub(crate) fn roots(&self) -> &[Root] {
        &self.roots
    }

    pub(crate) const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// The full path of a node, rebuilt from its parents.
    pub(crate) fn path(&self, index: NodeIndex) -> Option<PathBuf> {
        let mut chain = Vec::new();
        let mut current = index;
        while let Some(node) = self.node(current) {
            chain.push(current);
            current = node.parent;
        }
        let mut chain = chain.into_iter().rev();
        let mut path = PathBuf::from(OsStr::from_bytes(self.name(chain.next()?)));
        for part in chain {
            path.push(OsStr::from_bytes(self.name(part)));
        }
        Some(path)
    }

    /// The node at `path`: found from the root that holds the path, one name at a time.
    pub(crate) fn lookup(&self, path: &Path) -> Option<NodeIndex> {
        self.roots.iter().find_map(|root| {
            let inside = path.strip_prefix(OsStr::from_bytes(self.name(root.node))).ok()?;
            inside.components().try_fold(root.node, |folder, part| match part {
                Component::Normal(name) => self.children(folder).find(|&child| self.name(child) == name.as_bytes()),
                _ => None,
            })
        })
    }

    /// The bytes this tree holds on the heap: the capacity of every buffer.
    pub(crate) fn heap_bytes(&self) -> usize {
        self.nodes.capacity() * size_of::<Node>()
            + self.names.heap_bytes()
            + self.hits.capacity() * size_of::<Hit>()
            + self.roots.capacity() * size_of::<Root>()
            + self.links.heap_bytes()
            + self.clones.heap_bytes()
    }

    fn push(&mut self, name: &[u8], parent: NodeIndex, flags: NodeFlags) -> Result<NodeIndex, TreeError> {
        self.check_open()?;
        let index = NodeIndex::new(self.nodes.len()).ok_or(TreeError::Full)?;
        let name = self.names.push(name)?;
        self.nodes.push(Node::new(name.offset, name.len, parent, flags));
        Ok(index)
    }

    const fn check_open(&self) -> Result<(), TreeError> {
        if self.frozen { Err(TreeError::Frozen) } else { Ok(()) }
    }

    fn node_mut(&mut self, index: NodeIndex) -> Result<&mut Node, TreeError> {
        index.get().and_then(|position| self.nodes.get_mut(position)).ok_or(TreeError::NoSuchNode)
    }

    fn folder(&self, index: NodeIndex) -> Result<&Node, TreeError> {
        self.node(index).filter(|node| node.flags.has(Flag::Folder)).ok_or(TreeError::NotAFolder)
    }

    fn folder_mut(&mut self, index: NodeIndex) -> Result<&mut Node, TreeError> {
        self.node_mut(index)
            .and_then(|node| if node.flags.has(Flag::Folder) { Ok(node) } else { Err(TreeError::NotAFolder) })
    }

    fn sort_children(&mut self) {
        let mut children = Vec::new();
        for position in 0..self.nodes.len() {
            let Some(index) = NodeIndex::new(position) else { break };
            children.clear();
            children.extend(self.children(index));
            if children.len() < 2 {
                continue;
            }
            children.sort_by(|&left, &right| self.name(left).cmp(self.name(right)));
            let mut next = NodeIndex::NONE;
            for &child in children.iter().rev() {
                if let Some(node) = child.get().and_then(|child| self.nodes.get_mut(child)) {
                    node.next_sibling = next;
                }
                next = child;
            }
            if let Some(node) = self.nodes.get_mut(position) {
                node.first_child = next;
            }
        }
    }

    /// Gives each hard link and clone group's bytes to its first member in depth-first name order (the children are
    /// sorted already); a kept file whose link group is counted elsewhere is flagged `LinkCopy`.
    fn attribute_shared(&mut self) {
        if self.links.is_empty() && self.clones.is_empty() {
            return;
        }
        let ranks = self.depth_first_ranks();
        let rank =
            |holder: NodeIndex| holder.get().and_then(|position| ranks.get(position)).copied().unwrap_or(u32::MAX);
        let links = self.links.resolve(rank);
        let clones = self.clones.resolve(rank);
        for (holder, bytes) in links.credited.into_iter().chain(clones.credited) {
            if let Some(node) = holder.get().and_then(|position| self.nodes.get_mut(position)) {
                node.add_sums(bytes);
            }
        }
        for holder in links.others {
            if let Some(node) = holder.get().and_then(|position| self.nodes.get_mut(position))
                && node.flags.has(Flag::KeptFile)
            {
                node.flags.set(Flag::LinkCopy);
            }
        }
    }

    /// Each node's position in a depth-first walk of the roots in order, children in list order.
    fn depth_first_ranks(&self) -> Vec<u32> {
        let mut ranks = vec![u32::MAX; self.nodes.len()];
        let mut stack: Vec<NodeIndex> = self.roots.iter().rev().map(|root| root.node).collect();
        let mut next = 0_u32;
        let mut children = Vec::new();
        while let Some(index) = stack.pop() {
            if let Some(rank) = index.get().and_then(|position| ranks.get_mut(position)) {
                *rank = next;
                next = next.saturating_add(1);
            }
            children.clear();
            children.extend(self.children(index));
            stack.extend(children.iter().rev());
        }
        ranks
    }

    /// A child always comes after its parent in the arena, so the nodes in reverse are a bottom-up order: when a node
    /// is reached, everything below it is already summed.
    fn sum_bottom_up(&mut self) {
        for position in (0..self.nodes.len()).rev() {
            let Some(node) = self.nodes.get_mut(position) else { continue };
            if node.flags.is_incomplete() {
                node.flags.set(Flag::Partial);
            }
            let node = *node;
            let Some(parent) = node.parent.get().and_then(|parent| self.nodes.get_mut(parent)) else { continue };
            let below = u32::from(node.flags.has(Flag::Folder));
            parent.folders = parent.folders.saturating_add(node.folders).saturating_add(below);
            parent.add_sums(node.sums());
            if node.flags.has(Flag::Partial) {
                parent.flags.set(Flag::Partial);
            }
        }
    }
}

#[cfg(test)]
#[path = "arena_tests.rs"]
mod tests;

/// The children of a folder, in list order (by name once frozen).
#[derive(Clone, Debug)]
pub(crate) struct Children<'a> {
    tree: &'a Tree,
    next: NodeIndex,
}

impl Iterator for Children<'_> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        let current = self.next;
        self.next = self.tree.node(current)?.next_sibling;
        Some(current)
    }
}
