//! What a frozen tree no longer holds directly: each node's own bytes (its total minus its children's and minus the
//! bytes of the shared groups credited to it), the folders it hides, and the group members it holds. A rescan copies
//! nodes with them; the emit rebuilds a match's sizes from them.

use std::collections::HashMap;

use crate::tree::arena::{Shared, Tree};
use crate::tree::links::Member;
use crate::tree::node::{Flag, NodeIndex, Sums};

/// A frozen tree with the bytes credited to each node and the members each node holds.
#[derive(Debug)]
pub(crate) struct Frozen<'t> {
    pub(crate) tree: &'t Tree,
    credited: HashMap<NodeIndex, Sums>,
    members: HashMap<NodeIndex, Vec<(Shared, Member)>>,
}

impl<'t> Frozen<'t> {
    /// `tree` must be frozen: its groups are then sorted, the first member of each the one credited.
    pub(crate) fn new(tree: &'t Tree) -> Self {
        let mut credited: HashMap<NodeIndex, Sums> = HashMap::new();
        let mut members: HashMap<NodeIndex, Vec<(Shared, Member)>> = HashMap::new();
        for kind in [Shared::HardLink, Shared::Clone] {
            let all = tree.shared(kind);
            for group in all.chunk_by(|left, right| (left.device, left.id) == (right.device, right.id)) {
                let Some(first) = group.first() else { continue };
                let bytes = group.iter().fold(Sums::ZERO, |most, member| largest(most, member.bytes));
                let entry = credited.entry(first.holder).or_default();
                *entry = entry.saturating_add(bytes);
            }
            for member in all {
                members.entry(member.holder).or_default().push((kind, *member));
            }
        }
        Self { tree, credited, members }
    }

    /// The node's own bytes: what `freeze` started from.
    pub(crate) fn own(&self, index: NodeIndex) -> Sums {
        let Some(node) = self.tree.node(index) else { return Sums::ZERO };
        let mut own = minus(node.sums(), self.credited.get(&index).copied().unwrap_or(Sums::ZERO));
        for child in self.tree.children(index) {
            if let Some(child) = self.tree.node(child) {
                own = minus(own, child.sums());
            }
        }
        own
    }

    /// The folders below the node that are not nodes (inside an opaque or collapsed folder).
    pub(crate) fn hidden(&self, index: NodeIndex) -> u32 {
        let Some(node) = self.tree.node(index) else { return 0 };
        self.tree.children(index).filter_map(|child| self.tree.node(child)).fold(node.folders, |hidden, child| {
            hidden.saturating_sub(child.folders.saturating_add(u32::from(child.flags.has(Flag::Folder))))
        })
    }

    /// The group members the node holds.
    pub(crate) fn members(&self, index: NodeIndex) -> &[(Shared, Member)] {
        self.members.get(&index).map_or(&[], Vec::as_slice)
    }

    /// The bytes of every file the node counts, each once per name: its own bytes and the bytes of its group
    /// members. A kept file's sizes as the reader saw them; an opaque folder's sum of its files.
    pub(crate) fn raw(&self, index: NodeIndex) -> Sums {
        self.members(index).iter().fold(self.own(index), |sum, (_, member)| sum.saturating_add(member.bytes))
    }
}

/// `left` minus `right`, part by part, stopping at zero.
pub(crate) fn minus(left: Sums, right: Sums) -> Sums {
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
