//! What the Apps tests build an engine of: the home `/home`, the apps folders `/Applications` and
//! `/System/Applications`, the installed and running apps a test states, and the states the scans commit.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    Access, AppSummary, AppsQuery, BundleFacts, Capability, DiskState, InstalledApp, InstalledApps, KnownFolder,
    NodeKind, RunningApps, StateId, SystemPorts, Timestamp,
};
use fresher_testing::{
    FakeAccess, FakeInstallRecords, FakeKnownFolders, FakeRunningApps, FixedDiskState, NoFiles, bundle_facts,
    installed_app,
};

use crate::Engine;
use crate::testing::{Fixture, ScriptedWalk, bytes, scan};

/// The known folders of the tests: the home's, and both apps folders of the system.
pub(crate) fn folders() -> FakeKnownFolders {
    FakeKnownFolders::home_layout("/home")
        .with(KnownFolder::Applications, "/Applications")
        .with(KnownFolder::SystemApplications, "/System/Applications")
}

/// The system of the tests with `apps` installed and `running` running.
pub(crate) fn system(apps: Arc<dyn InstalledApps>, running: Arc<dyn RunningApps>) -> SystemPorts {
    SystemPorts {
        known_folders: Arc::new(folders()),
        apps,
        running,
        installs: Arc::new(FakeInstallRecords::default()),
        files: Arc::new(NoFiles),
    }
}

/// The state `id` of the files and folders `entries` (absolute paths, with their kinds and bytes), with every folder
/// that holds them.
pub(crate) fn state(id: u32, entries: &[(&str, NodeKind, u64)]) -> FixedDiskState {
    let mut folders = BTreeSet::new();
    let mut leaves = Vec::new();
    for &(path, kind, size) in entries {
        let path = PathBuf::from(path);
        folders.extend(path.ancestors().skip(1).filter(|at| *at != Path::new("/")).map(Path::to_path_buf));
        leaves.push((path, kind, bytes(size)));
    }
    let leaves: Vec<_> = leaves.into_iter().filter(|(path, _, _)| !folders.contains(path)).collect();
    let folders = folders.into_iter().map(|path| (path, NodeKind::Folder, bytes(0)));
    FixedDiskState::new(StateId::new(id), Timestamp::from_secs(0), folders.chain(leaves))
}

/// An app `id` named `name` in `/Applications`.
pub(crate) fn app(id: &str, name: &str) -> InstalledApp {
    installed_app(id, name, "1.0").unwrap()
}

/// `app` at `path`.
pub(crate) fn at(app: InstalledApp, path: &str) -> InstalledApp {
    InstalledApp { path: path.into(), ..app }
}

/// A removable bundle of `size` bytes, with `last_used`.
pub(crate) fn facts(size: u64, last_used: Option<u64>) -> BundleFacts {
    BundleFacts { last_used: last_used.map(Timestamp::from_secs), ..bundle_facts(size) }
}

/// The fixture over `states` committed in turn, with `apps` installed, `running` running and other apps' data
/// granted.
pub(crate) fn fixture(
    states: Vec<FixedDiskState>,
    apps: Arc<dyn InstalledApps>,
    running: Arc<dyn RunningApps>,
) -> Fixture {
    let states = states.into_iter().map(|state| Arc::new(state) as Arc<dyn DiskState>).collect();
    Fixture::new(ScriptedWalk::new().committing(states))
        .system(system(apps, running))
        .access(FakeAccess::with(&[(Capability::OtherAppsData, Access::Granted)]))
}

/// Nothing running.
pub(crate) fn idle() -> Arc<dyn RunningApps> {
    Arc::new(FakeRunningApps::default())
}

/// An engine over `state` with `apps` installed and nothing running, after one scan.
pub(crate) fn scanned(state: FixedDiskState, apps: Arc<dyn InstalledApps>) -> Engine {
    let engine = fixture(vec![state], apps, idle()).engine();
    scan(&engine);
    engine
}

/// The list's rows with no filter.
pub(crate) fn rows(engine: &Engine) -> Vec<AppSummary> {
    engine.apps(AppsQuery::default()).unwrap().apps
}

/// The names of the list's rows with no filter, in order.
pub(crate) fn names(engine: &Engine) -> Vec<String> {
    rows(engine).into_iter().map(|row| row.name).collect()
}
