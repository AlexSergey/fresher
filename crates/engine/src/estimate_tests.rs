use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{ByteSize, Completeness, Event, NodeKind, Sizes, StateId, Timestamp, VolumeFacts, VolumeId};
use fresher_testing::{FakeVolumes, FixedDiskState};

use crate::testing::ScriptedWalk;

/// 300 logical bytes on 400 allocated: the estimate is in the measure of the walk's progress, the logical one.
fn sizes() -> Sizes {
    Sizes {
        logical: ByteSize::new(300),
        allocated: ByteSize::new(400),
        reclaimable: ByteSize::new(400),
        completeness: Completeness::Complete,
    }
}

/// A state whose only root is `root`, of `sizes()`.
fn state_at(root: &str) -> Arc<dyn fresher_core::DiskState> {
    let nodes = [(PathBuf::from(root), NodeKind::Folder, sizes())];
    Arc::new(FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes))
}

/// The volume numbered `id`.
fn facts(id: u128) -> VolumeFacts {
    VolumeFacts { id: VolumeId::new(id), device: 1, case_sensitive: false, used: ByteSize::new(1) }
}

/// The start-up disk's space: 700 bytes used.
fn startup_space() -> fresher_core::DiskSpace {
    FakeVolumes::disk("Macintosh HD", ByteSize::new(1000), ByteSize::new(700), ByteSize::new(300))
}

/// The start-up disk holding the fixture home `/home`.
fn startup() -> FakeVolumes {
    FakeVolumes::default().with("/", facts(1)).with_space("/", startup_space())
}

/// A walk of the fixture home with two progress steps that commits `states` in turn.
fn walk(states: Vec<Arc<dyn fresher_core::DiskState>>) -> ScriptedWalk {
    ScriptedWalk::new().progress(10, 100).progress(20, 200).committing(states)
}

/// The estimate of every `ScanProgress` in `events`, in order.
fn estimates(events: &[Event]) -> Vec<Option<u64>> {
    let progress = events.iter().filter_map(|event| match event {
        Event::ScanProgress { expected, .. } => Some(expected.map(ByteSize::bytes)),
        _ => None,
    });
    progress.collect()
}

/// `Work`'s space: 250 bytes used.
fn work_space() -> fresher_core::DiskSpace {
    FakeVolumes::disk("Work", ByteSize::new(600), ByteSize::new(250), ByteSize::new(350))
}

mod negative {
    use std::path::PathBuf;

    use fresher_core::ByteSize;
    use fresher_testing::FakeVolumes;

    use super::super::expected;
    use super::{estimates, startup, state_at, walk};
    use crate::storage::roots::RootVolumes;
    use crate::testing::{Fixture, scan};

    #[test]
    fn a_first_scan_without_volumes_has_no_estimate() {
        let engine = Fixture::new(walk(Vec::new())).engine();

        let events = scan(&engine);

        assert_eq!(estimates(&events), [None, None]);
    }

    #[test]
    fn a_scan_of_other_roots_does_not_use_the_last_state() {
        let engine = Fixture::new(walk(vec![state_at("/Volumes/Work")])).volumes(startup()).engine();
        scan(&engine);

        let events = scan(&engine);

        assert_eq!(estimates(&events), [Some(700), Some(700)]);
    }

    #[test]
    fn a_volume_without_its_facts_leaves_the_estimate_unknown() {
        let roots = [PathBuf::from("/home"), PathBuf::from("/Volumes/Work")];
        let port = FakeVolumes::default()
            .with("/", super::facts(1))
            .with_space("/home", super::startup_space())
            .with("/Volumes/Work", super::facts(2));
        let volumes = RootVolumes::read(Some(&port), None, &roots);

        assert_eq!(expected(None, &roots, &volumes), None);
    }

    #[test]
    fn no_roots_have_no_estimate() {
        let state = state_at("/home");

        assert_eq!(expected(Some(&*state), &[], &RootVolumes::default()), None::<ByteSize>);
    }
}

mod positive {
    use std::path::PathBuf;

    use super::super::expected;
    use super::{estimates, startup, state_at, walk};
    use crate::storage::roots::RootVolumes;
    use crate::testing::{Fixture, scan};

    #[test]
    fn a_second_scan_estimates_by_the_last_state() {
        let engine = Fixture::new(walk(vec![state_at("/home")])).volumes(startup()).engine();
        scan(&engine);

        let events = scan(&engine);

        assert_eq!(estimates(&events), [Some(300), Some(300)]);
    }

    #[test]
    fn a_first_scan_estimates_by_the_volumes_used_space() {
        let engine = Fixture::new(walk(Vec::new())).volumes(startup()).engine();

        let events = scan(&engine);

        assert_eq!(estimates(&events), [Some(700), Some(700)]);
    }

    #[test]
    fn every_progress_of_a_scan_carries_the_same_estimate() {
        let engine = Fixture::new(walk(vec![state_at("/home")])).engine();
        scan(&engine);

        let events = scan(&engine);

        let estimates = estimates(&events);
        assert!(estimates.len() > 1);
        assert!(estimates.iter().all(|estimate| *estimate == Some(300)));
    }

    #[test]
    fn a_root_inside_another_is_counted_through_it() {
        let state = state_at("/home");
        let roots = [PathBuf::from("/home"), PathBuf::from("/home/Projects"), PathBuf::from("/home")];

        let estimate = expected(Some(&*state), &roots, &RootVolumes::default());

        assert_eq!(estimate.map(fresher_core::ByteSize::bytes), Some(300));
    }

    #[test]
    fn two_volumes_add_up_their_used_space() {
        let roots = [PathBuf::from("/home"), PathBuf::from("/Volumes/Work")];
        let volumes = startup().with("/Volumes/Work", super::facts(2)).with_space("/Volumes/Work", super::work_space());

        let estimate = expected(None, &roots, &RootVolumes::read(Some(&volumes), None, &roots));

        assert_eq!(estimate.map(fresher_core::ByteSize::bytes), Some(950));
    }
}
