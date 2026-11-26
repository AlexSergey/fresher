use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

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

/// The walk of a demo, in sandbox mode only. The inner walk runs as it is: its matches go on at once and its summary is
/// its own. Its progress numbers are made up for the recording, as a function of the time since the scan began: the
/// bytes go evenly toward `target` (the demo volume's used space, which the bar measures against; the walk's own bytes
/// without one) and the entries toward the walk's, reaching both once `minimum` has passed. A progress goes out every
/// 50 ms, never lower than the one before; the end is held until the walk is done and `minimum` has passed. A small
/// demo world is walked in about a second; paced, a recording shows the bar fill. A failed walk ends at once.
pub struct PacedWalk {
    inner: Arc<dyn DiskWalk>,
    clock: Arc<dyn Clock>,
    pause: Pause,
    minimum: Duration,
    target: Option<ByteSize>,
}

impl PacedWalk {
    pub fn new(
        inner: Arc<dyn DiskWalk>,
        clock: Arc<dyn Clock>,
        pause: Pause,
        minimum: Duration,
        target: Option<ByteSize>,
    ) -> Self {
        Self { inner, clock, pause, minimum, target }
    }
}

/// One paced walk's progress: what the walk reported so far, and what went out.
struct Pacer {
    began: SystemTime,
    minimum: Duration,
    target: Option<ByteSize>,
    /// The walk's own progress, the highest it reported.
    walked: WalkProgress,
    sent: WalkProgress,
    ticked: Option<SystemTime>,
}

impl Pacer {
    /// How far the paced progress is at `now`: `minimum`'s share elapsed of the bytes' target and the walk's entries.
    fn at(&self, now: SystemTime) -> WalkProgress {
        let whole = u64::try_from(self.minimum.as_nanos()).unwrap_or(u64::MAX).max(1);
        let elapsed =
            now.duration_since(self.began).map_or(0, |elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
        let elapsed = elapsed.min(whole);
        let bytes = self.target.unwrap_or(self.walked.bytes);
        WalkProgress {
            entries: share(self.walked.entries, elapsed, whole),
            bytes: ByteSize::new(share(bytes.bytes(), elapsed, whole)),
        }
    }

    /// Whether a tick is due at `now`: 50 ms after the last one, or the walk's start.
    fn due(&self, now: SystemTime) -> bool {
        let last = self.ticked.unwrap_or(self.began);
        now.duration_since(last).is_ok_and(|waited| waited >= TICK)
    }

    /// Sends the paced progress at `now` when it rose; it never goes down.
    fn tick(&mut self, now: SystemTime, on_progress: &mut dyn FnMut(WalkProgress)) {
        self.ticked = Some(now);
        let shown = self.at(now);
        let risen =
            WalkProgress { entries: self.sent.entries.max(shown.entries), bytes: self.sent.bytes.max(shown.bytes) };
        if risen != self.sent {
            self.sent = risen;
            on_progress(risen);
        }
    }
}

/// `part` of `whole` parts of `value`, rounded down.
fn share(value: u64, part: u64, whole: u64) -> u64 {
    let shared = u128::from(value) * u128::from(part) / u128::from(whole);
    u64::try_from(shared).unwrap_or(value)
}

impl fmt::Debug for PacedWalk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PacedWalk")
            .field("minimum", &self.minimum)
            .field("target", &self.target)
            .finish_non_exhaustive()
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
        let mut pacer = Pacer {
            began: self.clock.now(),
            minimum: self.minimum,
            target: self.target,
            walked: WalkProgress::default(),
            sent: WalkProgress::default(),
            ticked: None,
        };
        let summary = {
            let mut hold = |progress: WalkProgress| {
                pacer.walked = WalkProgress {
                    entries: pacer.walked.entries.max(progress.entries),
                    bytes: pacer.walked.bytes.max(progress.bytes),
                };
                let now = self.clock.now();
                if pacer.due(now) {
                    pacer.tick(now, on_progress);
                }
            };
            self.inner.walk(request, cancel, on_match, &mut hold)?
        };
        let end = pacer.began + self.minimum;
        loop {
            if cancel.is_cancelled() {
                return Err(WalkError::Cancelled);
            }
            let now = self.clock.now();
            pacer.tick(now, on_progress);
            match end.duration_since(now) {
                Ok(left) if !left.is_zero() => (self.pause)(left.min(TICK)),
                _ => return Ok(summary),
            }
        }
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
