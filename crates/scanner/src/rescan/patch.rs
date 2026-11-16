//! Patching the cached tree with what changed, in three passes:
//!
//! 1. every folder of the change set to read again is read by the workers, with the same fold as a full scan;
//! 2. a new arena is built by copying the cached tree in pre-order: a folder read again gets its new listing (its
//!    folded bytes, kept files and hits); of its subfolders, one still there under the same name and identity is
//!    copied, a new one (or one whose identity changed) is queued for a full walk, a gone one is left out; a node of
//!    the change set to walk again is queued with nothing below it;
//! 3. the queued folders are walked by the full walk's workers and aggregator into the new arena, which is then
//!    frozen. No hole stays in the arena: nothing dropped is ever copied.
//!
//! The owners are not copied: the scan's claims are committed again.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fresher_core::{CancelToken, EntryKind, FolderIdentity, Pattern, ReadError, WalkError, WalkProgress, WalkSummary};

use crate::matcher::entry::sibling_mask;
use crate::rescan::change_set::{ChangeSet, ancestry};
use crate::rescan::frozen::Frozen;
use crate::tree::TreeError;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, Hit, NodeFlags, NodeIndex};
use crate::walk::Walk;
use crate::walk::boundary::Decision;
use crate::walk::errors::unreadable;
use crate::walk::queue::Job;
use crate::walk::worker::{Body, Child, FolderReport};

/// The patched tree, frozen, and the folders of it that were copied without being read.
#[derive(Debug)]
pub(crate) struct Patched {
    pub(crate) tree: Tree,
    pub(crate) unchanged: Vec<NodeIndex>,
}

/// Patches `old` (frozen) with `set`, reading through `walk`, whose roots must be the cached tree's, in order.
pub(crate) fn patch(
    old: &Tree,
    set: &ChangeSet,
    walk: &Walk<'_>,
    cancel: &CancelToken,
    on_progress: &mut dyn FnMut(WalkProgress),
) -> Result<Patched, WalkError> {
    let devices: Vec<u64> = walk.roots.iter().map(|root| root.identity.device).collect();
    let mut rewalk = set.rewalk.clone();
    let (jobs, mut read) = reread_jobs(old, set, walk, &devices);
    let identities: HashMap<NodeIndex, FolderIdentity> = jobs.iter().map(|job| (job.node, job.identity)).collect();
    for report in walk.read_all(jobs, cancel)? {
        if matches!(report.body, Body::Opaque(_)) {
            rewalk.insert(report.node);
        } else if let Some(&identity) = identities.get(&report.node) {
            read.insert(report.node, (identity, report.body));
        }
    }
    let mut builder = Builder {
        old,
        frozen: Frozen::new(old),
        walk,
        new: Tree::with_capacity(old.len()),
        read,
        rewalk,
        stack: Vec::new(),
        jobs: Vec::new(),
        unchanged: Vec::new(),
    };
    builder.build().map_err(tree_full)?;
    let Builder { new, jobs, unchanged, .. } = builder;
    let (tree, _) = walk.finish(new, WalkSummary::default(), jobs, cancel, &mut |_| {}, on_progress)?;
    Ok(Patched { tree, unchanged })
}

/// The job of each folder to read again, and the answer of those whose identity could not even be asked.
fn reread_jobs(
    old: &Tree,
    set: &ChangeSet,
    walk: &Walk<'_>,
    devices: &[u64],
) -> (Vec<Job>, HashMap<NodeIndex, (FolderIdentity, Body)>) {
    let mut jobs = Vec::new();
    let mut failed = HashMap::new();
    for &node in &set.reread {
        let Some(path) = old.path(node) else { continue };
        let is_root = old.node(node).is_some_and(|found| found.parent == NodeIndex::NONE);
        let root_device = root_index(old, node).and_then(|index| devices.get(index)).copied().unwrap_or(0);
        let identity = if is_root {
            root_index(old, node)
                .and_then(|index| walk.roots.iter().nth(index))
                .map(|root| root.identity)
                .ok_or(ReadError::Failed)
        } else {
            walk.reader.identify(&path)
        };
        match identity {
            Ok(identity) => {
                let state = walk.matcher.root_state(&path);
                let job =
                    Job { node, path, identity, state, opaque: None, root_device, parent_siblings: None, is_root };
                jobs.push(job);
            }
            Err(error) => {
                failed.insert(node, (FolderIdentity { device: 0, file_id: 0 }, Body::Failed(error)));
            }
        }
    }
    (jobs, failed)
}

/// The position of the root that holds `node`.
fn root_index(tree: &Tree, node: NodeIndex) -> Option<usize> {
    let root = ancestry(tree, node).last()?;
    tree.roots().iter().position(|found| found.node == root)
}

/// A folder of the new tree whose old node was reached through a parent read again: what that listing says of it.
#[derive(Clone, Debug)]
struct Fresh {
    siblings: Option<Arc<[(u32, u64)]>>,
    child: Child,
}

/// One old node to bring into the new tree, whose node is already made.
#[derive(Clone, Debug)]
struct Task {
    old: NodeIndex,
    new: NodeIndex,
    root_device: u64,
    is_root: bool,
    fresh: Option<Fresh>,
}

struct Builder<'a> {
    old: &'a Tree,
    frozen: Frozen<'a>,
    walk: &'a Walk<'a>,
    new: Tree,
    /// The folders read again, with the identity they were read as.
    read: HashMap<NodeIndex, (FolderIdentity, Body)>,
    rewalk: HashSet<NodeIndex>,
    stack: Vec<Task>,
    /// The folders to walk in full once the arena is built.
    jobs: Vec<Job>,
    unchanged: Vec<NodeIndex>,
}

impl Builder<'_> {
    fn build(&mut self) -> Result<(), TreeError> {
        for (old_root, root) in self.old.roots().iter().zip(self.walk.roots.iter()) {
            let new = self.new.add_root(&root.path, root.identity)?;
            self.stack.push(Task {
                old: old_root.node,
                new,
                root_device: root.identity.device,
                is_root: true,
                fresh: None,
            });
        }
        self.stack.reverse();
        while let Some(task) = self.stack.pop() {
            if self.rewalk.contains(&task.old) {
                self.rewalk(task)?;
            } else if let Some((identity, body)) = self.read.remove(&task.old) {
                self.reread(task, identity, body)?;
            } else {
                self.copy(&task)?;
            }
        }
        Ok(())
    }

    /// Copies an unchanged node and queues its children.
    fn copy(&mut self, task: &Task) -> Result<(), TreeError> {
        let Some(node) = self.old.node(task.old).copied() else { return Ok(()) };
        let mut flags = node.flags;
        flags.unset(Flag::Partial);
        flags.unset(Flag::LinkCopy);
        self.new.set_flags(task.new, flags)?;
        self.new.set_own(task.new, self.frozen.own(task.old))?;
        self.new.add_hidden_folders(task.new, self.frozen.hidden(task.old))?;
        if task.fresh.is_none() {
            self.new.set_modified(task.new, node.modified())?;
        }
        self.old_hits(task)?;
        for &(kind, member) in self.frozen.members(task.old) {
            self.new.add_shared(kind, member.device, member.id, task.new, member.bytes)?;
        }
        if flags.has(Flag::Folder) {
            self.unchanged.push(task.new);
        }
        for child in self.old.children(task.old) {
            let Some(found) = self.old.node(child) else { continue };
            let new = self.new.add_child(task.new, self.old.name(child), found.flags.kind())?;
            self.stack.push(Task { old: child, new, root_device: task.root_device, is_root: false, fresh: None });
        }
        Ok(())
    }

    /// Gives a folder read again its new listing.
    fn reread(&mut self, task: Task, identity: FolderIdentity, body: Body) -> Result<(), TreeError> {
        if let Some(fresh) = &task.fresh
            && fresh.child.identity != identity
        {
            let Fresh { siblings, child } = fresh.clone();
            self.walk_new(task.new, child, siblings, task.root_device);
            return Ok(());
        }
        self.new.set_flags(task.new, NodeFlags::of(EntryKind::Folder))?;
        self.name_hits(&task)?;
        match body {
            Body::Failed(error) => self.new.mark_unreadable(task.new, unreadable(error)),
            Body::Folder(report) => self.apply(&task, report),
            Body::Opaque(_) => self.rewalk(task),
        }
    }

    fn apply(&mut self, task: &Task, report: FolderReport) -> Result<(), TreeError> {
        let FolderReport { mut batch, children, child_siblings, .. } = report;
        batch.folder = task.new;
        for hit in &mut batch.own_hits {
            hit.siblings = match &task.fresh {
                Some(fresh) => sibling_mask(fresh.siblings.as_deref().unwrap_or_default(), hit.interest),
                // The parent was not read again: its listing, and so the sibling names, did not change.
                None => self
                    .old
                    .hits(task.old)
                    .iter()
                    .find(|old| old.interest == hit.interest)
                    .map_or(hit.siblings, |old| old.siblings),
            };
        }
        let mut created = Vec::with_capacity(batch.kept.len());
        self.new.apply(&batch, &mut created)?;
        let old_folders: HashMap<&[u8], NodeIndex> = self
            .old
            .children(task.old)
            .filter(|&child| self.old.node(child).is_some_and(|node| node.flags.has(Flag::Folder)))
            .map(|child| (self.old.name(child), child))
            .collect();
        let mut children = children.into_iter();
        for (kept, &node) in batch.kept.iter().zip(&created) {
            if kept.entry.kind != EntryKind::Folder {
                continue;
            }
            let Some(child) = children.next() else { break };
            match child.decision {
                Decision::Enter => match old_folders.get(batch.name(kept)) {
                    Some(&old) => {
                        let fresh = Some(Fresh { siblings: child_siblings.clone(), child });
                        self.stack.push(Task { old, new: node, root_device: task.root_device, is_root: false, fresh });
                    }
                    None => self.walk_new(node, child, child_siblings.clone(), task.root_device),
                },
                Decision::NotScanned(reason) => {
                    let other = reason == fresher_core::NotScanned::OtherVolume;
                    self.new.mark(node, if other { Flag::OtherVolume } else { Flag::Avoided })?;
                }
                Decision::Protected => self.new.mark(node, Flag::Avoided)?,
            }
        }
        Ok(())
    }

    /// Queues a node of the change set for a full walk, with nothing below it.
    fn rewalk(&mut self, task: Task) -> Result<(), TreeError> {
        self.new.set_flags(task.new, NodeFlags::of(EntryKind::Folder))?;
        self.name_hits(&task)?;
        if let Some(Fresh { siblings, child }) = task.fresh {
            let whole = self.claims_whole(task.new);
            self.queue(task.new, child.path, child.identity, child.state, whole, siblings, task.root_device, false);
            return Ok(());
        }
        let Some(path) = self.new.path(task.new) else { return Ok(()) };
        let walk = self.walk;
        let identity = if task.is_root {
            walk.roots.iter().find(|root| root.path == path).map(|root| root.identity).ok_or(ReadError::Failed)
        } else {
            walk.reader.identify(&path)
        };
        let identity = match identity {
            Ok(identity) => identity,
            Err(error) => return self.new.mark_unreadable(task.new, unreadable(error)),
        };
        let decision = if task.is_root {
            if walk.boundary.is_avoided(&path) {
                Decision::NotScanned(fresher_core::NotScanned::Avoided)
            } else {
                Decision::Enter
            }
        } else {
            walk.boundary.decide(&path, identity.device, task.root_device)
        };
        match decision {
            Decision::Enter => {
                let state = walk.matcher.root_state(&path);
                let whole = self.claims_whole(task.new);
                self.queue(task.new, path, identity, state, whole, None, task.root_device, task.is_root);
                Ok(())
            }
            Decision::NotScanned(fresher_core::NotScanned::OtherVolume) => self.new.mark(task.new, Flag::OtherVolume),
            Decision::NotScanned(_) | Decision::Protected => self.new.mark(task.new, Flag::Avoided),
        }
    }

    /// Queues a subfolder the cached tree did not have.
    fn walk_new(&mut self, node: NodeIndex, child: Child, siblings: Option<Arc<[(u32, u64)]>>, root_device: u64) {
        let Child { path, identity, state, whole, .. } = child;
        self.queue(node, path, identity, state, whole, siblings, root_device, false);
    }

    #[expect(clippy::too_many_arguments, reason = "the fields of one job, named at each call")]
    fn queue(
        &mut self,
        node: NodeIndex,
        path: std::path::PathBuf,
        identity: FolderIdentity,
        state: crate::matcher::state::TrieState,
        whole: bool,
        parent_siblings: Option<Arc<[(u32, u64)]>>,
        root_device: u64,
        is_root: bool,
    ) {
        let opaque = whole.then_some(node);
        self.jobs.push(Job { node, path, identity, state, opaque, root_device, parent_siblings, is_root });
    }

    fn claims_whole(&self, node: NodeIndex) -> bool {
        self.new.hits(node).iter().any(|hit| self.walk.matcher.claims_whole(hit.interest))
    }

    /// The hits of a node copied as it was. Reached through a parent read again, its name hits come from that
    /// listing already: only its marker hits are kept, with that listing's sibling names.
    fn old_hits(&mut self, task: &Task) -> Result<(), TreeError> {
        let hits: Vec<Hit> = match &task.fresh {
            None => self.old.hits(task.old).to_vec(),
            Some(fresh) => self
                .old
                .hits(task.old)
                .iter()
                .filter(|hit| self.is_marker(hit.interest))
                .map(|hit| Hit {
                    siblings: sibling_mask(fresh.siblings.as_deref().unwrap_or_default(), hit.interest),
                    ..*hit
                })
                .collect(),
        };
        self.new.add_hits(task.new, &hits)
    }

    /// The hits of a node's name, from its parent's listing: kept from the cached tree when that listing was not read
    /// again (it did not change), already made from it when it was. Marker hits come from the node's own listing.
    fn name_hits(&mut self, task: &Task) -> Result<(), TreeError> {
        if task.fresh.is_some() {
            return Ok(());
        }
        if let Some(node) = self.old.node(task.old) {
            self.new.set_modified(task.new, node.modified())?;
        }
        let hits: Vec<Hit> =
            self.old.hits(task.old).iter().filter(|hit| !self.is_marker(hit.interest)).copied().collect();
        self.new.add_hits(task.new, &hits)
    }

    fn is_marker(&self, interest: u32) -> bool {
        self.walk
            .interests
            .get(interest as usize)
            .is_some_and(|interest| matches!(interest.pattern(), Pattern::MarkerFile(_)))
    }
}

/// A tree that refuses a change has run out of indices, as the full walk reports it.
fn tree_full(error: TreeError) -> WalkError {
    tracing::error!(%error, "the patched tree refused a change");
    WalkError::Commit { kind: std::io::ErrorKind::OutOfMemory }
}

#[cfg(test)]
#[path = "patch_tests.rs"]
mod tests;
