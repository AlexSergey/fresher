//! `BundleReader`: what an app's bundle takes, whether the user may move it, when it was last used and who installed
//! it. Metadata only: no file's contents are read, nothing is written, no link is followed.

mod entry;
mod groups;
mod last_used;
mod managers;
mod measure;

use std::fmt;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use fresher_core::{BundleFacts, CancelToken, CommandRunner, InstalledApp, SystemError};

use entry::{Access, entry_facts};
use groups::{GroupsUnread, user_groups};
use last_used::last_used;
use managers::{CASKROOMS, CaskIndex};
use measure::measure;

/// Reads the facts of installed bundles. On the Mac ([`BundleReader::mac`]) it asks Spotlight for the last use and
/// reads Homebrew's Caskrooms; a sandbox's or a test's ([`BundleReader::stated`]) asks nothing of the Mac. The
/// Caskrooms' links are read once per run (a listing of the installed apps starts one: [`BundleReader::start_run`]),
/// for every bundle measured in it, so a cask installed while the app runs is seen at the next listing. The user's
/// groups are kept once read; a failed read is tried again at the next run.
pub struct BundleReader {
    runner: Option<Arc<dyn CommandRunner>>,
    casks: Vec<PathBuf>,
    groups: OnceLock<Vec<u32>>,
    run: Mutex<Run>,
}

/// What one run has read so far.
#[derive(Debug, Default)]
struct Run {
    index: Option<Arc<CaskIndex>>,
    /// The user's groups could not be read in this run: no group counts until the next run.
    groups_failed: bool,
}

impl fmt::Debug for BundleReader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BundleReader").field("casks", &self.casks).finish_non_exhaustive()
    }
}

impl BundleReader {
    /// The Mac's reader: Spotlight's date and the user's groups through `runner`, Homebrew's casks from its Caskrooms.
    pub fn mac(runner: Arc<dyn CommandRunner>) -> Self {
        Self::with(Some(runner), CASKROOMS.iter().map(PathBuf::from).collect())
    }

    /// A reader that runs no command and reads no Caskroom: no last-used date, no package manager, no group (only
    /// the owner's and the others' write bits count). What a sandbox states is laid over it.
    pub fn stated() -> Self {
        Self::with(None, Vec::new())
    }

    pub(crate) fn with(runner: Option<Arc<dyn CommandRunner>>, casks: Vec<PathBuf>) -> Self {
        Self { runner, casks, groups: OnceLock::new(), run: Mutex::new(Run::default()) }
    }

    /// Starts a new run: the Caskrooms are read again at the next bundle, and a failed read of the groups is tried
    /// again.
    pub(crate) fn start_run(&self) {
        *self.lock_run() = Run::default();
    }

    /// The facts of `app`'s bundle for `user`. Only a whole bundle is read: a real folder (not a link) holding a real
    /// `Contents` folder with an `Info.plist` file, in a real folder; anything else, or a bundle `lstat` cannot read,
    /// is an error that names no path. A cancelled measure is an error too.
    pub fn read(&self, app: &InstalledApp, user: u32, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        let bundle = app.path.as_path();
        let (own, parent) = whole_bundle(bundle).ok_or_else(unreadable)?;
        let sizes = measure(bundle, cancel).map_err(|error| {
            if error.kind() == io::ErrorKind::Interrupted {
                SystemError { reason: "the bundle's measure was cancelled".into() }
            } else {
                tracing::debug!(%error, "a bundle cannot be measured");
                unreadable()
            }
        })?;
        let entry = entry_facts(Access::of(&own), Access::of(&parent), user, &self.groups(cancel));
        let last_used = self.runner.as_deref().and_then(|runner| last_used(runner, bundle, cancel));
        let installed_by = self.index().installed_by(bundle);
        Ok(BundleFacts { sizes, entry, last_used, installed_by })
    }

    /// The Caskrooms' links of this run, read at its first bundle.
    fn index(&self) -> Arc<CaskIndex> {
        let mut run = self.lock_run();
        Arc::clone(run.index.get_or_insert_with(|| Arc::new(CaskIndex::read(&self.casks))))
    }

    /// The user's groups: kept once read; a failure is kept for the rest of the run, a cancelled read is not kept.
    fn groups(&self, cancel: &CancelToken) -> Vec<u32> {
        if let Some(groups) = self.groups.get() {
            return groups.clone();
        }
        let Some(runner) = self.runner.as_deref() else { return Vec::new() };
        let mut run = self.lock_run();
        if run.groups_failed {
            return Vec::new();
        }
        match user_groups(runner, cancel) {
            Ok(groups) => self.groups.get_or_init(|| groups).clone(),
            Err(GroupsUnread::Failed) => {
                run.groups_failed = true;
                Vec::new()
            }
            Err(GroupsUnread::Cancelled) => Vec::new(),
        }
    }

    fn lock_run(&self) -> std::sync::MutexGuard<'_, Run> {
        self.run.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The `lstat` of `bundle` and of its folder when `bundle` is a whole bundle: a real folder with a real `Contents`
/// folder and a `Contents/Info.plist` file, in a real folder. A link at any of them is not one.
fn whole_bundle(bundle: &Path) -> Option<(Metadata, Metadata)> {
    let own = fs::symlink_metadata(bundle).ok().filter(Metadata::is_dir)?;
    let contents = bundle.join("Contents");
    fs::symlink_metadata(&contents).ok().filter(Metadata::is_dir)?;
    fs::symlink_metadata(contents.join("Info.plist")).ok().filter(Metadata::is_file)?;
    let parent = fs::symlink_metadata(bundle.parent()?).ok().filter(Metadata::is_dir)?;
    Some((own, parent))
}

fn unreadable() -> SystemError {
    SystemError { reason: "the bundle cannot be read".into() }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use fresher_core::InstalledApp;
    use fresher_testing::installed_app;

    /// The listed app whose bundle is at `path`.
    fn app_at(path: &Path) -> InstalledApp {
        InstalledApp { path: path.to_path_buf(), ..installed_app("com.example.Tool", "Tool", "1.0").unwrap() }
    }

    /// A Caskroom in `root` with the cask `tool` linking to `target`.
    fn caskroom(root: &Path, target: &Path) -> PathBuf {
        let version = root.join("Caskroom/tool/2.0");
        std::fs::create_dir_all(&version).unwrap();
        std::os::unix::fs::symlink(target, version.join("Tool.app")).unwrap();
        root.join("Caskroom")
    }

    mod negative {
        use std::fs;
        use std::sync::Arc;

        use fresher_core::CancelToken;
        use fresher_testing::FakeCommandRunner;

        use super::{app_at, caskroom};
        use crate::BundleReader;
        use crate::bundle::fixture::app;

        #[test]
        fn what_is_not_a_whole_bundle_is_not_read() {
            let root = tempfile::tempdir().unwrap();
            let apps = root.path().join("Applications");
            let real = app(&root.path().join("elsewhere"), "Real", "com.example.Real");
            fs::create_dir_all(&apps).unwrap();
            let link = apps.join("Link.app");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let empty = apps.join("Empty.app");
            fs::create_dir(&empty).unwrap();
            let no_plist = apps.join("NoPlist.app");
            fs::create_dir_all(no_plist.join("Contents")).unwrap();
            let file = apps.join("File.app");
            fs::write(&file, b"").unwrap();
            let linked_contents = apps.join("LinkedContents.app");
            fs::create_dir(&linked_contents).unwrap();
            std::os::unix::fs::symlink(real.join("Contents"), linked_contents.join("Contents")).unwrap();
            let linked_plist = apps.join("LinkedPlist.app");
            fs::create_dir_all(linked_plist.join("Contents")).unwrap();
            std::os::unix::fs::symlink(real.join("Contents/Info.plist"), linked_plist.join("Contents/Info.plist"))
                .unwrap();
            let folder_plist = apps.join("FolderPlist.app");
            fs::create_dir_all(folder_plist.join("Contents/Info.plist")).unwrap();
            let runner = Arc::new(FakeCommandRunner::default());
            let reader = BundleReader::with(
                Some(Arc::clone(&runner) as Arc<dyn fresher_core::CommandRunner>),
                vec![caskroom(root.path(), &link)],
            );

            for bundle in
                [link, empty, no_plist, file, linked_contents, linked_plist, folder_plist, apps.join("Gone.app")]
            {
                let error = reader.read(&app_at(&bundle), 501, &CancelToken::new()).unwrap_err();

                assert_eq!(error.reason, "the bundle cannot be read", "{}", bundle.display());
            }
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_bundle_in_a_folder_that_is_a_link_is_not_read() {
            let root = tempfile::tempdir().unwrap();
            app(&root.path().join("real"), "Tool", "com.example.Tool");
            std::os::unix::fs::symlink(root.path().join("real"), root.path().join("Applications")).unwrap();
            let bundle = root.path().join("Applications/Tool.app");

            let error = BundleReader::stated().read(&app_at(&bundle), 501, &CancelToken::new()).unwrap_err();

            assert_eq!(error.reason, "the bundle cannot be read");
        }

        #[test]
        fn a_cancelled_read_is_an_error() {
            let root = tempfile::tempdir().unwrap();
            let bundle = app(root.path(), "Tool", "com.example.Tool");
            let cancel = CancelToken::new();
            cancel.cancel();

            let error = BundleReader::stated().read(&app_at(&bundle), 501, &cancel).unwrap_err();

            assert_eq!(error.reason, "the bundle's measure was cancelled");
        }
    }

    mod positive {
        use std::fs;
        use std::sync::Arc;

        use fresher_core::{
            CancelToken, CommandRunner, Completeness, EntryFacts, InstalledBy, Manager, Removability, Timestamp,
        };
        use fresher_testing::FakeCommandRunner;

        use super::{app_at, caskroom};
        use crate::BundleReader;
        use crate::bundle::fixture::app;

        #[test]
        fn a_whole_bundle_has_its_sizes_entry_date_and_manager() {
            let root = tempfile::tempdir().unwrap();
            let apps = root.path().join("Applications");
            let bundle = app(&apps, "Tool", "com.example.Tool");
            fs::write(bundle.join("Contents/data"), vec![0_u8; 3_000]).unwrap();
            let plist = fs::metadata(bundle.join("Contents/Info.plist")).unwrap().len();
            let user = fresher_macos::current_user();
            let path = bundle.to_str().unwrap();
            let runner = FakeCommandRunner::default()
                .answering(&format!("/usr/bin/mdls -raw -name kMDItemLastUsedDate {path}"), "2026-11-20 14:03:05 +0000")
                .answering("/usr/bin/id -G", "20 80");
            let runner = Arc::new(runner);
            let reader = BundleReader::with(
                Some(Arc::clone(&runner) as Arc<dyn CommandRunner>),
                vec![caskroom(root.path(), &bundle)],
            );

            let facts = reader.read(&app_at(&bundle), user, &CancelToken::new()).unwrap();

            assert_eq!(facts.sizes.logical.bytes(), 3_000 + plist);
            assert_eq!(facts.sizes.completeness, Completeness::Complete);
            assert_eq!(facts.entry, EntryFacts::REMOVABLE);
            assert_eq!(facts.removability(), Removability::Removable);
            assert_eq!(facts.last_used, Some(Timestamp::from_secs(1_795_183_385)));
            assert_eq!(facts.installed_by, Some(InstalledBy::new(Manager::Homebrew, "tool").unwrap()));
        }

        #[test]
        fn a_stated_reader_asks_nothing_of_the_mac() {
            let root = tempfile::tempdir().unwrap();
            let bundle = app(root.path(), "Tool", "com.example.Tool");

            let facts =
                BundleReader::stated().read(&app_at(&bundle), fresher_macos::current_user(), &CancelToken::new());

            let facts = facts.unwrap();
            assert_eq!((facts.last_used, facts.installed_by), (None, None));
            assert!(facts.entry.owned_by_user);
        }

        #[test]
        fn the_casks_are_read_once_per_run_and_again_at_the_next() {
            let root = tempfile::tempdir().unwrap();
            let first = app(root.path(), "First", "com.example.First");
            let second = app(root.path(), "Second", "com.example.Second");
            let records = caskroom(root.path(), &first);
            let reader = BundleReader::with(None, vec![records.clone()]);
            let cancel = CancelToken::new();

            assert!(reader.read(&app_at(&first), 501, &cancel).unwrap().installed_by.is_some());
            fs::create_dir_all(records.join("second/1.0")).unwrap();
            std::os::unix::fs::symlink(&second, records.join("second/1.0/Second.app")).unwrap();
            // The same run keeps the links it read.
            assert_eq!(reader.read(&app_at(&second), 501, &cancel).unwrap().installed_by, None);

            reader.start_run();

            let found = reader.read(&app_at(&second), 501, &cancel).unwrap().installed_by;
            assert_eq!(found, Some(InstalledBy::new(Manager::Homebrew, "second").unwrap()));
        }

        #[test]
        fn read_groups_are_kept_and_a_failed_read_is_tried_at_the_next_run() {
            let root = tempfile::tempdir().unwrap();
            let bundle = app(root.path(), "Tool", "com.example.Tool");
            let cancel = CancelToken::new();
            let count =
                |runner: &FakeCommandRunner| runner.calls().iter().filter(|call| *call == "/usr/bin/id -G").count();

            let answering = Arc::new(FakeCommandRunner::default().answering("/usr/bin/id -G", "20"));
            let reader = BundleReader::with(Some(Arc::clone(&answering) as Arc<dyn CommandRunner>), Vec::new());
            for _ in 0..2 {
                reader.read(&app_at(&bundle), 501, &cancel).unwrap();
                reader.start_run();
            }
            assert_eq!(count(&answering), 1);

            let failing = Arc::new(FakeCommandRunner::default().exiting("/usr/bin/id -G", 1));
            let reader = BundleReader::with(Some(Arc::clone(&failing) as Arc<dyn CommandRunner>), Vec::new());
            reader.read(&app_at(&bundle), 501, &cancel).unwrap();
            reader.read(&app_at(&bundle), 501, &cancel).unwrap();
            assert_eq!(count(&failing), 1);
            reader.start_run();
            reader.read(&app_at(&bundle), 501, &cancel).unwrap();
            assert_eq!(count(&failing), 2);
        }
    }
}
