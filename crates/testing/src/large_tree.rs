//! Large generated trees for the benches and the deterministic bars: a seeded shape of folders, sparse files, hard
//! links, dependency trees and cache folders, built once and reused.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use fresher_core::stable_hash;

use crate::FixtureError;
use crate::access_time::settle_access_time;

/// Bumped whenever the generator changes, so an old tree is never taken for a new one.
const GENERATOR_VERSION: u64 = 2;
/// Written last: a folder without it was not finished and is never reused.
const COMPLETE_MARKER: &str = ".complete";
/// About one folder for this many entries.
const ENTRIES_PER_FOLDER: u64 = 10;
/// A small file holds at most this many bytes; a large one at least 1 MiB.
const SMALL_FILE_MAX: u64 = 64 * 1024;
const LARGE_FILE_MIN: u64 = 1 << 20;
const LARGE_FILE_SPAN: u64 = 63 << 20;
/// The content of every generated `CACHEDIR.TAG`.
const TAG_CONTENT: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55\n# A cache directory tag.\n";

/// The shape of a generated tree. The same spec always gives the same tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TreeSpec {
    /// Names the folder the tree is built in.
    pub name: &'static str,
    pub seed: u64,
    /// Every folder, file and link below the root; the root itself is not counted.
    pub entries: u64,
    /// How deep the general folders go below their top folder.
    pub depth: u32,
    /// At most this many subfolders a general folder.
    pub fan_out: u32,
    /// Per thousand files, how many get a large logical size (1 to 64 MiB); the rest get up to 64 KiB.
    pub large_file_share: u32,
    /// Files that are a second name of another file.
    pub hard_links: u32,
    /// Projects with a `node_modules`-like tree of packages.
    pub deep_dependency_folders: u32,
    /// Folders under `Library/Caches` that hold a valid `CACHEDIR.TAG`.
    pub cache_folders: u32,
}

impl TreeSpec {
    /// 100 000 entries: used by the tests.
    pub const SMALL: Self = Self {
        name: "small",
        seed: 0x5eed_0001,
        entries: 100_000,
        depth: 6,
        fan_out: 6,
        large_file_share: 7,
        hard_links: 200,
        deep_dependency_folders: 20,
        cache_folders: 40,
    };

    /// 3 000 000 entries: used by the benches.
    pub const LARGE: Self = Self {
        name: "large",
        seed: 0x5eed_0002,
        entries: 3_000_000,
        depth: 8,
        fan_out: 8,
        large_file_share: 7,
        hard_links: 6_000,
        deep_dependency_folders: 400,
        cache_folders: 600,
    };

    /// A stable hash of the spec and the generator's version: the tree's folder name.
    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::with_capacity(96);
        for value in [
            GENERATOR_VERSION,
            self.seed,
            self.entries,
            u64::from(self.depth),
            u64::from(self.fan_out),
            u64::from(self.large_file_share),
            u64::from(self.hard_links),
            u64::from(self.deep_dependency_folders),
            u64::from(self.cache_folders),
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(self.name.as_bytes());
        stable_hash(&bytes)
    }
}

/// What a spec promises: the entries below the root, the folders among them, and the sum of every file's logical size
/// (a hard link counts as many times as it has names).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Expected {
    pub entries: u64,
    pub folders: u64,
    pub logical: u64,
}

/// A generated tree on disk. Never removed by the generator: it lives under the given folder (`target/bench-trees/`
/// for the benches, which `cargo clean` removes; a temporary folder in a test).
#[derive(Clone, Debug)]
pub struct LargeTree {
    root: PathBuf,
    spec: TreeSpec,
}

impl LargeTree {
    /// The tree of `spec` under `under/<name>-<hash>/`, built unless a finished build is there. A folder left by an
    /// unfinished build is never built over: the next free name (`<name>-<hash>.1`, …) is used.
    pub fn build(spec: &TreeSpec, under: &Path) -> Result<Self, FixtureError> {
        if spec.entries == 0 {
            return Err(FixtureError::InvalidSpec { reason: "a tree needs at least one entry" });
        }
        let base = format!("{}-{:016x}", spec.name, spec.hash());
        for attempt in 0_u32.. {
            let root = if attempt == 0 { under.join(&base) } else { under.join(format!("{base}.{attempt}")) };
            if root.join(COMPLETE_MARKER).is_file() {
                return Ok(Self { root, spec: *spec });
            }
            if root.symlink_metadata().is_ok() {
                continue;
            }
            // Built aside and renamed into place, so parallel test processes never see half a tree: the one that
            // loses the race takes the winner's tree, and its own staging folder is removed when it is dropped.
            create(under, |path| fs::create_dir_all(path))?;
            let staging = tempfile::Builder::new()
                .prefix(&format!(".{base}."))
                .tempdir_in(under)
                .map_err(|source| FixtureError::Create { path: under.to_path_buf(), source })?;
            generate(spec, &mut |item| write(staging.path(), item))?;
            create(&staging.path().join(COMPLETE_MARKER), |path| File::create(path).map(drop))?;
            if fs::rename(staging.path(), &root).is_ok() || root.join(COMPLETE_MARKER).is_file() {
                return Ok(Self { root, spec: *spec });
            }
        }
        Err(FixtureError::InvalidSpec { reason: "no free folder name is left" })
    }

    /// The root of the tree: the folder to scan. Its `.complete` marker is the only entry not of the spec.
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn spec(&self) -> &TreeSpec {
        &self.spec
    }

    /// The counts the spec promises, computed again without touching the disk; the `.complete` marker is not counted.
    pub fn expected(&self) -> Expected {
        expected(&self.spec)
    }
}

/// The counts `spec` promises.
fn expected(spec: &TreeSpec) -> Expected {
    let mut counts = Expected::default();
    // Counting never fails: the sink only adds.
    let _counted = generate(spec, &mut |item| {
        counts.entries += 1;
        match item {
            Item::Folder(_) => counts.folders += 1,
            Item::File { size, .. } | Item::HardLink { size, .. } => counts.logical += size,
            Item::Tag(_) => counts.logical += TAG_CONTENT.len() as u64,
        }
        Ok(())
    });
    counts
}

/// One entry of the tree, its path relative to the root.
#[derive(Clone, Copy, Debug)]
enum Item<'a> {
    Folder(&'a Path),
    File { path: &'a Path, size: u64 },
    HardLink { path: &'a Path, target: &'a Path, size: u64 },
    Tag(&'a Path),
}

fn write(root: &Path, item: Item<'_>) -> Result<(), FixtureError> {
    match item {
        Item::Folder(path) => create(&root.join(path), |path| fs::create_dir(path)),
        Item::File { path, size } => create(&root.join(path), |path| File::create(path)?.set_len(size)),
        Item::HardLink { path, target, .. } => create(&root.join(path), |path| fs::hard_link(root.join(target), path)),
        Item::Tag(path) => {
            create(&root.join(path), |path| fs::write(path, TAG_CONTENT).and_then(|()| settle_access_time(path)))
        }
    }
}

fn create(path: &Path, make: impl FnOnce(&Path) -> std::io::Result<()>) -> Result<(), FixtureError> {
    make(path).map_err(|source| FixtureError::Create { path: path.to_path_buf(), source })
}

/// The seeded generator: xorshift64*, so a spec gives the same numbers on every machine.
struct Random(u64);

impl Random {
    const fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15 | 1)
    }

    const fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A number below `bound` (`bound` at least 1).
    const fn below(&mut self, bound: u64) -> u64 {
        self.next() % if bound == 0 { 1 } else { bound }
    }
}

/// Emits the tree of `spec` item by item, parents before children, exactly `spec.entries` items.
struct Generator<'a> {
    spec: &'a TreeSpec,
    random: Random,
    emitted: u64,
    folders: Vec<PathBuf>,
    link_targets: Vec<(PathBuf, u64)>,
    sink: &'a mut dyn FnMut(Item<'_>) -> Result<(), FixtureError>,
    names: u64,
}

fn generate(spec: &TreeSpec, sink: &mut dyn FnMut(Item<'_>) -> Result<(), FixtureError>) -> Result<(), FixtureError> {
    let mut generator = Generator {
        spec,
        random: Random::new(spec.seed),
        emitted: 0,
        folders: vec![PathBuf::new()],
        link_targets: Vec::new(),
        sink,
        names: 0,
    };
    generator.run()
}

impl Generator<'_> {
    fn run(&mut self) -> Result<(), FixtureError> {
        let folder_budget = (self.spec.entries / ENTRIES_PER_FOLDER).max(1);
        self.caches()?;
        self.projects()?;
        self.general(folder_budget)?;
        let links = u64::from(self.spec.hard_links);
        while self.left() > links {
            self.file()?;
        }
        let mut link = 0_usize;
        while self.left() > 0 {
            if self.link_targets.is_empty() {
                self.file()?;
            } else {
                let (target, size) = self.link_targets.get(link % self.link_targets.len()).cloned().unwrap_or_default();
                let path = self.folder().join(format!("link-{link}.bin"));
                self.emit(Item::HardLink { path: &path, target: &target, size })?;
                link += 1;
            }
        }
        Ok(())
    }

    const fn left(&self) -> u64 {
        self.spec.entries - self.emitted
    }

    fn emit(&mut self, item: Item<'_>) -> Result<bool, FixtureError> {
        if self.left() == 0 {
            return Ok(false);
        }
        if let Item::Folder(path) = &item {
            self.folders.push(path.to_path_buf());
        }
        (self.sink)(item)?;
        self.emitted += 1;
        Ok(true)
    }

    fn add_folder(&mut self, path: &Path) -> Result<bool, FixtureError> {
        self.emit(Item::Folder(path))
    }

    /// `Library/Caches/<app>` folders, each tagged.
    fn caches(&mut self) -> Result<(), FixtureError> {
        let caches = Path::new("Library/Caches");
        if self.spec.cache_folders == 0 || !self.add_folder(Path::new("Library"))? || !self.add_folder(caches)? {
            return Ok(());
        }
        for index in 0..self.spec.cache_folders {
            let folder = caches.join(format!("com.vendor{index}.app"));
            if !self.add_folder(&folder)? {
                return Ok(());
            }
            self.emit(Item::Tag(&folder.join("CACHEDIR.TAG")))?;
            let data = folder.join(format!("{:016x}", self.random.next()));
            self.add_folder(&data)?;
        }
        Ok(())
    }

    /// `Projects/app<n>` with a `package.json` and a `node_modules` of packages, some nesting their own.
    fn projects(&mut self) -> Result<(), FixtureError> {
        if self.spec.deep_dependency_folders == 0 || !self.add_folder(Path::new("Projects"))? {
            return Ok(());
        }
        for index in 0..self.spec.deep_dependency_folders {
            let project = PathBuf::from(format!("Projects/app{index}"));
            if !self.add_folder(&project)? {
                return Ok(());
            }
            self.emit(Item::File { path: &project.join("package.json"), size: 700 })?;
            let mut modules = project.join("node_modules");
            for _ in 0..3 {
                if !self.add_folder(&modules)? {
                    return Ok(());
                }
                let packages = 1 + self.random.below(u64::from(self.spec.fan_out.max(1)) * 2);
                let mut last = modules.clone();
                for package in 0..packages {
                    last = modules.join(format!(
                        "pkg-{package}-{}.{}.{}",
                        self.random.below(9),
                        self.random.below(30),
                        self.random.below(12)
                    ));
                    if !self.add_folder(&last)? || !self.add_folder(&last.join("lib"))? {
                        return Ok(());
                    }
                    self.emit(Item::File { path: &last.join("package.json"), size: 500 })?;
                }
                modules = last.join("node_modules");
            }
        }
        Ok(())
    }

    /// `Data/…`: folders breadth first, up to `depth` below `Data`, until `budget` folders exist.
    fn general(&mut self, budget: u64) -> Result<(), FixtureError> {
        let top = PathBuf::from("Data");
        if !self.add_folder(&top)? {
            return Ok(());
        }
        let mut level = vec![top];
        for _ in 0..self.spec.depth {
            let mut next = Vec::new();
            for parent in &level {
                let children = 1 + self.random.below(u64::from(self.spec.fan_out.max(1)));
                for _ in 0..children {
                    if self.folders.len() as u64 >= budget {
                        return Ok(());
                    }
                    let child = parent.join(self.name(false));
                    if !self.add_folder(&child)? {
                        return Ok(());
                    }
                    next.push(child);
                }
            }
            level = next;
        }
        Ok(())
    }

    /// One file in a random folder; some are kept as hard link targets.
    fn file(&mut self) -> Result<(), FixtureError> {
        let path = self.folder().join(self.name(true));
        let size = if self.random.below(1000) < u64::from(self.spec.large_file_share) {
            LARGE_FILE_MIN + self.random.below(LARGE_FILE_SPAN)
        } else {
            self.random.below(SMALL_FILE_MAX)
        };
        if self.emit(Item::File { path: &path, size })?
            && (self.link_targets.len() as u64) < u64::from(self.spec.hard_links).min(1024)
            && self.random.below(16) == 0
        {
            self.link_targets.push((path, size));
        }
        Ok(())
    }

    fn folder(&mut self) -> PathBuf {
        let index = self.random.below(self.folders.len() as u64);
        self.folders.get(usize::try_from(index).unwrap_or(0)).cloned().unwrap_or_default()
    }

    /// A unique name of a realistic shape: a version, a hash, a log, a source file.
    fn name(&mut self, file: bool) -> String {
        self.names += 1;
        let n = self.names;
        let hash = self.random.next();
        match (file, self.random.below(5)) {
            (false, 0) => format!("{}.{}.{}-{n}", hash % 10, hash % 40, hash % 17),
            (false, 1) => format!("{hash:016x}"),
            (false, 2) => format!("cache-{n}"),
            (false, _) => format!("folder-{n}"),
            (true, 0) => format!("index-{n}.js"),
            (true, 1) => format!("{hash:016x}.tmp"),
            (true, 2) => format!("log-{n}.txt"),
            (true, 3) => format!("v{}.{}.{}-{n}.json", hash % 10, hash % 40, hash % 17),
            (true, _) => format!("data-{n}.bin"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TreeSpec;

    /// A tiny spec of every shape.
    pub(super) const TINY: TreeSpec = TreeSpec {
        name: "tiny",
        seed: 7,
        entries: 600,
        depth: 3,
        fan_out: 3,
        large_file_share: 50,
        hard_links: 10,
        deep_dependency_folders: 2,
        cache_folders: 3,
    };

    mod negative {
        use std::fs;

        use super::TINY;
        use crate::{FixtureError, LargeTree, TreeSpec};

        #[test]
        fn a_spec_with_zero_entries_is_refused() {
            let under = tempfile::tempdir().unwrap();

            let error = LargeTree::build(&TreeSpec { entries: 0, ..TINY }, under.path()).unwrap_err();

            assert!(matches!(error, FixtureError::InvalidSpec { .. }), "{error:?}");
            assert_eq!(fs::read_dir(under.path()).unwrap().count(), 0);
        }

        #[test]
        fn an_unfinished_tree_is_rebuilt() {
            let under = tempfile::tempdir().unwrap();
            let first = LargeTree::build(&TINY, under.path()).unwrap();
            fs::rename(first.root().join(".complete"), under.path().join("moved-away")).unwrap();

            let second = LargeTree::build(&TINY, under.path()).unwrap();

            assert_ne!(second.root(), first.root());
            assert!(first.root().join("Data").is_dir(), "the unfinished tree is left as it was");
            assert!(second.root().join(".complete").is_file());
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt;

        use super::TINY;
        use crate::{LargeTree, TreeReport, TreeSpec};

        fn report_without_marker(tree: &LargeTree) -> TreeReport {
            let mut report = TreeReport::of(tree.root()).unwrap();
            report.entries.retain(|entry| entry.path != ".complete");
            report
        }

        #[test]
        fn builds_running_at_once_share_one_whole_tree() {
            let under = tempfile::tempdir().unwrap();

            let roots: Vec<_> = std::thread::scope(|scope| {
                let builds: Vec<_> =
                    (0..4).map(|_| scope.spawn(|| LargeTree::build(&TINY, under.path()).unwrap())).collect();
                builds.into_iter().map(|build| build.join().unwrap().root().to_path_buf()).collect()
            });

            assert!(roots.iter().all(|root| root == &roots[0]));
            let tree = LargeTree::build(&TINY, under.path()).unwrap();
            assert_eq!(report_without_marker(&tree).entries.len() as u64, tree.expected().entries);
            let leftovers = std::fs::read_dir(under.path()).unwrap().count();
            assert_eq!(leftovers, 1, "only the finished tree stays");
        }

        #[test]
        fn the_same_seed_builds_the_same_tree() {
            let (one, two) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            let first = LargeTree::build(&TINY, one.path()).unwrap();
            let second = LargeTree::build(&TINY, two.path()).unwrap();
            let other = LargeTree::build(&TreeSpec { seed: 8, ..TINY }, two.path()).unwrap();

            assert_eq!(report_without_marker(&first), report_without_marker(&second));
            assert_ne!(report_without_marker(&first), report_without_marker(&other));
        }

        #[test]
        fn a_built_tree_matches_its_expected_counts() {
            let under = tempfile::tempdir().unwrap();
            let tree = LargeTree::build(&TINY, under.path()).unwrap();

            let report = report_without_marker(&tree);
            let expected = tree.expected();

            assert_eq!(report.entries.len() as u64, expected.entries);
            assert_eq!(expected.entries, TINY.entries);
            assert_eq!(report.total_logical_size, expected.logical);
            let folders = report.entries.iter().filter(|entry| entry.kind == crate::EntryKind::Dir).count();
            assert_eq!(folders as u64, expected.folders);
            let tag = report.entries.iter().find(|entry| entry.path.ends_with("CACHEDIR.TAG")).unwrap();
            assert!(fresher_core::is_valid_cachedir_tag(&std::fs::read(tree.root().join(&tag.path)).unwrap()));
            assert!(report.entries.iter().any(|entry| entry.path.contains("node_modules/pkg-")));
            assert_eq!(LargeTree::build(&TINY, under.path()).unwrap().root(), tree.root(), "a finished tree is reused");
        }

        #[test]
        fn a_built_tree_holds_its_hard_links() {
            let under = tempfile::tempdir().unwrap();
            let tree = LargeTree::build(&TINY, under.path()).unwrap();

            let links = TreeReport::of(tree.root())
                .unwrap()
                .entries
                .into_iter()
                .filter(|entry| entry.path.rsplit('/').next().is_some_and(|name| name.starts_with("link-")))
                .count();

            assert_eq!(links, 10);
            let link = walk_find(tree.root(), "link-0.bin").unwrap();
            assert!(std::fs::metadata(link).unwrap().nlink() >= 2);
        }

        #[test]
        fn sparse_files_take_no_data_blocks() {
            let under = tempfile::tempdir().unwrap();
            let tree = LargeTree::build(&TINY, under.path()).unwrap();

            let (mut logical, mut allocated) = (0, 0);
            for entry in TreeReport::of(tree.root()).unwrap().entries {
                let metadata = std::fs::symlink_metadata(tree.root().join(&entry.path)).unwrap();
                if metadata.is_file() {
                    logical += metadata.len();
                    allocated += metadata.blocks() * 512;
                }
            }

            assert!(allocated * 100 < logical, "{allocated} allocated for {logical} logical");
        }

        fn walk_find(folder: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
            for entry in std::fs::read_dir(folder).ok()? {
                let path = entry.ok()?.path();
                if path.file_name().is_some_and(|found| found == name) {
                    return Some(path);
                }
                if path.symlink_metadata().ok()?.is_dir()
                    && let Some(found) = walk_find(&path, name)
                {
                    return Some(found);
                }
            }
            None
        }
    }
}
