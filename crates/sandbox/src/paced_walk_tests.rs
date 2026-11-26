use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use fresher_core::{
    ByteSize, CancelToken, ClaimedPath, Clock, DiskState, DiskWalk, InterestSet, Match, StateId, Subtree, SubtreeQuery,
    Timestamp, WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
};
use fresher_testing::{FakeClock, FixedDiskState};

use crate::{DEMO_SCAN_TIME, PacedWalk};

/// The moment every walk starts, in seconds after the epoch.
const START: u64 = 1_000;
/// The demo volume's used space the paced bytes go toward: far more than the walk's 4 MB.
const TARGET: u64 = 425_000_000_000;

/// A walk on a fake clock: before each of `progress` it moves the clock by its share of `takes`; it ends with `ends`.
struct FakeWalk {
    clock: Arc<FakeClock>,
    progress: Vec<WalkProgress>,
    takes: Duration,
    ends: Result<(), WalkError>,
}

impl DiskWalk for FakeWalk {
    fn walk(
        &self,
        _request: &WalkRequest<'_>,
        _cancel: &CancelToken,
        _on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        let count = u32::try_from(self.progress.len()).unwrap();
        for progress in &self.progress {
            self.clock.advance(self.takes / count);
            on_progress(*progress);
        }
        self.ends.clone().map(|()| WalkSummary { entries: 7, ..WalkSummary::default() })
    }

    fn commit(&self, _claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        Ok(Arc::new(FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), [])))
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        vec![None; queries.len()]
    }
}

fn progress(entries: u64, bytes: u64) -> WalkProgress {
    WalkProgress { entries, bytes: ByteSize::new(bytes) }
}

/// The walk's progress up to its last, 1000 entries of 4 MB, reported 16 times.
fn rising_progress() -> Vec<WalkProgress> {
    (1..=16).map(|part| progress(1_000 * part / 16, 4_000_000 * part / 16)).collect()
}

/// How a paced walk is played.
struct Play {
    takes: Duration,
    ends: Result<(), WalkError>,
    target: Option<ByteSize>,
    /// Cancels the scan once that many progresses went out.
    cancel_after: Option<usize>,
}

impl Play {
    fn taking(takes: Duration) -> Self {
        Self { takes, ends: Ok(()), target: Some(ByteSize::new(TARGET)), cancel_after: None }
    }
}

/// What a paced walk gave out: each progress with when it went out (from the start), how long it waited in all, and
/// how it ended.
struct Paced {
    sent: Vec<(Duration, WalkProgress)>,
    waited: Duration,
    ended: Result<WalkSummary, WalkError>,
    elapsed: Duration,
}

impl Paced {
    fn last(&self) -> Option<WalkProgress> {
        self.sent.last().map(|(_, last)| *last)
    }
}

fn paced(play: Play) -> Paced {
    let clock = Arc::new(FakeClock::at(START));
    let inner = FakeWalk { clock: Arc::clone(&clock), progress: rising_progress(), takes: play.takes, ends: play.ends };
    let waited = Arc::new(Mutex::new(Duration::ZERO));
    let pause = {
        let (clock, waited) = (Arc::clone(&clock), Arc::clone(&waited));
        Arc::new(move |span: Duration| {
            clock.advance(span);
            *waited.lock().unwrap() += span;
        })
    };
    let walk =
        PacedWalk::new(Arc::new(inner), Arc::clone(&clock) as Arc<dyn Clock>, pause, DEMO_SCAN_TIME, play.target);
    let (cancel, interests) = (CancelToken::new(), InterestSet::new());
    let request = WalkRequest { roots: &[], interests: &interests, avoid: &[], places: &[], mode: WalkMode::Full };
    let start = SystemTime::UNIX_EPOCH + Duration::from_secs(START);
    let mut sent = Vec::new();
    let ended = walk.walk(&request, &cancel, &mut |_| {}, &mut |progress| {
        sent.push((clock.now().duration_since(start).unwrap(), progress));
        if play.cancel_after == Some(sent.len()) {
            cancel.cancel();
        }
    });
    let elapsed = clock.now().duration_since(start).unwrap();
    let waited = *waited.lock().unwrap();
    Paced { sent, waited, ended, elapsed }
}

mod negative {
    use std::time::Duration;

    use fresher_core::WalkError;

    use super::{Play, TARGET, paced, progress};
    use crate::DEMO_SCAN_TIME;

    #[test]
    fn a_walk_slower_than_the_minimum_is_not_held() {
        let walk = paced(Play::taking(Duration::from_secs(5)));

        assert_eq!(walk.waited, Duration::ZERO);
        assert_eq!(walk.elapsed, Duration::from_secs(5));
        assert_eq!(walk.last(), Some(progress(1_000, TARGET)));
        assert!(walk.ended.is_ok());
    }

    #[test]
    fn a_walk_that_fails_ends_at_once_with_its_error() {
        let walk = paced(Play { ends: Err(WalkError::Cancelled), ..Play::taking(Duration::from_millis(200)) });

        assert_eq!(walk.ended, Err(WalkError::Cancelled));
        assert_eq!(walk.waited, Duration::ZERO);
        assert_eq!(walk.elapsed, Duration::from_millis(200));
    }

    #[test]
    fn a_scan_cancelled_while_paced_stops_cancelled() {
        let walk = paced(Play { cancel_after: Some(12), ..Play::taking(Duration::from_millis(200)) });

        assert_eq!(walk.ended, Err(WalkError::Cancelled));
        assert_eq!(walk.sent.len(), 12);
        assert!(walk.elapsed < DEMO_SCAN_TIME);
    }

    #[test]
    fn the_progress_never_goes_down_though_the_walk_holds_far_less_than_the_target() {
        let walk = paced(Play::taking(Duration::from_millis(800)));

        for pair in walk.sent.windows(2) {
            let ((_, before), (_, after)) = (pair[0], pair[1]);
            assert!(after.entries >= before.entries && after.bytes >= before.bytes, "{before:?} then {after:?}");
            assert_ne!(after, before);
        }
        assert_eq!(walk.last().map(|last| last.bytes.bytes()), Some(TARGET));
    }
}

mod positive {
    use std::time::Duration;

    use fresher_core::ByteSize;

    use super::{Play, TARGET, paced, progress};
    use crate::DEMO_SCAN_TIME;

    #[test]
    fn a_fast_walk_lasts_the_minimum_time() {
        let walk = paced(Play::taking(Duration::from_millis(800)));

        assert_eq!(walk.elapsed, DEMO_SCAN_TIME);
        assert_eq!(walk.waited, Duration::from_millis(2200));
        assert_eq!(walk.ended.map(|summary| summary.entries), Ok(7));
    }

    #[test]
    fn the_progress_rises_from_the_first_tick() {
        let walk = paced(Play::taking(Duration::from_millis(800)));

        let (first_at, first) = walk.sent[0];
        assert!(first_at <= Duration::from_millis(100), "the first progress went out at {first_at:?}");
        assert!(first.bytes.bytes() > 0 && first.bytes.bytes() < TARGET / 20, "the first progress is {first:?}");
        assert!(walk.sent.len() >= 55, "only {} progresses went out", walk.sent.len());
        for pair in walk.sent.windows(2) {
            let ((before_at, before), (after_at, after)) = (pair[0], pair[1]);
            assert!(after.bytes > before.bytes, "{before:?} then {after:?}");
            assert!(
                after_at.checked_sub(before_at).is_some_and(|gap| !gap.is_zero() && gap <= Duration::from_millis(60))
            );
        }
    }

    #[test]
    fn the_progress_reaches_the_target_with_the_walks_entries() {
        let walk = paced(Play::taking(Duration::from_millis(800)));

        assert_eq!(walk.last(), Some(progress(1_000, TARGET)));
    }

    #[test]
    fn without_a_target_the_bytes_go_toward_the_walks_own() {
        let walk = paced(Play { target: None, ..Play::taking(Duration::from_millis(800)) });

        assert_eq!(walk.last(), Some(progress(1_000, 4_000_000)));
        assert!(walk.sent.iter().all(|(_, sent)| sent.bytes <= ByteSize::new(4_000_000)));
    }
}
