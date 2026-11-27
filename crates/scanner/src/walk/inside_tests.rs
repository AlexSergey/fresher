//! The times of a match and the inside of a folder claimed whole, through the full walk, over a reader that gives
//! every entry the times a test names.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    Claim, EntryAccess, FolderIdentity, FolderListing, FolderReader, KnownFolder, Match, Pattern, RawEntry, ReadError,
    Timestamp,
};
use fresher_testing::FixtureTree;

use crate::matcher::testing::{interest, set_of};
use crate::walk::std_reader::StdReader;
use crate::walk::tests::{Scanned, canonical, scan};

/// A day, in seconds.
const DAY: u64 = 86_400;
/// 2019-01-01: the modification time an archive tool kept.
const OLD: u64 = 1_546_300_800;
/// The scan's "yesterday".
const YESTERDAY: u64 = 1_763_000_000;

/// The portable reader, with the modification, status change and access times of every entry named in `times`
/// replaced; an entry not named keeps none.
struct Retimed {
    inner: StdReader,
    times: HashMap<&'static str, [u64; 3]>,
}

impl FolderReader for Retimed {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        self.inner.access(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.inner.read(path, expected, out)?;
        let entries: Vec<(Vec<u8>, RawEntry)> = out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        out.clear();
        for (name, entry) in entries {
            let times = std::str::from_utf8(&name).ok().and_then(|name| self.times.get(name));
            let at = |index: usize| times.map(|times| Timestamp::from_secs(times[index]));
            out.push(&name, RawEntry { modified: at(0), changed: at(1), accessed: at(2), ..entry })?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

/// A home with a folder `cache` claimed whole (`old.bin`, `deep/deeper/late.txt`), a file `note.txt` and a link
/// `alias.txt` matched by their extension, and a file `big.dmg` claimed whole; every entry with the times of `times`.
fn scanned(times: &[(&'static str, [u64; 3])]) -> (Scanned, std::path::PathBuf, FixtureTree) {
    let tree = FixtureTree::from_toml_str(
        "[[file]]\npath = \"cache/old.bin\"\nsize = 10\n[[file]]\npath = \"cache/deep/deeper/late.txt\"\nsize = 5\n\
         [[file]]\npath = \"note.txt\"\nsize = 3\n[[file]]\npath = \"big.dmg\"\nsize = 7\n\
         [[symlink]]\npath = \"alias.txt\"\ntarget = \"note.txt\"\n",
    )
    .unwrap();
    let root = canonical(tree.root());
    let reader = Retimed { inner: StdReader::new(), times: times.iter().copied().collect() };
    let interests = set_of(&[
        interest(Pattern::Under(KnownFolder::Home, "cache".into()), Claim::Whole, &[]),
        interest(Pattern::Extension("txt".into()), Claim::Item, &[]),
        interest(Pattern::Extension("dmg".into()), Claim::Whole, &[]),
    ]);
    (scan(Arc::new(reader), &root, &interests, 2), root, tree)
}

/// The times of every entry of the home: the folder's own, then the ones inside it, then the others.
fn every_time() -> Vec<(&'static str, [u64; 3])> {
    vec![
        ("cache", [100, 150, 9_999]),
        ("old.bin", [OLD, OLD, 1_000]),
        ("deep", [200, 200, 99_999]),
        ("deeper", [OLD, OLD, 99_999]),
        ("late.txt", [OLD, YESTERDAY, 500]),
        ("note.txt", [10, 20, 30]),
        ("alias.txt", [11, 21, 31]),
        ("big.dmg", [12, 22, 32]),
    ]
}

fn found<'a>(scanned: &'a Scanned, path: &Path) -> &'a Match {
    scanned.matches.iter().find(|found| found.path == path).unwrap()
}

#[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
fn at(secs: u64) -> Option<Timestamp> {
    Some(Timestamp::from_secs(secs))
}

/// A home with the folder `cache` claimed whole, holding `name` deep inside.
fn holding(name: &str) -> (Scanned, std::path::PathBuf, FixtureTree) {
    let tree = FixtureTree::from_toml_str(&format!("[[file]]\npath = \"cache/deep/{name}\"\nsize = 1\n")).unwrap();
    let root = canonical(tree.root());
    let interests = set_of(&[interest(Pattern::Under(KnownFolder::Home, "cache".into()), Claim::Whole, &[])]);
    (scan(Arc::new(StdReader::new()), &root, &interests, 2), root, tree)
}

mod negative {
    use super::{DAY, OLD, YESTERDAY, at, every_time, found, holding, scanned};

    #[test]
    fn a_whole_folders_own_change_is_not_a_change_inside_it_for_its_parent() {
        let mut times = every_time();
        times[0] = ("cache", [100, YESTERDAY + DAY, 0]);

        let (scanned, root, _tree) = scanned(&times);

        let tree = scanned.tree();
        assert_eq!(tree.newest(tree.lookup(&root).unwrap()), at(YESTERDAY));
    }

    #[test]
    fn a_lock_files_name_deep_inside_marks_the_folder() {
        for name in ["app.lock", "server.pid", "LCK..ttyS0", ".#notes.txt"] {
            let (scanned, root, _tree) = holding(name);

            assert!(found(&scanned, &root.join("cache")).inside.unwrap().lock_named, "{name}");
        }
    }

    #[test]
    fn an_old_looking_file_changed_yesterday_deep_inside_gives_yesterday() {
        let (scanned, root, _tree) = scanned(&every_time());

        let inside = found(&scanned, &root.join("cache")).inside.unwrap();

        assert_eq!(inside.newest_change, at(YESTERDAY));
        assert_ne!(inside.newest_change, at(OLD));
    }

    #[test]
    fn a_folders_own_access_time_never_counts() {
        let (scanned, root, _tree) = scanned(&every_time());

        let cache = found(&scanned, &root.join("cache"));

        assert_eq!(cache.accessed, None);
        assert_eq!(cache.inside.unwrap().newest_file_access, at(1_000));
    }

    #[test]
    fn a_folders_own_change_counts_when_it_is_the_newest() {
        let mut times = every_time();
        times[0] = ("cache", [100, YESTERDAY + DAY, 0]);

        let (scanned, root, _tree) = scanned(&times);

        assert_eq!(found(&scanned, &root.join("cache")).inside.unwrap().newest_change, at(YESTERDAY + DAY));
    }

    #[test]
    fn a_file_or_a_link_has_no_inside() {
        let (scanned, root, _tree) = scanned(&every_time());

        for name in ["note.txt", "alias.txt", "big.dmg"] {
            assert_eq!(found(&scanned, &root.join(name)).inside, None, "{name}");
        }
    }
}

mod positive {
    use fresher_core::{EntryKind, Inside};

    use super::{YESTERDAY, at, every_time, found, holding, scanned};

    #[test]
    fn a_name_like_but_not_a_lock_files_leaves_the_folder_unmarked() {
        for name in ["lockfile", "x.locked", "notes.txt"] {
            let (scanned, root, _tree) = holding(name);

            assert!(!found(&scanned, &root.join("cache")).inside.unwrap().lock_named, "{name}");
        }
    }

    #[test]
    fn a_whole_folder_gives_its_own_times_and_its_inside() {
        let (scanned, root, _tree) = scanned(&every_time());

        let cache = found(&scanned, &root.join("cache"));

        assert_eq!((cache.modified, cache.changed, cache.accessed), (at(100), at(150), None));
        let expected = Inside { newest_change: at(YESTERDAY), newest_file_access: at(1_000), ..Inside::EMPTY };
        assert_eq!(cache.inside, Some(expected));
    }

    #[test]
    fn a_change_deep_inside_a_whole_folder_makes_it_and_its_parent_recent() {
        let (scanned, root, _tree) = scanned(&every_time());

        let tree = scanned.tree();
        assert_eq!(tree.newest(tree.lookup(&root.join("cache")).unwrap()), at(YESTERDAY));
        assert_eq!(tree.newest(tree.lookup(&root).unwrap()), at(YESTERDAY));
    }

    #[test]
    fn a_file_and_a_link_give_their_own_times() {
        let (scanned, root, _tree) = scanned(&every_time());

        let note = found(&scanned, &root.join("note.txt"));
        let alias = found(&scanned, &root.join("alias.txt"));

        assert_eq!((note.kind, note.modified, note.changed, note.accessed), (EntryKind::File, at(10), at(20), at(30)));
        assert_eq!((alias.kind, alias.changed, alias.accessed), (EntryKind::Symlink, at(21), at(31)));
    }
}
