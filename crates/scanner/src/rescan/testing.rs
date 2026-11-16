//! Test helpers of the rescan: a fixture tree scanned once into a cache in memory, the scope it was scanned for,
//! scripted journals and readers that refuse or record.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use fresher_core::{
    ByteSize, CancelToken, ChangeJournal, Changes, DiskWalk, FolderIdentity, FolderListing, FolderReader, InterestSet,
    JournalGap, JournalPosition, Match, ReadError, VolumeId, WalkMode, WalkRequest, WalkSummary,
};
use fresher_testing::{FakeChangeJournal, FakeClock, FixtureTree, MemoryScanCache};

use crate::codec::testing::snapshot;
use crate::rescan::Scope;
use crate::rescan::load::{Cached, Use};
use crate::scanner::{Scanner, ScannerOptions, home_anchor};
use crate::tree::arena::Tree;
use crate::walk::roots::tests::one_volume;
use crate::walk::std_reader::StdReader;

/// The volume of every fixture (`one_volume`).
pub(crate) const VOLUME: VolumeId = VolumeId::new(1);
/// Where the fake journal stands when the first scan is taken.
pub(crate) const AT: JournalPosition = JournalPosition { journal: 7, cursor: 100 };
/// Where it stands after the scripted changes.
pub(crate) const LATER: JournalPosition = JournalPosition { journal: 7, cursor: 200 };
/// When the first scan is taken.
pub(crate) const TAKEN: u64 = 1_735_689_600;

/// A fixture tree, the cache its scans are kept in, and what they are asked for.
pub(crate) struct Rig {
    _fixture: Option<FixtureTree>,
    /// Where removed entries go: the tests never delete (docs/restrictions/deletion-safety.md), they move out.
    graveyard: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) cache: Arc<MemoryScanCache>,
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) interests: InterestSet,
    pub(crate) avoid: Vec<PathBuf>,
    pub(crate) roots: Vec<PathBuf>,
}

/// What one walk gave.
#[derive(Debug)]
pub(crate) struct Walked {
    pub(crate) matches: Vec<Match>,
    pub(crate) summary: Result<WalkSummary, fresher_core::WalkError>,
    /// The tree of the walk, before it was committed.
    pub(crate) tree: Option<Tree>,
}

impl Rig {
    /// The fixture of `toml`, scanned for `interests`; nothing scanned yet.
    pub(crate) fn new(toml: &str, interests: InterestSet) -> Self {
        let fixture = FixtureTree::from_toml_str(toml).unwrap();
        let root = std::fs::canonicalize(fixture.root()).unwrap();
        Self {
            _fixture: Some(fixture),
            graveyard: tempfile::tempdir().unwrap(),
            roots: vec![root.clone()],
            root,
            cache: Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20))),
            clock: Arc::new(FakeClock::at(TAKEN)),
            interests,
            avoid: Vec::new(),
        }
    }

    /// The tree at `root`, which the rig never changes, scanned for `interests`; nothing scanned yet.
    pub(crate) fn over(root: &Path, interests: InterestSet) -> Self {
        let root = std::fs::canonicalize(root).unwrap();
        Self {
            _fixture: None,
            graveyard: tempfile::tempdir().unwrap(),
            roots: vec![root.clone()],
            root,
            cache: Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20))),
            clock: Arc::new(FakeClock::at(TAKEN)),
            interests,
            avoid: Vec::new(),
        }
    }

    /// Takes `path` out of the tree by moving it to a folder outside it.
    pub(crate) fn remove(&self, path: &Path) {
        let taken = std::fs::read_dir(self.graveyard.path()).unwrap().count();
        std::fs::rename(path, self.graveyard.path().join(taken.to_string())).unwrap();
    }

    /// The path of `relative` under the root.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        if relative.is_empty() { self.root.clone() } else { self.root.join(relative) }
    }

    /// A scanner over this rig's cache and clock, reading with `reader` and asking `journal`, with 2 threads.
    pub(crate) fn scanner(&self, reader: Arc<dyn FolderReader>, journal: Arc<dyn ChangeJournal>) -> Scanner {
        let options = ScannerOptions { threads: 2.try_into().unwrap(), ..ScannerOptions::default() };
        let cache = Arc::clone(&self.cache);
        Scanner::new(reader, Arc::new(one_volume()), journal, cache, home_anchor(&self.root), options)
            .with_clock(Arc::clone(&self.clock) as _)
    }

    pub(crate) fn request(&self, mode: WalkMode) -> WalkRequest<'_> {
        WalkRequest { roots: &self.roots, interests: &self.interests, avoid: &self.avoid, mode }
    }

    /// Walks in `mode` with `scanner` and commits when the walk succeeds.
    pub(crate) fn walk(&self, scanner: &Scanner, mode: WalkMode) -> Walked {
        let mut matches = Vec::new();
        let summary =
            scanner.walk(&self.request(mode), &CancelToken::new(), &mut |found| matches.push(found), &mut |_| {});
        let tree =
            scanner.last.lock().unwrap_or_else(PoisonError::into_inner).as_ref().map(|walked| walked.tree.clone());
        if summary.is_ok() {
            scanner.commit(&[]).unwrap();
        }
        matches.sort_by(|left, right| (&left.path, left.interest).cmp(&(&right.path, right.interest)));
        Walked { matches, summary, tree }
    }

    /// A full scan with the portable reader, the journal standing at `AT`, committed into the cache.
    pub(crate) fn scan(&self) -> Walked {
        self.scan_with(Arc::new(StdReader::new()))
    }

    pub(crate) fn scan_with(&self, reader: Arc<dyn FolderReader>) -> Walked {
        let scanner = self.scanner(reader, Arc::new(FakeChangeJournal::default().at(VOLUME, AT)));
        self.walk(&scanner, WalkMode::Full)
    }

    /// The scope a walk of this rig is of.
    pub(crate) fn scope(&self) -> Scope {
        let scanner = self.scanner(Arc::new(StdReader::new()), Arc::new(FakeChangeJournal::default()));
        scanner.prepare(&self.request(WalkMode::Auto)).unwrap().scope
    }

    /// The cached tree, loaded for a rescan.
    pub(crate) fn cached(&self) -> Cached {
        Cached::load(&*self.cache, &self.scope(), Use::Rescan).unwrap()
    }
}

/// A journal that answers `changes` once, from `AT` to `LATER`.
pub(crate) fn journal_of(folders: &[PathBuf], subtrees: &[PathBuf]) -> FakeChangeJournal {
    FakeChangeJournal::default().at(VOLUME, LATER).answering(Ok(Changes {
        folders: folders.to_vec(),
        subtrees: subtrees.to_vec(),
        end: LATER,
    }))
}

/// A journal that answers `gap`.
pub(crate) fn journal_gap(gap: JournalGap) -> FakeChangeJournal {
    FakeChangeJournal::default().at(VOLUME, LATER).answering(Err(gap))
}

/// A reader that refuses the folders under `refused` as guarded, and reads the rest with the portable reader.
#[derive(Debug)]
pub(crate) struct Refusing {
    pub(crate) refused: Vec<PathBuf>,
}

impl FolderReader for Refusing {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        StdReader::new().identify(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        if self.refused.iter().any(|refused| path == refused) {
            return Err(ReadError::Guarded);
        }
        StdReader::new().read(path, expected, out)
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        StdReader::new().head(folder, name, limit)
    }
}

/// A log shared by a reader and a journal: what was called, in order.
#[derive(Debug, Default)]
pub(crate) struct CallLog(Mutex<Vec<&'static str>>);

impl CallLog {
    pub(crate) fn push(&self, call: &'static str) {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).push(call);
    }

    pub(crate) fn calls(&self) -> Vec<&'static str> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

/// A reader that logs each read in a [`CallLog`].
#[derive(Debug)]
pub(crate) struct LoggingReader(pub(crate) Arc<CallLog>);

impl FolderReader for LoggingReader {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        StdReader::new().identify(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.0.push("read");
        StdReader::new().read(path, expected, out)
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        StdReader::new().head(folder, name, limit)
    }
}

/// A journal that logs each position asked in a [`CallLog`].
#[derive(Debug)]
pub(crate) struct LoggingJournal(pub(crate) Arc<CallLog>);

impl ChangeJournal for LoggingJournal {
    fn position(&self, _volume: &VolumeId) -> Option<JournalPosition> {
        self.0.push("position");
        Some(AT)
    }

    fn changes_since(
        &self,
        _volume: &VolumeId,
        _since: &JournalPosition,
        _cancel: &CancelToken,
    ) -> Result<Changes, JournalGap> {
        Err(JournalGap::Failed)
    }
}

impl Rig {
    /// A rescan (`Auto`) reading with `reader` and asking `journal`, committed into the cache.
    pub(crate) fn rescan(&self, reader: Arc<dyn FolderReader>, journal: FakeChangeJournal) -> Walked {
        let scanner = self.scanner(reader, Arc::new(journal));
        self.walk(&scanner, WalkMode::Auto)
    }

    /// A full scan of the disk as it is now, with a cache of its own.
    pub(crate) fn full(&self) -> Walked {
        let options = ScannerOptions { threads: 2.try_into().unwrap(), ..ScannerOptions::default() };
        let cache = Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20)));
        let journal = Arc::new(FakeChangeJournal::default());
        let scanner = Scanner::new(
            Arc::new(StdReader::new()),
            Arc::new(one_volume()),
            journal,
            cache,
            home_anchor(&self.root),
            options,
        )
        .with_clock(Arc::clone(&self.clock) as _);
        self.walk(&scanner, WalkMode::Full)
    }
}

/// Asserts that a rescan gave what a full scan of the same disk gives: the same tree node for node (times, groups
/// and roots included), the same matches and the same counts.
pub(crate) fn assert_same(rescan: &Walked, full: &Walked) {
    let (patched, scanned) = (rescan.tree.as_ref().unwrap(), full.tree.as_ref().unwrap());
    let (left, right) = (snapshot(patched), snapshot(scanned));
    let differences: Vec<_> =
        left.shapes.iter().zip(&right.shapes).filter(|(left, right)| left != right).take(3).collect();
    assert!(differences.is_empty(), "{differences:#?}");
    assert_eq!(left.shapes.len(), right.shapes.len());
    let times: Vec<_> = left
        .times_and_owners
        .iter()
        .zip(&right.times_and_owners)
        .enumerate()
        .filter(|(_, (l, r))| l != r)
        .take(3)
        .collect();
    assert!(times.is_empty(), "{times:?}");
    assert_eq!(left, right);
    assert_eq!(rescan.matches, full.matches);
    let (left, right) = (rescan.summary.as_ref().unwrap(), full.summary.as_ref().unwrap());
    assert_eq!(
        (left.entries, &left.unreadable, &left.not_scanned),
        (right.entries, &right.unreadable, &right.not_scanned)
    );
}
