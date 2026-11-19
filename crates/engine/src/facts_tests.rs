//! Tests of the system's facts in a scan: one `SystemFacts` per scan, gathered only when a module asks.

use std::path::Path;
use std::sync::Arc;

use fresher_core::SystemPorts;
use fresher_testing::{FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

/// The system of `/home` with `apps` installed and `running` running.
fn system(apps: &Arc<FakeInstalledApps>, running: &Arc<FakeRunningApps>) -> SystemPorts {
    SystemPorts {
        known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
        apps: Arc::clone(apps) as _,
        running: Arc::clone(running) as _,
        installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
    }
}

mod negative {
    use std::sync::Arc;

    use fresher_core::DangerLevel;
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use super::system;
    use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

    #[test]
    fn a_scan_whose_modules_never_ask_runs_nothing_and_lists_the_apps_once_for_the_map() {
        let (apps, running) = (Arc::new(FakeInstalledApps::default()), Arc::new(FakeRunningApps::default()));
        let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
        let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
        let engine = Fixture::new(walk).module(module).system(system(&apps, &running)).engine();

        scan(&engine);
        scan(&engine);

        assert_eq!((apps.calls(), running.calls()), (2, 0), "each commit lists the apps once");
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use fresher_core::{DangerLevel, KnownFolder};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps, installed_app};

    use super::system;
    use crate::testing::{Fixture, ScriptedWalk, TestModule, scan};

    #[test]
    fn each_scan_asks_the_system_once_however_often_modules_ask() {
        let app = installed_app("com.example.Editor", "Editor", "2026.3").unwrap();
        let apps = Arc::new(FakeInstalledApps::new(vec![app]));
        let running = Arc::new(FakeRunningApps::default());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let module = |id: &'static str, category: &'static str| {
            let seen = Arc::clone(&seen);
            TestModule::builder(id).category(category, DangerLevel::Safe).on_finish(move |cx, _| {
                let system = cx.system();
                let names: Vec<String> = system.installed_apps().unwrap().iter().map(|app| app.name.clone()).collect();
                system.installed_apps().unwrap();
                system.running_apps().unwrap();
                let caches = system.locate(KnownFolder::AppCaches);
                seen.lock().unwrap().push((names, caches));
                Ok(())
            })
        };
        let engine = Fixture::new(ScriptedWalk::new())
            .module(module("alpha", "alpha.caches"))
            .module(module("beta", "beta.caches"))
            .system(system(&apps, &running))
            .engine();

        scan(&engine);
        assert_eq!((apps.calls(), running.calls()), (1, 1));
        scan(&engine);

        assert_eq!((apps.calls(), running.calls()), (2, 2));
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 4);
        assert!(seen.iter().all(|(names, caches)| *names == ["Editor"]
            && caches.as_deref() == Some(PathBuf::from("/home/Library/Caches").as_path())));
    }
}
