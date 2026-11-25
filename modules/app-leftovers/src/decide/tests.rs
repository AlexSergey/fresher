//! The decision on plain values: each row's keep before its offer, then the precedence of the rows, then properties.

use std::cell::Cell;

use fresher_core::{BundleId, EntryFacts, Inside, InstalledApp, RunningSet, Sizes, Timestamp};
use fresher_testing::{installed_app, running_app};

use crate::decide::{Facts, IdVerdict, NoSystem, Search, decide_id};
use crate::exclude::Exclude;
use crate::gathered::Leftover;
use crate::places::{Form, Places};

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;
const GONE: &str = "com.example.Gone";

/// Rows of the shipped table, by index.
const CACHES: usize = 0;
const SAVED_STATE: usize = 2;
const PREFERENCES: usize = 3;
const SUPPORT: usize = 8;

fn id(id: &str) -> BundleId {
    BundleId::new(id).unwrap()
}

fn days_ago(days: u64) -> Option<Timestamp> {
    NOW.checked_sub(days * DAY).map(Timestamp::from_secs)
}

/// A folder of `row` named after `of`, read in full, whose newest change inside is `newest`.
fn folder_at(row: usize, of: &str, newest: Option<Timestamp>) -> Leftover {
    Leftover {
        row,
        id: id(of),
        path: format!("/home/Library/{row}/{of}").into(),
        kind: Form::Folder,
        sizes: Sizes::ZERO,
        modified: newest,
        changed: newest,
        inside: Some(Inside { newest_change: newest, ..Inside::EMPTY }),
        facts: EntryFacts::REMOVABLE,
    }
}

/// A folder of `row` named after `of`, unchanged for 40 days.
fn folder(row: usize, of: &str) -> Leftover {
    folder_at(row, of, days_ago(40))
}

/// A file of `row` named after `of`, unchanged for 40 days.
fn file(row: usize, of: &str) -> Leftover {
    Leftover { kind: Form::File, inside: None, ..folder(row, of) }
}

/// The leftovers of an app that is gone: its saved state (the proof), caches and settings.
fn gone() -> Vec<Leftover> {
    vec![folder(SAVED_STATE, GONE), folder(CACHES, GONE), file(PREFERENCES, GONE), folder(SUPPORT, GONE)]
}

/// What the system says, and what Spotlight answers; by default nothing installed or running, every list read in
/// full, every place seen, 30 days, and Spotlight finds nothing.
struct World {
    installed: Result<Vec<InstalledApp>, NoSystem>,
    complete: bool,
    running: Result<RunningSet, NoSystem>,
    places_complete: bool,
    days: u32,
    search: Search,
}

impl Default for World {
    fn default() -> Self {
        Self {
            installed: Ok(Vec::new()),
            complete: true,
            running: Ok(RunningSet::default()),
            places_complete: true,
            days: 30,
            search: Search::NotFound,
        }
    }
}

impl World {
    fn installed(ids: &[&str]) -> Self {
        Self { installed: Ok(ids.iter().map(|&bundle| app(bundle, &[])).collect()), ..Self::default() }
    }

    fn running(ids: &[&str]) -> Self {
        Self { running: Ok(RunningSet::new(vec![running_app("App", ids).unwrap()])), ..Self::default() }
    }

    /// The verdict on `of`'s `leftovers`, and how many times Spotlight was asked.
    fn decide(&self, of: &str, leftovers: &[Leftover]) -> (IdVerdict, usize) {
        let places = Places::shipped().unwrap();
        let exclude = Exclude::shipped().unwrap();
        let facts = Facts {
            installed: self.installed.as_deref().map_err(|&error| error),
            complete: self.complete,
            running: self.running.as_ref().map_err(|&error| error),
            places_complete: self.places_complete,
            now: Timestamp::from_secs(NOW),
            days: self.days,
        };
        let asked = Cell::new(0);
        let mut search = |_: &BundleId| {
            asked.set(asked.get() + 1);
            self.search
        };
        let verdict = decide_id(&id(of), leftovers, places.rows(), &exclude, &facts, &mut search);
        (verdict, asked.get())
    }

    /// The verdict on the leftovers of the app that is gone.
    fn verdict(&self) -> IdVerdict {
        self.decide(GONE, &gone()).0
    }
}

/// An installed app with the id `bundle` and the helpers `helpers`.
fn app(bundle: &str, helpers: &[&str]) -> InstalledApp {
    InstalledApp {
        helpers: helpers.iter().map(|&helper| id(helper)).collect(),
        ..installed_app(bundle, "App", "1").unwrap()
    }
}

/// Up to six leftovers of the app that is gone, of any row and kind, unchanged for 0 to 99 days.
fn leftovers() -> impl proptest::strategy::Strategy<Value = Vec<Leftover>> {
    use proptest::prelude::*;

    prop::collection::vec((0usize..10, any::<bool>(), 0u64..100), 1..7).prop_map(|items| {
        items
            .into_iter()
            .map(|(row, is_file, days)| {
                let leftover = folder_at(row, GONE, Some(Timestamp::from_secs(NOW - days * DAY)));
                if is_file { Leftover { kind: Form::File, inside: None, ..leftover } } else { leftover }
            })
            .collect()
    })
}

mod negative {
    use fresher_core::{EntryFacts, Inside, Timestamp};

    use super::{
        CACHES, GONE, NOW, PREFERENCES, SAVED_STATE, SUPPORT, World, app, days_ago, file, folder, folder_at, gone,
    };
    use crate::decide::{IdVerdict, ItemVerdict, NoSystem, Reason, Search, decide_item};
    use crate::gathered::Leftover;

    #[test]
    fn an_excluded_id_is_not_claimed() {
        let apple = "com.apple.Safari";

        let (verdict, _) = World::default().decide(apple, &[folder(SAVED_STATE, apple), folder(CACHES, apple)]);

        assert_eq!(verdict, IdVerdict::NotClaimed);
    }

    #[test]
    fn an_id_with_only_caches_and_preferences_is_not_claimed() {
        let (verdict, _) = World::default().decide(GONE, &[folder(CACHES, GONE), file(PREFERENCES, GONE)]);

        assert_eq!(verdict, IdVerdict::NotClaimed);
    }

    #[test]
    fn an_id_whose_proof_place_holds_nothing_of_it_is_not_claimed_whatever_else_holds() {
        let (verdict, asked) = World::default().decide(GONE, &[folder(CACHES, GONE), folder(SUPPORT, GONE)]);

        assert_eq!((verdict, asked), (IdVerdict::NotClaimed, 0));
    }

    #[test]
    fn an_id_with_no_proof_is_not_claimed_whether_every_place_was_seen_or_not() {
        for places_complete in [true, false] {
            let world = World { places_complete, ..World::default() };

            let decided = world.decide(GONE, &[folder(CACHES, GONE), file(PREFERENCES, GONE), folder(SUPPORT, GONE)]);

            assert_eq!(decided, (IdVerdict::NotClaimed, 0), "places complete: {places_complete}");
        }
    }

    #[test]
    fn a_saved_state_that_is_a_file_is_no_proof() {
        let (verdict, _) = World::default().decide(GONE, &[file(SAVED_STATE, GONE), folder(CACHES, GONE)]);

        assert_eq!(verdict, IdVerdict::NotClaimed);
    }

    #[test]
    fn a_version_like_name_without_a_proof_is_not_claimed() {
        for name in ["IntelliJIdea2019.3", "PyCharm2019.2", "Python3.11", "1.2.3"] {
            let leftovers = [folder(CACHES, name), folder(SUPPORT, name), file(PREFERENCES, name)];

            assert_eq!(World::default().decide(name, &leftovers).0, IdVerdict::NotClaimed, "{name}");
        }
    }

    #[test]
    fn a_leftover_of_a_row_that_does_not_exist_is_no_proof() {
        let (verdict, _) = World::default().decide(GONE, &[folder(99, GONE)]);

        assert_eq!(verdict, IdVerdict::NotClaimed);
    }

    #[test]
    fn the_installed_apps_that_cannot_be_listed_keep_everything() {
        let world = World { installed: Err(NoSystem), ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::NoSystem));
    }

    #[test]
    fn an_installed_app_with_the_id_a_helper_or_a_related_id_is_not_claimed() {
        let installed = [
            World::installed(&[GONE]),
            World { installed: Ok(vec![app("com.example.Suite", &[GONE])]), ..World::default() },
            World::installed(&["com.example.Gone.Updater"]),
            World::installed(&["com.example"]),
            World::installed(&["COM.EXAMPLE.GONE"]),
        ];

        for world in installed {
            assert_eq!(world.verdict(), IdVerdict::NotClaimed);
        }
    }

    #[test]
    fn an_id_under_an_installed_app_is_not_claimed() {
        let world = World::installed(&["com.example.App"]);
        for under in ["com.example.App.Updater", "com.example"] {
            let leftovers = [folder(SAVED_STATE, under), folder(CACHES, under)];

            assert_eq!(world.decide(under, &leftovers).0, IdVerdict::NotClaimed, "{under}");
        }
    }

    #[test]
    fn a_partial_list_of_apps_keeps_everything() {
        let world = World { complete: false, ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::AppsUnread));
    }

    #[test]
    fn the_running_apps_that_cannot_be_listed_keep_everything() {
        let world = World { running: Err(NoSystem), ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::NoSystem));
    }

    #[test]
    fn a_running_app_with_a_related_id_keeps_everything() {
        for running in [GONE, "com.example.Gone.Helper", "com.example"] {
            assert_eq!(World::running(&[running]).verdict(), IdVerdict::LeftOut(Reason::Running), "{running}");
        }
    }

    #[test]
    fn a_place_not_seen_in_full_keeps_everything() {
        let world = World { places_complete: false, ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::ScanAgain));
    }

    #[test]
    fn a_folder_read_in_part_or_not_read_keeps_everything() {
        let part = Leftover { inside: Some(Inside { complete: false, ..Inside::EMPTY }), ..folder(CACHES, GONE) };
        let unread = Leftover { inside: None, ..folder(SUPPORT, GONE) };

        for leftover in [part, unread] {
            let leftovers = [folder(SAVED_STATE, GONE), leftover];

            assert_eq!(World::default().decide(GONE, &leftovers).0, IdVerdict::LeftOut(Reason::ScanAgain));
        }
    }

    #[test]
    fn a_leftover_changed_five_days_ago_keeps_everything() {
        let mut leftovers = gone();
        leftovers.push(folder_at(CACHES, GONE, days_ago(5)));

        assert_eq!(World::default().decide(GONE, &leftovers).0, IdVerdict::LeftOut(Reason::UsedRecently));
    }

    #[test]
    fn a_time_unknown_or_in_the_future_is_recent() {
        let future = Some(Timestamp::from_secs(NOW + 1));
        let file_without_change = Leftover { changed: None, ..file(PREFERENCES, GONE) };
        for leftover in [folder_at(CACHES, GONE, None), folder_at(CACHES, GONE, future), file_without_change] {
            let mut leftovers = gone();
            leftovers.push(leftover);

            assert_eq!(World::default().decide(GONE, &leftovers).0, IdVerdict::LeftOut(Reason::UsedRecently));
        }
    }

    #[test]
    fn a_longer_setting_keeps_what_a_shorter_one_offers() {
        let world = World { days: 41, ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::UsedRecently));
    }

    #[test]
    fn spotlight_that_cannot_tell_keeps_everything() {
        let world = World { search: Search::Unread, ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::SpotlightUnread));
    }

    #[test]
    fn an_app_spotlight_finds_keeps_everything() {
        let world = World { search: Search::Found, ..World::default() };

        assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::InstalledElsewhere));
    }

    mod precedence {
        use super::super::{CACHES, GONE, SAVED_STATE, World, days_ago, folder, folder_at, gone};
        use crate::decide::{IdVerdict, NoSystem, Reason, Search};

        #[test]
        fn excluded_holds_over_no_proof() {
            let apple = "com.apple.Notes";

            assert_eq!(World::default().decide(apple, &[folder(CACHES, apple)]).0, IdVerdict::NotClaimed);
        }

        #[test]
        fn no_proof_holds_over_the_apps_failing() {
            let world = World { installed: Err(NoSystem), running: Err(NoSystem), ..World::default() };

            assert_eq!(world.decide(GONE, &[folder(CACHES, GONE)]).0, IdVerdict::NotClaimed);
        }

        #[test]
        fn installed_holds_over_a_partial_list() {
            let world = World { complete: false, ..World::installed(&[GONE]) };

            assert_eq!(world.verdict(), IdVerdict::NotClaimed);
        }

        #[test]
        fn a_partial_list_holds_over_running() {
            let world = World { complete: false, ..World::running(&[GONE]) };

            assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::AppsUnread));
        }

        #[test]
        fn running_holds_over_the_scan() {
            let world = World { places_complete: false, ..World::running(&[GONE]) };

            assert_eq!(world.verdict(), IdVerdict::LeftOut(Reason::Running));
        }

        #[test]
        fn the_scan_holds_over_the_age() {
            let world = World { places_complete: false, ..World::default() };
            let leftovers = [folder(SAVED_STATE, GONE), folder_at(CACHES, GONE, days_ago(1))];

            assert_eq!(world.decide(GONE, &leftovers).0, IdVerdict::LeftOut(Reason::ScanAgain));
        }

        #[test]
        fn the_age_holds_over_spotlight_which_is_not_asked() {
            let world = World { search: Search::Found, ..World::default() };
            let mut leftovers = gone();
            leftovers.push(folder_at(CACHES, GONE, days_ago(1)));

            assert_eq!(world.decide(GONE, &leftovers), (IdVerdict::LeftOut(Reason::UsedRecently), 0));
        }

        #[test]
        fn spotlight_is_not_asked_for_any_earlier_keep() {
            let keeps = [
                World { installed: Err(NoSystem), ..World::default() },
                World::installed(&[GONE]),
                World { complete: false, ..World::default() },
                World { running: Err(NoSystem), ..World::default() },
                World::running(&[GONE]),
                World { places_complete: false, ..World::default() },
                World { days: 365, ..World::default() },
            ];

            for world in keeps {
                assert_eq!(world.decide(GONE, &gone()).1, 0);
            }
        }
    }

    #[test]
    fn an_item_with_a_socket_inside_is_special() {
        let socket = Leftover { inside: Some(Inside { special: true, ..Inside::EMPTY }), ..folder(CACHES, GONE) };

        assert_eq!(decide_item(&socket), ItemVerdict::LeftOut(Reason::Special));
    }

    #[test]
    fn an_item_not_the_users_flagged_or_with_a_foreign_entry_inside_is_not_yours() {
        let facts = [
            EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE },
            EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE },
            EntryFacts { restricted: true, ..EntryFacts::REMOVABLE },
            EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE },
        ];
        for facts in facts {
            assert_eq!(
                decide_item(&Leftover { facts, ..file(PREFERENCES, GONE) }),
                ItemVerdict::LeftOut(Reason::NotYours)
            );
        }
        let foreign = Leftover { inside: Some(Inside { foreign: true, ..Inside::EMPTY }), ..folder(SUPPORT, GONE) };
        assert_eq!(decide_item(&foreign), ItemVerdict::LeftOut(Reason::NotYours));
    }

    #[test]
    fn an_item_folder_not_read_in_full_is_kept() {
        let unread = Leftover { inside: None, ..folder(CACHES, GONE) };

        assert_eq!(decide_item(&unread), ItemVerdict::LeftOut(Reason::ScanAgain));
    }

    mod properties {
        use fresher_core::Timestamp;
        use proptest::prelude::*;

        use super::super::{DAY, GONE, NOW, World, app, folder_at, leftovers};
        use crate::decide::IdVerdict;

        /// An id related to the app that is gone.
        fn related() -> impl Strategy<Value = &'static str> {
            prop::sample::select(vec![GONE, "COM.EXAMPLE.GONE", "com.example.Gone.Helper", "com.example"])
        }

        proptest! {
            #[test]
            fn an_installed_related_app_never_lets_an_id_be_offered(leftovers in leftovers(), installed in related()) {
                let world = World { installed: Ok(vec![app("com.example.Suite", &[installed])]), ..World::default() };

                prop_assert_ne!(world.decide(GONE, &leftovers).0, IdVerdict::Offer);
                prop_assert_ne!(World::installed(&[installed]).decide(GONE, &leftovers).0, IdVerdict::Offer);
            }

            #[test]
            fn a_running_related_app_never_lets_an_id_be_offered(leftovers in leftovers(), running in related()) {
                prop_assert_ne!(World::running(&[running]).decide(GONE, &leftovers).0, IdVerdict::Offer);
            }

            #[test]
            fn a_leftover_changed_within_the_days_never_lets_an_id_be_offered(
                mut leftovers in leftovers(),
                row in 0usize..10,
                days in 0u64..30,
            ) {
                leftovers.push(folder_at(row, GONE, Some(Timestamp::from_secs(NOW - days * DAY))));

                prop_assert_ne!(World::default().decide(GONE, &leftovers).0, IdVerdict::Offer);
            }
        }
    }
}

mod positive {
    use fresher_core::Timestamp;

    use super::{CACHES, DAY, GONE, NOW, SAVED_STATE, World, folder_at, gone};
    use crate::decide::{IdVerdict, ItemVerdict, decide_item};

    #[test]
    fn leftovers_unchanged_for_the_days_exactly_are_offered_after_spotlight_is_asked_once() {
        let exactly = Some(Timestamp::from_secs(NOW - 30 * DAY));
        let leftovers = [folder_at(SAVED_STATE, GONE, exactly), folder_at(CACHES, GONE, exactly)];

        assert_eq!(World::default().decide(GONE, &leftovers), (IdVerdict::Offer, 1));
    }

    #[test]
    fn the_leftovers_of_an_app_that_is_gone_are_offered() {
        assert_eq!(World::default().verdict(), IdVerdict::Offer);
    }

    #[test]
    fn an_app_that_only_shares_a_start_with_the_id_does_not_keep_it() {
        let world = World::installed(&["com.example.Gonex"]);

        assert_eq!(world.verdict(), IdVerdict::Offer);
        assert_eq!(World::running(&["com.example.Gonex"]).verdict(), IdVerdict::Offer);
    }

    #[test]
    fn a_leftover_of_the_users_with_nothing_special_is_offered() {
        for leftover in gone() {
            assert_eq!(decide_item(&leftover), ItemVerdict::Offered);
        }
    }

    mod properties {
        use proptest::prelude::*;

        use super::super::{GONE, World, leftovers};

        proptest! {
            #[test]
            fn the_verdict_is_the_same_whatever_the_order_of_the_leftovers(
                (leftovers, shuffled) in leftovers().prop_flat_map(|leftovers| (Just(leftovers.clone()), Just(leftovers).prop_shuffle())),
            ) {
                let world = World::default();

                prop_assert_eq!(world.decide(GONE, &shuffled), world.decide(GONE, &leftovers));
            }
        }
    }
}
