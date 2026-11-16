//! Tests of the check and the fallbacks, and the deterministic bars of the rescan on `SMALL`.

use crate::matcher::testing::set_of;
use crate::rescan::testing::Rig;
use crate::rescan::tests::twenty_folders;

/// The tree of 20 folders, scanned once: fewer folders than the sample, so the check reads every unchanged one.
fn rig() -> Rig {
    let rig = Rig::new(&twenty_folders(), set_of(&[]));
    rig.scan();
    rig
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{FullScanReason, ScanKind, WalkMode};

    use super::rig;
    use crate::rescan::testing::{assert_same, journal_of};
    use crate::walk::counting::CountingReader;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_change_the_journal_missed_fails_the_check() {
        let rig = rig();
        std::fs::write(rig.path("x03/missed.txt"), vec![0; 700]).unwrap();

        let rescan = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        let summary = rescan.summary.as_ref().unwrap();
        assert_eq!(summary.fallback, Some(FullScanReason::CheckFailed));
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
        assert_eq!((summary.kind, summary.fallback), (ScanKind::Full, Some(FullScanReason::CheckFailed)));
        assert_eq!(reader.reads()[&rig.path("a/b/missed")], 1);
        assert_same(&rescan, &rig.full());
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

    use super::rig;
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
        assert_eq!((summary.kind, summary.fallback), (fresher_core::ScanKind::Rescan { changed_folders: 2 }, None));
        assert_same(&rescan, &rig.full());
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
