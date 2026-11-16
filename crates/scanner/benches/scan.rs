//! The scan benches, run by hand: `npm run bench` (docs/testing.md, "Performance"). The tree is generated once under
//! `target/bench-trees/`: `FRESHER_BENCH_TREE=large` picks the 3-million-entry tree, anything else the small one.
//! After each group the peak memory of the process is printed.

#![expect(clippy::panic, reason = "a bench stops at its first broken input")]

use std::hint::black_box;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use criterion::{Criterion, criterion_group, criterion_main};
use fresher_core::{
    ByteSize, CancelToken, ChangeJournal, Changes, Claim, DiskWalk, EntryKind, Interest, InterestSet, JournalGap,
    JournalPosition, KnownFolder, ModuleId, Pattern, VolumeId, WalkMode, WalkRequest,
};
use fresher_macos::{MacFolderReader, MacVolumes};
use fresher_scanner::bench::{CacheBench, MatcherBench, QueryBench, walked_folders};
use fresher_scanner::{Scanner, ScannerOptions, home_anchor};
use fresher_testing::{FakeChangeJournal, LargeTree, MemoryScanCache, TreeSpec};

/// The generated tree of this run, built on first use.
fn bench_tree() -> &'static LargeTree {
    static TREE: OnceLock<LargeTree> = OnceLock::new();
    TREE.get_or_init(|| {
        let spec = match std::env::var("FRESHER_BENCH_TREE").as_deref() {
            Ok("large") => TreeSpec::LARGE,
            _ => TreeSpec::SMALL,
        };
        let under = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-trees");
        let started = std::time::Instant::now();
        let tree = LargeTree::build(&spec, &under).unwrap_or_else(|error| panic!("the bench tree: {error}"));
        report(&format!("tree {} ready in {:.1?} at {}", spec.name, started.elapsed(), tree.root().display()));
        tree
    })
}

#[expect(clippy::print_stdout, reason = "the bench's report")]
fn report(line: &str) {
    println!("{line}");
}

fn report_peak(group: &str) {
    report(&format!("{group}: peak memory {} bytes", fresher_macos::peak_memory().bytes()));
}

/// The 64-interest set of the matcher's checks: 16 folder names, extensions, markers and anchored paths.
fn interests() -> InterestSet {
    let interest = |pattern, claim, siblings: &[&str]| {
        Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect())
            .unwrap_or_else(|error| panic!("a bench interest: {error}"))
    };
    let mut list = Vec::new();
    for index in 0..16 {
        list.push(interest(Pattern::FolderName(format!("folder{index}")), Claim::Whole, &["package.json"]));
        list.push(interest(Pattern::Extension(format!("e{index}")), Claim::Item, &[]));
        list.push(interest(Pattern::MarkerFile(format!("marker{index}")), Claim::Item, &[]));
        list.push(interest(Pattern::Under(KnownFolder::Home, format!("Library/Caches{index}/*")), Claim::Item, &[]));
    }
    let mut set = InterestSet::new();
    set.add(ModuleId::new("bench").unwrap_or_else(|error| panic!("{error}")), &list);
    set
}

fn matcher(criterion: &mut Criterion) {
    let home = PathBuf::from("/home");
    let bench = MatcherBench::new(&interests(), &home, &home);
    let names: Vec<String> = (0..1_000).map(|index| format!("some-file-{index}.e{}", index % 40)).collect();
    let mut group = criterion.benchmark_group("matcher");
    group.bench_function("entry/1000-names", |bencher| {
        bencher
            .iter(|| names.iter().map(|name| bench.entry(black_box(name.as_bytes()), EntryKind::File)).sum::<usize>());
    });
    group.finish();
    report_peak("matcher");
}

/// The interests of a real scan's shape: a whole dependency folder, a valid tag, an anchored cache, an extension.
fn scan_interests() -> InterestSet {
    let interest = |pattern, claim, siblings: &[&str]| {
        Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect())
            .unwrap_or_else(|error| panic!("a bench interest: {error}"))
    };
    let mut set = InterestSet::new();
    set.add(
        ModuleId::new("bench").unwrap_or_else(|error| panic!("{error}")),
        &[
            interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
            interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
            interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
            interest(Pattern::Extension("tmp".into()), Claim::Item, &[]),
        ],
    );
    set
}

/// The interests of the cache and rescan benches: those of the scan without the extension, which matches one file in
/// five of the generated tree (a matched file is kept, and no subtree holding one can be collapsed to fit the cap).
fn cache_interests() -> InterestSet {
    let interest = |pattern, claim, siblings: &[&str]| {
        Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect())
            .unwrap_or_else(|error| panic!("a bench interest: {error}"))
    };
    let mut set = InterestSet::new();
    set.add(
        ModuleId::new("bench").unwrap_or_else(|error| panic!("{error}")),
        &[
            interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
            interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
            interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
        ],
    );
    set
}

/// One full scan of `root` with the macOS reader and `threads` workers; the number of matches.
fn full_scan(root: &Path, interests: &InterestSet, threads: usize) -> usize {
    let options = ScannerOptions {
        threads: NonZeroUsize::new(threads).unwrap_or(NonZeroUsize::MIN),
        ..ScannerOptions::default()
    };
    let journal = Arc::new(FakeChangeJournal::default());
    let cache = Arc::new(MemoryScanCache::with_cap(ByteSize::new(8 << 20)));
    let scanner = Scanner::new(
        Arc::new(MacFolderReader::new()),
        Arc::new(MacVolumes::new()),
        journal,
        cache,
        home_anchor(root),
        options,
    );
    let roots = [root.to_path_buf()];
    let request = WalkRequest { roots: &roots, interests, avoid: &[], mode: WalkMode::Full };
    let mut matches = 0;
    scanner
        .walk(&request, &CancelToken::new(), &mut |_| matches += 1, &mut |_| {})
        .unwrap_or_else(|error| panic!("the bench scan: {error}"));
    matches
}

fn scan(criterion: &mut Criterion) {
    let tree = bench_tree();
    let interests = scan_interests();
    let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let mut group = criterion.benchmark_group("scan");
    group.sample_size(10);
    for threads in [1, cores, 2 * cores, 3 * cores, 4 * cores] {
        group.bench_function(format!("full/{}/{threads}-threads", tree.spec().name), |bencher| {
            bencher.iter(|| full_scan(tree.root(), &interests, threads));
        });
    }
    group.finish();
    report_peak("scan");
}

/// The scanner over the bench tree with the macOS reader, a cache in memory and a journal with no position.
fn bench_scanner(root: &Path, threads: usize) -> Scanner {
    scanner_with(root, threads, Arc::new(FakeChangeJournal::default()), 100)
}

/// The scanner over the bench tree with the macOS reader, a cache in memory, `journal`, and a rescan limit of `limit`
/// percent.
fn scanner_with(root: &Path, threads: usize, journal: Arc<dyn ChangeJournal>, limit: u64) -> Scanner {
    let options = ScannerOptions {
        threads: NonZeroUsize::new(threads).unwrap_or(NonZeroUsize::MIN),
        rescan_limit_percent: limit,
        ..ScannerOptions::default()
    };
    let cache = Arc::new(MemoryScanCache::with_cap(ByteSize::new(64 << 20)));
    Scanner::new(
        Arc::new(MacFolderReader::new()),
        Arc::new(MacVolumes::new()),
        journal,
        cache,
        home_anchor(root),
        options,
    )
}

/// The cache body of the bench tree: writing it, reading it back, fitting it to the store's 8 MiB file.
fn cache(criterion: &mut Criterion) {
    const FILE_CAP: u64 = 8 << 20;
    const BODY_LIMIT: u64 = 64 << 20;
    let tree = bench_tree();
    let interests = cache_interests();
    let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let scanner = bench_scanner(tree.root(), cores.min(16));
    let roots = [tree.root().to_path_buf()];
    let request = WalkRequest { roots: &roots, interests: &interests, avoid: &[], mode: WalkMode::Full };
    let bench = CacheBench::of(&scanner, &request).unwrap_or_else(|error| panic!("the bench scan: {error}"));
    let body = bench.encode();
    let sizes = bench.sizes(&body);
    let fitted = bench.fit(FILE_CAP, BODY_LIMIT);
    let fitted_sizes = bench.sizes(&fitted);
    report(&format!(
        "cache/{}: {} nodes, body {} B ({} B a node), compressed {} B; fitted body {} B, compressed {} B",
        tree.spec().name,
        sizes.nodes,
        sizes.body,
        sizes.body / sizes.nodes.max(1),
        sizes.packed,
        fitted_sizes.body,
        fitted_sizes.packed,
    ));
    let mut group = criterion.benchmark_group("cache");
    group.sample_size(10);
    let name = tree.spec().name;
    group.bench_function(format!("encode/{name}"), |bencher| bencher.iter(|| bench.encode().len()));
    group.bench_function(format!("decode/{name}"), |bencher| bencher.iter(|| CacheBench::decode(black_box(&body))));
    group.bench_function(format!("fit/{name}"), |bencher| bencher.iter(|| bench.fit(FILE_CAP, BODY_LIMIT).len()));
    group.finish();
    report_peak("cache");
}

/// A journal that gives the same changes at every call, set by the bench: the benches measure the scanner, not the
/// system's journal.
#[derive(Debug, Default)]
struct ScriptedJournal {
    folders: Mutex<Vec<PathBuf>>,
}

impl ScriptedJournal {
    fn set(&self, folders: Vec<PathBuf>) {
        *self.folders.lock().unwrap_or_else(PoisonError::into_inner) = folders;
    }
}

impl ChangeJournal for ScriptedJournal {
    fn position(&self, _volume: &VolumeId) -> Option<JournalPosition> {
        Some(JournalPosition { journal: 1, cursor: 1 })
    }

    fn changes_since(
        &self,
        _volume: &VolumeId,
        since: &JournalPosition,
        _cancel: &CancelToken,
    ) -> Result<Changes, JournalGap> {
        let folders = self.folders.lock().unwrap_or_else(PoisonError::into_inner).clone();
        Ok(Changes { folders, subtrees: Vec::new(), end: *since })
    }
}

/// Every `step`-th folder of `folders`, `count` of them at most.
fn spread(folders: &[PathBuf], count: usize) -> Vec<PathBuf> {
    let step = (folders.len() / count.max(1)).max(1);
    folders.iter().step_by(step).take(count).cloned().collect()
}

/// One rescan or restore of the bench tree; panics when it scanned in full.
fn rescan_once(scanner: &Scanner, request: &WalkRequest<'_>) -> usize {
    let mut matches = 0;
    let summary = scanner
        .walk(request, &CancelToken::new(), &mut |_| matches += 1, &mut |_| {})
        .unwrap_or_else(|error| panic!("the bench rescan: {error}"));
    if let Some(reason) = summary.fallback {
        panic!("the bench rescan scanned in full: {reason:?}");
    }
    matches
}

/// The restore of the committed state (the cache decoded, the matches emitted, no folder read), and the rescan of the
/// bench tree with the macOS reader and a scripted journal: nothing changed, an ordinary day (1 000 changed folders,
/// 01/02), and a growing share of changed folders against a full scan, for `RESCAN_LIMIT_PERCENT`.
fn rescan(criterion: &mut Criterion) {
    let tree = bench_tree();
    let interests = cache_interests();
    let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get).min(16);
    let journal = Arc::new(ScriptedJournal::default());
    let scanner = scanner_with(tree.root(), cores, Arc::clone(&journal) as _, 100);
    let roots = [tree.root().to_path_buf()];
    let full = WalkRequest { roots: &roots, interests: &interests, avoid: &[], mode: WalkMode::Full };
    let auto = WalkRequest { mode: WalkMode::Auto, ..full };
    scanner.walk(&full, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap_or_else(|error| panic!("{error}"));
    let folders = walked_folders(&scanner);
    scanner.commit(&[]).unwrap_or_else(|error| panic!("{error}"));
    report(&format!("rescan/{}: {} folders to change", tree.spec().name, folders.len()));
    let name = tree.spec().name;
    let mut group = criterion.benchmark_group("scan");
    group.sample_size(10);
    let restore = WalkRequest { mode: WalkMode::Restore, ..full };
    group.bench_function(format!("restore/{name}"), |bencher| bencher.iter(|| rescan_once(&scanner, &restore)));
    journal.set(Vec::new());
    group.bench_function(format!("rescan-unchanged/{name}"), |bencher| bencher.iter(|| rescan_once(&scanner, &auto)));
    journal.set(spread(&folders, 1_000));
    group.bench_function(format!("rescan-day/{name}"), |bencher| bencher.iter(|| rescan_once(&scanner, &auto)));
    for percent in [1, 2, 5, 10, 20, 30, 50] {
        journal.set(spread(&folders, folders.len() * percent / 100));
        group.bench_function(format!("rescan-limit/{name}/{percent}%"), |bencher| {
            bencher.iter(|| rescan_once(&scanner, &auto));
        });
    }
    group.bench_function(format!("rescan-limit/{name}/full"), |bencher| {
        bencher.iter(|| full_scan(tree.root(), &interests, cores));
    });
    group.finish();
    report_peak("rescan");
}

/// The read queries over the state of the bench tree, scanned with the scan's interests (a matched `*.tmp` file is
/// kept as its own node, so the tree has many names): the largest files, and search by a common name, a rare one and
/// one letter. The lowercase copy of the names is built by the first search, before the timed ones.
fn state(criterion: &mut Criterion) {
    let tree = bench_tree();
    let interests = scan_interests();
    let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get).min(16);
    let scanner = bench_scanner(tree.root(), cores);
    let roots = [tree.root().to_path_buf()];
    let request = WalkRequest { roots: &roots, interests: &interests, avoid: &[], mode: WalkMode::Full };
    let bench = QueryBench::of(&scanner, &request).unwrap_or_else(|error| panic!("the bench scan: {error}"));
    let before = bench.heap_bytes();
    let started = std::time::Instant::now();
    bench.search("cache", 500);
    let first = started.elapsed();
    report(&format!(
        "state/{}: {} nodes, tree {} B, lowercase copy {} B, first search (builds the copy) {first:.1?}",
        tree.spec().name,
        bench.nodes(),
        before,
        bench.heap_bytes() - before,
    ));
    let name = tree.spec().name;
    let mut group = criterion.benchmark_group("state");
    group.bench_function(format!("largest/{name}"), |bencher| bencher.iter(|| bench.largest_files(500)));
    for (label, query) in [("common", "cache"), ("rare", "vendor42."), ("one-letter", "a")] {
        group.bench_function(format!("search/{name}/{label}"), |bencher| {
            bencher.iter(|| bench.search(black_box(query), 500));
        });
    }
    group.finish();
    report_peak("state");
}

criterion_group!(benches, matcher, scan, cache, rescan, state);
criterion_main!(benches);
