//! `SpotlightSearch`: Spotlight's index, asked by bundle id, and whether it can see every app on the Mac.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fresher_core::{CancelToken, CommandRunner, SystemError};
use fresher_macos::MountedVolume;

/// How long `mdutil -s` may take for one volume before the volume counts as not indexed.
pub const INDEX_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// Spotlight's command line, run by its absolute path (never looked up in `PATH`) and without a shell.
const MDFIND: &str = "/usr/bin/mdfind";

/// Spotlight's index status command, run by its absolute path (never looked up in `PATH`) and without a shell.
const MDUTIL: &str = "/usr/bin/mdutil";

/// The bundle id of the Finder, which Spotlight knows on every Mac whose index is on.
const FINDER: &str = "com.apple.finder";

/// What `mdutil -s` says of a volume Spotlight indexes.
const INDEXING_ENABLED: &str = "Indexing enabled.";

/// The mounted volumes of the system.
pub trait Mounts: Send + Sync {
    /// Every mounted volume that may hold an app.
    fn mounted(&self) -> io::Result<Vec<MountedVolume>>;

    /// Where the volume that holds `path` is mounted.
    fn holding(&self, path: &Path) -> io::Result<PathBuf>;
}

/// The mounts of this Mac.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacMounts;

impl Mounts for MacMounts {
    fn mounted(&self) -> io::Result<Vec<MountedVolume>> {
        fresher_macos::mounted_volumes()
    }

    fn holding(&self, path: &Path) -> io::Result<PathBuf> {
        fresher_macos::mount_point_of(path)
    }
}

/// Spotlight's index: `mdfind` through `find`, `mdutil -s` through `index` (a runner with its own, short time limit).
#[derive(Clone)]
pub struct SpotlightSearch {
    find: Arc<dyn CommandRunner>,
    index: Arc<dyn CommandRunner>,
    mounts: Arc<dyn Mounts>,
}

impl fmt::Debug for SpotlightSearch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("SpotlightSearch").finish_non_exhaustive()
    }
}

impl SpotlightSearch {
    pub fn new(find: Arc<dyn CommandRunner>, index: Arc<dyn CommandRunner>, mounts: Arc<dyn Mounts>) -> Self {
        Self { find, index, mounts }
    }

    /// Whether Spotlight can see every app on the Mac: it knows the Finder (its index is on), and every volume the
    /// system shows, and the one that holds `home` (on APFS a volume the system does not show), is local and indexed.
    /// A network volume is never asked (the command may hang): it counts as not indexed. Anything unknown is an
    /// error; a mount point is logged, never put in the error's text.
    pub fn search_complete(&self, home: &Path, cancel: &CancelToken) -> Result<(), SystemError> {
        if self.paths(FINDER, cancel)?.is_empty() {
            return Err(SystemError { reason: "Spotlight does not know the Finder: its index is off".into() });
        }
        let mounted = self.mounts.mounted().map_err(|error| {
            tracing::warn!(%error, "the mounted volumes cannot be listed");
            SystemError { reason: "the mounted volumes cannot be listed".into() }
        })?;
        let home_volume = self.mounts.holding(home).map_err(|error| {
            tracing::warn!(%error, "the volume of the home folder is unknown");
            SystemError { reason: "the volume of the home folder is unknown".into() }
        })?;
        if !mounted.iter().any(|volume| volume.mount_point == home_volume) {
            tracing::warn!(mount_point = %home_volume.display(), "the volume of the home folder is not listed");
            return Err(SystemError { reason: "the volume of the home folder is not mounted".into() });
        }
        for volume in mounted.iter().filter(|volume| volume.browsable || volume.mount_point == home_volume) {
            if !volume.local {
                tracing::info!(mount_point = %volume.mount_point.display(), "a network volume is mounted");
                return Err(SystemError { reason: "a network volume is mounted".into() });
            }
            if !self.is_indexed(&volume.mount_point, cancel) {
                tracing::info!(mount_point = %volume.mount_point.display(), "a volume is not indexed");
                return Err(SystemError { reason: "a volume is not indexed".into() });
            }
        }
        Ok(())
    }

    /// Whether `mdutil -s` ends with the status 0 and says the volume at `mount_point` is indexed: a line after the
    /// first (`<mount point>:`, which may hold any text) reads exactly `Indexing enabled.`.
    fn is_indexed(&self, mount_point: &Path, cancel: &CancelToken) -> bool {
        // A mount point that is not UTF-8 cannot be passed as an argument: it is not known to be indexed.
        let Some(mount_point) = mount_point.to_str() else { return false };
        match self.index.run(MDUTIL, &["-s", mount_point], cancel) {
            Ok(output) => {
                output.status == 0 && output.stdout.lines().skip(1).any(|line| line.trim() == INDEXING_ENABLED)
            }
            Err(error) => {
                tracing::info!(%error, "the volume's index status cannot be read");
                false
            }
        }
    }

    /// The paths Spotlight answers for the bundle id `id`, read NUL-separated so a path with a line break stays whole.
    /// `id` is checked reverse-DNS by the caller, so it cannot break out of the quotes.
    pub(crate) fn paths(&self, id: &str, cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        let query = format!("kMDItemCFBundleIdentifier == '{id}'c");
        let output = self
            .find
            .run(MDFIND, &["-0", &query], cancel)
            .map_err(|error| SystemError { reason: format!("Spotlight could not be asked: {error}") })?;
        if output.status != 0 {
            return Err(SystemError { reason: format!("Spotlight answered with the status {}", output.status) });
        }
        Ok(output.stdout.split('\0').filter(|path| !path.is_empty()).map(PathBuf::from).collect())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use fresher_core::{CancelToken, CommandOutput, CommandRunError, CommandRunner, SystemError};
    use fresher_macos::MountedVolume;
    use fresher_testing::FakeCommandRunner;

    use super::{Mounts, SpotlightSearch};

    const FINDER: &str = "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.apple.finder'c";
    pub(crate) const HOME: &str = "/Users/someone";
    const DATA: &str = "/System/Volumes/Data";
    const ENABLED: &str = "/:\n\tIndexing enabled. \n";

    /// The mounted volumes a test sets, or an error; the home is on the data volume.
    struct FakeMounts(io::Result<Vec<MountedVolume>>);

    impl Mounts for FakeMounts {
        fn mounted(&self) -> io::Result<Vec<MountedVolume>> {
            self.0.as_ref().map(Clone::clone).map_err(|error| io::Error::new(error.kind(), error.to_string()))
        }

        fn holding(&self, path: &Path) -> io::Result<PathBuf> {
            if path == Path::new(HOME) { Ok(DATA.into()) } else { Err(io::Error::from(io::ErrorKind::NotFound)) }
        }
    }

    fn volume(mount_point: &str, local: bool, browsable: bool) -> MountedVolume {
        MountedVolume { mount_point: mount_point.into(), local, browsable }
    }

    /// The root and the data volume of an APFS Mac: the data volume, which holds the home, is not browsable.
    fn system_volumes() -> Vec<MountedVolume> {
        vec![volume("/", true, true), volume(DATA, true, false)]
    }

    /// A runner whose Spotlight knows the Finder and calls the root and the data volume indexed.
    pub(crate) fn indexed() -> FakeCommandRunner {
        FakeCommandRunner::default()
            .answering(FINDER, "/System/Library/CoreServices/Finder.app\0")
            .answering("/usr/bin/mdutil -s /", ENABLED)
            .answering(&format!("/usr/bin/mdutil -s {DATA}"), ENABLED)
    }

    pub(crate) fn search(
        runner: FakeCommandRunner,
        mounts: io::Result<Vec<MountedVolume>>,
    ) -> (Arc<FakeCommandRunner>, SpotlightSearch) {
        let runner = Arc::new(runner);
        let search =
            SpotlightSearch::new(Arc::clone(&runner) as _, Arc::clone(&runner) as _, Arc::new(FakeMounts(mounts)));
        (runner, search)
    }

    /// A Spotlight search of `runner` on the volumes of an APFS Mac, for `MacInstalledApps`.
    pub(crate) fn spotlight(runner: FakeCommandRunner) -> crate::AppSearch {
        crate::AppSearch::Spotlight(search(runner, Ok(system_volumes())).1)
    }

    fn check(runner: FakeCommandRunner, mounts: io::Result<Vec<MountedVolume>>) -> Result<(), SystemError> {
        search(runner, mounts).1.search_complete(Path::new(HOME), &CancelToken::new())
    }

    fn error(reason: &str) -> Result<(), SystemError> {
        Err(SystemError { reason: reason.into() })
    }

    /// Records every program and its arguments as given, and calls every volume indexed.
    #[derive(Default)]
    struct Recording(Mutex<Vec<(String, Vec<String>)>>);

    impl CommandRunner for Recording {
        fn run(&self, program: &str, args: &[&str], _cancel: &CancelToken) -> Result<CommandOutput, CommandRunError> {
            self.0.lock().unwrap().push((program.into(), args.iter().map(|arg| (*arg).to_owned()).collect()));
            Ok(CommandOutput { status: 0, stdout: ENABLED.into(), stderr: String::new() })
        }
    }

    /// Ends every `mdutil -s` with the status 1 while saying the volume is indexed.
    struct FailingButEnabled;

    impl CommandRunner for FailingButEnabled {
        fn run(&self, _program: &str, _args: &[&str], _cancel: &CancelToken) -> Result<CommandOutput, CommandRunError> {
            Ok(CommandOutput { status: 1, stdout: ENABLED.into(), stderr: String::new() })
        }
    }

    mod negative {
        use std::io;
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{CancelToken, CommandRunError};
        use fresher_testing::FakeCommandRunner;

        use super::{
            DATA, FINDER, FailingButEnabled, FakeMounts, HOME, check, error, indexed, search, system_volumes, volume,
        };
        use crate::SpotlightSearch;

        #[test]
        fn a_spotlight_that_does_not_know_the_finder_is_off() {
            let runner = FakeCommandRunner::default().answering(FINDER, "");

            let (runner, search) = search(runner, Ok(system_volumes()));

            assert_eq!(
                search.search_complete(Path::new(HOME), &CancelToken::new()),
                error("Spotlight does not know the Finder: its index is off")
            );
            assert_eq!(runner.calls(), [FINDER]);
        }

        #[test]
        fn a_spotlight_that_cannot_be_asked_or_fails_is_an_error() {
            let refused = CommandRunError::Failed { program: "/usr/bin/mdfind".into(), reason: "timed out".into() };
            for runner in [
                FakeCommandRunner::default(),
                FakeCommandRunner::default().failing(FINDER, refused),
                FakeCommandRunner::default().exiting(FINDER, 1),
            ] {
                let answer = check(runner, Ok(system_volumes()));

                assert!(answer.is_err(), "{answer:?}");
            }
        }

        #[test]
        fn mounts_that_cannot_be_listed_are_an_error() {
            let answer = check(indexed(), Err(io::Error::other("no table")));

            assert_eq!(answer, error("the mounted volumes cannot be listed"));
        }

        #[test]
        fn a_home_whose_volume_is_unknown_or_not_mounted_is_an_error() {
            let (_, search) = search(indexed(), Ok(system_volumes()));

            assert_eq!(
                search.search_complete(Path::new("/elsewhere"), &CancelToken::new()),
                error("the volume of the home folder is unknown")
            );
            assert_eq!(
                check(indexed(), Ok(vec![volume("/", true, true)])),
                error("the volume of the home folder is not mounted")
            );
        }

        #[test]
        fn a_network_volume_is_not_indexed_and_never_asked() {
            let mut volumes = system_volumes();
            volumes.push(volume("/Volumes/share", false, true));
            let (runner, search) = search(indexed(), Ok(volumes));

            let answer = search.search_complete(Path::new(HOME), &CancelToken::new());

            assert_eq!(answer, error("a network volume is mounted"));
            assert!(runner.calls().iter().all(|call| !call.contains("/Volumes/share")), "{:?}", runner.calls());
        }

        #[test]
        fn a_volume_whose_index_is_off_unknown_or_unasked_is_not_indexed() {
            let work = "/usr/bin/mdutil -s /Volumes/Work";
            let failed =
                || CommandRunError::Failed { program: "/usr/bin/mdutil".into(), reason: "it ran too long".into() };
            let cancelled = CommandRunError::Cancelled { program: "/usr/bin/mdutil".into() };
            for runner in [
                indexed().answering(work, "/Volumes/Work:\n\tIndexing disabled.\n"),
                indexed().answering(work, "/Volumes/Work:\n\tError: unknown indexing state.\n"),
                indexed().exiting(work, 1),
                indexed().failing(work, failed()),
                indexed().failing(work, cancelled),
                indexed(),
            ] {
                let mut volumes = system_volumes();
                volumes.push(volume("/Volumes/Work", true, true));

                assert_eq!(check(runner, Ok(volumes)), error("a volume is not indexed"));
            }
        }

        #[test]
        fn a_volume_named_as_the_enabled_status_is_not_indexed_by_its_name() {
            let name = "/Volumes/Indexing enabled.";
            let runner =
                indexed().answering(&format!("/usr/bin/mdutil -s {name}"), &format!("{name}:\n\tIndexing disabled.\n"));
            let mut volumes = system_volumes();
            volumes.push(volume(name, true, true));

            assert_eq!(check(runner, Ok(volumes)), error("a volume is not indexed"));
        }

        #[test]
        fn a_volume_whose_status_command_fails_is_not_indexed_though_it_says_enabled() {
            let mounts = FakeMounts(Ok(system_volumes()));
            let search = SpotlightSearch::new(Arc::new(indexed()), Arc::new(FailingButEnabled), Arc::new(mounts));

            assert_eq!(search.search_complete(Path::new(HOME), &CancelToken::new()), error("a volume is not indexed"));
        }

        #[test]
        fn a_data_volume_that_is_not_indexed_is_asked_though_not_browsable() {
            let runner = indexed().answering(&format!("/usr/bin/mdutil -s {DATA}"), "Indexing disabled.\n");

            let (runner, search) = search(runner, Ok(system_volumes()));

            assert_eq!(search.search_complete(Path::new(HOME), &CancelToken::new()), error("a volume is not indexed"));
            assert!(runner.calls().contains(&format!("/usr/bin/mdutil -s {DATA}")), "{:?}", runner.calls());
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::CancelToken;

        use super::{DATA, FINDER, FakeMounts, HOME, Recording, check, indexed, search, system_volumes, volume};
        use crate::SpotlightSearch;

        #[test]
        fn every_local_volume_indexed_is_a_complete_search() {
            let mut volumes = system_volumes();
            volumes.push(volume("/Volumes/Work", true, true));
            let runner =
                indexed().answering("/usr/bin/mdutil -s /Volumes/Work", "/Volumes/Work:\n\tIndexing enabled. \n");

            let (runner, search) = search(runner, Ok(volumes));

            assert_eq!(search.search_complete(Path::new(HOME), &CancelToken::new()), Ok(()));
            assert_eq!(
                runner.calls(),
                [
                    FINDER,
                    "/usr/bin/mdutil -s /",
                    &format!("/usr/bin/mdutil -s {DATA}"),
                    "/usr/bin/mdutil -s /Volumes/Work"
                ]
            );
        }

        #[test]
        fn a_volume_the_system_does_not_show_is_skipped_but_the_homes() {
            let mut volumes = system_volumes();
            volumes.push(volume("/System/Volumes/Preboot", true, false));
            volumes.push(volume("/Volumes/Hidden Share", false, false));

            assert_eq!(check(indexed(), Ok(volumes)), Ok(()));
        }

        #[test]
        fn each_volume_is_asked_with_its_mount_point_as_one_argument_without_a_shell() {
            let finder = indexed();
            let index = Arc::new(Recording::default());
            let mounts =
                Ok(vec![volume("/", true, true), volume(DATA, true, false), volume("/Volumes/Work Disk", true, true)]);
            let search = SpotlightSearch::new(Arc::new(finder), Arc::clone(&index) as _, Arc::new(FakeMounts(mounts)));

            assert_eq!(search.search_complete(Path::new(HOME), &CancelToken::new()), Ok(()));

            let asked = index.0.lock().unwrap().clone();
            let expected: Vec<(String, Vec<String>)> = ["/", DATA, "/Volumes/Work Disk"]
                .iter()
                .map(|mount_point| ("/usr/bin/mdutil".to_owned(), vec!["-s".to_owned(), (*mount_point).to_owned()]))
                .collect();
            assert_eq!(asked, expected);
        }

        #[test]
        fn spotlight_paths_are_read_whole_from_its_nul_separated_answer() {
            let runner = indexed().answering(
                "/usr/bin/mdfind -0 kMDItemCFBundleIdentifier == 'com.example.App'c",
                "/Volumes/Work/two\nlines/App.app\0/Applications/App.app\0",
            );
            let (_, search) = search(runner, Ok(system_volumes()));

            assert_eq!(
                search.paths("com.example.App", &CancelToken::new()),
                Ok(vec!["/Volumes/Work/two\nlines/App.app".into(), "/Applications/App.app".into()])
            );
        }
    }
}
