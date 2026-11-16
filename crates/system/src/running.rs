//! `MacRunningApps`: the processes that run from a bundle, as their apps.

use std::collections::HashMap;
use std::path::PathBuf;

use fresher_core::{BundleId, RunningApp, RunningApps, SystemError};
use fresher_macos::running_processes;

use crate::bundle::{Bundle, is_bundle, read_bundle};

/// The running apps of the process table: every process whose executable the system names, the user's own and,
/// where the system allows it, other users' services.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacRunningApps;

impl MacRunningApps {
    pub const fn new() -> Self {
        Self
    }
}

impl RunningApps for MacRunningApps {
    fn list(&self) -> Result<Vec<RunningApp>, SystemError> {
        let processes = running_processes().map_err(|error| SystemError { reason: error.to_string() })?;
        Ok(apps_of(processes.into_iter().filter_map(|process| process.executable)))
    }
}

/// The app of each executable that runs from a bundle: the ids of every enclosing bundle (innermost first) and the
/// name of the outermost one. An executable outside any bundle, or whose bundles have no id, gives no app. Each
/// bundle is read once; the same app from several processes is listed once.
pub(crate) fn apps_of(executables: impl IntoIterator<Item = PathBuf>) -> Vec<RunningApp> {
    let mut read: HashMap<PathBuf, Option<Bundle>> = HashMap::new();
    let mut apps: Vec<RunningApp> = Vec::new();
    for executable in executables {
        let bundles: Vec<Bundle> = executable
            .ancestors()
            .skip(1)
            .filter(|folder| is_bundle(folder))
            .filter_map(|folder| read.entry(folder.to_path_buf()).or_insert_with(|| read_bundle(folder)).clone())
            .collect();
        let Some(outermost) = bundles.last() else { continue };
        let app = RunningApp {
            name: outermost.name.clone(),
            bundle_ids: bundles.iter().map(|bundle| bundle.id.clone()).collect::<Vec<BundleId>>(),
        };
        if !apps.contains(&app) {
            apps.push(app);
        }
    }
    apps
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::running::apps_of;

        #[test]
        fn an_executable_outside_a_bundle_or_in_one_without_an_id_gives_no_app() {
            let root = tempfile::tempdir().unwrap();
            let nameless = root.path().join("Nameless.app");
            fs::create_dir_all(nameless.join("Contents/MacOS")).unwrap();

            let apps = apps_of([
                root.path().join("bin/tool"),
                nameless.join("Contents/MacOS/Nameless"),
                "/nonexistent/Gone.app/Contents/MacOS/Gone".into(),
            ]);

            assert_eq!(apps, []);
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use crate::bundle::fixture::{app, bundle};
        use crate::running::{MacRunningApps, apps_of};

        #[test]
        fn a_helper_gives_its_own_id_and_its_apps_under_the_apps_name() {
            let root = tempfile::tempdir().unwrap();
            let editor = app(root.path(), "Editor", "com.example.Editor");
            let helper = bundle(
                &editor.join("Contents/Frameworks/Editor Helper.app"),
                &[("CFBundleIdentifier", "com.example.Editor.Helper"), ("CFBundleName", "Editor Helper")],
            );

            let apps = apps_of([
                helper.join("Contents/MacOS/Editor Helper"),
                editor.join("Contents/MacOS/Editor"),
                editor.join("Contents/MacOS/Editor"),
            ]);

            let id = |id| BundleId::new(id).unwrap();
            assert_eq!(apps.len(), 2);
            assert_eq!(apps[0].name, "Editor");
            assert_eq!(apps[0].bundle_ids, [id("com.example.Editor.Helper"), id("com.example.Editor")]);
            assert_eq!(apps[1].bundle_ids, [id("com.example.Editor")]);
        }

        /// Reads this Mac's process table, read-only: the call itself must succeed.
        #[test]
        fn the_process_table_of_this_mac_can_be_listed() {
            assert!(fresher_core::RunningApps::list(&MacRunningApps::new()).is_ok());
        }
    }
}
