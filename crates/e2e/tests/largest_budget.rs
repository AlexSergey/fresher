//! The Largest screen's items within their budget (plan 035, design section 4: 50 ms and 5 MB) over a state the real
//! scanner committed on the generated `LARGE` tree, with the claims of a real scan's shape (every dependency tree and
//! every cache folder, as `storage_budget.rs`). Measured at the default smallest size (100 MB), at none (the tree's
//! files are sparse and kept as no node, so the engine's rule reads every folder and keeps nothing), and with a rule
//! that calls every folder from some depth an item, filling the list to its cap. Run by hand, in release:
//! `cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture items_of_the_large_tree` (the Files list) and
//! `... folders_of_the_large_tree` (the Folders list, plan 044: at no smallest size, once with every folder left alone,
//! once with none, so the query enters every folder).

#[cfg(test)]
mod positive {
    use std::collections::BTreeMap;
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::Instant;

    use fresher_core::{
        ByteSize, CancelToken, CategoryId, Claim, ClaimedPath, DiskState, DiskWalk, Interest, InterestSet, ItemRule,
        ItemVisit, KnownFolder, MAX_ITEMS, ModuleId, NodeKind, Pattern, Timestamp, VolumeFacts, VolumeId, WalkMode,
        WalkRequest,
    };
    use fresher_engine::bench::{FoldersBench, LargestBench};
    use fresher_scanner::{Scanner, ScannerOptions, StdReader, home_anchor};
    use fresher_testing::{FakeChangeJournal, FakeVolumes, LargeTree, MemoryScanCache, TreeSpec};

    const MOST_MILLIS: u128 = 50;
    const MOST_HEAP: usize = 5 * 1024 * 1024;
    /// The setting's default, `100-mb`, in decimal bytes.
    const DEFAULT_MIN: ByteSize = ByteSize::new(100_000_000);

    /// The interests of a real scan's shape, as the footprint test has them.
    fn interests() -> InterestSet {
        let interest = |pattern, claim, siblings: &[&str]| {
            Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
        };
        let mut set = InterestSet::new();
        set.add(
            ModuleId::new("bench").unwrap(),
            &[
                interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
                interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
            ],
        );
        set
    }

    /// The state the real scanner commits on `root`, every match claimed.
    fn committed(root: &Path) -> Arc<dyn DiskState> {
        let volumes = FakeVolumes::default()
            .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
        let threads = std::thread::available_parallelism().map_or(1, NonZeroUsize::get).min(16);
        let options = ScannerOptions { threads: NonZeroUsize::new(threads).unwrap(), ..ScannerOptions::default() };
        let scanner = Scanner::new(
            Arc::new(StdReader::new()),
            Arc::new(volumes),
            Arc::new(FakeChangeJournal::default()),
            Arc::new(MemoryScanCache::with_cap(ByteSize::new(1 << 30))),
            home_anchor(root),
            options,
        );
        let module = ModuleId::new("bench").unwrap();
        let deps = CategoryId::new(module, "bench.deps").unwrap();
        let caches = CategoryId::new(module, "bench.caches").unwrap();
        let (roots, interests) = ([root.to_path_buf()], interests());
        let request =
            WalkRequest { roots: &roots, interests: &interests, avoid: &[], places: &[], mode: WalkMode::Full };
        let mut claimed: BTreeMap<PathBuf, CategoryId> = BTreeMap::new();
        scanner
            .walk(
                &request,
                &CancelToken::new(),
                &mut |found| {
                    let category = if found.path.ends_with("node_modules") { deps } else { caches };
                    claimed.entry(found.path).or_insert(category);
                },
                &mut |_| {},
            )
            .unwrap();
        let claims: Vec<ClaimedPath> =
            claimed.into_iter().map(|(path, category)| ClaimedPath { path, category }).collect();
        scanner.commit(&claims).unwrap()
    }

    /// Enters the levels above `0` and calls every node at that depth or deeper an item: the tree keeps no small file
    /// as a node (its files are sparse), so the engine's rule finds nothing in it; deep enough, this fills the list to
    /// its cap, the most the items can weigh.
    struct FoldersFrom(u32);

    impl ItemRule for FoldersFrom {
        fn visit(&self, _name: &str, kind: NodeKind, depth: u32) -> ItemVisit {
            match kind {
                NodeKind::Link => ItemVisit::Skip,
                _ if depth >= self.0 => ItemVisit::Item,
                _ => ItemVisit::Enter,
            }
        }
    }

    fn cap() -> usize {
        usize::try_from(MAX_ITEMS).unwrap()
    }

    /// Times `build`, prints and checks the time and the heap of the kept items.
    fn measure(what: &str, build: impl FnOnce() -> LargestBench) -> LargestBench {
        let started = Instant::now();
        let items = build();
        let elapsed = started.elapsed();

        let heap = items.heap_bytes();
        eprintln!("items of the large tree, {what}: {} kept, {elapsed:.1?}, {heap} heap bytes", items.len());
        assert!(elapsed.as_millis() <= MOST_MILLIS, "{elapsed:?}");
        assert!(heap <= MOST_HEAP, "{heap} bytes");
        items
    }

    #[test]
    #[ignore = "the large tree, run by hand in release"]
    fn items_of_the_large_tree_stay_within_the_budget() {
        let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
        let tree = LargeTree::build(&TreeSpec::LARGE, &under).unwrap();
        let root = std::fs::canonicalize(tree.root()).unwrap();
        let state = committed(&root);
        assert!(state.owned().len() >= 1_000, "{} owners", state.owned().len());
        measure("the default minimum", || LargestBench::build(&*state, &root, DEFAULT_MIN));
        measure("no minimum", || LargestBench::build(&*state, &root, ByteSize::new(0)));
        let full_at = (3..=16)
            .find(|&depth| {
                LargestBench::build_with(&*state, &root, &FoldersFrom(depth), ByteSize::new(0)).len() == cap()
            })
            .unwrap();
        let full = measure(&format!("every folder from depth {full_at} an item, no minimum"), || {
            LargestBench::build_with(&*state, &root, &FoldersFrom(full_at), ByteSize::new(0))
        });
        assert_eq!(full.len(), cap(), "the list is full");
    }

    /// Times the Folders list over `state` with folders left alone since `before`; prints and checks the time and the
    /// heap of the list and its first page.
    fn measure_folders(what: &str, state: &dyn DiskState, root: &Path, before: Timestamp) -> FoldersBench {
        let started = Instant::now();
        let folders = FoldersBench::build(state, root, ByteSize::new(0), before);
        let elapsed = started.elapsed();

        let heap = folders.heap_bytes();
        eprintln!("folders of the large tree, {what}: {} listed, {elapsed:.1?}, {heap} heap bytes", folders.len());
        assert!(elapsed.as_millis() <= MOST_MILLIS, "{elapsed:?}");
        assert!(heap <= MOST_HEAP, "{heap} bytes");
        folders
    }

    #[test]
    #[ignore = "the large tree, run by hand in release"]
    fn folders_of_the_large_tree_stay_within_the_budget() {
        let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
        let tree = LargeTree::build(&TreeSpec::LARGE, &under).unwrap();
        let root = std::fs::canonicalize(tree.root()).unwrap();
        let state = committed(&root);
        let none_left =
            measure_folders("none left alone, every folder entered", &*state, &root, Timestamp::from_secs(0));
        assert!(none_left.is_empty(), "nothing is older than the epoch");
        measure_folders("every folder left alone, the highest of each branch", &*state, &root, state.taken());
    }
}
