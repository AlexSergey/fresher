//! The work queue: folder jobs in two lanes, the interesting lane first.

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use fresher_core::FolderIdentity;

use crate::matcher::state::TrieState;
use crate::tree::node::NodeIndex;
use crate::tree::times::Times;

/// One folder to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Job {
    /// The folder's node; `NodeIndex::NONE` inside an opaque subtree, whose folders are not nodes.
    pub(crate) node: NodeIndex,
    pub(crate) path: PathBuf,
    /// What the parent's listing announced: the reader refuses anything else at the path.
    pub(crate) identity: FolderIdentity,
    pub(crate) state: TrieState,
    /// The opaque subtree the folder is in (its root may be the folder itself); `None` outside one.
    pub(crate) opaque: Option<NodeIndex>,
    /// The device of the folder's scan root: a folder on another device is not entered.
    pub(crate) root_device: u64,
    /// The sibling masks of the parent's listing, for the folder's own marker hits; `None` when it has none.
    pub(crate) parent_siblings: Option<Arc<[(u32, u64)]>>,
    /// A scan root: it is never matched itself.
    pub(crate) is_root: bool,
    /// The folder's own times, from its parent's listing, for its own hits; none for a root.
    pub(crate) times: Times,
}

/// Which lane a job waits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lane {
    /// On the way to an anchored interest: read first, so categories fill in early.
    Interesting,
    Rest,
}

/// The jobs waiting, and how many jobs are not finished yet (waiting, being read, or read but not yet applied).
#[derive(Debug, Default)]
struct Lanes {
    interesting: Vec<Job>,
    rest: Vec<Job>,
    outstanding: usize,
    cancelled: bool,
}

/// A `Mutex` over two stacks (depth first in each lane keeps them short) and a `Condvar`. A job stays outstanding
/// until the aggregator has applied what it read and queued its subfolders, so the queue ends only when no job can
/// add another.
#[derive(Debug, Default)]
pub(crate) struct WorkQueue {
    lanes: Mutex<Lanes>,
    ready: Condvar,
}

impl WorkQueue {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn push(&self, job: Job, lane: Lane) {
        self.push_all([(job, lane)]);
    }

    /// Queues every job of `jobs` at once, so a worker never takes one of them before the others are in.
    pub(crate) fn push_all(&self, jobs: impl IntoIterator<Item = (Job, Lane)>) {
        let mut lanes = self.lock();
        for (job, lane) in jobs {
            lanes.outstanding += 1;
            match lane {
                Lane::Interesting => lanes.interesting.push(job),
                Lane::Rest => lanes.rest.push(job),
            }
        }
        drop(lanes);
        self.ready.notify_all();
    }

    /// The next job, the interesting lane first. Waits while jobs are outstanding; `None` once none is, or once the
    /// queue is cancelled.
    pub(crate) fn pop(&self) -> Option<Job> {
        let mut lanes = self.lock();
        loop {
            if lanes.cancelled {
                return None;
            }
            if let Some(job) = lanes.interesting.pop().or_else(|| lanes.rest.pop()) {
                return Some(job);
            }
            if lanes.outstanding == 0 {
                return None;
            }
            lanes = self.ready.wait(lanes).unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// One job is finished: its batch is applied and its subfolders are queued. The last one wakes every worker.
    pub(crate) fn done(&self) {
        let mut lanes = self.lock();
        lanes.outstanding = lanes.outstanding.saturating_sub(1);
        let finished = lanes.outstanding == 0;
        drop(lanes);
        if finished {
            self.ready.notify_all();
        }
    }

    /// Stops the walk: every `pop` answers `None` from now on.
    pub(crate) fn cancel(&self) {
        self.lock().cancelled = true;
        self.ready.notify_all();
    }

    fn lock(&self) -> MutexGuard<'_, Lanes> {
        self.lanes.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::FolderIdentity;

    use crate::matcher::state::TrieState;
    use crate::tree::node::NodeIndex;
    use crate::tree::times::Times;
    use crate::walk::queue::Job;

    fn job(name: &str) -> Job {
        Job {
            node: NodeIndex::NONE,
            path: PathBuf::from(name),
            identity: FolderIdentity { device: 1, file_id: 1 },
            state: TrieState::DEAD,
            opaque: None,
            root_device: 1,
            parent_siblings: None,
            is_root: false,
            times: Times::NONE,
        }
    }

    mod negative {
        use std::sync::Arc;

        use super::job;
        use crate::walk::queue::{Lane, WorkQueue};

        #[test]
        fn the_queue_ends_when_the_last_busy_worker_finds_it_empty() {
            let queue = Arc::new(WorkQueue::new());
            queue.push(job("a"), Lane::Rest);
            let taken = queue.pop();
            let waiting = std::thread::spawn({
                let queue = Arc::clone(&queue);
                move || queue.pop()
            });

            queue.done();

            assert_eq!(taken.map(|job| job.path), Some("a".into()));
            assert_eq!(waiting.join().unwrap(), None);
            assert_eq!(queue.pop(), None);
        }

        #[test]
        fn a_cancelled_queue_gives_nothing() {
            let queue = WorkQueue::new();
            queue.push(job("a"), Lane::Interesting);

            queue.cancel();

            assert_eq!(queue.pop(), None);
        }
    }

    mod positive {
        use std::sync::Arc;

        use super::job;
        use crate::walk::queue::{Lane, WorkQueue};

        #[test]
        fn the_interesting_lane_is_taken_first_and_each_lane_is_a_stack() {
            let queue = WorkQueue::new();
            for (name, lane) in
                [("r1", Lane::Rest), ("i1", Lane::Interesting), ("r2", Lane::Rest), ("i2", Lane::Interesting)]
            {
                queue.push(job(name), lane);
            }

            let order: Vec<_> = (0..4).filter_map(|_| queue.pop()).map(|job| job.path).collect();

            assert_eq!(order, ["i2", "i1", "r2", "r1"].map(std::path::PathBuf::from));
        }

        #[test]
        fn a_waiting_worker_gets_a_job_pushed_later() {
            let queue = Arc::new(WorkQueue::new());
            queue.push(job("parent"), Lane::Rest);
            let _parent = queue.pop();
            let waiting = std::thread::spawn({
                let queue = Arc::clone(&queue);
                move || queue.pop()
            });

            queue.push(job("child"), Lane::Rest);
            queue.done();

            assert_eq!(waiting.join().unwrap().map(|job| job.path), Some("child".into()));
        }
    }
}
