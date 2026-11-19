//! The trie of anchored paths, from the file system root down. Built with `*` branches, then made deterministic, so a
//! folder sits at one trie node at most and a child is matched by one lookup.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use fresher_core::is_bundle_id_shaped;

use crate::matcher::small_list::SmallList;

/// One segment of an anchored path: a name, `*` (exactly one name), a `*` that takes only a name shaped as a bundle
/// id, or a `*` that takes only a name ending with one of its lowercased suffixes ([`ends_with_suffix`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Segment {
    Literal(Box<[u8]>),
    Any,
    BundleId,
    Suffixes(Suffixes),
}

/// The lowercased suffixes of a suffixed `*`.
pub(crate) type Suffixes = Box<[Box<[u8]>]>;

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

/// A node of the deterministic trie: the next node by exact name, the next nodes for any other name ending with a
/// suffix of a suffixed `*` that leads from here, the next node for any other name shaped as a bundle id (where a
/// shaped `*` leads from here), the next node for any other name, and the interests whose path ends here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrieNode {
    pub(crate) literal: HashMap<Box<[u8]>, TrieIndex>,
    pub(crate) suffixed: Vec<SuffixBranch>,
    pub(crate) bundle_id: Option<TrieIndex>,
    pub(crate) any: Option<TrieIndex>,
    pub(crate) ends: SmallList,
}

/// Where a name goes that no literal took and whose longest suffix among those of the node's suffixed `*`s is
/// `suffix`. The suffixes a name ends with are all suffixes of the longest one, so it alone tells which suffixed `*`s
/// take the name: one node per combination that can happen, each holding those `*`s' targets, `any`'s, and for
/// `shaped`, the shaped `*`'s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SuffixBranch {
    pub(crate) suffix: Box<[u8]>,
    /// The next node for a name not shaped as a bundle id.
    pub(crate) plain: TrieIndex,
    /// The next node for a name shaped as a bundle id; `plain` when no shaped `*` leads from here.
    pub(crate) shaped: TrieIndex,
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
    bundle_id: Option<usize>,
    suffixed: Vec<(Suffixes, usize)>,
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
                Segment::BundleId => match node.bundle_id {
                    Some(next) => (next, false),
                    None => (*node.bundle_id.insert(fresh), true),
                },
                Segment::Suffixes(suffixes) => {
                    if let Some(&(_, next)) = node.suffixed.iter().find(|(known, _)| known == suffixes) {
                        (next, false)
                    } else {
                        node.suffixed.push((suffixes.clone(), fresh));
                        (fresh, true)
                    }
                }
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
    /// only a UTF-8 name, as the reference walk does; a shaped `*` only a name shaped as a bundle id, and a suffixed
    /// `*` only a name ending with one of its suffixes, which a plain `*` takes too.
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
            let bundle_id: Vec<usize> = sorted(members.iter().filter_map(|node| node.bundle_id));
            let suffixed: Vec<(&[Box<[u8]>], usize)> =
                members.iter().flat_map(|node| node.suffixed.iter().map(|(set, next)| (&**set, *next))).collect();
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
                    let taken =
                        suffixed.iter().filter(|(set, _)| set.iter().any(|suffix| ends_with_suffix(name, suffix)));
                    targets.extend(taken.map(|&(_, next)| next));
                }
                if is_bundle_id_shaped(name) {
                    targets.extend_from_slice(&bundle_id);
                }
                if let Some(target) = intern(sorted(targets), &mut trie, &mut pending) {
                    node.literal.insert(name.into(), target);
                }
            }
            for suffix in longest_first(&suffixed) {
                // Every suffixed `*` with a suffix that ends `suffix` (itself included) takes a name whose longest
                // suffix is `suffix`.
                let taken: Vec<usize> = suffixed
                    .iter()
                    .filter(|(set, _)| set.iter().any(|other| suffix.ends_with(other)))
                    .map(|&(_, next)| next)
                    .chain(any.iter().copied())
                    .collect();
                let Some(plain) = intern(sorted(taken.iter().copied()), &mut trie, &mut pending) else { continue };
                let shaped = if bundle_id.is_empty() {
                    Some(plain)
                } else {
                    intern(sorted(taken.into_iter().chain(bundle_id.iter().copied())), &mut trie, &mut pending)
                };
                if let Some(shaped) = shaped {
                    node.suffixed.push(SuffixBranch { suffix: suffix.into(), plain, shaped });
                }
            }
            if !bundle_id.is_empty() {
                node.bundle_id =
                    intern(sorted(bundle_id.into_iter().chain(any.iter().copied())), &mut trie, &mut pending);
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

/// Whether `name` ends with `suffix` (lowercase), ASCII case ignored, and is longer than it.
pub(crate) fn ends_with_suffix(name: &[u8], suffix: &[u8]) -> bool {
    name.len() > suffix.len()
        && name.get(name.len() - suffix.len()..).is_some_and(|end| end.eq_ignore_ascii_case(suffix))
}

/// Every distinct suffix of `suffixed`, the longest first (then in byte order), so the first that ends a name is its
/// longest.
fn longest_first<'a>(branches: &[(&'a [Box<[u8]>], usize)]) -> Vec<&'a [u8]> {
    let mut all: Vec<&[u8]> = branches.iter().flat_map(|(set, _)| set.iter().map(|suffix| &**suffix)).collect();
    all.sort_unstable_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    all.dedup();
    all
}

fn sorted(values: impl IntoIterator<Item = usize>) -> Vec<usize> {
    let mut values: Vec<usize> = values.into_iter().collect();
    values.sort_unstable();
    values.dedup();
    values
}
