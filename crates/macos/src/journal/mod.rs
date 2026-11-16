//! `MacChangeJournal`: the core's `ChangeJournal` over `FSEvents`.

mod changes;
mod stream;

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fresher_core::{CancelToken, ChangeJournal, Changes, JournalGap, JournalPosition, VolumeId, Volumes};

use crate::MacVolumes;
use crate::ffi::{fsevents, volume};
use crate::volume::{c_string, is_derived};
use stream::HistoryStream;

/// How long a replay may take before it is given up (a week of a developer's history took 470 s in 01).
const TIME_LIMIT: Duration = Duration::from_mins(10);

/// Reads the folders that changed on a volume from the system's change journal (`FSEvents`). No helper runs in the
/// background: each call replays the journal's history and releases its stream before it returns.
#[derive(Clone, Debug)]
pub struct MacChangeJournal {
    watched: Vec<PathBuf>,
    time_limit: Duration,
}

impl MacChangeJournal {
    /// Watches each volume whole, from its mount point.
    pub fn new() -> Self {
        Self { watched: Vec::new(), time_limit: TIME_LIMIT }
    }

    /// Watches only `paths` (and below), each on its own volume: changes elsewhere are not reported.
    pub fn under(paths: Vec<PathBuf>) -> Self {
        Self { watched: paths, time_limit: TIME_LIMIT }
    }

    /// Gives up a replay that takes longer than `limit` (`JournalGap::Failed`).
    #[must_use]
    pub const fn with_time_limit(mut self, limit: Duration) -> Self {
        self.time_limit = limit;
        self
    }

    /// The paths to replay for `volume`: the watched ones on it, or its mount point.
    fn paths_on(&self, volume: VolumeId, mount_point: &Path) -> Vec<PathBuf> {
        if self.watched.is_empty() {
            return vec![mount_point.to_path_buf()];
        }
        let volumes = MacVolumes::new();
        self.watched
            .iter()
            .filter(|path| volumes.identify(path).is_ok_and(|facts| facts.id == volume))
            .cloned()
            .collect()
    }
}

impl Default for MacChangeJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeJournal for MacChangeJournal {
    fn position(&self, volume: &VolumeId) -> Option<JournalPosition> {
        let mount = Mount::of(*volume)?;
        position(mount.device)
    }

    /// The end is the position taken when the call starts, so nothing that changes later is lost to the next call.
    fn changes_since(
        &self,
        volume: &VolumeId,
        since: &JournalPosition,
        cancel: &CancelToken,
    ) -> Result<Changes, JournalGap> {
        let mount = Mount::of(*volume).ok_or(JournalGap::NoJournal)?;
        let end = position(mount.device).ok_or(JournalGap::NoJournal)?;
        if end.journal != since.journal {
            return Err(JournalGap::Reset);
        }
        let paths = self.paths_on(*volume, &mount.point);
        if paths.is_empty() {
            return Ok(Changes { folders: Vec::new(), subtrees: Vec::new(), end });
        }
        let events = HistoryStream::start(&paths, since.cursor)?.collect(cancel, self.time_limit)?;
        changes::from_events(&events, end)
    }
}

fn position(device: libc::dev_t) -> Option<JournalPosition> {
    let journal = fsevents::journal_id(device)?;
    Some(JournalPosition { journal, cursor: fsevents::current_event_id() })
}

/// A mounted local volume: where it is mounted and its device.
struct Mount {
    point: PathBuf,
    device: libc::dev_t,
}

impl Mount {
    /// The local mount of `volume`; `None` for a volume with a made-up id or one that is not mounted.
    fn of(volume: VolumeId) -> Option<Self> {
        if is_derived(volume) {
            return None;
        }
        let volumes = MacVolumes::new();
        let mounts = volume::mounts().map_err(|error| tracing::warn!(%error, "the mounts are unknown")).ok()?;
        mounts
            .iter()
            .filter(|stats| stats.f_flags & libc::MNT_LOCAL.cast_unsigned() != 0)
            .filter_map(|stats| c_string(&stats.f_mntonname))
            .map(|point| PathBuf::from(OsStr::from_bytes(point.as_bytes())))
            .find_map(|point| {
                let facts = volumes.identify(&point).ok().filter(|facts| facts.id == volume)?;
                let device = libc::dev_t::try_from(facts.device.cast_signed()).ok()?;
                Some(Self { point, device })
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use fresher_core::{CancelToken, ChangeJournal, Changes, JournalPosition, VolumeId, Volumes};
    use tempfile::TempDir;

    use super::MacChangeJournal;
    use super::stream::live_streams;
    use crate::MacVolumes;

    /// A quiet folder to watch (`root`), inside a temporary folder of its own, so no other test's change is near.
    pub(super) struct Watched {
        _folder: TempDir,
        pub(super) root: PathBuf,
        pub(super) volume: VolumeId,
        pub(super) journal: MacChangeJournal,
    }

    impl Watched {
        pub(super) fn new() -> Self {
            let folder = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(folder.path()).unwrap().join("root");
            fs::create_dir(&root).unwrap();
            let volume = MacVolumes::new().identify(&root).unwrap().id;
            Self { _folder: folder, journal: MacChangeJournal::under(vec![root.clone()]), root, volume }
        }

        pub(super) fn changes_since(&self, since: &JournalPosition) -> Changes {
            let changes = self.journal.changes_since(&self.volume, since, &CancelToken::new()).unwrap();
            assert_eq!(live_streams(), 0);
            changes
        }

        /// The position once every change made so far is in the journal: the system numbers an event a little after
        /// the change, so the setup's own events are waited for before the test's position is taken.
        pub(super) fn settled(&self) -> JournalPosition {
            let before = self.journal.position(&self.volume).unwrap();
            fs::write(self.root.join(".settle"), b"").unwrap();
            let started = Instant::now();
            while !self.changes_since(&before).folders.contains(&self.root) {
                assert!(started.elapsed() < Duration::from_secs(10), "the journal never saw the setup");
            }
            self.journal.position(&self.volume).unwrap()
        }

        pub(super) fn path(&self, relative: &str) -> PathBuf {
            relative.split('/').fold(self.root.clone(), |path, name| path.join(name))
        }
    }

    pub(super) fn folders(changes: &Changes) -> Vec<&Path> {
        changes.folders.iter().map(PathBuf::as_path).collect()
    }

    mod negative {
        use std::fs;

        use fresher_core::{CancelToken, ChangeJournal, JournalGap, JournalPosition, VolumeId, Volumes};

        use super::{Watched, folders};
        use crate::disk_image::{DiskImage, Format};
        use crate::journal::stream::live_streams;
        use crate::{MacChangeJournal, MacVolumes};

        #[test]
        fn another_journal_id_is_a_reset() {
            let watched = Watched::new();
            let position = watched.journal.position(&watched.volume).unwrap();
            let elsewhere = JournalPosition { journal: position.journal ^ 1, ..position };

            let answer = watched.journal.changes_since(&watched.volume, &elsewhere, &CancelToken::new());

            assert_eq!(answer, Err(JournalGap::Reset));
        }

        #[test]
        fn a_volume_without_a_journal_has_no_position() {
            let Some(image) = DiskImage::mount(Format::Apfs) else {
                return;
            };
            let volume = MacVolumes::new().identify(image.mount_point()).unwrap().id;
            let journal = MacChangeJournal::new();
            let since = JournalPosition { journal: 1, cursor: 1 };

            assert_eq!(journal.position(&volume), None);
            assert_eq!(journal.changes_since(&volume, &since, &CancelToken::new()), Err(JournalGap::NoJournal));
        }

        #[test]
        fn a_volume_with_a_made_up_id_or_not_mounted_has_no_position() {
            let journal = MacChangeJournal::new();

            assert_eq!(journal.position(&VolumeId::new(0x6672_7368 << 96)), None);
            assert_eq!(journal.position(&VolumeId::new(42)), None);
        }

        #[test]
        fn a_cancelled_collect_returns_cancelled_and_releases_the_stream() {
            let watched = Watched::new();
            let since = watched.journal.position(&watched.volume).unwrap();
            fs::write(watched.path("new"), b"x").unwrap();
            let cancel = CancelToken::new();
            cancel.cancel();

            let answer = watched.journal.changes_since(&watched.volume, &since, &cancel);

            assert_eq!(answer, Err(JournalGap::Cancelled));
            assert_eq!(live_streams(), 0);
        }

        #[test]
        fn a_replay_past_its_time_limit_fails_and_releases_the_stream() {
            let watched = Watched::new();
            let journal =
                MacChangeJournal::under(vec![watched.root.clone()]).with_time_limit(std::time::Duration::ZERO);
            // From the oldest event: the history cannot be over before the first check.
            let since = JournalPosition { cursor: 1, ..journal.position(&watched.volume).unwrap() };

            let answer = journal.changes_since(&watched.volume, &since, &CancelToken::new());

            assert_eq!(answer, Err(JournalGap::Failed));
            assert_eq!(live_streams(), 0);
        }

        #[test]
        fn changes_outside_the_root_are_not_reported() {
            let watched = Watched::new();
            let sibling = watched.root.with_file_name("sibling");
            fs::create_dir(&sibling).unwrap();
            let since = watched.settled();

            fs::write(sibling.join("new"), b"x").unwrap();
            fs::write(watched.path("inside"), b"x").unwrap();

            let changes = watched.changes_since(&since);
            assert!(folders(&changes).contains(&watched.root.as_path()));
            assert!(changes.folders.iter().chain(&changes.subtrees).all(|folder| !folder.starts_with(&sibling)));
        }

        #[test]
        fn nothing_changed_reports_nothing() {
            let watched = Watched::new();
            fs::create_dir(watched.path("a")).unwrap();
            fs::write(watched.path("a/file"), b"x").unwrap();
            let since = watched.settled();

            let changes = watched.changes_since(&since);

            assert!(changes.folders.iter().all(|folder| !folder.starts_with(&watched.root)), "{changes:?}");
            assert_eq!(changes.subtrees, [] as [std::path::PathBuf; 0]);
        }
    }

    mod positive {
        use std::fs::{self, OpenOptions};
        use std::io::Write as _;
        use std::process::Command;

        use fresher_core::ChangeJournal as _;

        use super::{Watched, folders};

        #[test]
        fn a_new_file_reports_its_folder() {
            let watched = Watched::new();
            fs::create_dir(watched.path("a")).unwrap();
            let since = watched.settled();

            fs::write(watched.path("a/new"), b"x").unwrap();

            let changes = watched.changes_since(&since);
            assert!(folders(&changes).contains(&watched.path("a").as_path()), "{changes:?}");
            assert!(changes.end.cursor >= since.cursor);
        }

        #[test]
        fn a_file_growing_in_place_reports_its_folder() {
            let watched = Watched::new();
            fs::create_dir(watched.path("a")).unwrap();
            fs::write(watched.path("a/log"), b"first line\n").unwrap();
            let since = watched.settled();

            OpenOptions::new()
                .append(true)
                .open(watched.path("a/log"))
                .unwrap()
                .write_all(&vec![b'x'; 65_536])
                .unwrap();

            let changes = watched.changes_since(&since);
            assert!(folders(&changes).contains(&watched.path("a").as_path()), "{changes:?}");
        }

        #[test]
        fn a_removed_folder_reports_its_parent() {
            let watched = Watched::new();
            fs::create_dir_all(watched.path("a/gone")).unwrap();
            let since = watched.settled();

            let removed = Command::new("/bin/rmdir").arg(watched.path("a/gone")).status().unwrap();
            assert!(removed.success());

            let changes = watched.changes_since(&since);
            assert!(folders(&changes).contains(&watched.path("a").as_path()), "{changes:?}");
        }

        #[test]
        fn a_renamed_folder_reports_both_parents() {
            let watched = Watched::new();
            fs::create_dir_all(watched.path("a/moving")).unwrap();
            fs::create_dir(watched.path("b")).unwrap();
            let since = watched.settled();

            fs::rename(watched.path("a/moving"), watched.path("b/moving")).unwrap();

            let changes = watched.changes_since(&since);
            let found = folders(&changes);
            assert!(
                found.contains(&watched.path("a").as_path()) && found.contains(&watched.path("b").as_path()),
                "{changes:?}"
            );
        }

        #[test]
        fn the_whole_volume_is_watched_from_its_mount_point() {
            let watched = Watched::new();
            let journal = crate::MacChangeJournal::default();
            let since = watched.settled();

            fs::write(watched.path("new"), b"x").unwrap();

            // A whole busy volume may overflow the stream while other tests write: a gap, which means a full scan.
            match journal.changes_since(&watched.volume, &since, &fresher_core::CancelToken::new()) {
                Ok(changes) => assert!(changes.folders.contains(&watched.root), "{} folders", changes.folders.len()),
                Err(gap) => assert_eq!(gap, fresher_core::JournalGap::Dropped),
            }
        }
    }
}
