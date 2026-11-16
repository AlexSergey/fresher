//! The data folder after ten scans of a generated tree with changes between them (plan 004/08/03): within its budget,
//! holding only the settings file and the cache's slot files. The scanner commits and saves as in the app; a scripted
//! journal reports the changed folder, so the scans after the first are rescans that load the saved cache.

#[cfg(test)]
mod run {
    use std::fs;
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::{
        ByteSize, CancelToken, ChangeJournal, Changes, Claim, DiskWalk, Interest, InterestSet, JournalGap,
        JournalPosition, ModuleId, Pattern, ScanKind, SettingsStore, StoredSettings, VolumeFacts, VolumeId, WalkMode,
        WalkRequest,
    };
    use fresher_scanner::{Scanner, ScannerOptions, StdReader, home_anchor};
    use fresher_store::{CachePolicy, DataFolder, FOOTPRINT_BUDGET, FileScanCache, FileSettingsStore, Footprint};
    use fresher_testing::{FakeVolumes, LargeTree, TreeSpec};

    const SCANS: usize = 10;

    /// A journal that always stands at the same position and reports the folder of the changes as changed throughout.
    #[derive(Debug)]
    struct ChangedFolder(PathBuf);

    impl ChangeJournal for ChangedFolder {
        fn position(&self, _volume: &VolumeId) -> Option<JournalPosition> {
            Some(JournalPosition { journal: 1, cursor: 1 })
        }

        fn changes_since(
            &self,
            _volume: &VolumeId,
            since: &JournalPosition,
            _cancel: &CancelToken,
        ) -> Result<Changes, JournalGap> {
            Ok(Changes { folders: Vec::new(), subtrees: vec![self.0.clone()], end: *since })
        }
    }

    /// The interests of a real scan's shape; no extension, so the tree stays within the cache's cap.
    fn interests() -> InterestSet {
        let interest = |pattern, claim, siblings: &[&str]| {
            Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
        };
        let mut set = InterestSet::new();
        set.add(
            ModuleId::new("footprint").unwrap(),
            &[
                interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
                interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, &[]),
            ],
        );
        set
    }

    /// Round `round` of changes: a new folder of `round` × 100 subfolders with a sparse 2 MiB file each (kept files), and
    /// the folder of two rounds before moved out of the scanned root into `away`.
    fn change(changes: &Path, away: &Path, round: usize) {
        let folder = changes.join(format!("round-{round}"));
        fs::create_dir_all(&folder).unwrap();
        for index in 0..round * 100 {
            let sub = folder.join(format!("sub-{index}"));
            fs::create_dir_all(&sub).unwrap();
            fs::File::create(sub.join("data.bin")).unwrap().set_len(2 << 20).unwrap();
        }
        if round >= 2 {
            let old = format!("round-{}", round - 2);
            fs::rename(changes.join(&old), away.join(&old)).unwrap();
        }
    }

    /// What ten scans of `spec` left in the data folder.
    pub(super) struct Outcome {
        pub(super) footprints: Vec<Footprint>,
        pub(super) kinds: Vec<ScanKind>,
        pub(super) names: Vec<String>,
        /// The length of the current cache file after each scan.
        pub(super) cache_files: Vec<u64>,
    }

    pub(super) fn ten_scans(spec: &TreeSpec) -> Outcome {
        let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
        let tree = LargeTree::build(spec, &under).unwrap();
        let (changes_dir, away) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let data = tempfile::tempdir().unwrap();
        let (root, changes) = (fs::canonicalize(tree.root()).unwrap(), fs::canonicalize(changes_dir.path()).unwrap());
        let folder = DataFolder::open(data.path()).unwrap();
        FileSettingsStore::new(folder.clone()).save(&StoredSettings::default()).unwrap();
        let volumes = FakeVolumes::default()
            .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
        let threads = std::thread::available_parallelism().map_or(1, NonZeroUsize::get).min(16);
        let options = ScannerOptions {
            threads: NonZeroUsize::new(threads).unwrap(),
            cache_body_limit: CachePolicy::DEFAULT.body_limit,
            ..ScannerOptions::default()
        };
        let scanner = Scanner::new(
            Arc::new(StdReader::new()),
            Arc::new(volumes),
            Arc::new(ChangedFolder(changes.clone())),
            Arc::new(FileScanCache::new(folder.clone())),
            home_anchor(&root),
            options,
        );
        let (roots, interests) = ([root, changes.clone()], interests());
        let request = WalkRequest { roots: &roots, interests: &interests, avoid: &[], mode: WalkMode::Auto };
        let mut outcome =
            Outcome { footprints: Vec::new(), kinds: Vec::new(), names: Vec::new(), cache_files: Vec::new() };
        for round in 0..SCANS {
            change(&changes, away.path(), round);
            let summary = scanner.walk(&request, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();
            let state = scanner.commit(&[]).unwrap();
            let round_root = state.node(state.roots()[1]).unwrap();
            let kept: usize = (round.saturating_sub(1)..=round).map(|kept| kept * 100 + 1).sum();
            assert_eq!(
                usize::try_from(round_root.folders).unwrap(),
                kept,
                "round {round}: the changes are in the tree"
            );
            outcome.kinds.push(summary.kind);
            outcome.footprints.push(folder.footprint().unwrap());
            outcome.cache_files.push(fs::metadata(data.path().join("scan-1.bin")).unwrap().len());
        }
        let mut names: Vec<String> = fs::read_dir(data.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into())
            .collect();
        names.sort();
        outcome.names = names;
        outcome
    }

    pub(super) fn check(outcome: &Outcome) {
        assert_eq!(outcome.kinds[0], ScanKind::Full);
        assert!(
            outcome.kinds[1..].iter().all(|kind| matches!(kind, ScanKind::Rescan { .. })),
            "every scan after the first loads the cache: {:?}",
            outcome.kinds
        );
        for footprint in &outcome.footprints {
            assert!(!footprint.over() && footprint.total <= FOOTPRINT_BUDGET, "{footprint:?}");
        }
        assert_eq!(outcome.names, ["scan-1.bin", "scan-1.previous.bin", "settings.json"]);
    }
}

#[cfg(test)]
mod positive {
    use fresher_testing::TreeSpec;

    use super::run::{check, ten_scans};

    #[test]
    fn ten_scans_of_the_small_tree_stay_within_the_budget() {
        check(&ten_scans(&TreeSpec::SMALL));
    }

    /// Run by hand: `cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture ten_scans_of_the_large`.
    #[test]
    #[ignore = "the large tree, run by hand"]
    fn ten_scans_of_the_large_tree_stay_within_the_budget() {
        let started = std::time::Instant::now();
        let outcome = ten_scans(&TreeSpec::LARGE);
        let totals: Vec<u64> = outcome.footprints.iter().map(|footprint| footprint.total.bytes()).collect();
        eprintln!("footprint after each scan (bytes): {totals:?}");
        eprintln!("cache file after each scan (bytes): {:?}", outcome.cache_files);
        eprintln!("scans: {:?}; files: {:?}; in {:.1?}", outcome.kinds, outcome.names, started.elapsed());
        check(&outcome);
    }
}
