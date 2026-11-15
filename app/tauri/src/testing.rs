//! What the shell's tests share: an engine of fakes over the app's real modules, in a temporary data folder.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use fresher_core::{CancelToken, DiskWalk, Event, InterestSet, Match, WalkError, WalkProgress, WalkSummary};
use fresher_engine::{Engine, Ports};
use fresher_store::{DataFolder, FileSettingsStore};
use fresher_testing::{FakeClock, FakeCommandRunner};
use serde_json::Value;
use tauri::Manager;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::test::{MockRuntime, mock_app};
use tempfile::TempDir;

use crate::adapters::{NoWalk, UnknownAccess};
use crate::compose::registry;
use crate::events::EventForwarder;

/// A walk that holds the scan until it is cancelled.
#[derive(Debug)]
pub(crate) struct HeldWalk;

impl DiskWalk for HeldWalk {
    fn walk(
        &self,
        _roots: &[PathBuf],
        _interests: &InterestSet,
        cancel: &CancelToken,
        _on_match: &mut dyn FnMut(Match),
        _on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        while !cancel.is_cancelled() {
            thread::yield_now();
        }
        Err(WalkError::Cancelled)
    }
}

/// A mocked Tauri app managing an engine of fakes; the data folder lives as long as the app.
pub(crate) struct Shell {
    pub(crate) app: tauri::App<MockRuntime>,
    _folder: TempDir,
}

impl Shell {
    /// The app's modules with a walk that finds nothing.
    pub(crate) fn new() -> Self {
        Self::with_walk(Arc::new(NoWalk))
    }

    /// The app's modules with a scan that runs until it is cancelled.
    pub(crate) fn held() -> Self {
        Self::with_walk(Arc::new(HeldWalk))
    }

    fn with_walk(walk: Arc<dyn DiskWalk>) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let ports = Ports {
            walk,
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(folder.path()).unwrap())),
            clock: Arc::new(FakeClock::at(1_700_000_000)),
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::new(UnknownAccess),
        };
        let app = mock_app();
        app.manage(Engine::new(registry().unwrap(), ports).unwrap());
        app.manage(EventForwarder::default());
        Self { app, _folder: folder }
    }

    pub(crate) fn engine(&self) -> tauri::State<'_, Engine> {
        self.app.state::<Engine>()
    }

    pub(crate) fn forwarder(&self) -> tauri::State<'_, EventForwarder> {
        self.app.state::<EventForwarder>()
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
