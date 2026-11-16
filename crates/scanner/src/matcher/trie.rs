//! The trie of anchored paths, from the file system root down. Built with `*` branches, then made deterministic, so a
//! folder sits at one trie node at most and a child is matched by one lookup.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::matcher::small_list::SmallList;

/// One segment of an anchored path: a name, or `*` (exactly one name).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Segment {
    Literal(Box<[u8]>),
    Any,
}

impl Segment {
    /// The segments of a relative pattern such as `Library/Caches/*`.
    pub(crate) fn parse(pattern: &str) -> Vec<Self> {
        pattern
            .split('/')
            .map(|part| if part == "*" { Self::Any } else { Self::Literal(part.as_bytes().into()) })
            .collect()
    }
}

/// The index of a node of the deterministic trie.
pub(crate) type TrieIndex = u32;

/// A node of the deterministic trie: the next node by exact name, the next node for any other name, and the interests
/// whose path ends here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrieNode {
    pub(crate) literal: HashMap<Box<[u8]>, TrieIndex>,
    pub(crate) any: Option<TrieIndex>,
    pub(crate) ends: SmallList,
}

/// The trie as patterns are added: a node may have both a literal and a `*` branch for the same name.
#[derive(Debug)]
pub(crate) struct TrieBuilder {
    nodes: Vec<Branches>,
}

#[derive(Debug, Default)]
struct Branches {
    literal: HashMap<Box<[u8]>, usize>,
    any: Option<usize>,
    ends: Vec<u32>,
}

impl Default for TrieBuilder {
    fn default() -> Self {
        Self { nodes: vec![Branches::default()] }
    }
}

impl TrieBuilder {
    /// Adds the path `segments` (from the file system root) ending at `interest`.
    pub(crate) fn insert(&mut self, segments: &[Segment], interest: u32) {
        let mut current = 0;
        for segment in segments {
            let fresh = self.nodes.len();
            let Some(node) = self.nodes.get_mut(current) else { return };
            let (next, created) = match segment {
                Segment::Literal(name) => match node.literal.entry(name.clone()) {
                    Entry::Occupied(found) => (*found.get(), false),
                    Entry::Vacant(slot) => (*slot.insert(fresh), true),
                },
                Segment::Any => match node.any {
                    Some(next) => (next, false),
                    None => (*node.any.insert(fresh), true),
                },
            };
            if created {
                self.nodes.push(Branches::default());
            }
            current = next;
        }
        if let Some(node) = self.nodes.get_mut(current) {
            node.ends.push(interest);
        }
    }

    /// The deterministic trie: each of its nodes stands for the set of builder nodes a path can reach. A `*` matches
    /// only a UTF-8 name, as the reference walk does.
    pub(crate) fn build(&self) -> Vec<TrieNode> {
        let mut trie = Vec::new();
        let mut known: HashMap<Vec<usize>, TrieIndex> = HashMap::new();
        let mut pending: Vec<(TrieIndex, Vec<usize>)> = Vec::new();
        let mut intern = |set: Vec<usize>, trie: &mut Vec<TrieNode>, pending: &mut Vec<(TrieIndex, Vec<usize>)>| {
            if let Some(&index) = known.get(&set) {
                return Some(index);
            }
            let index = TrieIndex::try_from(trie.len()).ok()?;
            trie.push(TrieNode::default());
            known.insert(set.clone(), index);
            pending.push((index, set));
            Some(index)
        };
        intern(vec![0], &mut trie, &mut pending);
        while let Some((index, set)) = pending.pop() {
            let members: Vec<&Branches> = set.iter().filter_map(|&member| self.nodes.get(member)).collect();
            let any: Vec<usize> = sorted(members.iter().filter_map(|node| node.any));
            let mut names: Vec<&[u8]> =
                members.iter().flat_map(|node| node.literal.keys().map(|name| &**name)).collect();
            names.sort_unstable();
            names.dedup();
            let mut node = TrieNode::default();
            for name in names {
                let mut targets: Vec<usize> =
                    members.iter().filter_map(|node| node.literal.get(name).copied()).collect();
                if std::str::from_utf8(name).is_ok() {
                    targets.extend_from_slice(&any);
                }
                if let Some(target) = intern(sorted(targets), &mut trie, &mut pending) {
                    node.literal.insert(name.into(), target);
                }
            }
            if !any.is_empty() {
                node.any = intern(any, &mut trie, &mut pending);
            }
            for member in &members {
                node.ends.extend_from_slice(&member.ends);
            }
            node.ends.sort_dedup();
            if let Some(slot) = usize::try_from(index).ok().and_then(|index| trie.get_mut(index)) {
                *slot = node;
            }
        }
        trie
    }
}

fn sorted(values: impl IntoIterator<Item = usize>) -> Vec<usize> {
    let mut values: Vec<usize> = values.into_iter().collect();
    values.sort_unstable();
    values.dedup();
    values
}
