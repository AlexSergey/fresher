//! The scanner with the portable reader against the reference walk: the same matches and the same summary.

#[cfg(test)]
mod support;

mod negative {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::Arc;

    use fresher_core::{Claim, Completeness, EntryFacts, EntryKind, Pattern, Removability, Timestamp, Unreadable};
    use fresher_scanner::StdReader;
    use fresher_testing::FixtureTree;

    #[cfg(target_os = "macos")]
    use crate::support::LockedHome;
    use crate::support::{
        OLD, SpecialHome, TimesHome, canonical, every_interest, facts_of, interest, reference, scanner, set_of,
    };

    #[test]
    fn an_unreadable_folder_is_reported_by_kind_and_makes_whole_sizes_partial() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"Library/Logs/locked/a\"\nsize = 5\n[[file]]\npath = \"Library/Logs/b\"\nsize = 7\n\
             [[file]]\npath = \"Projects/locked/c\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let locked = [root.join("Library/Logs/locked"), root.join("Projects/locked")];
        for folder in &locked {
            fs::set_permissions(folder, fs::Permissions::from_mode(0o000)).unwrap();
        }
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);
        let slow = reference(&root, &set, &[]);
        for folder in &locked {
            fs::set_permissions(folder, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(fast, slow);
        assert_eq!(fast.unreadable.get(&Unreadable::NoPermission), Some(&1), "the one outside the whole claim");
        let logs = fast.matches.iter().find(|found| found.path == root.join("Library/Logs")).unwrap();
        assert_eq!((logs.sizes.logical.bytes(), logs.sizes.completeness), (7, Completeness::Partial));
    }

    #[test]
    fn a_link_to_a_folder_is_listed_and_not_followed() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"real/node_modules/a.json\"\nsize = 9\n\
             [[symlink]]\npath = \"linked\"\ntarget = \"real\"\n[[symlink]]\npath = \"node_modules\"\ntarget = \"real/node_modules\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        assert!(fast.matches.iter().all(|found| !found.path.starts_with(root.join("linked/"))));
        let link = fast.matches.iter().find(|found| found.path == root.join("node_modules")).unwrap();
        assert_eq!((link.kind, link.sizes.logical.bytes()), (EntryKind::Symlink, 0));
    }

    #[test]
    fn a_link_out_of_the_root_is_not_followed() {
        let outside = FixtureTree::from_toml_str("[[file]]\npath = \"Caches/x.json\"\nsize = 100\n").unwrap();
        let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"home\"\n").unwrap();
        let root = canonical(&tree.root().join("home"));
        symlink(outside.root(), root.join("out")).unwrap();
        symlink(outside.root().join("Caches"), root.join("Caches")).unwrap();
        let set = set_of(&[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])]);

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        assert_eq!(fast.entries, 2, "the two links, nothing behind them");
        assert_eq!(fast.matches.len(), 1);
        assert_eq!(fast.matches[0].sizes.logical.bytes(), 0);
    }

    #[test]
    fn a_look_alike_is_not_matched_and_case_counts() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"Caches.dmg.txt\"\n[[file]]\npath = \"Library/Caches\"\n[[file]]\npath = \"cache/cachedir.tag\"\n\
             [[dir]]\npath = \"app/node_modules\"\n[[file]]\npath = \"app/Package.json\"\n[[file]]\npath = \"rust/cargo.toml\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        let paths: Vec<_> = fast.matches.iter().map(|found| found.path.strip_prefix(&root).unwrap()).collect();
        assert_eq!(paths, ["app/Package.json", "app/node_modules"].map(std::path::Path::new));
        assert_eq!(fast.matches[1].siblings, Vec::<String>::new(), "Package.json is not package.json");
    }

    #[test]
    fn a_pipe_named_like_a_marker_is_not_a_marker() {
        let home = SpecialHome::new();
        let set = SpecialHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert_eq!(home.matched(&fast), ["x/pipe", "x/socket"].map(std::path::PathBuf::from));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn the_std_reader_gives_no_removable_facts_to_what_the_user_cannot_remove() {
        let home = LockedHome::new();
        let set = LockedHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

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
    fn an_archive_tools_old_times_do_not_hide_the_status_change() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        let archive = home.inside(&fast, "archive");
        assert_eq!(archive.newest_change, Some(home.archive_status_change()));
        assert!(archive.newest_change > Some(Timestamp::from_secs(OLD)));
        assert!(archive.complete && !archive.special && !archive.foreign);
    }

    #[test]
    fn a_folder_holding_an_unreadable_folder_is_not_complete() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert!(!home.inside(&fast, "locked").complete);
    }

    #[test]
    fn a_folder_holding_a_pipe_is_special() {
        let home = TimesHome::new();
        let set = TimesHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        let pipes = home.inside(&fast, "pipes");
        assert!(pipes.special && pipes.complete);
        assert!(!home.inside(&fast, "archive").special);
    }
}

mod positive {
    use std::fs;
    use std::sync::Arc;

    use fresher_core::{CACHEDIR_SIGNATURE, Claim, EntryFacts, EntryKind, KnownFolder, MarkerCheck, Pattern, Sizes};
    use fresher_scanner::StdReader;
    use fresher_testing::FixtureTree;

    #[cfg(target_os = "macos")]
    use crate::support::LockedHome;
    use crate::support::{
        SpecialHome, canonical, every_interest, facts_of, fixtures, interest, reference, scanner, set_of,
    };

    #[test]
    fn a_folder_matched_whole_and_as_an_item_is_summed_and_reported_twice() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"Downloads/Install.app/Contents/Info.plist\"\nsize = 5\n\
             [[file]]\npath = \"Downloads/Install.app/Contents/MacOS/run\"\nsize = 7\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let downloads = |claim| interest(Pattern::Under(KnownFolder::Downloads, "*".into()), claim, &[]);
        let set =
            set_of(&[downloads(Claim::Whole).only_suffixes(vec![".app".into()]).unwrap(), downloads(Claim::Item)]);

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        let bundle = root.join("Downloads/Install.app");
        let hits: Vec<_> = fast.matches.iter().filter(|found| found.path == bundle).collect();
        assert_eq!(hits.iter().map(|found| found.interest).collect::<Vec<_>>(), [0, 1]);
        assert_eq!(hits[0].sizes.logical.bytes(), 12, "summed whole");
        assert!(hits[0].inside.is_some());
        assert_eq!((hits[1].sizes, hits[1].inside), (Sizes::ZERO, None));
        assert_eq!(fast.matches.len(), 2, "nothing inside the whole claim is reported");
    }

    #[test]
    fn a_pipe_and_a_socket_match_as_themselves() {
        let home = SpecialHome::new();
        let set = SpecialHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

        assert_eq!(fast, reference(&home.root, &set, &[]));
        assert_eq!(fast.matches.len(), 2);
        for found in &fast.matches {
            assert_eq!((found.kind, found.sizes.allocated.bytes()), (EntryKind::Other, 0), "{}", found.path.display());
        }
    }

    #[test]
    fn the_scanner_and_the_reference_agree_on_every_fixture_home() {
        let set = set_of(&every_interest());
        let mut homes = 0;
        for entry in fs::read_dir(fixtures()).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "toml") {
                continue;
            }
            let tree = FixtureTree::from_toml(&path).unwrap();
            let root = canonical(tree.root());
            let avoid = [root.join("Documents")];

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &avoid);

            assert_eq!(fast, reference(&root, &set, &avoid), "{}", path.display());
            homes += 1;
        }
        assert!(homes >= 2, "the fixture homes were not found");
    }

    #[test]
    fn the_scanner_and_the_reference_agree_on_every_kind_of_match() {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"Downloads/b.dmg\"\nsize = 7\nmodified = 1000\n[[file]]\npath = \"Library/Caches/com.app/data\"\nsize = 3\n\
             [[file]]\npath = \"Library/Logs/x.log\"\nsize = 11\n[[file]]\npath = \"project/cache/blob\"\nsize = 13\n\
             [[file]]\npath = \"app/node_modules/pkg/index.json\"\nsize = 17\n[[file]]\npath = \"app/package.json\"\n\
             [[file]]\npath = \"rust/Cargo.toml\"\n[[file]]\npath = \"rust/Cargo.lock\"\n[[file]]\npath = \"Caches/x/Caches/y.json\"\n\
             [[hardlink]]\npath = \"app/node_modules/pkg/same.json\"\ntarget = \"app/node_modules/pkg/index.json\"\n",
        )
        .unwrap();
        fs::write(tree.root().join("project/cache/CACHEDIR.TAG"), CACHEDIR_SIGNATURE).unwrap();
        fresher_testing::settle_access_time(&tree.root().join("project/cache/CACHEDIR.TAG")).unwrap();
        let root = canonical(tree.root());
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        let tagged = fast.matches.iter().find(|found| found.path == root.join("project/cache")).unwrap();
        assert_eq!(tagged.marker, Some(MarkerCheck::Valid));
        let modules = fast.matches.iter().find(|found| found.path == root.join("app/node_modules")).unwrap();
        assert_eq!(modules.sizes.logical.bytes(), 34, "a hard link counts once per name in a match");
        assert_eq!(fast.matches.len(), 12, "the cache folder matched under the home and under its known folder");
        assert!(fast.matches.iter().all(|found| found.facts == EntryFacts::REMOVABLE), "all the user's own");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn the_std_reader_gives_the_users_own_files_removable_facts() {
        let home = LockedHome::new();
        let set = LockedHome::interests();

        let fast = scanner(Arc::new(StdReader::new()), &home.root, &set, &[]);

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

    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    use fresher_core::{CancelToken, DiskWalk, InterestSet, Subtree, SubtreeQuery, WalkMode, WalkRequest};
    use fresher_scanner::StdReader;
    use fresher_testing::NaiveWalk;
    use proptest::prelude::*;

    use crate::support::{
        AVOIDABLE, bundle_id_interests, canonical, every_interest, held_interests, place_interests, random_cache_tree,
        random_download_tree, random_items, random_tree, reference, scanner, scanner_over, set_of, suffixed_interests,
        tagged,
    };

    /// What `walk` saw below each of `queries` after a full walk of `root` for `interests`, before any commit.
    #[expect(clippy::unwrap_used, reason = "a test helper: a walk that fails fails the test")]
    fn walked_subtrees(
        walk: &dyn DiskWalk,
        root: &Path,
        interests: &InterestSet,
        queries: &[SubtreeQuery],
    ) -> Vec<Option<Subtree>> {
        let roots = [root.to_path_buf()];
        let request = WalkRequest { roots: &roots, interests, avoid: &[], places: &[], mode: WalkMode::Full };
        walk.walk(&request, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();
        walk.subtrees(queries)
    }

    /// Every folder below `root`, links not followed, sorted; a folder that cannot be listed has nothing below it.
    fn folders_below(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(folder) = pending.pop() {
            let Ok(entries) = fs::read_dir(&folder) else { continue };
            let inside: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.symlink_metadata().is_ok_and(|metadata| metadata.is_dir()))
                .collect();
            found.extend(inside.iter().cloned());
            pending.extend(inside);
        }
        found.sort();
        found
    }

    /// Gives each of `folders` its own modification time, older than every file's of the random trees
    /// (`1_500_000_000` and up): a walk that counted a file's time would see a newer one.
    fn date_folders(folders: &[PathBuf]) {
        for (index, folder) in (0_u64..).zip(folders) {
            let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_400_000_000 + index * 1_000);
            let opened = fs::File::open(folder);
            let _ = opened.and_then(|file| file.set_times(fs::FileTimes::new().set_modified(modified)));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

        #[test]
        fn the_scanner_and_the_reference_agree_on_random_trees(
            items in random_items(),
            chosen in prop::collection::vec(any::<bool>(), 8),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_tree(&root, &items);
            let interests: Vec<_> =
                every_interest().into_iter().zip(&chosen).filter(|(_, keep)| **keep).map(|(interest, _)| interest).collect();
            let set = set_of(&interests);

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_scanner_and_the_reference_agree_on_what_matched_folders_hold(
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

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_scanner_and_the_reference_agree_on_names_shaped_as_bundle_ids(
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

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_scanner_and_the_reference_agree_on_suffixed_names(
            items in random_items(),
            chosen in prop::collection::vec(any::<bool>(), 8),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = random_download_tree(&root, &items);
            let interests: Vec<_> = suffixed_interests()
                .into_iter()
                .zip(&chosen)
                .filter(|(_, keep)| **keep)
                .map(|(interest, _)| interest)
                .collect();
            let set = set_of(&interests);

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);
            let slow = reference(&root, &set, &[]);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_scanner_and_the_reference_agree_on_what_they_saw_of_each_place(
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

            let fast = scanner(Arc::new(StdReader::new()), &root, &set, &avoid);
            let slow = reference(&root, &set, &avoid);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            prop_assert_eq!(fast, slow);
        }

        #[test]
        fn the_scanner_and_the_reference_agree_on_subtrees(
            items in random_items(),
            chosen in prop::collection::vec(any::<bool>(), 10),
            skipped in prop::collection::vec(any::<bool>(), 1..16),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            // Links are summed differently by design: the scanner counts a link's own bytes (its target's length) and a
            // hard link group's bytes once, the reference neither; the sizes are compared only on a tree without them.
            let linked = items.iter().any(|(kind, _, _)| matches!(kind % 8, 2..=4));
            let locked = random_tree(&root, &items);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }
            let folders = folders_below(&root);
            date_folders(&folders);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o000));
            }
            let interests: Vec<_> =
                every_interest().into_iter().zip(&chosen).filter(|(_, keep)| **keep).map(|(interest, _)| interest).collect();
            let set = set_of(&interests);
            // Every folder, the root among them, each skipping a pseudo-random set of the folders below it.
            let mut draws = skipped.iter().cycle();
            let queries: Vec<_> = std::iter::once(root.clone())
                .chain(folders.iter().cloned())
                .map(|queried| {
                    let below = folders.iter().filter(|other| other.starts_with(&queried) && **other != queried);
                    let skip = below.filter(|_| draws.next().copied().unwrap_or(false)).cloned().collect();
                    SubtreeQuery { folder: queried, skip }
                })
                .collect();

            let fast = walked_subtrees(&scanner_over(Arc::new(StdReader::new()), &root), &root, &set, &queries);
            let slow = walked_subtrees(&NaiveWalk::new(&root), &root, &set, &queries);
            for folder in &locked {
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o755));
            }

            let seen = |answers: &[Option<Subtree>]| -> Vec<_> {
                answers.iter().map(|found| found.map(|found| (found.newest, found.complete))).collect()
            };
            let sizes = |answers: &[Option<Subtree>]| -> Vec<_> {
                answers.iter().map(|found| found.map(|found| found.sizes.logical)).collect()
            };
            prop_assert_eq!(seen(&fast), seen(&slow));
            if !linked {
                prop_assert_eq!(sizes(&fast), sizes(&slow));
            }
        }
    }
}
