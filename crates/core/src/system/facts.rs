//! One scan's view of the system: each fact gathered on the first ask and kept until the scan ends.

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::{
    BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, KnownFolder, KnownFolders, Project, RunningApps,
    RunningSet, SystemError,
};

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
    apps: OnceLock<Result<InstalledList, SystemError>>,
    running: OnceLock<Result<RunningSet, SystemError>>,
    search: OnceLock<Result<(), SystemError>>,
    projects: OnceLock<Vec<Project>>,
}

impl SystemFacts {
    pub fn new(ports: &SystemPorts) -> Self {
        Self {
            ports: ports.clone(),
            apps: OnceLock::new(),
            running: OnceLock::new(),
            search: OnceLock::new(),
            projects: OnceLock::new(),
        }
    }

    /// Where this system keeps `folder`.
    pub fn locate(&self, folder: KnownFolder) -> Option<PathBuf> {
        self.ports.known_folders.locate(folder)
    }

    /// The installed apps, listed on the first ask; what the listing could not read is left out.
    pub fn installed_apps(&self) -> Result<&[InstalledApp], SystemError> {
        self.installed().map(|list| list.apps.as_slice())
    }

    /// The installed apps only when the listing read every apps folder and bundle: a partial list is an error, for a
    /// module that must not take an app it could not read for one that is not installed.
    pub fn installed_apps_complete(&self) -> Result<&[InstalledApp], SystemError> {
        let list = self.installed()?;
        if list.is_complete() {
            return Ok(&list.apps);
        }
        let reason = match list.unread.len() {
            1 => "an app or a folder could not be read".to_owned(),
            count => format!("{count} apps or folders could not be read"),
        };
        Err(SystemError { reason })
    }

    /// The bundles with one of `ids` anywhere the system's index knows them, asked anew at every call (a module asks
    /// it only for what it is about to offer), after checking once per scan that the index sees every mounted volume:
    /// when it cannot, its error is answered and nothing is asked, since an empty answer would prove nothing.
    pub fn find_apps(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        if let Err(error) = self.search.get_or_init(|| self.ports.apps.search_complete(cancel)) {
            return Err(error.clone());
        }
        self.ports.apps.find(ids, cancel)
    }

    fn installed(&self) -> Result<&InstalledList, SystemError> {
        match self.apps.get_or_init(|| self.ports.apps.list()) {
            Ok(list) => Ok(list),
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
            .field("apps", &self.apps.get().map(|list| list.as_ref().map(|list| list.apps.len())))
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
        fn list(&self) -> Result<InstalledList, SystemError> {
            Ok(InstalledList::default())
        }

        fn find(&self, _ids: &[BundleId], _cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
            Ok(Vec::new())
        }

        fn search_complete(&self, _cancel: &CancelToken) -> Result<(), SystemError> {
            Ok(())
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
        BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, KnownFolder, KnownFolders, RunningApp,
        RunningApps, SystemError, SystemPorts,
    };

    /// Counts the asks; answers one app and one running app, or an error; with `partial`, a bundle left unread; with
    /// `blind`, a search that cannot see every volume. The search's checks are counted on their own.
    #[derive(Debug, Default)]
    struct Counting {
        asked: AtomicUsize,
        searched: AtomicUsize,
        fails: bool,
        partial: bool,
        blind: bool,
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
        fn list(&self) -> Result<InstalledList, SystemError> {
            let app = InstalledApp {
                bundle_id: BundleId::new("com.example.App").unwrap(),
                team_id: None,
                name: "App".into(),
                version: None,
                build: None,
                path: "/Applications/App.app".into(),
                helpers: Vec::new(),
            };
            let unread = if self.partial { vec![PathBuf::from("/Applications/Broken.app")] } else { Vec::new() };
            self.answer(InstalledList { apps: vec![app], unread })
        }

        fn find(&self, ids: &[BundleId], _cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
            let found = ids.iter().map(|id| PathBuf::from(format!("/Volumes/Drive/{id}.app"))).collect();
            self.answer(found)
        }

        fn search_complete(&self, _cancel: &CancelToken) -> Result<(), SystemError> {
            self.searched.fetch_add(1, Ordering::SeqCst);
            if self.blind { Err(SystemError { reason: "a volume is not indexed".into() }) } else { Ok(()) }
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
        use crate::{BundleId, CancelToken, KnownFolder, Project, SystemError, SystemFacts};

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
                assert_eq!(facts.installed_apps_complete(), Err(refused.clone()));
                assert_eq!(facts.running_apps().map(|set| set.owner_of(Path::new("/"))), Err(refused.clone()));
            }
            assert_eq!(counting.asked.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn a_partial_list_is_an_error_only_when_asked_complete() {
            let counting = Arc::new(Counting { partial: true, ..Counting::default() });
            let facts = SystemFacts::new(&ports(&counting));

            assert_eq!(facts.installed_apps().unwrap().len(), 1);
            let error = facts.installed_apps_complete().unwrap_err();
            assert_eq!(error.reason, "an app or a folder could not be read");
            assert_eq!(counting.asked.load(Ordering::SeqCst), 1);
        }

        #[test]
        fn an_incomplete_search_is_answered_without_finding_and_checked_once() {
            let counting = Arc::new(Counting { blind: true, ..Counting::default() });
            let facts = SystemFacts::new(&ports(&counting));
            let ids = [BundleId::new("com.example.App").unwrap()];

            for _ in 0..2 {
                assert_eq!(
                    facts.find_apps(&ids, &CancelToken::new()),
                    Err(SystemError { reason: "a volume is not indexed".into() })
                );
            }
            assert_eq!(counting.searched.load(Ordering::SeqCst), 1);
            assert_eq!(counting.asked.load(Ordering::SeqCst), 0);
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
        use crate::{BundleId, CancelToken, KnownFolder, SystemFacts};

        #[test]
        fn finding_apps_checks_the_search_once_and_asks_the_port_at_every_call() {
            let counting = Arc::new(Counting::default());
            let facts = SystemFacts::new(&ports(&counting));
            let ids = [BundleId::new("com.example.App").unwrap()];

            for _ in 0..2 {
                assert_eq!(
                    facts.find_apps(&ids, &CancelToken::new()),
                    Ok(vec![PathBuf::from("/Volumes/Drive/com.example.App.app")])
                );
            }
            assert_eq!(counting.asked.load(Ordering::SeqCst), 2);
            assert_eq!(counting.searched.load(Ordering::SeqCst), 1);
        }

        #[test]
        fn two_asks_in_one_scan_ask_once_and_a_new_scan_asks_again() {
            let counting = Arc::new(Counting::default());
            let ports = ports(&counting);
            let facts = SystemFacts::new(&ports);

            for _ in 0..2 {
                assert_eq!(facts.installed_apps().unwrap().len(), 1);
                assert_eq!(facts.installed_apps_complete().unwrap().len(), 1);
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
