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

/// Anchored interests that look for names in their folders, over the names of the random trees: an item and a whole
/// claim, `*` and a tag among the held names, two interests ending at the same folder.
pub(crate) fn held_interests() -> Vec<Interest> {
    let held = |path: &str, claim, names: &[&str]| {
        interest(Pattern::Under(KnownFolder::Home, path.into()), claim, &[])
            .holding(names.iter().map(|&name| (*name).into()).collect())
            .unwrap()
    };
    vec![
        held("Library/Caches/*", Claim::Item, &["CACHEDIR.TAG", "x.json", "a"]),
        held("a", Claim::Whole, &["b", "CACHEDIR.TAG", "Logs"]),
        held("*/b", Claim::Item, &["Caches", "CACHEDIR.TAG", "package.json"]),
        held("Library", Claim::Item, &["Caches", "Logs", "node_modules"]),
        held("Library", Claim::Whole, &["CACHEDIR.TAG"]),
    ]
}

/// Interests at one place, `Library/Caches`, over the names of the random cache trees: a shaped `*` claimed whole and as
/// an item, a plain `*`, literals beside them, and a shaped `*` below a plain one.
pub(crate) fn bundle_id_interests() -> Vec<Interest> {
    let at = |path: &str, claim| interest(Pattern::Under(KnownFolder::Home, path.into()), claim, &[]);
    let shaped = |path: &str, claim| at(path, claim).only_bundle_ids().unwrap();
    vec![
        shaped("Library/Caches/*", Claim::Whole),
        shaped("Library/Caches/*", Claim::Item),
        at("Library/Caches/*", Claim::Item),
        at("Library/Caches/JetBrains/*", Claim::Item),
        at("Library/Caches/com.example.App", Claim::Item),
        shaped("Library/Caches/*/*", Claim::Item),
    ]
}

/// Interests at one place, `Downloads`, over the names of the random download trees: a suffixed `*` claimed whole and
/// as an item, overlapping suffix sets (`.download` in two, `.gz` ending `.tar.gz`), a shaped and a plain `*` beside
/// them, a literal, and a suffixed `*` below a plain one.
pub(crate) fn suffixed_interests() -> Vec<Interest> {
    let at = |path: &str, claim| interest(Pattern::Under(KnownFolder::Downloads, path.into()), claim, &[]);
    let suffixed = |path: &str, claim, suffixes: &[&str]| {
        at(path, claim).only_suffixes(suffixes.iter().map(|&suffix| suffix.into()).collect()).unwrap()
    };
    vec![
        suffixed("*", Claim::Whole, &[".download", ".APP"]),
        suffixed("*", Claim::Item, &[".download"]),
        suffixed("*", Claim::Item, &[".gz"]),
        suffixed("*", Claim::Item, &[".tar.gz", ".zip"]),
        at("*", Claim::Item).only_bundle_ids().unwrap(),
        at("*", Claim::Item),
        at("a", Claim::Item),
        suffixed("*/*", Claim::Item, &[".app"]),
    ]
}

/// Anchored interests whose places lie on the ways the random trees build, at and below a folder claimed whole among
/// them (`Library/Logs`), and outside the root.
pub(crate) fn place_interests() -> Vec<Interest> {
    let at = |path: &str, claim| interest(Pattern::Under(KnownFolder::Home, path.into()), claim, &[]);
    vec![
        at("Library/Caches/*", Claim::Item),
        at("Library/Logs", Claim::Whole),
        at("Library/Logs/*", Claim::Item),
        at("Library/Logs/a/*", Claim::Item),
        at("a/b/*", Claim::Item),
        at("a/Caches/node_modules", Claim::Whole),
        at("Caches/*", Claim::Item),
        at("*/b/*", Claim::Item),
        interest(Pattern::Under(KnownFolder::SystemApplications, "*".into()), Claim::Item, &[]),
    ]
}

/// The folders of the random trees a walk may be asked to avoid.
pub(crate) const AVOIDABLE: [&str; 4] = ["Library", "a", "Library/Caches", "a/b"];

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
    /// What the walk saw of each anchored interest's place.
    pub(crate) places: std::collections::BTreeMap<usize, fresher_core::PlaceSeen>,
}

/// Every match with its entry facts, sorted by path and interest.
fn walked(mut matches: Vec<Match>, summary: &WalkSummary) -> Walked {
    matches.sort_by(|left, right| (&left.path, left.interest).cmp(&(&right.path, right.interest)));
    Walked { matches, entries: summary.entries, unreadable: summary.unreadable.clone(), places: summary.places.clone() }
}

fn request<'a>(roots: &'a [PathBuf], interests: &'a InterestSet, avoid: &'a [PathBuf]) -> WalkRequest<'a> {
    WalkRequest { roots, interests, avoid, places: &[], mode: WalkMode::Full }
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

/// The scanner over `root` (also the home folder) with `reader`, a cache in memory and a journal with no position.
pub(crate) fn scanner_over(reader: Arc<dyn FolderReader>, root: &Path) -> Scanner {
    let volumes = FakeVolumes::default()
        .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
    let journal = Arc::new(FakeChangeJournal::default());
    let cache = Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20)));
    let anchor = known_folders_anchor(&FakeKnownFolders::home_layout(root));
    Scanner::new(reader, Arc::new(volumes), journal, cache, anchor, ScannerOptions::default())
}

/// The scanner's walk of `root` (also the home folder) with `reader`.
pub(crate) fn scanner(
    reader: Arc<dyn FolderReader>,
    root: &Path,
    interests: &InterestSet,
    avoid: &[PathBuf],
) -> Walked {
    let scanner = scanner_over(reader, root);
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

/// A home of special entries, every one made in the test's own temporary folder: pipes named like the markers
/// `project/CACHEDIR.TAG` and `rust/Cargo.toml`, and a pipe `x/pipe` and a socket `x/socket` under an interest of
/// their own. The socket stays bound while the home lives.
pub(crate) struct SpecialHome {
    _tree: fresher_testing::FixtureTree,
    _socket: std::os::unix::net::UnixListener,
    pub(crate) root: PathBuf,
}

impl SpecialHome {
    pub(crate) fn new() -> Self {
        let tree = fresher_testing::FixtureTree::from_toml_str(
            "[[dir]]\npath = \"project\"\n[[dir]]\npath = \"rust\"\n[[dir]]\npath = \"x\"\n",
        )
        .unwrap();
        let root = canonical(tree.root());
        for pipe in ["project/CACHEDIR.TAG", "rust/Cargo.toml", "x/pipe"] {
            fresher_testing::make_pipe(&root.join(pipe)).unwrap();
        }
        let socket = root.join("x/socket");
        assert!(socket.as_os_str().len() < 104, "the socket's path fits its 104 bytes");
        let socket = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        Self { _tree: tree, _socket: socket, root }
    }

    /// Every interest, and one that matches anything in `x`.
    pub(crate) fn interests() -> InterestSet {
        let mut interests = every_interest();
        interests.push(interest(Pattern::Under(KnownFolder::Home, "x/*".into()), Claim::Item, &[]));
        set_of(&interests)
    }

    /// The paths of the matches, relative to the root.
    pub(crate) fn matched(&self, walked: &Walked) -> Vec<PathBuf> {
        walked.matches.iter().map(|found| found.path.strip_prefix(&self.root).unwrap().to_path_buf()).collect()
    }
}

/// 2019-01-01: the modification time an archive tool kept.
pub(crate) const OLD: u64 = 1_546_300_800;

/// A home of three folders claimed whole under `w`: `archive`, whose folders and files all carry the modification
/// time `OLD`; `locked`, holding a folder the walk cannot open; `pipes`, holding a named pipe. Dropping it opens the
/// locked folder again, so it can be removed.
pub(crate) struct TimesHome {
    _tree: fresher_testing::FixtureTree,
    pub(crate) root: PathBuf,
}

impl TimesHome {
    pub(crate) fn new() -> Self {
        let tree = fresher_testing::FixtureTree::from_toml_str(&format!(
            "[[file]]\npath = \"w/archive/a.log\"\nsize = 5\nmodified = {OLD}\n\
             [[file]]\npath = \"w/archive/sub/b.log\"\nsize = 7\nmodified = {OLD}\n\
             [[dir]]\npath = \"w/archive\"\nmodified = {OLD}\n[[dir]]\npath = \"w/archive/sub\"\nmodified = {OLD}\n\
             [[file]]\npath = \"w/locked/closed/c.log\"\n[[dir]]\npath = \"w/pipes\"\n",
        ))
        .unwrap();
        let root = canonical(tree.root());
        fresher_testing::make_pipe(&root.join("w/pipes/fifo")).unwrap();
        set_mode(&root.join("w/locked/closed"), 0o000);
        Self { _tree: tree, root }
    }

    /// One interest: every folder in `w`, claimed whole.
    pub(crate) fn interests() -> InterestSet {
        set_of(&[interest(Pattern::Under(KnownFolder::Home, "w/*".into()), Claim::Whole, &[])])
    }

    /// The inside of the match of `w/<name>`.
    pub(crate) fn inside(&self, walked: &Walked, name: &str) -> fresher_core::Inside {
        let path = self.root.join("w").join(name);
        walked.matches.iter().find(|found| found.path == path).and_then(|found| found.inside).unwrap()
    }

    /// The newest status change of `w/archive` and everything inside it, as `lstat` reads it.
    pub(crate) fn archive_status_change(&self) -> fresher_core::Timestamp {
        use std::os::unix::fs::MetadataExt as _;

        let mut newest = 0;
        let mut folders = vec![self.root.join("w/archive")];
        while let Some(folder) = folders.pop() {
            newest = newest.max(std::fs::symlink_metadata(&folder).unwrap().ctime());
            for entry in std::fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                let metadata = std::fs::symlink_metadata(&path).unwrap();
                newest = newest.max(metadata.ctime());
                if metadata.is_dir() {
                    folders.push(path);
                }
            }
        }
        fresher_core::Timestamp::from_secs(u64::try_from(newest).unwrap())
    }
}

impl Drop for TimesHome {
    fn drop(&mut self) {
        set_mode(&self.root.join("w/locked/closed"), 0o755);
    }
}

fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

const NAMES: [&str; 10] =
    ["a", "b", "node_modules", "Caches", "Library", "Logs", "x.json", "CACHEDIR.TAG", "package.json", "Cargo.toml"];

/// One thing to create: a kind (folder, file, link out, link in, hard link, tag, unreadable folder, pipe), a path of
/// names from `NAMES`, and a size, from which a file's times are drawn too.
pub(crate) type RandomItem = (u8, Vec<usize>, u64);

pub(crate) fn random_items() -> impl proptest::strategy::Strategy<Value = Vec<RandomItem>> {
    use proptest::prelude::*;

    prop::collection::vec((0_u8..8, prop::collection::vec(0_usize..10, 1..4), 0_u64..5_000_000), 0..40)
}

/// `items` after four tags, valid when their `sizes` are even, in the folders `held_interests` match: `Library`, `a`,
/// a cache, and `b/b`.
pub(crate) fn tagged(sizes: &[u64], items: Vec<RandomItem>) -> Vec<RandomItem> {
    let placed = [vec![4, 7], vec![0, 7], vec![4, 3, 1, 7], vec![1, 1, 7]];
    placed.into_iter().zip(sizes.iter().copied()).map(|(path, size)| (5, path, size)).chain(items).collect()
}

/// Builds `items` under `root` and returns the folders it locked (mode 0), to open again after the walks. A file's
/// modification and access times come from its size; a tag's access time is settled, so the walks' reads of it do not
/// move it.
pub(crate) fn random_tree(root: &Path, items: &[RandomItem]) -> Vec<PathBuf> {
    random_tree_of(root, &NAMES, items)
}

/// Names shaped as bundle ids and not, no two differing only in case.
const CACHE_NAMES: [&str; 7] =
    ["com.example.App", "com.example.App.plist", "org.example-tool_2.x", "JetBrains", ".DS_Store", "Yarn", "a"];

/// Names with and without the suffixes of [`suffixed_interests`], in either case, shaped as bundle ids and not, no two
/// differing only in case.
const DOWNLOAD_NAMES: [&str; 12] = [
    "movie.download",
    "Trip.ZIP.download",
    ".download",
    "Install macOS Sonoma.APP",
    "com.example.App.app",
    "a.TAR.gz",
    "x.gz",
    "Archive.ZIP",
    "a",
    "report.pdf",
    "download",
    "com.example.Tool",
];

/// Builds `items` under `root/Downloads` from `DOWNLOAD_NAMES`, as [`random_tree`] does.
pub(crate) fn random_download_tree(root: &Path, items: &[RandomItem]) -> Vec<PathBuf> {
    random_tree_of(&root.join("Downloads"), &DOWNLOAD_NAMES, items)
}

/// Builds `items` under `root/Library/Caches` from `CACHE_NAMES`, as [`random_tree`] does.
pub(crate) fn random_cache_tree(root: &Path, items: &[RandomItem]) -> Vec<PathBuf> {
    random_tree_of(&root.join("Library/Caches"), &CACHE_NAMES, items)
}

fn random_tree_of(root: &Path, names_of: &[&str], items: &[RandomItem]) -> Vec<PathBuf> {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt as _, symlink};
    use std::time::{Duration, SystemTime};

    let times = |size: u64| {
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_500_000_000 + size);
        fs::FileTimes::new().set_modified(modified).set_accessed(modified + Duration::from_secs(size % 1_000))
    };
    let mut files = Vec::new();
    let mut locked = Vec::new();
    for (kind, names, size) in items {
        let path = names.iter().fold(root.to_path_buf(), |path, &name| path.join(names_of[name % names_of.len()]));
        if path.symlink_metadata().is_ok() || beneath_a_link(root, &path) {
            continue;
        }
        let Some(parent) = path.parent() else { continue };
        if fs::create_dir_all(parent).is_err() {
            continue;
        }
        let made = match kind % 8 {
            0 => fs::create_dir(&path).is_ok(),
            1 => fs::File::create(&path)
                .and_then(|file| file.set_len(*size).and_then(|()| file.set_times(times(*size))))
                .is_ok(),
            2 => symlink("/usr", &path).is_ok(),
            3 => symlink(root.join("a"), &path).is_ok(),
            4 => files.first().is_some_and(|target: &PathBuf| fs::hard_link(target, &path).is_ok()),
            5 => fs::write(&path, if size % 2 == 0 { fresher_core::CACHEDIR_SIGNATURE } else { b"Signature: 0" })
                .and_then(|()| fresher_testing::settle_access_time(&path))
                .is_ok(),
            7 => fresher_testing::make_pipe(&path).is_ok(),
            _ => {
                fs::create_dir(&path).is_ok() && {
                    locked.push(path.clone());
                    true
                }
            }
        };
        if made && matches!(kind % 8, 1 | 5) {
            files.push(path);
        }
    }
    for folder in &locked {
        let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o000));
    }
    locked
}

/// Whether a folder between `root` and `path` is a link: an item there would be made through it, outside the tree
/// when the link points out (`/usr`).
fn beneath_a_link(root: &Path, path: &Path) -> bool {
    path.ancestors()
        .skip(1)
        .take_while(|folder| *folder != root)
        .any(|folder| folder.symlink_metadata().is_ok_and(|metadata| metadata.file_type().is_symlink()))
}
