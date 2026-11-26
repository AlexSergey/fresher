use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use fresher_core::{
    ByteSize, CancelToken, ClaimedPath, Clock, DiskState, DiskWalk, Match, Subtree, SubtreeQuery, WalkError,
    WalkProgress, WalkRequest, WalkSummary,
};

/// How long a demo's scan lasts at least, from its start: long enough for a recording to show the bar filling, short
/// enough to keep every scenario inside its budget.
pub const DEMO_SCAN_TIME: Duration = Duration::from_millis(3000);

/// How often the paced progress goes out: the engine's throttle lets one through every 50 ms.
const TICK: Duration = Duration::from_millis(50);

/// Waits for a span of time; the app sleeps the scan's thread, a test moves its clock.
pub type Pause = Arc<dyn Fn(Duration) + Send + Sync>;

/// The walk of a demo, in sandbox mode only: the inner walk runs as it is and its matches go on at once, but its
/// progress is held back and then given out rising evenly until `minimum` has passed since the walk began, ending on
/// the walk's last progress. A small demo world is walked in about a second; paced, a recording shows the scan run.
/// A walk that took longer than `minimum`, or ended in an error, is not held.
pub struct PacedWalk {
    inner: Arc<dyn DiskWalk>,
    clock: Arc<dyn Clock>,
    wait: Pause,
    minimum: Duration,
}

impl PacedWalk {
    pub fn new(inner: Arc<dyn DiskWalk>, clock: Arc<dyn Clock>, wait: Pause, minimum: Duration) -> Self {
        Self { inner, clock, wait, minimum }
    }

    /// Gives out `last` rising evenly from nothing until `minimum` after `began`, checking `cancel` at each tick.
    fn pace(
        &self,
        began: std::time::SystemTime,
        last: WalkProgress,
        cancel: &CancelToken,
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<(), WalkError> {
        let start = self.clock.now();
        let span = (began + self.minimum).duration_since(start).unwrap_or(Duration::ZERO);
        let steps = u64::try_from(span.as_nanos() / TICK.as_nanos()).unwrap_or(u64::MAX).max(1);
        let mut sent = WalkProgress::default();
        for step in 1..=steps {
            if cancel.is_cancelled() {
                return Err(WalkError::Cancelled);
            }
            let due =
                start + Duration::from_nanos(share(u64::try_from(span.as_nanos()).unwrap_or(u64::MAX), step, steps));
            if let Ok(left) = due.duration_since(self.clock.now())
                && !left.is_zero()
            {
                (self.wait)(left);
            }
            let shown = WalkProgress {
                entries: share(last.entries, step, steps),
                bytes: ByteSize::new(share(last.bytes.bytes(), step, steps)),
            };
            if shown != sent {
                sent = shown;
                on_progress(shown);
            }
        }
        Ok(())
    }
}

/// `step` of `steps` parts of `whole`, rounded down; the whole at the last step.
fn share(whole: u64, step: u64, steps: u64) -> u64 {
    let part = u128::from(whole) * u128::from(step) / u128::from(steps);
    u64::try_from(part).unwrap_or(whole)
}

impl fmt::Debug for PacedWalk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("PacedWalk").field("minimum", &self.minimum).finish_non_exhaustive()
    }
}

impl DiskWalk for PacedWalk {
    fn walk(
        &self,
        request: &WalkRequest<'_>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        let began = self.clock.now();
        let mut last = WalkProgress::default();
        let mut hold = |progress: WalkProgress| {
            last = WalkProgress { entries: last.entries.max(progress.entries), bytes: last.bytes.max(progress.bytes) };
        };
        let summary = self.inner.walk(request, cancel, on_match, &mut hold)?;
        self.pace(began, last, cancel, on_progress)?;
        Ok(summary)
    }

    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        self.inner.commit(claims)
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        self.inner.subtrees(queries)
    }
}

#[cfg(test)]
#[path = "paced_walk_tests.rs"]
mod tests;
