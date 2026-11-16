//! What the scanner's integration tests share: the interests, both walks, the fixture homes.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, Claim, DiskWalk, FolderReader, Interest, InterestSet, Match, ModuleId, Pattern, VolumeFacts,
    VolumeId, WalkMode, WalkRequest, WalkSummary,
};
use fresher_scanner::{Scanner, ScannerOptions, home_anchor};
use fresher_testing::{FakeChangeJournal, FakeVolumes, MemoryScanCache, NaiveWalk};

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
        interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, &[]),
        interest(Pattern::UnderHome("Library/Logs".into()), Claim::Whole, &[]),
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
    let scanner = Scanner::new(reader, Arc::new(volumes), journal, cache, home_anchor(root), ScannerOptions::default());
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
