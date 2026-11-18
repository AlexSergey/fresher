//! The scanner with the macOS bulk reader against the reference walk: the same matches and the same summary.

#![cfg(target_os = "macos")]

#[cfg(test)]
mod support;

mod negative {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::Arc;

    use fresher_core::{Claim, EntryFacts, Pattern, Removability, Timestamp, Unreadable};
    use fresher_macos::MacFolderReader;
    use fresher_testing::FixtureTree;

    use crate::support::{
        LockedHome, OLD, SpecialHome, TimesHome, canonical, every_interest, facts_of, interest, reference, scanner,
        set_of,
    };

    #[test]
    fn the_macos_reader_reports_an_unreadable_folder_like_the_reference() {
        let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Projects/locked/a\"\n[[file]]\npath = \"b.json\"\n")
            .unwrap();
        let root = canonical(tree.root());
        let locked = root.join("Projects/locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);
        let slow = reference(&root, &set, &[]);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(fast, slow);
        assert_eq!(fast.unreadable.get(&Unreadable::NoPermission), Some(&1));
    }

    #[test]
    fn the_macos_reader_does_not_follow_a_link_out_of_the_root() {
        let outside = FixtureTree::from_toml_str("[[file]]\npath = \"Caches/x\"\nsize = 100\n").unwrap();
        let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"home\"\n").unwrap();
        let root = canonical(&tree.root().join("home"));
        symlink(outside.root().join("Caches"), root.join("Caches")).unwrap();
        let set = set_of(&[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])]);

        let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        assert_eq!(fast.matches[0].sizes.logical.bytes(), 0);
    }

    #[test]
    fn the_macos_reader_gives_no_removable_facts_to_what_the_user_cannot_remove() {
        let home = LockedHome::new();
        let set = LockedHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        let closed = facts_of(&fast, &home.root.join("closed/inside.json"));
        let folder = facts_of(&fast, &home.root.join("Caches"));
        let locked = facts_of(&fast, &home.root.join("locked.json"));
        assert_eq!(closed, EntryFacts { parent_writable: false, ..EntryFacts::REMOVABLE });
        assert_eq!(folder, EntryFacts { folder_writable: false, ..EntryFacts::REMOVABLE });
        assert_eq!(locked, EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE });
        assert_eq!(Removability::of(closed), Removability::NeedsAdministrator);
        assert_eq!(Removability::of(folder), Removability::NeedsAdministrator);
        assert_eq!(Removability::of(locked), Removability::Locked);
    }

    #[test]
    fn a_pipe_named_like_a_marker_is_not_a_marker_to_the_macos_reader() {
        let home = SpecialHome::new();
        let set = SpecialHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert_eq!(home.matched(&fast), ["x/pipe", "x/socket"].map(std::path::PathBuf::from));
    }

    #[test]
    fn an_archive_tools_old_times_do_not_hide_the_status_change_to_the_macos_reader() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        let archive = home.inside(&fast, "archive");
        assert_eq!(archive.newest_change, Some(home.archive_status_change()));
        assert!(archive.newest_change > Some(Timestamp::from_secs(OLD)));
        assert!(archive.complete && !archive.special && !archive.foreign);
    }

    #[test]
    fn a_folder_holding_an_unreadable_folder_is_not_complete_to_the_macos_reader() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert!(!home.inside(&fast, "locked").complete);
    }

    #[test]
    fn a_folder_holding_a_pipe_is_special_to_the_macos_reader() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        let pipes = home.inside(&fast, "pipes");
        assert!(pipes.special && pipes.complete);
        assert!(!home.inside(&fast, "archive").special);
    }
}

mod positive {
    use std::fs;
    use std::sync::Arc;

    use fresher_core::{CACHEDIR_SIGNATURE, EntryFacts, EntryKind};
    use fresher_macos::MacFolderReader;
    use fresher_testing::FixtureTree;

    use crate::support::{
        LockedHome, SpecialHome, canonical, every_interest, facts_of, fixtures, reference, scanner, set_of,
    };

    #[test]
    fn a_pipe_and_a_socket_match_as_themselves_to_the_macos_reader() {
        let home = SpecialHome::new();
        let set = SpecialHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert_eq!(fast.matches.len(), 2);
        for found in &fast.matches {
            assert_eq!((found.kind, found.sizes.allocated.bytes()), (EntryKind::Other, 0), "{}", found.path.display());
        }
    }

    #[test]
    fn the_macos_reader_and_the_reference_agree_on_every_fixture_home() {
        let set = set_of(&every_interest());
        for entry in fs::read_dir(fixtures()).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "toml") {
                continue;
            }
            let tree = FixtureTree::from_toml(&path).unwrap();
            let root = canonical(tree.root());

            let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);

            assert_eq!(fast, reference(&root, &set, &[]), "{}", path.display());
        }
    }

    #[test]
    fn the_macos_reader_and_the_reference_agree_on_every_kind_of_match() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"Downloads/b.dmg\"\nsize = 7\nmodified = 1000\n[[file]]\npath = \"Library/Caches/com.app/data\"\n\
             [[file]]\npath = \"Library/Logs/x.log\"\nsize = 11\n[[file]]\npath = \"app/node_modules/pkg/index.json\"\nsize = 17\n\
             [[file]]\npath = \"app/package.json\"\n[[file]]\npath = \"rust/Cargo.toml\"\n\
             [[hardlink]]\npath = \"app/node_modules/pkg/same.json\"\ntarget = \"app/node_modules/pkg/index.json\"\n",
        )
        .unwrap();
        fs::create_dir(tree.root().join("project")).unwrap();
        fs::write(tree.root().join("project/CACHEDIR.TAG"), CACHEDIR_SIGNATURE).unwrap();
        fresher_testing::settle_access_time(&tree.root().join("project/CACHEDIR.TAG")).unwrap();
        fs::write(tree.root().join("project/blob"), vec![1_u8; 10_000]).unwrap();
        let root = canonical(tree.root());
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        assert!(fast.entries > 10);
        assert!(fast.matches.iter().all(|found| found.facts == EntryFacts::REMOVABLE), "all the user's own");
    }

    #[test]
    fn the_macos_reader_gives_the_users_own_files_removable_facts() {
        let home = LockedHome::new();
        let set = LockedHome::interests();

        let fast = scanner(Arc::new(MacFolderReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert_eq!(fast.matches.len(), 5);
        assert_eq!(facts_of(&fast, &home.root.join("mine.json")), EntryFacts::REMOVABLE);
        assert_eq!(facts_of(&fast, &home.root.join("read-only.json")), EntryFacts::REMOVABLE, "its folder is writable");
        assert!(fast.matches.iter().all(|found| found.facts != EntryFacts::default()), "every entry was read");
    }
}

mod property {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;

    use fresher_macos::MacFolderReader;
    use proptest::prelude::*;

    use crate::support::{
        AVOIDABLE, bundle_id_interests, canonical, every_interest, held_interests, place_interests, random_cache_tree,
        random_items, random_tree, reference, scanner, set_of, tagged,
    };

    proptest! {
        #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

        #[test]
        fn the_macos_reader_and_the_reference_agree_on_random_trees(
            items in random_items(),
            chosen in prop::collection::vec(any::<bool>(), 8),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_tree(&root, &items);
            let interests: Vec<_> =
                every_interest().into_iter().zip(&chosen).filter(|(_, keep)| **keep).map(|(interest, _)| interest).collect();
            let set = set_of(&interests);

            let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_macos_reader_and_the_reference_agree_on_what_matched_folders_hold(
            items in random_items(),
            tags in prop::collection::vec(0_u64..4, 4),
            chosen in prop::collection::vec(any::<bool>(), 9),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_tree(&root, &tagged(&tags, items));
            let interests: Vec<_> = held_interests()
                .into_iter()
                .chain(every_interest().into_iter().take(4))
                .zip(&chosen)
                .filter(|(_, keep)| **keep)
                .map(|(interest, _)| interest)
                .collect();
            let set = set_of(&interests);

            let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_macos_reader_and_the_reference_agree_on_names_shaped_as_bundle_ids(
            items in random_items(),
            chosen in prop::collection::vec(any::<bool>(), 6),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_cache_tree(&root, &items);
            let interests: Vec<_> = bundle_id_interests()
                .into_iter()
                .zip(&chosen)
                .filter(|(_, keep)| **keep)
                .map(|(interest, _)| interest)
                .collect();
            let set = set_of(&interests);

            let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_macos_reader_and_the_reference_agree_on_what_they_saw_of_each_place(
            items in random_items(),
            avoided in prop::collection::vec(any::<bool>(), AVOIDABLE.len()),
            chosen in prop::collection::vec(any::<bool>(), 8),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_tree(&root, &items);
            let interests: Vec<_> = place_interests()
                .into_iter()
                .zip(&chosen)
                .filter(|(_, keep)| **keep)
                .map(|(interest, _)| interest)
                .collect();
            let set = set_of(&interests);
            // The scanner compares an avoided place in its canonical form, the reference as given: no link on the way.
            let avoid: Vec<_> = AVOIDABLE
                .iter()
                .zip(&avoided)
                .filter(|(_, avoid)| **avoid)
                .map(|(path, _)| root.join(path))
                .filter(|path| fs::canonicalize(path).map_or(true, |canonical| canonical == *path))
                .collect();

            let fast = scanner(Arc::new(MacFolderReader::new()), &root, &set, &avoid);
            let slow = reference(&root, &set, &avoid);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }
    }
}
