use std::fs::{self, Metadata};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use fresher_core::{
    ByteSize, CancelToken, Claim, Completeness, DiskWalk, EntryKind, Interest, InterestSet, Match, Pattern, Sizes,
    Timestamp, Unreadable, WalkError, WalkProgress, WalkSummary,
};

/// The reference [`DiskWalk`]: simple and slow, with `std::fs` only. Links are listed, never followed; the walk never
/// leaves the device of a root. Entries are visited in name order, so its matches come in a stable order. The real
/// scanner is tested against it: the same fixture gives the same matches.
#[derive(Clone, Debug)]
pub struct NaiveWalk {
    home: PathBuf,
}

impl NaiveWalk {
    /// A walk for which `home` is the home folder that `Pattern::UnderHome` paths are relative to: in a test, the
    /// root of the fixture tree.
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }
}

impl DiskWalk for NaiveWalk {
    fn walk(
        &self,
        roots: &[PathBuf],
        interests: &InterestSet,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        let mut walker =
            Walker { home: &self.home, interests, cancel, on_match, on_progress, summary: WalkSummary::default() };
        let mut progress = WalkProgress::default();
        for (index, root) in roots.iter().enumerate() {
            let unreadable = |kind| WalkError::RootUnreadable { index, kind };
            let metadata = fs::symlink_metadata(root).map_err(|error| unreadable(error.kind()))?;
            if !metadata.is_dir() {
                return Err(unreadable(io::ErrorKind::NotADirectory));
            }
            walker.folder(root, metadata.dev(), &mut progress)?;
        }
        Ok(walker.summary)
    }
}

struct Walker<'a> {
    home: &'a Path,
    interests: &'a InterestSet,
    cancel: &'a CancelToken,
    on_match: &'a mut dyn FnMut(Match),
    on_progress: &'a mut dyn FnMut(WalkProgress),
    summary: WalkSummary,
}

impl Walker<'_> {
    fn folder(&mut self, dir: &Path, device: u64, progress: &mut WalkProgress) -> Result<(), WalkError> {
        let entries = match sorted_entries(dir) {
            Ok(entries) => entries,
            Err(error) => {
                *self.summary.unreadable.entry(unreadable(&error)).or_default() += 1;
                return Ok(());
            }
        };
        for path in entries {
            if self.cancel.is_cancelled() {
                return Err(WalkError::Cancelled);
            }
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                *self.summary.unreadable.entry(Unreadable::Failed).or_default() += 1;
                continue;
            };
            if metadata.dev() != device {
                continue;
            }
            let kind = entry_kind(&metadata);
            self.summary.entries += 1;
            progress.entries += 1;
            progress.bytes = progress.bytes.checked_add(own_sizes(&metadata, kind).logical).unwrap_or(progress.bytes);
            (self.on_progress)(*progress);
            let claimed_whole = self.report(&path, &metadata, kind, device);
            if kind == EntryKind::Folder && !claimed_whole {
                self.folder(&path, device, progress)?;
            }
        }
        Ok(())
    }

    /// Reports every interest `path` matches; `true` when one of them claims it whole.
    fn report(&mut self, path: &Path, metadata: &Metadata, kind: EntryKind, device: u64) -> bool {
        let mut claimed_whole = false;
        for (index, interest) in self.interests.iter() {
            if !matches(interest.pattern(), self.home, path, kind) {
                continue;
            }
            let whole = interest.claim() == Claim::Whole;
            claimed_whole |= whole;
            let sizes =
                if whole && kind == EntryKind::Folder { tree_sizes(path, device) } else { own_sizes(metadata, kind) };
            (self.on_match)(Match {
                interest: index,
                path: path.to_path_buf(),
                kind,
                sizes,
                modified: metadata.modified().ok().map(Timestamp::from_system_time),
                siblings: present_siblings(interest, path),
            });
        }
        claimed_whole
    }
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
            kind == EntryKind::Folder && path.join(marker).symlink_metadata().is_ok_and(|marker| marker.is_file())
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

/// Everything inside the folder `dir` on `device`, links not followed; `Partial` when a folder could not be read.
fn tree_sizes(dir: &Path, device: u64) -> Sizes {
    let Ok(entries) = sorted_entries(dir) else {
        return Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };
    };
    entries.iter().fold(Sizes::ZERO, |total, path| {
        let inner = match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.dev() != device => Sizes::ZERO,
            Ok(metadata) if metadata.is_dir() => tree_sizes(path, device),
            Ok(metadata) => own_sizes(&metadata, entry_kind(&metadata)),
            Err(_) => Sizes { completeness: Completeness::Partial, ..Sizes::ZERO },
        };
        total.checked_add(inner).unwrap_or(Sizes { completeness: Completeness::Partial, ..total })
    })
}

/// The interest's sibling names that exist next to `path`, links included.
fn present_siblings(interest: &Interest, path: &Path) -> Vec<String> {
    let Some(parent) = path.parent() else { return Vec::new() };
    interest.siblings().iter().filter(|name| parent.join(name).symlink_metadata().is_ok()).cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{CancelToken, Claim, DiskWalk, Interest, InterestSet, Match, ModuleId, Pattern, WalkError};

    use crate::{FixtureTree, NaiveWalk};

    fn interest(pattern: Pattern, claim: Claim, siblings: &[&str]) -> Interest {
        Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
    }

    /// Walks `tree` for `interests` and returns the matches with paths relative to the root.
    fn walk(tree: &FixtureTree, interests: &[Interest]) -> Result<Vec<(String, Match)>, WalkError> {
        let mut set = InterestSet::new();
        set.add(ModuleId::new("demo").unwrap(), interests);
        let mut matches = Vec::new();
        let roots = [tree.root().to_path_buf()];
        NaiveWalk::new(tree.root()).walk(
            &roots,
            &set,
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
            CancelToken, Claim, DiskWalk, EntryKind, InterestSet, ModuleId, Pattern, Unreadable, WalkError,
        };

        use super::{PathBuf, interest, paths, walk};
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
                    std::slice::from_ref(&home),
                    &set,
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

        #[test]
        fn a_missing_root_or_a_file_as_root_is_unreadable() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"file\"\n").unwrap();
            let walk = NaiveWalk::new(tree.root());
            let roots = [tree.root().to_path_buf(), tree.root().join("missing"), tree.root().join("file")];

            for (index, root) in roots.iter().enumerate().skip(1) {
                let error = walk.walk(
                    &[roots[0].clone(), root.clone()],
                    &InterestSet::new(),
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
                &[tree.root().into()],
                &InterestSet::new(),
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
                .walk(&[PathBuf::from(tree.root())], &summary_set, &CancelToken::new(), &mut |_| {}, &mut |_| {})
                .unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

            assert_eq!(paths(&item.unwrap()), ["Caches/locked"]);
            assert_eq!(whole.unwrap()[0].1.sizes.completeness, fresher_core::Completeness::Partial);
            assert_eq!(summary.unreadable.get(&Unreadable::NoPermission), Some(&1));
            assert!(!summary.is_complete());
        }
    }

    mod positive {
        use fresher_core::{
            ByteSize, CancelToken, Claim, DiskWalk, EntryKind, InterestSet, ModuleId, Pattern, Timestamp,
        };

        use super::{interest, paths, walk};
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
                .walk(&[tree.root().into()], &set, &CancelToken::new(), &mut |found| matches.push(found), &mut |seen| {
                    progress.push(seen);
                })
                .unwrap();

            assert_eq!(matches.len(), 2);
            assert_eq!(matches[0].sizes, fresher_core::Sizes::ZERO);
            assert_eq!(summary.entries, 4);
            assert!(summary.is_complete());
            assert_eq!(progress.last().map(|seen| (seen.entries, seen.bytes)), Some((4, ByteSize::new(3))));
        }
    }
}
