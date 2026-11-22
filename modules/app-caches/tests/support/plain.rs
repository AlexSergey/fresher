//! The fixture homes of plan 030's app-caches rows, the system each is scanned with, and one run on each.

use fresher_testing::{FakeRunningApps, FixtureTree, ModuleRun, running_app};

use crate::support::{System, app, scan};

/// The not-claimed home's system: `com.example.Editor` and its helper installed.
pub(crate) fn editor_system() -> System {
    System::with(vec![app("Editor", "com.example.Editor", &["com.example.Editor.Helper"])])
}

/// The kept home's system: its four apps installed, `com.example.Open` and `com.example.Agent`'s helper running.
pub(crate) fn kept_system() -> System {
    let apps = ["Busy", "Locked", "Open"].map(|name| app(name, &format!("com.example.{name}"), &[])).to_vec();
    let agent = app("Agent", "com.example.Agent", &["com.example.Agent.Menu"]);
    let running = FakeRunningApps::new(vec![
        running_app("Open", &["com.example.Open"]).unwrap(),
        running_app("Menu", &["com.example.Agent.Menu"]).unwrap(),
    ]);
    System::with([apps, vec![agent]].concat()).running(running)
}

/// The offered home's system: the editor with its helper, and the suite and its pro edition, installed.
pub(crate) fn offered_system() -> System {
    System::with(vec![
        app("Editor", "com.example.Editor", &["com.example.Editor.Helper"]),
        app("Suite", "com.example.Suite", &[]),
        app("Suite Pro", "com.example.Suite.Pro", &[]),
    ])
}

/// Names and ids nothing may claim, scanned with `system`.
pub(crate) fn not_claimed_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-not-claimed", system)
}

pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    not_claimed_with(editor_system())
}

/// One folder per reason to keep it, scanned with `system`.
pub(crate) fn kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-kept", system)
}

pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(kept_system())
}

/// Every cache of an installed app, scanned with `system`.
pub(crate) fn offered_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-offered", system)
}

pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    offered_with(offered_system())
}
