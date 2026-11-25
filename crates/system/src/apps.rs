//! `MacInstalledApps`: the app bundles in the apps folders, each as its own `Info.plist` and signature describe it.

use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    BundleFacts, BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, KnownFolder, KnownFolders,
    SystemError, TeamId, Timestamp,
};

use crate::bundle::{bundle_team, helpers, is_as_listed, is_bundle, read_bundle};
use crate::spotlight::SpotlightSearch;
use crate::{BundleReader, COMMAND_TIMEOUT, ProcessCommandRunner};

/// The known folders apps are installed in.
const APPS_FOLDERS: [KnownFolder; 3] =
    [KnownFolder::Applications, KnownFolder::UserApplications, KnownFolder::SystemApplications];

/// How deep an app is looked for in an apps folder: directly in it, or one folder below (`Utilities`, a vendor's
/// folder such as `JetBrains Toolbox`).
const APPS_FOLDER_DEPTH: usize = 2;

/// Where an older `JetBrains Toolbox` app kept the IDEs it installed, under the app support folder: each a few folders
/// down (`apps/<IDE>/ch-0/<build>/<IDE>.app`).
const TOOLBOX_APPS: &str = "JetBrains/Toolbox/apps";

/// How deep an app is looked for in the older Toolbox's folder.
const TOOLBOX_DEPTH: usize = 4;

/// Lists the apps installed in the apps folders (a bundle directly in one, or one folder below) and in the older
/// `JetBrains Toolbox` app's own folder (a few folders below). Never inside a bundle, never walking through a link,
/// never on another volume than the folder looked in; an app that is a link is read where it points. What it cannot
/// read — a folder, a link to a folder, another volume's folder, a bundle with no readable `Info.plist`, an app link
/// that leads to no bundle — it names in the list's `unread`.
pub struct MacInstalledApps {
    folders: Arc<dyn KnownFolders>,
    search: AppSearch,
    /// What reads a bundle's facts, and the user they are read for; without one a bundle is not known.
    bundles: Option<(BundleReader, u32)>,
}

/// Where `find` looks for an app beyond the apps folders.
#[derive(Clone)]
pub enum AppSearch {
    /// Spotlight's index: the whole Mac.
    Spotlight(SpotlightSearch),
    /// What a sandbox declares: its listed apps, the apps its case files keep elsewhere, and whether its Spotlight is
    /// on; never the Mac's.
    Declared(DeclaredSearch),
}

impl fmt::Debug for AppSearch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Spotlight(_) => "Spotlight",
            Self::Declared(_) => "Declared",
        })
    }
}

/// A sandbox's search: the listed apps, the `elsewhere` apps by bundle id and path, and whether the index is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredSearch {
    pub index_on: bool,
    pub elsewhere: Vec<(BundleId, PathBuf)>,
}

impl DeclaredSearch {
    /// The index on and nothing elsewhere: only the listed apps are found.
    pub const fn listed() -> Self {
        Self { index_on: true, elsewhere: Vec::new() }
    }
}

impl MacInstalledApps {
    pub fn new(folders: Arc<dyn KnownFolders>, search: AppSearch) -> Self {
        Self { folders, search, bundles: None }
    }

    /// The same apps, whose bundles `reader` reads for `user` (the user the app runs as).
    #[must_use]
    pub fn with_bundle_reader(self, reader: BundleReader, user: u32) -> Self {
        Self { bundles: Some((reader, user)), ..self }
    }
}

impl fmt::Debug for MacInstalledApps {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("MacInstalledApps").finish_non_exhaustive()
    }
}

impl InstalledApps for MacInstalledApps {
    /// Each listing starts a new run of the bundle reader: what it read of the Caskrooms is read again.
    fn list(&self) -> Result<InstalledList, SystemError> {
        if let Some((reader, _)) = &self.bundles {
            reader.start_run();
        }
        let mut found = Found::default();
        let mut failed = 0_usize;
        let mut readable = 0_usize;
        for folder in APPS_FOLDERS.into_iter().filter_map(|folder| self.folders.locate(folder)) {
            match found.look_in(&folder, APPS_FOLDER_DEPTH) {
                Ok(()) => readable += 1,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    failed += 1;
                    tracing::warn!(%error, "an apps folder cannot be read");
                    found.unread.push(folder);
                }
            }
        }
        if readable == 0 && failed > 0 {
            return Err(SystemError { reason: "no apps folder can be read".into() });
        }
        if let Some(toolbox) = self.folders.locate(KnownFolder::AppSupport).map(|folder| folder.join(TOOLBOX_APPS)) {
            found.look_in_toolbox(toolbox);
        }
        Ok(found.into_list())
    }

    /// Asks the system's `codesign` whether `file` is signed by `team`, by its absolute path and with its own runner
    /// (the listing runs no program of its own otherwise).
    fn signed_by(&self, file: &Path, team: &TeamId, cancel: &CancelToken) -> bool {
        crate::code_check::signed_by(&ProcessCommandRunner::new(COMMAND_TIMEOUT), file, team, cancel)
    }

    /// Asks Spotlight once per id (the id is checked reverse-DNS, so it cannot break out of the quotes); a path it
    /// answers that does not exist (an index not yet updated) is not an app. Any failure of the command is an error.
    /// An empty answer proves something only after `search_complete`. A declared search answers the listed apps and
    /// the apps it keeps elsewhere with one of `ids`.
    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        let mut found = Vec::new();
        match &self.search {
            AppSearch::Spotlight(search) => {
                for id in ids {
                    let paths = search.paths(id.as_str(), cancel)?;
                    found.extend(paths.into_iter().filter(|path| fs::symlink_metadata(path).is_ok()));
                }
            }
            AppSearch::Declared(declared) => {
                let listed = self.list()?.apps.into_iter().map(|app| (app.bundle_id, app.path));
                found.extend(
                    listed
                        .chain(declared.elsewhere.iter().cloned())
                        .filter(|(id, _)| ids.contains(id))
                        .map(|(_, path)| path),
                );
            }
        }
        found.sort();
        found.dedup();
        Ok(found)
    }

    /// Spotlight's own check, for the volume that holds the home among the others; a declared search is complete when
    /// its index is on.
    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError> {
        match &self.search {
            AppSearch::Spotlight(search) => {
                let home = self
                    .folders
                    .locate(KnownFolder::Home)
                    .ok_or_else(|| SystemError { reason: "the home folder is unknown".into() })?;
                search.search_complete(&home, cancel)
            }
            AppSearch::Declared(DeclaredSearch { index_on: true, .. }) => Ok(()),
            AppSearch::Declared(DeclaredSearch { index_on: false, .. }) => {
                Err(SystemError { reason: "the sandbox's Spotlight is off".into() })
            }
        }
    }

    /// The bundle reader's answer; without one, the port's default error.
    fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        match &self.bundles {
            Some((reader, user)) => reader.read(app, *user, cancel),
            None => Err(SystemError { reason: "this system does not read bundles".into() }),
        }
    }

    fn is_as_listed(&self, path: &Path) -> bool {
        is_as_listed(path)
    }
}

/// The bundles found so far and what could not be read.
#[derive(Debug, Default)]
struct Found {
    bundles: Vec<PathBuf>,
    unread: Vec<PathBuf>,
}

impl Found {
    /// Looks for apps in `folder` and up to `depth - 1` folders below it, on `folder`'s volume; an error only when
    /// `folder` itself cannot be read.
    fn look_in(&mut self, folder: &Path, depth: usize) -> io::Result<()> {
        let device = fs::metadata(folder)?.dev();
        let entries = entries(folder)?;
        self.walk(entries, device, depth);
        Ok(())
    }

    /// Looks for apps in the older Toolbox's folder, which is optional: missing, it holds nothing; a link, or not
    /// readable, it is unread.
    fn look_in_toolbox(&mut self, toolbox: PathBuf) {
        if fs::symlink_metadata(&toolbox).is_ok_and(|metadata| metadata.is_symlink()) {
            self.unread.push(toolbox);
            return;
        }
        match self.look_in(&toolbox, TOOLBOX_DEPTH) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                tracing::warn!(%error, "the Toolbox apps folder cannot be read");
                self.unread.push(toolbox);
            }
        }
    }

    fn walk(&mut self, paths: Vec<PathBuf>, device: u64, depth: usize) {
        for path in paths {
            let Ok(metadata) = fs::symlink_metadata(&path) else { continue };
            let here = metadata.is_dir() && metadata.dev() == device;
            if is_app(&path) {
                if metadata.is_symlink() {
                    self.linked(path);
                } else if here {
                    self.bundles.push(path);
                } else if metadata.is_dir() {
                    // Another volume's app: what it is cannot be read without leaving the folder's volume.
                    self.unread.push(path);
                }
            } else if depth > 1 && !is_bundle(&path) {
                if metadata.is_symlink() && fs::metadata(&path).map_or(true, |target| !target.is_file()) {
                    // A link to a folder (or to nothing readable) may hold apps: it is never walked through.
                    self.unread.push(path);
                } else if metadata.is_dir() && !here {
                    // A folder on another volume is never walked into.
                    self.unread.push(path);
                } else if here {
                    match entries(&path) {
                        Ok(inner) => self.walk(inner, device, depth - 1),
                        Err(error) => {
                            tracing::warn!(%error, "a folder among the apps cannot be read");
                            self.unread.push(path);
                        }
                    }
                }
            }
        }
    }

    /// An app that is a link (`/Applications/Safari.app` is one on every Mac): the folder it leads to is read as the
    /// bundle, nothing is walked through it. A dangling link, or one to anything but a folder, is unread under the
    /// link's own path; a folder it leads to that is no readable bundle is unread under that folder's path.
    fn linked(&mut self, link: PathBuf) {
        match fs::canonicalize(&link) {
            Ok(target) if fs::symlink_metadata(&target).is_ok_and(|metadata| metadata.is_dir()) => {
                self.bundles.push(target);
            }
            _ => self.unread.push(link),
        }
    }

    /// Every bundle read once, by path; a bundle that cannot be read is unread.
    fn into_list(mut self) -> InstalledList {
        self.bundles.sort();
        self.bundles.dedup();
        let mut apps = Vec::with_capacity(self.bundles.len());
        for path in self.bundles {
            match app(&path) {
                Some(app) => apps.push(app),
                None => self.unread.push(path),
            }
        }
        self.unread.sort();
        self.unread.dedup();
        if !self.unread.is_empty() {
            tracing::warn!(unread = self.unread.len(), "some apps or folders cannot be read: the list is partial");
        }
        tracing::debug!(apps = apps.len(), "installed apps");
        InstalledList { apps, unread: self.unread }
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
        changed: changed(path),
    })
}

/// The bundle's status change time in whole seconds; `None` when its metadata cannot be read.
fn changed(path: &Path) -> Option<Timestamp> {
    let metadata = fs::symlink_metadata(path).ok()?;
    Some(Timestamp::from_secs(u64::try_from(metadata.ctime()).ok()?))
}

fn entries(folder: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(folder)?.filter_map(Result::ok).map(|entry| entry.path()).collect())
}

fn is_app(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::{InstalledApp, InstalledApps, InstalledList, KnownFolder};
    use fresher_testing::FakeKnownFolders;

    use crate::MacInstalledApps;

    fn list_of(apps_folder: &Path) -> InstalledList {
        let folders = FakeKnownFolders::new().with(KnownFolder::Applications, apps_folder);
        MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
            .list()
            .unwrap()
    }

    fn listed(apps_folder: &Path) -> Vec<InstalledApp> {
        let list = list_of(apps_folder);
        assert!(list.is_complete(), "{:?}", list.unread);
        list.apps
    }

    /// The list of a home whose app support folder is `home/Library/Application Support`, with no apps folder but the
    /// user's `home/Applications`.
    fn list_of_home(home: &Path) -> InstalledList {
        let folders = FakeKnownFolders::new()
            .with(KnownFolder::UserApplications, home.join("Applications"))
            .with(KnownFolder::AppSupport, home.join("Library/Application Support"));
        MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
            .list()
            .unwrap()
    }

    fn toolbox(home: &Path) -> PathBuf {
        home.join("Library/Application Support/JetBrains/Toolbox/apps")
    }

    fn ids(apps: &[InstalledApp]) -> Vec<&str> {
        apps.iter().map(|app| app.bundle_id.as_str()).collect()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{
            BundleId, CancelToken, CommandRunError, InstalledApps, InstalledList, KnownFolder, SystemError,
        };
        use fresher_testing::{FakeCommandRunner, FakeKnownFolders};

        use super::{ids, list_of, list_of_home, listed, toolbox};
        use crate::MacInstalledApps;
        use crate::bundle::fixture::app;

        /// Makes `folder` unreadable while `check` runs, then readable again so the temporary folder can go.
        fn closed<T>(folder: &Path, check: impl FnOnce() -> T) -> T {
            fs::set_permissions(folder, fs::Permissions::from_mode(0o000)).unwrap();
            let result = check();
            fs::set_permissions(folder, fs::Permissions::from_mode(0o755)).unwrap();
            result
        }

        #[test]
        fn bundles_inside_bundles_and_deeper_folders_are_not_apps() {
            let root = tempfile::tempdir().unwrap();
            let host = app(root.path(), "Host", "com.example.Host");
            app(&host.join("Contents/Applications"), "Inner", "com.example.Inner");
            app(&root.path().join("Vendor/Deeper"), "Deep", "com.example.Deep");

            assert_eq!(ids(&listed(root.path())), ["com.example.Host"]);
        }

        #[test]
        fn an_app_link_that_leads_to_no_bundle_is_unread() {
            let root = tempfile::tempdir().unwrap();
            let elsewhere = tempfile::tempdir().unwrap();
            fs::write(elsewhere.path().join("file"), "").unwrap();
            std::os::unix::fs::symlink(elsewhere.path().join("missing.app"), root.path().join("Gone.app")).unwrap();
            std::os::unix::fs::symlink(elsewhere.path().join("file"), root.path().join("File.app")).unwrap();
            let empty = elsewhere.path().join("Empty.app");
            fs::create_dir(&empty).unwrap();
            std::os::unix::fs::symlink(&empty, root.path().join("Empty.app")).unwrap();

            let list = list_of(root.path());

            assert_eq!(list.apps, []);
            let unread =
                [root.path().join("File.app"), fs::canonicalize(&empty).unwrap(), root.path().join("Gone.app")];
            let mut expected = unread.to_vec();
            expected.sort();
            assert_eq!(list.unread, expected);
        }

        #[test]
        fn a_link_to_a_folder_below_an_apps_folder_is_unread_and_never_walked() {
            let root = tempfile::tempdir().unwrap();
            let elsewhere = tempfile::tempdir().unwrap();
            app(elsewhere.path(), "Hidden", "com.example.Hidden");
            fs::write(elsewhere.path().join("note.txt"), "").unwrap();
            std::os::unix::fs::symlink(elsewhere.path(), root.path().join("Vendor")).unwrap();
            std::os::unix::fs::symlink(elsewhere.path().join("note.txt"), root.path().join("note")).unwrap();

            let list = list_of(root.path());

            assert_eq!(list.apps, []);
            assert_eq!(list.unread, [root.path().join("Vendor")]);
        }

        #[test]
        fn missing_apps_folders_list_no_app() {
            let missing = FakeKnownFolders::new()
                .with(KnownFolder::Applications, "/nonexistent/fresher-apps")
                .with(KnownFolder::AppSupport, "/nonexistent/fresher-support");

            assert_eq!(
                MacInstalledApps::new(Arc::new(missing), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list(),
                Ok(InstalledList::default())
            );
            assert_eq!(
                MacInstalledApps::new(
                    Arc::new(FakeKnownFolders::new()),
                    crate::AppSearch::Declared(crate::DeclaredSearch::listed())
                )
                .list(),
                Ok(InstalledList::default())
            );
        }

        #[test]
        fn without_a_reader_the_bundle_is_unknown() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "Tool", "com.example.Tool");
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, root.path());
            let apps =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()));
            let listed = apps.list().unwrap().apps;

            assert!(apps.bundle(&listed[0], &CancelToken::new()).is_err());
        }

        #[test]
        fn an_apps_folder_that_cannot_be_read_is_an_error_when_it_is_the_only_one() {
            let root = tempfile::tempdir().unwrap();
            let closed_folder = root.path().join("closed");
            fs::create_dir(&closed_folder).unwrap();
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, &closed_folder);

            let listed = closed(&closed_folder, || {
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list()
            });

            assert!(listed.is_err(), "{listed:?}");
        }

        #[test]
        fn an_apps_folder_that_cannot_be_read_beside_a_readable_one_is_unread() {
            let root = tempfile::tempdir().unwrap();
            app(&root.path().join("open"), "App", "com.example.App");
            let closed_folder = root.path().join("closed");
            fs::create_dir(&closed_folder).unwrap();
            let folders = FakeKnownFolders::new()
                .with(KnownFolder::Applications, root.path().join("open"))
                .with(KnownFolder::UserApplications, &closed_folder);

            let list = closed(&closed_folder, || {
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list()
                    .unwrap()
            });

            assert_eq!(ids(&list.apps), ["com.example.App"]);
            assert_eq!(list.unread, [closed_folder]);
        }

        #[test]
        fn a_folder_below_an_apps_folder_that_cannot_be_read_is_unread() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "App", "com.example.App");
            let vendor = root.path().join("Vendor");
            app(&vendor, "Tool", "com.example.Tool");

            let list = closed(&vendor, || list_of(root.path()));

            assert_eq!(ids(&list.apps), ["com.example.App"]);
            assert_eq!(list.unread, [vendor]);
        }

        #[test]
        fn a_bundle_whose_status_change_cannot_be_read_has_no_change_time() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(crate::apps::changed(&root.path().join("Missing.app")), None);
        }

        #[test]
        fn a_bundle_whose_info_plist_cannot_be_read_is_unread() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "App", "com.example.App");
            fs::create_dir(root.path().join("NoPlist.app")).unwrap();
            let broken = root.path().join("Vendor/Broken.app/Contents");
            fs::create_dir_all(&broken).unwrap();
            fs::write(broken.join("Info.plist"), "not a property list").unwrap();

            let list = list_of(root.path());

            assert_eq!(ids(&list.apps), ["com.example.App"]);
            assert_eq!(list.unread, [root.path().join("NoPlist.app"), root.path().join("Vendor/Broken.app")]);
            assert!(!list.is_complete());
        }

        fn spotlight(runner: FakeCommandRunner) -> MacInstalledApps {
            MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::spotlight::tests::spotlight(runner))
        }

        fn bundle_ids(ids: &[&str]) -> Vec<BundleId> {
            ids.iter().map(|id| BundleId::new(*id).unwrap()).collect()
        }

        const QUERY: &str = "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.jetbrains.datagrip'c";

        #[test]
        fn a_spotlight_that_cannot_be_asked_or_fails_is_an_error() {
            let refused = CommandRunError::Failed { program: "/usr/bin/mdfind".into(), reason: "timed out".into() };
            for runner in [
                FakeCommandRunner::default(),
                FakeCommandRunner::default().failing(QUERY, refused),
                FakeCommandRunner::default().exiting(QUERY, 1),
            ] {
                let found = spotlight(runner).find(&bundle_ids(&["com.jetbrains.datagrip"]), &CancelToken::new());

                assert!(found.is_err(), "{found:?}");
            }
        }

        #[test]
        fn a_spotlight_search_with_no_home_is_not_complete() {
            let complete = spotlight(crate::spotlight::tests::indexed()).search_complete(&CancelToken::new());

            assert_eq!(complete, Err(SystemError { reason: "the home folder is unknown".into() }));
        }

        #[test]
        fn a_declared_search_with_the_index_off_is_not_complete() {
            let declared = crate::DeclaredSearch { index_on: false, elsewhere: Vec::new() };
            let apps = MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::AppSearch::Declared(declared));

            assert_eq!(
                apps.search_complete(&CancelToken::new()),
                Err(SystemError { reason: "the sandbox's Spotlight is off".into() })
            );
        }

        #[test]
        fn a_path_spotlight_answers_that_does_not_exist_is_not_an_app() {
            let runner = FakeCommandRunner::default().answering(QUERY, "/nonexistent/fresher/DataGrip.app\0");

            assert_eq!(
                spotlight(runner).find(&bundle_ids(&["com.jetbrains.datagrip"]), &CancelToken::new()),
                Ok(vec![])
            );
        }

        #[test]
        fn the_toolbox_folder_is_never_read_through_a_link_nor_deeper_than_four_folders() {
            let home = tempfile::tempdir().unwrap();
            let elsewhere = tempfile::tempdir().unwrap();
            app(&elsewhere.path().join("DataGrip/ch-0/253.1"), "DataGrip", "com.jetbrains.datagrip");
            fs::create_dir_all(toolbox(home.path()).parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(elsewhere.path(), toolbox(home.path())).unwrap();

            let list = list_of_home(home.path());

            assert_eq!(list.apps, []);
            assert_eq!(list.unread, [toolbox(home.path())]);

            let other = tempfile::tempdir().unwrap();
            app(&toolbox(other.path()).join("DataGrip/ch-0/253.1/deeper"), "DataGrip", "com.jetbrains.datagrip");
            assert_eq!(list_of_home(other.path()), InstalledList::default());
        }

        #[test]
        fn a_toolbox_folder_that_cannot_be_read_is_unread() {
            let home = tempfile::tempdir().unwrap();
            let folder = toolbox(home.path());
            fs::create_dir_all(&folder).unwrap();

            let list = closed(&folder, || list_of_home(home.path()));

            assert_eq!(list.unread, [folder]);
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{InstalledApps, KnownFolder};
        use fresher_testing::{FIXTURE_NOW, FakeKnownFolders, TreeDescription};

        use super::{ids, list_of_home, listed, toolbox};
        use crate::bundle::fixture::{app, bundle};
        use crate::{MacInstalledApps, MacKnownFolders};

        #[test]
        fn each_listing_reads_the_casks_again() {
            let root = tempfile::tempdir().unwrap();
            let apps_folder = root.path().join("Applications");
            let tool = app(&apps_folder, "Tool", "com.example.Tool");
            let casks = root.path().join("Caskroom");
            let version = casks.join("tool/1.0");
            std::fs::create_dir_all(&version).unwrap();
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, &apps_folder);
            let reader = crate::BundleReader::with(None, vec![casks]);
            let apps =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .with_bundle_reader(reader, fresher_macos::current_user());
            let cancel = fresher_core::CancelToken::new();

            let listed = apps.list().unwrap().apps;
            assert_eq!(apps.bundle(&listed[0], &cancel).unwrap().installed_by, None);
            std::os::unix::fs::symlink(&tool, version.join("Tool.app")).unwrap();
            assert_eq!(apps.bundle(&listed[0], &cancel).unwrap().installed_by, None);

            let listed = apps.list().unwrap().apps;

            let found = apps.bundle(&listed[0], &cancel).unwrap().installed_by;
            assert_eq!(found.map(|by| by.token().to_owned()), Some("tool".into()));
        }

        #[test]
        fn with_a_reader_a_listed_bundle_is_measured() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "Tool", "com.example.Tool");
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, root.path());
            let apps =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .with_bundle_reader(crate::BundleReader::stated(), fresher_macos::current_user());
            let listed = apps.list().unwrap().apps;

            let facts = apps.bundle(&listed[0], &fresher_core::CancelToken::new()).unwrap();

            assert!(facts.sizes.logical.bytes() > 0);
            assert!(facts.entry.owned_by_user);
        }

        #[test]
        fn an_installed_app_carries_its_bundles_status_change() {
            use std::os::unix::fs::MetadataExt as _;

            let root = tempfile::tempdir().unwrap();
            let bundle = app(root.path(), "Zed", "com.example.Zed");
            let status_change = std::fs::symlink_metadata(&bundle).unwrap().ctime();

            let apps = listed(root.path());

            let status_change = fresher_core::Timestamp::from_secs(u64::try_from(status_change).unwrap());
            assert_eq!(apps[0].changed, Some(status_change));
        }

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
        fn spotlight_is_asked_for_every_id_and_its_existing_paths_are_found() {
            let drive = tempfile::tempdir().unwrap();
            let release = app(drive.path(), "DataGrip", "com.jetbrains.datagrip");
            let eap = app(drive.path(), "DataGrip EAP", "com.jetbrains.datagrip-EAP");
            let runner = fresher_testing::FakeCommandRunner::default()
                .answering(
                    "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.jetbrains.datagrip'c",
                    &format!("{}\0", release.display()),
                )
                .answering(
                    "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.jetbrains.datagrip-EAP'c",
                    &format!("{}\0{}\0", eap.display(), release.display()),
                );
            let apps =
                MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::spotlight::tests::spotlight(runner));
            let ids: Vec<_> = ["com.jetbrains.datagrip", "com.jetbrains.datagrip-EAP"]
                .iter()
                .map(|id| fresher_core::BundleId::new(*id).unwrap())
                .collect();

            let mut expected = vec![release, eap];
            expected.sort();
            assert_eq!(apps.find(&ids, &fresher_core::CancelToken::new()), Ok(expected));
        }

        #[test]
        fn a_path_with_a_line_break_spotlight_answers_stays_whole() {
            let drive = tempfile::tempdir().unwrap();
            let odd = app(&drive.path().join("two\nlines"), "DataGrip", "com.jetbrains.datagrip");
            let runner = fresher_testing::FakeCommandRunner::default().answering(
                "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.jetbrains.datagrip'c",
                &format!("{}\0", odd.display()),
            );
            let apps =
                MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::spotlight::tests::spotlight(runner));
            let id = fresher_core::BundleId::new("com.jetbrains.datagrip").unwrap();

            assert_eq!(apps.find(&[id], &fresher_core::CancelToken::new()), Ok(vec![odd]));
        }

        #[test]
        fn a_listed_search_finds_only_the_listed_apps() {
            let root = tempfile::tempdir().unwrap();
            let listed = app(root.path(), "DataGrip", "com.jetbrains.datagrip");
            let folders = FakeKnownFolders::new().with(KnownFolder::Applications, root.path());
            let apps =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()));
            let id = |id: &str| fresher_core::BundleId::new(id).unwrap();

            assert_eq!(apps.find(&[id("com.jetbrains.datagrip")], &fresher_core::CancelToken::new()), Ok(vec![listed]));
            assert_eq!(apps.find(&[id("com.jetbrains.goland")], &fresher_core::CancelToken::new()), Ok(vec![]));
        }

        #[test]
        fn a_declared_search_finds_its_elsewhere_paths_for_their_ids_only() {
            let id = |id: &str| fresher_core::BundleId::new(id).unwrap();
            let declared = crate::DeclaredSearch {
                index_on: true,
                elsewhere: vec![
                    (id("com.example.Far"), "/Volumes/Drive/Far.app".into()),
                    (id("com.example.Other"), "/Volumes/Drive/Other.app".into()),
                ],
            };
            let apps = MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::AppSearch::Declared(declared));
            let cancel = fresher_core::CancelToken::new();

            assert_eq!(apps.find(&[id("com.example.Far")], &cancel), Ok(vec!["/Volumes/Drive/Far.app".into()]));
            assert_eq!(apps.find(&[id("com.example.None")], &cancel), Ok(vec![]));
            assert_eq!(apps.search_complete(&cancel), Ok(()));
        }

        #[test]
        fn finding_no_longer_asks_spotlight_for_the_finder() {
            let query = "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.jetbrains.datagrip'c";
            let runner = fresher_testing::FakeCommandRunner::default().answering(query, "");
            let (runner, search) = crate::spotlight::tests::search(runner, Ok(Vec::new()));
            let apps = MacInstalledApps::new(Arc::new(FakeKnownFolders::new()), crate::AppSearch::Spotlight(search));
            let id = fresher_core::BundleId::new("com.jetbrains.datagrip").unwrap();

            assert_eq!(apps.find(&[id], &fresher_core::CancelToken::new()), Ok(vec![]));
            assert_eq!(runner.calls(), [query]);
        }

        #[test]
        fn a_spotlight_search_is_complete_for_the_volume_of_the_home() {
            let folders = FakeKnownFolders::new().with(KnownFolder::Home, crate::spotlight::tests::HOME);
            let apps = MacInstalledApps::new(
                Arc::new(folders),
                crate::spotlight::tests::spotlight(crate::spotlight::tests::indexed()),
            );

            assert_eq!(apps.search_complete(&fresher_core::CancelToken::new()), Ok(()));
        }

        #[test]
        fn an_app_that_is_a_link_is_read_where_it_points() {
            let root = tempfile::tempdir().unwrap();
            let elsewhere = tempfile::tempdir().unwrap();
            let target = app(elsewhere.path(), "DataGrip", "com.jetbrains.datagrip");
            std::os::unix::fs::symlink(&target, root.path().join("DataGrip.app")).unwrap();

            let apps = listed(root.path());

            assert_eq!(ids(&apps), ["com.jetbrains.datagrip"]);
            assert_eq!(apps[0].path, std::fs::canonicalize(&target).unwrap());
        }

        #[test]
        fn an_ide_the_toolbox_app_installed_in_its_users_apps_folder_is_listed() {
            let home = tempfile::tempdir().unwrap();
            app(&home.path().join("Applications/JetBrains Toolbox"), "DataGrip", "com.jetbrains.datagrip");

            let list = list_of_home(home.path());

            assert!(list.is_complete(), "{:?}", list.unread);
            assert_eq!(ids(&list.apps), ["com.jetbrains.datagrip"]);
        }

        #[test]
        fn an_ide_the_older_toolbox_app_kept_in_its_own_folder_is_listed() {
            let home = tempfile::tempdir().unwrap();
            let folder = toolbox(home.path());
            app(&folder.join("DataGrip/ch-0/253.1"), "DataGrip", "com.jetbrains.datagrip");
            app(&folder.join("WebStorm"), "WebStorm", "com.jetbrains.WebStorm");
            std::fs::create_dir_all(folder.join("DataGrip/ch-0/253.1.plugins/some-plugin")).unwrap();

            let list = list_of_home(home.path());

            assert!(list.is_complete(), "{:?}", list.unread);
            assert_eq!(ids(&list.apps), ["com.jetbrains.datagrip", "com.jetbrains.WebStorm"]);
            assert_eq!(list.apps[0].path, folder.join("DataGrip/ch-0/253.1/DataGrip.app"));
        }

        // The bundles a sandbox case declares (`[[case.app]]`, written by the tree's `[[app]]`) read as a Mac's.
        #[test]
        fn the_apps_of_a_sandbox_tree_are_read_as_declared() {
            let sandbox = tempfile::tempdir().unwrap();
            let text = r#"
[[app]]
path = "Applications/WebStorm.app"
bundle-id = "com.jetbrains.WebStorm"
name = "WebStorm"
version = "2025.3.1"
build = "WS-253.1"
[[app]]
path = "System/Applications/Utilities/Plain.app"
bundle-id = "com.example.Plain"
name = "Plain"
[[app]]
path = "home/Applications/Odd.app"
bundle-id = "com.example.Odd"
name = "A & <B> \"c\" 'd'"
version = "1.0"
"#;
            let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(FIXTURE_NOW);
            TreeDescription::from_toml_str(text).unwrap().build_at(sandbox.path(), now).unwrap();
            let folders = FakeKnownFolders::home_layout(sandbox.path().join("home"))
                .with(KnownFolder::Applications, sandbox.path().join("Applications"))
                .with(KnownFolder::SystemApplications, sandbox.path().join("System/Applications"));

            let list =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list()
                    .unwrap();

            assert!(list.is_complete(), "{:?}", list.unread);
            let read: Vec<_> = list
                .apps
                .iter()
                .map(|app| (app.bundle_id.as_str(), app.name.as_str(), app.version.as_deref(), app.build.as_deref()))
                .collect();
            assert_eq!(
                read,
                [
                    ("com.jetbrains.WebStorm", "WebStorm", Some("2025.3.1"), Some("WS-253.1")),
                    ("com.example.Plain", "Plain", None, None),
                    ("com.example.Odd", "A & <B> \"c\" 'd'", Some("1.0"), None),
                ]
            );
            assert!(list.apps.iter().all(|app| app.team_id.is_none() && app.helpers.is_empty()));
        }

        #[test]
        fn the_same_folder_named_twice_lists_its_apps_once() {
            let root = tempfile::tempdir().unwrap();
            app(root.path(), "App", "com.example.App");
            let folders = FakeKnownFolders::new()
                .with(KnownFolder::Applications, root.path())
                .with(KnownFolder::UserApplications, root.path());

            assert_eq!(
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list()
                    .unwrap()
                    .apps
                    .len(),
                1
            );
        }

        /// Reads this Mac's own system apps, read-only; skipped where the folder is missing.
        #[test]
        fn the_system_apps_of_this_mac_have_apple_ids() {
            if !Path::new("/System/Applications").is_dir() {
                return;
            }
            let folders = MacKnownFolders::new("/nonexistent/fresher-home");

            let apps =
                MacInstalledApps::new(Arc::new(folders), crate::AppSearch::Declared(crate::DeclaredSearch::listed()))
                    .list()
                    .unwrap()
                    .apps;

            let system: Vec<_> = apps.iter().filter(|app| app.path.starts_with("/System/Applications")).collect();
            assert_ne!(system.len(), 0);
            assert!(system.iter().all(|app| app.bundle_id.folded().starts_with("com.apple.")), "{system:?}");
        }
    }
}
