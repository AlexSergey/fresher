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

/// A walk on a fake clock: it reports `progress`, moves the clock by `takes`, and ends with `ends`.
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
        for progress in &self.progress {
            on_progress(*progress);
        }
        self.clock.advance(self.takes);
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

/// The walk's progress up to its last, 1000 entries of 4 MB.
fn rising_progress() -> Vec<WalkProgress> {
    vec![progress(10, 4_000), progress(500, 2_000_000), progress(1_000, 4_000_000)]
}

/// What a paced walk gave out: each progress with when it went out, how long it waited in all, and how it ended.
struct Paced {
    sent: Vec<(SystemTime, WalkProgress)>,
    waited: Duration,
    ended: Result<WalkSummary, WalkError>,
    elapsed: Duration,
}

/// Walks `inner` (taking `takes`, ending with `ends`) through a demo's pacing; `cancel_after` cancels the scan once
/// that many progresses went out.
fn paced(takes: Duration, ends: Result<(), WalkError>, cancel_after: Option<usize>) -> Paced {
    let clock = Arc::new(FakeClock::at(START));
    let inner = FakeWalk { clock: Arc::clone(&clock), progress: rising_progress(), takes, ends };
    let waited = Arc::new(Mutex::new(Duration::ZERO));
    let wait = {
        let (clock, waited) = (Arc::clone(&clock), Arc::clone(&waited));
        Arc::new(move |span: Duration| {
            clock.advance(span);
            *waited.lock().unwrap() += span;
        })
    };
    let walk = PacedWalk::new(Arc::new(inner), Arc::clone(&clock) as Arc<dyn Clock>, wait, DEMO_SCAN_TIME);
    let (cancel, interests) = (CancelToken::new(), InterestSet::new());
    let request = WalkRequest { roots: &[], interests: &interests, avoid: &[], places: &[], mode: WalkMode::Full };
    let mut sent = Vec::new();
    let ended = walk.walk(&request, &cancel, &mut |_| {}, &mut |progress| {
        sent.push((clock.now(), progress));
        if cancel_after == Some(sent.len()) {
            cancel.cancel();
        }
    });
    let elapsed = clock.now().duration_since(SystemTime::UNIX_EPOCH + Duration::from_secs(START)).unwrap();
    let waited = *waited.lock().unwrap();
    Paced { sent, waited, ended, elapsed }
}

mod negative {
    use std::time::Duration;

    use fresher_core::WalkError;

    use super::{paced, progress};
    use crate::DEMO_SCAN_TIME;

    #[test]
    fn a_walk_slower_than_the_minimum_is_not_held() {
        let walk = paced(Duration::from_secs(5), Ok(()), None);

        assert_eq!(walk.waited, Duration::ZERO);
        assert_eq!(walk.elapsed, Duration::from_secs(5));
        assert_eq!(walk.sent.iter().map(|(_, sent)| *sent).collect::<Vec<_>>(), [progress(1_000, 4_000_000)]);
        assert!(walk.ended.is_ok());
    }

    #[test]
    fn a_walk_that_fails_ends_at_once_with_its_error() {
        let walk = paced(Duration::from_millis(200), Err(WalkError::Cancelled), None);

        assert_eq!(walk.ended, Err(WalkError::Cancelled));
        assert_eq!(walk.waited, Duration::ZERO);
        assert_eq!(walk.sent, []);
    }

    #[test]
    fn a_scan_cancelled_while_paced_stops_cancelled() {
        let walk = paced(Duration::from_millis(200), Ok(()), Some(3));

        assert_eq!(walk.ended, Err(WalkError::Cancelled));
        assert_eq!(walk.sent.len(), 3);
        assert!(walk.elapsed < DEMO_SCAN_TIME);
    }
}

mod positive {
    use std::time::Duration;

    use super::{paced, progress};
    use crate::DEMO_SCAN_TIME;

    #[test]
    fn a_fast_walk_lasts_the_minimum_time() {
        let walk = paced(Duration::from_millis(800), Ok(()), None);

        assert_eq!(walk.elapsed, DEMO_SCAN_TIME);
        assert_eq!(walk.waited, Duration::from_millis(2200));
        assert_eq!(walk.ended.map(|summary| summary.entries), Ok(7));
    }

    #[test]
    fn its_progress_rises_evenly_to_the_walks_last() {
        let walk = paced(Duration::from_millis(800), Ok(()), None);

        assert!(walk.sent.len() >= 40, "only {} progresses went out", walk.sent.len());
        for pair in walk.sent.windows(2) {
            let ((before_at, before), (after_at, after)) = (pair[0], pair[1]);
            assert!(after.entries > before.entries && after.bytes > before.bytes, "{before:?} then {after:?}");
            assert!(after_at > before_at);
            assert!(after_at.duration_since(before_at).unwrap() <= Duration::from_millis(60));
        }
        assert_eq!(walk.sent.last().map(|(_, last)| *last), Some(progress(1_000, 4_000_000)));
        assert!(walk.sent[0].1.entries < 100, "the first progress is {:?}", walk.sent[0].1);
    }
}
