//! The check of a rescan: the journal is trusted only as far as a sample confirms it. A sample of the folders the
//! rescan copied without reading is read again and compared with the patched tree; one difference the journal does
//! not name when asked again means a full scan.

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

use fresher_core::{CancelToken, Changes, CheckDifference, CheckFailure, EntryKind, FoldedPart, WalkError};

use crate::rescan::change_set::{self, ancestry};
use crate::rescan::frozen::Frozen;
use crate::tree::arena::Tree;
use crate::tree::fold::FolderBatch;
use crate::tree::node::{Flag, NodeIndex, Sums};
use crate::walk::Walk;
use crate::walk::queue::Job;
use crate::walk::worker::Body;

/// How many unchanged folders a rescan reads again to check the journal (plan 004/07/03).
pub(crate) const CHECK_SAMPLE: usize = 32;

/// A folder of the sample and what differed in it.
pub(crate) type Differed = (NodeIndex, CheckDifference);

/// Reads a sample of `unchanged` (folders of `tree`, frozen, that the rescan did not read) and compares each with its
/// node: its kept entries' names and kinds, its kept files' sizes, its folded bytes and whether an entry failed. The
/// tree keeps no file ids, so a file replaced by another of the same size is not seen; the journal reports it. Gives
/// every folder that differed, in the tree's order.
pub(crate) fn check(
    tree: &Tree,
    unchanged: &[NodeIndex],
    walk: &Walk<'_>,
    seed: u64,
    cancel: &CancelToken,
) -> Result<Vec<Differed>, WalkError> {
    let frozen = Frozen::new(tree);
    let mut jobs = Vec::new();
    let mut differed: Vec<Differed> = Vec::new();
    for node in sample(tree, unchanged, seed) {
        let Some(path) = tree.path(node) else { continue };
        let Some(root) = ancestry(tree, node).last() else { continue };
        let Some(root) = tree.roots().iter().find(|found| found.node == root) else { continue };
        let is_root = root.node == node;
        let identity = if is_root { Ok(root.identity) } else { walk.reader.identify(&path) };
        let Ok(identity) = identity else {
            differed.push((node, CheckDifference::NotIdentified));
            continue;
        };
        let state = walk.matcher.root_state(&path);
        let root_device = root.identity.device;
        let times = tree.stamp(node).times;
        jobs.push(Job {
            node,
            path,
            identity,
            state,
            opaque: None,
            root_device,
            parent_siblings: None,
            is_root,
            times,
        });
    }
    for report in walk.read_all(jobs, cancel)? {
        let difference = match &report.body {
            Body::Folder(folder) => difference(&frozen, report.node, &folder.batch),
            Body::Failed(_) => Some(CheckDifference::NotRead),
            Body::Opaque(_) => Some(CheckDifference::Summed),
        };
        differed.extend(difference.map(|difference| (report.node, difference)));
    }
    differed.sort_by_key(|&(node, _)| node);
    Ok(differed)
}

/// The folders of `differed` the journal does not name in `changes`, asked again after the check read them: a
/// folder it names now changed after the rescan first asked, or just before (the system numbers a change a little
/// after it is made). That is no change the journal missed; the rescan reads it again with the new answer.
pub(crate) fn not_named(tree: &Tree, differed: Vec<Differed>, changes: &[Changes]) -> Vec<Differed> {
    let nearest = |path: &PathBuf| change_set::nearest(tree, path);
    let folders: Vec<NodeIndex> = changes.iter().flat_map(|changes| &changes.folders).filter_map(nearest).collect();
    let subtrees: Vec<NodeIndex> = changes.iter().flat_map(|changes| &changes.subtrees).filter_map(nearest).collect();
    differed
        .into_iter()
        .filter(|&(node, _)| !folders.contains(&node) && !ancestry(tree, node).any(|up| subtrees.contains(&up)))
        .collect()
}

/// The check's failure: the first folder of `differed`, with what differed in it and how many folders did; `None`
/// when none did.
pub(crate) fn failure(tree: &Tree, differed: Vec<Differed>) -> Option<CheckFailure> {
    let folders = u32::try_from(differed.len()).unwrap_or(u32::MAX);
    let (node, difference) = differed.into_iter().next()?;
    tracing::warn!(folders, ?difference, "the journal missed changes the check found; a full scan follows");
    Some(CheckFailure { folder: inside_root(tree, node), difference, folders })
}

/// The path of `node` inside its root; empty for a root.
fn inside_root(tree: &Tree, node: NodeIndex) -> PathBuf {
    let root = ancestry(tree, node).last().and_then(|root| tree.path(root));
    match (tree.path(node), root) {
        (Some(path), Some(root)) => path.strip_prefix(&root).map(PathBuf::from).unwrap_or(path),
        _ => PathBuf::new(),
    }
}

/// `CHECK_SAMPLE` folders of `unchanged` that can be read and compared (not opaque, collapsed, avoided, on another
/// volume or unreadable), drawn by `seed` with a weight that grows with the folder's allocated size and doubles for a
/// folder with a hit: where a missed change costs the most. The same seed draws the same sample.
pub(crate) fn sample(tree: &Tree, unchanged: &[NodeIndex], seed: u64) -> Vec<NodeIndex> {
    let mut random = SplitMix(seed);
    let mut keyed: Vec<(f64, NodeIndex)> = unchanged
        .iter()
        .filter_map(|&index| {
            let node = tree.node(index)?;
            let flags = node.flags;
            let skipped = [Flag::Opaque, Flag::Collapsed, Flag::Avoided, Flag::OtherVolume];
            if !flags.has(Flag::Folder) || skipped.iter().any(|&flag| flags.has(flag)) || flags.unreadable().is_some() {
                return None;
            }
            let size = f64::from(64 - node.allocated.leading_zeros());
            let weight = (1.0 + size) * if tree.hits(index).is_empty() { 1.0 } else { 2.0 };
            Some((random.unit().ln() / weight, index))
        })
        .collect();
    keyed.sort_by(|left, right| right.0.total_cmp(&left.0).then(left.1.cmp(&right.1)));
    keyed.into_iter().take(CHECK_SAMPLE).map(|(_, index)| index).collect()
}

/// A kept entry as the check compares it: its name, its kind and, for a file, its logical and allocated sizes.
type Entry<'a> = (&'a [u8], EntryKind, Option<(u64, u64)>);

/// What differs between the folder's listing, folded, and what the node holds; `None` when nothing does.
fn difference(frozen: &Frozen<'_>, node: NodeIndex, batch: &FolderBatch) -> Option<CheckDifference> {
    let tree = frozen.tree;
    let Some(found) = tree.node(node) else { return Some(CheckDifference::NotRead) };
    if batch.incomplete != found.flags.has(Flag::Incomplete) {
        return Some(CheckDifference::Incomplete);
    }
    let mut kept: Vec<Entry<'_>> = batch
        .kept
        .iter()
        .map(|kept| {
            let sizes = (kept.entry.kind == EntryKind::File)
                .then(|| (kept.entry.sizes.logical.bytes(), kept.entry.sizes.allocated.bytes()));
            (batch.name(kept), kept.entry.kind, sizes)
        })
        .collect();
    kept.sort_by(|left, right| left.0.cmp(right.0));
    let mut children: Vec<Entry<'_>> = tree
        .children(node)
        .filter_map(|child| {
            let kind = tree.node(child)?.flags.kind();
            let raw = frozen.raw(child);
            Some((tree.name(child), kind, (kind == EntryKind::File).then_some((raw.logical, raw.allocated))))
        })
        .collect();
    children.sort_by(|left, right| left.0.cmp(right.0));
    entry_difference(&kept, &children).or_else(|| folded_difference(own_of(batch), frozen.own(node)))
}

/// The first difference between the listing's kept entries and the node's children, both sorted by name.
fn entry_difference(listed: &[Entry<'_>], held: &[Entry<'_>]) -> Option<CheckDifference> {
    let name = |name: &[u8]| PathBuf::from(OsStr::from_bytes(name));
    let (mut listed, mut held) = (listed.iter().peekable(), held.iter().peekable());
    loop {
        match (listed.peek(), held.peek()) {
            (None, None) => return None,
            (Some(added), None) => return Some(CheckDifference::EntryAdded(name(added.0))),
            (None, Some(missing)) => return Some(CheckDifference::EntryMissing(name(missing.0))),
            (Some(left), Some(right)) => match left.0.cmp(right.0) {
                Ordering::Less => return Some(CheckDifference::EntryAdded(name(left.0))),
                Ordering::Greater => return Some(CheckDifference::EntryMissing(name(right.0))),
                Ordering::Equal if left.1 != right.1 => return Some(CheckDifference::Kind(name(left.0))),
                Ordering::Equal if left.2 != right.2 => return Some(CheckDifference::Size(name(left.0))),
                Ordering::Equal => {
                    listed.next();
                    held.next();
                }
            },
        }
    }
}

/// The first part in which the folded files of the listing (`listed`) differ from the node's own bytes (`held`).
fn folded_difference(listed: Sums, held: Sums) -> Option<CheckDifference> {
    let part = if listed.files != held.files {
        FoldedPart::Files
    } else if listed.logical != held.logical {
        FoldedPart::Logical
    } else if listed.allocated != held.allocated {
        FoldedPart::Allocated
    } else if listed.reclaimable != held.reclaimable {
        FoldedPart::Reclaimable
    } else {
        return None;
    };
    Some(CheckDifference::Folded(part))
}

/// A folder's own bytes from its folded listing, as `Tree::apply` and `place_file` count them.
fn own_of(batch: &FolderBatch) -> Sums {
    batch.shared.iter().fold(batch.folded, |own, file| {
        let sizes = file.sizes;
        let shared =
            if file.clone.is_some() { sizes.allocated.bytes().saturating_sub(sizes.reclaimable.bytes()) } else { 0 };
        let counted = if file.links > 1 {
            Sums { files: 1, ..Sums::ZERO }
        } else {
            Sums {
                files: 1,
                logical: sizes.logical.bytes(),
                allocated: sizes.allocated.bytes() - shared,
                reclaimable: sizes.reclaimable.bytes(),
            }
        };
        own.saturating_add(counted)
    })
}

/// A small seeded generator of our own (SplitMix64): the same seed, the same sample.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    /// A number in (0, 1].
    fn unit(&mut self) -> f64 {
        let bits = u32::try_from(self.next() >> 32).unwrap_or(u32::MAX);
        (f64::from(bits) + 1.0) / 4_294_967_296.0
    }
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
