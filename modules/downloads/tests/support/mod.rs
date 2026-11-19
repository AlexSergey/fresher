//! The fixture homes of the downloads module's tests, the system each is scanned with, and one run of the module on
//! each.
//!
//! Each home holds only files under its `Downloads`; the installed and running apps, the receipts and the macOS
//! version are given to the module as fakes. Facts the disk cannot be made to show (a status change 10 days back, a
//! socket, the system's owner) are stated, and the run reads them through the facts lens.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{InstalledApp, Receipt, StoredSettings, Timestamp};
use fresher_module_downloads::DownloadsModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstallRecords, FakeInstalledApps, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports,
    changed_at, installed_app, run_module, running_app,
};

const DAY: u64 = 24 * 60 * 60;

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn downloads() -> DownloadsModule {
    DownloadsModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// The moment `days` days before the fixtures' now.
fn days_ago(days: u64) -> u64 {
    FIXTURE_NOW - days * DAY
}

/// An installed app of the id `id`, named `name`, at `version`, last changed `days` days ago.
pub(crate) fn app(id: &str, name: &str, version: Option<&str>, days: u64) -> InstalledApp {
    let app = InstalledApp { version: version.map(str::to_owned), ..installed_app(id, name, "1").unwrap() };
    changed_at(app, days_ago(days))
}

/// The apps installed in the kept home: Firefox 133.0 changed 10 days ago, Docker changed 20 days ago.
pub(crate) fn kept_apps() -> Vec<InstalledApp> {
    vec![app("org.mozilla.firefox", "Firefox", Some("133.0"), 10), app("com.docker.docker", "Docker", None, 20)]
}

/// A receipt of the package `package`, recorded `days` days ago.
pub(crate) fn receipt(package: &str, days: u64) -> Receipt {
    Receipt { package_file: package.into(), installed: Timestamp::from_secs(days_ago(days)) }
}

/// The system a home is scanned with: the installed and the running apps, the receipts and the macOS version, whether
/// the scan is restored from the cache, the order the matches come in and the folders the walk avoids.
pub(crate) struct System {
    pub(crate) apps: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    pub(crate) installs: FakeInstallRecords,
    pub(crate) restored: bool,
    pub(crate) order: MatchOrder,
    pub(crate) avoid: Vec<PathBuf>,
}

impl System {
    /// Nothing installed or running, macOS 15.4, a walked scan of every folder.
    pub(crate) fn plain() -> Self {
        Self {
            apps: FakeInstalledApps::default(),
            running: FakeRunningApps::default(),
            installs: FakeInstallRecords::default().with_version(15, 4),
            restored: false,
            order: MatchOrder::Walked,
            avoid: Vec::new(),
        }
    }

    /// The not-claimed home's system: Firefox 133.0 changed 10 days ago, Apple's Notes.
    pub(crate) fn not_claimed() -> Self {
        let apps = vec![
            app("org.mozilla.firefox", "Firefox", Some("133.0"), 10),
            app("com.apple.Notes", "Notes", Some("14.0"), 400),
        ];
        Self::plain().apps(FakeInstalledApps::new(apps))
    }

    /// The kept home's system: Chrome open, Firefox 133.0 changed 10 days ago, Docker changed 20 days ago, macOS 15.4.
    pub(crate) fn kept() -> Self {
        let running = FakeRunningApps::new(vec![running_app("Chrome", &["com.google.Chrome"]).unwrap()]);
        Self::plain().apps(FakeInstalledApps::new(kept_apps())).running(running)
    }

    /// The offered home's system: Firefox 133.0 and `Tool` (no version) changed 10 days ago, the receipt of
    /// `Example-2.0.pkg` 10 days ago, macOS 15.1, nothing open.
    pub(crate) fn offered() -> Self {
        let apps =
            vec![app("org.mozilla.firefox", "Firefox", Some("133.0"), 10), app("com.example.Tool", "Tool", None, 10)];
        let installs =
            FakeInstallRecords::default().with_receipts(vec![receipt("Example-2.0.pkg", 10)]).with_version(15, 1);
        Self::plain().apps(FakeInstalledApps::new(apps)).installs(installs)
    }

    pub(crate) fn apps(self, apps: FakeInstalledApps) -> Self {
        Self { apps, ..self }
    }

    pub(crate) fn running(self, running: FakeRunningApps) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn installs(self, installs: FakeInstallRecords) -> Self {
        Self { installs, ..self }
    }

    pub(crate) fn restored(self) -> Self {
        Self { restored: true, ..self }
    }

    pub(crate) fn reversed(self) -> Self {
        Self { order: MatchOrder::Reversed, ..self }
    }

    /// The walk does not open `folder`, relative to the home.
    pub(crate) fn avoiding(self, folder: &str) -> Self {
        Self { avoid: vec![PathBuf::from(folder)], ..self }
    }
}

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW` and `settings` stored.
pub(crate) fn scan_with(name: &str, system: System, settings: StoredSettings) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let ports = Ports {
        settings,
        apps: Arc::new(system.apps),
        running: Arc::new(system.running),
        installs: Arc::new(system.installs),
        restored: system.restored,
        order: system.order,
        avoid: system.avoid,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&downloads(), &tree, &ports).unwrap();
    (tree, run)
}

/// The home `name` scanned with `system`.
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    scan_with(name, system, StoredSettings::default())
}

/// Names that are not the module's to claim.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("downloads-not-claimed", System::not_claimed())
}

/// One item per reason to keep it, scanned with `system`.
pub(crate) fn kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("downloads-kept", system)
}

/// The kept home with its own system.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(System::kept())
}

/// Every category offered, scanned with `system`.
pub(crate) fn offered_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("downloads-offered", system)
}

/// The offered home with its own system.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    offered_with(System::offered())
}
