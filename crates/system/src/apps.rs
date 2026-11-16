//! `MacInstalledApps`: the app bundles in the apps folders, each as its own `Info.plist` and signature describe it.

use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{InstalledApp, InstalledApps, KnownFolder, KnownFolders, SystemError};

use crate::bundle::{bundle_team, helpers, is_bundle, read_bundle};

/// The known folders apps are installed in.
const APPS_FOLDERS: [KnownFolder; 3] =
    [KnownFolder::Applications, KnownFolder::UserApplications, KnownFolder::SystemApplications];

/// Lists the apps installed in the apps folders: a bundle directly in one, or one folder below (`Utilities`, a
/// vendor's folder). Never inside a bundle, never through a link, never on another volume than its apps folder.
pub struct MacInstalledApps {
    folders: Arc<dyn KnownFolders>,
}

impl MacInstalledApps {
    pub fn new(folders: Arc<dyn KnownFolders>) -> Self {
        Self { folders }
    }
}

impl fmt::Debug for MacInstalledApps {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("MacInstalledApps").finish_non_exhaustive()
    }
}

impl InstalledApps for MacInstalledApps {
    fn list(&self) -> Result<Vec<InstalledApp>, SystemError> {
        let mut bundles = Vec::new();
        let mut failed = 0_usize;
        let mut readable = 0_usize;
        for folder in APPS_FOLDERS.into_iter().filter_map(|folder| self.folders.locate(folder)) {
            match candidates(&folder) {
                Ok(found) => {
                    readable += 1;
                    bundles.extend(found);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    failed += 1;
                    tracing::warn!(%error, "an apps folder cannot be read");
                }
            }
        }
        if readable == 0 && failed > 0 {
            return Err(SystemError { reason: "no apps folder can be read".into() });
        }
        bundles.sort();
        bundles.dedup();
        let apps: Vec<InstalledApp> = bundles.iter().filter_map(|path| app(path)).collect();
        tracing::debug!(bundles = bundles.len(), apps = apps.len(), "installed apps");
        Ok(apps)
    }
}

/// The app bundle at `path`, or `None` when it cannot be read.
fn app(path: &Path) -> Option<InstalledApp> {
    let bundle = read_bundle(path)?;
    let team_id = bundle.executable.as_deref().and_then(|executable| bundle_team(path, executable));
    Some(InstalledApp {
        bundle_id: bundle.id,
        team_id,
        name: bundle.name,
        version: bundle.version,
        build: bundle.build,
        path: path.to_path_buf(),
        helpers: helpers(path),
    })
}

/// The `.app` folders in `folder` and one folder below, on `folder`'s volume; links are skipped.
fn candidates(folder: &Path) -> io::Result<Vec<PathBuf>> {
    let device = fs::metadata(folder)?.dev();
    let mut found = Vec::new();
    for path in entries(folder)? {
        let Some(metadata) = fs::symlink_metadata(&path).ok().filter(fs::Metadata::is_dir) else { continue };
        if metadata.dev() != device {
            continue;
        }
        if is_app(&path) {
            found.push(path);
        } else if !is_bundle(&path) {
            let below = entries(&path).unwrap_or_default().into_iter().filter(|inner| {
                is_app(inner)
                    && fs::symlink_metadata(inner).is_ok_and(|metadata| metadata.is_dir() && metadata.dev() == device)
            });
            found.extend(below);
        }
    }
    Ok(found)
}

fn entries(folder: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(folder)?.filter_map(Result::ok).map(|entry| entry.path()).collect())
}

fn is_app(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use fresher_core::{InstalledApp, InstalledApps, KnownFolder};
    use fresher_testing::FakeKnownFolders;

    use crate::MacInstalledApps;

    fn listed(apps_folder: &Path) -> Vec<InstalledApp> {
        let folders = FakeKnownFolders::new().with(KnownFolder::Applications, apps_folder);
        MacInstalledApps::new(Arc::new(folders)).list().unwrap()
    }

    fn ids(apps: &[InstalledApp]) -> Vec<&str> {
        apps.iter().map(|app| app.bundle_id.as_str()).collect()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;
        use std::sync::Arc;

        use fresher_core::{InstalledApps, KnownFolder};
        use fresher_testing::FakeKnownFolders;

        use super::{ids, listed};
        use crate::MacInstalledApps;
        use crate::bundle::fixture::app;

        #[test]
        fn links_bundles_inside_bundles_and_deeper_folders_are_not_apps() {
            let root = tempfile::tempdir().unwrap();
            let elsewhere = tempfile::tempdir().unwrap();
            let outside = app(elsewhere.path(), "Outside", "com.example.Outside");
            std::os::unix::fs::symlink(&outside, root.path().join("Outside.app")).unwrap();
            let host = app(root.path(), "Host", "com.example.Host");
            app(&host.join("Contents/Applications"), "Inner", "com.example.Inner");
            app(&root.path().join("Vendor/Deeper"), "Deep", "com.example.Deep");
            fs::create_dir(root.path().join("NoPlist.app")).unwrap();

            assert_eq!(ids(&listed(root.path())), ["com.example.Host"]);
        }

        #[test]
        fn missing_apps_folders_list_no_app() {
            let missing = FakeKnownFolders::new().with(KnownFolder::Applications, "/nonexistent/fresher-apps");

            assert_eq!(MacInstalledApps::new(Arc::new(missing)).list(), Ok(Vec::new()));
            assert_eq!(MacInstalledApps::new(Arc::new(FakeKnownFolders::new())).list(), Ok(Vec::new()));
        }

        #[test]
        fn an_apps_folder_that_cannot_be_read_is_an_error_when_it_is_the_only_one() {
            let root = tempfile::tempdir().unwrap();
            let closed = root.path().join("closed");
            fs::create_dir(&closed).unwrap();
            fs::set_permissions(&closed, fs::Permissions::from_mode(0o000)).unwrap();
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, &closed);

            let listed = MacInstalledApps::new(Arc::new(folders)).list();

            fs::set_permissions(&closed, fs::Permissions::from_mode(0o755)).unwrap();
            assert!(listed.is_err(), "{listed:?}");
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{InstalledApps, KnownFolder};
        use fresher_testing::FakeKnownFolders;

        use super::{ids, listed};
        use crate::bundle::fixture::{app, bundle};
        use crate::{MacInstalledApps, MacKnownFolders};

        #[test]
        fn apps_in_the_folder_and_one_below_are_listed_by_path() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "Zed", "com.example.Zed");
            app(&root.path().join("Utilities"), "Tool", "com.example.Tool");
            let editor = bundle(
                &root.path().join("Vendor/Editor.app"),
                &[
                    ("CFBundleIdentifier", "com.example.Editor"),
                    ("CFBundleName", "Editor"),
                    ("CFBundleShortVersionString", "2026.3"),
                    ("CFBundleExecutable", "editor"),
                ],
            );
            bundle(
                &editor.join("Contents/Library/LoginItems/Launcher.app"),
                &[("CFBundleIdentifier", "com.example.Launcher")],
            );

            let apps = listed(root.path());

            assert_eq!(ids(&apps), ["com.example.Tool", "com.example.Editor", "com.example.Zed"]);
            let editor = &apps[1];
            assert_eq!((editor.name.as_str(), editor.version.as_deref()), ("Editor", Some("2026.3")));
            assert_eq!(editor.team_id, None);
            assert_eq!(
                editor.helpers.iter().map(fresher_core::BundleId::as_str).collect::<Vec<_>>(),
                ["com.example.Launcher"]
            );
        }

        #[test]
        fn the_same_folder_named_twice_lists_its_apps_once() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "App", "com.example.App");
            let folders = FakeKnownFolders::new()
                .with(KnownFolder::Applications, root.path())
                .with(KnownFolder::UserApplications, root.path());

            assert_eq!(MacInstalledApps::new(Arc::new(folders)).list().unwrap().len(), 1);
        }

        /// Reads this Mac's own system apps, read-only; skipped where the folder is missing.
        #[test]
        fn the_system_apps_of_this_mac_have_apple_ids() {
            if !Path::new("/System/Applications").is_dir() {
                return;
            }
            let folders = MacKnownFolders::new("/nonexistent/fresher-home");

            let apps = MacInstalledApps::new(Arc::new(folders)).list().unwrap();

            let system: Vec<_> = apps.iter().filter(|app| app.path.starts_with("/System/Applications")).collect();
            assert_ne!(system.len(), 0);
            assert!(system.iter().all(|app| app.bundle_id.folded().starts_with("com.apple.")), "{system:?}");
        }
    }
}
