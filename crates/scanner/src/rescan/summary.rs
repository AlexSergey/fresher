//! The summary of a walk that did not read every folder, rebuilt from its tree.

use fresher_core::{NotScanned, ScanKind, Unreadable, WalkSummary};

use crate::rescan::frozen::Frozen;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// What a full scan of the same disk reports: the entries of every folder read outside an opaque subtree (its folded
/// files and its kept entries, a mount point not counted), the unreadable and not scanned folders. An entry error is
/// counted once per folder: the tree keeps only that the folder is incomplete.
pub(crate) fn summarize(tree: &Tree, kind: ScanKind) -> WalkSummary {
    let frozen = Frozen::new(tree);
    let mut summary = WalkSummary { kind, ..WalkSummary::default() };
    for index in (0..tree.len()).filter_map(NodeIndex::new) {
        let Some(node) = tree.node(index) else { continue };
        if !node.flags.has(Flag::Folder) {
            continue;
        }
        if node.flags.has(Flag::OtherVolume) {
            *summary.not_scanned.entry(NotScanned::OtherVolume).or_default() += 1;
        } else if node.flags.has(Flag::Avoided) {
            *summary.not_scanned.entry(NotScanned::Avoided).or_default() += 1;
        }
        if node.flags.has(Flag::Opaque) {
            continue;
        }
        if let Some(reason) = node.flags.unreadable() {
            *summary.unreadable.entry(reason).or_default() += 1;
            continue;
        }
        if node.flags.has(Flag::Incomplete) {
            *summary.unreadable.entry(Unreadable::Failed).or_default() += 1;
        }
        let kept = tree
            .children(index)
            .filter(|&child| tree.node(child).is_some_and(|child| !child.flags.has(Flag::OtherVolume)))
            .count();
        summary.entries += u64::from(frozen.own(index).files) + kept as u64;
    }
    summary
}
