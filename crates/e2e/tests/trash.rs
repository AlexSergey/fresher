//! The Trash view end to end, on files: the real scanner (its reader seen through the facts lens) with the `trial`
//! module, the real settings file, scan cache and journal, the real `DiskCleaner` reaching only inside the sandbox
//! folder, a fake Trash at the home's `home/.Trash` (its known Trash folder) and a fake clock. Proven here what the
//! sandbox autotest cannot lay out: an entry the user emptied from the Trash, a put back, a permanent cleanup, a second
//! Trash, a rescan after a cleanup, Finder's 30-day setting, and Open Trash. Nothing in this file empties a Trash: the
//! one entry that leaves it is moved out by the test to a folder outside the sandbox, standing in for the user emptying
//! it in Finder; nothing here deletes.

#[cfg(test)]
#[expect(dead_code, reason = "the Trash flows write no golden file")]
mod support;

#[cfg(test)]
#[path = "trash_home/mod.rs"]
mod flow;

mod negative {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;

    use fresher_core::{CommandError, Sizes};

    use crate::flow::{CACHE_A, CACHE_B, Home, USERS_FILE};

    #[test]
    fn no_line_before_any_scan_or_cleanup() {
        let home = Home::new();

        assert_eq!(home.flow.engine.trash().trashes, []);
        home.assert_untouched();
    }

    #[test]
    fn the_users_own_file_counts_only_in_the_first_number() {
        let home = Home::new();

        home.flow.scan();

        let trashes = home.flow.engine.trash().trashes;
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        let waiting = trashes[0].waiting.unwrap();
        let file = fs::metadata(home.flow.path(USERS_FILE)).unwrap();
        assert_eq!((waiting.logical.bytes(), waiting.allocated.bytes()), (16384, file.blocks() * 512));
        assert_eq!(trashes[0].from_cleanups, Sizes::ZERO);
        assert_eq!(trashes[0].gone_by, None);
        home.assert_untouched();
    }

    #[test]
    fn an_entry_emptied_from_the_trash_counts_nothing() {
        let home = Home::new();
        home.scan_and_clean(&[CACHE_A]);
        let trash = home.flow.sandbox.trash();
        let entry = fs::read_dir(&trash)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.file_name().is_some_and(|name| name.to_string_lossy().ends_with("Sample app cache")))
            .unwrap();
        let emptied = tempfile::tempdir().unwrap();
        assert_ne!(home.flow.engine.trash().trashes[0].from_cleanups, Sizes::ZERO, "counted while there");

        fs::rename(&entry, emptied.path().join("gone")).unwrap();

        let trashes = home.flow.engine.trash().trashes;
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].from_cleanups, Sizes::ZERO);
        assert!(home.flow.path(CACHE_B).is_dir());
        home.assert_untouched();
    }

    #[test]
    fn an_item_put_back_counts_nothing() {
        let home = Home::new();
        let cleanup = home.scan_and_clean(&[CACHE_A]);
        assert_ne!(home.flow.engine.trash().trashes[0].from_cleanups, Sizes::ZERO, "counted while there");

        home.flow.engine.put_back(cleanup, None).unwrap();

        let trashes = home.flow.engine.trash().trashes;
        assert!(home.flow.path(CACHE_A).is_dir(), "put back");
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].from_cleanups, Sizes::ZERO);
        home.assert_untouched();
    }

    #[test]
    fn a_permanent_cleanup_adds_nothing_from_cleanups() {
        let home = Home::new();
        home.flow.scan();
        let view = home.prepare(&[CACHE_A, CACHE_B]);

        home.clean_permanently(&view);

        let trashes = home.flow.engine.trash().trashes;
        assert!(!home.flow.path(CACHE_A).exists() && !home.flow.path(CACHE_B).exists(), "deleted");
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].from_cleanups, Sizes::ZERO);
        home.assert_untouched();
    }

    #[test]
    fn open_trash_before_any_answer_is_refused() {
        let home = Home::new();
        home.flow.scan();

        let error = home.flow.engine.open_trash(0);

        assert_eq!(error, Err(CommandError::UnknownTrash { index: 0 }));
        assert_eq!(home.system_ui.calls(), []);
        home.assert_untouched();
    }
}

mod positive {
    use std::time::Duration;

    use fresher_core::{AutoEmpty, ByteSize, Clock as _, Sizes, Timestamp};
    use fresher_testing::SystemUiCall;

    use crate::flow::{CACHE_A, CACHE_B, Extras, Home};

    /// Finder keeps an item in the Trash 30 days with its setting on.
    const THIRTY_DAYS: u64 = 30 * 24 * 60 * 60;

    #[test]
    fn a_cleanup_adds_its_items_to_both_numbers() {
        let home = Home::new();
        home.flow.scan();
        let users_file = home.users_file();
        let view = home.prepare(&[CACHE_A, CACHE_B]);

        let (cleanup, _) = home.flow.clean(&view);

        let trashes = home.flow.engine.trash().trashes;
        let share = home.in_trash(cleanup);
        assert!(share.allocated.bytes() >= 65536 + 32768, "{share:?}");
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].from_cleanups, share);
        assert_eq!(trashes[0].waiting, users_file.checked_add(share));
        home.assert_untouched();
    }

    #[test]
    fn a_rescan_does_not_count_the_moved_items_twice() {
        let home = Home::new();
        home.flow.scan();
        let users_file = home.users_file();
        let (cleanup, _) = home.flow.clean(&home.prepare(&[CACHE_A, CACHE_B]));
        let share = home.in_trash(cleanup);
        home.clock.advance(Duration::from_secs(60));

        home.flow.scan();

        let trashes = home.flow.engine.trash().trashes;
        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].from_cleanups, share);
        assert_eq!(
            trashes[0].waiting.map(|waiting| waiting.allocated),
            Some(ByteSize::new(users_file.allocated.bytes() + share.allocated.bytes()))
        );
        home.assert_untouched();
    }

    #[test]
    fn an_item_trashed_outside_the_home_trash_gets_a_line_of_its_own() {
        let elsewhere = tempfile::tempdir().unwrap();
        let other = elsewhere.path().join(".Trash");
        let home = Home::with(Extras { other_trash: Some(other.clone()), ..Extras::default() });
        home.flow.scan();
        let users_file = home.users_file();
        let view = home.prepare(&[CACHE_A, CACHE_B]);

        let (cleanup, _) = home.flow.clean(&view);

        let trashes = home.flow.engine.trash().trashes;
        let share = home.in_trash(cleanup);
        assert_eq!(trashes.len(), 2, "{trashes:?}");
        assert_eq!(
            (trashes[0].index, trashes[0].waiting, trashes[0].from_cleanups),
            (0, Some(users_file), Sizes::ZERO)
        );
        assert_eq!((trashes[1].index, trashes[1].waiting, trashes[1].from_cleanups), (1, None, share));
        assert_eq!(std::fs::read_dir(&other).unwrap().count(), 2, "both caches went to the second Trash");
        home.flow.engine.open_trash(1).unwrap();
        assert_eq!(home.system_ui.calls(), [SystemUiCall::OpenFolder(other)]);
        home.assert_untouched();
    }

    #[test]
    fn open_trash_opens_the_home_trash() {
        let home = Home::new();
        home.flow.scan();
        let trashes = home.flow.engine.trash().trashes;
        assert_eq!(trashes.len(), 1, "{trashes:?}");

        home.flow.engine.open_trash(0).unwrap();

        assert_eq!(home.system_ui.calls(), [SystemUiCall::OpenFolder(home.flow.path("home/.Trash"))]);
        home.assert_untouched();
    }

    #[test]
    fn finders_setting_on_dates_the_share() {
        let home = Home::with(Extras { finder: AutoEmpty::After30Days, ..Extras::default() });
        home.flow.scan();
        let ended = Timestamp::from_system_time(home.clock.now());
        let view = home.prepare(&[CACHE_A]);
        home.flow.clean(&view);
        home.clock.advance(Duration::from_secs(60));

        let trashes = home.flow.engine.trash().trashes;

        assert_eq!(trashes.len(), 1, "{trashes:?}");
        assert_eq!(trashes[0].gone_by, Some(Timestamp::from_secs(ended.secs() + THIRTY_DAYS)));
        home.assert_untouched();
    }
}
