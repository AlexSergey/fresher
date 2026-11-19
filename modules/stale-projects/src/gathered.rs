//! The folders the module matched in one scan, gathered while the walk runs and read in `finish`: the module offers
//! only a folder it matched itself. Nothing here reads the disk.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryKind, Match};

/// The folder paths gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish`
/// leaves its paths behind, and the next scan, with another token, drops them instead of reading them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, BTreeSet<PathBuf>)>>,
}

impl Gathered {
    /// Keeps the path of `matched` for the scan of `token` when it is a folder, dropping what an earlier scan left; a
    /// file, a link or an `Other` entry adds nothing.
    pub(crate) fn add(&self, token: &CancelToken, matched: &Match) {
        if matched.kind != EntryKind::Folder {
            return;
        }
        let mut scan = self.lock();
        match scan.as_mut() {
            Some((kept, paths)) if kept.is_same(token) => {
                paths.insert(matched.path.clone());
            }
            _ => *scan = Some((token.clone(), BTreeSet::from([matched.path.clone()]))),
        }
    }

    /// The folders gathered for the scan of `token`; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> BTreeSet<PathBuf> {
        match self.lock().take() {
            Some((kept, paths)) if kept.is_same(token) => paths,
            _ => BTreeSet::new(),
        }
    }

    /// The lock, even after a panic while it was held: what it guards is a plain set.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, BTreeSet<PathBuf>)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{EntryFacts, EntryKind, Match, Sizes};

    fn matched(interest: usize, path: &str, kind: EntryKind) -> Match {
        Match {
            interest,
            path: path.into(),
            kind,
            sizes: Sizes::ZERO,
            modified: None,
            changed: None,
            accessed: None,
            inside: None,
            siblings: Vec::new(),
            holds: Vec::new(),
            marker: None,
            facts: EntryFacts::REMOVABLE,
        }
    }

    mod negative {
        use fresher_core::{CancelToken, EntryKind};

        use super::matched;
        use crate::gathered::Gathered;

        #[test]
        fn a_file_a_link_or_an_other_entry_adds_nothing() {
            let (gathered, scan) = (Gathered::default(), CancelToken::new());
            for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
                gathered.add(&scan, &matched(0, "/home/Code/shop/node_modules", kind));
            }

            assert!(gathered.take(&scan).is_empty());
        }

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add(&cancelled, &matched(0, "/home/a/node_modules", EntryKind::Folder));

            assert!(gathered.take(&next).is_empty());

            gathered.add(&cancelled, &matched(0, "/home/a/node_modules", EntryKind::Folder));
            gathered.add(&next, &matched(0, "/home/b/target", EntryKind::Folder));
            let paths: Vec<_> = gathered.take(&next).into_iter().collect();
            assert_eq!(paths, [std::path::PathBuf::from("/home/b/target")]);
            assert!(gathered.take(&next).is_empty());
        }
    }

    mod positive {
        use fresher_core::{CancelToken, EntryKind};

        use super::matched;
        use crate::gathered::Gathered;

        #[test]
        fn the_folders_of_one_scan_of_any_interest_come_back_once_each() {
            let (gathered, scan) = (Gathered::default(), CancelToken::new());
            for (interest, path) in [(8, "/home/b/target"), (7, "/home/a/node_modules"), (8, "/home/b/target")] {
                gathered.add(&scan, &matched(interest, path, EntryKind::Folder));
            }

            let paths: Vec<_> = gathered.take(&scan).into_iter().collect();

            assert_eq!(paths, [std::path::PathBuf::from("/home/a/node_modules"), "/home/b/target".into()]);
        }
    }
}
