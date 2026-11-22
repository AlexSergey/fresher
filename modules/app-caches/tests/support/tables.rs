//! The fixture homes of the app-caches module's browser and web-engine tables, the system each is scanned with, and one
//! run on each.

use fresher_core::InstalledApp;
use fresher_testing::{FakeRunningApps, FixtureTree, ModuleRun, running_app};

use crate::support::{System, app, scan};

/// The tables' apps: Google Chrome, Chromium, Firefox and Redis Insight, each with the helper its table row knows.
fn tables_apps() -> Vec<InstalledApp> {
    vec![
        app("Google Chrome", "com.google.Chrome", &["com.google.Chrome.helper"]),
        app("Chromium", "org.chromium.Chromium", &["org.chromium.Chromium.helper"]),
        app("Firefox", "org.mozilla.firefox", &[]),
        app("Redis Insight", "org.RedisLabs.RedisInsight-V2", &[]),
    ]
}

/// The tables' not-claimed home's system: Google Chrome and `OtherChat` installed.
pub(crate) fn tables_system() -> System {
    System::with(vec![
        app("Google Chrome", "com.google.Chrome", &["com.google.Chrome.helper"]),
        app("OtherChat", "com.example.OtherChat", &[]),
    ])
}

/// The tables' kept home's system: the four apps installed, Firefox and Chromium's helper running.
pub(crate) fn tables_kept_system() -> System {
    let running = FakeRunningApps::new(vec![
        running_app("Firefox", &["org.mozilla.firefox"]).unwrap(),
        running_app("Chromium Helper", &["org.chromium.Chromium.helper"]).unwrap(),
    ]);
    System::with(tables_apps()).running(running)
}

/// The tables' offered home's system: the four apps installed, nothing running.
pub(crate) fn tables_offered_system() -> System {
    System::with(tables_apps())
}

/// The tables' names and shapes nothing may claim, scanned with `system`.
pub(crate) fn tables_not_claimed_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-tables-not-claimed", system)
}

pub(crate) fn tables_not_claimed() -> (FixtureTree, ModuleRun) {
    tables_not_claimed_with(tables_system())
}

/// One cache of the tables per reason to keep it, scanned with `system`.
pub(crate) fn tables_kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-tables-kept", system)
}

pub(crate) fn tables_kept() -> (FixtureTree, ModuleRun) {
    tables_kept_with(tables_kept_system())
}

/// Every cache of the tables, scanned with `system`.
pub(crate) fn tables_offered_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-tables-offered", system)
}

pub(crate) fn tables_offered() -> (FixtureTree, ModuleRun) {
    tables_offered_with(tables_offered_system())
}
