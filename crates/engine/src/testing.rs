//! Fakes for the engine's own tests: a module whose behaviour a test sets, a scripted walk, settings in memory.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fresher_core::{
    Access, AccessProbe, AccessStatus, ByteSize, CancelToken, CategoryDescriptor, CategoryId, Claim, ClaimedPath,
    DangerLevel, DiskState, DiskWalk, EntryKind, Event, Finding, FindingSink, Interest, Locales, Match, Module,
    ModuleContext, ModuleDescriptor, ModuleError, ModuleId, NewFinding, Pattern, Removal, ScanKind, SettingsStore,
    Sizes, StateId, StoreError, StoredSettings, Subject, Text, TextKey, Timestamp, Unreadable, WalkError, WalkMode,
    WalkProgress, WalkRequest, WalkSummary,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FixedDiskState};

use crate::{Engine, Ports, Registry};

/// How long a test waits for an event before it fails: a guard against a hang, never a timing assumption.
const PATIENCE: Duration = Duration::from_secs(10);

type Finish = dyn Fn(&ModuleContext<'_>, &mut dyn FindingSink) -> Result<(), ModuleError> + Send + Sync;

/// A module built by a test: its categories, its interests, what it does with a match and in `finish`.
pub(crate) struct TestModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
    on_match: OnMatch,
    finish: Box<Finish>,
    seen: Arc<Mutex<Vec<Match>>>,
}

/// What a [`TestModule`] does with a match.
#[derive(Clone, Copy, Debug)]
enum OnMatch {
    Find,
    Fail,
    Panic,
}

/// Builds a [`TestModule`].
pub(crate) struct TestModuleBuilder {
    id: ModuleId,
    categories: Vec<CategoryDescriptor>,
    locales: &'static str,
    interests: Vec<Interest>,
    on_match: OnMatch,
    finish: Box<Finish>,
    seen: Arc<Mutex<Vec<Match>>>,
}

impl TestModule {
    /// A module `id` without categories, settings, texts or interests, that finds nothing.
    pub(crate) fn builder(id: &'static str) -> TestModuleBuilder {
        TestModuleBuilder {
            id: ModuleId::new(id).unwrap(),
            categories: Vec::new(),
            locales: "{}",
            interests: Vec::new(),
            on_match: OnMatch::Find,
            finish: Box::new(|_, _| Ok(())),
            seen: Arc::default(),
        }
    }
}

impl TestModuleBuilder {
    pub(crate) fn category(mut self, id: &'static str, danger: DangerLevel) -> Self {
        self.categories.push(CategoryDescriptor::new(CategoryId::new(self.id, id).unwrap(), danger));
        self
    }

    pub(crate) const fn locales(mut self, en: &'static str) -> Self {
        self.locales = en;
        self
    }

    /// The module wants every folder named `name`; each match becomes one finding of its first category.
    pub(crate) fn folder(mut self, name: &str) -> Self {
        self.interests.push(Interest::new(Pattern::FolderName(name.into()), Claim::Whole, vec![]).unwrap());
        self
    }

    pub(crate) const fn failing_on_match(mut self) -> Self {
        self.on_match = OnMatch::Fail;
        self
    }

    pub(crate) const fn panicking_on_match(mut self) -> Self {
        self.on_match = OnMatch::Panic;
        self
    }

    pub(crate) fn on_finish(
        mut self,
        finish: impl Fn(&ModuleContext<'_>, &mut dyn FindingSink) -> Result<(), ModuleError> + Send + Sync + 'static,
    ) -> Self {
        self.finish = Box::new(finish);
        self
    }

    /// The matches the module will receive, in order, with the module's own interest index.
    pub(crate) fn seen(&self) -> Arc<Mutex<Vec<Match>>> {
        Arc::clone(&self.seen)
    }

    pub(crate) fn build(self) -> TestModule {
        let locales = Locales { en: self.locales };
        let descriptor = ModuleDescriptor::new(self.id, self.categories, vec![], locales).unwrap();
        TestModule {
            descriptor,
            interests: self.interests,
            on_match: self.on_match,
            finish: self.finish,
            seen: self.seen,
        }
    }

    pub(crate) fn boxed(self) -> Box<dyn Module> {
        Box::new(self.build())
    }
}

impl Module for TestModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    fn on_match(&self, matched: &Match, _cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        self.seen.lock().unwrap().push(matched.clone());
        match self.on_match {
            OnMatch::Find => {
                out.push(finding(&self.descriptor.categories()[0], matched.path.clone(), matched.sizes));
                Ok(())
            }
            OnMatch::Fail => Err(ModuleError::Failed { reason: text("test.module.failed") }),
            OnMatch::Panic => panic_now("a module panics on a match"),
        }
    }

    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        (self.finish)(cx, out)
    }
}

pub(crate) fn text(key: &str) -> Text {
    Text::new(TextKey::new(key).unwrap())
}

/// Category `id` of the module its first segment names.
pub(crate) fn category(id: &'static str, danger: DangerLevel) -> CategoryDescriptor {
    let module = ModuleId::new(id.split_once('.').unwrap().0.to_owned().leak()).unwrap();
    CategoryDescriptor::new(CategoryId::new(module, id).unwrap(), danger)
}

/// A finding of `category` about `path`, removed by moving it to the Trash.
pub(crate) fn finding(category: &CategoryDescriptor, path: impl Into<PathBuf>, sizes: Sizes) -> NewFinding {
    let path = path.into();
    NewFinding::new(
        category,
        path.to_string_lossy().into_owned(),
        Subject::Path(path.clone()),
        sizes,
        text("test.finding.title"),
        text("test.finding.reason"),
        Removal::MoveToTrash(path),
    )
}

/// Complete sizes of `size` bytes.
pub(crate) const fn bytes(size: u64) -> Sizes {
    Sizes::complete(ByteSize::new(size), ByteSize::new(size))
}

/// A walk that delivers a fixed list of folders to the interests that name them; held, it then waits until it is
/// released or cancelled. Its `commit` returns an empty state numbered by the commits so far, or a scripted error.
#[derive(Default)]
pub(crate) struct ScriptedWalk {
    folders: Vec<(String, PathBuf, Sizes)>,
    progress: Vec<WalkProgress>,
    unreadable: u64,
    kind: ScanKind,
    error: Option<WalkError>,
    commit_error: Option<WalkError>,
    panics: bool,
    deaf: bool,
    hold: Option<Gate>,
    roots: Arc<Mutex<Vec<PathBuf>>>,
    commits: Arc<Mutex<Vec<Vec<ClaimedPath>>>>,
    modes: Arc<Mutex<Vec<WalkMode>>>,
    states: Vec<Arc<dyn DiskState>>,
}

/// Lets a held [`ScriptedWalk`] finish.
#[derive(Clone, Debug, Default)]
pub(crate) struct Gate(Arc<AtomicBool>);

impl Gate {
    pub(crate) fn release(&self) {
        self.0.store(true, Ordering::Release);
    }

    fn is_released(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl ScriptedWalk {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// A walk that does not end until its gate is released or the scan is cancelled.
    pub(crate) fn held() -> (Self, Gate) {
        Self::new().hold()
    }

    /// This walk, held after it delivered its folders.
    pub(crate) fn hold(mut self) -> (Self, Gate) {
        let gate = Gate::default();
        self.hold = Some(gate.clone());
        (self, gate)
    }

    /// The walk finds a folder `name` at `path`.
    pub(crate) fn folder(mut self, name: &str, path: &str, sizes: Sizes) -> Self {
        self.folders.push((name.into(), path.into(), sizes));
        self
    }

    pub(crate) fn progress(mut self, entries: u64, bytes: u64) -> Self {
        self.progress.push(WalkProgress { entries, bytes: ByteSize::new(bytes) });
        self
    }

    /// The walk could not read `count` entries.
    pub(crate) const fn unreadable(mut self, count: u64) -> Self {
        self.unreadable = count;
        self
    }

    /// The walk reports that it read the disk as `kind`.
    pub(crate) const fn kind(mut self, kind: ScanKind) -> Self {
        self.kind = kind;
        self
    }

    /// The commit fails with `error`.
    pub(crate) fn failing_commit(mut self, error: WalkError) -> Self {
        self.commit_error = Some(error);
        self
    }

    /// The claims of every commit, in order.
    pub(crate) fn commits(&self) -> Arc<Mutex<Vec<Vec<ClaimedPath>>>> {
        Arc::clone(&self.commits)
    }

    /// The walk ends with `error` after delivering its folders.
    pub(crate) fn failing(mut self, error: WalkError) -> Self {
        self.error = Some(error);
        self
    }

    /// The walk panics after delivering its folders and its progress.
    pub(crate) const fn panicking(mut self) -> Self {
        self.panics = true;
        self
    }

    /// Held, the walk ignores a cancel and waits for its gate alone: a port that does not stop.
    pub(crate) const fn deaf(mut self) -> Self {
        self.deaf = true;
        self
    }

    /// The roots of every walk, in order.
    pub(crate) fn roots(&self) -> Arc<Mutex<Vec<PathBuf>>> {
        Arc::clone(&self.roots)
    }

    /// The commits return `states` in turn; past the last, an empty state numbered by the commits so far.
    pub(crate) fn committing(mut self, states: Vec<Arc<dyn DiskState>>) -> Self {
        self.states = states;
        self
    }

    /// The mode of every walk asked, in order.
    pub(crate) fn modes(&self) -> Arc<Mutex<Vec<WalkMode>>> {
        Arc::clone(&self.modes)
    }
}

impl std::fmt::Debug for ScriptedWalk {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ScriptedWalk").field("folders", &self.folders).finish_non_exhaustive()
    }
}

impl DiskWalk for ScriptedWalk {
    fn walk(
        &self,
        request: &WalkRequest<'_>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        self.roots.lock().unwrap().extend_from_slice(request.roots);
        self.modes.lock().unwrap().push(request.mode);
        for (name, path, sizes) in &self.folders {
            for (interest, wanted) in request.interests.iter() {
                if wanted.pattern() == &Pattern::FolderName(name.clone()) {
                    let (path, sizes) = (path.clone(), *sizes);
                    on_match(Match {
                        interest,
                        path,
                        kind: EntryKind::Folder,
                        sizes,
                        modified: None,
                        siblings: vec![],
                        marker: None,
                    });
                }
            }
        }
        self.progress.iter().for_each(|progress| on_progress(*progress));
        if self.panics {
            panic_now("the walk panics");
        }
        if let Some(gate) = &self.hold {
            while !gate.is_released() {
                if cancel.is_cancelled() && !self.deaf {
                    return Err(WalkError::Cancelled);
                }
                thread::yield_now();
            }
        }
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let unreadable = BTreeMap::from([(Unreadable::NoPermission, self.unreadable)]);
        let entries = u64::try_from(self.folders.len()).unwrap();
        Ok(WalkSummary { entries, unreadable, kind: self.kind, ..WalkSummary::default() })
    }

    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        let mut commits = self.commits.lock().unwrap();
        commits.push(claims.to_vec());
        if let Some(error) = &self.commit_error {
            return Err(error.clone());
        }
        if let Some(state) = self.states.get(commits.len() - 1) {
            return Ok(Arc::clone(state));
        }
        let id = StateId::new(u32::try_from(commits.len()).unwrap());
        Ok(Arc::new(FixedDiskState::new(id, Timestamp::from_secs(0), [])))
    }
}

/// The settings file in memory; it can refuse to be read or written.
#[derive(Debug, Default)]
pub(crate) struct MemorySettings {
    stored: Mutex<StoredSettings>,
    load_error: Option<StoreError>,
    save_error: Option<StoreError>,
    saves: Mutex<usize>,
}

impl MemorySettings {
    /// A file holding `values`, as plain JSON.
    pub(crate) fn with(values: &[(&str, serde_json::Value)]) -> Self {
        let values = values.iter().map(|(key, value)| ((*key).to_owned(), value.clone())).collect();
        Self { stored: Mutex::new(StoredSettings { values }), ..Self::default() }
    }

    /// A file that cannot be loaded: `load` answers `error`.
    pub(crate) fn unloadable(error: StoreError) -> Self {
        Self { load_error: Some(error), ..Self::default() }
    }

    /// Every save answers `error` and changes nothing.
    pub(crate) fn failing_saves(mut self, error: StoreError) -> Self {
        self.save_error = Some(error);
        self
    }

    pub(crate) fn stored(&self) -> StoredSettings {
        self.stored.lock().unwrap().clone()
    }

    /// How many saves were tried.
    pub(crate) fn saves(&self) -> usize {
        *self.saves.lock().unwrap()
    }
}

impl SettingsStore for MemorySettings {
    fn load(&self) -> Result<StoredSettings, StoreError> {
        match &self.load_error {
            Some(error) => Err(error.clone()),
            None => Ok(self.stored()),
        }
    }

    fn save(&self, settings: &StoredSettings) -> Result<(), StoreError> {
        *self.saves.lock().unwrap() += 1;
        if let Some(error) = &self.save_error {
            return Err(error.clone());
        }
        settings.clone_into(&mut self.stored.lock().unwrap());
        Ok(())
    }
}

/// An access probe that always gives the same answer.
#[derive(Debug)]
pub(crate) struct FixedAccess;

impl AccessProbe for FixedAccess {
    fn status(&self) -> AccessStatus {
        AccessStatus { full_disk_access: Access::Unknown }
    }
}

/// The seconds since the Unix epoch the fixture's clock starts at.
pub(crate) const START: u64 = 1_735_689_600;

/// What an engine under test is made of.
pub(crate) struct Fixture {
    modules: Vec<Box<dyn Module>>,
    walk: ScriptedWalk,
    pub(crate) settings: Arc<MemorySettings>,
    pub(crate) clock: Arc<FakeClock>,
}

impl Fixture {
    pub(crate) fn new(walk: ScriptedWalk) -> Self {
        Self { modules: Vec::new(), walk, settings: Arc::default(), clock: Arc::new(FakeClock::at(START)) }
    }

    pub(crate) fn module(mut self, module: TestModuleBuilder) -> Self {
        self.modules.push(module.boxed());
        self
    }

    pub(crate) fn settings(mut self, settings: MemorySettings) -> Self {
        self.settings = Arc::new(settings);
        self
    }

    pub(crate) fn engine(self) -> Engine {
        let mut registry = Registry::new().unwrap();
        for module in self.modules {
            registry.register(module).unwrap();
        }
        let ports = Ports {
            walk: Arc::new(self.walk),
            settings: self.settings,
            clock: self.clock,
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::new(FixedAccess),
        };
        Engine::new(registry, ports).unwrap()
    }
}

/// An engine without modules over `walk`.
pub(crate) fn engine(walk: ScriptedWalk) -> Engine {
    Fixture::new(walk).engine()
}

/// The events up to and including the next `ScanFinished`.
pub(crate) fn wait_for_finish(events: &Receiver<Event>) -> Vec<Event> {
    let mut received = Vec::new();
    loop {
        let event = events.recv_timeout(PATIENCE).unwrap();
        let finished = matches!(event, Event::ScanFinished { .. });
        received.push(event);
        if finished {
            return received;
        }
    }
}

/// Waits until `engine` is idle; fails after 10 s instead of hanging.
pub(crate) fn wait_until_idle(engine: &Engine) {
    let deadline = std::time::Instant::now() + PATIENCE;
    while engine.status() != Ok(fresher_core::EngineStatus::Idle) {
        assert!(std::time::Instant::now() < deadline, "the engine did not go idle");
        thread::yield_now();
    }
}

/// Runs one scan of `engine` to its end and gives its events.
pub(crate) fn scan(engine: &Engine) -> Vec<Event> {
    let events = engine.subscribe();
    engine.start_scan(crate::ScanMode::Auto).unwrap();
    wait_for_finish(&events)
}

/// The findings of every `FindingsAdded` in `events`, one list per event.
pub(crate) fn batches(events: &[Event]) -> Vec<Vec<Finding>> {
    let added = events.iter().filter_map(|event| match event {
        Event::FindingsAdded { findings, .. } => Some(findings.clone()),
        _ => None,
    });
    added.collect()
}

/// Panics with `message`: what a broken module or port does.
#[expect(clippy::panic, reason = "the tests check that a panic in a module or a port does not break the engine")]
pub(crate) fn panic_now(message: &str) -> ! {
    panic!("{message}");
}

/// Poisons `mutex` the only way there is: a thread panics while it holds the lock.
#[expect(clippy::panic, reason = "a lock is poisoned only by a panic while it is held")]
pub(crate) fn poison<T: Send>(mutex: &Mutex<T>) {
    let poisoned = thread::scope(|scope| {
        scope
            .spawn(|| {
                let _guard = mutex.lock();
                panic!("poison the lock");
            })
            .join()
    });
    assert!(poisoned.is_err());
}
