use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Read};
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    ByteSize, CACHEDIR_SIGNATURE, CACHEDIR_TAG, CancelToken, Claim, ClaimedPath, Completeness, DiskState, DiskWalk,
    EntryKind, Inside, Interest, KnownFolder, KnownFolders, MarkerCheck, Match, NodeKind, NotScanned, Pattern, Sizes,
    StateId, Subtree, SubtreeQuery, Timestamp, Unreadable, WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
    is_bundle_id_shaped, is_valid_cachedir_tag,
};

use crate::entry_facts::{Seen, disk_kind, entry_facts};
use crate::fakes::lock;
use crate::naive_places::{Listing, Listings};
use crate::{FakeKnownFolders, FixedDiskState, StatedFacts};

/// A size nothing is known of: a folder not read.
const NOT_READ: Sizes = Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };

/// The reference [`DiskWalk`]: simple and slow, with `std::fs` only. Links are listed, never followed; the walk never
/// leaves the device of a root and never opens an avoided folder. Every size is complete and private (`std::fs`
/// cannot see clones). Entries are visited in name order, so its matches come in a stable order. The real scanner is
/// tested against it: the same fixture gives the same matches.
///
/// `Full` and `Auto` both walk the disk (always a full scan); there is no cache, so `Restore` has nothing to restore.
/// `commit` returns a [`FixedDiskState`] of the folders of the last walk, with their sizes; files are not kept.
/// The request's places are walked after the roots (one inside a root, or holding one, is not walked; one that cannot
/// be read is not scanned): only anchored interests match there and a folder of another user is not entered.
/// `<UserTemp>/TemporaryItems` is never opened.
/// Each match carries the facts of its entry, read for the owner of the home folder (see `entry_facts.rs`); a fact the
/// tree states ([`NaiveWalk::with_stated`]) is read instead of the disk's, as `FactsLens` reports it to the scanner.
/// The summary's `places` says what it saw of each anchored interest's place, from the folders it listed.
/// `subtrees` answers from the folders the last walk recorded: their own times and whether each was listed in full.
#[derive(Debug)]
pub struct NaiveWalk {
    home: PathBuf,
    folders: FakeKnownFolders,
    stated: StatedFacts,
    walked: Mutex<Walked>,
}

/// What the last walk saw, for `commit` and `subtrees`.
#[derive(Debug, Default)]
struct Walked {
    folders: Vec<WalkedFolder>,
    not_scanned: Vec<(PathBuf, NotScanned)>,
    commits: u32,
}

/// A folder the walk recorded; none inside a folder claimed whole.
#[derive(Debug)]
struct WalkedFolder {
    path: PathBuf,
    sizes: Sizes,
    /// Its own modification time; a scan root has none.
    modified: Option<Timestamp>,
    /// Listed in full: read with every entry, not avoided, not on another volume, not another user's.
    listed: bool,
    /// Claimed whole: nothing inside it is recorded.
    whole: bool,
}

impl Walked {
    fn record(&mut self, path: &Path, sizes: Sizes, modified: Option<Timestamp>, listed: bool) {
        self.folders.push(WalkedFolder { path: path.to_path_buf(), sizes, modified, listed, whole: false });
    }

    /// Records a folder claimed whole, listed in full when its sizes are complete.
    fn record_whole(&mut self, path: &Path, sizes: Sizes, modified: Option<Timestamp>) {
        let listed = sizes.completeness == Completeness::Complete;
        self.folders.push(WalkedFolder { path: path.to_path_buf(), sizes, modified, listed, whole: true });
    }

    /// The recorded folder of `query`, and the newest time and completeness of the recorded folders at or below it
    /// and below no skip under it.
    fn subtree(&self, query: &SubtreeQuery) -> Option<Subtree> {
        let folder = self.folders.iter().find(|folder| folder.path == query.folder)?;
        let skips: Vec<&PathBuf> =
            query.skip.iter().filter(|skip| skip.starts_with(&query.folder) && **skip != query.folder).collect();
        let counted: Vec<&WalkedFolder> = self
            .folders
            .iter()
            .filter(|below| below.path.starts_with(&query.folder))
            .filter(|below| !skips.iter().any(|skip| below.path.starts_with(skip)))
            .collect();
        Some(Subtree {
            sizes: folder.sizes,
            newest: counted.iter().filter_map(|below| below.modified).max(),
            // A folder claimed whole below the queried one: what changed inside it is not known.
            complete: counted.iter().all(|below| below.listed && !(below.whole && below.path != query.folder)),
        })
    }
}

impl NaiveWalk {
    /// A walk for which `home` is the home folder, its known folders laid out under it as macOS lays them out
    /// (`FakeKnownFolders::home_layout`), that `Pattern::Under` paths are relative to: in a test, the
    /// root of the fixture tree.
    pub fn new(home: impl Into<PathBuf>) -> Self {
        let home = home.into();
        Self {
            folders: FakeKnownFolders::home_layout(&home),
            home,
            stated: StatedFacts::new(),
            walked: Mutex::default(),
        }
    }

    /// This walk, resolving known folders with `folders` instead of the home layout.
    #[must_use]
    pub fn with_known_folders(mut self, folders: FakeKnownFolders) -> Self {
        self.folders = folders;
        self
    }

    /// This walk, reading the facts `stated` (a fixture tree's) instead of the disk's.
    #[must_use]
    pub fn with_stated(mut self, stated: StatedFacts) -> Self {
        self.stated = stated;
        self
    }
}

impl DiskWalk for NaiveWalk {
    fn walk(
        &self,
        request: &WalkRequest<'_>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        if request.mode == WalkMode::Restore {
            return Err(WalkError::NothingToRestore);
        }
        let temporary_items = self.folders.locate(KnownFolder::UserTemp).map(|temp| temp.join("TemporaryItems"));
        let mut walker = Walker {
            folders: &self.folders,
            stated: &self.stated,
            user: fs::metadata(&self.home).ok().map(|home| home.uid()),
            request,
            avoid: request.avoid.iter().cloned().chain(temporary_items.clone()).collect(),
            system: temporary_items,
            in_place: false,
            cancel,
            on_match,
            on_progress,
            summary: WalkSummary::default(),
            walked: Walked::default(),
            listings: Listings::default(),
        };
        let mut progress = WalkProgress::default();
        for (index, root) in request.roots.iter().enumerate() {
            let unreadable = |kind| WalkError::RootUnreadable { index, kind };
            let metadata = fs::symlink_metadata(root).map_err(|error| unreadable(error.kind()))?;
            if !metadata.is_dir() {
                return Err(unreadable(io::ErrorKind::NotADirectory));
            }
            walker.listings.root(root);
            if walker.avoid.contains(root) {
                walker.not_scanned(root, NotScanned::Avoided, NOT_READ, None);
                continue;
            }
            walker.folder(root, metadata.dev(), None, &mut progress)?;
        }
        walker.in_place = true;
        for place in request.places {
            let nested = |root: &PathBuf| place.path.starts_with(root) || root.starts_with(&place.path);
            if request.roots.iter().any(nested) {
                continue;
            }
            // A place that cannot be read is not scanned, with why; it never stops the walk nor makes it incomplete.
            let metadata = fs::symlink_metadata(&place.path).ok().filter(Metadata::is_dir);
            match (metadata, fs::read_dir(&place.path)) {
                (Some(metadata), Ok(_)) => {
                    walker.listings.root(&place.path);
                    walker.folder(&place.path, metadata.dev(), None, &mut progress)?;
                }
                (metadata, Err(error)) => {
                    // A folder there that cannot be listed is walked as a root and seen unreadable.
                    if metadata.is_some() {
                        walker.listings.root(&place.path);
                        walker.listings.saw(&place.path, Listing::Unreadable);
                    }
                    let reason = NotScanned::Unreadable(unreadable(&error));
                    *walker.summary.not_scanned.entry(reason).or_default() += 1;
                }
                (None, Ok(_)) => {
                    *walker.summary.not_scanned.entry(NotScanned::Unreadable(Unreadable::Failed)).or_default() += 1;
                }
            }
        }
        let Walker { mut summary, walked, listings, .. } = walker;
        summary.places = listings.places(request.interests, &self.folders);
        let mut kept = lock(&self.walked);
        kept.folders = walked.folders;
        kept.not_scanned = walked.not_scanned;
        Ok(summary)
    }

    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        let mut walked = lock(&self.walked);
        walked.commits = walked.commits.saturating_add(1);
        let folders = walked.folders.iter().map(|folder| (folder.path.clone(), NodeKind::Folder, folder.sizes));
        // The reference walk has no clock: every state is taken at the epoch.
        let state = FixedDiskState::new(StateId::new(walked.commits), Timestamp::from_secs(0), folders);
        let state =
            walked.not_scanned.iter().fold(state, |state, (path, reason)| state.with_not_scanned(path, *reason));
        let state = claims.iter().fold(state, |state, claim| state.with_owner(&claim.path, claim.category));
        Ok(Arc::new(state))
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        let walked = lock(&self.walked);
        queries.iter().map(|query| walked.subtree(query)).collect()
    }
}

struct Walker<'a> {
    folders: &'a FakeKnownFolders,
    stated: &'a StatedFacts,
    /// The owner of the home folder: the user the facts of an entry are read for.
    user: Option<u32>,
    request: &'a WalkRequest<'a>,
    /// The request's avoided places and the system's own items in the user's temporary folder.
    avoid: Vec<PathBuf>,
    /// The system's own items (`<UserTemp>/TemporaryItems`), not opened for their own reason.
    system: Option<PathBuf>,
    /// Walking a place outside the scan roots: only anchored interests match, another user's folder is not entered.
    in_place: bool,
    cancel: &'a CancelToken,
    on_match: &'a mut dyn FnMut(Match),
    on_progress: &'a mut dyn FnMut(WalkProgress),
    summary: WalkSummary,
    walked: Walked,
    /// What the walk saw of every folder, for the places of the anchored interests.
    listings: Listings,
}

impl Walker<'_> {
    /// Walks the folder `dir`, whose own modification time is `modified`, and gives everything inside it.
    fn folder(
        &mut self,
        dir: &Path,
        device: u64,
        modified: Option<Timestamp>,
        progress: &mut WalkProgress,
    ) -> Result<Sizes, WalkError> {
        let entries = match sorted_entries(dir) {
            Ok(entries) => entries,
            Err(error) => {
                let reason = unreadable(&error);
                *self.summary.unreadable.entry(reason).or_default() += 1;
                self.walked.not_scanned.push((dir.to_path_buf(), NotScanned::Unreadable(reason)));
                self.walked.record(dir, NOT_READ, modified, false);
                self.listings.saw(dir, Listing::Unreadable);
                return Ok(NOT_READ);
            }
        };
        let mut total = Sizes::ZERO;
        let mut complete = true;
        for path in entries {
            if self.cancel.is_cancelled() {
                return Err(WalkError::Cancelled);
            }
            let Ok(seen) = Seen::read(&path, self.stated) else {
                *self.summary.unreadable.entry(Unreadable::Failed).or_default() += 1;
                total = add(total, NOT_READ);
                complete = false;
                continue;
            };
            let metadata = seen.metadata();
            if metadata.dev() != device {
                if metadata.is_dir() {
                    self.not_scanned(&path, NotScanned::OtherVolume, Sizes::ZERO, seen.modified());
                }
                continue;
            }
            let kind = seen.kind();
            self.summary.entries += 1;
            progress.entries += 1;
            progress.bytes = progress.bytes.checked_add(own_sizes(metadata, kind).logical).unwrap_or(progress.bytes);
            (self.on_progress)(*progress);
            let inner = if kind == EntryKind::Folder && self.system.as_ref() == Some(&path) {
                self.not_scanned(&path, NotScanned::SystemItems, NOT_READ, seen.modified());
                NOT_READ
            } else if kind == EntryKind::Folder && self.avoid.contains(&path) {
                self.not_scanned(&path, NotScanned::Avoided, NOT_READ, seen.modified());
                NOT_READ
            } else if kind == EntryKind::Folder && self.in_place && self.user != Some(seen.owner()) {
                self.others_folder(&path, &seen);
                NOT_READ
            } else if kind == EntryKind::Folder {
                if self.report(&path, &seen, kind, device) {
                    let sizes = tree_sizes(&path, device, &self.avoid);
                    self.listings.saw(&path, Listing::Whole);
                    self.walked.record_whole(&path, sizes, seen.modified());
                    sizes
                } else {
                    self.folder(&path, device, seen.modified(), progress)?
                }
            } else {
                self.report(&path, &seen, kind, device);
                own_sizes(seen.metadata(), kind)
            };
            total = add(total, inner);
        }
        self.walked.record(dir, total, modified, complete);
        self.listings.saw(dir, if complete { Listing::Read } else { Listing::Incomplete });
        Ok(total)
    }

    /// Records the folder `path` as not entered on purpose, counted in the summary.
    fn not_scanned(&mut self, path: &Path, reason: NotScanned, sizes: Sizes, modified: Option<Timestamp>) {
        *self.summary.not_scanned.entry(reason).or_default() += 1;
        self.walked.not_scanned.push((path.to_path_buf(), reason));
        self.walked.record(path, sizes, modified, false);
        self.listings.saw(path, Listing::NotEntered);
    }

    /// A folder of another user in a place: not entered, reported as itself by the interests that match it, its
    /// sizes not known and its inside foreign and incomplete.
    fn others_folder(&mut self, path: &Path, seen: &Seen<'_>) {
        self.not_scanned(path, NotScanned::OthersFolder, NOT_READ, seen.modified());
        let kind = EntryKind::Folder;
        for (index, interest) in self.request.interests.iter() {
            if !self.matches(interest, path, kind) {
                continue;
            }
            let whole = interest.claim() == Claim::Whole;
            let mut inside = Inside { foreign: true, complete: false, ..Inside::EMPTY };
            inside.newest_change = seen.modified().max(seen.changed());
            (self.on_match)(Match {
                interest: index,
                path: path.to_path_buf(),
                kind,
                sizes: if whole { NOT_READ } else { Sizes::ZERO },
                modified: seen.modified(),
                changed: seen.changed(),
                accessed: None,
                inside: whole.then_some(inside),
                siblings: present_siblings(interest, path),
                holds: Vec::new(),
                marker: None,
                facts: entry_facts(path, seen, kind, self.user, self.stated),
            });
        }
    }

    /// Whether `interest` matches `path`: in a place, only an anchored one does.
    fn matches(&self, interest: &Interest, path: &Path, kind: EntryKind) -> bool {
        (!self.in_place || matches!(interest.pattern(), Pattern::Under(..)))
            && matches(interest, self.folders, path, kind)
    }

    /// Reports every interest `path` matches; `true` when one of them claims it whole.
    fn report(&mut self, path: &Path, seen: &Seen<'_>, kind: EntryKind, device: u64) -> bool {
        let mut claimed_whole = false;
        for (index, interest) in self.request.interests.iter() {
            if !self.matches(interest, path, kind) {
                continue;
            }
            let whole = interest.claim() == Claim::Whole;
            claimed_whole |= whole;
            let whole_folder = whole && kind == EntryKind::Folder;
            let sizes =
                if whole_folder { tree_sizes(path, device, &self.avoid) } else { own_sizes(seen.metadata(), kind) };
            let inside = whole_folder.then(|| self.tree_inside(path, seen, device));
            let (holds, held_tag) = self.held(interest, path, kind);
            (self.on_match)(Match {
                interest: index,
                path: path.to_path_buf(),
                kind,
                sizes,
                modified: seen.modified(),
                changed: seen.changed(),
                accessed: (kind != EntryKind::Folder).then(|| seen.accessed()).flatten(),
                inside,
                siblings: present_siblings(interest, path),
                holds,
                marker: marker_check(interest.pattern(), path).or(held_tag),
                facts: entry_facts(path, seen, kind, self.user, self.stated),
            });
        }
        claimed_whole
    }

    /// Which of the interest's held names the folder `path` lists, in the interest's order, and, when they name a
    /// `CACHEDIR.TAG`, whether a valid one is there. An entry that cannot be read is absent; any kind counts, but a
    /// tag only as a regular file. Nothing, and no check, for an entry that is not a folder or a folder that cannot
    /// be listed.
    fn held(&self, interest: &Interest, path: &Path, kind: EntryKind) -> (Vec<String>, Option<MarkerCheck>) {
        if interest.holds().is_empty() || kind != EntryKind::Folder {
            return (Vec::new(), None);
        }
        let Ok(entries) = sorted_entries(path) else { return (Vec::new(), None) };
        let holds: Vec<String> = interest
            .holds()
            .iter()
            .filter(|name| {
                entries.iter().any(|entry| {
                    entry.file_name().is_some_and(|found| found == name.as_str())
                        && Seen::read(entry, self.stated)
                            .is_ok_and(|seen| name.as_str() != CACHEDIR_TAG || seen.kind() == EntryKind::File)
                })
            })
            .cloned()
            .collect();
        let tag = interest.holds().iter().any(|name| name == CACHEDIR_TAG).then(|| {
            let held = holds.iter().any(|name| name == CACHEDIR_TAG);
            if held && holds_valid_tag(path) { MarkerCheck::Valid } else { MarkerCheck::Invalid }
        });
        (holds, tag)
    }

    /// What the folder `dir` (seen as `seen`) holds, for the walk's user: the newest modification or status change of
    /// the folder and of everything inside it, the newest access of a file inside, whether anything inside is a
    /// socket, a pipe or a device, or not the user's or locked, and whether every folder inside was read (one on
    /// another device or avoided is not).
    fn tree_inside(&self, dir: &Path, seen: &Seen<'_>, device: u64) -> Inside {
        let mut inside = Inside::EMPTY;
        inside.newest_change = seen.modified().max(seen.changed());
        self.gather_inside(dir, device, &mut inside);
        inside
    }

    fn gather_inside(&self, dir: &Path, device: u64, inside: &mut Inside) {
        let Ok(entries) = sorted_entries(dir) else {
            inside.complete = false;
            return;
        };
        for path in entries {
            if let Some(name) = path.file_name() {
                inside.join_name(name.as_bytes());
            }
            let Ok(seen) = Seen::read(&path, self.stated) else {
                inside.complete = false;
                continue;
            };
            let kind = seen.kind();
            inside.newest_change = inside.newest_change.max(seen.modified()).max(seen.changed());
            if kind == EntryKind::File {
                inside.newest_file_access = inside.newest_file_access.max(seen.accessed());
            }
            inside.special |= kind == EntryKind::Other;
            let (user_lock, system_lock, restricted) = seen.flags();
            inside.foreign |= self.user != Some(seen.owner()) || user_lock || system_lock || restricted;
            if kind == EntryKind::Folder {
                if seen.metadata().dev() != device || self.avoid.contains(&path) {
                    inside.complete = false;
                } else {
                    self.gather_inside(&path, device, inside);
                }
            }
        }
    }
}

/// Adds `more` to `total`; a sum that does not fit leaves `total` as it was, `Partial`.
fn add(total: Sizes, more: Sizes) -> Sizes {
    total.checked_add(more).unwrap_or(Sizes { completeness: Completeness::Partial, ..total })
}

/// The entries of `dir`, sorted by name.
fn sorted_entries(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries =
        fs::read_dir(dir)?.map(|entry| entry.map(|entry| entry.path())).collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn unreadable(error: &io::Error) -> Unreadable {
    if error.kind() == io::ErrorKind::PermissionDenied { Unreadable::NoPermission } else { Unreadable::Failed }
}

fn matches(interest: &Interest, folders: &FakeKnownFolders, path: &Path, kind: EntryKind) -> bool {
    let name = path.file_name().and_then(|name| name.to_str());
    match interest.pattern() {
        Pattern::FolderName(folder) => kind != EntryKind::File && name == Some(folder.as_str()),
        Pattern::MarkerFile(marker) => {
            kind == EntryKind::Folder && listed(path, marker).is_some_and(|marker| marker.is_file())
        }
        Pattern::Extension(extension) => {
            kind != EntryKind::Folder && path.extension().and_then(|found| found.to_str()) == Some(extension.as_str())
        }
        Pattern::Under(folder, pattern) => folders.locate(*folder).is_some_and(|anchor| {
            path.strip_prefix(anchor).is_ok_and(|relative| {
                let names: Vec<_> = relative.iter().filter_map(|name| name.to_str()).collect();
                let segments: Vec<_> = pattern.split('/').collect();
                let shaped = interest.matches_only_bundle_ids();
                let suffixes = interest.name_suffixes();
                names.len() == relative.iter().count()
                    && names.len() == segments.len()
                    && names.iter().zip(&segments).all(|(name, segment)| *segment == "*" || name == segment)
                    && (!shaped || names.last().is_some_and(|last| is_bundle_id_shaped(last.as_bytes())))
                    && (suffixes.is_empty() || names.last().is_some_and(|last| has_suffix(last, suffixes)))
            })
        }),
    }
}

/// Whether `name` ends with one of `suffixes` (lowercase), ASCII case ignored, and is longer than it.
fn has_suffix(name: &str, suffixes: &[String]) -> bool {
    let name = name.to_ascii_lowercase();
    suffixes.iter().any(|suffix| name.len() > suffix.len() && name.ends_with(suffix.as_str()))
}

/// The size of the entry itself: a file's or a socket's, pipe's or device's bytes; nothing for a folder or a link.
fn own_sizes(metadata: &Metadata, kind: EntryKind) -> Sizes {
    if matches!(kind, EntryKind::File | EntryKind::Other) {
        Sizes::complete(ByteSize::new(metadata.len()), ByteSize::new(metadata.blocks() * 512))
    } else {
        Sizes::ZERO
    }
}

/// The metadata of the entry named exactly `name` in the folder `dir`, as the folder lists it: on a volume that
/// ignores case, `cachedir.tag` is not `CACHEDIR.TAG`. `None` when there is none or it cannot be read.
fn listed(dir: &Path, name: &str) -> Option<Metadata> {
    let entry = fs::read_dir(dir).ok()?.filter_map(Result::ok).find(|entry| entry.file_name() == name)?;
    fs::symlink_metadata(entry.path()).ok()
}

/// Everything inside the folder `dir` on `device`, links not followed, avoided folders not entered; `Partial` when a
/// folder could not be read or was avoided.
fn tree_sizes(dir: &Path, device: u64, avoid: &[PathBuf]) -> Sizes {
    let Ok(entries) = sorted_entries(dir) else {
        return NOT_READ;
    };
    entries.iter().fold(Sizes::ZERO, |total, path| {
        let inner = match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.dev() != device => Sizes::ZERO,
            Ok(metadata) if metadata.is_dir() && avoid.contains(path) => NOT_READ,
            Ok(metadata) if metadata.is_dir() => tree_sizes(path, device, avoid),
            Ok(metadata) => own_sizes(&metadata, disk_kind(&metadata)),
            Err(_) => NOT_READ,
        };
        add(total, inner)
    })
}

/// For a `CACHEDIR.TAG` marker, whether the tag in `folder` is valid; `None` for every other pattern.
fn marker_check(pattern: &Pattern, folder: &Path) -> Option<MarkerCheck> {
    match pattern {
        Pattern::MarkerFile(marker) if marker == CACHEDIR_TAG => {
            Some(if holds_valid_tag(folder) { MarkerCheck::Valid } else { MarkerCheck::Invalid })
        }
        _ => None,
    }
}

/// `true` when `folder` holds a regular file `CACHEDIR.TAG` (not a link) that starts with the signature. Reads the
/// signature's length and no more; any error means "not valid".
///
/// The file is opened once and checked through its handle: `O_NOFOLLOW` refuses a link, `O_NONBLOCK` keeps a FIFO or
/// a device from blocking the open, and the handle's own metadata must say "regular file". Nothing can be swapped in
/// between a check and the read.
fn holds_valid_tag(folder: &Path) -> bool {
    let opened =
        OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(folder.join(CACHEDIR_TAG));
    let Ok(file) = opened else { return false };
    if !file.metadata().is_ok_and(|metadata| metadata.is_file()) {
        return false;
    }
    let mut head = Vec::with_capacity(CACHEDIR_SIGNATURE.len());
    file.take(CACHEDIR_SIGNATURE.len() as u64).read_to_end(&mut head).is_ok() && is_valid_cachedir_tag(&head)
}

/// The interest's sibling names that the folder of `path` lists, links included, compared exactly.
fn present_siblings(interest: &Interest, path: &Path) -> Vec<String> {
    let Some(parent) = path.parent() else { return Vec::new() };
    interest.siblings().iter().filter(|name| listed(parent, name).is_some()).cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{
        CancelToken, Claim, DiskWalk, Interest, InterestSet, Match, ModuleId, Pattern, WalkError, WalkMode, WalkRequest,
    };

    use crate::{FixtureTree, NaiveWalk};

    fn interest(pattern: Pattern, claim: Claim, siblings: &[&str]) -> Interest {
        Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
    }

    /// Walks `tree` for `interests` and returns the matches with paths relative to the root.
    /// A full walk of `roots` for `interests`, avoiding nothing.
    fn request<'a>(roots: &'a [PathBuf], interests: &'a InterestSet) -> WalkRequest<'a> {
        WalkRequest { roots, interests, avoid: &[], places: &[], mode: WalkMode::Full }
    }

    fn walk(tree: &FixtureTree, interests: &[Interest]) -> Result<Vec<(String, Match)>, WalkError> {
        let mut set = InterestSet::new();
        set.add(ModuleId::new("demo").unwrap(), interests);
        let mut matches = Vec::new();
        let roots = [tree.root().to_path_buf()];
        NaiveWalk::new(tree.root()).walk(
            &request(&roots, &set),
            &CancelToken::new(),
            &mut |found| matches.push(found),
            &mut |_| {},
        )?;
        Ok(matches
            .into_iter()
            .map(|found| (found.path.strip_prefix(tree.root()).unwrap().to_string_lossy().into_owned(), found))
            .collect())
    }

    /// A home that states its facts: a folder `cache` 30 days old in full (a file and a socket inside), a folder
    /// `theirs` holding a file of the system and a lock file, and top-level `.log` files: one changed 30 days ago, a
    /// pipe, one of the system and one that states nothing.
    const STATED: &str = "[[dir]]\npath = \"cache\"\nage = \"30d\"\nchanged = \"30d\"\n\
        [[file]]\npath = \"cache/a\"\nage = \"30d\"\nchanged = \"30d\"\n\
        [[file]]\npath = \"cache/s\"\nkind = \"socket\"\nage = \"30d\"\nchanged = \"30d\"\n\
        [[file]]\npath = \"theirs/f\"\nowner = \"system\"\n\
        [[file]]\npath = \"theirs/server.pid\"\n\
        [[file]]\npath = \"old.log\"\nage = \"30d\"\nchanged = \"30d\"\naccessed = \"35d\"\n\
        [[file]]\npath = \"pipe.log\"\nkind = \"pipe\"\n\
        [[file]]\npath = \"root.log\"\nowner = \"system\"\nflags = [\"system-lock\"]\n\
        [[file]]\npath = \"new.log\"\n";

    /// Walks the [`STATED`] home, reading its stated facts when `stated`; the match at each path relative to the root.
    fn walk_stated(stated: bool) -> std::collections::BTreeMap<String, Match> {
        use fresher_core::KnownFolder;

        let tree = FixtureTree::from_toml_str(STATED).unwrap();
        let whole = |name: &str| interest(Pattern::Under(KnownFolder::Home, name.into()), Claim::Whole, &[]);
        let mut set = InterestSet::new();
        set.add(
            ModuleId::new("demo").unwrap(),
            &[whole("cache"), whole("theirs"), interest(Pattern::Extension("log".into()), Claim::Item, &[])],
        );
        let roots = [tree.root().to_path_buf()];
        let walk = NaiveWalk::new(tree.root());
        let walk = if stated { walk.with_stated(tree.stated().clone()) } else { walk };
        let mut matches = std::collections::BTreeMap::new();
        walk.walk(
            &request(&roots, &set),
            &CancelToken::new(),
            &mut |found| {
                let path = found.path.strip_prefix(tree.root()).unwrap().to_string_lossy().into_owned();
                matches.insert(path, found);
            },
            &mut |_| {},
        )
        .unwrap();
        matches
    }

    /// `days` before the fixtures' "now".
    #[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
    fn days_ago(days: u64) -> Option<fresher_core::Timestamp> {
        Some(fresher_core::Timestamp::from_secs(crate::FIXTURE_NOW - days * 86_400))
    }

    fn paths(matches: &[(String, Match)]) -> Vec<&str> {
        matches.iter().map(|(path, _)| path.as_str()).collect()
    }

    /// Walks `home/` of `tree` as the scan root and `tmp/` and `T/` as the places, for the top-level items of both
    /// places and every `node_modules` folder; the matches' paths relative to the tree's root, and the summary.
    fn walk_places(tree: &FixtureTree) -> (Vec<String>, fresher_core::WalkSummary) {
        use fresher_core::{KnownFolder, ModuleId, Place};

        let root = std::fs::canonicalize(tree.root()).unwrap();
        let places = [
            Place { folder: KnownFolder::SharedTemp, path: root.join("tmp") },
            Place { folder: KnownFolder::UserTemp, path: root.join("T") },
        ];
        let folders = crate::FakeKnownFolders::new()
            .with(KnownFolder::Home, root.join("home"))
            .with(KnownFolder::SharedTemp, root.join("tmp"))
            .with(KnownFolder::UserTemp, root.join("T"));
        let whole = |folder| interest(Pattern::Under(folder, "*".into()), Claim::Whole, &[]);
        let by_name = interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[]);
        let mut set = InterestSet::new();
        set.add(
            ModuleId::new("temp").unwrap(),
            &[whole(KnownFolder::SharedTemp), whole(KnownFolder::UserTemp), by_name],
        );
        let roots = [root.join("home")];
        let request = WalkRequest { places: &places, ..request(&roots, &set) };
        let mut matches = Vec::new();
        let summary = NaiveWalk::new(root.join("home"))
            .with_known_folders(folders)
            .walk(&request, &CancelToken::new(), &mut |found| matches.push(found), &mut |_| {})
            .unwrap();
        let mut found: Vec<String> = matches
            .iter()
            .map(|found| format!("{}#{}", found.path.strip_prefix(&root).unwrap().display(), found.interest))
            .collect();
        found.sort();
        (found, summary)
    }

    mod negative {
        #[test]
        fn an_entry_not_stated_reads_as_the_disk_has_it() {
            let matches = super::walk_stated(true);

            let new = &matches["new.log"];
            assert_eq!(new.kind, EntryKind::File);
            assert!(new.facts.owned_by_user);
            assert!(new.changed > super::days_ago(1), "a file written now changed now");
        }

        #[test]
        fn without_the_stated_facts_the_walk_reads_the_disk() {
            let matches = super::walk_stated(false);

            let inside = matches["cache"].inside.unwrap();
            assert!(inside.newest_change > super::days_ago(1) && !inside.special && !inside.foreign);
            assert!(matches["root.log"].facts.owned_by_user);
            assert_eq!(matches["pipe.log"].kind, EntryKind::File);
        }

        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use fresher_core::{
            CACHEDIR_SIGNATURE, CACHEDIR_TAG, CancelToken, Claim, Completeness, DiskWalk, EntryKind, InterestSet,
            KnownFolder, MarkerCheck, ModuleId, NotScanned, Pattern, ScanKind, SubtreeQuery, Timestamp, Unreadable,
            WalkError, WalkMode, WalkRequest,
        };

        use super::{PathBuf, interest, paths, request, walk};
        use crate::{FixtureTree, NaiveWalk};

        #[test]
        fn the_walk_does_not_follow_a_symlink_out_of_the_root() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"outside/Caches/big.bin\"\nsize = 100\n\
                 [[symlink]]\npath = \"home/link\"\ntarget = \"../outside\"\n\
                 [[symlink]]\npath = \"home/Caches\"\ntarget = \"../outside/Caches\"\n",
            )
            .unwrap();
            let home = tree.root().join("home");
            let mut set = InterestSet::new();
            set.add(
                ModuleId::new("demo").unwrap(),
                &[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])],
            );
            let mut matches = Vec::new();

            let summary = NaiveWalk::new(&home)
                .walk(
                    &request(std::slice::from_ref(&home), &set),
                    &CancelToken::new(),
                    &mut |found| matches.push(found),
                    &mut |_| {},
                )
                .unwrap();

            assert_eq!(summary.entries, 2, "the two links, nothing behind them");
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].path, home.join("Caches"));
            assert_eq!(matches[0].kind, EntryKind::Symlink);
            assert_eq!(matches[0].sizes.logical.bytes(), 0);
        }

        #[test]
        fn a_look_alike_is_not_matched() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"Caches.dmg.txt\"\n[[file]]\npath = \"Library/Caches\"\n\
                 [[dir]]\npath = \"a/Library/Caches/x\"\n[[dir]]\npath = \"tagged\"\n[[dir]]\npath = \"tagged/CACHEDIR.TAG\"\n",
            )
            .unwrap();
            let interests = [
                interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
                interest(Pattern::Extension("dmg".into()), Claim::Item, &[]),
                interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Item, &[]),
            ];

            let matches = walk(&tree, &interests).unwrap();

            assert_eq!(paths(&matches), ["a/Library/Caches"]);
        }

        /// The walk's check of the tag in `folder`.
        fn tag_check(folder: &std::path::Path) -> Option<MarkerCheck> {
            super::super::marker_check(&Pattern::MarkerFile(CACHEDIR_TAG.into()), folder)
        }

        #[test]
        fn a_fifo_named_like_the_tag_is_not_valid_and_does_not_block() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n").unwrap();
            let fifo = tree.root().join("cache/CACHEDIR.TAG");
            assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());

            assert_eq!(tag_check(&tree.root().join("cache")), Some(MarkerCheck::Invalid));
            let matches = walk(&tree, &[interest(Pattern::MarkerFile(CACHEDIR_TAG.into()), Claim::Item, &[])]);
            assert!(matches.unwrap().iter().all(|(_, found)| found.marker != Some(MarkerCheck::Valid)));
        }

        #[test]
        fn a_tag_linked_to_a_valid_tag_is_not_valid() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n[[dir]]\npath = \"linked\"\n").unwrap();
            fs::write(tree.root().join("cache/CACHEDIR.TAG"), CACHEDIR_SIGNATURE).unwrap();
            std::os::unix::fs::symlink("../cache/CACHEDIR.TAG", tree.root().join("linked/CACHEDIR.TAG")).unwrap();

            assert_eq!(tag_check(&tree.root().join("cache")), Some(MarkerCheck::Valid));
            assert_eq!(tag_check(&tree.root().join("linked")), Some(MarkerCheck::Invalid));
        }

        #[test]
        fn a_tag_without_the_signature_is_reported_invalid() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"cache/CACHEDIR.TAG\"\n").unwrap();
            fs::write(tree.root().join("cache/CACHEDIR.TAG"), b"Signature: 0000").unwrap();

            let matches = walk(&tree, &[interest(Pattern::MarkerFile(CACHEDIR_TAG.into()), Claim::Item, &[])]).unwrap();

            assert_eq!(paths(&matches), ["cache"]);
            assert_eq!(matches[0].1.marker, Some(MarkerCheck::Invalid));
        }

        #[test]
        fn another_marker_carries_no_check() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"project/Cargo.toml\"\n").unwrap();

            let matches = walk(&tree, &[interest(Pattern::MarkerFile("Cargo.toml".into()), Claim::Item, &[])]).unwrap();

            assert_eq!(paths(&matches), ["project"]);
            assert_eq!(matches[0].1.marker, None);
        }

        #[test]
        fn a_marker_or_a_sibling_differing_in_case_is_not_found() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"cache/cachedir.tag\"\n[[dir]]\npath = \"app/node_modules\"\n\
                 [[file]]\npath = \"app/Package.json\"\n",
            )
            .unwrap();
            let interests = [
                interest(Pattern::MarkerFile(CACHEDIR_TAG.into()), Claim::Item, &[]),
                interest(Pattern::FolderName("node_modules".into()), Claim::Item, &["package.json"]),
            ];

            let matches = walk(&tree, &interests).unwrap();

            assert_eq!(paths(&matches), ["app/node_modules"]);
            assert_eq!(matches[0].1.siblings, Vec::<String>::new());
        }

        #[test]
        fn a_whole_claim_does_not_enter_an_avoided_folder() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"Caches/kept\"\nsize = 3\n[[file]]\npath = \"Caches/Documents/a\"\nsize = 5\n",
            )
            .unwrap();
            let mut set = InterestSet::new();
            set.add(
                ModuleId::new("demo").unwrap(),
                &[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])],
            );
            let (roots, avoid) = ([tree.root().to_path_buf()], [tree.root().join("Caches/Documents")]);
            let mut matches = Vec::new();

            NaiveWalk::new(tree.root())
                .walk(
                    &WalkRequest { avoid: &avoid, ..request(&roots, &set) },
                    &CancelToken::new(),
                    &mut |found| matches.push(found),
                    &mut |_| {},
                )
                .unwrap();

            assert_eq!(matches[0].sizes.logical.bytes(), 3);
            assert_eq!(matches[0].sizes.completeness, Completeness::Partial);
        }

        #[test]
        fn the_reference_walk_has_nothing_to_restore() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
            let roots = [tree.root().to_path_buf()];
            let set = InterestSet::new();
            let restore = WalkRequest { mode: WalkMode::Restore, ..request(&roots, &set) };

            let error = NaiveWalk::new(tree.root()).walk(&restore, &CancelToken::new(), &mut |_| {}, &mut |_| {});

            assert_eq!(error, Err(WalkError::NothingToRestore));
        }

        #[test]
        fn the_reference_walk_does_not_enter_an_avoided_folder() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"Documents/Caches/a\"\nsize = 5\n[[file]]\npath = \"Library/Caches/b\"\nsize = 3\n",
            )
            .unwrap();
            let mut set = InterestSet::new();
            set.add(
                ModuleId::new("demo").unwrap(),
                &[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])],
            );
            let (roots, avoid) = ([tree.root().to_path_buf()], [tree.root().join("Documents")]);
            let avoiding = WalkRequest { avoid: &avoid, places: &[], mode: WalkMode::Auto, ..request(&roots, &set) };
            let walk = NaiveWalk::new(tree.root());
            let mut matches = Vec::new();

            let summary =
                walk.walk(&avoiding, &CancelToken::new(), &mut |found| matches.push(found.path), &mut |_| {}).unwrap();
            let state = walk.commit(&[]).unwrap();

            assert_eq!(matches, [tree.root().join("Library/Caches")]);
            assert_eq!(summary.not_scanned.get(&NotScanned::Avoided), Some(&1));
            assert_eq!(summary.kind, ScanKind::Full);
            assert!(summary.is_complete());
            let places = state.unreadable(10);
            assert_eq!(
                (places.len(), places[0].node.name.as_str(), places[0].reason),
                (1, "Documents", NotScanned::Avoided)
            );
            let root = state.node(state.roots()[0]).unwrap();
            assert_eq!((root.sizes.logical.bytes(), root.sizes.completeness), (3, Completeness::Partial));
        }

        #[test]
        fn an_avoided_root_is_not_entered() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
            let roots = [tree.root().to_path_buf()];
            let set = InterestSet::new();
            let avoiding = WalkRequest { avoid: &roots, ..request(&roots, &set) };

            let summary =
                NaiveWalk::new(tree.root()).walk(&avoiding, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();

            assert_eq!((summary.entries, summary.not_scanned.get(&NotScanned::Avoided)), (0, Some(&1)));
        }

        #[test]
        fn a_missing_root_or_a_file_as_root_is_unreadable() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"file\"\n").unwrap();
            let walk = NaiveWalk::new(tree.root());
            let roots = [tree.root().to_path_buf(), tree.root().join("missing"), tree.root().join("file")];

            for (index, root) in roots.iter().enumerate().skip(1) {
                let error = walk.walk(
                    &request(&[roots[0].clone(), root.clone()], &InterestSet::new()),
                    &CancelToken::new(),
                    &mut |_| {},
                    &mut |_| {},
                );

                assert!(matches!(error, Err(WalkError::RootUnreadable { index: 1, .. })), "{index}");
            }
        }

        #[test]
        fn a_cancelled_walk_stops() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
            let cancel = CancelToken::new();
            cancel.cancel();

            let error = NaiveWalk::new(tree.root()).walk(
                &request(&[tree.root().into()], &InterestSet::new()),
                &cancel,
                &mut |_| {},
                &mut |_| {},
            );

            assert_eq!(error, Err(WalkError::Cancelled));
        }

        #[test]
        fn an_unreadable_folder_is_counted_and_makes_a_whole_size_partial() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Caches/locked/a\"\nsize = 5\n").unwrap();
            let locked = tree.root().join("Caches/locked");
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
            let item = walk(&tree, &[interest(Pattern::FolderName("locked".into()), Claim::Item, &[])]);
            let whole = walk(&tree, &[interest(Pattern::FolderName("Caches".into()), Claim::Whole, &[])]);
            let mut summary_set = InterestSet::new();
            summary_set.add(ModuleId::new("demo").unwrap(), &[]);
            let summary = NaiveWalk::new(tree.root())
                .walk(
                    &request(&[PathBuf::from(tree.root())], &summary_set),
                    &CancelToken::new(),
                    &mut |_| {},
                    &mut |_| {},
                )
                .unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

            assert_eq!(paths(&item.unwrap()), ["Caches/locked"]);
            assert_eq!(whole.unwrap()[0].1.sizes.completeness, Completeness::Partial);
            assert_eq!(summary.unreadable.get(&Unreadable::NoPermission), Some(&1));
            assert!(!summary.is_complete());
        }

        #[test]
        fn in_a_place_only_anchored_interests_match_and_the_system_items_are_not_opened() {
            let tree = FixtureTree::from_toml_str(
                "[[dir]]\npath = \"home/node_modules\"\n[[dir]]\npath = \"tmp/node_modules\"\n\
                 [[file]]\npath = \"T/TemporaryItems/x\"\n",
            )
            .unwrap();

            let (found, summary) = super::walk_places(&tree);

            assert_eq!(found, ["home/node_modules#2", "tmp/node_modules#0"]);
            assert_eq!(summary.not_scanned.get(&NotScanned::SystemItems), Some(&1));
            assert!(summary.is_complete());
        }

        #[test]
        fn a_place_that_does_not_exist_is_not_scanned_and_the_walk_goes_on() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"home/a\"\n[[file]]\npath = \"tmp/b\"\n").unwrap();

            let (found, summary) = super::walk_places(&tree);

            assert_eq!(found, ["tmp/b#0"]);
            assert_eq!(summary.not_scanned.get(&NotScanned::Unreadable(Unreadable::Failed)), Some(&1));
            assert!(summary.is_complete());
        }

        #[test]
        fn a_folder_not_listed_makes_the_naive_answer_incomplete() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"p/src/a\"\n[[file]]\npath = \"p/vendor/b\"\n\
                 [[dir]]\npath = \"p/src\"\nmodified = 1700000200\n[[dir]]\npath = \"p/vendor\"\nmodified = 1700000300\n\
                 [[dir]]\npath = \"p\"\nmodified = 1700000100\n",
            )
            .unwrap();
            let p = tree.root().join("p");
            let (roots, avoid, set) = ([tree.root().to_path_buf()], [p.join("vendor")], InterestSet::new());
            let avoiding = WalkRequest { avoid: &avoid, ..request(&roots, &set) };
            let walk = NaiveWalk::new(tree.root());
            walk.walk(&avoiding, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();

            let answers = walk.subtrees(&[
                SubtreeQuery { folder: p.clone(), skip: vec![] },
                SubtreeQuery { folder: p.clone(), skip: vec![p.join("vendor")] },
                SubtreeQuery { folder: p.join("vendor/inside"), skip: vec![] },
            ]);

            let whole = answers[0].unwrap();
            assert_eq!((whole.newest, whole.complete), (Some(Timestamp::from_secs(1_700_000_300)), false));
            let skipped = answers[1].unwrap();
            assert_eq!((skipped.newest, skipped.complete), (Some(Timestamp::from_secs(1_700_000_200)), true));
            assert_eq!(answers[2], None);
        }
    }

    mod positive {
        use fresher_core::{
            ByteSize, CACHEDIR_TAG, CancelToken, CategoryId, ChildOrder, Claim, ClaimedPath, DiskWalk, EntryKind,
            InterestSet, KnownFolder, MarkerCheck, ModuleId, NotScanned, Pattern, SubtreeQuery, Timestamp, Unreadable,
        };

        use super::{interest, paths, request, walk};
        use crate::{FixtureTree, NaiveWalk};

        #[test]
        fn a_stated_change_reads_in_the_match_and_in_its_folders_inside() {
            let matches = super::walk_stated(true);

            let old = &matches["old.log"];
            assert_eq!((old.changed, old.accessed), (super::days_ago(30), super::days_ago(35)));
            let inside = matches["cache"].inside.unwrap();
            assert_eq!(inside.newest_change, super::days_ago(30));
            assert!(inside.complete && !inside.foreign);
        }

        #[test]
        fn a_stated_socket_or_pipe_is_another_kind() {
            let matches = super::walk_stated(true);

            assert_eq!(matches["pipe.log"].kind, EntryKind::Other);
            assert!(matches["cache"].inside.unwrap().special);
        }

        #[test]
        fn a_stated_owner_or_flag_is_the_systems() {
            let matches = super::walk_stated(true);

            let root = matches["root.log"].facts;
            assert!(!root.owned_by_user && root.system_immutable);
            assert!(matches["theirs"].inside.unwrap().foreign);
        }

        #[test]
        fn a_lock_files_name_inside_marks_only_its_folder() {
            let matches = super::walk_stated(true);

            assert!(matches["theirs"].inside.unwrap().lock_named);
            assert!(!matches["cache"].inside.unwrap().lock_named);
        }

        #[test]
        fn matches_every_kind_of_pattern_in_name_order() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"Downloads/b.dmg\"\nsize = 7\nmodified = 1000\n[[file]]\npath = \"Downloads/a.dmg\"\n\
                 [[file]]\npath = \"Library/Caches/com.example/data\"\n[[file]]\npath = \"project/cache/CACHEDIR.TAG\"\n\
                 [[dir]]\npath = \"project/node_modules\"\n[[file]]\npath = \"project/package.json\"\n",
            )
            .unwrap();
            let interests = [
                interest(Pattern::Extension("dmg".into()), Claim::Item, &[]),
                interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Whole, &[]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
                interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json", "yarn.lock"]),
            ];

            let matches = walk(&tree, &interests).unwrap();

            assert_eq!(
                paths(&matches),
                [
                    "Downloads/a.dmg",
                    "Downloads/b.dmg",
                    "Library/Caches/com.example",
                    "project/cache",
                    "project/node_modules"
                ]
            );
            let indexes: Vec<usize> = matches.iter().map(|(_, found)| found.interest).collect();
            assert_eq!(indexes, [0, 0, 1, 2, 3]);
            assert_eq!(matches[1].1.sizes.logical, ByteSize::new(7));
            assert_eq!(matches[1].1.modified, Some(Timestamp::from_secs(1000)));
            assert_eq!(matches[1].1.kind, EntryKind::File);
            assert_eq!(matches[4].1.siblings, ["package.json"]);
        }

        #[test]
        fn the_reference_walk_reports_a_valid_tag() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"project/cache/CACHEDIR.TAG\"\n").unwrap();
            let content = b"Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag.\n";
            std::fs::write(tree.root().join("project/cache/CACHEDIR.TAG"), content).unwrap();

            let matches =
                walk(&tree, &[interest(Pattern::MarkerFile(CACHEDIR_TAG.into()), Claim::Whole, &[])]).unwrap();

            assert_eq!(paths(&matches), ["project/cache"]);
            assert_eq!(matches[0].1.marker, Some(MarkerCheck::Valid));
        }

        #[test]
        fn the_reference_walk_reports_what_a_matched_folder_holds_and_its_tag() {
            let tree = FixtureTree::from_toml_str(
                "[[dir]]\npath = \"cache/index-v5\"\n[[file]]\npath = \"cache/CACHEDIR.TAG\"\n\
                 [[dir]]\npath = \"other\"\n",
            )
            .unwrap();
            std::fs::write(tree.root().join("cache/CACHEDIR.TAG"), fresher_core::CACHEDIR_SIGNATURE).unwrap();
            let held = |path: &str| {
                interest(Pattern::Under(KnownFolder::Home, path.into()), Claim::Item, &[])
                    .holding(vec!["content-v2".into(), CACHEDIR_TAG.into(), "index-v5".into()])
                    .unwrap()
            };

            let matches = walk(&tree, &[held("cache"), held("other")]).unwrap();

            assert_eq!(paths(&matches), ["cache", "other"]);
            assert_eq!(matches[0].1.holds, [CACHEDIR_TAG, "index-v5"]);
            assert_eq!(matches[0].1.marker, Some(MarkerCheck::Valid));
            assert_eq!((matches[1].1.holds.len(), matches[1].1.marker), (0, Some(MarkerCheck::Invalid)));
        }

        #[test]
        fn the_reference_walk_commits_its_folders_with_their_sizes_and_claims() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"a/b/f\"\nsize = 10\n[[file]]\npath = \"a/g\"\nsize = 5\n[[dir]]\npath = \"locked\"\n",
            )
            .unwrap();
            let locked = tree.root().join("locked");
            std::fs::set_permissions(&locked, std::os::unix::fs::PermissionsExt::from_mode(0o000)).unwrap();
            let roots = [tree.root().to_path_buf()];
            let set = InterestSet::new();
            let walk = NaiveWalk::new(tree.root());
            walk.walk(&request(&roots, &set), &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();
            std::fs::set_permissions(&locked, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
            let category = CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap();
            let claims = [ClaimedPath { path: tree.root().join("a/b"), category }];

            let first = walk.commit(&[]).unwrap();
            let second = walk.commit(&claims).unwrap();

            assert_ne!(first.id(), second.id());
            let root = second.node(second.roots()[0]).unwrap();
            assert_eq!((root.sizes.logical, root.folders, root.files), (ByteSize::new(15), 3, 0));
            let children = second.children(root.id, ChildOrder::ByName, 0, 10).unwrap();
            let names: Vec<_> = children.nodes.iter().map(|node| node.name.as_str()).collect();
            assert_eq!(names, ["a", "locked"]);
            assert_eq!(children.nodes[1].not_scanned, Some(NotScanned::Unreadable(Unreadable::NoPermission)));
            let owned = second.search("b", 10).into_iter().find(|node| node.name == "b").unwrap();
            assert_eq!((owned.owner, owned.sizes.logical), (Some(category), ByteSize::new(10)));
        }

        #[test]
        fn a_whole_claim_sums_the_folder_and_is_not_walked_inside() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"node_modules/a/index.js\"\nsize = 10\n[[file]]\npath = \"node_modules/b\"\nsize = 5\n\
                 [[file]]\npath = \"node_modules/a/node_modules/c\"\nsize = 1\n\
                 [[symlink]]\npath = \"node_modules/link\"\ntarget = \"/usr\"\n",
            )
            .unwrap();

            let matches =
                walk(&tree, &[interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[])]).unwrap();

            assert_eq!(paths(&matches), ["node_modules"]);
            assert_eq!(matches[0].1.sizes.logical, ByteSize::new(16));
            assert_eq!(matches[0].1.sizes.reclaimable, matches[0].1.sizes.allocated);
            assert_eq!(matches[0].1.sizes.completeness, fresher_core::Completeness::Complete);
        }

        #[test]
        fn an_item_claim_walks_on_inside_and_reports_progress() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Caches/x/Caches/y\"\nsize = 3\n").unwrap();
            let mut set = InterestSet::new();
            set.add(
                ModuleId::new("demo").unwrap(),
                &[interest(Pattern::FolderName("Caches".into()), Claim::Item, &[])],
            );
            let (mut matches, mut progress) = (Vec::new(), Vec::new());

            let summary = NaiveWalk::new(tree.root())
                .walk(
                    &request(&[tree.root().into()], &set),
                    &CancelToken::new(),
                    &mut |found| matches.push(found),
                    &mut |seen| progress.push(seen),
                )
                .unwrap();

            assert_eq!(matches.len(), 2);
            assert_eq!(matches[0].sizes, fresher_core::Sizes::ZERO);
            assert_eq!(summary.entries, 4);
            assert!(summary.is_complete());
            assert_eq!(progress.last().map(|seen| (seen.entries, seen.bytes)), Some((4, ByteSize::new(3))));
        }

        #[test]
        fn a_place_is_walked_after_the_scan_roots_for_its_interests() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"home/a\"\n[[file]]\npath = \"tmp/old.log\"\n[[file]]\npath = \"T/item\"\n",
            )
            .unwrap();

            let (found, summary) = super::walk_places(&tree);

            assert_eq!(found, ["T/item#1", "tmp/old.log#0"]);
            assert!(summary.is_complete());
        }

        #[test]
        fn the_naive_walk_answers_from_the_folders_it_recorded() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"p/src/a.rs\"\nsize = 5\n[[file]]\npath = \"p/node_modules/x/b.js\"\nsize = 7\n\
                 [[dir]]\npath = \"p/src\"\nmodified = 1700000200\n[[dir]]\npath = \"p/node_modules/x\"\nmodified = 1700000900\n\
                 [[dir]]\npath = \"p/node_modules\"\nmodified = 1700000800\n[[dir]]\npath = \"p\"\nmodified = 1700000100\n",
            )
            .unwrap();
            let p = tree.root().join("p");
            let mut set = InterestSet::new();
            set.add(
                ModuleId::new("demo").unwrap(),
                &[interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[])],
            );
            let roots = [tree.root().to_path_buf()];
            let walk = NaiveWalk::new(tree.root());
            let before = walk.subtrees(&[SubtreeQuery { folder: p.clone(), skip: vec![] }]);
            walk.walk(&request(&roots, &set), &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();

            let answers = walk.subtrees(&[
                SubtreeQuery { folder: p.clone(), skip: vec![p.join("node_modules")] },
                SubtreeQuery { folder: p.clone(), skip: vec![] },
                SubtreeQuery { folder: p.join("node_modules/x"), skip: vec![] },
                SubtreeQuery { folder: tree.root().to_path_buf(), skip: vec![] },
            ]);

            assert_eq!(before, vec![None]);
            let own = answers[0].unwrap();
            assert_eq!(
                (own.newest, own.complete, own.sizes.logical),
                (Some(Timestamp::from_secs(1_700_000_200)), true, ByteSize::new(12))
            );
            assert_eq!(
                answers[1].unwrap().newest,
                Some(Timestamp::from_secs(1_700_000_800)),
                "claimed whole: its own time"
            );
            assert_eq!(answers[2], None, "inside a folder claimed whole");
            assert_eq!(answers[3].unwrap().newest, Some(Timestamp::from_secs(1_700_000_800)), "the root has no time");
        }
    }
}
