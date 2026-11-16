//! The scanner with the portable reader against the reference walk: the same matches and the same summary.

#[cfg(test)]
mod support;

mod negative {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::Arc;

    use fresher_core::{Claim, Completeness, EntryKind, Pattern, Unreadable};
    use fresher_scanner::StdReader;
    use fresher_testing::FixtureTree;

    use crate::support::{canonical, every_interest, interest, reference, scanner, set_of};

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
}

mod positive {
    use std::fs;
    use std::sync::Arc;

    use fresher_core::{CACHEDIR_SIGNATURE, MarkerCheck};
    use fresher_scanner::StdReader;
    use fresher_testing::FixtureTree;

    use crate::support::{canonical, every_interest, fixtures, reference, scanner, set_of};

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
        let root = canonical(tree.root());
        let set = set_of(&every_interest());

        let fast = scanner(Arc::new(StdReader::new()), &root, &set, &[]);

        assert_eq!(fast, reference(&root, &set, &[]));
        let tagged = fast.matches.iter().find(|found| found.path == root.join("project/cache")).unwrap();
        assert_eq!(tagged.marker, Some(MarkerCheck::Valid));
        let modules = fast.matches.iter().find(|found| found.path == root.join("app/node_modules")).unwrap();
        assert_eq!(modules.sizes.logical.bytes(), 34, "a hard link counts once per name in a match");
        assert_eq!(fast.matches.len(), 11);
    }
}

mod property {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::CACHEDIR_SIGNATURE;
    use fresher_scanner::StdReader;
    use proptest::prelude::*;

    use crate::support::{canonical, every_interest, reference, scanner, set_of};

    const NAMES: [&str; 10] =
        ["a", "b", "node_modules", "Caches", "Library", "Logs", "x.json", "CACHEDIR.TAG", "package.json", "Cargo.toml"];

    /// One thing to create: a kind (folder, file, link out, link in, hard link, tag, unreadable folder), a path of
    /// names from `NAMES`, and a size.
    type Item = (u8, Vec<usize>, u64);

    fn build(root: &Path, items: &[Item]) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut locked = Vec::new();
        for (kind, names, size) in items {
            let path = names.iter().fold(root.to_path_buf(), |path, &name| path.join(NAMES[name % NAMES.len()]));
            if path.symlink_metadata().is_ok() {
                continue;
            }
            let Some(parent) = path.parent() else { continue };
            if fs::create_dir_all(parent).is_err() {
                continue;
            }
            let made = match kind % 7 {
                0 => fs::create_dir(&path).is_ok(),
                1 => fs::File::create(&path).and_then(|file| file.set_len(*size)).is_ok(),
                2 => symlink("/usr", &path).is_ok(),
                3 => symlink(root.join("a"), &path).is_ok(),
                4 => files.first().is_some_and(|target: &PathBuf| fs::hard_link(target, &path).is_ok()),
                5 => fs::write(&path, if size % 2 == 0 { CACHEDIR_SIGNATURE } else { b"Signature: 0" }).is_ok(),
                _ => {
                    fs::create_dir(&path).is_ok() && {
                        locked.push(path.clone());
                        true
                    }
                }
            };
            if made && matches!(kind % 7, 1 | 5) {
                files.push(path);
            }
        }
        for folder in &locked {
            let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o000));
        }
        locked
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

        #[test]
        fn the_scanner_and_the_reference_agree_on_random_trees(
            items in prop::collection::vec((0_u8..7, prop::collection::vec(0_usize..10, 1..4), 0_u64..5_000_000), 0..40),
            chosen in prop::collection::vec(any::<bool>(), 8),
        ) {
            let folder = tempfile::tempdir().unwrap();
            let root = canonical(folder.path());
            let locked = build(&root, &items);
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
    }
}
