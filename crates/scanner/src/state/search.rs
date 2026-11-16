//! Search by name over the tree in memory: a lowercase copy of the names, one linear substring pass, no index.

use std::fmt;

use crate::state::largest::{Largest, PathOrder};
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// The longest query searched, in bytes; a longer one is cut (at a character boundary).
const MAX_QUERY: usize = 255;

/// Ends each name in [`Lowered::text`]: no name holds it, so no match runs across two names.
const SEPARATOR: char = '\0';

/// The names of the folders and kept files of one tree, lowercased (Unicode lowercase of `std`, with no
/// normalization: a name typed in another normal form is not found), in path order, each followed by [`SEPARATOR`].
/// A name that is not UTF-8 is read lossily. The scan roots are left out: their name is their whole path.
pub(super) struct Lowered {
    text: String,
    /// Where each name starts in `text`, in path order.
    starts: Vec<u32>,
    /// The node of each name, in the same order.
    nodes: Vec<NodeIndex>,
}

impl Lowered {
    /// Built from a frozen tree, once per state.
    pub(super) fn of(tree: &Tree) -> Self {
        let mut lowered = Self { text: String::new(), starts: Vec::new(), nodes: Vec::new() };
        for index in PathOrder::new(tree) {
            let Some(node) = tree.node(index) else { continue };
            if node.parent == NodeIndex::NONE || !(node.flags.has(Flag::Folder) || node.flags.has(Flag::KeptFile)) {
                continue;
            }
            let Ok(start) = u32::try_from(lowered.text.len()) else { break };
            lowered.starts.push(start);
            lowered.nodes.push(index);
            lowered.text.push_str(&String::from_utf8_lossy(tree.name(index)).to_lowercase());
            lowered.text.push(SEPARATOR);
        }
        lowered.text.shrink_to_fit();
        lowered.starts.shrink_to_fit();
        lowered.nodes.shrink_to_fit();
        lowered
    }

    pub(super) fn heap_bytes(&self) -> usize {
        self.text.capacity()
            + self.starts.capacity() * size_of::<u32>()
            + self.nodes.capacity() * size_of::<NodeIndex>()
    }

    /// The nodes whose name contains `query` (from [`lowered_query`]), the largest allocated size first (ties in path
    /// order), at most `limit` (capped at 500). Small files are folded into their folder and have no name in the
    /// tree: a search never finds them.
    pub(super) fn search(&self, tree: &Tree, query: &str, limit: u32) -> Vec<NodeIndex> {
        let mut largest = Largest::new(limit);
        let mut name = 0_usize;
        let mut last_found = None;
        for (at, _) in self.text.match_indices(query) {
            while self.starts.get(name + 1).is_some_and(|&next| next as usize <= at) {
                name += 1;
            }
            if last_found == Some(name) {
                continue;
            }
            last_found = Some(name);
            let Some(&index) = self.nodes.get(name) else { break };
            let allocated = tree.node(index).map_or(0, |node| node.allocated);
            largest.offer(allocated, name, index);
        }
        largest.into_sorted()
    }
}

impl fmt::Debug for Lowered {
    /// Sizes only: the names never reach a log.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Lowered")
            .field("names", &self.nodes.len())
            .field("heap_bytes", &self.heap_bytes())
            .finish_non_exhaustive()
    }
}

/// `query` as it is searched: cut to at most [`MAX_QUERY`] bytes at a character boundary, then lowercased; `None`
/// when it is empty or whitespace only (it finds nothing).
pub(super) fn lowered_query(query: &str) -> Option<String> {
    let query = query.get(..query.floor_char_boundary(MAX_QUERY)).unwrap_or(query);
    (!query.trim().is_empty() && !query.contains(SEPARATOR)).then(|| query.to_lowercase())
}
