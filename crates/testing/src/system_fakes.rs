//! Fakes of the system's facts: the installed apps and the running apps a test makes up.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fresher_core::{
    AppIdError, BundleId, InstalledApp, InstalledApps, RunningApp, RunningApps, SystemError, SystemPorts, TeamId,
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

/// The installed apps a test sets, or an error; counts how often it is asked.
#[derive(Debug)]
pub struct FakeInstalledApps {
    answer: Result<Vec<InstalledApp>, SystemError>,
    calls: AtomicUsize,
}

impl FakeInstalledApps {
    /// Answers `apps`.
    pub fn new(apps: Vec<InstalledApp>) -> Self {
        Self { answer: Ok(apps), calls: AtomicUsize::new(0) }
    }

    /// Answers an error.
    pub fn failing(reason: &str) -> Self {
        Self { answer: Err(SystemError { reason: reason.into() }), calls: AtomicUsize::new(0) }
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
    fn list(&self) -> Result<Vec<InstalledApp>, SystemError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.answer.clone()
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
        use fresher_core::{InstalledApps, RunningApps, SystemError};

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
            assert_eq!(FakeInstalledApps::default().list(), Ok(Vec::new()));
            assert_eq!(FakeRunningApps::default().list(), Ok(Vec::new()));
        }
    }

    mod positive {
        use fresher_core::{InstalledApps, RunningApps};

        use crate::{FakeInstalledApps, FakeRunningApps, installed_app, running_app, signed_by};

        #[test]
        fn a_fake_system_has_the_home_layout_and_nothing_else() {
            let system = crate::fake_system(std::path::Path::new("/home"));

            assert_eq!(system.known_folders.locate(fresher_core::KnownFolder::Logs), Some("/home/Library/Logs".into()));
            assert_eq!(system.apps.list(), Ok(Vec::new()));
            assert_eq!(system.running.list(), Ok(Vec::new()));
        }

        #[test]
        fn the_installed_apps_are_what_the_test_says() {
            let app =
                signed_by(installed_app("com.example.Editor", "Editor", "2026.3").unwrap(), "ABCDE12345").unwrap();
            let apps = FakeInstalledApps::new(vec![app.clone()]);

            assert_eq!(apps.list(), Ok(vec![app.clone()]));
            assert_eq!(app.path.to_str(), Some("/Applications/Editor.app"));
            assert_eq!(app.team_id.map(|team| team.to_string()), Some("ABCDE12345".into()));
            assert_eq!(apps.calls(), 1);
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
