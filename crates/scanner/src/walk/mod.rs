//! The full walk: the roots and their boundaries, the readers, the workers and the aggregator.

pub(crate) mod aggregator;
pub(crate) mod boundary;
pub(crate) mod counting;
pub(crate) mod errors;
pub(crate) mod queue;
pub(crate) mod roots;
pub(crate) mod std_reader;
pub(crate) mod worker;

use std::io;
use std::sync::mpsc;

use fresher_core::{CancelToken, FolderReader, InterestSet, Match, NotScanned, WalkError, WalkProgress, WalkSummary};

use crate::matcher::Matcher;
use crate::tree::access::NodeAccess;
use crate::tree::arena::Tree;
use crate::tree::node::Flag;
use crate::tree::times::Times;
use crate::walk::aggregator::{Aggregator, Callbacks};
use crate::walk::boundary::Boundary;
use crate::walk::queue::{Job, Lane, WorkQueue};
use crate::walk::roots::Roots;
use crate::walk::worker::{Context, Report};

/// How many reports may wait for the aggregator per worker: a slow aggregator slows the workers instead of growing
/// memory.
const REPORTS_PER_WORKER: usize = 4;

/// One full walk of prepared roots.
#[derive(Clone, Copy)]
pub(crate) struct Walk<'a> {
    pub(crate) roots: &'a Roots,
    pub(crate) interests: &'a InterestSet,
    pub(crate) matcher: &'a Matcher,
    pub(crate) boundary: &'a Boundary,
    pub(crate) reader: &'a dyn FolderReader,
    pub(crate) threads: usize,
    pub(crate) threshold: u64,
    /// The user the entries' access is read for; `None` owns nothing.
    pub(crate) user: Option<u32>,
}

impl Walk<'_> {
    /// Reads every folder under the roots with `threads` workers; the aggregator runs on the calling thread, which
    /// calls `on_match` and `on_progress`. Returns the frozen tree and the summary, or `Cancelled`.
    pub(crate) fn run(
        &self,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<(Tree, WalkSummary), WalkError> {
        let mut tree = Tree::new();
        let mut summary = WalkSummary::default();
        let mut jobs = Vec::new();
        for root in self.roots.iter() {
            let node = tree.add_root(&root.path, root.identity).map_err(tree_full)?;
            tree.set_access(node, NodeAccess::of(root.access, self.user)).map_err(tree_full)?;
            if self.boundary.is_avoided(&root.path) {
                tree.mark(node, Flag::Avoided).map_err(tree_full)?;
                *summary.not_scanned.entry(NotScanned::Avoided).or_default() += 1;
                continue;
            }
            jobs.push(Job {
                node,
                path: root.path.clone(),
                identity: root.identity,
                state: self.matcher.root_state(&root.path),
                opaque: None,
                root_device: root.identity.device,
                parent_siblings: None,
                is_root: true,
                times: Times::NONE,
            });
        }
        self.finish(tree, summary, jobs, cancel, on_match, on_progress)
    }

    /// Walks every job of `jobs` and all below them into `tree`, whose nodes they name; the aggregator runs on the
    /// calling thread. Returns the frozen tree and `summary` with what the walk saw, or `Cancelled`.
    pub(crate) fn finish(
        &self,
        tree: Tree,
        summary: WalkSummary,
        jobs: Vec<Job>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<(Tree, WalkSummary), WalkError> {
        let queue = WorkQueue::new();
        let callbacks = Callbacks { on_match, on_progress };
        let mut aggregator = Aggregator::new(tree, summary, self.matcher, self.interests, &queue, cancel, callbacks);
        for job in jobs {
            aggregator.push(job);
        }
        aggregator.flush();
        let threads = self.threads.max(1);
        let context = Context {
            matcher: self.matcher,
            boundary: self.boundary,
            reader: self.reader,
            queue: &queue,
            cancel,
            threshold: self.threshold,
            user: self.user,
        };
        let (sender, reports) = mpsc::sync_channel(threads * REPORTS_PER_WORKER);
        let applied = std::thread::scope(|scope| {
            for _ in 0..threads {
                let sender = sender.clone();
                scope.spawn(move || worker::run(context, &sender));
            }
            drop(sender);
            aggregator.run(&reports)
        });
        if cancel.is_cancelled() {
            return Err(WalkError::Cancelled);
        }
        applied.map_err(tree_full)?;
        let Aggregator { mut tree, summary, .. } = aggregator;
        tree.freeze();
        Ok((tree, summary))
    }

    /// Reads the folder of every job of `jobs` with the workers, and nothing below them: one report each, in no
    /// particular order. `Cancelled` when the token is.
    pub(crate) fn read_all(&self, jobs: Vec<Job>, cancel: &CancelToken) -> Result<Vec<Report>, WalkError> {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }
        let queue = WorkQueue::new();
        let threads = self.threads.clamp(1, jobs.len());
        queue.push_all(jobs.into_iter().map(|job| (job, Lane::Rest)));
        let context = Context {
            matcher: self.matcher,
            boundary: self.boundary,
            reader: self.reader,
            queue: &queue,
            cancel,
            threshold: self.threshold,
            user: self.user,
        };
        let (sender, reports) = mpsc::sync_channel(threads * REPORTS_PER_WORKER);
        let read = std::thread::scope(|scope| {
            for _ in 0..threads {
                let sender = sender.clone();
                scope.spawn(move || worker::run(context, &sender));
            }
            drop(sender);
            let mut read = Vec::new();
            for report in reports {
                if cancel.is_cancelled() {
                    queue.cancel();
                }
                read.push(report);
                queue.done();
            }
            read
        });
        if cancel.is_cancelled() {
            return Err(WalkError::Cancelled);
        }
        Ok(read)
    }
}

/// A tree that refuses a change has run out of indices: the walk stops, as if out of memory.
fn tree_full(error: crate::tree::TreeError) -> WalkError {
    tracing::error!(%error, "the scan's tree refused a change");
    WalkError::Commit { kind: io::ErrorKind::OutOfMemory }
}

#[cfg(test)]
mod facts_lens_tests;
#[cfg(test)]
mod holds_tests;
#[cfg(test)]
mod inside_tests;
#[cfg(test)]
mod places_tests;
#[cfg(test)]
pub(crate) mod tests;
