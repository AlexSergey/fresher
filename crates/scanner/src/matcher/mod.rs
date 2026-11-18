//! The compiled matcher: every interest of every module in hash maps and one trie, so an entry is matched by a few
//! lookups and no path is built.

pub(crate) mod compile;
pub(crate) mod entry;
mod fingerprint;
pub(crate) mod holds;
pub(crate) mod small_list;
pub(crate) mod state;
pub(crate) mod trie;

use std::collections::HashMap;

use small_list::SmallList;
use trie::{Segment, TrieNode};

/// The interests an entry matched, by index in the interest set, in increasing order.
pub(crate) type Hits = SmallList;

/// A sibling or held name of an interest: bit `bit` of that interest's sibling or held mask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SiblingSlot {
    interest: u32,
    bit: u8,
}

/// What the walk needs to know of one interest beyond its pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct InterestFacts {
    whole: bool,
    cachedir_tag: bool,
    /// The bit of a held `CACHEDIR.TAG` in the interest's held mask.
    held_tag: Option<u8>,
    /// It names entries its matched folder must be checked for.
    holds: bool,
}

/// Every interest compiled once per scan; shared by the workers, read-only.
#[derive(Clone, Debug, Default)]
pub(crate) struct Matcher {
    folder_names: HashMap<Box<[u8]>, SmallList>,
    markers: HashMap<Box<[u8]>, SmallList>,
    extensions: HashMap<Box<[u8]>, SmallList>,
    siblings: HashMap<Box<[u8]>, Vec<SiblingSlot>>,
    held: HashMap<Box<[u8]>, Vec<SiblingSlot>>,
    trie: Vec<TrieNode>,
    interests: Vec<InterestFacts>,
    /// Every anchored interest that resolved, with its absolute segments.
    anchors: Vec<(u32, Vec<Segment>)>,
    fingerprint: u64,
}

impl Matcher {
    /// Whether the interest claims its match whole: nothing inside is matched or kept as a node.
    pub(crate) fn claims_whole(&self, interest: u32) -> bool {
        self.facts(interest).whole
    }

    /// Whether the interest's marker is a `CACHEDIR.TAG`, whose first bytes the walk must check.
    pub(crate) fn checks_cachedir_tag(&self, interest: u32) -> bool {
        self.facts(interest).cachedir_tag
    }

    /// Every anchored interest that resolved, with its segments from the file system root.
    pub(crate) fn anchors(&self) -> &[(u32, Vec<Segment>)] {
        &self.anchors
    }

    fn facts(&self, interest: u32) -> InterestFacts {
        usize::try_from(interest).ok().and_then(|index| self.interests.get(index)).copied().unwrap_or_default()
    }
}

#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;
