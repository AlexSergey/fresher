//! What the Largest flows share: a fixture sandbox (its marker at the top, the home `home/`, a marker beside the trial
//! cache) scanned by the real scanner, its reader seen through the facts lens, with the `trial` module; the real
//! settings file (the smallest size listed set to 10 MB through the engine), scan cache and journal in the sandbox's
//! data folder; the real `DiskCleaner` reaching only inside the sandbox folder, as sandbox mode composes it, with this
//! Mac's protection of the sandbox's home joined (its standard folders whole), and a fake Trash at `home/.Trash` (the
//! shared flow home of `support/home*.rs`).
//! The change journal answers one rescan, that `home/Movies` changed. `restart` builds a new engine on the same files,
//! as when the app starts again.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the flows of other modules restart with other parts")]
mod home;
#[expect(dead_code, reason = "the flows of other modules write a report of the cleanup")]
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_rescan.rs"]
mod home_rescan;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[expect(dead_code, reason = "the flows of other modules sort their reports and unnumber the Trash")]
#[path = "../support/report.rs"]
mod report;

use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{
    CleanupPlanView, CommandError, Finding, LargestPage, LargestQuery, NodeId, OwnPlace, Protection, SettingUpdate,
    SettingValue, Unchanged,
};
use fresher_module_trial::TrialModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Extras, Parts, engine_with, no_installs, unknown_access};
use home_rescan::journal;

impl Flow {
    /// The sandbox holding `fixtures/homes/largest.toml`, its engine on it, the smallest size listed 10 MB.
    pub(crate) fn new() -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("largest.toml")).unwrap();
        let mut flow = Self::open(tree, parts);
        // `Flow::open` builds the engine of the default extras; the flow's own takes its place before anything runs.
        flow.restart();
        flow.engine
            .update_setting(SettingUpdate {
                key: "core.largest.min-size".into(),
                value: SettingValue::Choice("10-mb".into()),
            })
            .unwrap();
        flow
    }

    /// A new engine on the same files: the app started again.
    pub(crate) fn restart(&mut self) {
        let extras = Extras { protection: mac_protection(&self.sandbox), ..Extras::default() };
        self.engine = engine_with(&self.sandbox, &self.stated, &self.open_files, parts(&self.sandbox), extras);
    }

    /// The first page of the Files list, no filter.
    pub(crate) fn largest(&self) -> LargestPage {
        let query = LargestQuery { kind: None, place: None, unchanged: Unchanged::Any, offset: 0, limit: 100 };
        self.engine.largest(&query).unwrap()
    }

    /// The Files list's rows as paths relative to the sandbox, in their order.
    pub(crate) fn listed(&self) -> Vec<String> {
        self.largest().rows.iter().map(|row| self.relative_path(&self.node_path(row.node.id))).collect()
    }

    /// The first page of the Folders view of `folder` (relative to the sandbox; `None`: the home).
    pub(crate) fn place(&self, folder: Option<&str>) -> Result<OwnPlace, CommandError> {
        self.engine.largest_place(folder.map(|folder| self.node(folder)), 0, 100)
    }

    /// The node at `relative` in the last state.
    pub(crate) fn node(&self, relative: &str) -> NodeId {
        let state = self.engine.disk_state().unwrap().unwrap();
        state.find(&self.path(relative)).unwrap()
    }

    /// The path of `node` in the last state.
    pub(crate) fn node_path(&self, node: NodeId) -> PathBuf {
        self.engine.disk_state().unwrap().unwrap().path(node).unwrap()
    }

    /// Picks the row at `relative`, as a tick on the Largest screen does.
    pub(crate) fn pick(&self, relative: &str) -> Result<Finding, CommandError> {
        self.engine.pick_own_item(self.node(relative))
    }

    /// The plan of the picks `picked`.
    pub(crate) fn prepare(&self, picked: &[&Finding]) -> CleanupPlanView {
        let ids: Vec<Value> = picked.iter().map(|finding| serde_json::to_value(finding.id()).unwrap()).collect();
        let selection = serde_json::from_value(json!({ "findings": ids, "categories": [] })).unwrap();
        self.engine.prepare_cleanup(&selection).unwrap()
    }

    /// `report` with the sandbox's path written `~` and the nodes' ids and times removed (a walk's order and a
    /// folder's time are the machine's).
    pub(crate) fn report_of(&self, report: &impl serde::Serialize) -> Value {
        let text = serde_json::to_string(report).unwrap().replace(&self.fixture.display().to_string(), "~");
        let mut report: Value = serde_json::from_str(&text).unwrap();
        for key in ["id", "modified"] {
            report::strip(&mut report, key);
        }
        report
    }
}

/// This Mac's protection of the sandbox's home, as sandbox mode joins it: the home and its standard folders whole. The
/// fixture lies in the user's temporary folder (`T`, below `/private/var/folders`, which the protection refuses whole),
/// so that folder is given as the user's own and carved out, as the app gives its own.
fn mac_protection(sandbox: &Sandbox) -> Protection {
    fresher_macos::protection(&sandbox.home(), &sandbox.data_folder(), None, Some(&std::env::temp_dir())).unwrap()
}

/// The engine's parts: the `trial` module, nothing installed or open, a journal that answers one rescan of
/// `home/Movies`, two threads, the clock at `FIXTURE_NOW`.
fn parts(sandbox: &Sandbox) -> Parts {
    Parts {
        modules: vec![Box::new(TrialModule::new().unwrap())],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: journal(sandbox, &[vec!["home/Movies"]]),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
