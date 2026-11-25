//! The projects of a scan, found during the one shared walk and handed to every module's `finish`: the engine, the
//! reference walk and a fixture home with projects, packages inside them and a build inside a build folder.

#[cfg(test)]
mod app {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::Receiver;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use fresher_core::{
        Event, FindingSink, Interest, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleId,
        Project, SettingUpdate, SettingValue,
    };
    use fresher_engine::{Engine, Ports, Registry};
    use fresher_store::{DataFolder, FileSettingsStore};
    use fresher_testing::{FakeAccess, FakeClock, FakeCommandRunner, FakeSystemUi, NaiveWalk, fake_system};

    /// What the module saw: whether projects were known during the walk, and the projects in `finish`.
    #[derive(Debug, Default)]
    pub(crate) struct Seen {
        pub(crate) during_walk: Vec<bool>,
        pub(crate) in_finish: Option<Vec<Project>>,
        pub(crate) matches: usize,
    }

    /// A module with one interest of its own, that records what the system's facts say of the projects.
    struct Recorder {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
        seen: Arc<Mutex<Seen>>,
    }

    impl Module for Recorder {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &self.interests
        }

        fn on_match(
            &self,
            _found: &Match,
            cx: &ModuleContext<'_>,
            _out: &mut dyn FindingSink,
        ) -> Result<(), ModuleError> {
            let mut seen = self.seen.lock().unwrap();
            seen.during_walk.push(cx.system().projects().is_some());
            seen.matches += 1;
            Ok(())
        }

        fn finish(&self, cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
            self.seen.lock().unwrap().in_finish = cx.system().projects().map(<[Project]>::to_vec);
            Ok(())
        }
    }

    /// Scans `home` with the recorder and gives what it saw.
    pub(crate) fn scan(folder: &Path, home: &Path) -> Seen {
        scan_claiming(folder, home, fresher_core::Claim::Item)
    }

    /// Scans `home` with the recorder, whose interest in every `src` folder claims it as `claim` says.
    pub(crate) fn scan_claiming(folder: &Path, home: &Path, claim: fresher_core::Claim) -> Seen {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let id = ModuleId::new("recorder").unwrap();
        let interest = Interest::new(fresher_core::Pattern::FolderName("src".into()), claim, Vec::new()).unwrap();
        let recorder = Recorder {
            descriptor: ModuleDescriptor::new(
                id,
                fresher_core::ModuleGroup::Development,
                vec![],
                vec![],
                Locales { en: "{}" },
            )
            .unwrap(),
            interests: vec![interest],
            seen: Arc::clone(&seen),
        };
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(recorder)).unwrap();
        let ports = Ports {
            walk: Arc::new(NaiveWalk::new(home)),
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(folder).unwrap())),
            clock: Arc::new(FakeClock::at(1_763_200_000)),
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::new(FakeAccess::default()),
            system_ui: Arc::new(FakeSystemUi::default()),
            system: fake_system(home),
        };
        let engine = Engine::new(registry, ports, fresher_core::HighLevel::Offered).unwrap();
        let roots = SettingValue::Folders(vec![home.to_path_buf()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
        let events = engine.subscribe();
        engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();
        until_finished(&events);
        drop(engine);
        std::mem::take(&mut *seen.lock().unwrap())
    }

    fn until_finished(events: &Receiver<Event>) {
        loop {
            if matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanFinished { .. }) {
                return;
            }
        }
    }

    /// The paths of `projects`, relative to `home`.
    pub(crate) fn roots(projects: &[Project], home: &Path) -> Vec<PathBuf> {
        projects.iter().map(|project| project.root.strip_prefix(home).unwrap().to_path_buf()).collect()
    }

    pub(crate) const HOME: &str = r#"
        [[file]]
        path = "Projects/web/package.json"
        [[file]]
        path = "Projects/web/src/index.ts"
        [[file]]
        path = "Projects/web/node_modules/left-pad/package.json"
        [[file]]
        path = "Projects/tool/Cargo.toml"
        [[file]]
        path = "Projects/tool/package.json"
        [[file]]
        path = "Projects/tool/src/main.rs"
        [[dir]]
        path = "Projects/tool/target/debug"
        [[file]]
        path = "Projects/tool/target/package/tool-1.0/Cargo.toml"
        [[file]]
        path = "Projects/notes/readme.md"
        [[dir]]
        path = "Projects/notes/package.json"
    "#;

    /// A monorepo whose every folder is 400 days old but the nested package's `node_modules`, 3 days old.
    pub(crate) const DATED_HOME: &str = r#"
        [[file]]
        path = "Code/mono/package.json"
        [[file]]
        path = "Code/mono/packages/web/package.json"
        [[file]]
        path = "Code/mono/packages/web/node_modules/left-pad.js"
        [[dir]]
        path = "Code/mono"
        age = "400d"
        [[dir]]
        path = "Code/mono/packages"
        age = "400d"
        [[dir]]
        path = "Code/mono/packages/web"
        age = "400d"
        [[dir]]
        path = "Code/mono/packages/web/node_modules"
        age = "3d"
    "#;
}

mod negative {
    use std::path::PathBuf;

    use fresher_core::{Claim, LastChange};
    use fresher_testing::FixtureTree;

    use crate::app::{HOME, roots, scan, scan_claiming};

    #[test]
    fn packages_builds_and_a_folder_named_like_a_marker_are_not_projects() {
        let (folder, tree) = (tempfile::tempdir().unwrap(), FixtureTree::from_toml_str(HOME).unwrap());
        let home = tree.root().canonicalize().unwrap();

        let seen = scan(folder.path(), &home);

        let projects = seen.in_finish.unwrap();
        assert_eq!(roots(&projects, &home), [PathBuf::from("Projects/tool"), PathBuf::from("Projects/web")]);
        assert_eq!(seen.during_walk, [false, false], "no projects while the walk runs");
        assert_eq!(seen.matches, 2, "the module gets only its own matches");
    }

    #[test]
    fn a_folder_claimed_whole_in_a_project_s_source_makes_its_activity_at_least() {
        let (folder, tree) = (tempfile::tempdir().unwrap(), FixtureTree::from_toml_str(HOME).unwrap());
        let home = tree.root().canonicalize().unwrap();

        let projects = scan_claiming(folder.path(), &home, Claim::Whole).in_finish.unwrap();

        for project in &projects {
            assert!(matches!(project.last_activity, LastChange::AtLeast(_)), "{:?}", project.last_activity);
        }
        assert_eq!(projects.len(), 2);
    }
}

mod positive {
    use fresher_core::{LastChange, ProjectKind, Timestamp};
    use fresher_testing::{FIXTURE_NOW, FixtureTree};

    use crate::app::{DATED_HOME, HOME, scan};

    const DAY: u64 = 86_400;

    #[test]
    fn every_module_gets_the_projects_with_their_kinds_and_artifacts_in_finish() {
        let (folder, tree) = (tempfile::tempdir().unwrap(), FixtureTree::from_toml_str(HOME).unwrap());
        let home = tree.root().canonicalize().unwrap();

        let projects = scan(folder.path(), &home).in_finish.unwrap();

        let tool = &projects[0];
        assert_eq!(tool.kinds, [ProjectKind::Node, ProjectKind::Rust]);
        let paths = |project: &fresher_core::Project| -> Vec<_> {
            project.artifacts.iter().map(|artifact| artifact.path.clone()).collect()
        };
        assert_eq!(paths(tool), [home.join("Projects/tool/target")]);
        assert!(matches!(tool.last_activity, LastChange::Known(_)), "{:?}", tool.last_activity);
        let web = &projects[1];
        assert_eq!(web.kinds, [ProjectKind::Node]);
        assert_eq!(paths(web), [home.join("Projects/web/node_modules")]);
    }

    #[test]
    fn a_nested_package_s_artifacts_are_not_the_outer_project_s_activity() {
        let (folder, tree) = (tempfile::tempdir().unwrap(), FixtureTree::from_toml_str(DATED_HOME).unwrap());
        let home = tree.root().canonicalize().unwrap();

        let projects = scan(folder.path(), &home).in_finish.unwrap();

        let old = LastChange::Known(Timestamp::from_secs(FIXTURE_NOW - 400 * DAY));
        let (mono, web) = (&projects[0], &projects[1]);
        assert_eq!(mono.root, home.join("Code/mono"));
        assert_eq!(mono.last_activity, old);
        assert_eq!(web.root, home.join("Code/mono/packages/web"));
        assert_eq!(web.last_activity, old);
        let modules = &web.artifacts[0];
        assert_eq!(modules.path, home.join("Code/mono/packages/web/node_modules"));
        assert_eq!(modules.last_change, LastChange::Known(Timestamp::from_secs(FIXTURE_NOW - 3 * DAY)));
    }
}
