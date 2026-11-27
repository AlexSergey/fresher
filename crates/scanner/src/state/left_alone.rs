//! The left-alone query over a frozen tree: the highest folders nothing changed in since a time.

use fresher_core::{ItemRule, ItemVisit};

use crate::state::items::offered;
use crate::state::kind_of;
use crate::state::largest::Largest;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, Node, NodeIndex};

/// The folders below `root` that `rule` enters, of at least `min` allocated bytes, read in full, whose newest change is
/// known and older than `before` (as [`crate::tree::node::time_bits`] says), each with that change; the highest of
/// every branch, the largest first, ties in path order, with no cap. Depth first with an explicit stack, children by
/// name. A node `items` would not offer, a file, a collapsed subtree, and what `rule` skips or calls an item are never
/// answered nor entered; a folder looked at that is not left alone is entered.
pub(super) fn left_alone(
    tree: &Tree,
    root: NodeIndex,
    rule: &dyn ItemRule,
    min: u64,
    before: i64,
) -> Vec<(NodeIndex, i64)> {
    let mut largest = Largest::capped(u32::MAX, u32::MAX);
    if tree.node(root).is_some_and(|node| offered(node, min)) {
        let mut stack: Vec<(NodeIndex, u32)> = Vec::new();
        let mut children: Vec<NodeIndex> = Vec::new();
        push_children(tree, root, 1, &mut stack, &mut children);
        let mut rank = 0_usize;
        while let Some((index, depth)) = stack.pop() {
            rank += 1;
            let Some(node) = tree.node(index) else { continue };
            if !is_looked_at(node, min) {
                continue;
            }
            let name = String::from_utf8_lossy(tree.name(index));
            if rule.visit(&name, kind_of(node), depth) != ItemVisit::Enter {
                continue;
            }
            if is_left_alone(node, before) {
                largest.offer(node.allocated, rank, index);
            } else {
                push_children(tree, index, depth.saturating_add(1), &mut stack, &mut children);
            }
        }
    }
    largest.into_sorted().into_iter().filter_map(|index| Some((index, tree.node(index)?.newest))).collect()
}

/// Whether the query looks at `node`: one `items` offers, and a folder that is not a collapsed subtree.
const fn is_looked_at(node: &Node, min: u64) -> bool {
    offered(node, min) && node.flags.has(Flag::Folder) && !node.flags.has(Flag::Collapsed)
}

/// Whether nothing inside the folder changed since `before`: read in full, with a known newest change older than it.
const fn is_left_alone(node: &Node, before: i64) -> bool {
    !node.flags.has(Flag::Partial) && node.newest != Node::NO_TIME && node.newest < before
}

/// Pushes the children of `index` at `depth` so that the first by name is popped first.
fn push_children(
    tree: &Tree,
    index: NodeIndex,
    depth: u32,
    stack: &mut Vec<(NodeIndex, u32)>,
    children: &mut Vec<NodeIndex>,
) {
    children.clear();
    children.extend(tree.children(index));
    stack.extend(children.iter().rev().map(|&child| (child, depth)));
}

#[cfg(test)]
#[path = "left_alone_tests.rs"]
mod tests;
