//! What the cleanup's end-to-end tests share: the fixture home `cleanup.toml`, an engine that scans it with the
//! reference walk and cleans it through the real `DiskCleaner` (`Reach::Sandbox`, a fake Trash next to the home) and
//! the real journal in a temporary data folder, and the reports a golden file holds.

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
use fresher_core::{
    Access, BeginRecord, ByteSize, CancelToken, Capability, CategoryDescriptor, CategoryId, Claim, Cleaner,
    CleanupContext, CleanupId, CleanupJournal, CleanupPlanView, CleanupSelection, Confirmation, ConfirmedPlan,
    DangerLevel, Depth, EndRecord, Ended, Event, FindingOrder, FindingSink, FindingsQuery, Interest, ItemDone,
    ItemRecord, JournalError, JournalPage, JournalState, Locales, Match, Module, ModuleContext, ModuleDescriptor,
    ModuleError, ModuleId, NewFinding, Pattern, PlanItem, Protection, PutBackOutcome, PutBackRecord, RecordedCleanup,
    Rehearsal, Removal, SANDBOX_MARKER, SettingUpdate, SettingValue, Subject, Text, TextKey, TrashState, TrashedItem,
    VolumeFacts, VolumeId,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, ScanMode};
use fresher_module_trial::TrialModule;
use fresher_store::{DataFolder, FileCleanupJournal, FileSettingsStore};
use fresher_testing::{
    FakeAccess, FakeClock, FakeCommandRunner, FakeOpenFiles, FakeSystemUi, FakeTrash, FakeVolumes, FixtureTree,
    NaiveWalk, TreeReport,
};
use serde_json::{Value, json};

use crate::support::fixtures;

/// The folders of the home that hold a real sandbox marker.
const MARKED: [&str; 3] = ["home/sandbox", "home/sandbox/fresher-e2e-build", "home/protected"];

const TEXTS: &str = r#"{
    "build.folders.name": "Build folders",
    "build.folders.what": "Folders a test build left behind.",
    "build.folders.made-by": "The test tree.",
    "build.folders.if-removed": "Nothing: it is a test.",
    "build.folders.comes-back": "With the next build.",
    "build.finding.title": "A build folder",
    "build.finding.reason": "The test's build left it."
}"#;

/// A module that claims every `fresher-e2e-build` folder as an item, so the walk goes on inside it: the trial cache
/// there is a finding of its own, nested in this one.
struct BuildModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
}

impl BuildModule {
    fn new() -> Self {
        let id = ModuleId::new("build").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(id, "build.folders").unwrap(), DangerLevel::Low);
        let interest = Interest::new(Pattern::FolderName("fresher-e2e-build".into()), Claim::Item, Vec::new()).unwrap();
        Self {
            descriptor: ModuleDescriptor::new(id, vec![category], vec![], Locales { en: TEXTS }).unwrap(),
            interests: vec![interest],
        }
    }
}

impl Module for BuildModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    fn on_match(&self, found: &Match, _cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let text = |key: &str| Text::new(TextKey::new(key).unwrap());
        out.push(NewFinding::new(
            &self.descriptor.categories()[0],
            found.path.to_string_lossy(),
            Subject::Path(found.path.clone()),
            found.sizes,
            text("build.finding.title"),
            text("build.finding.reason"),
            Removal::MoveToTrash(found.path.clone()),
        ));
        Ok(())
    }

    fn finish(&self, _cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        Ok(())
    }
}

/// The journal the engine writes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JournalKind {
    /// `cleanups.jsonl` in the data folder.
    File,
    /// The same file on a disk too full for the first record.
    DiskFull,
}

/// How a test sets the home up.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Setup {
    /// Whether the markers of `MARKED` are real; otherwise each is a file of the marker's name that is not one.
    pub(crate) marked: bool,
    pub(crate) journal: JournalKind,
    /// Whether the cleaner waits after its first item until the cleanup is cancelled.
    pub(crate) pause: bool,
}

impl Setup {
    pub(crate) const DEFAULT: Self = Self { marked: true, journal: JournalKind::File, pause: false };
}

/// The fixture home with its engine. `fixture` is the canonical root of the fixture tree: the scan root is
/// `fixture/home`, the outside target `fixture/outside`, the Trash `fixture/trash`.
pub(crate) struct Home {
    _tree: FixtureTree,
    pub(crate) fixture: PathBuf,
    pub(crate) engine: Engine,
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) data: tempfile::TempDir,
    /// Sends once the cleaner ended its first item, when the setup pauses there.
    pub(crate) paused: Option<Receiver<()>>,
}

pub(crate) fn home(setup: Setup) -> Home {
    let tree = FixtureTree::from_toml(&fixtures().join("homes").join("cleanup.toml")).unwrap();
    for folder in MARKED {
        if setup.marked {
            tree.mark_sandbox(Path::new(folder)).unwrap();
        } else {
            fs::write(tree.root().join(folder).join(SANDBOX_MARKER), "not a sandbox\n").unwrap();
        }
    }
    let fixture = fs::canonicalize(tree.root()).unwrap();
    let root = fixture.join("home");
    let data = tempfile::tempdir().unwrap();
    let folder = DataFolder::open(data.path()).unwrap();
    let clock = Arc::new(FakeClock::at(1_763_200_000));
    let commands = Arc::new(
        FakeCommandRunner::default()
            .answering("fresher-trial-tool version", "fresher-trial-tool 1.0\n")
            .answering("fresher-trial-tool list", "trial-object 4096\n"),
    );
    let protection = Protection::new(vec![root.join("protected")], vec![], vec![]).unwrap();
    let ports = CleanerPorts {
        trash: Arc::new(FakeTrash::new(fixture.join("trash"))),
        open_files: Arc::new(FakeOpenFiles::default()),
        running: Arc::new(fresher_testing::FakeRunningApps::default()),
        commands: Arc::clone(&commands) as _,
        clock: Arc::clone(&clock) as _,
    };
    let user = fs::metadata(&fixture).unwrap().uid();
    let disk = Arc::new(DiskCleaner::new(Reach::Sandbox, user, root.clone(), protection, ports));
    let (cleaner, paused): (Arc<dyn Cleaner>, _) = if setup.pause {
        let (sender, receiver) = std::sync::mpsc::channel();
        (Arc::new(PausingCleaner { inner: disk, reached: Mutex::new(sender) }), Some(receiver))
    } else {
        (disk, None)
    };
    let file = FileCleanupJournal::open(&folder);
    let journal: Arc<dyn CleanupJournal> = match setup.journal {
        JournalKind::File => Arc::new(file),
        JournalKind::DiskFull => Arc::new(FullDisk(file)),
    };
    let volumes = FakeVolumes::default()
        .with(&fixture, VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
    let cleanup = CleanupPorts { cleaner, journal, volumes: Arc::new(volumes) };
    let mut registry = Registry::new().unwrap();
    registry.register(Box::new(TrialModule::new().unwrap())).unwrap();
    registry.register(Box::new(BuildModule::new())).unwrap();
    let ports = Ports {
        walk: Arc::new(NaiveWalk::new(&root)),
        settings: Arc::new(FileSettingsStore::new(folder)),
        clock: Arc::clone(&clock) as _,
        commands,
        access: Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)])),
        system_ui: Arc::new(FakeSystemUi::default()),
        system: fresher_testing::fake_system(&root),
    };
    let engine = Engine::with_cleanup(registry, ports, cleanup).unwrap();
    // The reference walk reads its roots as they are written: the default `~` is given as the home itself.
    let roots = SettingValue::Folders(vec![root]);
    engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
    Home { _tree: tree, fixture, engine, clock, data, paused }
}

impl Home {
    /// Scans the home in full and waits for the end.
    pub(crate) fn scan(&self) {
        let events = self.engine.subscribe();
        self.engine.start_scan(ScanMode::Full).unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }));
    }

    /// The path of `relative` in the fixture tree.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.fixture.join(relative)
    }

    /// Every entry of the fixture tree (the home, the outside target, the Trash), sorted by path.
    pub(crate) fn tree(&self) -> TreeReport {
        TreeReport::of(&self.fixture).unwrap()
    }

    /// The selection of the whole path: the trial module's three categories, and the build folder by its id.
    pub(crate) fn selection(&self) -> CleanupSelection {
        let query = FindingsQuery::new(Some("build".into()), None, FindingOrder::Found, 0, 10);
        let builds = self.engine.findings(&query).unwrap().findings;
        assert_eq!(builds.len(), 1, "the build folder is one finding");
        let categories =
            ["trial.caches", "trial.kept", "trial.tool"].map(|category| json!({ "category": category, "except": [] }));
        serde_json::from_value(json!({ "findings": [builds[0].id()], "categories": categories })).unwrap()
    }

    /// Prepares the whole path's selection.
    pub(crate) fn prepare(&self) -> CleanupPlanView {
        self.engine.prepare_cleanup(&self.selection()).unwrap()
    }

    /// Confirms `view` as the dialog shows it, then waits for the cleanup's end; gives its id and its events.
    pub(crate) fn clean(&self, view: &CleanupPlanView, method: &str) -> (CleanupId, Vec<Event>) {
        let events = self.engine.subscribe();
        let cleanup = self.engine.confirm_cleanup(&echo(view, method)).unwrap();
        (cleanup, until_finished(&events))
    }

    /// The journal's records as JSON, without the items' identities (device and file numbers are the machine's).
    pub(crate) fn journal(&self) -> Vec<Value> {
        let Ok(text) = fs::read_to_string(self.data.path().join("cleanups.jsonl")) else { return Vec::new() };
        text.lines()
            .map(|line| {
                let mut record: Value = serde_json::from_str(line).unwrap();
                strip(&mut record, "identity");
                record
            })
            .collect()
    }

    /// The remaining findings: module, category and path relative to the fixture, sorted.
    pub(crate) fn findings(&self) -> Vec<Value> {
        let page = self.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let mut found: Vec<Value> = page
            .findings
            .iter()
            .map(|finding| {
                let finding = finding.finding();
                let subject = match finding.subject() {
                    Subject::Path(path) => path.display().to_string(),
                    Subject::External { tool, id } => format!("{}: {id}", tool.as_str()),
                };
                json!({ "category": finding.category().as_str(), "subject": subject })
            })
            .collect();
        found.sort_by_key(ToString::to_string);
        found
    }

    /// `report` with the fixture's path written `~`.
    pub(crate) fn relative(&self, report: &Value) -> Value {
        let text = serde_json::to_string(report).unwrap().replace(&self.fixture.display().to_string(), "~");
        serde_json::from_str(&text).unwrap()
    }
}

/// The confirmation of `view` with `method`, as the dialog sends it.
pub(crate) fn echo(view: &CleanupPlanView, method: &str) -> Confirmation {
    let summary = view.summary;
    let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                       "highest": summary.highest, "method": method });
    serde_json::from_value(echo).unwrap()
}

/// The events of a cleanup until its end.
pub(crate) fn until_finished(events: &Receiver<Event>) -> Vec<Event> {
    until(events, |event| matches!(event, Event::CleanupFinished { .. }))
}

/// Every event until `is_last`; fails after 30 s instead of hanging.
fn until(events: &Receiver<Event>, is_last: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut seen = Vec::new();
    loop {
        let event = events.recv_timeout(Duration::from_secs(30)).unwrap();
        let last = is_last(&event);
        seen.push(event);
        if last {
            return seen;
        }
    }
}

/// Removes every field named `key`, at any depth.
fn strip(value: &mut Value, key: &str) {
    match value {
        Value::Object(fields) => {
            fields.remove(key);
            fields.values_mut().for_each(|field| strip(field, key));
        }
        Value::Array(items) => items.iter_mut().for_each(|item| strip(item, key)),
        _ => {}
    }
}

/// The real cleaner, waiting after its first item until the cleanup is cancelled: the user's Stop, at a known point.
struct PausingCleaner {
    inner: Arc<DiskCleaner>,
    reached: Mutex<Sender<()>>,
}

impl Cleaner for PausingCleaner {
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
        self.inner.execute(plan, cx, cancel, &mut |done| {
            out(done);
            if std::mem::take(&mut first) {
                self.reached.lock().unwrap().send(()).unwrap();
                while !cancel.is_cancelled() {
                    thread::yield_now();
                }
            }
        })
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.inner.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.inner.put_back(item, cx)
    }
}

/// The real journal on a disk too full for the first record of a cleanup.
struct FullDisk(FileCleanupJournal);

impl CleanupJournal for FullDisk {
    fn state(&self) -> JournalState {
        self.0.state()
    }

    fn begin(&self, _record: &BeginRecord) -> Result<CleanupId, JournalError> {
        Err(JournalError::NoSpace)
    }

    fn item(&self, record: &ItemRecord) -> Result<(), JournalError> {
        self.0.item(record)
    }

    fn sync(&self) -> Result<(), JournalError> {
        self.0.sync()
    }

    fn end(&self, record: &EndRecord) -> Result<(), JournalError> {
        self.0.end(record)
    }

    fn put_back(&self, record: &PutBackRecord) -> Result<(), JournalError> {
        self.0.put_back(record)
    }

    fn cleanups(&self, offset: u32, limit: u32) -> Result<JournalPage, JournalError> {
        self.0.cleanups(offset, limit)
    }

    fn cleanup(&self, id: CleanupId) -> Result<Option<RecordedCleanup>, JournalError> {
        self.0.cleanup(id)
    }
}
