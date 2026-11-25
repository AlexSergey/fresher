//! What the shell's tests share: an engine of fakes over the app's real modules and the composition's cleaner, in a
//! temporary data folder.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use fresher_cleaner::{CleanerPorts, Reach};
use fresher_core::{
    AccessProbe, ByteSize, CancelToken, ClaimedPath, Cleaner, CleanupContext, CleanupPlanView, CleanupSelection, Clock,
    CommandRunner, Confirmation, ConfirmedPlan, DangerLevel, Depth, DiskState, DiskWalk, Ended, Event, ItemDone, Match,
    PlanItem, PutBackOutcome, Rehearsal, SANDBOX_MARKER, SettingUpdate, SettingValue, StateId, Subtree, SubtreeQuery,
    Timestamp, Trash, TrashState, TrashedItem, WalkError, WalkProgress, WalkRequest, WalkSummary,
};
use fresher_engine::{CleanupPorts, Engine, Ports};
use fresher_store::{DataFolder, FileSettingsStore};
use fresher_testing::{
    FakeClock, FakeCommandRunner, FakeOpenFiles, FakeSystemUi, FakeTrash, FakeVolumes, FixedDiskState, FixtureTree,
    NaiveWalk,
};
use serde_json::{Value, json};
use tauri::Manager;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::test::{MockRuntime, mock_app};
use tempfile::TempDir;

use crate::adapters::UnknownAccess;
use crate::compose::{cleanup, registry, user_temp};
use crate::events::EventForwarder;

/// A walk that finds nothing and commits an empty state: the shell's tests drive the commands, not the disk.
#[derive(Debug)]
pub(crate) struct NoWalk;

impl DiskWalk for NoWalk {
    fn walk(
        &self,
        _request: &WalkRequest<'_>,
        _cancel: &CancelToken,
        _on_match: &mut dyn FnMut(Match),
        _on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        Ok(WalkSummary::default())
    }

    fn commit(&self, _claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        Ok(Arc::new(FixedDiskState::new(StateId::new(0), Timestamp::from_secs(0), [])))
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        vec![None; queries.len()]
    }
}

/// A walk that holds the scan until it is cancelled.
#[derive(Debug)]
pub(crate) struct HeldWalk;

impl DiskWalk for HeldWalk {
    fn walk(
        &self,
        _request: &WalkRequest<'_>,
        cancel: &CancelToken,
        _on_match: &mut dyn FnMut(Match),
        _on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        while !cancel.is_cancelled() {
            thread::yield_now();
        }
        Err(WalkError::Cancelled)
    }

    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        NoWalk.commit(claims)
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        vec![None; queries.len()]
    }
}

/// The composition's cleaner, holding each cleanup until it is cancelled: a cleanup still running when the window
/// closes.
struct HeldCleaner(Arc<dyn Cleaner>);

impl Cleaner for HeldCleaner {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        self.0.rehearse(roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        while !cancel.is_cancelled() {
            thread::yield_now();
        }
        self.0.execute(plan, cx, cancel, out)
    }

    fn protection(&self) -> fresher_core::Protection {
        self.0.protection()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.0.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.0.put_back(item, cx)
    }
}

/// A mocked Tauri app managing an engine of fakes; the data folder and the Trash live as long as the app.
pub(crate) struct Shell {
    pub(crate) app: tauri::App<MockRuntime>,
    folder: TempDir,
}

/// The Trash a shell's cleaner is given.
#[derive(Clone, Copy)]
enum ShellTrash {
    /// A fake Trash in the shell's own folder ([`Shell::trash`]).
    Fake,
    /// The development build's Trash, which refuses every call.
    #[cfg(debug_assertions)]
    Refusing,
}

impl Shell {
    /// The app's modules with a walk that finds nothing.
    pub(crate) fn new() -> Self {
        Self::with_access(Arc::new(UnknownAccess))
    }

    /// As `new`, with `access` as the system's permissions in place of `UnknownAccess`.
    pub(crate) fn with_access(access: Arc<dyn AccessProbe>) -> Self {
        Self::with_walk(Arc::new(NoWalk), None, false, access, Reach::Sandbox, ShellTrash::Fake).accepted()
    }

    /// The app's modules with a scan that runs until it is cancelled.
    pub(crate) fn held() -> Self {
        Self::with_walk(Arc::new(HeldWalk), None, false, Arc::new(UnknownAccess), Reach::Sandbox, ShellTrash::Fake)
            .accepted()
    }

    /// The app's modules scanning `home` (a fixture tree, the only scan root) with the reference walk; a cleanup goes
    /// through the composition's cleaner reaching only below a sandbox folder, and the journal, with a Trash in the
    /// shell's own folder.
    pub(crate) fn cleaning(home: &Path) -> Self {
        Self::cleaning_unaccepted(home).accepted()
    }

    /// As `cleaning`, before the user accepted the agreement.
    pub(crate) fn cleaning_unaccepted(home: &Path) -> Self {
        Self::scanning(home, false, Reach::Sandbox, ShellTrash::Fake)
    }

    /// As `cleaning`, with each cleanup held until it is cancelled.
    pub(crate) fn cleaning_held(home: &Path) -> Self {
        Self::scanning(home, true, Reach::Sandbox, ShellTrash::Fake).accepted()
    }

    /// As `cleaning`, with the reach of a development build (the dry run) and the fake Trash, so a test can see it
    /// stays empty.
    #[cfg(debug_assertions)]
    pub(crate) fn dry_run(home: &Path) -> Self {
        Self::scanning(home, false, crate::compose::reach(), ShellTrash::Fake).accepted()
    }

    /// As `cleaning`, with the development build's Trash, which refuses: the second wall alone.
    #[cfg(debug_assertions)]
    pub(crate) fn refusing_trash(home: &Path) -> Self {
        Self::scanning(home, false, Reach::Sandbox, ShellTrash::Refusing).accepted()
    }

    fn scanning(home: &Path, held: bool, reach: Reach, trash: ShellTrash) -> Self {
        let walk = Arc::new(NaiveWalk::new(home));
        let shell = Self::with_walk(walk, Some(home), held, Arc::new(UnknownAccess), reach, trash);
        // The reference walk reads its roots as they are written: the default `~` is given as the home itself.
        let roots = SettingValue::Folders(vec![home.to_path_buf()]);
        shell.engine().update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
        shell
    }

    fn with_walk(
        walk: Arc<dyn DiskWalk>,
        home: Option<&Path>,
        held: bool,
        access: Arc<dyn AccessProbe>,
        reach: Reach,
        trash: ShellTrash,
    ) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let data = DataFolder::open(folder.path().join("data")).unwrap();
        let home = home.map_or_else(|| folder.path().join("home"), Path::to_path_buf);
        fs::create_dir_all(&home).unwrap();
        let clock: Arc<dyn Clock> = Arc::new(FakeClock::at(1_700_000_000));
        let commands: Arc<dyn CommandRunner> = Arc::new(FakeCommandRunner::default());
        let trash: Arc<dyn Trash> = match trash {
            ShellTrash::Fake => Arc::new(FakeTrash::new(folder.path().join("trash"))),
            #[cfg(debug_assertions)]
            ShellTrash::Refusing => Arc::new(crate::dry_run::DryRunTrash),
        };
        let cleaner = CleanerPorts {
            trash,
            open_files: Arc::new(FakeOpenFiles::default()),
            running: Arc::new(fresher_testing::FakeRunningApps::default()),
            commands: Arc::clone(&commands),
            clock: Arc::clone(&clock),
        };
        // The engine reads the volumes of the scan roots from the cleanup's volumes: the home sits on a 1 000-byte disk.
        let disk = FakeVolumes::disk("Test disk", ByteSize::new(1_000), ByteSize::new(400), ByteSize::new(600));
        let volumes = Arc::new(FakeVolumes::default().with_space(&home, disk));
        // The test's home lies in the user's temporary folder: the real Mac's list carves it out, as the app's does.
        let finder = Arc::new(fresher_testing::FakeFinderSettings::new(fresher_core::AutoEmpty::Unknown));
        let mut cleanup = cleanup(&home, user_temp().as_deref(), &data, cleaner, volumes, finder, reach).unwrap();
        if held {
            cleanup = CleanupPorts { cleaner: Arc::new(HeldCleaner(cleanup.cleaner)), ..cleanup };
        }
        let ports = Ports {
            walk,
            settings: Arc::new(FileSettingsStore::new(data.clone())),
            clock,
            commands,
            access,
            system_ui: Arc::new(FakeSystemUi::default()),
            system: fresher_testing::fake_system(&home),
        };
        let app = mock_app();
        app.manage(
            Engine::with_cleanup(registry().unwrap(), ports, cleanup, fresher_core::HighLevel::Offered).unwrap(),
        );
        app.manage(EventForwarder::default());
        app.manage(data);
        Self { app, folder }
    }

    /// The shell after the user accepted the agreement, as before their first cleanup.
    fn accepted(self) -> Self {
        self.engine().accept_agreement(fresher_core::AGREEMENT_VERSION).unwrap();
        self
    }

    pub(crate) fn engine(&self) -> tauri::State<'_, Engine> {
        self.app.state::<Engine>()
    }

    pub(crate) fn forwarder(&self) -> tauri::State<'_, EventForwarder> {
        self.app.state::<EventForwarder>()
    }

    pub(crate) fn data_folder(&self) -> tauri::State<'_, DataFolder> {
        self.app.state::<DataFolder>()
    }

    /// The folder the fake Trash moves items into.
    pub(crate) fn trash(&self) -> PathBuf {
        self.folder.path().join("trash")
    }
}

/// Every event until `ScanFinished`; fails the test after 10 s instead of hanging.
pub(crate) fn until_finished<T>(events: &Receiver<T>, is_finish: impl Fn(&T) -> bool) -> Vec<T> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut seen = Vec::new();
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let event = events.recv_timeout(left).unwrap();
        let done = is_finish(&event);
        seen.push(event);
        if done {
            return seen;
        }
    }
}

pub(crate) fn is_scan_finished(event: &Event) -> bool {
    matches!(event, Event::ScanFinished { .. })
}

/// A channel as the webview's: every event it is sent arrives on the receiver as JSON.
pub(crate) fn recording_channel() -> (Channel<Event>, Receiver<Value>) {
    let (sender, received) = mpsc::channel();
    let channel = Channel::new(move |body| {
        if let InvokeResponseBody::Json(json) = body {
            // A test that stopped listening drops its receiver; the events after that go nowhere.
            let _ = sender.send(serde_json::from_str::<Value>(&json).unwrap());
        }
        Ok(())
    });
    (channel, received)
}

pub(crate) fn is_cleanup_finished(event: &Event) -> bool {
    matches!(event, Event::CleanupFinished { .. })
}

/// The trial module's home (`fixtures/homes/trial.toml`): in `sandbox/`, a folder to move to the Trash, one to empty
/// and a file to keep. `marked` writes the sandbox marker there; otherwise a file of the marker's name that is not one,
/// so the trial module still offers the folders and only the cleaner can refuse them.
pub(crate) fn trial_home(marked: bool) -> FixtureTree {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/homes/trial.toml");
    let tree = FixtureTree::from_toml(&fixture).unwrap();
    if marked {
        tree.mark_sandbox(Path::new("sandbox")).unwrap();
    } else {
        fs::write(tree.root().join("sandbox").join(SANDBOX_MARKER), "not a sandbox\n").unwrap();
    }
    tree
}

/// Scans the shell's home and waits for the scan's end.
pub(crate) fn scan(shell: &Shell) {
    let events = shell.engine().subscribe();
    shell.engine().start_scan(fresher_engine::ScanMode::Full).unwrap();
    until_finished(&events, is_scan_finished);
}

/// The trial module's caches, picked whole.
pub(crate) fn caches() -> CleanupSelection {
    serde_json::from_value(json!({ "findings": [], "categories": [{ "category": "trial.caches", "except": [] }] }))
        .unwrap()
}

/// The confirmation of `view` to the Trash, as the dialog shows it.
pub(crate) fn echo(view: &CleanupPlanView) -> Confirmation {
    let summary = view.summary;
    let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                       "highest": summary.highest, "method": "trash",
                       "highRiskUnderstood": summary.highest == DangerLevel::High });
    serde_json::from_value(echo).unwrap()
}

/// Every entry below `root`, sorted, with the length of each file (`None` for a folder): what a test compares before
/// and after a cleanup that must change nothing.
#[cfg(debug_assertions)]
pub(crate) fn listing(root: &Path) -> Vec<(PathBuf, Option<u64>)> {
    let mut entries = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                folders.push(path.clone());
                entries.push((path, None));
            } else {
                entries.push((path, Some(metadata.len())));
            }
        }
    }
    entries.sort();
    entries
}
