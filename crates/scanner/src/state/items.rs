//! The items query: the largest nodes below one node that the caller's rule calls items, in one pass.

use fresher_core::{ItemRule, ItemVisit, MAX_ITEMS};

use crate::state::kind_of;
use crate::state::largest::Largest;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, Node, NodeIndex};

/// The nodes below `root` that `rule` calls items, of at least `min` allocated bytes, the largest first, ties in path
/// order; at most `limit` (capped at one more than [`MAX_ITEMS`], so a caller can tell that more were left). Depth
/// first with an explicit stack, children by name. A claimed node, a link, a special entry and a hard link counted
/// elsewhere are never offered to `rule`; neither is a node below `min` (a file of it cannot be answered, a folder of
/// it holds nothing that can).
pub(super) fn items(tree: &Tree, root: NodeIndex, rule: &dyn ItemRule, min: u64, limit: u32) -> Vec<NodeIndex> {
    let mut largest = Largest::capped(limit, MAX_ITEMS.saturating_add(1));
    if !tree.node(root).is_some_and(|node| offered(node, min)) {
        return largest.into_sorted();
    }
    let mut stack: Vec<(NodeIndex, u32)> = Vec::new();
    let mut children: Vec<NodeIndex> = Vec::new();
    push_children(tree, root, 1, &mut stack, &mut children);
    let mut rank = 0_usize;
    while let Some((index, depth)) = stack.pop() {
        rank += 1;
        let Some(node) = tree.node(index) else { continue };
        if !offered(node, min) {
            continue;
        }
        let name = String::from_utf8_lossy(tree.name(index));
        match visit(rule, &name, node, depth) {
            ItemVisit::Skip => {}
            ItemVisit::Item => largest.offer(node.allocated, rank, index),
            ItemVisit::Enter => push_children(tree, index, depth.saturating_add(1), &mut stack, &mut children),
        }
    }
    largest.into_sorted()
}

/// Whether the query may look at `node` at all: unclaimed, not a link, a special entry or a hard link counted
/// elsewhere, and at least `min`.
const fn offered(node: &Node, min: u64) -> bool {
    node.owner == Node::NO_OWNER
        && !node.flags.has(Flag::Link)
        && !node.flags.has(Flag::Special)
        && !node.flags.has(Flag::LinkCopy)
        && node.allocated >= min
}

/// The rule's answer for `node`; a kept file and a collapsed subtree have nothing to enter, so their `Enter` is
/// `Item`.
fn visit(rule: &dyn ItemRule, name: &str, node: &Node, depth: u32) -> ItemVisit {
    match rule.visit(name, kind_of(node), depth) {
        ItemVisit::Enter if !node.flags.has(Flag::Folder) || node.flags.has(Flag::Collapsed) => ItemVisit::Item,
        visit => visit,
    }
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
