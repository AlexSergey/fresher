use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime};

use fresher_core::{
    ByteSize, CancelToken, ChangeJournal, Changes, Clock, CommandOutput, CommandRunError, CommandRunner, JournalGap,
    JournalPosition, Landed, ReadError, ScanCache, ScopeKey, StoreError, Trash, TrashError, VolumeFacts, VolumeId,
    Volumes,
};

/// Locks `mutex`; a test that panicked while holding it leaves the data as it was.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A clock that stands still until a test moves it.
#[derive(Debug)]
pub struct FakeClock {
    now: Mutex<SystemTime>,
}

impl FakeClock {
    /// A clock at `seconds` after the Unix epoch.
    pub fn at(seconds: u64) -> Self {
        Self { now: Mutex::new(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)) }
    }

    pub fn advance(&self, by: Duration) {
        *lock(&self.now) += by;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        *lock(&self.now)
    }
}

/// A Trash inside the fixture tree: items are moved (renamed) into its folder as `<n>-<name>`, never deleted, and put
/// back without replacing anything.
#[derive(Debug)]
pub struct FakeTrash {
    dir: PathBuf,
    moved: Mutex<Vec<PathBuf>>,
    refused: HashSet<PathBuf>,
    no_trash: bool,
    reports: bool,
}

impl FakeTrash {
    /// A Trash whose folder is `dir`; it is created on the first move.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), moved: Mutex::new(Vec::new()), refused: HashSet::new(), no_trash: false, reports: true }
    }

    /// Makes the Trash refuse `path`, as macOS does for a locked item or a read-only folder.
    #[must_use]
    pub fn refusing(mut self, path: impl Into<PathBuf>) -> Self {
        self.refused.insert(path.into());
        self
    }

    /// A volume without a Trash: every move answers `NoTrash` and moves nothing.
    #[must_use]
    pub const fn without_trash(mut self) -> Self {
        self.no_trash = true;
        self
    }

    /// A Trash call that moves but returns no landed path, as the system's call sometimes does.
    #[must_use]
    pub const fn reporting_nothing(mut self) -> Self {
        self.reports = false;
        self
    }

    /// The original paths moved to the Trash, in order.
    pub fn moved(&self) -> Vec<PathBuf> {
        lock(&self.moved).clone()
    }
}

impl Trash for FakeTrash {
    fn move_to_trash(&self, path: &Path, _held: &File) -> Result<Landed, TrashError> {
        if self.refused.contains(path) {
            return Err(TrashError::NotAllowed);
        }
        if self.no_trash {
            return Err(TrashError::NoTrash);
        }
        let (Some(name), Ok(_)) = (path.file_name(), path.symlink_metadata()) else {
            return Err(TrashError::NotFound);
        };
        let refused = |error: std::io::Error| TrashError::Refused { reason: error.to_string() };
        fs::create_dir_all(&self.dir).map_err(refused)?;
        let mut moved = lock(&self.moved);
        let landed = self.dir.join(format!("{}-{}", moved.len(), name.to_string_lossy()));
        fs::rename(path, &landed).map_err(refused)?;
        moved.push(path.to_path_buf());
        Ok(Landed { reported: self.reports.then(|| landed.clone()), at: landed })
    }

    fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError> {
        if original.symlink_metadata().is_ok() {
            return Err(TrashError::Occupied);
        }
        if !original.parent().is_some_and(Path::is_dir) {
            return Err(TrashError::NoParent);
        }
        if landed.symlink_metadata().is_err() {
            return Err(TrashError::NotFound);
        }
        fs::rename(landed, original).map_err(|error| TrashError::Refused { reason: error.to_string() })
    }
}

/// Answers external commands from a table, and records every call.
#[derive(Debug, Default)]
pub struct FakeCommandRunner {
    answers: HashMap<String, Result<CommandOutput, CommandRunError>>,
    calls: Mutex<Vec<String>>,
}

impl FakeCommandRunner {
    /// Answers `command` (the program and its arguments joined by spaces) with a successful output.
    #[must_use]
    pub fn answering(mut self, command: &str, stdout: &str) -> Self {
        let output = CommandOutput { status: 0, stdout: stdout.into(), stderr: String::new() };
        self.answers.insert(command.into(), Ok(output));
        self
    }

    /// Answers `command` with the exit status `status` and nothing on its outputs.
    #[must_use]
    pub fn exiting(mut self, command: &str, status: i32) -> Self {
        let output = CommandOutput { status, stdout: String::new(), stderr: String::new() };
        self.answers.insert(command.into(), Ok(output));
        self
    }

    /// Answers `command` with an error.
    #[must_use]
    pub fn failing(mut self, command: &str, error: CommandRunError) -> Self {
        self.answers.insert(command.into(), Err(error));
        self
    }

    /// The commands run so far, in order.
    pub fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CommandRunError> {
        let command = std::iter::once(program).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
        lock(&self.calls).push(command.clone());
        self.answers
            .get(&command)
            .cloned()
            .unwrap_or_else(|| Err(CommandRunError::NotInstalled { program: program.into() }))
    }
}

/// Fixed volume facts per root: a path is on the volume of the longest root it starts with. The available space is
/// played from a script, one value per call.
#[derive(Debug, Default)]
pub struct FakeVolumes {
    roots: Vec<(PathBuf, VolumeFacts)>,
    available: Mutex<VecDeque<Result<ByteSize, ReadError>>>,
}

impl FakeVolumes {
    /// Puts `root` and everything under it on the volume `facts`.
    #[must_use]
    pub fn with(mut self, root: impl Into<PathBuf>, facts: VolumeFacts) -> Self {
        self.roots.push((root.into(), facts));
        self
    }

    /// Adds the next answer of `available`; once the script runs out, it answers `ReadError::Failed`.
    #[must_use]
    pub fn answering_available(self, answer: Result<ByteSize, ReadError>) -> Self {
        lock(&self.available).push_back(answer);
        self
    }
}

impl Volumes for FakeVolumes {
    fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError> {
        self.roots
            .iter()
            .filter(|(root, _)| path.starts_with(root))
            .max_by_key(|(root, _)| root.components().count())
            .map(|&(_, facts)| facts)
            .ok_or(ReadError::Vanished)
    }

    fn available(&self, _path: &Path) -> Result<ByteSize, ReadError> {
        lock(&self.available).pop_front().unwrap_or(Err(ReadError::Failed))
    }
}

/// A change journal played from a script: a position per volume, and the answers of `changes_since` in order (a gap
/// once the script runs out). Records every call.
#[derive(Debug, Default)]
pub struct FakeChangeJournal {
    positions: HashMap<VolumeId, JournalPosition>,
    answers: Mutex<VecDeque<Result<Changes, JournalGap>>>,
    calls: Mutex<Vec<(VolumeId, JournalPosition)>>,
}

impl FakeChangeJournal {
    /// Makes the journal of `volume` stand at `position`; a volume without one keeps no journal.
    #[must_use]
    pub fn at(mut self, volume: VolumeId, position: JournalPosition) -> Self {
        self.positions.insert(volume, position);
        self
    }

    /// Adds the next answer of `changes_since`.
    #[must_use]
    pub fn answering(self, answer: Result<Changes, JournalGap>) -> Self {
        lock(&self.answers).push_back(answer);
        self
    }

    /// The `changes_since` calls so far: the volume and the position asked from, in order.
    pub fn calls(&self) -> Vec<(VolumeId, JournalPosition)> {
        lock(&self.calls).clone()
    }
}

impl ChangeJournal for FakeChangeJournal {
    fn position(&self, volume: &VolumeId) -> Option<JournalPosition> {
        self.positions.get(volume).copied()
    }

    fn changes_since(
        &self,
        volume: &VolumeId,
        since: &JournalPosition,
        cancel: &CancelToken,
    ) -> Result<Changes, JournalGap> {
        lock(&self.calls).push((*volume, *since));
        if cancel.is_cancelled() {
            return Err(JournalGap::Cancelled);
        }
        lock(&self.answers).pop_front().unwrap_or(Err(JournalGap::Failed))
    }
}

/// A scan cache in memory: two generations per scope, within a cap.
#[derive(Debug)]
pub struct MemoryScanCache {
    cap: ByteSize,
    generations: Mutex<HashMap<ScopeKey, Generations>>,
}

/// The current and the previous body of one scope.
#[derive(Debug, Default)]
struct Generations {
    current: Option<Vec<u8>>,
    previous: Option<Vec<u8>>,
}

impl MemoryScanCache {
    /// An empty cache that refuses a body over `cap`.
    pub fn with_cap(cap: ByteSize) -> Self {
        Self { cap, generations: Mutex::new(HashMap::new()) }
    }
}

impl ScanCache for MemoryScanCache {
    fn cap(&self) -> ByteSize {
        self.cap
    }

    fn load(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(lock(&self.generations).get(&scope).and_then(|kept| kept.current.clone()))
    }

    fn load_previous(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(lock(&self.generations).get(&scope).and_then(|kept| kept.previous.clone()))
    }

    fn save(&self, scope: ScopeKey, body: &[u8]) -> Result<(), StoreError> {
        let size = ByteSize::new(u64::try_from(body.len()).unwrap_or(u64::MAX));
        if size > self.cap {
            return Err(StoreError::TooLarge { size, cap: self.cap });
        }
        let mut generations = lock(&self.generations);
        let kept = generations.entry(scope).or_default();
        kept.previous = kept.current.replace(body.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{ByteSize, VolumeFacts, VolumeId};

    /// An open handle of `path`, as the cleaner holds one; the fake Trash does not read it.
    fn held(path: &std::path::Path) -> std::fs::File {
        std::fs::File::open(path).unwrap()
    }

    fn facts(id: u128) -> VolumeFacts {
        VolumeFacts { id: VolumeId::new(id), device: 16, case_sensitive: false, used: ByteSize::new(1 << 30) }
    }

    mod negative {
        use std::path::Path;

        use std::fs;

        use fresher_core::{
            ByteSize, CancelToken, ChangeJournal, CommandRunError, CommandRunner, JournalGap, JournalPosition,
            ReadError, ScanCache, ScopeKey, StoreError, Trash, TrashError, VolumeId, Volumes,
        };

        use super::held;
        use crate::{FakeChangeJournal, FakeCommandRunner, FakeTrash, FakeVolumes, FixtureTree, MemoryScanCache};

        #[test]
        fn the_trash_reports_a_missing_path() {
            let tree = FixtureTree::from_toml_str("").unwrap();
            let trash = FakeTrash::new(tree.root().join(".Trash"));

            let error = trash.move_to_trash(&tree.root().join("missing"), &held(tree.root())).unwrap_err();

            assert_eq!(error, TrashError::NotFound);
            assert_eq!(trash.moved(), Vec::<std::path::PathBuf>::new());
        }

        #[test]
        fn the_trash_reports_a_path_without_a_name() {
            let trash = FakeTrash::new("/nonexistent/.Trash");

            assert_eq!(trash.move_to_trash(Path::new("/"), &held(Path::new("/"))), Err(TrashError::NotFound));
        }

        #[test]
        fn the_trash_refuses_a_path_it_was_told_to() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"locked.db\"\n").unwrap();
            let locked = tree.root().join("locked.db");
            let trash = FakeTrash::new(tree.root().join(".Trash")).refusing(&locked);

            assert_eq!(trash.move_to_trash(&locked, &held(&locked)), Err(TrashError::NotAllowed));
            assert!(locked.exists());
        }

        #[test]
        fn the_fake_trash_without_a_trash_moves_nothing() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.bin\"\n").unwrap();
            let item = tree.root().join("a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash")).without_trash();

            assert_eq!(trash.move_to_trash(&item, &held(&item)), Err(TrashError::NoTrash));
            assert!(item.exists());
            assert!(!tree.root().join(".Trash").exists());
        }

        #[test]
        fn the_fake_trash_never_replaces_on_put_back() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.bin\"\nsize = 3\n").unwrap();
            let item = tree.root().join("a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash"));
            let landed = trash.move_to_trash(&item, &held(&item)).unwrap().at;
            fs::write(&item, b"new").unwrap();

            assert_eq!(trash.put_back(&landed, &item), Err(TrashError::Occupied));
            assert_eq!(fs::read(&item).unwrap(), b"new");
            assert!(landed.exists());
            let gone = tree.root().join("gone/a.bin");
            assert_eq!(trash.put_back(&landed, &gone), Err(TrashError::NoParent));
            fs::rename(&item, tree.root().join("aside")).unwrap();
            assert_eq!(trash.put_back(&tree.root().join(".Trash/9-x"), &item), Err(TrashError::NotFound));
        }

        #[test]
        fn a_call_that_reports_nothing_still_says_where_the_handle_is() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.bin\"\n").unwrap();
            let item = tree.root().join("a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash")).reporting_nothing();

            let landed = trash.move_to_trash(&item, &held(&item)).unwrap();

            assert_eq!(landed.reported, None);
            assert_eq!(landed.at, tree.root().join(".Trash/0-a.bin"));
        }

        #[test]
        fn an_unknown_command_is_not_installed() {
            let runner = FakeCommandRunner::default();

            let error = runner.run("docker", &["images"]).unwrap_err();

            assert_eq!(error, CommandRunError::NotInstalled { program: "docker".into() });
        }

        #[test]
        fn the_fake_journal_returns_its_scripted_gap() {
            let volume = VolumeId::new(7);
            let since = JournalPosition { journal: 1, cursor: 10 };
            let journal = FakeChangeJournal::default().answering(Err(JournalGap::Dropped));

            assert_eq!(journal.position(&volume), None);
            assert_eq!(journal.changes_since(&volume, &since, &CancelToken::new()), Err(JournalGap::Dropped));
            assert_eq!(journal.changes_since(&volume, &since, &CancelToken::new()), Err(JournalGap::Failed));
        }

        #[test]
        fn the_fake_journal_stops_when_cancelled() {
            let cancel = CancelToken::new();
            cancel.cancel();
            let journal = FakeChangeJournal::default().answering(Err(JournalGap::Reset));
            let since = JournalPosition { journal: 1, cursor: 0 };

            assert_eq!(journal.changes_since(&VolumeId::new(1), &since, &cancel), Err(JournalGap::Cancelled));
        }

        #[test]
        fn the_fake_volumes_do_not_know_a_path_outside_their_roots() {
            let volumes = FakeVolumes::default().with("/home", super::facts(1));

            assert_eq!(volumes.identify(Path::new("/elsewhere")), Err(ReadError::Vanished));
            assert_eq!(volumes.identify(Path::new("/homeless")), Err(ReadError::Vanished));
            assert_eq!(volumes.available(Path::new("/home")), Err(ReadError::Failed));
        }

        #[test]
        fn the_memory_cache_refuses_a_body_over_its_cap() {
            let cache = MemoryScanCache::with_cap(ByteSize::new(4));
            let scope = ScopeKey::new(1);

            let error = cache.save(scope, b"12345");

            assert_eq!(error, Err(StoreError::TooLarge { size: ByteSize::new(5), cap: ByteSize::new(4) }));
            assert_eq!(cache.load(scope), Ok(None));
        }

        #[test]
        fn an_empty_scope_loads_nothing() {
            let cache = MemoryScanCache::with_cap(ByteSize::new(100));
            cache.save(ScopeKey::new(1), b"body").unwrap();

            assert_eq!(cache.load(ScopeKey::new(2)), Ok(None));
            assert_eq!(cache.load_previous(ScopeKey::new(2)), Ok(None));
            assert_eq!(cache.load_previous(ScopeKey::new(1)), Ok(None));
        }

        #[test]
        fn a_failing_command_returns_its_error() {
            let error = CommandRunError::Failed { program: "docker".into(), reason: "daemon down".into() };
            let runner = FakeCommandRunner::default().failing("docker ps", error.clone());

            assert_eq!(runner.run("docker", &["ps"]), Err(error));
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};
        use std::time::{Duration, SystemTime};

        use fresher_core::{
            ByteSize, CancelToken, ChangeJournal, Changes, Clock, CommandRunner, JournalPosition, Landed, ScanCache,
            ScopeKey, Trash, VolumeId, Volumes,
        };

        use crate::{
            FakeChangeJournal, FakeClock, FakeCommandRunner, FakeTrash, FakeVolumes, FixtureTree, MemoryScanCache,
        };

        #[test]
        fn save_moves_the_current_generation_to_previous() {
            let cache = MemoryScanCache::with_cap(ByteSize::new(100));
            let scope = ScopeKey::new(1);

            cache.save(scope, b"first").unwrap();
            cache.save(scope, b"second").unwrap();

            assert_eq!(cache.cap(), ByteSize::new(100));
            assert_eq!(cache.load(scope), Ok(Some(b"second".to_vec())));
            assert_eq!(cache.load_previous(scope), Ok(Some(b"first".to_vec())));
        }

        #[test]
        fn the_fake_journal_records_each_call() {
            let (home, external) = (VolumeId::new(1), VolumeId::new(2));
            let now = JournalPosition { journal: 1, cursor: 20 };
            let changes = Changes { folders: vec![PathBuf::from("/home/a")], subtrees: vec![], end: now };
            let journal = FakeChangeJournal::default().at(home, now).answering(Ok(changes.clone()));
            let since = JournalPosition { journal: 1, cursor: 10 };

            assert_eq!(journal.position(&home), Some(now));
            assert_eq!(journal.changes_since(&home, &since, &CancelToken::new()), Ok(changes));
            let _gap = journal.changes_since(&external, &now, &CancelToken::new());

            assert_eq!(journal.calls(), [(home, since), (external, now)]);
        }

        #[test]
        fn the_fake_volumes_answer_per_root() {
            let volumes = FakeVolumes::default().with("/home", super::facts(1)).with("/home/external", super::facts(2));

            assert_eq!(volumes.identify(Path::new("/home")), Ok(super::facts(1)));
            assert_eq!(volumes.identify(Path::new("/home/a/b")), Ok(super::facts(1)));
            assert_eq!(volumes.identify(Path::new("/home/external/c")), Ok(super::facts(2)));
        }

        #[test]
        fn the_clock_stands_still_until_advanced() {
            let clock = FakeClock::at(100);
            assert_eq!(clock.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(100));

            clock.advance(Duration::from_secs(5));

            assert_eq!(clock.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(105));
        }

        #[test]
        fn the_fake_trash_returns_where_the_item_landed() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Library/Caches/a.bin\"\nsize = 10\n").unwrap();
            let item = tree.root().join("Library/Caches/a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash"));

            let landed = trash.move_to_trash(&item, &super::held(&item)).unwrap();

            assert!(!item.exists());
            let at = tree.root().join(".Trash/0-a.bin");
            assert!(at.exists());
            assert_eq!(landed, Landed { at: at.clone(), reported: Some(at) });
            assert_eq!(trash.moved(), [item]);
        }

        #[test]
        fn the_fake_trash_puts_back() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.bin\"\nsize = 10\n").unwrap();
            let item = tree.root().join("a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash"));
            let landed = trash.move_to_trash(&item, &super::held(&item)).unwrap();

            trash.put_back(&landed.at, &item).unwrap();

            assert_eq!(item.metadata().unwrap().len(), 10);
            assert!(!landed.at.exists());
        }

        #[test]
        fn the_fake_volumes_answer_the_scripted_space() {
            let volumes = FakeVolumes::default()
                .answering_available(Ok(ByteSize::new(100)))
                .answering_available(Err(fresher_core::ReadError::Vanished));

            assert_eq!(volumes.available(Path::new("/home")), Ok(ByteSize::new(100)));
            assert_eq!(volumes.available(Path::new("/home")), Err(fresher_core::ReadError::Vanished));
        }

        #[test]
        fn a_known_command_answers_and_is_recorded() {
            let runner = FakeCommandRunner::default().answering("docker images --format json", "[]");

            let output = runner.run("docker", &["images", "--format", "json"]).unwrap();

            assert_eq!(output.stdout, "[]");
            assert_eq!(runner.calls(), ["docker images --format json"]);
            let stopped = FakeCommandRunner::default().exiting("docker info", 1);
            assert_eq!(stopped.run("docker", &["info"]).map(|output| output.status), Ok(1));
        }
    }
}
