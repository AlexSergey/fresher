//! What a walk saw of each anchored interest's place: the same for a full walk, a rescan and a restore of the same
//! tree.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{Claim, Interest, InterestSet, KnownFolder, ModuleId, Pattern, PlaceSeen, WalkMode};

use crate::rescan::testing::{Rig, journal_of};
use crate::walk::std_reader::StdReader;

/// A home whose caches hold one app's folder and which has no logs folder.
const HOME: &str = r#"
[[file]]
path = "Library/Caches/com.example.App/data.bin"
size = 4096
[[file]]
path = "Library/Preferences/com.example.App.plist"
"#;

/// Every item of the caches (whole) and of the logs, and the preferences named by their full path.
fn interests() -> InterestSet {
    let under =
        |relative: &str, claim| Interest::new(Pattern::Under(KnownFolder::Home, relative.into()), claim, vec![]);
    let all = [
        under("Library/Caches/*", Claim::Whole).unwrap(),
        under("Library/Logs/*", Claim::Whole).unwrap(),
        under("Library/*/com.example.App.plist", Claim::Item).unwrap(),
    ];
    let mut set = InterestSet::new();
    set.add(ModuleId::new("demo").unwrap(), &all);
    set
}

/// The places of the last walk of `walked`.
fn places(walked: &crate::rescan::testing::Walked) -> BTreeMap<usize, PlaceSeen> {
    walked.summary.as_ref().unwrap().places.clone()
}

/// A folder's permissions set for a test, given back on drop, even when the test fails.
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

mod negative {
    use super::*;

    #[test]
    fn a_rescan_after_the_place_became_unreadable_sees_it_unread() {
        let rig = Rig::new(HOME, interests());
        rig.scan();
        let caches = rig.path("Library/Caches");
        let _locked = Mode::set(caches.clone(), 0o000);

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[caches], &[]));

        assert_eq!(places(&rescan), [(0, PlaceSeen::Unread), (1, PlaceSeen::Absent)].into());
    }
}

mod positive {
    use super::*;

    #[test]
    fn a_full_walk_sees_the_place_read_and_the_missing_one_absent() {
        let rig = Rig::new(HOME, interests());

        let walked = rig.scan();

        assert_eq!(places(&walked), [(0, PlaceSeen::Read), (1, PlaceSeen::Absent)].into());
    }

    #[test]
    fn rescans_and_a_restore_of_an_unchanged_tree_see_what_the_full_walk_saw() {
        let rig = Rig::new(HOME, interests());
        let full = places(&rig.scan());
        let caches = rig.path("Library/Caches");

        let copied = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));
        let read_again = rig.rescan(Arc::new(StdReader::new()), journal_of(&[caches], &[]));
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(journal_of(&[], &[])));
        let restored = rig.walk(&scanner, WalkMode::Restore);

        assert_eq!(copied.summary.as_ref().unwrap().fallback, None);
        assert_eq!(read_again.summary.as_ref().unwrap().fallback, None);
        assert_eq!(places(&copied), full);
        assert_eq!(places(&read_again), full);
        assert_eq!(places(&restored), full);
    }
}
