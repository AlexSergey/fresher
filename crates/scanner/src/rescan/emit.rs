//! Emitting the matches of a tree without reading the disk: every hit, in pre-order, as the `Match` a full scan sends
//! for it.

use fresher_core::{ByteSize, Completeness, EntryKind, Inside, InterestSet, Match, Sizes};

use crate::matcher::Matcher;
use crate::rescan::frozen::Frozen;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex, Sums, masked};

/// Calls `on_match` for every hit of `tree` (frozen): a folder's hit with no size, a whole claim with its subtree's
/// files (partial when something in it was not read), a kept file's hit with its sizes, a link's with none. A whole
/// claim on a folder that is not opaque is not reported, as a full scan does not report it. A whole claim carries the
/// folder's inside as the walk recorded it (an incomplete one if none was). Returns the count.
pub(crate) fn emit(tree: &Tree, matcher: &Matcher, interests: &InterestSet, on_match: &mut dyn FnMut(Match)) -> u64 {
    let frozen = Frozen::new(tree);
    let mut count = 0;
    let mut stack: Vec<NodeIndex> = tree.roots().iter().rev().map(|root| root.node).collect();
    let mut children = Vec::new();
    while let Some(index) = stack.pop() {
        let Some(node) = tree.node(index) else { continue };
        let hits = tree.hits(index);
        if !hits.is_empty() {
            let kind = node.flags.kind();
            let path = tree.path(index);
            let stamp = tree.stamp(index);
            for hit in hits {
                let whole = kind == EntryKind::Folder && matcher.claims_whole(hit.interest);
                let sizes = match kind {
                    EntryKind::Folder if whole => {
                        if !node.flags.has(Flag::Opaque) {
                            continue;
                        }
                        let completeness = if node.flags.has(Flag::Incomplete) {
                            Completeness::Partial
                        } else {
                            Completeness::Complete
                        };
                        Sizes { completeness, ..sizes(frozen.raw(index)) }
                    }
                    EntryKind::File | EntryKind::Other => sizes(frozen.raw(index)),
                    EntryKind::Folder | EntryKind::Symlink => Sizes::ZERO,
                };
                let (Some(path), Some(interest)) = (path.clone(), interests.get(hit.interest as usize)) else {
                    continue;
                };
                let siblings = (0..64_u32)
                    .zip(interest.siblings())
                    .filter(|&(bit, _)| hit.siblings & (1 << bit) != 0)
                    .map(|(_, name)| name.clone())
                    .collect();
                on_match(Match {
                    interest: hit.interest as usize,
                    path,
                    kind,
                    sizes,
                    modified: node.modified(),
                    changed: stamp.times.changed,
                    accessed: stamp.times.accessed,
                    inside: whole.then(|| stamp.inside.unwrap_or(Inside { complete: false, ..Inside::EMPTY })),
                    siblings,
                    holds: masked(interest.holds(), hit.holds),
                    marker: hit.marker,
                    facts: tree.facts(index),
                });
                count += 1;
            }
        }
        children.clear();
        children.extend(tree.children(index));
        stack.extend(children.iter().rev());
    }
    count
}

fn sizes(sums: Sums) -> Sizes {
    Sizes::new(ByteSize::new(sums.logical), ByteSize::new(sums.allocated), ByteSize::new(sums.reclaimable))
}
