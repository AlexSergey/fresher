//! What the dev-caches flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real
//! scanner, its reader seen through the facts lens, with the `dev-caches` module; the real settings file, scan cache
//! and journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder, as
//! sandbox mode composes it, with a fake Trash at `home/.Trash`. Xcode is installed; the open apps, the folders the
//! change journal names at each rescan and the files held open are what a flow sets. `restart` builds a new engine on
//! the same files, as when the app starts again.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/report.rs"]
mod report;

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
use fresher_core::{
    Access, ByteSize, Capability, Changes, CleanupId, CleanupPlanView, CleanupSelection, Confirmation, Event,
    FindingOrder, FindingsQuery, ItemOutcome, JournalPosition, LeftOutCount, ModuleId, Protection, RunningApp,
    ScanKind, Subject, SystemPorts, VolumeFacts, VolumeId,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, ScanMode};
use fresher_module_dev_caches::DevCachesModule;
use fresher_sandbox::Sandbox;
use fresher_scanner::{Scanner, ScannerOptions, StdReader, known_folders_anchor};
use fresher_store::{DataFolder, FileCleanupJournal, FileScanCache, FileSettingsStore};
use fresher_testing::{
    FIXTURE_NOW, FactsLens, FakeAccess, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeInstalledApps,
    FakeOpenFiles, FakeRunningApps, FakeSystemUi, FakeTrash, FakeVolumes, FixtureTree, StatedFacts, installed_app,
    running_app,
};
use serde_json::{Value, json};

use crate::support::fixtures;
use events::until;
use report::{sort_by_path, strip, trash_unnumbered};

/// The folders of a sandbox's layout besides what the fixture holds (`crates/sandbox/src/layout.rs`, checked by
/// `Sandbox::open`).
const LAYOUT: [&str; 9] = [
    "home/.Trash",
    "Applications",
    "System/Applications",
    ".fresher-data",
    "private/var/folders/xx/sandbox/C",
    "private/var/folders/xx/sandbox/0",
    "private/var/folders/xx/sandbox/T",
    "private/var/folders/xx/sandbox/X",
    "private/tmp",
];

/// Xcode's bundle id: the derived data projects are kept while it is open.
pub(crate) const XCODE: &str = "com.apple.dt.Xcode";

/// Where the journal of the one volume stands; every answer ends there too.
const AT: JournalPosition = JournalPosition { journal: 1, cursor: 10 };

/// The world an engine runs in besides the files.
#[derive(Debug, Default)]
pub(crate) struct Setup {
    /// The open apps at each listing, in order; the last one repeats. Nothing is open when empty.
    pub(crate) running: Vec<Vec<RunningApp>>,
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
}

/// Xcode open.
pub(crate) fn xcode_open() -> Vec<RunningApp> {
    vec![running_app("Xcode", &[XCODE]).unwrap()]
}

/// The fixture sandbox with its engine. `fixture` is the canonical sandbox folder.
pub(crate) struct Flow {
    _tree: FixtureTree,
    pub(crate) fixture: PathBuf,
    pub(crate) sandbox: Sandbox,
    pub(crate) stated: StatedFacts,
    pub(crate) open_files: Arc<FakeOpenFiles>,
    pub(crate) engine: Engine,
}

impl Flow {
    /// The sandbox holding `fixtures/homes/dev-caches-flow.toml`, its engine with nothing open and no change named.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("dev-caches-flow.toml")).unwrap();
        for folder in LAYOUT {
            fs::create_dir_all(tree.root().join(folder)).unwrap();
        }
        let tree = tree.with_sandbox().unwrap();
        let fixture = fs::canonicalize(tree.root()).unwrap();
        let sandbox = Sandbox::open(&fixture, user(&fixture)).unwrap();
        let stated = tree.stated().clone();
        let open_files = Arc::new(FakeOpenFiles::default());
        let engine = engine(&sandbox, &stated, &open_files, setup);
        Self { _tree: tree, fixture, sandbox, stated, open_files, engine }
    }

    /// A new engine on the same files in `setup`: the app started again.
    pub(crate) fn restart(&mut self, setup: Setup) {
        self.engine = engine(&self.sandbox, &self.stated, &self.open_files, setup);
    }

    /// Scans in full and gives the events up to the end.
    pub(crate) fn scan(&self) -> Vec<Event> {
        self.walk(ScanMode::Full)
    }

    /// Scans again, reading only what the journal names, and gives the events up to the end; fails when the scan
    /// was not a rescan.
    pub(crate) fn rescan(&self) -> Vec<Event> {
        let events = self.walk(ScanMode::Auto);
        let kind = events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        });
        assert!(matches!(kind, Some(ScanKind::Rescan { .. })), "{kind:?}");
        events
    }

    /// Shows the last scan again without reading the disk, and gives the events up to the end.
    pub(crate) fn restore(&self) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.restore().unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }))
    }

    fn walk(&self, mode: ScanMode) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.start_scan(mode).unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }))
    }

    /// The path of `relative` in the sandbox.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.fixture.join(relative)
    }

    /// The findings the engine answers: category and path relative to the sandbox, sorted.
    pub(crate) fn findings(&self) -> Vec<(String, String)> {
        let page = self.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let mut found: Vec<(String, String)> = page
            .findings
            .iter()
            .map(|finding| {
                let finding = finding.finding();
                let subject = match finding.subject() {
                    Subject::Path(path) => self.relative_path(path),
                    Subject::External { id, .. } => id.clone(),
                };
                (finding.category().as_str().to_owned(), subject)
            })
            .collect();
        found.sort();
        found
    }

    /// Confirms `view` to the Trash as the dialog shows it and waits for the end.
    pub(crate) fn clean(&self, view: &CleanupPlanView) -> (CleanupId, Vec<Event>) {
        let events = self.engine.subscribe();
        let summary = view.summary;
        let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                           "highest": summary.highest, "method": "trash" });
        let confirmation: Confirmation = serde_json::from_value(echo).unwrap();
        let cleanup = self.engine.confirm_cleanup(&confirmation).unwrap();
        (cleanup, until(&events, |event| matches!(event, Event::CleanupFinished { .. })))
    }

    /// How each item of `view` ended in the cleanup's `events`, by its path relative to the sandbox, sorted.
    pub(crate) fn outcomes(&self, view: &CleanupPlanView, events: &[Event]) -> Vec<(String, ItemOutcome)> {
        let paths: BTreeMap<_, _> = view
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .filter_map(|item| Some((item.finding, self.relative_path(item.path.as_ref()?))))
            .collect();
        let mut ended: Vec<(String, ItemOutcome)> = events
            .iter()
            .filter_map(|event| match event {
                Event::CleanupItems { items, .. } => Some(items),
                _ => None,
            })
            .flatten()
            .map(|item| (paths[&item.finding].clone(), item.outcome.clone()))
            .collect();
        ended.sort_by(|left, right| left.0.cmp(&right.0));
        ended
    }

    /// The journal's records as JSON, without the items' identities (device and file numbers are the machine's), the
    /// item records sorted by the path they moved, in the places the item records take.
    pub(crate) fn journal(&self) -> Vec<Value> {
        let Ok(text) = fs::read_to_string(self.sandbox.data_folder().join("cleanups.jsonl")) else { return Vec::new() };
        let mut records: Vec<Value> = text
            .lines()
            .map(|line| {
                let mut record: Value = serde_json::from_str(line).unwrap();
                strip(&mut record, "identity");
                record
            })
            .collect();
        let at: Vec<usize> = (0..records.len()).filter(|&at| records[at]["type"] == "item").collect();
        let mut items: Vec<Value> = at.iter().map(|&at| records[at].clone()).collect();
        items.sort_by_key(|item| item["trashed"][0]["original"].as_str().unwrap_or_default().to_owned());
        for (&at, item) in at.iter().zip(items) {
            records[at] = item;
        }
        records
    }

    /// `report` with the sandbox's path written `~`, the fake Trash's numbering dropped, the findings' ids and the
    /// items' indexes removed (the walk's order is not the report's), and the lists of paths sorted by path.
    pub(crate) fn relative(&self, report: &Value) -> Value {
        let text = serde_json::to_string(report).unwrap().replace(&self.fixture.display().to_string(), "~");
        let mut report: Value = serde_json::from_str(&trash_unnumbered(&text, ".Trash/")).unwrap();
        for key in ["finding", "findings", "index"] {
            strip(&mut report, key);
        }
        sort_by_path(&mut report);
        report
    }

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.fixture).unwrap_or(path).display().to_string()
    }
}

/// The selection of `categories`, whole.
pub(crate) fn select(categories: &[&str]) -> CleanupSelection {
    let categories: Vec<Value> =
        categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
    serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
}

/// What the `dev-caches` module left out in the events' `ModuleFinished`, as category, reason key and item count,
/// sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new("dev-caches").unwrap();
    let mut counts: Vec<(String, String, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module: finished, left_out, .. } if *finished == module => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| {
            (count.category.as_str().to_owned(), count.reason.key().as_str().to_owned(), count.items)
        })
        .collect();
    counts.sort();
    counts
}

fn user(folder: &Path) -> u32 {
    fs::metadata(folder).unwrap().uid()
}

fn engine(sandbox: &Sandbox, stated: &StatedFacts, open_files: &Arc<FakeOpenFiles>, setup: Setup) -> Engine {
    let data = sandbox.data_folder();
    let folder = DataFolder::open(&data).unwrap();
    let clock = Arc::new(FakeClock::at(FIXTURE_NOW));
    let volume = VolumeId::new(1);
    let facts = VolumeFacts { id: volume, device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let volumes = Arc::new(FakeVolumes::default().with("/", facts));
    let folders = Arc::new(sandbox.known_folders());
    let options = ScannerOptions { threads: 2.try_into().unwrap(), ..ScannerOptions::default() };
    let cache = Arc::new(FileScanCache::new(DataFolder::open(&data).unwrap()));
    let journal = setup.changed.iter().fold(FakeChangeJournal::default().at(volume, AT), |journal, changed| {
        let folders = changed.iter().map(|relative| sandbox.path().join(relative)).collect();
        journal.answering(Ok(Changes { folders, subtrees: Vec::new(), end: AT }))
    });
    let scanner = Scanner::new(
        Arc::new(FactsLens::new(StdReader::new(), stated.clone())),
        Arc::clone(&volumes) as _,
        Arc::new(journal),
        cache,
        known_folders_anchor(&*folders),
        options,
    )
    .with_clock(Arc::clone(&clock) as _);
    let mut answers = setup.running.into_iter();
    let first = FakeRunningApps::new(answers.next().unwrap_or_default());
    let running = Arc::new(answers.fold(first, FakeRunningApps::then));
    let xcode = installed_app(XCODE, "Xcode", "26.1").unwrap();
    let system = SystemPorts {
        known_folders: Arc::clone(&folders) as _,
        apps: Arc::new(FakeInstalledApps::new(vec![xcode])),
        running: Arc::clone(&running) as _,
    };
    let commands = Arc::new(FakeCommandRunner::default());
    let ports = Ports {
        walk: Arc::new(scanner),
        settings: Arc::new(FileSettingsStore::new(DataFolder::open(&data).unwrap())),
        clock: Arc::clone(&clock) as _,
        commands: Arc::clone(&commands) as _,
        access: Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)])),
        system_ui: Arc::new(FakeSystemUi::default()),
        system,
    };
    let cleaner_ports = CleanerPorts {
        trash: Arc::new(FakeTrash::new(sandbox.trash())),
        open_files: Arc::clone(open_files) as _,
        running,
        commands,
        clock,
    };
    let user = user(sandbox.path());
    let reach = Reach::SandboxFolder(SandboxFolder::new(sandbox.path(), user, &Protection::default()).unwrap());
    let protection = sandbox.places_protection().unwrap();
    let disk = Arc::new(DiskCleaner::new(reach, user, sandbox.home(), protection, cleaner_ports));
    let cleanup = CleanupPorts { cleaner: disk, journal: Arc::new(FileCleanupJournal::open(&folder)), volumes };
    let mut registry = Registry::new().unwrap();
    registry.register(Box::new(DevCachesModule::new().unwrap())).unwrap();
    Engine::with_cleanup(registry, ports, cleanup).unwrap()
}
