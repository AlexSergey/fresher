//! What the jetbrains flows share: a fixture home scanned by the real scanner with the `jetbrains` module, cleaned by
//! the real `DiskCleaner` (`Reach::Sandbox`, a fake Trash next to the home), with the settings, the scan cache and
//! the journal in a temporary data folder. The installed and the open apps are fakes a flow gives; `restart` builds a
//! new engine on the same files, as when the app starts again.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
use fresher_core::{
    Access, ByteSize, Capability, CleanupId, CleanupPlanView, CleanupSelection, Confirmation, Event, FindingOrder,
    FindingsQuery, InstalledApp, LeftOutCount, ModuleId, Protection, RunningApp, Subject, SystemPorts, VolumeFacts,
    VolumeId,
};
use fresher_engine::{CleanupPorts, Engine, Ports, Registry, ScanMode};
use fresher_module_jetbrains::JetBrainsModule;
use fresher_scanner::{Scanner, ScannerOptions, StdReader, known_folders_anchor};
use fresher_store::{DataFolder, FileCleanupJournal, FileScanCache, FileSettingsStore};
use fresher_testing::{
    FIXTURE_NOW, FakeAccess, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders,
    FakeOpenFiles, FakeRunningApps, FakeSystemUi, FakeTrash, FakeVolumes, FixtureTree, installed_app, running_app,
};
use serde_json::{Value, json};

use crate::support::fixtures;

/// The fixture home with its engine. `fixture` is the canonical root of the fixture tree: the scan root is
/// `fixture/home`, the Trash `fixture/trash`.
pub(crate) struct Flow {
    _tree: FixtureTree,
    pub(crate) fixture: PathBuf,
    pub(crate) engine: Engine,
    pub(crate) data: tempfile::TempDir,
}

/// The apps a flow's system has: the installed ones as the list answers, the open ones.
pub(crate) struct Apps {
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: Vec<RunningApp>,
}

impl Apps {
    pub(crate) fn installed(apps: Vec<InstalledApp>) -> Self {
        Self::with(FakeInstalledApps::new(apps))
    }

    pub(crate) fn with(installed: FakeInstalledApps) -> Self {
        Self { installed, running: Vec::new() }
    }

    pub(crate) fn running(self, running: Vec<RunningApp>) -> Self {
        Self { running, ..self }
    }
}

pub(crate) fn app(bundle_id: &str, name: &str, version: &str) -> InstalledApp {
    installed_app(bundle_id, name, version).unwrap()
}

pub(crate) fn open(name: &str, bundle_id: &str) -> RunningApp {
    running_app(name, &[bundle_id]).unwrap()
}

/// `WebStorm` 2025.3.1 and `GoLand` 2025.2.1: the versions in use of the flow home.
pub(crate) fn current_apps() -> Vec<InstalledApp> {
    vec![app("com.jetbrains.WebStorm", "WebStorm", "2025.3.1"), app("com.jetbrains.goland", "GoLand", "2025.2.1")]
}

impl Flow {
    /// The home `fixtures/homes/<home>.toml` with `apps`.
    pub(crate) fn new(home: &str, apps: Apps) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join(format!("{home}.toml"))).unwrap();
        let fixture = fs::canonicalize(tree.root()).unwrap();
        let data = tempfile::tempdir().unwrap();
        let engine = engine(&fixture, data.path(), apps);
        Self { _tree: tree, fixture, engine, data }
    }

    /// A new engine on the same files with `apps`: the app started again.
    pub(crate) fn restart(&mut self, apps: Apps) {
        self.engine = engine(&self.fixture, self.data.path(), apps);
    }

    /// Scans the home in full and gives the events up to the end.
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

    /// The path of `relative` in the fixture tree.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.fixture.join(relative)
    }

    /// The findings the engine answers: category and path relative to the fixture, sorted.
    pub(crate) fn findings(&self) -> Vec<(String, String)> {
        let page = self.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let mut found: Vec<(String, String)> = page
            .findings
            .iter()
            .map(|finding| {
                let finding = finding.finding();
                let subject = match finding.subject() {
                    Subject::Path(path) => path.strip_prefix(&self.fixture).unwrap_or(path).display().to_string(),
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

    /// The journal's records as JSON, without the items' identities (device and file numbers are the machine's). The
    /// item records come in the order the findings were found, which the real scanner's threads leave open: they are
    /// sorted by the path they moved, in the places the item records take.
    pub(crate) fn journal(&self) -> Vec<Value> {
        let Ok(text) = fs::read_to_string(self.data.path().join("cleanups.jsonl")) else { return Vec::new() };
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

    /// `report` with the fixture's path written `~`, and without what the real scanner's threads order at random: the
    /// findings' ids, the items' indexes and the
    /// Trash's numbering, and the lists of paths sorted by path.
    pub(crate) fn relative(&self, report: &Value) -> Value {
        let text = serde_json::to_string(report).unwrap().replace(&self.fixture.display().to_string(), "~");
        let mut report: Value = serde_json::from_str(&trash_unnumbered(&text)).unwrap();
        for key in ["finding", "findings", "index"] {
            strip(&mut report, key);
        }
        sort_by_path(&mut report);
        report
    }
}

/// The selection of the three places' categories and the removed IDEs' category, whole.
pub(crate) fn select_all() -> CleanupSelection {
    let categories = ["jetbrains.settings", "jetbrains.caches", "jetbrains.logs", "jetbrains.uninstalled"]
        .map(|category| json!({ "category": category, "except": [] }));
    serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
}

/// Every file under `folder` with its bytes, by path.
pub(crate) fn contents(folder: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                files.insert(path, bytes);
            }
        }
    }
    files
}

/// What the `jetbrains` module left out in the events' `ModuleFinished`, as category, reason key and item count.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let jetbrains = ModuleId::new("jetbrains").unwrap();
    let counts: Vec<&LeftOutCount> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module, left_out, .. } if *module == jetbrains => Some(left_out),
            _ => None,
        })
        .flatten()
        .collect();
    counts
        .iter()
        .map(|count| (count.category.as_str().to_owned(), count.reason.key().as_str().to_owned(), count.items))
        .collect()
}

/// The left-outs of `reason` as category and count, from `left_out`.
pub(crate) fn because<'a>(left_out: &'a [(String, String, u32)], reason: &str) -> Vec<(&'a str, u32)> {
    left_out
        .iter()
        .filter(|(_, key, _)| key == reason)
        .map(|(category, _, items)| (category.as_str(), *items))
        .collect()
}

fn engine(fixture: &Path, data: &Path, apps: Apps) -> Engine {
    // The flow home keeps its sandbox in `home/`; the homes of the module's own tests are rooted at the tree.
    let root = if fixture.join("home").is_dir() { fixture.join("home") } else { fixture.to_path_buf() };
    let folder = DataFolder::open(data).unwrap();
    let clock = Arc::new(FakeClock::at(FIXTURE_NOW));
    let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let volumes = Arc::new(FakeVolumes::default().with("/", facts));
    let folders = Arc::new(FakeKnownFolders::home_layout(&root));
    let options = ScannerOptions { threads: 1.try_into().unwrap(), ..ScannerOptions::default() };
    let cache = Arc::new(FileScanCache::new(DataFolder::open(data).unwrap()));
    let scanner = Scanner::new(
        Arc::new(StdReader::new()),
        Arc::clone(&volumes) as _,
        Arc::new(FakeChangeJournal::default()),
        cache,
        known_folders_anchor(&*folders),
        options,
    )
    .with_clock(Arc::clone(&clock) as _);
    let running = Arc::new(FakeRunningApps::new(apps.running));
    let system = SystemPorts {
        known_folders: Arc::clone(&folders) as _,
        apps: Arc::new(apps.installed),
        running: Arc::clone(&running) as _,
    };
    let commands = Arc::new(FakeCommandRunner::default());
    let ports = Ports {
        walk: Arc::new(scanner),
        settings: Arc::new(FileSettingsStore::new(DataFolder::open(data).unwrap())),
        clock: Arc::clone(&clock) as _,
        commands: Arc::clone(&commands) as _,
        access: Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)])),
        system_ui: Arc::new(FakeSystemUi::default()),
        system,
    };
    let cleaner_ports = CleanerPorts {
        trash: Arc::new(FakeTrash::new(fixture.join("trash"))),
        open_files: Arc::new(FakeOpenFiles::default()),
        running,
        commands,
        clock,
    };
    let user = fs::metadata(fixture).unwrap().uid();
    let protection = Protection::new(vec![], vec![], vec![]).unwrap();
    let disk = Arc::new(DiskCleaner::new(Reach::Sandbox, user, root, protection, cleaner_ports));
    let cleanup = CleanupPorts { cleaner: disk, journal: Arc::new(FileCleanupJournal::open(&folder)), volumes };
    let mut registry = Registry::new().unwrap();
    registry.register(Box::new(JetBrainsModule::new().unwrap())).unwrap();
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

/// `text` with the fake Trash's order number dropped from the names it gives (`trash/3-Name` is `trash/Name`).
fn trash_unnumbered(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("trash/") {
        let (before, after) = rest.split_at(at + "trash/".len());
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
