//! Writing a frozen tree. The body, after the header: the number of roots and of nodes; every node in depth-first
//! order, children by name; then the hard link groups and the clone groups.
//!
//! A node: its name front-coded against the previous sibling (the shared prefix's length, then the rest), its flags,
//! for a folder its child count and its folders not kept as nodes, its own file count and three sizes, its
//! modification time and its newest change (its subtree's), each as a delta from its parent's, its hits (interest,
//! marker check, sibling bits, held bits), when it has hits its stamp (see `stamp`), its owner (0 for none, else the
//! claim's index + 1), its access (one byte) and, for a root, its folder identity. Totals are not written: the reader
//! sums them again. A node's own bytes are its total minus its children's and minus the shared bytes credited to it,
//! so the reader's `freeze` credits the groups again and reaches the same totals. A collapsed folder's newest change is
//! its subtree's, so nothing of it is lost.
//!
//! A group: its device and id (as deltas from the previous group's), the bytes it counts once, its member count and
//! the depth-first positions of the nodes that hold them (as deltas).

use fresher_core::MarkerCheck;

use crate::codec::CodecError;
use crate::codec::header::{Header, write_len};
use crate::codec::stamp::write_stamp;
use crate::codec::varint::{write_i64, write_u64};
use crate::tree::TreeError;
use crate::tree::arena::{Shared, Tree};
use crate::tree::node::{Flag, Node, NodeFlags, NodeIndex, Sums};

/// The two kinds of groups, in the order they are written.
pub(crate) const GROUP_KINDS: [Shared; 2] = [Shared::HardLink, Shared::Clone];

/// Writes `tree`, which must be frozen, under `header`.
pub(crate) fn encode(tree: &Tree, header: &Header) -> Result<Vec<u8>, CodecError> {
    Ok(Layout::new(tree, header, None)?.write(header, None))
}

/// One member of a group as written: the group's key, the position of the node that holds it, its bytes.
#[derive(Clone, Copy, Debug)]
struct Member {
    device: u64,
    id: u64,
    rank: u32,
    bytes: Sums,
}

/// What is written, worked out before writing: the nodes in order, the members of every group and the bytes each
/// node is credited with.
#[derive(Debug)]
pub(crate) struct Layout<'t> {
    tree: &'t Tree,
    /// By arena position: the folders written as one node, with nothing below them.
    collapsed: Option<&'t [bool]>,
    /// The written nodes in depth-first order.
    order: Vec<NodeIndex>,
    /// For each written node, its previous sibling (the name it is front-coded against).
    previous: Vec<NodeIndex>,
    /// By arena position: the node's place in `order`; for a node inside a collapsed folder, the folder's.
    rank: Vec<u32>,
    /// By arena position: the bytes of the groups credited to the node by `freeze`.
    credited: Vec<Sums>,
    groups: [Vec<Member>; 2],
}

impl<'t> Layout<'t> {
    /// The layout of `tree`, with the folders flagged in `collapsed` (by arena position) written as single nodes.
    pub(crate) fn new(tree: &'t Tree, header: &Header, collapsed: Option<&'t [bool]>) -> Result<Self, CodecError> {
        if !tree.is_frozen() {
            return Err(CodecError::NotFrozen);
        }
        let mut layout = Self {
            tree,
            collapsed,
            order: Vec::with_capacity(tree.len()),
            previous: Vec::with_capacity(tree.len()),
            rank: vec![u32::MAX; tree.len()],
            credited: vec![Sums::ZERO; tree.len()],
            groups: [Vec::new(), Vec::new()],
        };
        layout.walk(header)?;
        layout.inherit_ranks();
        layout.credit_groups();
        Ok(layout)
    }

    /// The written nodes, in depth-first order.
    pub(crate) fn order(&self) -> &[NodeIndex] {
        &self.order
    }

    /// The body. `lengths`, when given, receives the length of each written node's record, in order.
    pub(crate) fn write(&self, header: &Header, mut lengths: Option<&mut Vec<u32>>) -> Vec<u8> {
        let mut out = Vec::new();
        header.write(&mut out);
        write_len(&mut out, self.tree.roots().len());
        write_len(&mut out, self.order.len());
        for (position, &index) in self.order.iter().enumerate() {
            let start = out.len();
            self.write_node(&mut out, position, index, self.is_collapsed(index));
            if let Some(lengths) = lengths.as_deref_mut() {
                lengths.push(u32::try_from(out.len() - start).unwrap_or(u32::MAX));
            }
        }
        for members in &self.groups {
            write_groups(&mut out, members);
        }
        out
    }

    /// The length of the record of the node at `position` in `order` if it were collapsed.
    pub(crate) fn collapsed_len(&self, position: usize) -> usize {
        let mut out = Vec::new();
        if let Some(&index) = self.order.get(position) {
            self.write_node(&mut out, position, index, true);
        }
        out.len()
    }

    fn is_collapsed(&self, index: NodeIndex) -> bool {
        self.collapsed.and_then(|collapsed| collapsed.get(index.get()?)).copied().unwrap_or(false)
    }

    /// Fills `order` and `previous`: every root in turn, depth-first, children in name order, nothing below a
    /// collapsed folder. Checks that every owner is in the claims table.
    fn walk(&mut self, header: &Header) -> Result<(), CodecError> {
        let tree = self.tree;
        let mut stack: Vec<(NodeIndex, NodeIndex)> = Vec::new();
        let mut previous = NodeIndex::NONE;
        for root in tree.roots() {
            stack.push((root.node, previous));
            previous = root.node;
        }
        stack.reverse();
        let mut children = Vec::new();
        while let Some((index, previous)) = stack.pop() {
            let node = tree.node(index).ok_or(CodecError::Tree(TreeError::NoSuchNode))?;
            if node.owner != Node::NO_OWNER && node.owner as usize >= header.claims.len() {
                return Err(CodecError::UnknownOwner);
            }
            if let Some(rank) = index.get().and_then(|position| self.rank.get_mut(position)) {
                *rank = u32::try_from(self.order.len()).unwrap_or(u32::MAX);
            }
            self.order.push(index);
            self.previous.push(previous);
            if self.is_collapsed(index) {
                continue;
            }
            children.clear();
            let mut sibling = NodeIndex::NONE;
            for child in tree.children(index) {
                children.push((child, sibling));
                sibling = child;
            }
            stack.extend(children.iter().rev());
        }
        Ok(())
    }

    /// A child always comes after its parent in the arena: a node inside a collapsed folder takes the rank of its
    /// nearest written ancestor, in one pass.
    fn inherit_ranks(&mut self) {
        for position in 0..self.rank.len() {
            if self.rank[position] != u32::MAX {
                continue;
            }
            let inherited = NodeIndex::new(position)
                .and_then(|index| self.tree.node(index))
                .and_then(|node| node.parent.get())
                .and_then(|parent| self.rank.get(parent).copied());
            self.rank[position] = inherited.unwrap_or(u32::MAX);
        }
    }

    /// Orders every group's members by depth-first position and credits each group's bytes (the largest of its
    /// members', part by part) to its first member, as `freeze` does.
    fn credit_groups(&mut self) {
        for (slot, kind) in GROUP_KINDS.into_iter().enumerate() {
            let mut members: Vec<Member> = self
                .tree
                .shared(kind)
                .iter()
                .map(|member| Member {
                    device: member.device,
                    id: member.id,
                    rank: member.holder.get().and_then(|position| self.rank.get(position)).copied().unwrap_or(u32::MAX),
                    bytes: member.bytes,
                })
                .collect();
            members.sort_by_key(|member| (member.device, member.id, member.rank));
            for group in members.chunk_by_mut(|left, right| (left.device, left.id) == (right.device, right.id)) {
                let bytes = group.iter().fold(Sums::ZERO, |most, member| largest(most, member.bytes));
                for member in group.iter_mut() {
                    member.bytes = bytes;
                }
                let first = group.first().and_then(|first| self.order.get(first.rank as usize)).and_then(|i| i.get());
                if let Some(credited) = first.and_then(|position| self.credited.get_mut(position)) {
                    *credited = credited.saturating_add(bytes);
                }
            }
            self.groups[slot] = members;
        }
    }

    fn write_node(&self, out: &mut Vec<u8>, position: usize, index: NodeIndex, collapsed: bool) {
        let tree = self.tree;
        let Some(node) = tree.node(index) else { return };
        let previous = self.previous.get(position).map_or(&[][..], |&previous| tree.name(previous));
        write_name(out, previous, tree.name(index));
        let credited = index.get().and_then(|position| self.credited.get(position)).copied().unwrap_or(Sums::ZERO);
        let mut flags = node.flags;
        let mut own = minus(node.sums(), credited);
        if node.flags.has(Flag::Folder) {
            let mut hidden = node.folders;
            let mut children = 0_usize;
            if collapsed {
                flags.set(Flag::Collapsed);
            } else {
                for child in tree.children(index) {
                    let Some(child) = tree.node(child) else { continue };
                    children += 1;
                    own = minus(own, child.sums());
                    let below = child.folders.saturating_add(u32::from(child.flags.has(Flag::Folder)));
                    hidden = hidden.saturating_sub(below);
                }
            }
            write_flags(out, flags);
            write_len(out, children);
            write_u64(out, u64::from(hidden));
        } else {
            write_flags(out, flags);
        }
        write_sums(out, own);
        let parent = tree.node(node.parent);
        write_i64(out, node.modified.wrapping_sub(parent.map_or(0, |parent| parent.modified)));
        write_i64(out, node.newest.wrapping_sub(parent.map_or(0, |parent| parent.newest)));
        let hits = tree.hits(index);
        write_len(out, hits.len());
        for hit in hits {
            write_u64(out, u64::from(hit.interest));
            out.push(match hit.marker {
                None => 0,
                Some(MarkerCheck::Valid) => 1,
                Some(MarkerCheck::Invalid) => 2,
            });
            write_u64(out, hit.siblings);
            write_u64(out, hit.holds);
        }
        if !hits.is_empty() {
            write_stamp(out, node.modified, &tree.stamp(index));
        }
        write_u64(out, u64::from(node.owner.wrapping_add(1)));
        out.push(node.access.bits());
        if node.parent == NodeIndex::NONE
            && let Some(root) = tree.roots().iter().find(|root| root.node == index)
        {
            write_u64(out, root.identity.device);
            write_u64(out, root.identity.file_id);
        }
    }
}

fn write_name(out: &mut Vec<u8>, previous: &[u8], name: &[u8]) {
    let shared = previous.iter().zip(name).take_while(|(left, right)| left == right).count();
    let rest = name.get(shared..).unwrap_or_default();
    write_len(out, shared);
    write_len(out, rest.len());
    out.extend_from_slice(rest);
}

fn write_flags(out: &mut Vec<u8>, flags: NodeFlags) {
    write_u64(out, u64::from(flags.bits()));
}

pub(crate) fn write_sums(out: &mut Vec<u8>, sums: Sums) {
    write_u64(out, u64::from(sums.files));
    write_u64(out, sums.logical);
    write_u64(out, sums.allocated);
    write_u64(out, sums.reclaimable);
}

fn write_groups(out: &mut Vec<u8>, members: &[Member]) {
    let groups: Vec<&[Member]> =
        members.chunk_by(|left, right| (left.device, left.id) == (right.device, right.id)).collect();
    write_len(out, groups.len());
    let (mut device, mut id) = (0_u64, 0_u64);
    for group in groups {
        let Some(first) = group.first() else { continue };
        let device_delta = first.device.saturating_sub(device);
        write_u64(out, device_delta);
        write_u64(out, if device_delta == 0 { first.id.saturating_sub(id) } else { first.id });
        (device, id) = (first.device, first.id);
        write_sums(out, first.bytes);
        write_len(out, group.len());
        let mut rank = 0_u32;
        for member in group {
            write_u64(out, u64::from(member.rank.saturating_sub(rank)));
            rank = member.rank;
        }
    }
}

/// `left` minus `right`, part by part, stopping at zero.
fn minus(left: Sums, right: Sums) -> Sums {
    Sums {
        files: left.files.saturating_sub(right.files),
        logical: left.logical.saturating_sub(right.logical),
        allocated: left.allocated.saturating_sub(right.allocated),
        reclaimable: left.reclaimable.saturating_sub(right.reclaimable),
    }
}

fn largest(left: Sums, right: Sums) -> Sums {
    Sums {
        files: left.files.max(right.files),
        logical: left.logical.max(right.logical),
        allocated: left.allocated.max(right.allocated),
        reclaimable: left.reclaimable.max(right.reclaimable),
    }
}
