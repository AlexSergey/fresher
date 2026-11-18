//! Places walked outside the scan roots: walked as roots of their own, for their anchored interests only, never
//! entering another user's folder, always read again from the disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    Claim, EntryAccess, FolderIdentity, FolderListing, FolderReader, Interest, InterestSet, KnownFolder, ModuleId,
    Pattern, Place, ReadError,
};
use std::os::unix::ffi::OsStrExt as _;

use crate::rescan::testing::{Rig, journal_of};
use crate::walk::std_reader::StdReader;

/// A home and a shared temporary folder side by side: `home/` is the scan root, `tmp/` the place.
const SIDE_BY_SIDE: &str = r#"
[[file]]
path = "home/notes.txt"
[[file]]
path = "tmp/old.log"
modified = 1700000000
[[dir]]
path = "tmp/session"
[[file]]
path = "tmp/session/data.bin"
size = 4096
[[dir]]
path = "tmp/node_modules"
[[dir]]
path = "tmp/root-owned/inner"
[[file]]
path = "tmp/root-owned/inner/secret"
[[dir]]
path = "T/TemporaryItems/hidden"
[[file]]
path = "T/kept.txt"
[[dir]]
path = "outside/target"
[[file]]
path = "outside/target/far.bin"
[[symlink]]
path = "tmp/link"
target = "../outside/target"
"#;

/// The interests: every top-level item of both places, whole; every `node_modules` folder by name; the home's notes
/// (the fixture's root is the home anchor).
fn interests() -> InterestSet {
    let whole = |folder| Interest::new(Pattern::Under(folder, "*".into()), Claim::Whole, Vec::new()).unwrap();
    let by_name = Interest::new(Pattern::FolderName("node_modules".into()), Claim::Whole, Vec::new()).unwrap();
    let notes = Interest::new(Pattern::Under(KnownFolder::Home, "home/notes.txt".into()), Claim::Item, Vec::new());
    let mut set = InterestSet::new();
    let all = [whole(KnownFolder::SharedTemp), whole(KnownFolder::UserTemp), by_name, notes.unwrap()];
    set.add(ModuleId::new("temp").unwrap(), &all);
    set
}

/// The rig of [`SIDE_BY_SIDE`]: `home/` scanned, `tmp/` and `T/` walked as places.
fn rig() -> Rig {
    let mut rig = Rig::new(SIDE_BY_SIDE, interests());
    rig.roots = vec![rig.path("home")];
    rig.places = vec![
        Place { folder: KnownFolder::SharedTemp, path: rig.path("tmp") },
        Place { folder: KnownFolder::UserTemp, path: rig.path("T") },
    ];
    rig
}

/// The matches' paths relative to the rig's root, sorted.
fn matched(rig: &Rig, matches: &[fresher_core::Match]) -> Vec<String> {
    let relative = |path: &Path| path.strip_prefix(&rig.root).unwrap().to_string_lossy().into_owned();
    let mut paths: Vec<String> = matches.iter().map(|found| relative(&found.path)).collect();
    paths.sort();
    paths
}

/// The portable reader, reporting the folders of `others` as owned by another user (`root`).
#[derive(Debug)]
struct Disowning {
    others: Vec<PathBuf>,
}

impl FolderReader for Disowning {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        StdReader::new().identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        let access = StdReader::new().access(path)?;
        Ok(if self.others.iter().any(|other| other == path) {
            EntryAccess { owner: Some(0), ..access }
        } else {
            access
        })
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        let mut read = FolderListing::new();
        StdReader::new().read(path, expected, &mut read)?;
        out.clear();
        for (name, entry) in read.iter() {
            let mut entry = *entry;
            if self.others.iter().any(|other| *other == path.join(std::ffi::OsStr::from_bytes(name))) {
                entry.access.owner = Some(0);
            }
            out.push(name, entry)?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        StdReader::new().head(folder, name, limit)
    }
}

mod negative {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::Arc;

    use fresher_core::{Completeness, EntryKind, FullScanReason, NotScanned, Sizes, Unreadable, WalkMode};
    use fresher_testing::FakeChangeJournal;

    use super::{Disowning, matched, rescan, rig};
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_place_inside_a_scan_root_is_not_walked_twice() {
        let mut rig = rig();
        rig.roots = vec![rig.root.clone()];

        let walked = rig.scan();

        assert_eq!(walked.tree.unwrap().roots().len(), 1);
        let mut pairs: Vec<_> = walked.matches.iter().map(|found| (&found.path, found.interest)).collect();
        let all = pairs.len();
        pairs.dedup();
        assert_eq!(pairs.len(), all);
    }

    #[test]
    fn a_folder_of_another_user_in_a_place_is_not_entered_and_the_scan_stays_complete() {
        let rig = rig();
        let others = vec![rig.path("tmp/root-owned")];

        let walked = rig.scan_with(Arc::new(Disowning { others }));

        let summary = walked.summary.unwrap();
        assert!(summary.is_complete());
        assert_eq!(summary.not_scanned.get(&NotScanned::OthersFolder), Some(&1));
        let found = walked.matches.iter().find(|found| found.path == rig.path("tmp/root-owned")).unwrap();
        assert_eq!(found.sizes, Sizes { completeness: Completeness::Partial, ..Sizes::ZERO });
        let inside = found.inside.unwrap();
        assert!(inside.foreign && !inside.complete);
        assert!(!matched(&rig, &walked.matches).iter().any(|path| path.contains("inner")));
    }

    #[test]
    fn the_system_items_of_the_users_temporary_folder_are_not_opened_for_their_own_reason() {
        let rig = rig();

        let summary = rig.scan().summary.unwrap();

        assert_eq!(summary.not_scanned.get(&NotScanned::SystemItems), Some(&1));
        assert_eq!(summary.not_scanned.get(&NotScanned::Avoided), None);
        assert_eq!(summary.unreadable.get(&Unreadable::Guarded), None);
        assert!(summary.is_complete());
    }

    #[test]
    fn a_link_in_a_place_is_not_followed() {
        let rig = rig();

        let walked = rig.scan();

        let link = walked.matches.iter().find(|found| found.path == rig.path("tmp/link")).unwrap();
        assert_eq!(link.kind, EntryKind::Symlink);
        assert!(!matched(&rig, &walked.matches).iter().any(|path| path.starts_with("outside")));
    }

    #[test]
    fn only_the_interests_anchored_in_a_place_match_inside_it() {
        let rig = rig();

        let walked = rig.scan();

        let node_modules: Vec<usize> = walked
            .matches
            .iter()
            .filter(|found| found.path == rig.path("tmp/node_modules"))
            .map(|found| found.interest)
            .collect();
        assert_eq!(node_modules, [0]);
    }

    #[test]
    fn other_places_make_the_rescan_a_full_scan() {
        let mut rig = rig();
        rig.scan();
        rig.places.truncate(1);

        let kind = rescan(&rig).summary.unwrap().fallback;

        assert_eq!(kind, Some(FullScanReason::OtherRoots));
    }

    #[test]
    fn a_place_that_does_not_exist_is_not_scanned_and_the_scan_goes_on() {
        let rig = rig();
        rig.remove(&rig.path("T"));

        let walked = rig.scan();

        let summary = walked.summary.unwrap();
        assert!(summary.is_complete());
        assert_eq!(summary.not_scanned.get(&NotScanned::Unreadable(Unreadable::Failed)), Some(&1));
        assert!(matched(&rig, &walked.matches).contains(&"tmp/old.log".to_owned()));
    }

    #[test]
    fn a_place_that_cannot_be_read_is_not_scanned_and_the_scan_stays_complete() {
        let rig = rig();
        let tmp = rig.path("tmp");
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o000)).unwrap();

        let walked = rig.scan();
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)).unwrap();

        let summary = walked.summary.unwrap();
        assert!(summary.is_complete());
        assert_eq!(summary.not_scanned.get(&NotScanned::Unreadable(Unreadable::NoPermission)), Some(&1));
        assert_eq!(matched(&rig, &walked.matches), ["T/kept.txt", "home/notes.txt"]);
    }

    #[test]
    fn a_restore_whose_place_went_missing_still_restores_the_scan_roots() {
        let rig = rig();
        rig.scan();
        rig.remove(&rig.path("T"));

        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(FakeChangeJournal::default()));
        let restored = rig.walk(&scanner, WalkMode::Restore);

        assert!(restored.summary.is_ok());
        assert_eq!(matched(&rig, &restored.matches), ["home/notes.txt"]);
    }

    #[test]
    fn a_restore_offers_nothing_inside_a_place() {
        let rig = rig();
        rig.scan();

        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(FakeChangeJournal::default()));
        let restored = rig.walk(&scanner, WalkMode::Restore);

        assert!(restored.summary.is_ok());
        assert_eq!(matched(&rig, &restored.matches), ["home/notes.txt"]);
    }
}

mod positive {
    use std::time::{Duration, SystemTime};

    use fresher_core::{EntryKind, ScanKind, Timestamp};

    use super::{matched, rescan, rig};

    #[test]
    fn the_interests_of_a_place_match_inside_it_and_the_place_is_a_root_after_the_scan_roots() {
        let rig = rig();

        let walked = rig.scan();

        assert_eq!(
            matched(&rig, &walked.matches),
            [
                "T/kept.txt",
                "home/notes.txt",
                "tmp/link",
                "tmp/node_modules",
                "tmp/old.log",
                "tmp/root-owned",
                "tmp/session"
            ]
        );
        let tree = walked.tree.unwrap();
        let roots: Vec<_> = tree.roots().iter().map(|root| tree.path(root.node).unwrap()).collect();
        assert_eq!(roots, [rig.path("home"), rig.path("tmp"), rig.path("T")]);
        let session = walked.matches.iter().find(|found| found.path == rig.path("tmp/session")).unwrap();
        assert_eq!((session.kind, session.sizes.logical.bytes()), (EntryKind::Folder, 4096));
    }

    #[test]
    fn a_rescan_with_the_same_places_uses_the_cache_and_reads_the_places_again() {
        let rig = rig();
        rig.scan();
        let later = SystemTime::UNIX_EPOCH + Duration::from_secs(1_750_000_000);
        std::fs::File::options().write(true).open(rig.path("tmp/old.log")).unwrap().set_modified(later).unwrap();

        let walked = rescan(&rig);

        assert!(matches!(walked.summary.unwrap().kind, ScanKind::Rescan { .. }));
        let log = walked.matches.iter().find(|found| found.path == rig.path("tmp/old.log")).unwrap();
        assert_eq!(log.modified, Some(Timestamp::from_secs(1_750_000_000)));
    }
}

/// A rescan of `rig` whose journal reports no change since the first scan.
fn rescan(rig: &Rig) -> crate::rescan::testing::Walked {
    let journal = Arc::new(journal_of(&[], &[]));
    rig.walk(&rig.scanner(Arc::new(StdReader::new()), journal), fresher_core::WalkMode::Auto)
}
