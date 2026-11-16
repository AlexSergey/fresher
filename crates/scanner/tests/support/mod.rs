//! What the scanner's integration tests share: the interests, both walks, the fixture homes.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, Claim, DiskWalk, FolderReader, Interest, InterestSet, KnownFolder, Match, ModuleId, Pattern,
    VolumeFacts, VolumeId, WalkMode, WalkRequest, WalkSummary,
};
use fresher_scanner::{Scanner, ScannerOptions, known_folders_anchor};
use fresher_testing::{FakeChangeJournal, FakeKnownFolders, FakeVolumes, MemoryScanCache, NaiveWalk};

pub(crate) fn interest(pattern: Pattern, claim: Claim, siblings: &[&str]) -> Interest {
    Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
}

/// Every pattern, both claims, siblings and the checked tag.
pub(crate) fn every_interest() -> Vec<Interest> {
    vec![
        interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json", "yarn.lock"]),
        interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
        interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
        interest(Pattern::MarkerFile("Cargo.toml".into()), Claim::Item, &["Cargo.lock"]),
        interest(Pattern::Extension("json".into()), Claim::Item, &[]),
        interest(Pattern::Extension("dmg".into()), Claim::Whole, &[]),
        interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
        interest(Pattern::Under(KnownFolder::Home, "Library/Logs".into()), Claim::Whole, &[]),
        // The same caches through their known folder, and a known folder the fixture home does not have.
        interest(Pattern::Under(KnownFolder::AppCaches, "*".into()), Claim::Item, &[]),
        interest(Pattern::Under(KnownFolder::SystemApplications, "*".into()), Claim::Item, &[]),
    ]
}

pub(crate) fn set_of(interests: &[Interest]) -> InterestSet {
    let mut set = InterestSet::new();
    set.add(ModuleId::new("demo").unwrap(), interests);
    set
}

/// What both walks give, matches sorted by path and interest.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Walked {
    pub(crate) matches: Vec<Match>,
    pub(crate) entries: u64,
    pub(crate) unreadable: std::collections::BTreeMap<fresher_core::Unreadable, u64>,
}

/// Every match with its entry facts, sorted by path and interest.
fn walked(mut matches: Vec<Match>, summary: &WalkSummary) -> Walked {
    matches.sort_by(|left, right| (&left.path, left.interest).cmp(&(&right.path, right.interest)));
    Walked { matches, entries: summary.entries, unreadable: summary.unreadable.clone() }
}

fn request<'a>(roots: &'a [PathBuf], interests: &'a InterestSet, avoid: &'a [PathBuf]) -> WalkRequest<'a> {
    WalkRequest { roots, interests, avoid, mode: WalkMode::Full }
}

/// The reference walk of `root` (also the home folder).
pub(crate) fn reference(root: &Path, interests: &InterestSet, avoid: &[PathBuf]) -> Walked {
    let roots = [root.to_path_buf()];
    let mut matches = Vec::new();
    let summary = NaiveWalk::new(root)
        .walk(&request(&roots, interests, avoid), &CancelToken::new(), &mut |found| matches.push(found), &mut |_| {})
        .unwrap();
    walked(matches, &summary)
}

/// The scanner's walk of `root` (also the home folder) with `reader`.
pub(crate) fn scanner(
    reader: Arc<dyn FolderReader>,
    root: &Path,
    interests: &InterestSet,
    avoid: &[PathBuf],
) -> Walked {
    let volumes = FakeVolumes::default()
        .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
    let journal = Arc::new(FakeChangeJournal::default());
    let cache = Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20)));
    let anchor = known_folders_anchor(&FakeKnownFolders::home_layout(root));
    let scanner = Scanner::new(reader, Arc::new(volumes), journal, cache, anchor, ScannerOptions::default());
    let roots = [root.to_path_buf()];
    let mut matches = Vec::new();
    let summary = scanner
        .walk(&request(&roots, interests, avoid), &CancelToken::new(), &mut |found| matches.push(found), &mut |_| {})
        .unwrap();
    walked(matches, &summary)
}

pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/homes")
}

pub(crate) fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap()
}

/// A home whose matches cover every entry fact, all of the user who runs the test: `mine.json` (writable), a
/// read-only `read-only.json`, `closed/inside.json` in a read-only folder, a read-only folder `Caches` and
/// `locked.json`, locked by the user. Dropping it gives the modes and the flag back, so the folder can be removed.
#[cfg(target_os = "macos")]
pub(crate) struct LockedHome {
    _tree: fresher_testing::FixtureTree,
    pub(crate) root: PathBuf,
}

#[cfg(target_os = "macos")]
impl LockedHome {
    pub(crate) fn new() -> Self {
        let tree = fresher_testing::FixtureTree::from_toml_str(
            "[[file]]\npath = \"mine.json\"\n[[file]]\npath = \"read-only.json\"\n[[file]]\npath = \"closed/inside.json\"\n\
             [[file]]\npath = \"Caches/a\"\n[[file]]\npath = \"locked.json\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        let home = Self { _tree: tree, root };
        home.set_mode("read-only.json", 0o444);
        home.set_mode("closed", 0o555);
        home.set_mode("Caches", 0o555);
        assert!(chflags("uchg", &home.root.join("locked.json")), "chflags uchg");
        home
    }

    /// The interests that match every entry of the home.
    pub(crate) fn interests() -> InterestSet {
        set_of(&[
            interest(Pattern::Extension("json".into()), Claim::Item, &[]),
            interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
        ])
    }

    fn set_mode(&self, path: &str, mode: u32) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(self.root.join(path), std::fs::Permissions::from_mode(mode)).unwrap();
    }
}

#[cfg(target_os = "macos")]
impl Drop for LockedHome {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt as _;

        let _ = chflags("nouchg", &self.root.join("locked.json"));
        for path in ["read-only.json", "closed", "Caches"] {
            let _ = std::fs::set_permissions(self.root.join(path), std::fs::Permissions::from_mode(0o755));
        }
    }
}

/// Runs `chflags` with `flags` on `path`; `true` when it succeeded.
#[cfg(target_os = "macos")]
fn chflags(flags: &str, path: &Path) -> bool {
    let status = std::process::Command::new("/usr/bin/chflags").arg(flags).arg(path).status();
    status.is_ok_and(|status| status.success())
}

/// The facts of the match at `path`.
pub(crate) fn facts_of(walked: &Walked, path: &Path) -> fresher_core::EntryFacts {
    walked.matches.iter().find(|found| found.path == path).map(|found| found.facts).unwrap()
}
