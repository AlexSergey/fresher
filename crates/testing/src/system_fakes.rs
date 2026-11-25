//! Fakes of the system's facts: the installed apps and the running apps a test makes up.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fresher_core::{
    AppIdError, AutoEmpty, BundleFacts, BundleId, ByteSize, CancelToken, EntryFacts, FinderSettings, InstallRecords,
    InstalledApp, InstalledApps, InstalledList, Receipt, RunningApp, RunningApps, Sizes, SystemError, SystemPorts,
    SystemVersion, TeamId, Timestamp,
};

use crate::fakes::lock;
use crate::{DiskFiles, FakeKnownFolders};

/// The system of a fixture home: its known folders laid out under `home`, no app installed, nothing running, its files
/// read from the disk.
pub fn fake_system(home: &Path) -> SystemPorts {
    SystemPorts {
        known_folders: Arc::new(FakeKnownFolders::home_layout(home)),
        apps: Arc::new(FakeInstalledApps::default()),
        running: Arc::new(FakeRunningApps::default()),
        installs: Arc::new(FakeInstallRecords::default()),
        files: Arc::new(DiskFiles),
    }
}

/// An installed app with only what a test names: `bundle_id` in `/Applications/<name>.app`, no team, no helpers.
pub fn installed_app(bundle_id: &str, name: &str, version: &str) -> Result<InstalledApp, AppIdError> {
    Ok(InstalledApp {
        bundle_id: BundleId::new(bundle_id)?,
        team_id: None,
        name: name.into(),
        version: Some(version.into()),
        build: None,
        path: PathBuf::from("/Applications").join(format!("{name}.app")),
        helpers: Vec::new(),
        changed: None,
    })
}

/// `app` whose bundle was last changed `seconds` after the Unix epoch.
pub fn changed_at(app: InstalledApp, seconds: u64) -> InstalledApp {
    InstalledApp { changed: Some(Timestamp::from_secs(seconds)), ..app }
}

/// `app` signed by `team`.
pub fn signed_by(app: InstalledApp, team: &str) -> Result<InstalledApp, AppIdError> {
    Ok(InstalledApp { team_id: Some(TeamId::new(team)?), ..app })
}

/// A running app named `name` running from the bundles `bundle_ids`.
pub fn running_app(name: &str, bundle_ids: &[&str]) -> Result<RunningApp, AppIdError> {
    let bundle_ids = bundle_ids.iter().map(|&id| BundleId::new(id)).collect::<Result<_, _>>()?;
    Ok(RunningApp { name: name.into(), bundle_ids })
}

/// The installed apps a test sets, or an error, one answer per listing, the last repeating; counts how often they are
/// listed. Its search finds the apps of the latest listing and the apps a test installs elsewhere, by bundle id; it is
/// complete unless a test makes it fail.
#[derive(Debug)]
pub struct FakeInstalledApps {
    /// The first listing's answer.
    answer: Result<InstalledList, SystemError>,
    /// The answers of the listings after the first, in order; the last one repeats.
    then: Vec<Result<InstalledList, SystemError>>,
    elsewhere: Vec<InstalledApp>,
    search: Result<(), SystemError>,
    /// The answer for each bundle's facts, by the bundle's path.
    bundles: HashMap<PathBuf, Result<BundleFacts, SystemError>>,
    /// The bundles with facts that are not on disk as listed (a link, another name's case or form, no `Info.plist`).
    strays: Vec<PathBuf>,
    calls: AtomicUsize,
    bundle_calls: AtomicUsize,
    /// The programs signed by a developer, each with its team.
    signed: Vec<(PathBuf, TeamId)>,
}

impl FakeInstalledApps {
    /// Answers `apps`.
    pub fn new(apps: Vec<InstalledApp>) -> Self {
        Self::answering(Ok(InstalledList::complete(apps)))
    }

    fn answering(answer: Result<InstalledList, SystemError>) -> Self {
        Self {
            answer,
            then: Vec::new(),
            elsewhere: Vec::new(),
            search: Ok(()),
            bundles: HashMap::new(),
            strays: Vec::new(),
            calls: AtomicUsize::new(0),
            bundle_calls: AtomicUsize::new(0),
            signed: Vec::new(),
        }
    }

    /// The same apps, and the program at `file` signed by `team`: every other program is signed by no one.
    #[must_use]
    pub fn signing(mut self, file: impl Into<PathBuf>, team: TeamId) -> Self {
        self.signed.push((file.into(), team));
        self
    }

    /// The same apps, and `facts` answered for the bundle at `path`; any other bundle's facts are an error.
    #[must_use]
    pub fn with_bundle(mut self, path: impl Into<PathBuf>, facts: Result<BundleFacts, SystemError>) -> Self {
        self.bundles.insert(path.into(), facts);
        self
    }

    /// The same apps, and the bundle at `path` no longer on disk as listed: `is_as_listed` answers `false` for it.
    #[must_use]
    pub fn with_stray_bundle(mut self, path: impl Into<PathBuf>) -> Self {
        self.strays.push(path.into());
        self
    }

    /// Answers `apps` once the answers before it have been given: the apps installed or removed between scans.
    #[must_use]
    pub fn then(mut self, apps: Vec<InstalledApp>) -> Self {
        self.then.push(Ok(InstalledList::complete(apps)));
        self
    }

    /// The answer of the listing `at` (counted from 0).
    fn answer(&self, at: usize) -> &Result<InstalledList, SystemError> {
        at.checked_sub(1).and_then(|later| self.then.get(later).or(self.then.last())).unwrap_or(&self.answer)
    }

    /// The same apps, and `apps` installed outside the apps folders: only the search finds them.
    #[must_use]
    pub fn with_elsewhere(self, apps: Vec<InstalledApp>) -> Self {
        Self { elsewhere: apps, ..self }
    }

    /// The same apps, and a search that cannot see every app: `search_complete` answers an error.
    #[must_use]
    pub fn failing_search(self, reason: &str) -> Self {
        Self { search: Err(SystemError { reason: reason.into() }), ..self }
    }

    /// Answers `apps` and says that the bundles or folders `unread` could not be read: a partial list.
    pub fn partial(apps: Vec<InstalledApp>, unread: &[&str]) -> Self {
        let unread = unread.iter().map(PathBuf::from).collect();
        Self::answering(Ok(InstalledList { apps, unread }))
    }

    /// Answers an error.
    pub fn failing(reason: &str) -> Self {
        Self::answering(Err(SystemError { reason: reason.into() }))
    }

    /// How many times the apps were listed.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    /// How many bundles' facts were asked.
    pub fn bundle_calls(&self) -> usize {
        self.bundle_calls.load(Ordering::SeqCst)
    }
}

impl Default for FakeInstalledApps {
    /// No app installed.
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl InstalledApps for FakeInstalledApps {
    fn list(&self) -> Result<InstalledList, SystemError> {
        let at = self.calls.fetch_add(1, Ordering::SeqCst);
        self.answer(at).clone()
    }

    fn find(&self, ids: &[BundleId], _cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        let latest = self.answer(self.calls().saturating_sub(1));
        let listed = latest.as_ref().map(|list| list.apps.as_slice()).unwrap_or_default();
        Ok(listed
            .iter()
            .chain(&self.elsewhere)
            .filter(|app| ids.contains(&app.bundle_id))
            .map(|app| app.path.clone())
            .collect())
    }

    fn search_complete(&self, _cancel: &CancelToken) -> Result<(), SystemError> {
        self.search.clone()
    }

    fn bundle(&self, app: &InstalledApp, _cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        self.bundle_calls.fetch_add(1, Ordering::SeqCst);
        self.bundles
            .get(&app.path)
            .cloned()
            .unwrap_or_else(|| Err(SystemError { reason: "no facts stated for this bundle".into() }))
    }

    /// A bundle is on disk as listed when its facts are stated and it is no stray.
    fn is_as_listed(&self, path: &Path) -> bool {
        self.bundles.get(path).is_some_and(Result::is_ok) && !self.strays.iter().any(|stray| stray == path)
    }

    fn signed_by(&self, file: &Path, team: &TeamId, _cancel: &CancelToken) -> bool {
        self.signed.iter().any(|(signed, by)| signed == file && by == team)
    }
}

/// The facts of a bundle of `logical` bytes that take as many on disk, completely known: the user may move it, no
/// last use is known and no package manager installed it.
pub fn bundle_facts(logical: u64) -> BundleFacts {
    BundleFacts {
        sizes: Sizes::complete(ByteSize::new(logical), ByteSize::new(logical)),
        entry: EntryFacts::REMOVABLE,
        last_used: None,
        installed_by: None,
    }
}

/// Finder's settings as a test states them.
#[derive(Clone, Copy, Debug)]
pub struct FakeFinderSettings(AutoEmpty);

impl FakeFinderSettings {
    /// Finder answers `answer` for its Trash setting.
    pub fn new(answer: AutoEmpty) -> Self {
        Self(answer)
    }
}

impl FinderSettings for FakeFinderSettings {
    fn trash_auto_empty(&self) -> AutoEmpty {
        self.0
    }
}

/// The running apps of a script, one answer per call; the last answer repeats. Counts how often it is asked.
#[derive(Debug)]
pub struct FakeRunningApps {
    script: Mutex<VecDeque<Result<Vec<RunningApp>, SystemError>>>,
    calls: AtomicUsize,
}

impl FakeRunningApps {
    /// Answers `apps` at every call, until `then` adds another answer.
    pub fn new(apps: Vec<RunningApp>) -> Self {
        Self { script: Mutex::new(VecDeque::from([Ok(apps)])), calls: AtomicUsize::new(0) }
    }

    /// Answers an error at every call.
    pub fn failing(reason: &str) -> Self {
        let error = SystemError { reason: reason.into() };
        Self { script: Mutex::new(VecDeque::from([Err(error)])), calls: AtomicUsize::new(0) }
    }

    /// Answers `apps` once the answers before it have been given.
    #[must_use]
    pub fn then(self, apps: Vec<RunningApp>) -> Self {
        lock(&self.script).push_back(Ok(apps));
        self
    }

    /// How many times the running apps were listed.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Default for FakeRunningApps {
    /// Nothing running.
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl RunningApps for FakeRunningApps {
    fn list(&self) -> Result<Vec<RunningApp>, SystemError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut script = lock(&self.script);
        if script.len() > 1 {
            script.pop_front().unwrap_or_else(|| Ok(Vec::new()))
        } else {
            script.front().cloned().unwrap_or_else(|| Ok(Vec::new()))
        }
    }
}

/// The installer receipts and the macOS version a test sets; counts the asks. By default no receipts and a failing
/// version ("no version stated").
#[derive(Debug)]
pub struct FakeInstallRecords {
    receipts: Result<Vec<Receipt>, SystemError>,
    version: Result<SystemVersion, SystemError>,
    calls: AtomicUsize,
}

impl FakeInstallRecords {
    /// Answers `receipts`.
    #[must_use]
    pub fn with_receipts(self, receipts: Vec<Receipt>) -> Self {
        Self { receipts: Ok(receipts), ..self }
    }

    /// Answers the macOS version `major.minor`.
    #[must_use]
    pub fn with_version(self, major: u16, minor: u16) -> Self {
        Self { version: Ok(SystemVersion { major, minor }), ..self }
    }

    /// Answers an error for the macOS version.
    #[must_use]
    pub fn failing_version(self, reason: &str) -> Self {
        Self { version: Err(SystemError { reason: reason.into() }), ..self }
    }

    /// Answers an error for the receipts.
    #[must_use]
    pub fn failing_receipts(self, reason: &str) -> Self {
        Self { receipts: Err(SystemError { reason: reason.into() }), ..self }
    }

    /// How many times the receipts or the version were asked.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Default for FakeInstallRecords {
    /// No receipts and no version stated.
    fn default() -> Self {
        Self {
            receipts: Ok(Vec::new()),
            version: Err(SystemError { reason: "no version stated".into() }),
            calls: AtomicUsize::new(0),
        }
    }
}

impl InstallRecords for FakeInstallRecords {
    fn receipts(&self) -> Result<Vec<Receipt>, SystemError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.receipts.clone()
    }

    fn system_version(&self) -> Result<SystemVersion, SystemError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.version.clone()
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{InstallRecords, InstalledApps, InstalledList, RunningApps, SystemError};

        use crate::{FakeInstallRecords, FakeInstalledApps, FakeRunningApps};

        #[test]
        fn a_failing_fake_answers_its_error_every_time() {
            let apps = FakeInstalledApps::failing("no folder");
            let running = FakeRunningApps::failing("no table");

            for _ in 0..2 {
                assert_eq!(apps.list(), Err(SystemError { reason: "no folder".into() }));
                assert_eq!(running.list(), Err(SystemError { reason: "no table".into() }));
            }
            assert_eq!((apps.calls(), running.calls()), (2, 2));
        }

        #[test]
        fn a_fake_answers_no_bundle_it_was_not_given() {
            let app = crate::installed_app("com.example.Tool", "Tool", "1.0").unwrap();
            let other = fresher_core::InstalledApp { path: "/Applications/Other.app".into(), ..app.clone() };
            let apps = FakeInstalledApps::new(vec![app.clone()]).with_bundle(&other.path, Ok(crate::bundle_facts(1)));

            assert!(apps.bundle(&app, &fresher_core::CancelToken::new()).is_err());
            assert!(FakeInstalledApps::default().bundle(&app, &fresher_core::CancelToken::new()).is_err());
        }

        #[test]
        fn the_default_install_records_have_no_receipts_and_no_version() {
            let records = FakeInstallRecords::default();

            assert_eq!(records.receipts(), Ok(Vec::new()));
            assert_eq!(records.system_version(), Err(SystemError { reason: "no version stated".into() }));
            assert_eq!(records.calls(), 2);
        }

        #[test]
        fn failing_receipts_answer_their_error() {
            let records = FakeInstallRecords::default().failing_receipts("no folder");

            assert_eq!(records.receipts(), Err(SystemError { reason: "no folder".into() }));
        }

        #[test]
        fn a_made_up_app_with_a_wrong_id_is_refused() {
            assert!(crate::installed_app("not an id", "App", "1").is_err());
            assert!(crate::running_app("App", &["com.example.App", ""]).is_err());
            let app = crate::installed_app("com.example.App", "App", "1").unwrap();
            assert!(crate::signed_by(app, "team").is_err());
        }

        #[test]
        fn the_default_fakes_know_nothing() {
            assert_eq!(FakeInstalledApps::default().list(), Ok(InstalledList::default()));
            assert_eq!(FakeRunningApps::default().list(), Ok(Vec::new()));
        }

        #[test]
        fn a_failing_search_is_not_complete() {
            let apps = FakeInstalledApps::default().failing_search("no index");

            assert_eq!(
                apps.search_complete(&fresher_core::CancelToken::new()),
                Err(SystemError { reason: "no index".into() })
            );
        }

        #[test]
        fn a_partial_fake_names_what_it_could_not_read() {
            let app = crate::installed_app("com.example.App", "App", "1").unwrap();

            let list = FakeInstalledApps::partial(vec![app.clone()], &["/Applications/Broken.app"]).list().unwrap();

            assert_eq!(list.apps, [app]);
            assert!(!list.is_complete());
        }
    }

    mod positive {
        use fresher_core::{
            AutoEmpty, ByteSize, CancelToken, Completeness, EntryFacts, FinderSettings, InstalledApps, InstalledList,
            RunningApps, SystemError,
        };

        use crate::{
            FakeFinderSettings, FakeInstallRecords, FakeInstalledApps, FakeRunningApps, bundle_facts, changed_at,
            installed_app, running_app, signed_by,
        };

        #[test]
        fn a_fake_answers_the_bundle_it_was_given() {
            let app = installed_app("com.example.Tool", "Tool", "1.0").unwrap();
            let failing = fresher_core::InstalledApp { path: "/Applications/Locked.app".into(), ..app.clone() };
            let unread = SystemError { reason: "the bundle cannot be read".into() };
            let apps = FakeInstalledApps::new(vec![app.clone(), failing.clone()])
                .with_bundle(&app.path, Ok(bundle_facts(4096)))
                .with_bundle(&failing.path, Err(unread.clone()));

            let facts = apps.bundle(&app, &CancelToken::new()).unwrap();

            assert_eq!(facts, bundle_facts(4096));
            assert_eq!((facts.sizes.logical, facts.sizes.allocated), (ByteSize::new(4096), ByteSize::new(4096)));
            assert_eq!(facts.sizes.completeness, Completeness::Complete);
            assert_eq!(facts.entry, EntryFacts::REMOVABLE);
            assert_eq!((facts.last_used, facts.installed_by), (None, None));
            assert_eq!(apps.bundle(&failing, &CancelToken::new()), Err(unread));
            assert_eq!(apps.bundle_calls(), 2);
        }
        use fresher_core::{InstallRecords, Receipt, SystemVersion, Timestamp};

        #[test]
        fn finder_settings_answer_what_the_test_states() {
            for answer in [AutoEmpty::After30Days, AutoEmpty::Never, AutoEmpty::Unknown] {
                assert_eq!(FakeFinderSettings::new(answer).trash_auto_empty(), answer);
            }
        }

        #[test]
        fn a_fake_system_has_the_home_layout_and_nothing_else() {
            let system = crate::fake_system(std::path::Path::new("/home"));

            assert_eq!(system.known_folders.locate(fresher_core::KnownFolder::Logs), Some("/home/Library/Logs".into()));
            assert_eq!(system.apps.list(), Ok(InstalledList::default()));
            assert_eq!(system.running.list(), Ok(Vec::new()));
        }

        #[test]
        fn the_install_records_are_what_the_test_says() {
            let receipt = Receipt { package_file: "Tool.pkg".into(), installed: Timestamp::from_secs(5) };
            let records = FakeInstallRecords::default().with_receipts(vec![receipt.clone()]).with_version(15, 4);
            let app = changed_at(installed_app("com.example.Tool", "Tool", "1").unwrap(), 9);

            assert_eq!(records.receipts(), Ok(vec![receipt]));
            assert_eq!(records.system_version(), Ok(SystemVersion { major: 15, minor: 4 }));
            assert_eq!(app.changed, Some(Timestamp::from_secs(9)));
        }

        #[test]
        fn the_installed_apps_are_what_the_test_says() {
            let app =
                signed_by(installed_app("com.example.Editor", "Editor", "2026.3").unwrap(), "ABCDE12345").unwrap();
            let apps = FakeInstalledApps::new(vec![app.clone()]);

            assert_eq!(apps.list(), Ok(InstalledList::complete(vec![app.clone()])));
            assert_eq!(app.path.to_str(), Some("/Applications/Editor.app"));
            assert_eq!(app.team_id.map(|team| team.to_string()), Some("ABCDE12345".into()));
            assert_eq!(apps.calls(), 1);
        }

        #[test]
        fn the_search_finds_the_listed_apps_and_those_elsewhere_by_id() {
            let listed = installed_app("com.example.Editor", "Editor", "1").unwrap();
            let mut far = installed_app("com.example.Far", "Far", "1").unwrap();
            far.path = "/Volumes/Drive/Far.app".into();
            let apps = FakeInstalledApps::new(vec![listed]).with_elsewhere(vec![far]);
            let ids = |ids: &[&str]| ids.iter().map(|id| fresher_core::BundleId::new(*id).unwrap()).collect::<Vec<_>>();

            let found = apps.find(&ids(&["com.example.Far", "com.example.Editor"]), &CancelToken::new());

            assert_eq!(found, Ok(vec!["/Applications/Editor.app".into(), "/Volumes/Drive/Far.app".into()]));
            assert_eq!(apps.find(&ids(&["com.example.None"]), &CancelToken::new()), Ok(vec![]));
            assert_eq!(apps.search_complete(&CancelToken::new()), Ok(()));
        }

        #[test]
        fn the_installed_apps_follow_their_script_and_the_search_finds_the_latest_listing() {
            let editor = installed_app("com.example.Editor", "Editor", "1").unwrap();
            let apps = FakeInstalledApps::new(Vec::new()).then(vec![editor.clone()]);
            let ids = [fresher_core::BundleId::new("com.example.Editor").unwrap()];
            let cancel = CancelToken::new();

            assert_eq!(apps.find(&ids, &cancel), Ok(vec![]));
            assert_eq!(apps.list(), Ok(InstalledList::default()));
            assert_eq!(apps.find(&ids, &cancel), Ok(vec![]));
            assert_eq!(apps.list(), Ok(InstalledList::complete(vec![editor.clone()])));
            assert_eq!(apps.find(&ids, &cancel), Ok(vec![editor.path.clone()]));
            assert_eq!(apps.list(), Ok(InstalledList::complete(vec![editor])));
            assert_eq!(apps.calls(), 3);
        }

        #[test]
        fn the_running_apps_follow_their_script_and_the_last_answer_repeats() {
            let mail = running_app("Mail", &["com.example.Mail"]).unwrap();
            let running = FakeRunningApps::new(Vec::new()).then(vec![mail.clone()]);

            assert_eq!(running.list(), Ok(Vec::new()));
            assert_eq!(running.list(), Ok(vec![mail.clone()]));
            assert_eq!(running.list(), Ok(vec![mail]));
            assert_eq!(running.calls(), 3);
        }
    }
}
