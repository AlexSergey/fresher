//! The largest kept files of a tree, and what the queries share: the tree in path order and a bounded "largest first".

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use fresher_core::FindingsQuery;

use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// The most nodes one query answers, like a page of findings.
pub(super) const MAX_ANSWERS: u32 = FindingsQuery::MAX_LIMIT;

/// The kept file records (every file of 1 MiB and up is one, and every matched file), the largest allocated size
/// first, ties in path order; at most `limit` (capped at [`MAX_ANSWERS`]). A hard link counted at another path has no
/// bytes of its own here and is skipped. One pass over the tree; nothing but the answer is sorted.
pub(super) fn largest_files(tree: &Tree, limit: u32) -> Vec<NodeIndex> {
    let mut largest = Largest::new(limit);
    for (rank, index) in PathOrder::new(tree).enumerate() {
        let Some(node) = tree.node(index) else { continue };
        if node.flags.has(Flag::KeptFile) && !node.flags.has(Flag::LinkCopy) {
            largest.offer(node.allocated, rank, index);
        }
    }
    largest.into_sorted()
}

/// The `limit` largest of the nodes offered, kept in a binary heap of at most `limit` entries; ties go to the lower
/// rank (the earlier in path order).
#[derive(Debug)]
pub(super) struct Largest {
    /// The heap's top is the worst kept: the smallest, then the latest.
    heap: BinaryHeap<(Reverse<u64>, usize, NodeIndex)>,
    limit: usize,
}

impl Largest {
    /// The most room taken up front: a query with no cap must not ask for `u32::MAX` entries.
    const FIRST_CAPACITY: usize = 1 << 10;

    pub(super) fn new(limit: u32) -> Self {
        Self::capped(limit, MAX_ANSWERS)
    }

    /// Keeps at most `limit`, itself capped at `cap`.
    pub(super) fn capped(limit: u32, cap: u32) -> Self {
        let limit = usize::try_from(limit.min(cap)).unwrap_or(0);
        Self { heap: BinaryHeap::with_capacity(limit.saturating_add(1).min(Self::FIRST_CAPACITY)), limit }
    }

    pub(super) fn offer(&mut self, allocated: u64, rank: usize, index: NodeIndex) {
        let entry = (Reverse(allocated), rank, index);
        if self.heap.len() < self.limit {
            self.heap.push(entry);
        } else if self.heap.peek().is_some_and(|worst| entry < *worst) {
            self.heap.pop();
            self.heap.push(entry);
        }
    }

    /// The nodes kept, the largest first.
    pub(super) fn into_sorted(self) -> Vec<NodeIndex> {
        self.heap.into_sorted_vec().into_iter().map(|(_, _, index)| index).collect()
    }
}

/// Every node of a frozen tree, depth first from the roots in order, children by name: the order of their paths.
#[derive(Debug)]
pub(super) struct PathOrder<'a> {
    tree: &'a Tree,
    stack: Vec<NodeIndex>,
    children: Vec<NodeIndex>,
}

impl<'a> PathOrder<'a> {
    pub(super) fn new(tree: &'a Tree) -> Self {
        Self { tree, stack: tree.roots().iter().rev().map(|root| root.node).collect(), children: Vec::new() }
    }
}

impl Iterator for PathOrder<'_> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        let index = self.stack.pop()?;
        self.children.clear();
        self.children.extend(self.tree.children(index));
        self.stack.extend(self.children.iter().rev());
        Some(index)
    }
}
