//! The aggregator: on the calling thread, it alone grows the tree, queues the subfolders, and reports matches and
//! progress.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use fresher_core::{
    ByteSize, CancelToken, Completeness, EntryKind, InterestSet, Match, NotScanned, Sizes, WalkProgress, WalkSummary,
};

use crate::matcher::Matcher;
use crate::matcher::state::TrieState;
use crate::tree::TreeError;
use crate::tree::arena::Tree;
use crate::tree::fold::SharedFile;
use crate::tree::node::{Flag, Hit, NodeIndex, Sums};
use crate::walk::boundary::Decision;
use crate::walk::errors::unreadable;
use crate::walk::queue::{Job, Lane, WorkQueue};
use crate::walk::worker::{Body, Child, Counted, FolderReport, OpaqueReport, Report};

/// A subtree claimed whole, while its folders are being read.
#[derive(Debug, Default)]
struct OpaqueTotals {
    sums: Sums,
    shared: Vec<SharedFile>,
    sizes: Sizes,
    folders: u32,
    incomplete: bool,
    /// Its folders queued and not yet reported.
    pending: u32,
}

/// The callbacks of the walk, called on the aggregator's thread only.
pub(crate) struct Callbacks<'a> {
    pub(crate) on_match: &'a mut dyn FnMut(Match),
    pub(crate) on_progress: &'a mut dyn FnMut(WalkProgress),
}

pub(crate) struct Aggregator<'a> {
    pub(crate) tree: Tree,
    pub(crate) summary: WalkSummary,
    matcher: &'a Matcher,
    interests: &'a InterestSet,
    queue: &'a WorkQueue,
    cancel: &'a CancelToken,
    callbacks: Callbacks<'a>,
    progress: WalkProgress,
    opaque: HashMap<NodeIndex, OpaqueTotals>,
    created: Vec<NodeIndex>,
    /// The jobs of the report being applied, queued together once it is.
    jobs: Vec<(Job, Lane)>,
}

impl<'a> Aggregator<'a> {
    pub(crate) fn new(
        tree: Tree,
        summary: WalkSummary,
        matcher: &'a Matcher,
        interests: &'a InterestSet,
        queue: &'a WorkQueue,
        cancel: &'a CancelToken,
        callbacks: Callbacks<'a>,
    ) -> Self {
        Self {
            tree,
            summary,
            matcher,
            interests,
            queue,
            cancel,
            callbacks,
            progress: WalkProgress::default(),
            opaque: HashMap::new(),
            created: Vec::new(),
            jobs: Vec::new(),
        }
    }

    /// Adds the job of a folder to those queued after the current report; one inside an opaque subtree counts as
    /// pending there.
    pub(crate) fn push(&mut self, job: Job) {
        let lane = if job.opaque.is_none() && job.state.is_live() { Lane::Interesting } else { Lane::Rest };
        if let Some(root) = job.opaque {
            let totals = self.opaque.entry(root).or_default();
            totals.pending = totals.pending.saturating_add(1);
        }
        self.jobs.push((job, lane));
    }

    /// Queues the jobs added since the last call.
    pub(crate) fn flush(&mut self) {
        if !self.jobs.is_empty() {
            self.queue.push_all(self.jobs.drain(..));
        }
    }

    /// Applies every report until the workers are gone. A cancelled walk, or a tree that refuses a change, stops the
    /// queue; the reports still in flight are drained so no worker waits on a full channel.
    pub(crate) fn run(&mut self, reports: &Receiver<Report>) -> Result<(), TreeError> {
        let mut outcome = Ok(());
        for report in reports {
            if outcome.is_err() || self.cancel.is_cancelled() {
                self.queue.cancel();
                continue;
            }
            outcome = self.receive(report);
            self.flush();
            self.queue.done();
        }
        outcome
    }

    fn receive(&mut self, report: Report) -> Result<(), TreeError> {
        let Report { node, opaque, root_device, body } = report;
        match body {
            Body::Failed(error) => {
                if opaque.is_some_and(|root| root != node) {
                    self.opaque.entry(opaque.unwrap_or(node)).or_default().incomplete = true;
                } else {
                    self.tree.mark_unreadable(node, unreadable(error))?;
                    if let Some(root) = opaque {
                        self.opaque.entry(root).or_default().incomplete = true;
                    } else {
                        *self.summary.unreadable.entry(unreadable(error)).or_default() += 1;
                    }
                }
            }
            Body::Folder(folder) => self.folder(node, root_device, folder)?,
            Body::Opaque(summed) => self.summed(node, opaque, root_device, summed)?,
        }
        if let Some(root) = opaque {
            let totals = self.opaque.entry(root).or_default();
            totals.pending = totals.pending.saturating_sub(1);
            if totals.pending == 0 {
                self.finish_opaque(root)?;
            }
        }
        Ok(())
    }

    fn folder(&mut self, node: NodeIndex, root_device: u64, report: FolderReport) -> Result<(), TreeError> {
        let FolderReport { batch, children, child_siblings, counted } = report;
        self.count(counted, true);
        self.created.clear();
        let mut created = std::mem::take(&mut self.created);
        self.tree.apply(&batch, &mut created)?;
        for hit in &batch.own_hits {
            self.emit(node, hit, EntryKind::Folder, Sizes::ZERO);
        }
        let mut children = children.into_iter();
        for (kept, &child_node) in batch.kept.iter().zip(&created) {
            let entry = &kept.entry;
            let sizes = if entry.kind == EntryKind::File { entry.sizes } else { Sizes::ZERO };
            let folder = entry.kind == EntryKind::Folder;
            for hit in batch.hits_of(kept) {
                if !(folder && self.matcher.claims_whole(hit.interest)) {
                    self.emit(child_node, hit, entry.kind, sizes);
                }
            }
            if folder && let Some(child) = children.next() {
                self.child(child_node, child, root_device, child_siblings.as_ref())?;
            }
        }
        self.created = created;
        Ok(())
    }

    /// Queues a subfolder, or marks why it is not entered.
    fn child(
        &mut self,
        node: NodeIndex,
        child: Child,
        root_device: u64,
        siblings: Option<&Arc<[(u32, u64)]>>,
    ) -> Result<(), TreeError> {
        let Child { path, identity, state, decision, whole } = child;
        match decision {
            Decision::Enter => {
                let opaque = whole.then_some(node);
                let parent_siblings = siblings.map(Arc::clone);
                self.push(Job { node, path, identity, state, opaque, root_device, parent_siblings, is_root: false });
            }
            Decision::NotScanned(reason) => {
                self.tree
                    .mark(node, if reason == NotScanned::OtherVolume { Flag::OtherVolume } else { Flag::Avoided })?;
                *self.summary.not_scanned.entry(reason).or_default() += 1;
            }
            Decision::Protected => self.tree.mark(node, Flag::Avoided)?,
        }
        Ok(())
    }

    fn summed(
        &mut self,
        node: NodeIndex,
        opaque: Option<NodeIndex>,
        root_device: u64,
        report: OpaqueReport,
    ) -> Result<(), TreeError> {
        let root = opaque.unwrap_or(node);
        self.count(report.counted, false);
        self.tree.add_hits(node, &report.own_hits)?;
        for hit in &report.own_hits {
            if !self.matcher.claims_whole(hit.interest) {
                self.emit(node, hit, EntryKind::Folder, Sizes::ZERO);
            }
        }
        let totals = self.opaque.entry(root).or_default();
        totals.sums = totals.sums.saturating_add(report.sums);
        totals.shared.extend(report.shared);
        totals.folders = totals.folders.saturating_add(report.folders);
        totals.incomplete |= report.incomplete;
        totals.sizes = totals.sizes.checked_add(report.sizes).unwrap_or_else(|| {
            totals.incomplete = true;
            totals.sizes
        });
        for child in report.children {
            if child.decision == Decision::Enter {
                self.push(Job {
                    node: NodeIndex::NONE,
                    path: child.path,
                    identity: child.identity,
                    state: TrieState::DEAD,
                    opaque: Some(root),
                    root_device,
                    parent_siblings: None,
                    is_root: false,
                });
            }
        }
        if opaque.is_none() && self.opaque.get(&root).is_some_and(|totals| totals.pending == 0) {
            self.finish_opaque(root)?;
        }
        Ok(())
    }

    /// Every folder of the subtree is read: its totals go to the tree, its whole claims are reported.
    fn finish_opaque(&mut self, root: NodeIndex) -> Result<(), TreeError> {
        let Some(totals) = self.opaque.remove(&root) else { return Ok(()) };
        self.tree.apply_opaque(root, totals.sums, totals.folders, &totals.shared)?;
        let mut sizes = totals.sizes;
        if totals.incomplete {
            self.tree.mark(root, Flag::Incomplete)?;
            sizes.completeness = Completeness::Partial;
        }
        let whole: Vec<Hit> =
            self.tree.hits(root).iter().filter(|hit| self.matcher.claims_whole(hit.interest)).copied().collect();
        for hit in &whole {
            self.emit(root, hit, EntryKind::Folder, sizes);
        }
        Ok(())
    }

    fn count(&mut self, counted: Counted, in_summary: bool) {
        if in_summary {
            self.summary.entries = self.summary.entries.saturating_add(counted.entries);
            if counted.failed > 0 {
                *self.summary.unreadable.entry(fresher_core::Unreadable::Failed).or_default() += counted.failed;
            }
        }
        self.progress.entries = self.progress.entries.saturating_add(counted.entries);
        self.progress.bytes = ByteSize::new(self.progress.bytes.bytes().saturating_add(counted.bytes));
        (self.callbacks.on_progress)(self.progress);
    }

    fn emit(&mut self, node: NodeIndex, hit: &Hit, kind: EntryKind, sizes: Sizes) {
        let (Some(path), Some(interest)) = (self.tree.path(node), self.interests.get(hit.interest as usize)) else {
            return;
        };
        let siblings = (0..64_u32)
            .zip(interest.siblings())
            .filter(|&(bit, _)| hit.siblings & (1 << bit) != 0)
            .map(|(_, name)| name.clone())
            .collect();
        let modified = self.tree.node(node).and_then(crate::tree::node::Node::modified);
        (self.callbacks.on_match)(Match {
            interest: hit.interest as usize,
            path,
            kind,
            sizes,
            modified,
            siblings,
            marker: hit.marker,
        });
    }
}
