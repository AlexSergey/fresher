//! The decision row by row on plain values, negatives first.

use std::ffi::OsStr;

use fresher_core::{EntryFacts, EntryKind, Inside, RunningSet, Timestamp};
use fresher_testing::running_app;

use crate::decide::{DAY, Item, Reason, Scan, Verdict, decide};

const NOW: u64 = 1_767_225_600;
const DAYS: u32 = 7;
/// The setting's days in seconds.
const N: u64 = DAYS as u64 * DAY;

#[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
fn ago(secs: u64) -> Option<Timestamp> {
    Some(Timestamp::from_secs(NOW - secs))
}

/// An old file of the user's, `name`, changed and read 30 days ago.
fn file(name: &str) -> Item<'_> {
    Item {
        name: OsStr::new(name),
        kind: EntryKind::File,
        facts: EntryFacts::REMOVABLE,
        modified: ago(30 * DAY),
        changed: ago(30 * DAY),
        accessed: ago(30 * DAY),
        inside: None,
    }
}

/// An old folder of the user's, `name`, read in full, nothing inside special or foreign.
fn folder(name: &str) -> Item<'_> {
    let inside = Inside { newest_change: ago(30 * DAY), newest_file_access: ago(30 * DAY), ..Inside::EMPTY };
    Item { kind: EntryKind::Folder, accessed: None, inside: Some(inside), ..file(name) }
}

fn with_inside(item: Item<'_>, change: impl FnOnce(&mut Inside)) -> Item<'_> {
    let mut inside = item.inside.unwrap_or(Inside::EMPTY);
    change(&mut inside);
    Item { inside: Some(inside), ..item }
}

fn running() -> RunningSet {
    RunningSet::new(vec![running_app("Editor", &["com.example.Editor"]).unwrap()])
}

/// Decides `item` now, with the default days, the running set `running` and a fresh scan.
fn verdict(item: &Item<'_>, running: &RunningSet) -> Verdict {
    decide(item, &Scan { restored: false, now: Timestamp::from_secs(NOW), days: DAYS, running: Ok(running) })
}

fn kept(item: &Item<'_>) -> Verdict {
    verdict(item, &running())
}

fn left_out(reason: Reason) -> Verdict {
    Verdict::LeftOut(reason)
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, RunningSet, Timestamp};

    use super::{DAY, N, NOW, ago, file, folder, kept, left_out, running, with_inside};
    use crate::decide::{Item, NoSystem, Reason, Scan, Verdict, decide};

    #[test]
    fn a_restored_scan_keeps_everything_even_what_would_be_offered() {
        let set = running();
        for item in [file("old.tmp"), folder("build"), file("app.lock"), file("com.apple.x")] {
            let scan = Scan { restored: true, now: Timestamp::from_secs(NOW), days: 7, running: Ok(&set) };

            assert_eq!(decide(&item, &scan), left_out(Reason::ScanAgain), "{:?}", item.name);
        }
    }

    #[test]
    fn a_lock_name_is_kept() {
        for name in [".X0-lock", "app.lock", "server.pid", "LCK..ttyS0", ".X11-unix"] {
            assert_eq!(kept(&file(name)), left_out(Reason::Lock), "{name}");
        }
    }

    #[test]
    fn a_socket_pipe_or_device_and_a_folder_holding_one_are_special() {
        let other = Item { kind: EntryKind::Other, ..file("dev.sock") };
        let holding = with_inside(folder("srv"), |inside| inside.special = true);

        assert_eq!(kept(&other), left_out(Reason::Special));
        assert_eq!(kept(&holding), left_out(Reason::Special));
    }

    #[test]
    fn an_item_of_the_system_or_another_user_is_kept() {
        let facts = EntryFacts::REMOVABLE;
        for item in [
            Item { facts: EntryFacts { owned_by_user: false, ..facts }, ..file("theirs") },
            Item { facts: EntryFacts { system_immutable: true, ..facts }, ..folder("daemon") },
            Item { facts: EntryFacts { restricted: true, ..facts }, ..file("guarded") },
            with_inside(folder("mixed"), |inside| inside.foreign = true),
            folder("com.apple.someagent"),
            Item { facts: EntryFacts::default(), ..file("unknown") },
        ] {
            assert_eq!(kept(&item), left_out(Reason::System), "{:?}", item.name);
        }
    }

    #[test]
    fn an_item_the_user_locked_is_kept() {
        let item = Item { facts: EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE }, ..file("kept.txt") };

        assert_eq!(kept(&item), left_out(Reason::Locked));
    }

    #[test]
    fn a_failed_running_list_keeps_everything() {
        let scan = Scan { restored: false, now: Timestamp::from_secs(NOW), days: 7, running: Err(NoSystem) };

        assert_eq!(decide(&file("old.tmp"), &scan), left_out(Reason::NoSystem));
        assert_eq!(decide(&folder("build"), &scan), left_out(Reason::NoSystem));
    }

    #[test]
    fn an_item_named_after_a_running_app_is_kept() {
        for name in ["com.example.Editor", "com.example.editor.tmp", "COM.EXAMPLE.EDITOR"] {
            assert_eq!(kept(&folder(name)), left_out(Reason::AppOpen), "{name}");
            assert_eq!(kept(&file(name)), left_out(Reason::AppOpen), "{name}");
        }
    }

    #[test]
    fn a_folder_not_read_in_full_is_not_claimed() {
        let incomplete = with_inside(folder("build"), |inside| inside.complete = false);
        let unread = Item { inside: None, ..folder("build") };

        assert_eq!(kept(&incomplete), Verdict::NotClaimed);
        assert_eq!(kept(&unread), Verdict::NotClaimed);
    }

    #[test]
    fn exactly_the_days_old_is_recent() {
        let item = Item { modified: ago(N), changed: ago(N), ..file("boundary") };
        let old_folder = with_inside(Item { modified: ago(N), changed: ago(N), ..folder("f") }, |inside| {
            inside.newest_change = ago(N);
        });

        assert_eq!(kept(&item), left_out(Reason::Recent));
        assert_eq!(kept(&old_folder), left_out(Reason::Recent));
    }

    #[test]
    fn a_future_time_is_recent() {
        let future = Some(Timestamp::from_secs(NOW + 2 * DAY));
        let file = Item { modified: future, ..file("future") };
        let folder = with_inside(folder("future"), |inside| inside.newest_change = future);

        assert_eq!(kept(&file), left_out(Reason::Recent));
        assert_eq!(kept(&folder), left_out(Reason::Recent));
    }

    #[test]
    fn an_unknown_time_is_recent() {
        for item in [
            Item { modified: None, ..file("a") },
            Item { changed: None, ..file("b") },
            Item { modified: None, changed: None, ..file("c") },
            Item { changed: None, ..folder("d") },
            with_inside(folder("e"), |inside| inside.newest_change = None),
        ] {
            assert_eq!(kept(&item), left_out(Reason::Recent), "{:?}", item.name);
        }
    }

    #[test]
    fn an_old_modification_with_a_fresh_status_change_is_recent() {
        let unpacked = Item { modified: ago(2000 * DAY), changed: ago(60), ..file("unpacked") };
        let folder = Item { modified: ago(2000 * DAY), changed: ago(60), ..folder("unpacked") };

        assert_eq!(kept(&unpacked), left_out(Reason::Recent));
        assert_eq!(kept(&folder), left_out(Reason::Recent));
    }

    #[test]
    fn an_old_folder_with_one_file_changed_yesterday_is_recent() {
        let item = with_inside(folder("build"), |inside| inside.newest_change = ago(DAY));

        assert_eq!(kept(&item), left_out(Reason::Recent));
    }

    #[test]
    fn a_file_read_within_the_days_keeps_its_item() {
        let read = Item { accessed: ago(DAY), ..file("read.dat") };
        let read_at_the_boundary = Item { accessed: ago(N), ..file("read.dat") };
        let read_in_the_future = Item { accessed: Some(Timestamp::from_secs(NOW + DAY)), ..file("read.dat") };
        let folder = with_inside(folder("build"), |inside| inside.newest_file_access = ago(DAY));

        for item in [read, read_at_the_boundary, read_in_the_future, folder] {
            assert_eq!(kept(&item), left_out(Reason::Recent), "{:?}", item.accessed);
        }
    }

    #[test]
    fn a_higher_setting_keeps_what_the_default_offers() {
        let set = RunningSet::default();
        let item = Item { modified: ago(10 * DAY), changed: ago(10 * DAY), accessed: None, ..file("ten-days") };
        let scan = Scan { restored: false, now: Timestamp::from_secs(NOW), days: 14, running: Ok(&set) };

        assert_eq!(decide(&item, &scan), left_out(Reason::Recent));
    }

    #[test]
    fn a_hidden_name_after_a_running_app_is_kept() {
        for name in [".com.example.Editor.XXXXXX", ".com.example.Editor"] {
            assert_eq!(kept(&folder(name)), left_out(Reason::AppOpen), "{name}");
        }
    }

    #[test]
    fn a_name_that_is_not_utf8_after_a_running_app_is_kept() {
        use std::os::unix::ffi::OsStrExt as _;

        for name in [&b"com.example.Editor.\xff"[..], b".com.example.Editor.\xfe\xff"] {
            let item = Item { name: std::ffi::OsStr::from_bytes(name), ..file("x") };

            assert_eq!(kept(&item), left_out(Reason::AppOpen), "{name:?}");
        }
    }

    #[test]
    fn a_folder_holding_a_lock_files_name_is_kept_as_a_lock() {
        let item = with_inside(folder("build"), |inside| inside.lock_named = true);

        assert_eq!(kept(&item), left_out(Reason::Lock));
    }

    #[test]
    fn a_restore_comes_before_every_other_row() {
        let scan = Scan { restored: true, now: Timestamp::from_secs(NOW), days: 7, running: Err(NoSystem) };
        let worst = Item {
            kind: EntryKind::Other,
            facts: EntryFacts { owned_by_user: false, user_immutable: true, ..EntryFacts::default() },
            modified: None,
            ..file("com.apple.x.lock")
        };

        assert_eq!(decide(&worst, &scan), left_out(Reason::ScanAgain));
    }

    #[test]
    fn a_lock_name_comes_before_special_and_system() {
        let item = Item { kind: EntryKind::Other, facts: EntryFacts::default(), ..file("com.apple.x.pid") };

        assert_eq!(kept(&item), left_out(Reason::Lock));
    }

    #[test]
    fn special_comes_before_system() {
        let socket = Item { kind: EntryKind::Other, facts: EntryFacts::default(), ..file("com.apple.sock") };
        let folder = with_inside(folder("com.apple.srv"), |inside| {
            inside.special = true;
            inside.foreign = true;
        });

        assert_eq!(kept(&socket), left_out(Reason::Special));
        assert_eq!(kept(&folder), left_out(Reason::Special));
    }

    #[test]
    fn system_comes_before_locked() {
        let facts = EntryFacts { owned_by_user: false, user_immutable: true, ..EntryFacts::REMOVABLE };

        assert_eq!(kept(&Item { facts, ..file("updater.pkg") }), left_out(Reason::System));
    }

    #[test]
    fn every_structural_row_comes_before_age() {
        let fresh = |item: Item<'static>| Item { modified: ago(60), changed: ago(60), ..item };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        for (item, reason) in [
            (fresh(file("app.lock")), Reason::Lock),
            (fresh(Item { kind: EntryKind::Other, ..file("pipe") }), Reason::Special),
            (fresh(file("com.apple.x")), Reason::System),
            (fresh(Item { facts: locked, ..file("kept") }), Reason::Locked),
            (fresh(file("com.example.Editor")), Reason::AppOpen),
        ] {
            assert_eq!(kept(&item), left_out(reason), "{:?}", item.name);
        }
        let scan = Scan { restored: false, now: Timestamp::from_secs(NOW), days: 7, running: Err(NoSystem) };
        assert_eq!(decide(&fresh(file("x")), &scan), left_out(Reason::NoSystem));
        let incomplete = with_inside(fresh(folder("x")), |inside| inside.complete = false);
        assert_eq!(kept(&incomplete), Verdict::NotClaimed);
    }
}

mod positive {
    use fresher_core::{EntryKind, RunningSet};

    use super::{DAY, N, ago, file, folder, kept, verdict, with_inside};
    use crate::decide::{Item, Verdict};

    #[test]
    fn the_days_and_one_second_is_offered() {
        let item = Item { modified: ago(N + 1), changed: ago(N + 1), accessed: ago(N + 1), ..file("boundary") };

        assert_eq!(kept(&item), Verdict::Offered { age_days: 7 });
    }

    #[test]
    fn an_old_file_is_offered_with_its_whole_days() {
        assert_eq!(kept(&file("tmp.IKKNWYIRnq")), Verdict::Offered { age_days: 30 });
    }

    #[test]
    fn an_old_link_is_offered_as_itself() {
        let link = Item { kind: EntryKind::Symlink, accessed: None, ..file("docs-link") };

        assert_eq!(kept(&link), Verdict::Offered { age_days: 30 });
    }

    #[test]
    fn an_old_folder_is_offered_by_its_newest_change_inside() {
        let item = with_inside(folder("node-compile-cache"), |inside| inside.newest_change = ago(20 * DAY));

        assert_eq!(kept(&item), Verdict::Offered { age_days: 20 });
    }

    #[test]
    fn an_item_named_like_but_not_after_a_running_app_is_offered() {
        for name in ["com.example.Editorx", "com.example", "x.com.example.Editor"] {
            assert_eq!(kept(&folder(name)), Verdict::Offered { age_days: 30 }, "{name}");
        }
    }

    #[test]
    fn a_folder_read_long_ago_is_offered_though_its_own_access_is_unknown() {
        assert_eq!(verdict(&folder("build"), &RunningSet::default()), Verdict::Offered { age_days: 30 });
    }
}

mod property {
    use fresher_core::{EntryFacts, EntryKind, Inside, RunningSet, Timestamp};
    use proptest::prelude::*;

    use crate::decide::{Item, Scan, Verdict, decide};

    fn time() -> impl Strategy<Value = Option<Timestamp>> {
        proptest::option::weighted(0.9, (0..4_000_000_000_u64).prop_map(Timestamp::from_secs))
    }

    fn item(
        folder: bool,
        modified: Option<Timestamp>,
        changed: Option<Timestamp>,
        accessed: Option<Timestamp>,
        newest_change: Option<Timestamp>,
    ) -> Item<'static> {
        let inside = Inside { newest_change, newest_file_access: accessed, ..Inside::EMPTY };
        Item {
            name: std::ffi::OsStr::new("tmp.item"),
            kind: if folder { EntryKind::Folder } else { EntryKind::File },
            facts: EntryFacts::REMOVABLE,
            modified,
            changed,
            accessed: if folder { None } else { accessed },
            inside: folder.then_some(inside),
        }
    }

    fn offered(item: &Item<'_>, now: u64, days: u32) -> bool {
        let set = RunningSet::default();
        let scan = Scan { restored: false, now: Timestamp::from_secs(now), days, running: Ok(&set) };
        matches!(decide(item, &scan), Verdict::Offered { .. })
    }

    proptest! {
        #[test]
        fn a_later_now_never_keeps_by_age_what_an_earlier_one_offers(
            folder: bool, modified in time(), changed in time(), accessed in time(), newest in time(),
            now in 0..4_000_000_000_u64, later in 0..4_000_000_000_u64, days in 7..=30_u32,
        ) {
            let item = item(folder, modified, changed, accessed, newest);
            prop_assert!(!offered(&item, now, days) || offered(&item, now + later, days));
        }

        #[test]
        fn a_higher_setting_never_offers_what_a_lower_one_keeps(
            folder: bool, modified in time(), changed in time(), accessed in time(), newest in time(),
            now in 0..4_000_000_000_u64, days in 7..=30_u32, more in 0..=23_u32,
        ) {
            let item = item(folder, modified, changed, accessed, newest);
            prop_assert!(offered(&item, now, days) || !offered(&item, now, days + more));
        }
    }
}
