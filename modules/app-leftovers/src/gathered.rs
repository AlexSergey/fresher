//! The leftovers of one scan, gathered while the walk runs and decided in `finish`: an id's leftovers lie in several
//! places, and matches come in no order. Nothing here reads the disk.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{BundleId, CancelToken, EntryFacts, Inside, Sizes, Timestamp};

use crate::places::Form;

/// A match of a row: a folder or a file named after the id `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Leftover {
    /// The index of the row in the places table.
    pub(crate) row: usize,
    pub(crate) id: BundleId,
    pub(crate) path: PathBuf,
    pub(crate) kind: Form,
    pub(crate) sizes: Sizes,
    pub(crate) modified: Option<Timestamp>,
    pub(crate) changed: Option<Timestamp>,
    /// What a folder holds; `None` for a file, and for a folder the walk did not read.
    pub(crate) inside: Option<Inside>,
    pub(crate) facts: EntryFacts,
}

/// The leftovers gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves
/// its leftovers behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Vec<Leftover>)>>,
}

impl Gathered {
    /// Keeps `leftover` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add(&self, token: &CancelToken, leftover: Leftover) {
        let mut scan = self.lock();
        match scan.as_mut() {
            Some((kept, leftovers)) if kept.is_same(token) => leftovers.push(leftover),
            _ => *scan = Some((token.clone(), vec![leftover])),
        }
    }

    /// The leftovers gathered for the scan of `token`, by id, each id's sorted by path then row; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> BTreeMap<BundleId, Vec<Leftover>> {
        let mut by_id: BTreeMap<BundleId, Vec<Leftover>> = BTreeMap::new();
        if let Some((kept, leftovers)) = self.lock().take()
            && kept.is_same(token)
        {
            for leftover in leftovers {
                by_id.entry(leftover.id.clone()).or_default().push(leftover);
            }
        }
        for leftovers in by_id.values_mut() {
            leftovers.sort_by(|first, second| (&first.path, first.row).cmp(&(&second.path, second.row)));
        }
        by_id
    }

    /// The lock, even after a panic while it was held: what it guards is a plain list.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Vec<Leftover>)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{BundleId, EntryFacts, Inside, Sizes};

    use crate::gathered::Leftover;
    use crate::places::Form;

    fn leftover(id: &str, path: &str) -> Leftover {
        Leftover {
            row: 0,
            id: BundleId::new(id).unwrap(),
            path: path.into(),
            kind: Form::Folder,
            sizes: Sizes::ZERO,
            modified: None,
            changed: None,
            inside: Some(Inside::EMPTY),
            facts: EntryFacts::REMOVABLE,
        }
    }

    mod negative {
        use fresher_core::{BundleId, CancelToken};

        use super::leftover;
        use crate::gathered::Gathered;

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add(&cancelled, leftover("com.example.Gone", "/a"));

            assert!(gathered.take(&next).is_empty());

            gathered.add(&cancelled, leftover("com.example.Gone", "/a"));
            gathered.add(&next, leftover("com.example.Other", "/b"));
            let taken = gathered.take(&next);
            assert_eq!(taken.keys().map(BundleId::as_str).collect::<Vec<_>>(), ["com.example.Other"]);
        }

        #[test]
        fn nothing_is_kept_once_taken() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, leftover("com.example.Gone", "/a"));

            assert_eq!(gathered.take(&token).len(), 1);
            assert!(gathered.take(&token).is_empty());
        }
    }

    mod positive {
        use fresher_core::CancelToken;

        use super::leftover;
        use crate::gathered::Gathered;

        #[test]
        fn the_leftovers_of_one_scan_come_back_by_id_sorted_by_path() {
            let gathered = Gathered::default();
            let scan = CancelToken::new();
            for (id, path) in [("com.example.B", "/w"), ("com.example.A", "/d/2"), ("COM.EXAMPLE.A", "/d/1")] {
                gathered.add(&scan, leftover(id, path));
            }

            let by_id = gathered.take(&scan);

            let paths: Vec<(&str, Vec<&str>)> = by_id
                .iter()
                .map(|(id, leftovers)| {
                    (id.folded(), leftovers.iter().map(|leftover| leftover.path.to_str().unwrap()).collect())
                })
                .collect();
            assert_eq!(paths, [("com.example.a", vec!["/d/1", "/d/2"]), ("com.example.b", vec!["/w"])]);
        }
    }
}
