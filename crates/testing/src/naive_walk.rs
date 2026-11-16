use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    ByteSize, CACHEDIR_SIGNATURE, CACHEDIR_TAG, CancelToken, Claim, ClaimedPath, Completeness, DiskState, DiskWalk,
    EntryKind, Interest, MarkerCheck, Match, NodeKind, NotScanned, Pattern, Sizes, StateId, Timestamp, Unreadable,
    WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary, is_valid_cachedir_tag,
};

use crate::FixedDiskState;
use crate::fakes::lock;

/// A size nothing is known of: a folder not read.
const NOT_READ: Sizes = Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };

/// The reference [`DiskWalk`]: simple and slow, with `std::fs` only. Links are listed, never followed; the walk never
/// leaves the device of a root and never opens an avoided folder. Every size is complete and private (`std::fs`
/// cannot see clones). Entries are visited in name order, so its matches come in a stable order. The real scanner is
/// tested against it: the same fixture gives the same matches.
///
/// `Full` and `Auto` both walk the disk (always a full scan); there is no cache, so `Restore` has nothing to restore.
/// `commit` returns a [`FixedDiskState`] of the folders of the last walk, with their sizes; files are not kept.
#[derive(Debug)]
pub struct NaiveWalk {
    home: PathBuf,
    walked: Mutex<Walked>,
}

/// What the last walk saw, for `commit`.
#[derive(Debug, Default)]
struct Walked {
    folders: Vec<(PathBuf, Sizes)>,
    not_scanned: Vec<(PathBuf, NotScanned)>,
    commits: u32,
}

impl NaiveWalk {
    /// A walk for which `home` is the home folder that `Pattern::UnderHome` paths are relative to: in a test, the
    /// root of the fixture tree.
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into(), walked: Mutex::default() }
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
        let mut walker = Walker {
            home: &self.home,
            request,
            cancel,
            on_match,
            on_progress,
            summary: WalkSummary::default(),
            walked: Walked::default(),
        };
        let mut progress = WalkProgress::default();
        for (index, root) in request.roots.iter().enumerate() {
            let unreadable = |kind| WalkError::RootUnreadable { index, kind };
            let metadata = fs::symlink_metadata(root).map_err(|error| unreadable(error.kind()))?;
            if !metadata.is_dir() {
                return Err(unreadable(io::ErrorKind::NotADirectory));
            }
            if request.avoid.contains(root) {
                walker.not_scanned(root, NotScanned::Avoided, NOT_READ);
                continue;
            }
            walker.folder(root, metadata.dev(), &mut progress)?;
        }
        let Walker { summary, walked, .. } = walker;
        let mut kept = lock(&self.walked);
        kept.folders = walked.folders;
        kept.not_scanned = walked.not_scanned;
        Ok(summary)
    }

    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        let mut walked = lock(&self.walked);
        walked.commits = walked.commits.saturating_add(1);
        let folders = walked.folders.iter().map(|(path, sizes)| (path.clone(), NodeKind::Folder, *sizes));
        // The reference walk has no clock: every state is taken at the epoch.
        let state = FixedDiskState::new(StateId::new(walked.commits), Timestamp::from_secs(0), folders);
        let state =
            walked.not_scanned.iter().fold(state, |state, (path, reason)| state.with_not_scanned(path, *reason));
        let state = claims.iter().fold(state, |state, claim| state.with_owner(&claim.path, claim.category));
        Ok(Arc::new(state))
    }
}

struct Walker<'a> {
    home: &'a Path,
    request: &'a WalkRequest<'a>,
    cancel: &'a CancelToken,
    on_match: &'a mut dyn FnMut(Match),
    on_progress: &'a mut dyn FnMut(WalkProgress),
    summary: WalkSummary,
    walked: Walked,
}

impl Walker<'_> {
    /// Walks the folder `dir` and gives everything inside it.
    fn folder(&mut self, dir: &Path, device: u64, progress: &mut WalkProgress) -> Result<Sizes, WalkError> {
        let entries = match sorted_entries(dir) {
            Ok(entries) => entries,
            Err(error) => {
                let reason = unreadable(&error);
                *self.summary.unreadable.entry(reason).or_default() += 1;
                self.walked.not_scanned.push((dir.to_path_buf(), NotScanned::Unreadable(reason)));
                self.walked.folders.push((dir.to_path_buf(), NOT_READ));
                return Ok(NOT_READ);
            }
        };
        let mut total = Sizes::ZERO;
        for path in entries {
            if self.cancel.is_cancelled() {
                return Err(WalkError::Cancelled);
            }
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                *self.summary.unreadable.entry(Unreadable::Failed).or_default() += 1;
                total = add(total, NOT_READ);
                continue;
            };
            if metadata.dev() != device {
                if metadata.is_dir() {
                    self.not_scanned(&path, NotScanned::OtherVolume, Sizes::ZERO);
                }
                continue;
            }
            let kind = entry_kind(&metadata);
            self.summary.entries += 1;
            progress.entries += 1;
            progress.bytes = progress.bytes.checked_add(own_sizes(&metadata, kind).logical).unwrap_or(progress.bytes);
            (self.on_progress)(*progress);
            let inner = if kind == EntryKind::Folder && self.request.avoid.contains(&path) {
                self.not_scanned(&path, NotScanned::Avoided, NOT_READ);
                NOT_READ
            } else if kind == EntryKind::Folder {
                if self.report(&path, &metadata, kind, device) {
                    let sizes = tree_sizes(&path, device, self.request.avoid);
                    self.walked.folders.push((path, sizes));
                    sizes
                } else {
                    self.folder(&path, device, progress)?
                }
            } else {
                self.report(&path, &metadata, kind, device);
                own_sizes(&metadata, kind)
            };
            total = add(total, inner);
        }
        self.walked.folders.push((dir.to_path_buf(), total));
        Ok(total)
    }

    /// Records the folder `path` as not entered on purpose, counted in the summary.
    fn not_scanned(&mut self, path: &Path, reason: NotScanned, sizes: Sizes) {
        *self.summary.not_scanned.entry(reason).or_default() += 1;
        self.walked.not_scanned.push((path.to_path_buf(), reason));
        self.walked.folders.push((path.to_path_buf(), sizes));
    }

    /// Reports every interest `path` matches; `true` when one of them claims it whole.
    fn report(&mut self, path: &Path, metadata: &Metadata, kind: EntryKind, device: u64) -> bool {
        let mut claimed_whole = false;
        for (index, interest) in self.request.interests.iter() {
            if !matches(interest.pattern(), self.home, path, kind) {
                continue;
            }
            let whole = interest.claim() == Claim::Whole;
            claimed_whole |= whole;
            let sizes = if whole && kind == EntryKind::Folder {
                tree_sizes(path, device, self.request.avoid)
            } else {
                own_sizes(metadata, kind)
            };
            (self.on_match)(Match {
                interest: index,
                path: path.to_path_buf(),
                kind,
                sizes,
                modified: metadata.modified().ok().map(Timestamp::from_system_time),
                siblings: present_siblings(interest, path),
                marker: marker_check(interest.pattern(), path),
            });
        }
        claimed_whole
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

fn entry_kind(metadata: &Metadata) -> EntryKind {
    if metadata.is_symlink() {
        EntryKind::Symlink
    } else if metadata.is_dir() {
        EntryKind::Folder
    } else {
        EntryKind::File
    }
}

fn matches(pattern: &Pattern, home: &Path, path: &Path, kind: EntryKind) -> bool {
    let name = path.file_name().and_then(|name| name.to_str());
    match pattern {
        Pattern::FolderName(folder) => kind != EntryKind::File && name == Some(folder.as_str()),
        Pattern::MarkerFile(marker) => {
            kind == EntryKind::Folder && listed(path, marker).is_some_and(|marker| marker.is_file())
        }
        Pattern::Extension(extension) => {
            kind != EntryKind::Folder && path.extension().and_then(|found| found.to_str()) == Some(extension.as_str())
        }
        Pattern::UnderHome(pattern) => path.strip_prefix(home).is_ok_and(|relative| {
            let names: Vec<_> = relative.iter().filter_map(|name| name.to_str()).collect();
            let segments: Vec<_> = pattern.split('/').collect();
            names.len() == relative.iter().count()
                && names.len() == segments.len()
                && names.iter().zip(&segments).all(|(name, segment)| *segment == "*" || name == segment)
        }),
    }
}

/// The size of the entry itself: a file's bytes; nothing for a folder or a link.
fn own_sizes(metadata: &Metadata, kind: EntryKind) -> Sizes {
    if kind == EntryKind::File {
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
            Ok(metadata) => own_sizes(&metadata, entry_kind(&metadata)),
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
        WalkRequest { roots, interests, avoid: &[], mode: WalkMode::Full }
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

    fn paths(matches: &[(String, Match)]) -> Vec<&str> {
        matches.iter().map(|(path, _)| path.as_str()).collect()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use fresher_core::{
            CACHEDIR_SIGNATURE, CACHEDIR_TAG, CancelToken, Claim, Completeness, DiskWalk, EntryKind, InterestSet,
            MarkerCheck, ModuleId, NotScanned, Pattern, ScanKind, Unreadable, WalkError, WalkMode, WalkRequest,
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
                interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, &[]),
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
            let avoiding = WalkRequest { avoid: &avoid, mode: WalkMode::Auto, ..request(&roots, &set) };
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
    }

    mod positive {
        use fresher_core::{
            ByteSize, CACHEDIR_TAG, CancelToken, CategoryId, ChildOrder, Claim, ClaimedPath, DiskWalk, EntryKind,
            InterestSet, MarkerCheck, ModuleId, NotScanned, Pattern, Timestamp, Unreadable,
        };

        use super::{interest, paths, request, walk};
        use crate::{FixtureTree, NaiveWalk};

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
                interest(Pattern::UnderHome("Library/Caches/*".into()), Claim::Whole, &[]),
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
    }
}
