//! The Trash as the scan screen shows it: what waits in each Trash and how much of it this app's cleanups put there;
//! opening one in Finder. Reads only: nothing here empties a Trash.

use std::path::{Path, PathBuf};

use fresher_core::{
    AutoEmpty, ByteSize, CleanupsQuery, CommandError, DiskState, KnownFolder, RecordedCleanup, Sizes, Timestamp,
    TrashLine, TrashView, Volumes,
};

use crate::cleanup::history::Located;
use crate::engine::{CleanupPorts, Shared};
use crate::error::{EngineError, lock_anyway};

/// How long Finder keeps an item in the Trash with its 30-day setting on, in seconds.
const FINDER_KEEPS_FOR: u64 = 30 * 24 * 60 * 60;
/// The folder at a volume's root that holds each user's Trash of that volume, by user id.
const VOLUME_TRASHES: &str = ".Trashes";

/// Each Trash that holds something, the home's first, from the last scan's measure and the journal's items found in
/// the Trash again now; its folders replace those `open` opens.
pub(crate) fn view(shared: &Shared) -> TrashView {
    let state = lock_anyway(&shared.results).state();
    let home = shared.ports.system.known_folders.locate(KnownFolder::Trash);
    let trashes = shared.cleanup.as_ref().map(|ports| ports.trashes.clone()).unwrap_or_default();
    let mut tally = Tally { home: home.clone(), trashes, ..Tally::default() };
    if let Some(home) = &home {
        tally.folder(home);
    }
    if let Some(ports) = &shared.cleanup {
        tally.read(ports, state.as_ref().map(|state| state.taken()));
    }
    let mut folders: Vec<Folder> = tally
        .folders
        .into_iter()
        .map(|folder| folder.measured(state.as_deref()))
        .filter(|folder| !folder.is_empty())
        .collect();
    folders.sort_by(|a, b| (Some(&a.path) != home.as_ref(), &a.path).cmp(&(Some(&b.path) != home.as_ref(), &b.path)));
    let auto_empty_now = auto_empty_now(shared, &folders);
    let trashes = (0_u32..)
        .zip(&folders)
        .map(|(index, folder)| TrashLine {
            index,
            volume: shared
                .cleanup
                .as_ref()
                .and_then(|ports| ports.volumes.space(&folder.path).ok())
                .map(|space| space.name),
            waiting: folder.waiting,
            from_cleanups: folder.share,
            gone_by: auto_empty_now.and_then(|now| folder.gone_by(now)),
        })
        .collect();
    *lock_anyway(&shared.trash_folders) = folders.into_iter().map(|folder| folder.path).collect();
    TrashView { trashes }
}

/// Opens Trash `index` of the last `view` in Finder; `UnknownTrash` for an index it did not have.
pub(crate) fn open(shared: &Shared, index: u32) -> Result<(), CommandError> {
    let folder = usize::try_from(index).ok().and_then(|at| lock_anyway(&shared.trash_folders).get(at).cloned());
    let folder = folder.ok_or(CommandError::UnknownTrash { index })?;
    shared.ports.system_ui.open_folder(&folder).map_err(|error| EngineError::SystemUi(error).into())
}

/// The clock's now when Finder empties the Trash by itself after 30 days; `None` when it does not, or cannot say.
/// Finder is asked only when some folder holds something from cleanups.
fn auto_empty_now(shared: &Shared, folders: &[Folder]) -> Option<Timestamp> {
    let ports = shared.cleanup.as_ref()?;
    let some_share = folders.iter().any(|folder| folder.share.allocated > ByteSize::ZERO);
    (some_share && ports.finder.trash_auto_empty() == AutoEmpty::After30Days)
        .then(|| Timestamp::from_system_time(shared.ports.clock.now()))
}

/// The Trash folders found so far, with what cleanups put in each.
#[derive(Default)]
struct Tally {
    /// The home's Trash, as the known folders spell it.
    home: Option<PathBuf>,
    /// The other Trash folders the composition names.
    trashes: Vec<PathBuf>,
    folders: Vec<Folder>,
    /// The folders asked about, and whether each is a Trash.
    checked: Vec<(PathBuf, bool)>,
}

struct Folder {
    path: PathBuf,
    /// The items of cleanups still there.
    share: Sizes,
    /// The part of `share` from cleanups that began at or after the scan: what its measure could not see.
    since_scan: Sizes,
    /// The latest moment an item of `share` went.
    latest: Option<Timestamp>,
    /// Everything in the folder; filled by [`Folder::measured`].
    waiting: Option<Sizes>,
}

impl Tally {
    fn folder(&mut self, path: &Path) -> &mut Folder {
        let at = self.folders.iter().position(|folder| folder.path == path).unwrap_or_else(|| {
            let path = path.to_path_buf();
            self.folders.push(Folder {
                path,
                share: Sizes::ZERO,
                since_scan: Sizes::ZERO,
                latest: None,
                waiting: None,
            });
            self.folders.len() - 1
        });
        &mut self.folders[at]
    }

    /// Every cleanup of the journal, page by page; a journal error stops the reading, what was read counts.
    fn read(&mut self, ports: &CleanupPorts, taken: Option<Timestamp>) {
        let mut offset = 0_u32;
        loop {
            let page = match ports.journal.cleanups(offset, CleanupsQuery::MAX_LIMIT) {
                Ok(page) => page,
                Err(error) => {
                    tracing::warn!(%error, "the Trash view could not read the history of cleanups");
                    return;
                }
            };
            for summary in &page.cleanups {
                match ports.journal.cleanup(summary.cleanup) {
                    Ok(Some(recorded)) => self.cleanup(ports, &recorded, taken),
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(%error, "the Trash view could not read a cleanup");
                        return;
                    }
                }
            }
            offset = offset.saturating_add(u32::try_from(page.cleanups.len()).unwrap_or(u32::MAX));
            if page.cleanups.is_empty() || offset >= page.total {
                return;
            }
        }
    }

    /// Whether `folder` is a Trash: the home's, one the composition names, or a volume's `.Trashes/<uid>` at the root
    /// of that volume. An entry elsewhere (a move the system reported outside any Trash) is no Trash's.
    fn is_trash(&mut self, volumes: &dyn Volumes, folder: &Path) -> bool {
        if self.home.as_deref() == Some(folder) || self.trashes.iter().any(|trash| trash == folder) {
            return true;
        }
        if let Some(&(_, known)) = self.checked.iter().find(|(path, _)| path == folder) {
            return known;
        }
        let uid = folder.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.parse::<u32>().is_ok());
        let trash = uid
            && volumes
                .space(folder)
                .is_ok_and(|space| folder.parent() == Some(space.mount_point.join(VOLUME_TRASHES).as_path()));
        self.checked.push((folder.to_path_buf(), trash));
        trash
    }

    fn cleanup(&mut self, ports: &CleanupPorts, recorded: &RecordedCleanup, taken: Option<Timestamp>) {
        let went = recorded.end.as_ref().map_or(recorded.begin.at, |end| end.at);
        let since_scan = taken.is_some_and(|taken| recorded.begin.at >= taken);
        let located = Located::of(ports, recorded);
        for (_, sizes, entry) in located.waiting_items() {
            let Some(path) = entry.landed.parent() else { continue };
            if !self.is_trash(&*ports.volumes, path) {
                continue;
            }
            let folder = self.folder(path);
            folder.share = sum(folder.share, sizes);
            if since_scan {
                folder.since_scan = sum(folder.since_scan, sizes);
            }
            folder.latest = folder.latest.max(Some(went));
        }
    }
}

impl Folder {
    /// The folder with `waiting`: the scan's measure of it, when the scan read it, plus what cleanups moved there
    /// since; never below `share`.
    fn measured(mut self, state: Option<&dyn DiskState>) -> Self {
        let scanned = state.and_then(|state| {
            let node = state.node(state.find(&self.path)?)?;
            node.not_scanned.is_none().then_some(node.sizes)
        });
        self.waiting = scanned.map(|sizes| {
            let waiting = sum(sizes, self.since_scan);
            if waiting.allocated < self.share.allocated { self.share } else { waiting }
        });
        self
    }

    /// When the last item of `share` leaves the Trash by itself, Finder keeping each 30 days; only a date after `now`.
    fn gone_by(&self, now: Timestamp) -> Option<Timestamp> {
        if self.share.allocated == ByteSize::ZERO {
            return None;
        }
        let gone = Timestamp::from_secs(self.latest?.secs().saturating_add(FINDER_KEEPS_FOR));
        (gone > now).then_some(gone)
    }

    fn is_empty(&self) -> bool {
        self.waiting.is_none_or(|waiting| waiting.allocated == ByteSize::ZERO) && self.share.allocated == ByteSize::ZERO
    }
}

/// `a` and `b` summed, each count saturating instead of overflowing.
fn sum(a: Sizes, b: Sizes) -> Sizes {
    a.checked_add(b).unwrap_or_else(|| Sizes {
        logical: ByteSize::new(a.logical.bytes().saturating_add(b.logical.bytes())),
        allocated: ByteSize::new(a.allocated.bytes().saturating_add(b.allocated.bytes())),
        reclaimable: ByteSize::new(a.reclaimable.bytes().saturating_add(b.reclaimable.bytes())),
        completeness: a.completeness.max(b.completeness),
    })
}
