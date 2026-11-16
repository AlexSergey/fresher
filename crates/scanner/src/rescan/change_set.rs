//! The change set: which nodes of the cached tree a rescan reads again, from the journal's changes since the cached
//! position. A folder whose own entries changed is read again with its parent (whose listing holds the folder's
//! modification time, identity and sibling names); a change inside an opaque or collapsed node, a subtree the journal
//! could only name whole, and every unreadable or other-volume folder are walked again from their node.

use std::collections::HashSet;
use std::path::Path;

use fresher_core::{CancelToken, ChangeJournal, FullScanReason, JournalGap, JournalPosition, Timestamp, VolumeId};

use crate::rescan::load::Cached;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// The most folders a rescan reads again, as a share of the tree's folders in percent; past it a full scan is
/// faster (the bench `scan/rescan-limit` of plan 004/07/03: half the folders named still beat a full scan).
pub(crate) const RESCAN_LIMIT_PERCENT: u64 = 50;
/// The oldest cached position the journal is asked about: past it a replay costs more than a full scan, and the
/// journal may no longer reach it (plan 004/01: about three and a half weeks of history).
pub(crate) const MAX_POSITION_AGE_SECS: u64 = 14 * 24 * 60 * 60;

/// The nodes of the cached tree a rescan reads again.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChangeSet {
    /// Folders whose own entries are read again; their subfolders are kept unless gone or new.
    pub(crate) reread: HashSet<NodeIndex>,
    /// Nodes whose subtree is dropped and walked again; no node of `reread` lies at or under one.
    pub(crate) rewalk: HashSet<NodeIndex>,
}

/// What `gather` needs beyond the cache and the journal.
#[derive(Clone, Copy)]
pub(crate) struct Gather<'a> {
    pub(crate) journal: &'a dyn ChangeJournal,
    pub(crate) now: Timestamp,
    /// Some marker interest has sibling names: a folder walked again without its parent's listing could not tell
    /// them, so the parent of every walked node is read as well.
    pub(crate) sibling_markers: bool,
    /// The most folders read again, in percent of the tree's folders; past it a full scan is faster.
    pub(crate) limit_percent: u64,
    pub(crate) cancel: &'a CancelToken,
}

impl ChangeSet {
    /// The folders read again, plus the nodes walked again with every folder below them.
    pub(crate) fn folders(&self, tree: &Tree) -> u64 {
        let below = |&node: &NodeIndex| tree.node(node).map_or(0, |node| u64::from(node.folders));
        let walked: u64 = self.rewalk.iter().map(|node| 1 + below(node)).sum();
        walked + self.reread.len() as u64
    }

    /// Whether `node` or one of its ancestors is walked again.
    pub(crate) fn is_rewalked(&self, tree: &Tree, node: NodeIndex) -> bool {
        ancestry(tree, node).any(|up| self.rewalk.contains(&up))
    }

    /// The change set of `cached` since its journal positions; any doubt is the reason for a full scan.
    pub(crate) fn gather(cached: &Cached, with: Gather<'_>) -> Result<Self, FullScanReason> {
        let tree = &cached.tree;
        if with.now.secs().saturating_sub(cached.header.taken.secs()) > MAX_POSITION_AGE_SECS {
            return Err(FullScanReason::JournalGap(JournalGap::TooOld));
        }
        let mut named = Self::default();
        for (volume, since) in positions(cached)? {
            let changes =
                with.journal.changes_since(&volume, &since, with.cancel).map_err(FullScanReason::JournalGap)?;
            for folder in &changes.folders {
                named.place(tree, folder, false);
            }
            for subtree in &changes.subtrees {
                named.place(tree, subtree, true);
            }
        }
        let mut set = Self::default();
        set.rewalk.extend(named.rewalk.iter().copied());
        let mut always = Vec::new();
        for position in 0..tree.len() {
            let Some(index) = NodeIndex::new(position) else { break };
            if tree
                .node(index)
                .is_some_and(|node| node.flags.unreadable().is_some() || node.flags.has(Flag::OtherVolume))
            {
                set.rewalk.insert(index);
                always.push(index);
            }
        }
        set.rewalk = set.rewalk.iter().copied().filter(|&node| !set.is_rewalked_above(tree, node)).collect();
        let mut with_parents: Vec<NodeIndex> = named.reread.iter().chain(&named.rewalk).copied().collect();
        if with.sibling_markers {
            with_parents.extend(always);
        }
        for &node in &named.reread {
            if !set.is_rewalked(tree, node) {
                set.reread.insert(node);
            }
        }
        for node in with_parents {
            let Some(parent) = tree.node(node).map(|node| node.parent).filter(|&up| up != NodeIndex::NONE) else {
                continue;
            };
            if !set.is_rewalked(tree, parent) {
                set.reread.insert(parent);
            }
        }
        let total: u64 =
            tree.roots().iter().map(|root| 1 + tree.node(root.node).map_or(0, |node| u64::from(node.folders))).sum();
        if set.folders(tree) * 100 > total * with.limit_percent {
            return Err(FullScanReason::TooManyChanges);
        }
        Ok(set)
    }

    /// Places the changed folder at `path` on its node, or its nearest ancestor in the tree; a path under no root is
    /// ignored, and so is a change in a place the walk does not open.
    fn place(&mut self, tree: &Tree, path: &Path, subtree: bool) {
        let Some(node) = nearest(tree, path) else { return };
        let Some(found) = tree.node(node) else { return };
        if found.flags.has(Flag::Avoided) {
            return;
        }
        let whole = found.flags.has(Flag::Opaque) || found.flags.has(Flag::Collapsed);
        if subtree || whole || found.flags.unreadable().is_some() || found.flags.has(Flag::OtherVolume) {
            self.rewalk.insert(node);
        } else {
            self.reread.insert(node);
        }
    }

    fn is_rewalked_above(&self, tree: &Tree, node: NodeIndex) -> bool {
        ancestry(tree, node).skip(1).any(|up| self.rewalk.contains(&up))
    }
}

/// The volumes of the roots, once each, with the position the cache holds for them.
fn positions(cached: &Cached) -> Result<Vec<(VolumeId, JournalPosition)>, FullScanReason> {
    let mut asked: Vec<(VolumeId, JournalPosition)> = Vec::new();
    for root in &cached.header.roots {
        let position = root.journal.ok_or(FullScanReason::JournalGap(JournalGap::NoJournal))?;
        if !asked.iter().any(|&(volume, _)| volume == root.volume) {
            asked.push((root.volume, position));
        }
    }
    Ok(asked)
}

/// The folder node at `path` or the nearest ancestor of it in the tree; `None` for a path under no root. A node that
/// is not a folder (a file replaced by a folder) stands for its parent.
fn nearest(tree: &Tree, path: &Path) -> Option<NodeIndex> {
    let root = tree.roots().iter().find(|root| path.starts_with(tree.path(root.node).unwrap_or_default()))?;
    let root_path = tree.path(root.node)?;
    let mut at = path;
    loop {
        if let Some(node) = tree.lookup(at) {
            let found = tree.node(node)?;
            return if found.flags.has(Flag::Folder) { Some(node) } else { Some(found.parent) };
        }
        at = at.parent().filter(|up| up.starts_with(&root_path))?;
    }
}

/// `node`, then each of its ancestors up to its root.
pub(crate) fn ancestry(tree: &Tree, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
    std::iter::successors(Some(node), |&at| tree.node(at).map(|node| node.parent))
        .take_while(|&at| at != NodeIndex::NONE)
}
