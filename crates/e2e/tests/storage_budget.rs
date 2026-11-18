//! The storage map's shares within their budget (plan 012, design section 7: 50 ms and 5 MB) over a state the real
//! scanner committed on the generated `LARGE` tree, with the claims of a real scan's shape: every dependency tree and
//! every cache folder. Run by hand, in release:
//! `cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture shares_of_the_large_tree`.

#[cfg(test)]
mod positive {
    use std::collections::BTreeMap;
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::Instant;

    use fresher_core::{
        ByteSize, CancelToken, CategoryId, Claim, ClaimedPath, DiskWalk, Interest, InterestSet, KnownFolder, ModuleId,
        Pattern, VolumeFacts, VolumeId, WalkMode, WalkRequest,
    };
    use fresher_engine::bench::SharesBench;
    use fresher_scanner::{Scanner, ScannerOptions, StdReader, home_anchor};
    use fresher_testing::{FakeChangeJournal, FakeVolumes, LargeTree, MemoryScanCache, TreeSpec};

    const MOST_MILLIS: u128 = 50;
    const MOST_HEAP: usize = 5 * 1024 * 1024;

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

    #[test]
    #[ignore = "the large tree, run by hand in release"]
    fn shares_of_the_large_tree_stay_within_the_budget() {
        let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
        let tree = LargeTree::build(&TreeSpec::LARGE, &under).unwrap();
        let root = std::fs::canonicalize(tree.root()).unwrap();
        let volumes = FakeVolumes::default()
            .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
        let threads = std::thread::available_parallelism().map_or(1, NonZeroUsize::get).min(16);
        let options = ScannerOptions { threads: NonZeroUsize::new(threads).unwrap(), ..ScannerOptions::default() };
        let scanner = Scanner::new(
            Arc::new(StdReader::new()),
            Arc::new(volumes),
            Arc::new(FakeChangeJournal::default()),
            Arc::new(MemoryScanCache::with_cap(ByteSize::new(1 << 30))),
            home_anchor(&root),
            options,
        );
        let module = ModuleId::new("bench").unwrap();
        let deps = CategoryId::new(module, "bench.deps").unwrap();
        let caches = CategoryId::new(module, "bench.caches").unwrap();
        let (roots, interests) = ([root], interests());
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
        let state = scanner.commit(&claims).unwrap();

        let started = Instant::now();
        let shares = SharesBench::build(&*state);
        let elapsed = started.elapsed();

        let heap = shares.heap_bytes();
        let owners = state.owned().len();
        eprintln!("Shares::of over {} claims, {owners} owned nodes: {elapsed:.1?}, {heap} heap bytes", claims.len());
        assert!(owners >= 1_000, "{owners} owners");
        assert!(elapsed.as_millis() <= MOST_MILLIS, "{elapsed:?}");
        assert!(heap <= MOST_HEAP, "{heap} bytes");
    }
}
