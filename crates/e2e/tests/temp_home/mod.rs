//! What the temp flows share: a fixture sandbox (its marker at the top, the home `home/`, the two temporary places
//! beside it, outside the scan root) scanned by the real scanner, its reader seen through the facts lens, with the
//! `temp` module; the real settings file, scan cache and journal in the sandbox's data folder; the real `DiskCleaner`
//! reaching only inside the sandbox folder, as sandbox mode composes it, with the sandbox's places protection and a
//! fake Trash at `home/.Trash`. `restart` builds a new engine on the same files, as when the app starts again.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
use fresher_core::{
    Access, ByteSize, Capability, CleanupId, CleanupPlanView, CleanupSelection, Confirmation, Event, FindingOrder,
    FindingsQuery, ItemOutcome, LeftOutCount, ModuleId, Protection, Subject, SystemPorts, VolumeFacts, VolumeId,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, ScanMode};
use fresher_module_temp::TempModule;
use fresher_sandbox::Sandbox;
use fresher_scanner::{Scanner, ScannerOptions, StdReader, known_folders_anchor};
use fresher_store::{DataFolder, FileCleanupJournal, FileScanCache, FileSettingsStore};
use fresher_testing::{
    FIXTURE_NOW, FactsLens, FakeAccess, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeInstalledApps,
    FakeOpenFiles, FakeRunningApps, FakeSystemUi, FakeTrash, FakeVolumes, FixtureTree, StatedFacts,
};
use serde_json::{Value, json};

use crate::support::fixtures;

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

/// The user's temporary folder and the shared one, relative to the sandbox.
pub(crate) const USER_TEMP: &str = "private/var/folders/xx/sandbox/T";
pub(crate) const SHARED_TEMP: &str = "private/tmp";
pub(crate) const CATEGORY: &str = "temp.old-files";
pub(crate) const DAY: u64 = 24 * 60 * 60;

/// What a sandbox holds besides its layout.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Home {
    /// `fixtures/homes/temp-flow.toml`: old and fresh items with stated times.
    Flow,
    /// Nothing: a flow makes its own entries on the real disk, and nothing is stated.
    Bare,
}

/// The system an engine runs with: the clock's moment, and whether the `temp` module is registered.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Setup {
    pub(crate) now: u64,
    pub(crate) temp: bool,
}

impl Setup {
    /// The clock at `now`, the module registered.
    pub(crate) const fn at(now: u64) -> Self {
        Self { now, temp: true }
    }

    /// The clock at `FIXTURE_NOW`, no module registered: no module names a place.
    pub(crate) const fn without_temp() -> Self {
        Self { now: FIXTURE_NOW, temp: false }
    }
}

/// The fixture sandbox with its engine. `fixture` is the canonical sandbox folder.
pub(crate) struct Flow {
    _tree: FixtureTree,
    pub(crate) fixture: PathBuf,
    pub(crate) sandbox: Sandbox,
    pub(crate) stated: StatedFacts,
    pub(crate) engine: Engine,
}

impl Flow {
    /// The sandbox holding `home`, its engine at `FIXTURE_NOW` with the module.
    pub(crate) fn new(home: Home) -> Self {
        let tree = match home {
            Home::Flow => FixtureTree::from_toml(&fixtures().join("homes").join("temp-flow.toml")).unwrap(),
            Home::Bare => FixtureTree::from_toml_str("").unwrap(),
        };
        for folder in LAYOUT {
            fs::create_dir_all(tree.root().join(folder)).unwrap();
        }
        let tree = tree.with_sandbox().unwrap();
        let fixture = fs::canonicalize(tree.root()).unwrap();
        let sandbox = Sandbox::open(&fixture, user(&fixture)).unwrap();
        let stated = tree.stated().clone();
        let engine = engine(&sandbox, &stated, Setup::at(FIXTURE_NOW));
        Self { _tree: tree, fixture, sandbox, stated, engine }
    }

    /// A new engine on the same files with `setup`: the app started again.
    pub(crate) fn restart(&mut self, setup: Setup) {
        self.engine = engine(&self.sandbox, &self.stated, setup);
    }

    /// Scans in full and gives the events up to the end.
    pub(crate) fn scan(&self) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.start_scan(ScanMode::Full).unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }))
    }

    /// Shows the last scan again without reading the disk, and gives the events up to the end.
    pub(crate) fn restore(&self) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.restore().unwrap();
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

    /// The paths of `view`'s items relative to the sandbox, sorted.
    pub(crate) fn planned(&self, view: &CleanupPlanView) -> Vec<String> {
        let mut paths: Vec<String> = view
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .filter_map(|item| Some(self.relative_path(item.path.as_ref()?)))
            .collect();
        paths.sort();
        paths
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
        let mut report: Value = serde_json::from_str(&trash_unnumbered(&text)).unwrap();
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

/// The selection of the module's one category, whole.
pub(crate) fn select_all() -> CleanupSelection {
    serde_json::from_value(json!({ "findings": [], "categories": [{ "category": CATEGORY, "except": [] }] })).unwrap()
}

/// Every file at or under `at` with its bytes, by path.
pub(crate) fn contents(at: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![at.to_path_buf()];
    while let Some(next) = pending.pop() {
        if !next.is_dir() {
            files.insert(next.clone(), fs::read(&next).unwrap());
            continue;
        }
        for entry in fs::read_dir(&next).unwrap() {
            pending.push(entry.unwrap().path());
        }
    }
    files
}

/// What the `temp` module left out in the events' `ModuleFinished`, as reason key and item count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, u32)> {
    let temp = ModuleId::new("temp").unwrap();
    let mut counts: Vec<(String, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module, left_out, .. } if *module == temp => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| {
            assert_eq!(count.category.as_str(), CATEGORY);
            (count.reason.key().as_str().to_owned(), count.items)
        })
        .collect();
    counts.sort();
    counts
}

/// The newest status change of `paths` (the real disk's), in seconds.
pub(crate) fn newest_change(paths: &[PathBuf]) -> u64 {
    paths.iter().map(|path| u64::try_from(fs::symlink_metadata(path).unwrap().ctime()).unwrap()).max().unwrap()
}

fn user(folder: &Path) -> u32 {
    fs::metadata(folder).unwrap().uid()
}

fn engine(sandbox: &Sandbox, stated: &StatedFacts, setup: Setup) -> Engine {
    let data = sandbox.data_folder();
    let folder = DataFolder::open(&data).unwrap();
    let clock = Arc::new(FakeClock::at(setup.now));
    let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let volumes = Arc::new(FakeVolumes::default().with("/", facts));
    let folders = Arc::new(sandbox.known_folders());
    let options = ScannerOptions { threads: 1.try_into().unwrap(), ..ScannerOptions::default() };
    let cache = Arc::new(FileScanCache::new(DataFolder::open(&data).unwrap()));
    let scanner = Scanner::new(
        Arc::new(FactsLens::new(StdReader::new(), stated.clone())),
        Arc::clone(&volumes) as _,
        Arc::new(FakeChangeJournal::default()),
        cache,
        known_folders_anchor(&*folders),
        options,
    )
    .with_clock(Arc::clone(&clock) as _);
    let running = Arc::new(FakeRunningApps::new(Vec::new()));
    let system = SystemPorts {
        known_folders: Arc::clone(&folders) as _,
        apps: Arc::new(FakeInstalledApps::new(Vec::new())),
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
        open_files: Arc::new(FakeOpenFiles::default()),
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
    if setup.temp {
        registry.register(Box::new(TempModule::new().unwrap())).unwrap();
    }
    Engine::with_cleanup(registry, ports, cleanup).unwrap()
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

/// `text` with the fake Trash's order number dropped from the names it gives (`.Trash/3-Name` is `.Trash/Name`).
fn trash_unnumbered(text: &str) -> String {
    const TRASH: &str = ".Trash/";
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(TRASH) {
        let (before, after) = rest.split_at(at + TRASH.len());
        out.push_str(before);
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let named = after.get(digits..).and_then(|tail| tail.strip_prefix('-'));
        rest = named.filter(|_| digits > 0).unwrap_or(after);
    }
    out.push_str(rest);
    out
}

/// Sorts every list of objects that all name a place (`path` or `original`), by it, at any depth.
fn sort_by_path(value: &mut Value) {
    fn place(item: &Value) -> Option<String> {
        item["path"].as_str().or_else(|| item["original"].as_str()).map(str::to_owned)
    }
    match value {
        Value::Object(fields) => fields.values_mut().for_each(sort_by_path),
        Value::Array(items) => {
            items.iter_mut().for_each(sort_by_path);
            if items.iter().all(|item| place(item).is_some()) {
                items.sort_by_key(|item| place(item).unwrap_or_default());
            }
        }
        _ => {}
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
