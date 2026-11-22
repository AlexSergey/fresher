//! What the uninstall flows share: a fixture sandbox (its marker at the top, its apps folder `Applications/`, the home
//! `home/`) scanned by the real scanner, its reader seen through the facts lens, with `app-caches` registered (or the
//! modules a flow gives); the real settings file, scan cache and journal in the sandbox's data folder; the real
//! `DiskCleaner` reaching only inside the sandbox folder, with a fake Trash at `home/.Trash` (the shared flow home of
//! `support/home*.rs`). The installed apps are the system's own listing of the sandbox's apps folders, their bundles
//! read with the stated reader (no Spotlight, no Caskroom) and seen through the stated facts; Editor's team is stated,
//! as a fixture bundle is unsigned. The cleaner's protection is this Mac's for the sandbox's home joined with the
//! sandbox's apps folder as an app folder, opened like `/Applications`, as sandbox mode joins it
//! (`app/tauri/src/sandbox.rs`); the engine refuses both apps folders for every plan but an uninstall. The running apps are a flow's to change between the plan and its confirmation.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the flows of other modules restart with other parts")]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
#[expect(dead_code, reason = "the homes of other modules give the access sandbox mode stands in for")]
mod home_engine;
#[path = "../support/home_rescan.rs"]
#[expect(dead_code, reason = "the uninstall flows scan in full; only the rescans' journal serves them")]
mod home_rescan;
#[path = "../support/home_scan.rs"]
#[expect(dead_code, reason = "the flows of other modules restore")]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fresher_core::{
    Access, AppId, AppsQuery, BundleFacts, BundleId, CancelToken, Capability, CategoryDescriptor, CategoryId, Claim,
    CleanupPlanView, CommandError, DangerLevel, FindingSink, InstalledApp, InstalledApps, InstalledList, Interest,
    KnownFolder, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleId, NewFinding, Pattern,
    Protection, Removal, RunningApp, RunningApps, SettingUpdate, SettingValue, Subject, SystemError, Text, TextKey,
    UninstallChoice,
};
use fresher_module_app_caches::AppCachesModule;
use fresher_sandbox::Sandbox;
use fresher_system::{AppSearch, BundleReader, DeclaredSearch, MacInstalledApps};
use fresher_testing::{
    FIXTURE_NOW, FakeAccess, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree, StatedFacts,
    StatedInstalledApps, running_app, signed_by,
};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home::user;
use home_engine::{Extras, Parts, engine_with, no_installs};

/// The app every flow uninstalls, its helper and its team.
pub(crate) const EDITOR_ID: &str = "com.example.Editor";
pub(crate) const HELPER_ID: &str = "com.example.Editor.Helper";
const TEAM: &str = "TEAM123456";
/// Where a copy of Editor may be, relative to the sandbox.
const EDITOR_COPIES: [&str; 2] = ["Applications/Editor.app", "home/Applications/Editor.app"];
/// The bytes of Editor's executable.
pub(crate) const EXECUTABLE: &[u8] = b"the executable of Editor";
/// The `Info.plist` of Editor's helper.
const HELPER_PLIST: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
\t<key>CFBundleIdentifier</key>
\t<string>com.example.Editor.Helper</string>
\t<key>CFBundleName</key>
\t<string>Editor Helper</string>
\t<key>CFBundlePackageType</key>
\t<string>APPL</string>
</dict>
</plist>
";

/// The world an engine runs in besides the files.
pub(crate) struct Setup {
    /// The access to other apps' data, which a container needs to go.
    pub(crate) other_apps_data: Access,
    /// More of a fixture tree, added to `fixtures/homes/uninstall-flow.toml`.
    pub(crate) extra: &'static str,
    /// The modules registered, in order.
    pub(crate) modules: Vec<Box<dyn Module>>,
}

impl Default for Setup {
    /// Other apps' data granted, the fixture as it is, `app-caches` registered.
    fn default() -> Self {
        Self { other_apps_data: Access::Granted, extra: "", modules: vec![Box::new(AppCachesModule::new().unwrap())] }
    }
}

/// The flow home with the running apps a flow changes.
pub(crate) struct Home {
    pub(crate) flow: Flow,
    pub(crate) running: Arc<Running>,
}

impl Deref for Home {
    type Target = Flow;

    fn deref(&self) -> &Flow {
        &self.flow
    }
}

impl Home {
    /// The sandbox holding `fixtures/homes/uninstall-flow.toml`, its engine in the default setup.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox with `setup.extra` added, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let text = fs::read_to_string(fixtures().join("homes").join("uninstall-flow.toml")).unwrap();
        let tree = FixtureTree::from_toml_str(&format!("{text}\n{}", setup.extra)).unwrap();
        let mut flow = Flow::open(tree, |sandbox| parts(sandbox, Vec::new(), Access::Granted));
        for copy in EDITOR_COPIES.map(|bundle| flow.path(bundle)).iter().filter(|bundle| bundle.is_dir()) {
            write_inside(copy);
        }
        let running = Arc::new(Running::default());
        // `Flow::open` builds the engine of the default extras; the home's own takes its place before anything runs.
        let extras = Extras {
            protection: protection(&flow.sandbox),
            apps: Some(installed_apps(&flow.sandbox, &flow.stated)),
            running: Some(Arc::clone(&running) as _),
            ..Extras::default()
        };
        let parts = parts(&flow.sandbox, setup.modules, setup.other_apps_data);
        flow.engine = engine_with(&flow.sandbox, &flow.stated, &flow.open_files, parts, extras);
        Self { flow, running }
    }

    /// Sets the scan roots to the home and the sandbox's apps folder.
    pub(crate) fn walk_the_apps_folder(&self) {
        let roots = vec![self.sandbox.home(), self.path("Applications")];
        let update = SettingUpdate { key: "core.scan.roots".into(), value: SettingValue::Folders(roots) };
        self.engine.update_setting(update).unwrap();
    }

    /// The id of the app whose bundle is at `bundle` (relative to the sandbox) on the Apps list; `None` when the list
    /// does not hold it.
    pub(crate) fn app_id(&self, bundle: &str) -> Option<AppId> {
        let path = self.path(bundle);
        let apps = self.engine.apps(AppsQuery::new(None)).unwrap().apps;
        apps.into_iter().map(|summary| summary.app).find(|&app| {
            let detail = self.engine.app(app).unwrap();
            detail.items.first().is_some_and(|item| item.path == path)
        })
    }

    /// The uninstall plan of the app at `bundle` with `choice`.
    pub(crate) fn uninstall(&self, bundle: &str, choice: UninstallChoice) -> Result<CleanupPlanView, CommandError> {
        let app = self.app_id(bundle).expect("the app is listed");
        self.engine.prepare_uninstall(app, choice)
    }

    /// The paths `view`'s items move, each relative to the sandbox, in the items' order.
    pub(crate) fn planned(&self, view: &CleanupPlanView) -> Vec<String> {
        view.groups
            .iter()
            .flat_map(|group| &group.items)
            .flat_map(|item| item.path.iter().chain(&item.more))
            .map(|path| self.relative_path(path))
            .collect()
    }
}

/// Writes into the bundle `copy` of Editor its executable and its helper's bundle.
fn write_inside(copy: &std::path::Path) {
    fs::write(copy.join("Contents").join("MacOS").join("Editor"), EXECUTABLE).unwrap();
    let helper = copy.join("Contents").join("Library").join("LoginItems").join("Editor Helper.app").join("Contents");
    fs::create_dir_all(helper.join("MacOS")).unwrap();
    fs::write(helper.join("Info.plist"), HELPER_PLIST).unwrap();
}

/// The running apps a flow sets as it goes; nothing at first.
#[derive(Debug, Default)]
pub(crate) struct Running(Mutex<Vec<RunningApp>>);

impl Running {
    /// The app `name` runs from the bundles of `ids` from now on.
    pub(crate) fn open(&self, name: &str, ids: &[&str]) {
        self.0.lock().unwrap().push(running_app(name, ids).unwrap());
    }
}

impl RunningApps for Running {
    fn list(&self) -> Result<Vec<RunningApp>, SystemError> {
        Ok(self.0.lock().unwrap().clone())
    }
}

/// The installed apps of `inner`, Editor's copies signed by its team: what a code signature would say.
struct Signed {
    inner: Arc<dyn InstalledApps>,
}

impl InstalledApps for Signed {
    fn list(&self) -> Result<InstalledList, SystemError> {
        let mut list = self.inner.list()?;
        for app in &mut list.apps {
            if app.bundle_id.as_str() == EDITOR_ID {
                *app = signed_by(app.clone(), TEAM).unwrap();
            }
        }
        Ok(list)
    }

    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        self.inner.find(ids, cancel)
    }

    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError> {
        self.inner.search_complete(cancel)
    }

    fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        self.inner.bundle(app, cancel)
    }

    fn is_as_listed(&self, path: &std::path::Path) -> bool {
        self.inner.is_as_listed(path)
    }
}

/// The system's listing of the sandbox's apps folders, as sandbox mode composes it: its bundles measured on its own
/// disk with the stated reader, the stated facts laid over them, Editor signed.
fn installed_apps(sandbox: &Sandbox, stated: &StatedFacts) -> Arc<dyn InstalledApps> {
    let listed =
        MacInstalledApps::new(Arc::new(sandbox.known_folders()), AppSearch::Declared(DeclaredSearch::listed()))
            .with_bundle_reader(BundleReader::stated(), user(sandbox.path()));
    Arc::new(Signed { inner: Arc::new(StatedInstalledApps::new(Arc::new(listed), stated.clone())) })
}

/// This Mac's protection of the sandbox's home (the fixture lies in the user's temporary folder, given as the user's
/// own), joined with the sandbox's apps folder as a never place that opens its bundles, the stand-in of
/// `/Applications`: the lists sandbox mode joins for these places.
pub(crate) fn protection(sandbox: &Sandbox) -> Protection {
    let mac =
        fresher_macos::protection(&sandbox.home(), &sandbox.data_folder(), None, Some(&std::env::temp_dir())).unwrap();
    let apps = sandbox.path().join("Applications");
    let folder =
        Protection::new(vec![apps.clone()], Vec::new(), Vec::new()).unwrap().with_app_folders(&[apps]).unwrap();
    mac.joined(&folder).unwrap()
}

/// The engine's parts: `modules`, nothing installed or open in the fakes (the home's own take their place), the
/// access of Full Disk Access unknown and of other apps' data `other_apps_data`, two threads, the clock at
/// `FIXTURE_NOW`.
fn parts(sandbox: &Sandbox, modules: Vec<Box<dyn Module>>, other_apps_data: Access) -> Parts {
    let access = [(Capability::AllFiles, Access::Unknown), (Capability::OtherAppsData, other_apps_data)];
    Parts {
        modules,
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, &[]),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: Arc::new(FakeAccess::with(&access)),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}

const TEXTS: &str = r#"{
    "bundles.apps.name": "App bundles",
    "bundles.apps.what": "Every app in an apps folder.",
    "bundles.apps.made-by": "The test.",
    "bundles.apps.if-removed": "The app is gone.",
    "bundles.apps.comes-back": "Never.",
    "bundles.finding.title": "An app bundle",
    "bundles.finding.reason": "The test claims every bundle."
}"#;

/// A module that claims every `*.app` folder of both apps folders (or, [`BundleModule::inside`], the `Contents` of
/// each) as a finding to move to the Trash: what a wrong rule would do. No ordinary cleanup may act on one.
pub(crate) struct BundleModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
}

impl BundleModule {
    pub(crate) fn new() -> Self {
        Self::claiming("*")
    }

    /// The module claiming the `Contents` folder inside every bundle of both apps folders.
    pub(crate) fn inside() -> Self {
        Self::claiming("*/Contents")
    }

    fn claiming(pattern: &str) -> Self {
        let id = ModuleId::new("bundles").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(id, "bundles.apps").unwrap(), DangerLevel::Low);
        let under = |folder| Interest::new(Pattern::Under(folder, pattern.into()), Claim::Whole, Vec::new()).unwrap();
        Self {
            descriptor: ModuleDescriptor::new(
                id,
                fresher_core::ModuleGroup::Development,
                vec![category],
                vec![],
                Locales { en: TEXTS },
            )
            .unwrap(),
            interests: vec![under(KnownFolder::Applications), under(KnownFolder::UserApplications)],
        }
    }
}

impl Module for BundleModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    fn on_match(&self, found: &Match, _cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if !found.path.ancestors().any(|at| at.extension().is_some_and(|extension| extension == "app")) {
            return Ok(());
        }
        let text = |key: &str| Text::new(TextKey::new(key).unwrap());
        out.push(NewFinding::new(
            &self.descriptor.categories()[0],
            found.path.to_string_lossy(),
            Subject::Path(found.path.clone()),
            found.sizes,
            text("bundles.finding.title"),
            text("bundles.finding.reason"),
            Removal::MoveToTrash(found.path.clone()),
        ));
        Ok(())
    }

    fn finish(&self, _cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        Ok(())
    }
}
