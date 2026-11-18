//! What a matched folder's own listing holds for the anchored interests that end at it and name entries.

use fresher_core::{CACHEDIR_TAG, EntryKind, FolderListing};

use crate::matcher::Matcher;
use crate::matcher::state::TrieState;
use crate::tree::node::{Held, Hit};

impl Matcher {
    /// For each anchored interest that ends at `state` and names entries, which of them `listing` holds, in one pass
    /// over its names; sorted by interest, the markers left to the walk (it reads a held tag). An entry that could not
    /// be read is absent; any kind counts, but a `CACHEDIR.TAG` only as a regular file. Empty, with no allocation, when
    /// no interest names entries or none ends at `state`.
    pub(crate) fn held(&self, state: TrieState, listing: &FolderListing) -> Vec<Held> {
        if self.held.is_empty() {
            return Vec::new();
        }
        let Some(node) = self.trie_node(state) else { return Vec::new() };
        let mut held: Vec<Held> = node
            .ends
            .as_slice()
            .iter()
            .filter(|&&interest| self.facts(interest).holds)
            .map(|&interest| Held { interest, holds: 0, marker: None })
            .collect();
        if held.is_empty() {
            return held;
        }
        for (name, entry) in listing.iter() {
            if entry.error.is_some() || name == CACHEDIR_TAG.as_bytes() && entry.kind != EntryKind::File {
                continue;
            }
            for slot in self.held.get(name).map(Vec::as_slice).unwrap_or_default() {
                if let Some(found) = held.iter_mut().find(|found| found.interest == slot.interest) {
                    found.holds |= 1 << slot.bit;
                }
            }
        }
        held
    }

    /// Whether the interest names entries its matched folder must be checked for.
    pub(crate) fn holds_names(&self, interest: u32) -> bool {
        self.facts(interest).holds
    }

    /// What `hits` hold when their folder's listing was not read: nothing, and no marker check.
    pub(crate) fn unread(&self, hits: &[Hit]) -> Vec<Held> {
        hits.iter()
            .filter(|hit| self.holds_names(hit.interest))
            .map(|hit| Held { interest: hit.interest, holds: 0, marker: None })
            .collect()
    }

    /// The bit of a `CACHEDIR.TAG` among the interest's held names, whose first bytes the walk must check.
    pub(crate) fn held_tag(&self, interest: u32) -> Option<u8> {
        self.facts(interest).held_tag
    }
}
