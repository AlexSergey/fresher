//! What the end-to-end tests of the real scanner share: a module with interests of every pattern, a fixture home
//! scanned by the engine with the portable reader, and the report a golden file holds.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_core::{
    Access, AccessProbe, AccessStatus, ByteSize, CategoryDescriptor, CategoryId, ChangeJournal, ChildOrder, Claim,
    DangerLevel, Event, FindingOrder, FindingSink, FindingsQuery, FolderReader, Interest, Locales, Match, Module,
    ModuleContext, ModuleDescriptor, ModuleError, ModuleId, NewFinding, Pattern, Removal, Subject, Text, TextKey,
    VolumeFacts, VolumeId,
};
use fresher_engine::{Engine, Ports, Registry};
use fresher_module_demo::DemoModule;
use fresher_scanner::{Scanner, ScannerOptions, StdReader, home_anchor};
use fresher_store::{DataFolder, FileScanCache, FileSettingsStore};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeVolumes, FixtureTree};
use serde_json::{Value, json};

use crate::support::fixtures;

const TEXTS: &str = r#"{
    "probe.found.name": "Probe matches",
    "probe.found.what": "Every entry the walk matched for the probe.",
    "probe.found.made-by": "The test tree.",
    "probe.found.if-removed": "Nothing: it is a test.",
    "probe.found.comes-back": "With every scan.",
    "probe.finding.title": "A match",
    "probe.finding.reason": "The walk matched it."
}"#;

/// A module with an interest of every pattern, nested claims among them, that finds each match.
struct Probe {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
}

impl Probe {
    fn new() -> Self {
        let id = ModuleId::new("probe").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(id, "probe.found").unwrap(), DangerLevel::Low);
        let interest = |pattern, claim, siblings: &[&str]| {
            Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
        };
        Self {
            descriptor: ModuleDescriptor::new(id, vec![category], vec![], Locales { en: TEXTS }).unwrap(),
            interests: vec![
                interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
                interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Whole, &[]),
                interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
                interest(Pattern::Extension("dmg".into()), Claim::Item, &[]),
            ],
        }
    }
}

impl Module for Probe {
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
            text("probe.finding.title"),
            text("probe.finding.reason"),
            Removal::MoveToTrash(found.path.clone()),
        ));
        Ok(())
    }

    fn finish(&self, _cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        Ok(())
    }
}

struct UnknownAccess;

impl AccessProbe for UnknownAccess {
    fn status(&self) -> AccessStatus {
        AccessStatus { full_disk_access: Access::Unknown }
    }
}

/// A fixture home and the engine that scans it as `~`, with `Documents` avoided as the app's standing list does, its
/// data folder (the settings and the scan cache) in a temporary folder.
pub(crate) struct Home {
    pub(crate) tree: FixtureTree,
    pub(crate) root: PathBuf,
    pub(crate) engine: Engine,
    pub(crate) data: tempfile::TempDir,
}

/// The fixture home `fixture` with an engine asking `journal` what changed.
pub(crate) fn home(fixture: &str, journal: Arc<dyn ChangeJournal>) -> Home {
    let tree = FixtureTree::from_toml(&fixtures().join("homes").join(fixture)).unwrap();
    let root = std::fs::canonicalize(tree.root()).unwrap();
    let data = tempfile::tempdir().unwrap();
    let engine = engine_on(&root, data.path(), journal, Arc::new(StdReader::new()));
    Home { tree, root, engine, data }
}

/// An engine scanning `root` as `~` with `reader`, its settings and scan cache in `data`, its clock fixed.
pub(crate) fn engine_on(
    root: &Path,
    data: &Path,
    journal: Arc<dyn ChangeJournal>,
    reader: Arc<dyn FolderReader>,
) -> Engine {
    let volumes = FakeVolumes::default()
        .with("/", VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) });
    let options = ScannerOptions {
        threads: 2.try_into().unwrap(),
        standing_avoid: vec![root.join("Documents")],
        ..ScannerOptions::default()
    };
    let clock = Arc::new(FakeClock::at(1_763_200_000));
    let cache = Arc::new(FileScanCache::new(DataFolder::open(data).unwrap()));
    let scanner = Scanner::new(reader, Arc::new(volumes), journal, cache, home_anchor(root), options)
        .with_clock(Arc::clone(&clock) as _);
    let mut registry = Registry::new().unwrap();
    registry.register(Box::new(DemoModule::new().unwrap())).unwrap();
    registry.register(Box::new(Probe::new())).unwrap();
    let ports = Ports {
        walk: Arc::new(scanner),
        settings: Arc::new(FileSettingsStore::new(DataFolder::open(data).unwrap())),
        clock,
        commands: Arc::new(FakeCommandRunner::default()),
        access: Arc::new(UnknownAccess),
    };
    Engine::new(registry, ports).unwrap()
}

/// Runs one scan and gives its events; fails after 30 s instead of hanging.
pub(crate) fn scan(engine: &Engine) -> Vec<Event> {
    let events: Receiver<Event> = engine.subscribe();
    engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();
    let mut seen = Vec::new();
    loop {
        let event = events.recv_timeout(Duration::from_secs(30)).unwrap();
        let done = matches!(event, Event::ScanFinished { .. });
        seen.push(event);
        if done {
            return seen;
        }
    }
}

/// The probe's findings by path: path relative to the home, sizes, siblings left out.
pub(crate) fn probe_findings(home: &Home) -> Vec<(String, u64)> {
    let page =
        home.engine.findings(&FindingsQuery::new(Some("probe".into()), None, FindingOrder::Found, 0, 1000)).unwrap();
    let mut found: Vec<_> = page
        .findings
        .iter()
        .filter_map(|finding| match finding.finding().subject() {
            Subject::Path(path) => Some((relative(&home.root, path), finding.finding().sizes().logical.bytes())),
            Subject::External { .. } => None,
        })
        .collect();
    found.sort();
    found
}

pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).map_or_else(|_| path.display().to_string(), |inside| inside.display().to_string())
}

/// What the golden file holds: the events but the findings batches, the findings without their ids sorted by
/// subject, the overview, and the committed state's top folders by size, without ids or folder times. The
/// home's path is written `~`.
pub(crate) fn report(home: &Home, events: &[Event]) -> Value {
    let events: Vec<Value> = events
        .iter()
        .filter(|event| !matches!(event, Event::FindingsAdded { .. }))
        .map(|event| serde_json::to_value(event).unwrap())
        .collect();
    let page = home.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
    let mut findings: Vec<Value> = page
        .findings
        .iter()
        .map(|finding| {
            let mut value = serde_json::to_value(finding).unwrap();
            value.as_object_mut().unwrap().remove("id");
            value
        })
        .collect();
    findings.sort_by_key(|finding| (finding["module"].to_string(), finding["subject"].to_string()));
    let state = home.engine.disk_state().unwrap().unwrap();
    let root = state.node(state.roots()[0]).unwrap();
    let children = state.children(root.id, ChildOrder::BySize, 0, 20).unwrap();
    // A folder's time is when the fixture made it, not part of the fixture.
    let without_id = |node: &fresher_core::NodeInfo| {
        let mut value = serde_json::to_value(node).unwrap();
        let fields = value.as_object_mut().unwrap();
        fields.remove("id");
        if node.kind == fresher_core::NodeKind::Folder {
            fields.remove("modified");
        }
        value
    };
    let report = json!({
        "events": events,
        "findings": findings,
        "overview": home.engine.overview().unwrap(),
        "state": {
            "root": without_id(&root),
            "children": children.nodes.iter().map(without_id).collect::<Vec<_>>(),
            "notScanned": state.unreadable(20).iter().map(|place| json!({
                "name": place.node.name,
                "reason": place.reason,
                "hiddenFolders": place.hidden_folders,
            })).collect::<Vec<_>>(),
        },
    });
    let text = serde_json::to_string(&report).unwrap().replace(&home.root.display().to_string(), "~");
    serde_json::from_str(&text).unwrap()
}
