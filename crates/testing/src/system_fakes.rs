//! Fakes of the system's facts: the installed apps and the running apps a test makes up.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fresher_core::{
    AppIdError, BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, RunningApp, RunningApps,
    SystemError, SystemPorts, TeamId,
};

use crate::FakeKnownFolders;
use crate::fakes::lock;

/// The system of a fixture home: its known folders laid out under `home`, no app installed, nothing running.
pub fn fake_system(home: &Path) -> SystemPorts {
    SystemPorts {
        known_folders: Arc::new(FakeKnownFolders::home_layout(home)),
        apps: Arc::new(FakeInstalledApps::default()),
        running: Arc::new(FakeRunningApps::default()),
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
    })
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

/// The installed apps a test sets, or an error; counts how often they are listed. Its search finds the listed apps and
/// the apps a test installs elsewhere, by bundle id, or answers the error a test sets.
#[derive(Debug)]
pub struct FakeInstalledApps {
    answer: Result<InstalledList, SystemError>,
    elsewhere: Result<Vec<InstalledApp>, SystemError>,
    calls: AtomicUsize,
}

impl FakeInstalledApps {
    /// Answers `apps`.
    pub fn new(apps: Vec<InstalledApp>) -> Self {
        Self::answering(Ok(InstalledList::complete(apps)))
    }

    fn answering(answer: Result<InstalledList, SystemError>) -> Self {
        Self { answer, elsewhere: Ok(Vec::new()), calls: AtomicUsize::new(0) }
    }

    /// The same apps, and `apps` installed outside the apps folders: only the search finds them.
    #[must_use]
    pub fn with_elsewhere(self, apps: Vec<InstalledApp>) -> Self {
        Self { elsewhere: Ok(apps), ..self }
    }

    /// The same apps, and a search that answers an error.
    #[must_use]
    pub fn failing_search(self, reason: &str) -> Self {
        Self { elsewhere: Err(SystemError { reason: reason.into() }), ..self }
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
}

impl Default for FakeInstalledApps {
    /// No app installed.
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl InstalledApps for FakeInstalledApps {
    fn list(&self) -> Result<InstalledList, SystemError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.answer.clone()
    }

    fn find(&self, ids: &[BundleId], _cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        let elsewhere = self.elsewhere.as_ref().map_err(Clone::clone)?;
        let listed = self.answer.as_ref().map(|list| list.apps.as_slice()).unwrap_or_default();
        Ok(listed
            .iter()
            .chain(elsewhere)
            .filter(|app| ids.contains(&app.bundle_id))
            .map(|app| app.path.clone())
            .collect())
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

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{InstalledApps, InstalledList, RunningApps, SystemError};

        use crate::{FakeInstalledApps, FakeRunningApps};

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
        fn a_failing_search_answers_its_error() {
            let apps = FakeInstalledApps::default().failing_search("no index");

            assert_eq!(
                apps.find(&[fresher_core::BundleId::new("a.b").unwrap()], &fresher_core::CancelToken::new()),
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
        use fresher_core::{InstalledApps, InstalledList, RunningApps};

        use crate::{FakeInstalledApps, FakeRunningApps, installed_app, running_app, signed_by};

        #[test]
        fn a_fake_system_has_the_home_layout_and_nothing_else() {
            let system = crate::fake_system(std::path::Path::new("/home"));

            assert_eq!(system.known_folders.locate(fresher_core::KnownFolder::Logs), Some("/home/Library/Logs".into()));
            assert_eq!(system.apps.list(), Ok(InstalledList::default()));
            assert_eq!(system.running.list(), Ok(Vec::new()));
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

            let found = apps.find(&ids(&["com.example.Far", "com.example.Editor"]), &fresher_core::CancelToken::new());

            assert_eq!(found, Ok(vec!["/Applications/Editor.app".into(), "/Volumes/Drive/Far.app".into()]));
            assert_eq!(apps.find(&ids(&["com.example.None"]), &fresher_core::CancelToken::new()), Ok(vec![]));
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
