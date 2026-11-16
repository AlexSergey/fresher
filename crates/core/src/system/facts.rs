//! One scan's view of the system: each fact gathered on the first ask and kept until the scan ends.

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::{InstalledApp, InstalledApps, KnownFolder, KnownFolders, Project, RunningApps, RunningSet, SystemError};

/// The ports of the system's facts, composed by the app (the macOS adapters) or a test (the fakes).
#[derive(Clone)]
pub struct SystemPorts {
    pub known_folders: Arc<dyn KnownFolders>,
    pub apps: Arc<dyn InstalledApps>,
    pub running: Arc<dyn RunningApps>,
}

impl fmt::Debug for SystemPorts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("SystemPorts").finish_non_exhaustive()
    }
}

/// The facts of the system for one scan. Lazy: a scan whose modules never ask reads nothing; each port is asked at
/// most once per scan, its answer (or its error) kept; a new scan makes a new one, so a rescan sees the system anew.
/// Nothing is written anywhere.
pub struct SystemFacts {
    ports: SystemPorts,
    apps: OnceLock<Result<Vec<InstalledApp>, SystemError>>,
    running: OnceLock<Result<RunningSet, SystemError>>,
    projects: OnceLock<Vec<Project>>,
}

impl SystemFacts {
    pub fn new(ports: &SystemPorts) -> Self {
        Self { ports: ports.clone(), apps: OnceLock::new(), running: OnceLock::new(), projects: OnceLock::new() }
    }

    /// Where this system keeps `folder`.
    pub fn locate(&self, folder: KnownFolder) -> Option<PathBuf> {
        self.ports.known_folders.locate(folder)
    }

    /// The installed apps, listed on the first ask.
    pub fn installed_apps(&self) -> Result<&[InstalledApp], SystemError> {
        match self.apps.get_or_init(|| self.ports.apps.list()) {
            Ok(apps) => Ok(apps),
            Err(error) => Err(error.clone()),
        }
    }

    /// The running apps, listed on the first ask: the moment of that ask.
    pub fn running_apps(&self) -> Result<&RunningSet, SystemError> {
        match self.running.get_or_init(|| self.ports.running.list().map(RunningSet::new)) {
            Ok(running) => Ok(running),
            Err(error) => Err(error.clone()),
        }
    }

    /// The projects the walk found; `None` while the walk runs (they are known before any module's `finish`).
    pub fn projects(&self) -> Option<&[Project]> {
        self.projects.get().map(Vec::as_slice)
    }

    /// Sets the projects of the walk, once; a second call changes nothing. Called by the engine after the walk.
    pub fn set_projects(&self, projects: Vec<Project>) {
        // The first set is kept: the engine sets them once per scan.
        let _ = self.projects.set(projects);
    }
}

impl fmt::Debug for SystemFacts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemFacts")
            .field("apps", &self.apps.get().map(|apps| apps.as_ref().map(Vec::len)))
            .field("projects", &self.projects.get().map(Vec::len))
            .finish_non_exhaustive()
    }
}

/// Facts of a system that knows nothing: no folder, no app, nothing running (the core's own tests).
#[cfg(test)]
pub(crate) fn nothing_known() -> SystemFacts {
    #[derive(Debug)]
    struct Nothing;

    impl KnownFolders for Nothing {
        fn locate(&self, _folder: KnownFolder) -> Option<PathBuf> {
            None
        }
    }

    impl InstalledApps for Nothing {
        fn list(&self) -> Result<Vec<InstalledApp>, SystemError> {
            Ok(Vec::new())
        }
    }

    impl RunningApps for Nothing {
        fn list(&self) -> Result<Vec<crate::RunningApp>, SystemError> {
            Ok(Vec::new())
        }
    }

    SystemFacts::new(&SystemPorts {
        known_folders: Arc::new(Nothing),
        apps: Arc::new(Nothing),
        running: Arc::new(Nothing),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::{
        BundleId, InstalledApp, InstalledApps, KnownFolder, KnownFolders, RunningApp, RunningApps, SystemError,
        SystemPorts,
    };

    /// Counts the asks; answers one app and one running app, or an error.
    #[derive(Debug, Default)]
    struct Counting {
        asked: AtomicUsize,
        fails: bool,
    }

    impl Counting {
        fn answer<T>(&self, value: T) -> Result<T, SystemError> {
            self.asked.fetch_add(1, Ordering::SeqCst);
            if self.fails { Err(SystemError { reason: "refused".into() }) } else { Ok(value) }
        }
    }

    impl KnownFolders for Counting {
        fn locate(&self, folder: KnownFolder) -> Option<PathBuf> {
            (folder == KnownFolder::Home).then(|| PathBuf::from("/home"))
        }
    }

    impl InstalledApps for Counting {
        fn list(&self) -> Result<Vec<InstalledApp>, SystemError> {
            let app = InstalledApp {
                bundle_id: BundleId::new("com.example.App").unwrap(),
                team_id: None,
                name: "App".into(),
                version: None,
                build: None,
                path: "/Applications/App.app".into(),
                helpers: Vec::new(),
            };
            self.answer(vec![app])
        }
    }

    impl RunningApps for Counting {
        fn list(&self) -> Result<Vec<RunningApp>, SystemError> {
            self.answer(vec![RunningApp {
                name: "App".into(),
                bundle_ids: vec![BundleId::new("com.example.App").unwrap()],
            }])
        }
    }

    fn ports(counting: &Arc<Counting>) -> SystemPorts {
        SystemPorts {
            known_folders: Arc::clone(counting) as Arc<dyn KnownFolders>,
            apps: Arc::clone(counting) as Arc<dyn InstalledApps>,
            running: Arc::clone(counting) as Arc<dyn RunningApps>,
        }
    }

    mod negative {
        use std::path::Path;
        use std::sync::Arc;
        use std::sync::atomic::Ordering;

        use super::{Counting, ports};
        use crate::{KnownFolder, Project, SystemError, SystemFacts};

        #[test]
        fn facts_never_asked_ask_nothing() {
            let counting = Arc::new(Counting::default());

            let facts = SystemFacts::new(&ports(&counting));

            assert_eq!(facts.projects(), None);
            assert_eq!(facts.locate(KnownFolder::Trash), None);
            assert_eq!(counting.asked.load(Ordering::SeqCst), 0);
            assert!(format!("{facts:?}").contains("SystemFacts"));
        }

        #[test]
        fn an_error_is_answered_to_every_ask_and_asked_once() {
            let counting = Arc::new(Counting { fails: true, ..Counting::default() });
            let facts = SystemFacts::new(&ports(&counting));
            let refused = SystemError { reason: "refused".into() };

            for _ in 0..3 {
                assert_eq!(facts.installed_apps(), Err(refused.clone()));
                assert_eq!(facts.running_apps().map(|set| set.owner_of(Path::new("/"))), Err(refused.clone()));
            }
            assert_eq!(counting.asked.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn the_projects_are_set_once() {
            let facts = SystemFacts::new(&ports(&Arc::new(Counting::default())));
            let project = Project { root: "/p".into(), kinds: Vec::new(), artifacts: Vec::new(), last_activity: None };

            facts.set_projects(vec![project.clone()]);
            facts.set_projects(Vec::new());

            assert_eq!(facts.projects(), Some([project].as_slice()));
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};
        use std::sync::Arc;
        use std::sync::atomic::Ordering;

        use super::{Counting, ports};
        use crate::{KnownFolder, SystemFacts};

        #[test]
        fn two_asks_in_one_scan_ask_once_and_a_new_scan_asks_again() {
            let counting = Arc::new(Counting::default());
            let ports = ports(&counting);
            let facts = SystemFacts::new(&ports);

            for _ in 0..2 {
                assert_eq!(facts.installed_apps().unwrap().len(), 1);
                assert_eq!(facts.running_apps().unwrap().owner_of(Path::new("/c/com.example.App")), Some("App"));
            }
            assert_eq!(counting.asked.load(Ordering::SeqCst), 2);
            assert_eq!(facts.locate(KnownFolder::Home), Some(PathBuf::from("/home")));

            let next = SystemFacts::new(&ports);
            next.installed_apps().unwrap();
            assert_eq!(counting.asked.load(Ordering::SeqCst), 3);
        }
    }
}
