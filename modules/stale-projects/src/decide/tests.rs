//! The decision tables, one test per row, keep before offer; then the properties.

use fresher_core::{Artifact, ByteSize, Completeness, LastChange, Sizes, Timestamp};

use crate::decide::Scan;

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;

/// A time `days` days before the scan.
fn ago(days: u64) -> Timestamp {
    Timestamp::from_secs(NOW - days * DAY)
}

/// A scan now, with the default 365 days, not restored.
fn scan() -> Scan {
    Scan { now: Timestamp::from_secs(NOW), days: 365, restored: false }
}

/// A folder of 100 bytes, read in full, last changed as `last_change` says.
fn folder(last_change: LastChange) -> Artifact {
    let size = ByteSize::new(100);
    Artifact { path: "/home/Code/app/target".into(), sizes: Sizes::complete(size, size), last_change }
}

fn partial(folder: Artifact) -> Artifact {
    Artifact { sizes: Sizes { completeness: Completeness::Partial, ..folder.sizes }, ..folder }
}

mod negative {
    use fresher_core::{Completeness, LastChange, Sizes, Timestamp};

    use super::{NOW, ago, folder, partial, scan};
    use crate::decide::{Reason, Scan, Staleness, Verdict, artifact, cache, group_activity, staleness};

    #[test]
    fn a_restored_scan_keeps_everything_to_scan_again() {
        let restored = Scan { restored: true, ..scan() };
        let old = LastChange::Known(ago(800));

        let group = staleness(old, &restored);

        assert_eq!(group, Staleness::Restored);
        assert_eq!(artifact(&folder(old), true, group, &restored), Verdict::LeftOut(Reason::ScanAgain));
        assert_eq!(cache(&folder(old), true, &restored), Verdict::LeftOut(Reason::ScanAgain));
    }

    #[test]
    fn a_project_changed_within_the_days_is_active() {
        for activity in [LastChange::Known(ago(364)), LastChange::AtLeast(ago(364)), LastChange::Known(ago(0))] {
            let group = staleness(activity, &scan());

            assert_eq!(group, Staleness::Active, "{activity:?}");
            assert_eq!(artifact(&folder(activity), true, group, &scan()), Verdict::LeftOut(Reason::Recent));
        }
    }

    #[test]
    fn a_project_changed_in_the_future_is_active() {
        let future = Timestamp::from_secs(NOW + 10 * super::DAY);

        for activity in [LastChange::Known(future), LastChange::AtLeast(future)] {
            assert_eq!(staleness(activity, &scan()), Staleness::Active, "{activity:?}");
        }
    }

    #[test]
    fn an_unread_project_older_than_the_days_keeps_its_artifacts() {
        let activity = LastChange::AtLeast(ago(800));

        let group = staleness(activity, &scan());

        assert_eq!(group, Staleness::Unread);
        assert_eq!(artifact(&folder(activity), true, group, &scan()), Verdict::LeftOut(Reason::Unread));
    }

    #[test]
    fn an_unknown_activity_keeps_its_artifacts() {
        let group = staleness(LastChange::Unknown, &scan());

        assert_eq!(group, Staleness::Unread);
        assert_eq!(artifact(&folder(LastChange::Unknown), true, group, &scan()), Verdict::LeftOut(Reason::Unread));
    }

    #[test]
    fn an_artifact_read_in_part_is_kept_unread() {
        let group = Staleness::Stale { since: ago(800) };
        let old = folder(LastChange::Known(ago(800)));
        let unknown = Sizes { completeness: Completeness::Unknown, ..old.sizes };

        assert_eq!(artifact(&partial(old.clone()), true, group, &scan()), Verdict::LeftOut(Reason::Unread));
        let unknown = fresher_core::Artifact { sizes: unknown, ..old };
        assert_eq!(artifact(&unknown, true, group, &scan()), Verdict::LeftOut(Reason::Unread));
    }

    #[test]
    fn an_artifact_changed_within_the_days_is_kept_even_in_a_stale_project() {
        let group = Staleness::Stale { since: ago(800) };
        let future = Timestamp::from_secs(NOW + super::DAY);

        for time in [ago(0), ago(7), ago(364), future] {
            for last_change in [LastChange::Known(time), LastChange::AtLeast(time)] {
                assert_eq!(
                    artifact(&folder(last_change), true, group, &scan()),
                    Verdict::LeftOut(Reason::Recent),
                    "{last_change:?}"
                );
            }
        }
    }

    #[test]
    fn an_artifact_of_a_stale_project_not_dated_in_full_is_kept_unread() {
        let group = Staleness::Stale { since: ago(800) };

        for last_change in [LastChange::Unknown, LastChange::AtLeast(ago(800))] {
            assert_eq!(
                artifact(&folder(last_change), true, group, &scan()),
                Verdict::LeftOut(Reason::Unread),
                "{last_change:?}"
            );
        }
    }

    #[test]
    fn an_artifact_the_module_did_not_match_is_not_claimed() {
        for group in [Staleness::Restored, Staleness::Active, Staleness::Unread, Staleness::Stale { since: ago(800) }] {
            assert_eq!(artifact(&folder(LastChange::Known(ago(800))), false, group, &scan()), Verdict::NotClaimed);
        }
    }

    #[test]
    fn a_cache_the_module_did_not_match_is_not_claimed() {
        let restored = Scan { restored: true, ..scan() };

        for scan in [scan(), restored] {
            assert_eq!(cache(&folder(LastChange::Known(ago(800))), false, &scan), Verdict::NotClaimed);
        }
    }

    #[test]
    fn a_cache_built_within_seven_days_is_kept() {
        let future = Timestamp::from_secs(NOW + super::DAY);

        for time in [ago(0), ago(6), future] {
            assert_eq!(cache(&folder(LastChange::Known(time)), true, &scan()), Verdict::LeftOut(Reason::CacheRecent));
        }
    }

    #[test]
    fn a_cache_read_in_part_is_kept_unread() {
        let old = partial(folder(LastChange::Known(ago(30))));

        assert_eq!(cache(&old, true, &scan()), Verdict::LeftOut(Reason::Unread));
    }

    #[test]
    fn an_unknown_or_old_at_least_cache_is_kept_unread() {
        for last_change in [LastChange::Unknown, LastChange::AtLeast(ago(7)), LastChange::AtLeast(ago(30))] {
            assert_eq!(cache(&folder(last_change), true, &scan()), Verdict::LeftOut(Reason::Unread), "{last_change:?}");
        }
    }

    #[test]
    fn an_at_least_cache_time_within_seven_days_is_kept() {
        let recent = folder(LastChange::AtLeast(ago(3)));

        assert_eq!(cache(&recent, true, &scan()), Verdict::LeftOut(Reason::CacheRecent));
    }

    #[test]
    fn a_member_that_cannot_be_dated_keeps_its_group_from_being_stale() {
        let unread = [
            group_activity([LastChange::Known(ago(800)), LastChange::Unknown]),
            group_activity([LastChange::Known(ago(800)), LastChange::AtLeast(ago(900))]),
            group_activity([]),
        ];

        for activity in unread {
            assert_eq!(staleness(activity, &scan()), Staleness::Unread, "{activity:?}");
        }
    }

    #[test]
    fn a_nested_member_changed_lately_keeps_its_group_active() {
        let members = [LastChange::Known(ago(800)), LastChange::Known(ago(3)), LastChange::Unknown];

        assert_eq!(staleness(group_activity(members), &scan()), Staleness::Active);
    }
}

mod positive {
    use fresher_core::{ByteSize, Completeness, LastChange, Sizes};

    use super::{ago, folder, scan};
    use crate::decide::{Staleness, Verdict, artifact, cache, group_activity, less, staleness};

    #[test]
    fn an_artifact_of_a_stale_project_is_offered() {
        let activity = LastChange::Known(ago(365));

        let group = staleness(activity, &scan());

        assert_eq!(group, Staleness::Stale { since: ago(365) });
        assert_eq!(artifact(&folder(LastChange::Known(ago(365))), true, group, &scan()), Verdict::Offered);
    }

    #[test]
    fn a_cache_unused_for_seven_days_is_offered_whatever_the_project_s_age() {
        for last_change in [LastChange::Known(ago(7)), LastChange::Known(ago(800))] {
            assert_eq!(cache(&folder(last_change), true, &scan()), Verdict::Offered, "{last_change:?}");
        }
    }

    #[test]
    fn less_never_goes_below_zero_and_keeps_the_completeness() {
        let sizes = Sizes {
            logical: ByteSize::new(100),
            allocated: ByteSize::new(80),
            reclaimable: ByteSize::new(60),
            completeness: Completeness::Partial,
        };
        let parts = [Sizes::new(ByteSize::new(30), ByteSize::new(50), ByteSize::new(70)), Sizes::ZERO];

        let left = less(sizes, &parts);

        assert_eq!(
            left,
            Sizes {
                logical: ByteSize::new(70),
                allocated: ByteSize::new(30),
                reclaimable: ByteSize::ZERO,
                completeness: Completeness::Partial,
            }
        );
        assert_eq!(less(sizes, &[]), sizes);
    }

    #[test]
    fn a_group_is_stale_since_its_newest_member() {
        let members = [LastChange::Known(ago(900)), LastChange::Known(ago(400)), LastChange::Known(ago(800))];

        assert_eq!(staleness(group_activity(members), &scan()), Staleness::Stale { since: ago(400) });
    }

    mod properties {
        use fresher_core::{LastChange, Timestamp};
        use proptest::prelude::*;

        use super::super::{DAY, NOW, ago, folder, scan};
        use crate::decide::{Scan, Verdict, artifact, cache, group_activity, staleness};

        fn activity() -> impl Strategy<Value = LastChange> {
            let time = (0..800 * DAY).prop_map(|ago| Timestamp::from_secs(NOW - ago));
            prop_oneof![
                Just(LastChange::Unknown),
                time.clone().prop_map(LastChange::Known),
                time.prop_map(LastChange::AtLeast),
            ]
        }

        /// The same kind of change, `later` seconds after.
        fn moved(activity: LastChange, later: u64) -> LastChange {
            let move_on = |time: Timestamp| Timestamp::from_secs(time.secs() + later);
            match activity {
                LastChange::Known(time) => LastChange::Known(move_on(time)),
                LastChange::AtLeast(time) => LastChange::AtLeast(move_on(time)),
                LastChange::Unknown => LastChange::Unknown,
            }
        }

        fn offered(verdict: Verdict) -> bool {
            verdict == Verdict::Offered
        }

        proptest! {
            #[test]
            fn a_later_activity_never_turns_a_keep_into_an_offer(
                first in activity(), second in activity(), later in 0..800 * DAY, days in 30..3650u32,
            ) {
                let scan = Scan { days, ..scan() };
                let artifact_of = |members: [LastChange; 2]| {
                    artifact(&folder(LastChange::Known(ago(800))), true, staleness(group_activity(members), &scan), &scan)
                };

                if !offered(artifact_of([first, second])) {
                    prop_assert!(!offered(artifact_of([moved(first, later), second])));
                    prop_assert!(!offered(artifact_of([first, moved(second, later)])));
                }
            }

            #[test]
            fn a_later_artifact_change_never_turns_a_keep_into_an_offer(
                last_change in activity(), later in 0..800 * DAY, days in 30..3650u32,
            ) {
                let scan = Scan { days, ..scan() };
                let group = staleness(LastChange::Known(ago(3650)), &scan);

                if !offered(artifact(&folder(last_change), true, group, &scan)) {
                    prop_assert!(!offered(artifact(&folder(moved(last_change, later)), true, group, &scan)));
                }
            }

            #[test]
            fn a_later_cache_change_never_turns_a_keep_into_an_offer(
                last_change in activity(), later in 0..800 * DAY,
            ) {
                if !offered(cache(&folder(last_change), true, &scan())) {
                    prop_assert!(!offered(cache(&folder(moved(last_change, later)), true, &scan())));
                }
            }

            #[test]
            fn a_restored_scan_never_offers(
                group in activity(), last_change in activity(), days in 30..3650u32,
            ) {
                let restored = Scan { days, restored: true, ..scan() };

                prop_assert!(!offered(artifact(&folder(last_change), true, staleness(group, &restored), &restored)));
                prop_assert!(!offered(cache(&folder(last_change), true, &restored)));
            }
        }
    }
}
