//! Fitting the body to the cache's cap: when a tree's body would be too large, its smallest subtrees that hold
//! nothing a rescan or a restore needs (no hit, no owner, nothing unreadable or avoided) are collapsed into their
//! folder — its totals and counts only — until the body fits. A tree that does not fit even then is not saved: the
//! next scan is a full one, never a wrong one.
//!
//! The cap applies to the file on disk (the user's decision of 2026-11-15), which the store compresses with deflate
//! level 1: the body is compressed here with the same level to be measured, so a body that fits once compressed is
//! not collapsed. The store also bounds the body before compression (its body limit), checked here as well.

use miniz_oxide::deflate::compress_to_vec;

use crate::codec::CodecError;
use crate::codec::encode::Layout;
use crate::codec::header::Header;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, Node};

/// What a saved file may take: `file` bytes on disk, the envelope included; `body` bytes before compression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Limits {
    pub(crate) file: u64,
    pub(crate) body: u64,
}

/// The store's level (plan 004/01: −60 % on a home folder's body for 49 ms).
const DEFLATE_LEVEL: u8 = 1;
/// Room for the store's envelope header (30 bytes today).
const ENVELOPE_RESERVE: u64 = 64;
/// Tries aimed at the cap before the last one, which collapses every subtree it may.
const ATTEMPTS: usize = 8;

/// The body of `tree` under `header` within `limits`, with the fewest, smallest subtrees collapsed; `TooLarge` when
/// even every collapsible subtree collapsed does not fit (logged).
pub(crate) fn fit(tree: &Tree, header: &Header, limits: Limits) -> Result<Vec<u8>, CodecError> {
    let layout = Layout::new(tree, header, None)?;
    let mut lengths = Vec::with_capacity(tree.len());
    let mut body = layout.write(header, Some(&mut lengths));
    let mut packed = packed_len(&body);
    if fits(&body, packed, limits) {
        return Ok(body);
    }
    let mut plan = Plan::new(tree, &layout, &lengths);
    for attempt in 0..=ATTEMPTS {
        let target = if attempt == ATTEMPTS { 0 } else { target(length(&body), packed, limits) };
        plan.collapse_until(&layout, length(&body), target);
        let collapsed = plan.collapsed_nodes(tree.len());
        body = Layout::new(tree, header, Some(&collapsed))?.write(header, None);
        packed = packed_len(&body);
        if fits(&body, packed, limits) {
            return Ok(body);
        }
        if plan.exhausted() {
            break;
        }
    }
    let error = CodecError::TooLarge { body: length(&body), packed };
    tracing::warn!(body = length(&body), packed, file = limits.file, limit = limits.body, "{error}; not saved");
    Err(error)
}

/// The length of `body` once compressed as the store compresses it.
pub(crate) fn packed_len(body: &[u8]) -> u64 {
    length(&compress_to_vec(body, DEFLATE_LEVEL))
}

fn length(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

fn fits(body: &[u8], packed: u64, limits: Limits) -> bool {
    length(body) <= limits.body && packed.saturating_add(ENVELOPE_RESERVE) <= limits.file
}

/// The body length to aim at: the body limit, and the length that the last body's compression ratio would bring
/// under the file cap, with a twentieth to spare.
fn target(raw: u64, packed: u64, limits: Limits) -> u64 {
    let room = u128::from(limits.file.saturating_sub(ENVELOPE_RESERVE));
    let scaled = u128::from(raw) * room / u128::from(packed.max(1)) * 19 / 20;
    u64::try_from(scaled).unwrap_or(u64::MAX).min(limits.body)
}

/// The candidates for collapsing and the bookkeeping of the ones collapsed so far, by position in the uncollapsed
/// depth-first order.
#[derive(Debug)]
struct Plan<'t> {
    order: &'t [crate::tree::node::NodeIndex],
    /// The position of each position's parent; `u32::MAX` for a root.
    parent: Vec<u32>,
    /// The encoded length of each position's subtree.
    size: Vec<u64>,
    /// The bytes already saved below each position by collapsing.
    saved_below: Vec<u64>,
    collapsed: Vec<bool>,
    /// Collapsible folders with something below them, smallest total allocated size first, deepest first on a tie.
    candidates: Vec<usize>,
    next: usize,
}

impl<'t> Plan<'t> {
    fn new(tree: &Tree, layout: &'t Layout<'_>, lengths: &[u32]) -> Self {
        let order = layout.order();
        let mut position_of = vec![u32::MAX; tree.len()];
        for (position, index) in order.iter().enumerate() {
            if let Some(slot) = index.get().and_then(|arena| position_of.get_mut(arena)) {
                *slot = u32::try_from(position).unwrap_or(u32::MAX);
            }
        }
        let nodes: Vec<Option<&Node>> = order.iter().map(|&index| tree.node(index)).collect();
        let parent: Vec<u32> = nodes
            .iter()
            .map(|node| node.and_then(|node| node.parent.get()).and_then(|arena| position_of.get(arena)).copied())
            .map(|parent| parent.unwrap_or(u32::MAX))
            .collect();
        let mut size: Vec<u64> = lengths.iter().map(|&length| u64::from(length)).collect();
        let mut collapsible: Vec<bool> = order
            .iter()
            .zip(&nodes)
            .map(|(&index, node)| node.is_some_and(may_collapse) && tree.hits(index).is_empty())
            .collect();
        for position in (0..order.len()).rev() {
            let Some(&up) = parent.get(position) else { continue };
            let up = up as usize;
            let (below, ok) = (size.get(position).copied().unwrap_or(0), collapsible.get(position) == Some(&true));
            if let Some(total) = size.get_mut(up) {
                *total = total.saturating_add(below);
            }
            if !ok && let Some(parent_ok) = collapsible.get_mut(up) {
                *parent_ok = false;
            }
        }
        let mut candidates: Vec<usize> = (0..order.len())
            .filter(|&position| {
                let is_folder = nodes[position].is_some_and(|node| node.flags.has(Flag::Folder));
                let has_below = size[position] > lengths.get(position).copied().map_or(0, u64::from);
                is_folder && has_below && collapsible[position] && parent[position] != u32::MAX
            })
            .collect();
        let allocated = |position: usize| nodes[position].map_or(0, |node| node.allocated);
        candidates.sort_by_key(|&position| (allocated(position), std::cmp::Reverse(position)));
        Self {
            order,
            parent,
            size,
            saved_below: vec![0; order.len()],
            collapsed: vec![false; order.len()],
            candidates,
            next: 0,
        }
    }

    const fn exhausted(&self) -> bool {
        self.next >= self.candidates.len()
    }

    /// Collapses the next candidates until the body, `current` bytes now, is estimated at `target` or under.
    fn collapse_until(&mut self, layout: &Layout<'_>, current: u64, target: u64) {
        let mut estimate = current;
        while estimate > target && !self.exhausted() {
            let position = self.candidates[self.next];
            self.next += 1;
            if self.ancestors(position).any(|up| self.collapsed[up]) {
                continue;
            }
            let now = self.size[position].saturating_sub(self.saved_below[position]);
            let after = u64::try_from(layout.collapsed_len(position)).unwrap_or(u64::MAX);
            let Some(saving) = now.checked_sub(after).filter(|&saving| saving > 0) else { continue };
            self.collapsed[position] = true;
            estimate = estimate.saturating_sub(saving);
            let ancestors: Vec<usize> = self.ancestors(position).collect();
            for up in ancestors {
                self.saved_below[up] = self.saved_below[up].saturating_add(saving);
            }
        }
    }

    fn ancestors(&self, position: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.parent.get(position).copied(), |&up| self.parent.get(up as usize).copied())
            .take_while(|&up| up != u32::MAX)
            .map(|up| up as usize)
    }

    /// The collapsed folders by arena position, as `Layout` takes them.
    fn collapsed_nodes(&self, arena: usize) -> Vec<bool> {
        let mut nodes = vec![false; arena];
        for (position, _) in self.collapsed.iter().enumerate().filter(|(_, collapsed)| **collapsed) {
            if let Some(slot) = self.order.get(position).and_then(|index| index.get()).and_then(|i| nodes.get_mut(i)) {
                *slot = true;
            }
        }
        nodes
    }
}

/// Whether the node itself holds nothing that must stay: no owner, nothing unread, not on another volume.
const fn may_collapse(node: &Node) -> bool {
    node.owner == Node::NO_OWNER
        && !node.flags.has(Flag::Partial)
        && !node.flags.is_incomplete()
        && !node.flags.has(Flag::OtherVolume)
}

/// The body with every collapsible subtree collapsed: the smallest body `fit` can reach.
#[cfg(test)]
pub(crate) fn fully_collapsed(tree: &Tree, header: &Header) -> Vec<u8> {
    let layout = Layout::new(tree, header, None).unwrap();
    let mut lengths = Vec::new();
    let body = layout.write(header, Some(&mut lengths));
    let mut plan = Plan::new(tree, &layout, &lengths);
    plan.collapse_until(&layout, length(&body), 0);
    let collapsed = plan.collapsed_nodes(tree.len());
    Layout::new(tree, header, Some(&collapsed)).unwrap().write(header, None)
}

#[cfg(test)]
#[path = "fit_tests.rs"]
mod tests;
