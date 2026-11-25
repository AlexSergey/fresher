//! Tests of loading the cache and gathering the change set.

use std::fmt::Write as _;
use std::path::Path;

use fresher_core::{ChangeJournal, FullScanReason, Timestamp};

use crate::matcher::testing::set_of;
use crate::rescan::change_set::{ChangeSet, Gather, RESCAN_LIMIT_PERCENT};
use crate::rescan::load::Cached;
use crate::rescan::testing::{Rig, TAKEN};
use crate::tree::node::NodeIndex;

/// A tree of 20 folders: `a/b/c` with a file, `d` with a file, and 16 empty folders.
pub(crate) fn twenty_folders() -> String {
    let mut toml = String::from("[[file]]\npath = \"a/b/c/f\"\nsize = 10\n[[file]]\npath = \"d/g\"\nsize = 20\n");
    for index in 0..16 {
        let _ = write!(toml, "[[dir]]\npath = \"x{index:02}\"\n");
    }
    toml
}

fn rig() -> Rig {
    let rig = Rig::new(&twenty_folders(), set_of(&[]));
    rig.scan();
    rig
}

fn gather(cached: &Cached, journal: &dyn ChangeJournal) -> Result<ChangeSet, FullScanReason> {
    let cancel = fresher_core::CancelToken::new();
    let with = Gather {
        journal,
        now: Timestamp::from_secs(TAKEN + 60),
        sibling_markers: false,
        limit_percent: RESCAN_LIMIT_PERCENT,
        places: &[],
        cancel: &cancel,
    };
    ChangeSet::gather(cached, with)
}

fn node(cached: &Cached, path: &Path) -> NodeIndex {
    cached.tree.lookup(path).unwrap()
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{FullScanReason, JournalGap, NotScanned, ScanCache, ScopeKey, Timestamp, VolumeId};
    use fresher_testing::FakeChangeJournal;

    use super::{gather, rig, twenty_folders};
    use crate::matcher::testing::{interest, set_of};
    use crate::rescan::change_set::{ChangeSet, Gather, MAX_POSITION_AGE_SECS, RESCAN_LIMIT_PERCENT};
    use crate::rescan::load::Cached;
    use crate::rescan::testing::{Rig, TAKEN, journal_gap, journal_of};
    use crate::walk::std_reader::StdReader;

    #[test]
    fn no_cache_means_a_full_scan() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));

        assert_eq!(Cached::load(&*rig.cache, &rig.scope()).err(), Some(FullScanReason::NoCache));
    }

    #[test]
    fn a_broken_cache_means_a_full_scan() {
        let rig = rig();
        let scope = rig.scope();
        rig.cache.save(scope.key, b"not a body").unwrap();

        assert_eq!(Cached::load(&*rig.cache, &scope).err(), Some(FullScanReason::BrokenCache));
    }

    #[test]
    fn other_interests_mean_a_full_scan() {
        let mut rig = rig();
        rig.interests =
            set_of(&[interest(fresher_core::Pattern::Extension("log".into()), fresher_core::Claim::Item, &[])]);

        let error = Cached::load(&*rig.cache, &rig.scope()).err();

        assert_eq!(error, Some(FullScanReason::OtherFingerprint));
    }

    #[test]
    fn other_roots_mean_a_full_scan() {
        let mut rig = rig();
        let body = rig.cache.load(rig.scope().key).unwrap().unwrap();
        rig.roots = vec![rig.path("a")];
        let scope = rig.scope();
        rig.cache.save(scope.key, &body).unwrap();

        assert_eq!(Cached::load(&*rig.cache, &scope).err(), Some(FullScanReason::OtherRoots));
    }

    #[test]
    fn another_avoid_list_means_a_full_scan() {
        let mut rig = rig();
        rig.avoid = vec![rig.path("d")];

        let error = Cached::load(&*rig.cache, &rig.scope()).err();

        assert_eq!(error, Some(FullScanReason::OtherAvoidList));
    }

    #[test]
    fn a_rescan_with_another_avoid_list_than_the_cache_reads_every_folder() {
        let mut rig = Rig::new(&twenty_folders(), set_of(&[]));
        rig.avoid = vec![rig.path("d")];
        rig.scan();
        rig.avoid = Vec::new();

        let rescanned = rig.rescan(Arc::new(StdReader::new()), journal_of(&[], &[]));

        let summary = rescanned.summary.unwrap();
        assert_eq!(summary.fallback, Some(FullScanReason::OtherAvoidList));
        assert_eq!(summary.kind, fresher_core::ScanKind::Full);
        assert_eq!(summary.not_scanned.get(&NotScanned::Avoided), None);
    }

    #[test]
    fn another_volume_means_a_full_scan() {
        let rig = rig();
        let mut scope = rig.scope();
        scope.roots[0].volume = VolumeId::new(2);

        assert_eq!(Cached::load(&*rig.cache, &scope).err(), Some(FullScanReason::OtherVolume));
    }

    #[test]
    fn a_volume_without_a_journal_means_a_full_scan() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(FakeChangeJournal::default()));
        rig.walk(&scanner, fresher_core::WalkMode::Full);

        let error = Cached::load(&*rig.cache, &rig.scope()).err();

        assert_eq!(error, Some(FullScanReason::JournalGap(JournalGap::NoJournal)));
    }

    #[test]
    fn a_journal_gap_means_a_full_scan() {
        let rig = rig();
        let cached = rig.cached();
        let gaps = [
            JournalGap::NoJournal,
            JournalGap::Reset,
            JournalGap::Dropped,
            JournalGap::Wrapped,
            JournalGap::RootChanged,
            JournalGap::TooOld,
            JournalGap::Cancelled,
            JournalGap::Failed,
        ];

        for gap in gaps {
            assert_eq!(gather(&cached, &journal_gap(gap)), Err(FullScanReason::JournalGap(gap)), "{gap:?}");
        }
    }

    #[test]
    fn a_position_older_than_two_weeks_is_too_old() {
        let rig = rig();
        let cached = rig.cached();
        let journal = journal_of(&[], &[]);
        let cancel = fresher_core::CancelToken::new();
        let now = Timestamp::from_secs(TAKEN + MAX_POSITION_AGE_SECS + 1);

        let set = ChangeSet::gather(
            &cached,
            Gather {
                journal: &journal,
                now,
                sibling_markers: false,
                limit_percent: RESCAN_LIMIT_PERCENT,
                places: &[],
                cancel: &cancel,
            },
        );

        assert_eq!(set, Err(FullScanReason::JournalGap(JournalGap::TooOld)));
        assert_eq!(journal.calls(), vec![]);
    }

    #[test]
    fn a_root_replaced_under_the_same_path_means_a_full_scan() {
        let rig = rig();
        let mut scope = rig.scope();
        scope.roots[0].identity.file_id += 1;

        let error = Cached::load(&*rig.cache, &scope).err();

        assert_eq!(error, Some(FullScanReason::JournalGap(JournalGap::RootChanged)));
    }

    #[test]
    fn too_many_changes_mean_a_full_scan() {
        let rig = rig();
        let changed: Vec<_> = (0..11).map(|index| rig.path(&format!("x{index:02}"))).collect();

        assert_eq!(gather(&rig.cached(), &journal_of(&changed, &[])), Err(FullScanReason::TooManyChanges));
    }

    #[test]
    fn a_change_outside_the_roots_is_ignored() {
        let rig = rig();
        let outside = rig.root.parent().unwrap().join("elsewhere");

        let set = gather(&rig.cached(), &journal_of(&[outside.clone(), "/".into()], &[outside])).unwrap();

        assert_eq!(set, ChangeSet::default());
    }

    #[test]
    fn a_scope_key_of_another_scope_finds_nothing() {
        let rig = rig();
        let mut scope = rig.scope();
        scope.key = ScopeKey::new(scope.key.get() ^ 1);

        assert_eq!(Cached::load(&*rig.cache, &scope).err(), Some(FullScanReason::NoCache));
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::WalkMode;

    use super::{gather, node, rig, twenty_folders};
    use crate::codec::decode::decode;
    use crate::codec::fit::fully_collapsed;
    use crate::matcher::testing::set_of;
    use crate::rescan::load::Cached;
    use crate::rescan::testing::{CallLog, LoggingJournal, LoggingReader, Refusing, Rig, journal_of};
    use crate::tree::node::Flag;
    use crate::tree::testing::canonical;
    use crate::walk::std_reader::StdReader;
    use crate::walk::tests::scan;

    #[test]
    fn a_changed_folder_is_placed_on_its_node_and_its_parent_is_read_too() {
        let rig = rig();
        let cached = rig.cached();

        let set = gather(&cached, &journal_of(&[rig.path("a/b")], &[])).unwrap();

        let expected = [node(&cached, &rig.path("a/b")), node(&cached, &rig.path("a"))];
        assert_eq!(set.reread, expected.into());
        assert!(set.rewalk.is_empty());
    }

    #[test]
    fn a_new_folder_is_placed_on_its_nearest_ancestor() {
        let rig = rig();
        let cached = rig.cached();

        let set = gather(&cached, &journal_of(&[rig.path("a/b/new/deeper")], &[])).unwrap();

        assert!(set.reread.contains(&node(&cached, &rig.path("a/b"))));
        assert_eq!(set.reread.len(), 2);
    }

    #[test]
    fn a_subtree_named_whole_is_walked_again() {
        let rig = rig();
        let cached = rig.cached();

        let set = gather(&cached, &journal_of(&[], &[rig.path("a/b")])).unwrap();

        assert_eq!(set.rewalk, [node(&cached, &rig.path("a/b"))].into());
        assert_eq!(set.reread, [node(&cached, &rig.path("a"))].into());
    }

    #[test]
    fn a_change_inside_a_collapsed_subtree_marks_the_collapsed_node() {
        let rig = rig();
        let Cached { tree, header } = rig.cached();
        let (tree, header) = decode(&fully_collapsed(&tree, &header)).unwrap();
        let cached = Cached { tree, header };
        let collapsed = node(&cached, &rig.path("a"));
        assert!(cached.tree.node(collapsed).unwrap().flags.has(Flag::Collapsed));

        let set = gather(&cached, &journal_of(&[rig.path("a/b/c")], &[])).unwrap();

        assert_eq!(set.rewalk, [collapsed].into());
        assert_eq!(set.reread, [node(&cached, &rig.root)].into());
    }

    #[test]
    fn a_collapsed_node_is_reported_as_collapsed_with_no_children() {
        use fresher_core::{ChildOrder, DiskState, NodeKind, StateId, Timestamp};

        let rig = rig();
        let Cached { tree, header } = rig.cached();
        let (tree, _) = decode(&fully_collapsed(&tree, &header)).unwrap();
        let state = crate::state::TreeState::new(StateId::new(1), Timestamp::from_secs(0), tree, Vec::new());
        let root = state.roots()[0];
        let page = state.children(root, ChildOrder::ByName, 0, 100).unwrap();
        let collapsed = page.nodes.iter().find(|node| node.name == "a").unwrap();

        assert_eq!(collapsed.kind, NodeKind::Collapsed);
        assert_eq!((collapsed.folders, collapsed.sizes.logical.bytes()), (2, 10));
        assert_eq!(state.children(collapsed.id, ChildOrder::ByName, 0, 100).unwrap().total, 0);
    }

    #[test]
    fn unreadable_folders_always_join_the_set() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        rig.scan_with(Arc::new(Refusing { refused: vec![rig.path("x03")] }));
        let cached = rig.cached();

        let set = gather(&cached, &journal_of(&[], &[])).unwrap();

        assert_eq!(set.rewalk, [node(&cached, &rig.path("x03"))].into());
        assert!(set.reread.is_empty());
    }

    #[test]
    fn the_position_is_taken_before_the_walk_reads() {
        let rig = Rig::new(&twenty_folders(), set_of(&[]));
        let log = Arc::new(CallLog::default());
        let scanner =
            rig.scanner(Arc::new(LoggingReader(Arc::clone(&log))), Arc::new(LoggingJournal(Arc::clone(&log))));

        rig.walk(&scanner, WalkMode::Full).summary.unwrap();

        let calls = log.calls();
        assert_eq!(calls.first(), Some(&"position"));
        assert_eq!(calls.iter().filter(|&&call| call == "position").count(), 1);
        assert!(calls.len() > 20);
    }

    #[test]
    fn a_committed_scan_is_saved_and_loads_back() {
        let rig = rig();
        let fresh = scan(Arc::new(StdReader::new()), &rig.root, &set_of(&[]), 1);

        let cached = rig.cached();

        assert_eq!(canonical(&cached.tree), canonical(&fresh.tree()));
        assert_eq!(cached.header.taken.secs(), super::TAKEN);
        assert_eq!(cached.header.roots[0].journal, Some(crate::rescan::testing::AT));
    }
}
