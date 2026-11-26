//! Tests of the patch: a rescan after scripted changes gives what a full scan of the changed disk gives.

use std::fmt::Write as _;
use std::path::PathBuf;

use fresher_core::{CACHEDIR_SIGNATURE, Claim, KnownFolder, MarkerCheck, Match, Pattern};

use crate::matcher::testing::{interest, set_of};
use crate::rescan::testing::Rig;
use crate::walk::tests::interests;

/// A home with a whole dependency folder, two cache folders, a few plain folders and 16 empty ones.
fn home() -> Rig {
    let mut toml = String::from(
        "[[file]]\npath = \"app/package.json\"\nsize = 10\n\
         [[file]]\npath = \"app/node_modules/a/index.js\"\nsize = 100\n\
         [[file]]\npath = \"app/src/main.rs\"\nsize = 50\n\
         [[file]]\npath = \"Library/Caches/com.x/data.bin\"\nsize = 2000\n\
         [[file]]\npath = \"Library/Caches/com.y/big.bin\"\nsize = 3000000\n\
         [[file]]\npath = \"docs/a.txt\"\nsize = 10\n\
         [[file]]\npath = \"docs/old/b.txt\"\nsize = 10\n\
         [[file]]\npath = \"docs/old/deeper/c.txt\"\nsize = 10\n",
    );
    for index in 0..16 {
        let _ = write!(toml, "[[dir]]\npath = \"x{index:02}\"\n");
    }
    let rig = Rig::new(&toml, interests());
    rig.scan();
    rig
}

fn paths(rig: &Rig, relative: &[&str]) -> Vec<PathBuf> {
    relative.iter().map(|path| rig.path(path)).collect()
}

/// Sets the mode of `path`, and gives `0o755` back when dropped, so the fixture can be removed.
struct Mode(PathBuf);

impl Mode {
    fn set(path: PathBuf, mode: u32) -> Self {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        Self(path)
    }
}

impl Drop for Mode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt as _;

        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
    }
}

/// The facts of the match at `path` among `matches`.
fn facts_at(matches: &[Match], path: &std::path::Path) -> fresher_core::EntryFacts {
    matches.iter().find(|found| found.path == path).unwrap().facts
}

/// The tag files of the held home.
const TAGS: [&str; 2] = [".gradle/caches/CACHEDIR.TAG", ".cargo/registry/CACHEDIR.TAG"];

/// A home whose matched folders are checked for what they hold: Gradle's caches claimed whole (a valid tag and
/// `modules-2`), npm's cache as an item (`content-v2`, not yet `index-v5`) and Cargo's registry as an item (a valid
/// tag), each in a parent of its own; scanned once.
fn held_home() -> Rig {
    let toml = "[[file]]\npath = \".gradle/caches/modules-2/files-2.1/a.jar\"\nsize = 100\n\
                [[file]]\npath = \".gradle/caches/jars-9/b.jar\"\nsize = 50\n\
                [[file]]\npath = \".npm/_cacache/content-v2/sha512/x\"\nsize = 10\n\
                [[file]]\npath = \".cargo/registry/cache/c.crate\"\nsize = 20\n\
                [[file]]\npath = \"docs/a.txt\"\nsize = 10\n";
    let held = |path: &str, claim, names: &[&str]| {
        interest(Pattern::Under(KnownFolder::Home, path.into()), claim, &[])
            .holding(names.iter().map(|&name| name.into()).collect())
            .unwrap()
    };
    let rig = Rig::new(
        toml,
        set_of(&[
            held(".gradle/caches", Claim::Whole, &["CACHEDIR.TAG", "modules-2"]),
            held(".npm/_cacache", Claim::Item, &["index-v5", "content-v2"]),
            held(".cargo/registry", Claim::Item, &["CACHEDIR.TAG"]),
        ]),
    );
    for tag in TAGS {
        write_tag(&rig, tag, CACHEDIR_SIGNATURE);
    }
    rig.scan();
    rig
}

fn write_tag(rig: &Rig, relative: &str, content: &[u8]) {
    std::fs::write(rig.path(relative), content).unwrap();
    fresher_testing::settle_access_time(&rig.path(relative)).unwrap();
}

/// What the match at `relative` holds, and its marker check.
fn held_at(rig: &Rig, matches: &[Match], relative: &str) -> (Vec<String>, Option<MarkerCheck>) {
    let found = matches.iter().find(|found| found.path == rig.path(relative)).unwrap();
    (found.holds.clone(), found.marker)
}

/// Asserts that the walk was a rescan, not a full scan in its place.
fn assert_rescanned(walked: &crate::rescan::testing::Walked) {
    let kind = walked.summary.as_ref().unwrap().kind;
    assert!(matches!(kind, fresher_core::ScanKind::Rescan { .. }), "{kind:?}");
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|&name| name.into()).collect()
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{
        CancelToken, DiskWalk, FolderIdentity, FolderListing, FolderReader, ReadError, WalkError, WalkMode,
    };

    use fresher_core::MarkerCheck;

    use super::{Mode, assert_rescanned, facts_at, held_at, held_home, home, names, paths, write_tag};
    use crate::rescan::testing::{assert_same, journal_of};
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_folder_made_read_only_takes_its_entries_removability_after_a_rescan() {
        let rig = home();
        let cache = rig.path("Library/Caches/com.x");
        assert_eq!(facts_at(&rig.full().matches, &cache), fresher_core::EntryFacts::REMOVABLE);
        let _mode = Mode::set(rig.path("Library/Caches"), 0o555);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["Library/Caches"]), &[]));

        let facts = facts_at(&rescan.matches, &cache);
        assert_eq!(facts, fresher_core::EntryFacts { parent_writable: false, ..fresher_core::EntryFacts::REMOVABLE });
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_removed_subtree_leaves_no_node_and_no_bytes() {
        let rig = home();
        rig.remove(&rig.path("docs/old"));
        let journal = journal_of(&paths(&rig, &["docs", "docs/old", "docs/old/deeper"]), &[]);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal);

        let tree = rescan.tree.as_ref().unwrap();
        assert_eq!(tree.lookup(&rig.path("docs/old")), None);
        assert_eq!(tree.sums(tree.lookup(&rig.path("docs")).unwrap()).unwrap().logical, 10);
        assert_eq!(rescan.summary.as_ref().unwrap().fallback, None);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_folder_replaced_under_the_same_name_is_walked_anew() {
        let rig = home();
        rig.remove(&rig.path("docs/old"));
        std::fs::create_dir_all(rig.path("docs/old/new")).unwrap();
        std::fs::write(rig.path("docs/old/new/x.txt"), b"replaced").unwrap();
        let journal = journal_of(&paths(&rig, &["docs", "docs/old", "docs/old/new"]), &[]);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal);

        let tree = rescan.tree.as_ref().unwrap();
        assert_eq!(tree.lookup(&rig.path("docs/old/deeper")), None);
        assert!(tree.lookup(&rig.path("docs/old/new")).is_some());
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn an_unchanged_folder_is_not_read() {
        let rig = home();
        std::fs::write(rig.path("docs/new.txt"), b"new").unwrap();
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let rescan = rig.rescan(Arc::clone(&reader) as _, journal_of(&paths(&rig, &["docs"]), &[]));

        let reads = reader.reads();
        assert!(reads.keys().all(|folder| !folder.starts_with(rig.path("app/node_modules"))), "{reads:?}");
        assert!(reads.values().all(|&count| count == 1), "{reads:?}");
        assert_eq!(reads.len(), 26);
        assert_eq!(rescan.summary.unwrap().kind, fresher_core::ScanKind::Rescan { changed_folders: 2 });
    }

    /// Cancels its token on its first read.
    struct Cancelling {
        cancel: CancelToken,
    }

    impl FolderReader for Cancelling {
        fn identify(&self, path: &std::path::Path) -> Result<FolderIdentity, ReadError> {
            StdReader::new().identify(path)
        }

        fn access(&self, path: &std::path::Path) -> Result<fresher_core::EntryAccess, ReadError> {
            StdReader::new().access(path)
        }

        fn read(
            &self,
            path: &std::path::Path,
            expected: FolderIdentity,
            out: &mut FolderListing,
        ) -> Result<(), ReadError> {
            self.cancel.cancel();
            StdReader::new().read(path, expected, out)
        }

        fn head(&self, folder: &std::path::Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
            StdReader::new().head(folder, name, limit)
        }
    }

    #[test]
    fn a_cancelled_patch_returns_cancelled_and_commits_nothing() {
        let rig = home();
        let before = rig.cached().tree.len();
        std::fs::create_dir_all(rig.path("docs/new/a")).unwrap();
        let cancel = CancelToken::new();
        let reader = Arc::new(Cancelling { cancel: cancel.clone() });
        let scanner = rig.scanner(reader, Arc::new(journal_of(&paths(&rig, &["docs", "docs/new", "docs/new/a"]), &[])));

        let walked = scanner.walk(&rig.request(WalkMode::Auto), &cancel, &mut |_| {}, &mut |_| {});

        assert_eq!(walked, Err(WalkError::Cancelled));
        assert!(scanner.commit(&[]).is_err());
        assert_eq!(rig.cached().tree.len(), before);
    }

    #[test]
    fn a_tag_rewritten_invalid_in_an_item_folder_read_again_turns_it_invalid() {
        let rig = held_home();
        write_tag(&rig, ".cargo/registry/CACHEDIR.TAG", b"Signature: 0");

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &[".cargo/registry"]), &[]));

        let held = held_at(&rig, &rescan.matches, ".cargo/registry");
        assert_eq!(held, (names(&["CACHEDIR.TAG"]), Some(MarkerCheck::Invalid)));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_tag_gone_from_a_whole_folder_walked_again_is_no_longer_held() {
        let rig = held_home();
        rig.remove(&rig.path(".gradle/caches/CACHEDIR.TAG"));

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &[".gradle/caches"]), &[]));

        let held = held_at(&rig, &rescan.matches, ".gradle/caches");
        assert_eq!(held, (names(&["modules-2"]), Some(MarkerCheck::Invalid)));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_matched_folder_that_can_no_longer_be_read_holds_nothing_after_a_rescan() {
        let rig = held_home();
        let _mode = Mode::set(rig.path(".cargo/registry"), 0o000);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &[".cargo/registry"]), &[]));

        assert_eq!(held_at(&rig, &rescan.matches, ".cargo/registry"), (Vec::new(), None));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }
}

mod positive {
    use std::fs::OpenOptions;
    use std::sync::Arc;

    use fresher_core::{MarkerCheck, ScanCache, WalkMode};

    use super::{Mode, assert_rescanned, held_at, held_home, home, names, paths};
    use crate::codec::decode::decode;
    use crate::codec::fit::fully_collapsed;
    use crate::rescan::testing::{assert_same, journal_of};
    use crate::tree::node::Flag;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_new_file_is_counted_after_a_rescan() {
        let rig = home();
        std::fs::write(rig.path("docs/new.txt"), vec![0; 30]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["docs"]), &[]));

        let tree = rescan.tree.as_ref().unwrap();
        assert_eq!(tree.sums(tree.lookup(&rig.path("docs")).unwrap()).unwrap().logical, 60);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_file_grown_in_place_is_counted_after_a_rescan() {
        let rig = home();
        let file = OpenOptions::new().write(true).open(rig.path("Library/Caches/com.x/data.bin")).unwrap();
        file.set_len(5_000).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["Library/Caches/com.x"]), &[]));

        let tree = rescan.tree.as_ref().unwrap();
        assert_eq!(tree.sums(tree.lookup(&rig.path("Library/Caches/com.x")).unwrap()).unwrap().logical, 5_000);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_new_subtree_is_walked() {
        let rig = home();
        std::fs::create_dir_all(rig.path("docs/new/a/b")).unwrap();
        std::fs::write(rig.path("docs/new/a/b/deep.json"), b"{}").unwrap();
        std::fs::create_dir_all(rig.path("docs/new/node_modules/z")).unwrap();
        std::fs::write(rig.path("docs/new/node_modules/z/z.js"), b"z").unwrap();
        let changed = paths(&rig, &["docs", "docs/new", "docs/new/a", "docs/new/a/b"]);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&changed, &[]));

        assert!(rescan.matches.iter().any(|found| found.path == rig.path("docs/new/a/b/deep.json")));
        assert!(rescan.matches.iter().any(|found| found.path == rig.path("docs/new/node_modules")));
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_change_inside_a_collapsed_subtree_rereads_it() {
        let rig = home();
        let scope = rig.scope();
        let (tree, header) = decode(&rig.cache.load(scope.key).unwrap().unwrap()).unwrap();
        let collapsed = fully_collapsed(&tree, &header);
        let (tree, _) = decode(&collapsed).unwrap();
        assert!(tree.node(tree.lookup(&rig.path("docs")).unwrap()).unwrap().flags.has(Flag::Collapsed));
        rig.cache.save(scope.key, &collapsed).unwrap();
        let file = OpenOptions::new().write(true).open(rig.path("docs/old/deeper/c.txt")).unwrap();
        file.set_len(7_000).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["docs/old/deeper"]), &[]));

        let tree = rescan.tree.as_ref().unwrap();
        assert!(tree.lookup(&rig.path("docs/old/deeper")).is_some());
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_rescan_after_one_file_changes_moves_its_folders_newest_change() {
        let rig = home();
        let later = fresher_core::Timestamp::from_secs(4_102_444_800);
        let file = OpenOptions::new().write(true).open(rig.path("app/node_modules/a/index.js")).unwrap();
        file.set_modified(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(later.secs())).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["app/node_modules/a"]), &[]));

        let whole = rescan.matches.iter().find(|found| found.path == rig.path("app/node_modules")).unwrap();
        assert_eq!(whole.inside.unwrap().newest_change, Some(later));
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_held_name_added_inside_a_folder_whose_parent_did_not_change_appears_after_a_rescan() {
        let rig = held_home();
        std::fs::create_dir(rig.path(".npm/_cacache/index-v5")).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &[".npm/_cacache"]), &[]));

        assert_eq!(held_at(&rig, &rescan.matches, ".npm/_cacache"), (names(&["index-v5", "content-v2"]), None));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }

    /// An unreadable folder is walked again by every rescan, its parent copied: its kept hit gets the fresh mask.
    #[test]
    fn a_matched_folder_readable_again_under_a_parent_not_read_again_gets_what_it_holds() {
        let rig = held_home();
        let mode = Mode::set(rig.path(".cargo/registry"), 0o000);
        let before = rig.scan().matches;
        drop(mode);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(held_at(&rig, &before, ".cargo/registry"), (Vec::new(), None));
        let held = held_at(&rig, &rescan.matches, ".cargo/registry");
        assert_eq!(held, (names(&["CACHEDIR.TAG"]), Some(MarkerCheck::Valid)));
        assert_rescanned(&rescan);
        // Only what the folder holds is compared: its mode changed under a parent the journal did not name, so its
        // access stays the cached one.
        assert_eq!(held, held_at(&rig, &rig.full().matches, ".cargo/registry"));
    }

    /// The same for a folder claimed whole: its walk's opaque root reports what it holds.
    #[test]
    fn a_whole_folder_readable_again_under_a_parent_not_read_again_gets_what_it_holds() {
        let rig = held_home();
        let mode = Mode::set(rig.path(".gradle/caches"), 0o000);
        let before = rig.scan().matches;
        drop(mode);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(held_at(&rig, &before, ".gradle/caches"), (Vec::new(), None));
        let held = held_at(&rig, &rescan.matches, ".gradle/caches");
        assert_eq!(held, (names(&["CACHEDIR.TAG", "modules-2"]), Some(MarkerCheck::Valid)));
        assert_rescanned(&rescan);
        assert_eq!(held, held_at(&rig, &rig.full().matches, ".gradle/caches"));
    }

    #[test]
    fn a_matched_folder_copied_under_a_parent_read_again_keeps_what_it_holds() {
        let rig = held_home();
        let before = rig.scan().matches;
        for parent in [".gradle", ".npm", ".cargo"] {
            std::fs::write(rig.path(&format!("{parent}/new.txt")), b"new").unwrap();
        }

        let rescan =
            rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &[".gradle", ".npm", ".cargo"]), &[]));

        for folder in [".gradle/caches", ".npm/_cacache", ".cargo/registry"] {
            assert_eq!(held_at(&rig, &rescan.matches, folder), held_at(&rig, &before, folder), "{folder}");
        }
        assert_eq!(held_at(&rig, &rescan.matches, ".gradle/caches").1, Some(MarkerCheck::Valid));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_folder_copied_from_the_cache_keeps_what_it_holds() {
        let rig = held_home();
        let scanned = rig.scan();
        std::fs::write(rig.path("docs/new.txt"), b"new").unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["docs"]), &[]));

        assert_eq!(rescan.matches, scanned.matches);
        let held = held_at(&rig, &rescan.matches, ".cargo/registry");
        assert_eq!(held, (names(&["CACHEDIR.TAG"]), Some(MarkerCheck::Valid)));
        assert_rescanned(&rescan);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn matches_after_a_rescan_equal_those_of_a_full_scan() {
        let rig = home();
        std::fs::write(rig.path("app/node_modules/a/more.js"), vec![0; 900]).unwrap();
        std::fs::write(rig.path("app/package-lock.json"), b"{}").unwrap();

        let rescan =
            rig.rescan(Arc::new(StdReader::new()), journal_of(&paths(&rig, &["app/node_modules/a", "app"]), &[]));

        let whole = rescan.matches.iter().find(|found| found.path == rig.path("app/node_modules")).unwrap();
        assert_eq!(whole.sizes.logical.bytes(), 1_000);
        assert_eq!(whole.siblings, ["package.json"]);
        assert_same(&rescan, &rig.full());
        assert_eq!(
            rig.walk(&rig.scanner(Arc::new(StdReader::new()), Arc::new(journal_of(&[], &[]))), WalkMode::Auto).matches,
            rescan.matches
        );
    }
}

mod property {
    use std::fmt::Write as _;
    use std::fs::OpenOptions;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::{Claim, FullScanReason, Pattern};
    use fresher_testing::settle_access_time;
    use proptest::prelude::*;

    use crate::matcher::testing::{interest, set_of};
    use crate::rescan::testing::{Rig, assert_same, journal_of};
    use crate::walk::std_reader::StdReader;

    /// A small generator of our own: the same seed, the same tree and script.
    struct Random(u64);

    impl Random {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % bound.max(1) as u64).unwrap()
        }

        fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
            &items[self.below(items.len())]
        }
    }

    const FOLDER_NAMES: [&str; 6] = ["node_modules", "Caches", "src", "cache", "pkg", "build"];
    const FILE_NAMES: [&str; 6] = ["package.json", "Cargo.toml", "Cargo.lock", "CACHEDIR.TAG", "a.json", "data.bin"];
    const SIZES: [u64; 4] = [0, 700, 40_000, 3_000_000];

    /// A random home of about 50 folders and 80 files, with `Library/Caches` always there.
    fn random_home(random: &mut Random) -> String {
        let mut folders = vec![String::new(), "Library".to_owned(), "Library/Caches".to_owned()];
        for index in 0..50 {
            let parent = random.pick(&folders).clone();
            let name = format!("{}{index}", random.pick(&FOLDER_NAMES));
            let name = if random.below(4) == 0 { random.pick(&FOLDER_NAMES).to_string() } else { name };
            let path = if parent.is_empty() { name } else { format!("{parent}/{name}") };
            if !folders.contains(&path) {
                folders.push(path);
            }
        }
        let mut toml = String::new();
        let mut files = Vec::new();
        for index in 0..80 {
            let folder = random.pick(&folders).clone();
            let name =
                if random.below(3) == 0 { random.pick(&FILE_NAMES).to_string() } else { format!("f{index}.bin") };
            let path = if folder.is_empty() { name } else { format!("{folder}/{name}") };
            if files.contains(&path) || folders.contains(&path) {
                continue;
            }
            let _ = write!(toml, "[[file]]\npath = \"{path}\"\nsize = {}\n", random.pick(&SIZES));
            files.push(path);
        }
        for folder in folders.iter().filter(|folder| !folder.is_empty()) {
            let _ = write!(toml, "[[dir]]\npath = \"{folder}\"\n");
        }
        toml
    }

    /// Every folder and every file under `root`, sorted.
    fn listed(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let (mut folders, mut files) = (vec![root.to_path_buf()], Vec::new());
        let mut index = 0;
        while let Some(folder) = folders.get(index).cloned() {
            let mut entries: Vec<_> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().path()).collect();
            entries.sort();
            for entry in entries {
                if entry.symlink_metadata().unwrap().is_dir() { folders.push(entry) } else { files.push(entry) }
            }
            index += 1;
        }
        (folders, files)
    }

    /// Applies one random change and returns the folders a journal names for it. A file it writes has its access time
    /// settled, so neither scan's read of it moves the time (known issue `scanner-rescan-property-access-time`).
    fn change(rig: &Rig, random: &mut Random, step: usize) -> Vec<PathBuf> {
        let (folders, files) = listed(&rig.root);
        let parent = |path: &Path| path.parent().unwrap().to_path_buf();
        match random.below(6) {
            0 => {
                let folder = random.pick(&folders).clone();
                let name = random.pick(&FILE_NAMES);
                let path =
                    folder.join(if folder.join(name).exists() { format!("n{step}.json") } else { (*name).to_owned() });
                std::fs::write(&path, vec![1; 300]).unwrap();
                settle_access_time(&path).unwrap();
                vec![folder]
            }
            1 if !files.is_empty() => {
                let file = random.pick(&files).clone();
                rig.remove(&file);
                vec![parent(&file)]
            }
            2 if !files.is_empty() => {
                let file = random.pick(&files).clone();
                let handle = OpenOptions::new().write(true).open(&file).unwrap();
                handle.set_len(handle.metadata().unwrap().len() + 2_000_000).unwrap();
                settle_access_time(&file).unwrap();
                vec![parent(&file)]
            }
            3 if !files.is_empty() => {
                let file = random.pick(&files).clone();
                let folder = random.pick(&folders).clone();
                let target = folder.join(format!("moved{step}"));
                std::fs::rename(&file, &target).unwrap();
                vec![parent(&file), folder]
            }
            4 => {
                let folder = random.pick(&folders).clone();
                let top = folder.join(format!("{}{step}", random.pick(&FOLDER_NAMES)));
                let deep = top.join(random.pick(&FOLDER_NAMES));
                std::fs::create_dir_all(&deep).unwrap();
                let file = deep.join(random.pick(&FILE_NAMES));
                std::fs::write(&file, vec![2; 500]).unwrap();
                settle_access_time(&file).unwrap();
                vec![folder, top, deep]
            }
            _ => {
                let Some(folder) = folders.iter().skip(3).nth(random.below(folders.len())).cloned() else {
                    return Vec::new();
                };
                let mut named = vec![parent(&folder)];
                named.extend(listed(&folder).0);
                rig.remove(&folder);
                named
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        #[test]
        fn a_rescan_equals_a_full_scan(seed in any::<u64>()) {
            let mut random = Random(seed);
            let mut interests = vec![
                interest(Pattern::MarkerFile("Cargo.toml".into()), Claim::Item, &["Cargo.lock"]),
                interest(Pattern::FolderName("pkg".into()), Claim::Item, &["package.json"]),
            ];
            interests.extend((0..4).filter_map(|index| crate::walk::tests::interests().get(index).cloned()));
            let rig = Rig::new(&random_home(&mut random), set_of(&interests));
            rig.scan();
            let mut changed = Vec::new();
            for step in 0..=random.below(3) {
                changed.extend(change(&rig, &mut random, step));
            }

            let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&changed, &[]));

            let fallback = rescan.summary.as_ref().unwrap().fallback.clone();
            prop_assert!(fallback.is_none() || fallback == Some(FullScanReason::TooManyChanges), "{fallback:?}");
            assert_same(&rescan, &rig.full());
        }
    }
}
