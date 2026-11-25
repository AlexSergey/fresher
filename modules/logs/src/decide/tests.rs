//! The decision row by row on plain values, negatives first.

use fresher_core::{EntryFacts, EntryKind, Timestamp};

use crate::decide::{DAY, Item, Reason, Scan, Verdict, decide};

const NOW: u64 = 1_767_225_600;
const DAYS: u32 = 7;
/// The setting's days in seconds.
const N: u64 = DAYS as u64 * DAY;

#[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
fn ago(secs: u64) -> Option<Timestamp> {
    Some(Timestamp::from_secs(NOW - secs))
}

/// An old log of the user's, modified and changed 30 days ago.
fn file() -> Item {
    Item {
        kind: EntryKind::File,
        facts: EntryFacts::REMOVABLE,
        modified: ago(30 * DAY),
        changed: ago(30 * DAY),
        state_or_lock: false,
    }
}

/// `file()`, but modified and changed `secs` ago.
fn aged(secs: u64) -> Item {
    Item { modified: ago(secs), changed: ago(secs), ..file() }
}

fn scan(days: u32) -> Scan {
    Scan { now: Timestamp::from_secs(NOW), days }
}

/// Decides `item` now, with the default days.
fn verdict(item: &Item) -> Verdict {
    decide(item, &scan(DAYS))
}

fn left_out(reason: Reason) -> Verdict {
    Verdict::LeftOut(reason)
}

fn facts(change: impl FnOnce(&mut EntryFacts)) -> EntryFacts {
    let mut facts = EntryFacts::REMOVABLE;
    change(&mut facts);
    facts
}

mod negative {
    use fresher_core::{EntryKind, Timestamp};

    use super::{DAY, N, NOW, aged, facts, file, left_out, scan, verdict};
    use crate::decide::{Item, Reason, Verdict, decide};

    #[test]
    fn a_folder_is_not_claimed() {
        assert_eq!(verdict(&Item { kind: EntryKind::Folder, ..file() }), Verdict::NotClaimed);
    }

    #[test]
    fn a_link_is_not_claimed() {
        assert_eq!(verdict(&Item { kind: EntryKind::Symlink, ..file() }), Verdict::NotClaimed);
    }

    #[test]
    fn a_state_or_lock_file_is_not_claimed() {
        assert_eq!(verdict(&Item { state_or_lock: true, ..file() }), Verdict::NotClaimed);
    }

    #[test]
    fn a_socket_pipe_or_device_is_special() {
        assert_eq!(verdict(&Item { kind: EntryKind::Other, ..file() }), left_out(Reason::Special));
    }

    #[test]
    fn a_file_of_the_system_or_another_user_is_not_yours() {
        for facts in [
            facts(|facts| facts.owned_by_user = false),
            facts(|facts| facts.system_immutable = true),
            facts(|facts| facts.restricted = true),
        ] {
            assert_eq!(verdict(&Item { facts, ..file() }), left_out(Reason::NotYours), "{facts:?}");
        }
    }

    #[test]
    fn a_file_the_user_locked_is_locked() {
        let item = Item { facts: facts(|facts| facts.user_immutable = true), ..file() };

        assert_eq!(verdict(&item), left_out(Reason::Locked));
    }

    #[test]
    fn exactly_the_days_old_is_recent() {
        assert_eq!(verdict(&aged(N)), left_out(Reason::Recent));
    }

    #[test]
    fn a_future_time_is_recent() {
        let future = Some(Timestamp::from_secs(NOW + 2 * DAY));
        for item in [Item { modified: future, ..file() }, Item { changed: future, ..file() }] {
            assert_eq!(verdict(&item), left_out(Reason::Recent), "{item:?}");
        }
    }

    #[test]
    fn an_unknown_time_is_recent() {
        for item in [
            Item { modified: None, ..file() },
            Item { changed: None, ..file() },
            Item { modified: None, changed: None, ..file() },
        ] {
            assert_eq!(verdict(&item), left_out(Reason::Recent), "{item:?}");
        }
    }

    #[test]
    fn an_old_modification_with_a_fresh_status_change_is_recent() {
        let rotated = Item { modified: super::ago(2000 * DAY), changed: super::ago(60), ..file() };

        assert_eq!(verdict(&rotated), left_out(Reason::Recent));
    }

    #[test]
    fn a_higher_setting_keeps_what_the_default_offers() {
        let item = aged(10 * DAY);

        assert!(matches!(verdict(&item), Verdict::Offered { .. }));
        assert_eq!(decide(&item, &scan(14)), left_out(Reason::Recent));
    }

    #[test]
    fn a_state_or_lock_name_comes_before_special() {
        assert_eq!(verdict(&Item { kind: EntryKind::Other, state_or_lock: true, ..file() }), Verdict::NotClaimed);
    }

    #[test]
    fn special_comes_before_not_yours() {
        let socket = Item { kind: EntryKind::Other, facts: facts(|facts| facts.owned_by_user = false), ..file() };

        assert_eq!(verdict(&socket), left_out(Reason::Special));
    }

    #[test]
    fn not_yours_comes_before_locked() {
        let item = Item {
            facts: facts(|facts| {
                facts.restricted = true;
                facts.user_immutable = true;
            }),
            ..file()
        };

        assert_eq!(verdict(&item), left_out(Reason::NotYours));
    }

    #[test]
    fn every_other_row_comes_before_age() {
        let fresh = |item: Item| Item { modified: super::ago(60), changed: super::ago(60), ..item };
        for (item, expected) in [
            (Item { kind: EntryKind::Folder, ..file() }, Verdict::NotClaimed),
            (Item { kind: EntryKind::Symlink, ..file() }, Verdict::NotClaimed),
            (Item { state_or_lock: true, ..file() }, Verdict::NotClaimed),
            (Item { kind: EntryKind::Other, ..file() }, left_out(Reason::Special)),
            (Item { facts: facts(|facts| facts.owned_by_user = false), ..file() }, left_out(Reason::NotYours)),
            (Item { facts: facts(|facts| facts.user_immutable = true), ..file() }, left_out(Reason::Locked)),
        ] {
            assert_eq!(verdict(&fresh(item)), expected, "{item:?}");
        }
    }
}

mod positive {
    use super::{DAY, N, aged, file, verdict};
    use crate::decide::{Reason, Verdict};

    #[test]
    fn the_days_and_one_second_is_offered() {
        assert_eq!(verdict(&aged(N + 1)), Verdict::Offered { age_days: 7 });
    }

    #[test]
    fn an_old_file_is_offered_with_its_whole_days() {
        assert_eq!(verdict(&file()), Verdict::Offered { age_days: 30 });
        assert_eq!(verdict(&aged(30 * DAY + DAY - 1)), Verdict::Offered { age_days: 30 });
    }

    #[test]
    fn every_reason_has_its_key_in_the_declared_order() {
        let keys: Vec<&str> = Reason::ALL.into_iter().map(Reason::key).collect();

        assert_eq!(
            keys,
            ["logs.left-out.special", "logs.left-out.not-yours", "logs.left-out.locked", "logs.left-out.recent"]
        );
    }
}

mod property {
    use fresher_core::{EntryFacts, EntryKind, Timestamp};
    use proptest::prelude::*;

    use crate::decide::{DAY, Item, Scan, Verdict, decide};

    fn time() -> impl Strategy<Value = Option<Timestamp>> {
        proptest::option::weighted(0.9, (0..4_000_000_000_u64).prop_map(Timestamp::from_secs))
    }

    proptest! {
        #[test]
        fn an_offered_file_is_always_older_than_the_days(
            modified in time(), changed in time(), now in 0..4_000_000_000_u64, days in 7..=90_u32,
        ) {
            let item = Item { kind: EntryKind::File, facts: EntryFacts::REMOVABLE, modified, changed, state_or_lock: false };
            let scan = Scan { now: Timestamp::from_secs(now), days };

            if let Verdict::Offered { age_days } = decide(&item, &scan) {
                let (modified, changed) = (modified.unwrap().secs(), changed.unwrap().secs());
                let age = now - modified.max(changed);
                prop_assert!(age > u64::from(days) * DAY);
                prop_assert_eq!(age_days, age / DAY);
                prop_assert!(age_days >= u64::from(days));
            }
        }
    }
}
