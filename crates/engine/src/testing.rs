//! Fakes for the engine's own tests: a module whose behaviour a test sets, a scripted walk, settings in memory.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fresher_core::{
    ByteSize, CancelToken, CategoryDescriptor, CategoryId, Claim, ClaimedPath, Cleaner, CleanupContext, ConfirmedPlan,
    DangerLevel, Depth, DiskState, DiskWalk, Ended, EntryFacts, EntryKind, Event, Finding, FindingSink, Interest,
    ItemDone, LeftOutItem, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId,
    Need, NewFinding, Pattern, PlaceSeen, PlanItem, PutBackOutcome, Rehearsal, Removal, ScanKind, SettingsStore, Sizes,
    StateId, StoreError, StoredSettings, Subject, Subtree, SubtreeQuery, Text, TextKey, Timestamp, ToolCommand, ToolId,
    TrashState, TrashedItem, Unreadable, WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
};
use fresher_testing::{
    FakeAccess, FakeCleaner, FakeClock, FakeCommandRunner, FakeSystemUi, FakeVolumes, FixedDiskState,
    MemoryCleanupJournal,
};

use crate::{CleanupPorts, Engine, Ports, Registry};

/// How long a test waits for an event before it fails: a guard against a hang, never a timing assumption.
const PATIENCE: Duration = Duration::from_secs(10);

type Finish = dyn Fn(&ModuleContext<'_>, &mut dyn FindingSink) -> Result<(), ModuleError> + Send + Sync;

/// A module built by a test: its categories, its interests, what it does with a match and in `finish`.
pub(crate) struct TestModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
    on_match: OnMatch,
    /// Reported on every match, before the module does what `on_match` says.
    match_left_out: Option<LeftOutItem>,
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
    group: ModuleGroup,
    categories: Vec<CategoryDescriptor>,
    tool_checks: Vec<ToolCommand>,
    problems: Vec<TextKey>,
    locales: &'static str,
    interests: Vec<Interest>,
    on_match: OnMatch,
    /// Reported on every match, before the module does what `on_match` says.
    match_left_out: Option<LeftOutItem>,
    finish: Box<Finish>,
    seen: Arc<Mutex<Vec<Match>>>,
}

impl TestModule {
    /// A module `id` of the group Development without categories, settings, texts or interests, that finds nothing.
    pub(crate) fn builder(id: &'static str) -> TestModuleBuilder {
        TestModuleBuilder {
            id: ModuleId::new(id).unwrap(),
            group: ModuleGroup::Development,
            categories: Vec::new(),
            tool_checks: Vec::new(),
            problems: Vec::new(),
            locales: "{}",
            interests: Vec::new(),
            on_match: OnMatch::Find,
            match_left_out: None,
            finish: Box::new(|_, _| Ok(())),
            seen: Arc::default(),
        }
    }
}

impl TestModuleBuilder {
    pub(crate) const fn group(mut self, group: ModuleGroup) -> Self {
        self.group = group;
        self
    }

    pub(crate) fn category(mut self, id: &'static str, danger: DangerLevel) -> Self {
        self.categories.push(CategoryDescriptor::new(CategoryId::new(self.id, id).unwrap(), danger));
        self
    }

    /// A category `id` that needs `needs`.
    pub(crate) fn needing(mut self, id: &'static str, danger: DangerLevel, needs: Vec<Need>) -> Self {
        self.categories.push(CategoryDescriptor::new(CategoryId::new(self.id, id).unwrap(), danger).with_needs(needs));
        self
    }

    /// The module checks `tool` with `args`.
    pub(crate) fn tool_check(mut self, tool: &'static str, args: &[&str]) -> Self {
        let args = args.iter().map(|&arg| arg.to_owned()).collect();
        self.tool_checks.push(ToolCommand::new(ToolId::new(tool).unwrap(), args).unwrap());
        self
    }

    /// The module declares `keys` as the problems it may report.
    pub(crate) fn problems(mut self, keys: &[&str]) -> Self {
        self.problems = keys.iter().map(|key| TextKey::new(*key).unwrap()).collect();
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

    /// The module wants what `interest` names; each match becomes one finding of its first category.
    pub(crate) fn interest(mut self, interest: Interest) -> Self {
        self.interests.push(interest);
        self
    }

    /// The module leaves out `item` on every match, before it finds, fails or panics.
    pub(crate) fn left_out_on_match(mut self, item: LeftOutItem) -> Self {
        self.match_left_out = Some(item);
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
        let mut descriptor = ModuleDescriptor::new(self.id, self.group, self.categories, vec![], locales).unwrap();
        for check in self.tool_checks {
            descriptor = descriptor.with_tool_check(check).unwrap();
        }
        descriptor = descriptor.with_problems(self.problems).unwrap();
        TestModule {
            descriptor,
            interests: self.interests,
            on_match: self.on_match,
            match_left_out: self.match_left_out,
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
        if let Some(item) = &self.match_left_out {
            out.left_out(item.clone());
        }
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
    /// What each found folder matches (a folder name, or a marker it holds), its path, sizes and facts.
    folders: Vec<(Pattern, PathBuf, Sizes, EntryFacts)>,
    progress: Vec<WalkProgress>,
    unreadable: u64,
    unreadable_by: BTreeMap<Unreadable, u64>,
    places: BTreeMap<usize, PlaceSeen>,
    kind: ScanKind,
    error: Option<WalkError>,
    commit_error: Option<WalkError>,
    panics: bool,
    deaf: bool,
    hold: Option<Gate>,
    /// Before the folder at each position, the walk waits for its gate.
    pauses: Vec<(usize, Gate)>,
    roots: Arc<Mutex<Vec<PathBuf>>>,
    avoided: Arc<Mutex<Vec<Vec<PathBuf>>>>,
    commits: Arc<Mutex<Vec<Vec<ClaimedPath>>>>,
    modes: Arc<Mutex<Vec<WalkMode>>>,
    states: Vec<Arc<dyn DiskState>>,
    subtrees: Vec<(PathBuf, Subtree)>,
    subtree_queries: Arc<Mutex<Vec<Vec<SubtreeQuery>>>>,
}

/// Lets a held [`ScriptedWalk`] finish.
#[derive(Clone, Debug, Default)]
pub(crate) struct Gate(Arc<AtomicBool>);

impl Gate {
    pub(crate) fn release(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Holds the next walks again, until the next release.
    pub(crate) fn close(&self) {
        self.0.store(false, Ordering::Release);
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

    /// The walk waits for the gate given back after the folders added so far, before those added next.
    pub(crate) fn pause(mut self) -> (Self, Gate) {
        let gate = Gate::default();
        self.pauses.push((self.folders.len(), gate.clone()));
        (self, gate)
    }

    /// Waits for `gate`; a cancel stops the wait unless the walk is deaf.
    fn wait(&self, gate: &Gate, cancel: &CancelToken) -> Result<(), WalkError> {
        while !gate.is_released() {
            if cancel.is_cancelled() && !self.deaf {
                return Err(WalkError::Cancelled);
            }
            thread::yield_now();
        }
        Ok(())
    }

    /// The walk finds a folder `name` at `path`, the user's own and removable.
    pub(crate) fn folder(self, name: &str, path: &str, sizes: Sizes) -> Self {
        self.folder_with(name, path, sizes, EntryFacts::REMOVABLE)
    }

    /// The walk finds a folder `name` at `path` with `facts`.
    pub(crate) fn folder_with(mut self, name: &str, path: &str, sizes: Sizes, facts: EntryFacts) -> Self {
        self.folders.push((Pattern::FolderName(name.into()), path.into(), sizes, facts));
        self
    }

    /// The walk finds a folder at `path` holding the file `marker`: a match of the interests in `marker`.
    pub(crate) fn marker(mut self, marker: &str, path: &str) -> Self {
        self.folders.push((Pattern::MarkerFile(marker.into()), path.into(), Sizes::ZERO, EntryFacts::REMOVABLE));
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

    /// The walk could not read `count` folders for `reason`.
    pub(crate) fn unreadable_by(mut self, reason: Unreadable, count: u64) -> Self {
        self.unreadable_by.insert(reason, count);
        self
    }

    /// The walk reports what it saw of the place of the interest at `interest` in the scan's interest set.
    pub(crate) fn seeing(mut self, interest: usize, seen: PlaceSeen) -> Self {
        self.places.insert(interest, seen);
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

    /// The places every walk was asked to avoid, one list per walk.
    pub(crate) fn avoided(&self) -> Arc<Mutex<Vec<Vec<PathBuf>>>> {
        Arc::clone(&self.avoided)
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

    /// `subtrees` answers a query of one of these folders with its subtree, any other with `None`.
    pub(crate) fn answering_subtrees(mut self, answers: Vec<(PathBuf, Subtree)>) -> Self {
        self.subtrees = answers;
        self
    }

    /// The queries of every `subtrees` call, in order.
    pub(crate) fn subtree_queries(&self) -> Arc<Mutex<Vec<Vec<SubtreeQuery>>>> {
        Arc::clone(&self.subtree_queries)
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
        self.avoided.lock().unwrap().push(request.avoid.to_vec());
        self.modes.lock().unwrap().push(request.mode);
        for (position, (pattern, path, sizes, facts)) in self.folders.iter().enumerate() {
            for (_, gate) in self.pauses.iter().filter(|(at, _)| *at == position) {
                self.wait(gate, cancel)?;
            }
            for (interest, wanted) in request.interests.iter() {
                if wanted.pattern() == pattern {
                    let (path, sizes) = (path.clone(), *sizes);
                    on_match(Match {
                        interest,
                        path,
                        kind: EntryKind::Folder,
                        sizes,
                        modified: None,
                        changed: None,
                        accessed: None,
                        inside: None,
                        siblings: vec![],
                        holds: Vec::new(),
                        marker: None,
                        facts: *facts,
                    });
                }
            }
        }
        self.progress.iter().for_each(|progress| on_progress(*progress));
        if self.panics {
            panic_now("the walk panics");
        }
        if let Some(gate) = &self.hold {
            self.wait(gate, cancel)?;
        }
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let mut unreadable = BTreeMap::from([(Unreadable::NoPermission, self.unreadable)]);
        unreadable.extend(&self.unreadable_by);
        let entries = u64::try_from(self.folders.len()).unwrap();
        let places = self.places.clone();
        Ok(WalkSummary { entries, unreadable, kind: self.kind, places, ..WalkSummary::default() })
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

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        self.subtree_queries.lock().unwrap().push(queries.to_vec());
        let answer = |query: &SubtreeQuery| {
            self.subtrees.iter().find(|(folder, _)| *folder == query.folder).map(|(_, subtree)| *subtree)
        };
        queries.iter().map(answer).collect()
    }
}

/// A cleaner that holds its first item in flight until its gate is released or the cleanup is cancelled, then acts
/// as the fake cleaner it wraps: what a cleanup looks like while it runs.
#[derive(Debug)]
pub(crate) struct HeldCleaner {
    inner: FakeCleaner,
    gate: Gate,
    holding: Gate,
}

impl HeldCleaner {
    pub(crate) fn new(inner: FakeCleaner) -> (Self, Gate) {
        let gate = Gate::default();
        (Self { inner, gate: gate.clone(), holding: Gate::default() }, gate)
    }

    /// Waits until the first item is in flight; fails after 10 s instead of hanging.
    pub(crate) fn wait_until_holding(&self) {
        let deadline = std::time::Instant::now() + PATIENCE;
        while !self.holding.is_released() {
            assert!(std::time::Instant::now() < deadline, "the cleaner never held an item");
            thread::yield_now();
        }
    }

    pub(crate) const fn inner(&self) -> &FakeCleaner {
        &self.inner
    }
}

impl Cleaner for HeldCleaner {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        self.inner.rehearse(roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        let mut first = true;
        self.inner.execute(plan, cx, cancel, &mut |item| {
            if first {
                self.holding.release();
            }
            while first && !self.gate.is_released() && !cancel.is_cancelled() {
                thread::yield_now();
            }
            first = false;
            out(item);
        })
    }

    fn protection(&self) -> fresher_core::Protection {
        self.inner.protection()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.inner.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.inner.put_back(item, cx)
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

/// The seconds since the Unix epoch the fixture's clock starts at.
pub(crate) const START: u64 = 1_735_689_600;

/// What an engine under test is made of.
pub(crate) struct Fixture {
    modules: Vec<Box<dyn Module>>,
    walk: ScriptedWalk,
    pub(crate) settings: Arc<MemorySettings>,
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) commands: Arc<FakeCommandRunner>,
    pub(crate) access: Arc<FakeAccess>,
    pub(crate) system_ui: Arc<FakeSystemUi>,
    /// The system of the fixture home `/home`: nothing installed, nothing running.
    pub(crate) system: fresher_core::SystemPorts,
    pub(crate) cleaner: Arc<dyn Cleaner>,
    pub(crate) journal: Arc<MemoryCleanupJournal>,
    pub(crate) volumes: Arc<FakeVolumes>,
}

impl Fixture {
    pub(crate) fn new(walk: ScriptedWalk) -> Self {
        Self {
            modules: Vec::new(),
            walk,
            settings: Arc::default(),
            clock: Arc::new(FakeClock::at(START)),
            commands: Arc::default(),
            access: Arc::default(),
            system_ui: Arc::default(),
            system: fresher_testing::fake_system(std::path::Path::new("/home")),
            cleaner: Arc::new(FakeCleaner::default()),
            journal: Arc::default(),
            volumes: Arc::default(),
        }
    }

    /// The cleaner the engine cleans through; the test keeps its own `Arc` to look at it.
    pub(crate) fn cleaner(mut self, cleaner: Arc<impl Cleaner + 'static>) -> Self {
        self.cleaner = cleaner;
        self
    }

    pub(crate) fn journal(mut self, journal: MemoryCleanupJournal) -> Self {
        self.journal = Arc::new(journal);
        self
    }

    pub(crate) fn volumes(mut self, volumes: FakeVolumes) -> Self {
        self.volumes = Arc::new(volumes);
        self
    }

    pub(crate) fn commands(mut self, commands: FakeCommandRunner) -> Self {
        self.commands = Arc::new(commands);
        self
    }

    pub(crate) fn access(mut self, access: FakeAccess) -> Self {
        self.access = Arc::new(access);
        self
    }

    pub(crate) fn system_ui(mut self, system_ui: FakeSystemUi) -> Self {
        self.system_ui = Arc::new(system_ui);
        self
    }

    pub(crate) fn system(mut self, system: fresher_core::SystemPorts) -> Self {
        self.system = system;
        self
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
            commands: self.commands,
            access: self.access,
            system_ui: self.system_ui,
            system: self.system,
        };
        let cleanup = CleanupPorts { cleaner: self.cleaner, journal: self.journal, volumes: self.volumes };
        Engine::with_cleanup(registry, ports, cleanup).unwrap()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{CancelToken, Claim, DiskWalk, Interest, InterestSet, ModuleId, Pattern, WalkMode, WalkRequest};

    use super::ScriptedWalk;

    /// The paths `walk` delivers to the interests `patterns`, each with its interest's index.
    fn delivered(walk: &ScriptedWalk, patterns: Vec<Pattern>) -> Vec<(usize, PathBuf)> {
        let interests: Vec<Interest> =
            patterns.into_iter().map(|pattern| Interest::new(pattern, Claim::Item, vec![]).unwrap()).collect();
        let mut set = InterestSet::new();
        set.add(ModuleId::new("demo").unwrap(), &interests);
        let request = WalkRequest { roots: &[], interests: &set, avoid: &[], places: &[], mode: WalkMode::Full };
        let mut found = Vec::new();
        walk.walk(
            &request,
            &CancelToken::new(),
            &mut |matched| found.push((matched.interest, matched.path)),
            &mut |_| {},
        )
        .unwrap();
        found
    }

    mod negative {
        use fresher_core::Pattern;

        use super::{ScriptedWalk, delivered};

        #[test]
        fn a_scripted_marker_does_not_match_a_folder_of_its_name() {
            let walk = ScriptedWalk::new().marker("package.json", "/home/app");

            let found = delivered(&walk, vec![Pattern::FolderName("package.json".into())]);

            assert!(found.is_empty(), "{found:?}");
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::Pattern;

        use super::{ScriptedWalk, delivered};
        use crate::testing::bytes;

        #[test]
        fn a_scripted_marker_matches_the_interests_in_it_beside_a_folder() {
            let walk = ScriptedWalk::new().marker("package.json", "/home/app").folder(
                "node_modules",
                "/home/app/node_modules",
                bytes(9),
            );

            let found = delivered(
                &walk,
                vec![Pattern::FolderName("node_modules".into()), Pattern::MarkerFile("package.json".into())],
            );

            assert_eq!(found, [(1, PathBuf::from("/home/app")), (0, PathBuf::from("/home/app/node_modules"))]);
        }
    }
}
