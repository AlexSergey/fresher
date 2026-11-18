use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{BundleId, DiskState, InstalledApp, NodeId, NodeKind, StateId, SystemFacts, SystemPorts, Timestamp};
use fresher_testing::{FakeInstalledApps, FakeKnownFolders, FakeRunningApps, FixedDiskState, installed_app};

use crate::testing::bytes;

/// A state of folders of one byte each at `paths`.
fn state(paths: &[&str]) -> FixedDiskState {
    let nodes = paths.iter().map(|&path| (PathBuf::from(path), NodeKind::Folder, bytes(1)));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)
}

/// The facts of the home `/home` with `apps` installed.
fn facts(apps: &Arc<FakeInstalledApps>) -> SystemFacts {
    SystemFacts::new(&SystemPorts {
        known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
        apps: Arc::clone(apps) as _,
        running: Arc::new(FakeRunningApps::default()),
    })
}

fn app(id: &str, name: &str) -> InstalledApp {
    installed_app(id, name, "1.0").unwrap()
}

fn id(id: &str) -> BundleId {
    BundleId::new(id).unwrap()
}

fn node(state: &FixedDiskState, path: &str) -> NodeId {
    state.find(Path::new(path)).unwrap()
}

const LIBRARY: [&str; 3] = ["/home", "/home/Library", "/home/Library/Caches"];

mod negative {
    use std::sync::Arc;

    use fresher_testing::FakeInstalledApps;

    use fresher_core::AppRef;

    use super::{LIBRARY, app, facts, id, node, state};
    use crate::storage::attribution::AppPlaces;

    #[test]
    fn an_installed_apps_own_name_wins_over_the_one_given() {
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App")]));
        let places = AppPlaces::of(&state(&["/home"]), &facts(&apps));

        let named = places.app(&id("com.example.App"), Some("Other"));

        assert_eq!(named, Some(AppRef { id: id("com.example.App"), name: "App".into(), installed: true }));
    }

    #[test]
    fn a_folder_named_like_an_app_is_not_its_without_the_dot() {
        let state = state(&[&LIBRARY[..], &["/home/Library/Caches/com.example.Appx"]].concat());
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App")]));

        let places = AppPlaces::of(&state, &facts(&apps));

        assert_eq!(places.data().count(), 0);
    }

    #[test]
    fn a_folder_outside_the_app_data_folders_is_not_attributed() {
        let state = state(&["/home", "/home/Documents", "/home/Documents/com.example.App", "/home/Library"]);
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App")]));

        let places = AppPlaces::of(&state, &facts(&apps));

        assert_eq!(places.data().count(), 0);
        assert_eq!(places.trash(), None, "no root holds the Trash");
    }

    #[test]
    fn a_listing_that_fails_names_no_app_and_keeps_the_trash() {
        let state = state(&[&LIBRARY[..], &["/home/Library/Caches/com.example.App", "/home/.Trash"]].concat());
        let apps = Arc::new(FakeInstalledApps::failing("refused"));

        let places = AppPlaces::of(&state, &facts(&apps));

        assert_eq!(places.data().count(), 0);
        assert_eq!(places.app(&id("com.example.App"), None), None);
        assert_eq!(places.trash(), Some(node(&state, "/home/.Trash")));
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::{AppRef, DiskState, InstalledApp};
    use fresher_testing::FakeInstalledApps;

    use super::{LIBRARY, app, facts, id, node, state};
    use crate::storage::attribution::AppPlaces;

    #[test]
    fn each_app_data_folder_child_named_after_an_app_is_its() {
        let library = [
            "/home/Library/Application Support",
            "/home/Library/Application Support/COM.EXAMPLE.APP",
            "/home/Library/Containers",
            "/home/Library/Containers/com.example.App.Helper",
            "/home/Library/Preferences",
            "/home/Library/Preferences/com.example.App.plist",
            "/home/Library/Saved Application State",
            "/home/Library/Saved Application State/com.example.Other.savedState",
            "/home/Library/Logs",
            "/home/Library/Group Containers",
            "/home/Library/Caches/com.example.App",
        ];
        let state = state(&[&LIBRARY[..], &library].concat());
        let main = InstalledApp { helpers: vec![id("com.example.App.Helper")], ..app("com.example.App", "App") };
        let apps = Arc::new(FakeInstalledApps::new(vec![main, app("com.example.Other", "Other")]));

        let places = AppPlaces::of(&state, &facts(&apps));

        let found: Vec<(&str, String)> = places
            .data()
            .map(|(place, app)| (app.id.as_str(), state.path(place).unwrap().display().to_string()))
            .collect();
        assert_eq!(
            found,
            [
                ("com.example.App", "/home/Library/Caches/com.example.App".to_owned()),
                ("com.example.App", "/home/Library/Application Support/COM.EXAMPLE.APP".to_owned()),
                ("com.example.App", "/home/Library/Containers/com.example.App.Helper".to_owned()),
                ("com.example.App", "/home/Library/Preferences/com.example.App.plist".to_owned()),
                ("com.example.Other", "/home/Library/Saved Application State/com.example.Other.savedState".to_owned()),
            ]
        );
    }

    #[test]
    fn a_helpers_folder_goes_to_the_app_whose_helper_it_is() {
        let state = state(&[&LIBRARY[..], &["/home/Library/Caches/com.example.App.Sync"]].concat());
        let sync = InstalledApp { helpers: vec![id("com.example.App.Sync")], ..app("com.example.Sync", "Sync") };
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App"), sync]));

        let places = AppPlaces::of(&state, &facts(&apps));

        let owners: Vec<&str> = places.data().map(|(_, app)| app.id.as_str()).collect();
        assert_eq!(owners, ["com.example.Sync"], "the longest id that names it wins");
    }

    #[test]
    fn a_bundle_a_root_holds_and_the_trash_are_places() {
        let state = state(&["/home", "/home/.Trash", "/Applications", "/Applications/App.app"]);
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App"), app("com.example.B", "B")]));

        let places = AppPlaces::of(&state, &facts(&apps));

        let found: Vec<_> = places.data().map(|(place, app)| (place, app.id.as_str())).collect();
        assert_eq!(found, [(node(&state, "/Applications/App.app"), "com.example.App")]);
        assert_eq!(places.trash(), Some(node(&state, "/home/.Trash")));
    }

    #[test]
    fn an_id_names_its_installed_app_or_itself_uninstalled() {
        let helper = InstalledApp { helpers: vec![id("com.example.App.Helper")], ..app("com.example.App", "App") };
        let apps = Arc::new(FakeInstalledApps::new(vec![helper]));

        let places = AppPlaces::of(&state(&["/home"]), &facts(&apps));

        let installed = AppRef { id: id("com.example.App"), name: "App".into(), installed: true };
        assert_eq!(places.app(&id("com.example.app.helper"), None), Some(installed));
        let gone = AppRef { id: id("com.example.Gone"), name: "com.example.Gone".into(), installed: false };
        assert_eq!(places.app(&id("com.example.Gone"), None), Some(gone));
    }

    #[test]
    fn a_removed_app_is_known_by_the_name_given() {
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App")]));
        let places = AppPlaces::of(&state(&["/home"]), &facts(&apps));

        let gone = AppRef { id: id("com.example.Gone"), name: "Gone App".into(), installed: false };
        assert_eq!(places.app(&id("com.example.Gone"), Some("Gone App")), Some(gone));
    }

    #[test]
    fn the_apps_are_listed_once_whoever_asked_first() {
        let apps = Arc::new(FakeInstalledApps::new(vec![app("com.example.App", "App")]));
        let facts = facts(&apps);
        facts.installed_apps().unwrap();

        AppPlaces::of(&state(&["/home"]), &facts);

        assert_eq!(apps.calls(), 1);
    }
}
