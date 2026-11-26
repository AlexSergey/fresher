//! Tests of the check and the fallbacks, and the deterministic bars of the rescan on `SMALL`.

use std::path::{Path, PathBuf};

use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use fresher_core::{
    ByteSize, CancelToken, ChangeJournal, Changes, CheckDifference, CheckFailure, EntryAccess, FolderIdentity,
    FolderListing, FolderReader, FullScanReason, JournalGap, JournalPosition, RawEntry, ReadError, Sizes, VolumeId,
};
use fresher_testing::FakeChangeJournal;

use crate::matcher::testing::set_of;
use crate::rescan::testing::{LATER, Rig, VOLUME, Walked};
use crate::rescan::tests::twenty_folders;
use crate::walk::std_reader::StdReader;

/// The tree of 20 folders, scanned once: fewer folders than the sample, so the check reads every unchanged one.
fn rig() -> Rig {
    let rig = Rig::new(&twenty_folders(), set_of(&[]));
    rig.scan();
    rig
}

/// The check's failure of a rescan that fell back for it; `None` for any other end.
fn failure(rescan: &Walked) -> Option<CheckFailure> {
    match &rescan.summary.as_ref().unwrap().fallback {
        Some(FullScanReason::CheckFailed(failure)) => Some(failure.clone()),
        _ => None,
    }
}

/// The failure of the folder `folder` (inside the root) with `difference`, alone in the sample.
#[expect(clippy::unnecessary_wraps, reason = "the failure it is compared with is an option")]
fn failed(folder: &str, difference: CheckDifference) -> Option<CheckFailure> {
    Some(CheckFailure { folder: PathBuf::from(folder), difference, folders: 1 })
}

/// The portable reader, with every entry named in `broken` read as failed, and every file named in `clones` read as
/// a clone (group 1) with that private size.
#[derive(Debug, Default)]
struct Altered {
    broken: Vec<&'static str>,
    clones: Vec<(&'static str, u64)>,
}

impl FolderReader for Altered {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        StdReader::new().identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        StdReader::new().access(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        StdReader::new().read(path, expected, out)?;
        let entries: Vec<(Vec<u8>, RawEntry)> = out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        out.clear();
        for (name, mut entry) in entries {
            let named = |wanted: &str| name == wanted.as_bytes();
            if self.broken.iter().any(|&broken| named(broken)) {
                entry.error = Some(ReadError::Vanished);
            }
            if let Some(&(_, private)) = self.clones.iter().find(|&&(clone, _)| named(clone)) {
                let sizes = entry.sizes;
                entry.clone = Some(1);
                entry.sizes = Sizes::new(sizes.logical, sizes.allocated, ByteSize::new(private));
            }
            out.push(&name, entry)?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        StdReader::new().head(folder, name, limit)
    }
}

/// A journal that answers as `inner`, with `written` landing in the tree right after its first answer: a change made
/// after the rescan asked the journal, which the journal names only when asked again.
#[derive(Debug)]
struct WrittenAfter {
    inner: FakeChangeJournal,
    written: PathBuf,
    done: AtomicBool,
}

impl WrittenAfter {
    fn new(inner: FakeChangeJournal, written: PathBuf) -> Self {
        Self { inner, written, done: AtomicBool::new(false) }
    }
}

impl ChangeJournal for WrittenAfter {
    fn position(&self, volume: &VolumeId) -> Option<JournalPosition> {
        self.inner.position(volume)
    }

    fn changes_since(
        &self,
        volume: &VolumeId,
        since: &JournalPosition,
        cancel: &CancelToken,
    ) -> Result<Changes, JournalGap> {
        let answer = self.inner.changes_since(volume, since, cancel);
        if !self.done.swap(true, AtomicOrdering::SeqCst) {
            std::fs::write(&self.written, vec![0; 700]).unwrap();
        }
        answer
    }
}

/// A journal answering nothing, then `folders` when asked again, from `AT` to `LATER`.
fn journal_late(folders: &[PathBuf]) -> FakeChangeJournal {
    let changes = |folders: &[PathBuf]| Changes { folders: folders.to_vec(), subtrees: Vec::new(), end: LATER };
    FakeChangeJournal::default().at(VOLUME, LATER).answering(Ok(changes(&[]))).answering(Ok(changes(folders)))
}

mod negative {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{
        CancelToken, CheckDifference, Claim, DiskWalk, FoldedPart, FullScanReason, Pattern, ScanKind, WalkMode,
    };

    use super::{Altered, WrittenAfter, failed, failure, journal_late, rig};
    use crate::matcher::testing::{interest, set_of};
    use crate::rescan::testing::{Refusing, Rig, assert_same, journal_of};
    use crate::rescan::tests::twenty_folders;
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_change_the_journal_missed_fails_the_check() {
        let rig = rig();
        std::fs::write(rig.path("x03/missed.txt"), vec![0; 700]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x03", CheckDifference::Folded(FoldedPart::Files)));
        let tree = rescan.tree.as_ref().unwrap();
        assert_eq!(tree.sums(tree.lookup(&rig.path("x03")).unwrap()).unwrap().logical, 700);
    }

    #[test]
    fn a_failed_check_falls_back_to_a_full_scan_with_its_reason() {
        let rig = rig();
        std::fs::create_dir(rig.path("a/b/missed")).unwrap();
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let rescan = rig.rescan(Arc::clone(&reader) as _, journal_of(&[rig.path("d")], &[]));

        let summary = rescan.summary.as_ref().unwrap();
        assert_eq!(summary.kind, ScanKind::Full);
        assert_eq!(failure(&rescan), failed("a/b", CheckDifference::EntryAdded(PathBuf::from("missed"))));
        assert_eq!(reader.reads()[&rig.path("a/b/missed")], 1);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_change_the_journal_still_does_not_name_when_asked_again_fails_the_check() {
        let rig = rig();
        let journal = WrittenAfter::new(journal_late(&[rig.path("d")]), rig.path("x03/late.txt"));
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(journal));

        let rescan = rig.walk(&scanner, WalkMode::Auto);

        assert_eq!(failure(&rescan), failed("x03", CheckDifference::Folded(FoldedPart::Files)));
    }

    #[test]
    fn a_file_grown_unseen_fails_the_check_in_its_folded_bytes() {
        let rig = rig();
        std::fs::write(rig.path("d/g"), vec![0; 30]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("d", CheckDifference::Folded(FoldedPart::Logical)));
    }

    #[test]
    fn a_folder_gone_unseen_fails_the_check_where_it_was_listed() {
        let rig = rig();
        rig.remove(&rig.path("x05"));

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        let gone = failure(&rescan).unwrap();
        assert_eq!((gone.folder, gone.difference), (PathBuf::new(), CheckDifference::EntryMissing("x05".into())));
        assert_eq!(gone.folders, 2, "the root, and the folder that is no longer there to identify");
    }

    #[test]
    fn a_large_file_resized_unseen_fails_the_check_in_its_size() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        std::fs::write(rig.path("x07/big.bin"), vec![1; 2 << 20]).unwrap();
        rig.scan();
        std::fs::write(rig.path("x07/big.bin"), vec![1; 3 << 20]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x07", CheckDifference::Size("big.bin".into())));
    }

    #[test]
    fn a_large_file_replaced_by_a_folder_unseen_fails_the_check_in_its_kind() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        std::fs::write(rig.path("x07/big.bin"), vec![1; 2 << 20]).unwrap();
        rig.scan();
        rig.remove(&rig.path("x07/big.bin"));
        std::fs::create_dir(rig.path("x07/big.bin")).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x07", CheckDifference::Kind("big.bin".into())));
    }

    #[test]
    fn a_folder_that_cannot_be_read_again_fails_the_check() {
        let rig = rig();
        let reader = Refusing { refused: vec![rig.path("x09")] };

        let rescan = rig.rescan(Arc::new(reader), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x09", CheckDifference::NotRead));
    }

    #[test]
    fn an_entry_that_fails_unseen_fails_the_check() {
        let rig = rig();

        let rescan = rig.rescan(Arc::new(Altered { broken: vec!["g"], ..Altered::default() }), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("d", CheckDifference::Incomplete));
    }

    #[test]
    fn a_clone_whose_private_size_changed_unseen_fails_the_check() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        std::fs::write(rig.path("x04/clone.bin"), vec![1; 700]).unwrap();
        rig.scan_with(Arc::new(Altered { clones: vec![("clone.bin", 0)], ..Altered::default() }));
        // Its other copy, elsewhere, was removed: its blocks are now its own, and its folder moves no event.
        let alone = Altered { clones: vec![("clone.bin", 4096)], ..Altered::default() };

        let rescan = rig.rescan(Arc::new(alone), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x04", CheckDifference::Folded(FoldedPart::Allocated)));
    }

    #[test]
    fn a_folder_claimed_whole_by_a_new_marker_fails_the_check() {
        let marker = interest(Pattern::MarkerFile("Gemfile".into()), Claim::Whole, &[]);
        let rig = Rig::new(&twenty_folders(), set_of(&[marker]));
        rig.scan();
        std::fs::write(rig.path("x02/Gemfile"), b"").unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        assert_eq!(failure(&rescan), failed("x02", CheckDifference::Summed));
    }

    #[test]
    fn a_failed_check_never_lowers_the_progress_the_rescan_reported() {
        let rig = rig();
        std::fs::write(rig.path("x03/missed.txt"), vec![0; 700]).unwrap();
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(journal_of(&[], &[rig.path("d")])));
        let mut progress = Vec::new();

        let summary = scanner
            .walk(&rig.request(WalkMode::Auto), &CancelToken::new(), &mut |_| {}, &mut |seen| progress.push(seen))
            .unwrap();

        assert!(matches!(summary.fallback, Some(FullScanReason::CheckFailed(_))), "{:?}", summary.fallback);
        let fallen: Vec<_> = progress
            .windows(2)
            .filter(|pair| pair[1].entries < pair[0].entries || pair[1].bytes < pair[0].bytes)
            .collect();
        assert!(fallen.is_empty(), "{fallen:?}");
        assert_eq!(progress.last().map(|last| last.entries), Some(summary.entries));
    }

    #[test]
    fn a_full_scan_asked_by_the_user_reads_every_folder() {
        let rig = rig();
        let reader = Arc::new(CountingReader::new(StdReader::new()));
        let scanner = rig.scanner(Arc::clone(&reader) as _, Arc::new(journal_of(&[], &[])));

        let full = rig.walk(&scanner, WalkMode::Full);

        let summary = full.summary.unwrap();
        assert_eq!((summary.kind, summary.fallback), (ScanKind::Full, Some(FullScanReason::AskedByUser)));
        assert_eq!(reader.total(), 21);
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::WalkMode;

    use super::{WrittenAfter, journal_late, rig};
    use crate::rescan::check::{CHECK_SAMPLE, sample};
    use crate::rescan::testing::{Refusing, Rig, assert_same, journal_of};
    use crate::tree::node::{Flag, NodeIndex};
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;
    use crate::walk::tests::{interests, small_tree};

    #[test]
    fn an_honest_journal_passes_the_check() {
        let rig = rig();
        std::fs::write(rig.path("x03/told.txt"), vec![0; 700]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[rig.path("x03")], &[]));

        let summary = rescan.summary.as_ref().unwrap();
        assert_eq!(summary.kind, fresher_core::ScanKind::Rescan { changed_folders: 2 });
        assert_eq!(summary.fallback, None);
        assert_same(&rescan, &rig.full());
    }

    #[test]
    fn a_change_made_after_the_journal_answered_and_named_when_asked_again_passes_the_check() {
        let rig = rig();
        let journal = WrittenAfter::new(journal_late(&[rig.path("x03")]), rig.path("x03/late.txt"));
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(journal));

        let rescan = rig.walk(&scanner, WalkMode::Auto);

        let summary = rescan.summary.as_ref().unwrap();
        assert_eq!((summary.kind, &summary.fallback), (fresher_core::ScanKind::Rescan { changed_folders: 0 }, &None));
    }

    #[test]
    fn the_sample_is_the_same_for_the_same_seed() {
        let rig = Rig::over(small_tree().root(), interests());
        rig.scan();
        let tree = rig.cached().tree;
        let folders: Vec<NodeIndex> = (0..tree.len())
            .filter_map(NodeIndex::new)
            .filter(|&index| tree.node(index).is_some_and(|node| node.flags.has(Flag::Folder)))
            .collect();

        let first = sample(&tree, &folders, 7);

        assert_eq!(first.len(), CHECK_SAMPLE);
        assert_eq!(sample(&tree, &folders, 7), first);
        assert_ne!(sample(&tree, &folders, 8), first);
    }

    #[test]
    fn a_rescan_with_nothing_changed_reads_only_the_sample_and_the_unreadable() {
        let small = small_tree();
        let rig = Rig::over(small.root(), interests());
        rig.scan();
        let tree = rig.cached().tree;
        let mut folders: Vec<_> = (0..tree.len())
            .filter_map(NodeIndex::new)
            .filter(|&index| {
                tree.node(index).is_some_and(|node| node.flags.has(Flag::Folder) && node.parent != NodeIndex::NONE)
            })
            .filter(|&index| tree.children(index).next().is_none() && tree.hits(index).is_empty())
            .filter_map(|index| tree.path(index))
            .collect();
        folders.sort();
        let unreadable: Vec<_> = folders.iter().step_by(13).take(3).cloned().collect();
        rig.scan_with(Arc::new(Refusing { refused: unreadable.clone() }));
        let reader = Arc::new(CountingReader::new(Refusing { refused: unreadable.clone() }));

        let rescan = rig.rescan(Arc::clone(&reader) as _, journal_of(&[], &[]));

        assert_eq!(rescan.summary.unwrap().fallback, None);
        assert_eq!(reader.total(), CHECK_SAMPLE + unreadable.len());
        assert!(unreadable.iter().all(|folder| reader.reads()[folder] == 1));
    }

    #[test]
    fn a_rescan_reads_each_changed_folder_once() {
        let small = small_tree();
        let rig = Rig::over(small.root(), interests());
        rig.scan();
        let tree = rig.cached().tree;
        let mut changed: Vec<_> = (0..tree.len())
            .filter_map(NodeIndex::new)
            .filter(|&index| {
                tree.node(index).is_some_and(|node| node.flags.has(Flag::Folder) && !node.flags.has(Flag::Opaque))
            })
            .filter_map(|index| tree.path(index))
            .collect();
        changed.sort();
        let changed: Vec<_> = changed.into_iter().step_by(50).collect();
        let reader = Arc::new(CountingReader::new(StdReader::new()));

        let rescan = rig.rescan(Arc::clone(&reader) as _, journal_of(&changed, &[]));

        assert!(matches!(rescan.summary.unwrap().kind, fresher_core::ScanKind::Rescan { .. }));
        let reads = reader.reads();
        assert!(changed.iter().all(|folder| reads.get(folder) == Some(&1)), "a changed folder read twice or never");
        assert!(reads.values().all(|&count| count == 1));
    }
}
