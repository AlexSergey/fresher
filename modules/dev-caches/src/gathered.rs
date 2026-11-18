//! The matches of one scan, gathered while the walk runs and decided in `finish`: two rows' signs lie in another
//! match, and matches come in no order. Nothing here reads the disk.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryFacts, EntryKind, Inside, MarkerCheck, Sizes};

/// What a folder that must hold a sign holds: the names of the sign it holds, and whether its `CACHEDIR.TAG` is valid.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Sign {
    pub(crate) holds: Vec<String>,
    pub(crate) marker: Option<MarkerCheck>,
}

/// A match of a row's folder: a cache folder the module may offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    /// The index of the row in the tools table.
    pub(crate) row: usize,
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
    pub(crate) sizes: Sizes,
    pub(crate) facts: EntryFacts,
    pub(crate) inside: Option<Inside>,
    /// The sign the folder itself holds; empty for a row whose sign is in its holder.
    pub(crate) sign: Sign,
}

/// A match of a sign place: the folder that holds the sign of the rows with a `sign-at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Holder {
    /// The index of the sign place (`Role::SignHolder`).
    pub(crate) index: usize,
    pub(crate) kind: EntryKind,
    pub(crate) sign: Sign,
}

impl Holder {
    /// The sign it holds; `None` when it is not a folder (a link to a folder holds nothing of its own).
    pub(crate) fn sign(&self) -> Option<&Sign> {
        (self.kind == EntryKind::Folder).then_some(&self.sign)
    }
}

/// The matches of one scan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Scan {
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) holders: Vec<Holder>,
}

impl Scan {
    /// The holder of the sign place `index`, when exactly one was matched.
    pub(crate) fn holder(&self, index: usize) -> Option<&Holder> {
        let mut found = self.holders.iter().filter(|holder| holder.index == index);
        match (found.next(), found.next()) {
            (Some(holder), None) => Some(holder),
            _ => None,
        }
    }
}

/// The matches gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves
/// its matches behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Scan)>>,
}

impl Gathered {
    /// Keeps `candidate` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add_candidate(&self, token: &CancelToken, candidate: Candidate) {
        self.with_scan(token, |scan| scan.candidates.push(candidate));
    }

    /// Keeps `holder` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add_holder(&self, token: &CancelToken, holder: Holder) {
        self.with_scan(token, |scan| scan.holders.push(holder));
    }

    /// The matches gathered for the scan of `token`, the candidates sorted by path then row; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Scan {
        let mut scan = match self.lock().take() {
            Some((kept, scan)) if kept.is_same(token) => scan,
            _ => Scan::default(),
        };
        scan.candidates.sort_by(|first, second| (&first.path, first.row).cmp(&(&second.path, second.row)));
        scan
    }

    fn with_scan(&self, token: &CancelToken, add: impl FnOnce(&mut Scan)) {
        let mut kept = self.lock();
        match kept.as_mut() {
            Some((owner, scan)) if owner.is_same(token) => add(scan),
            _ => {
                let mut scan = Scan::default();
                add(&mut scan);
                *kept = Some((token.clone(), scan));
            }
        }
    }

    /// The lock, even after a panic while it was held: what it guards is plain lists.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Scan)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{EntryFacts, EntryKind, Inside, Sizes};

    use crate::gathered::{Candidate, Holder, Sign};

    fn candidate(row: usize, path: &str) -> Candidate {
        Candidate {
            row,
            path: path.into(),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            facts: EntryFacts::REMOVABLE,
            inside: Some(Inside::EMPTY),
            sign: Sign::default(),
        }
    }

    fn holder(index: usize, kind: EntryKind) -> Holder {
        Holder { index, kind, sign: Sign { holds: vec!["api".into()], marker: None } }
    }

    mod negative {
        use fresher_core::{CancelToken, EntryKind};

        use super::{candidate, holder};
        use crate::gathered::{Gathered, Scan};

        #[test]
        fn a_holder_that_is_not_a_folder_holds_no_sign() {
            for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
                assert_eq!(holder(0, kind).sign(), None);
            }
        }

        #[test]
        fn a_sign_place_matched_twice_or_never_has_no_holder() {
            let scan = Scan {
                candidates: Vec::new(),
                holders: vec![holder(0, EntryKind::Folder), holder(0, EntryKind::Folder)],
            };

            assert_eq!(scan.holder(0), None);
            assert_eq!(scan.holder(1), None);
        }

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add_candidate(&cancelled, candidate(0, "/home/.npm/_cacache"));
            gathered.add_holder(&cancelled, holder(0, EntryKind::Folder));

            gathered.add_candidate(&next, candidate(1, "/home/Library/Caches/Yarn"));
            let scan = gathered.take(&next);

            assert_eq!(scan.candidates, [candidate(1, "/home/Library/Caches/Yarn")]);
            assert_eq!(scan.holders, []);
        }

        #[test]
        fn another_scans_token_takes_nothing_and_matches_are_taken_once() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add_candidate(&token, candidate(0, "/home/.npm/_cacache"));

            assert_eq!(gathered.take(&CancelToken::new()), Scan::default());
            gathered.add_candidate(&token, candidate(0, "/home/.npm/_cacache"));
            assert_eq!(gathered.take(&token).candidates.len(), 1);
            assert_eq!(gathered.take(&token), Scan::default());
        }
    }

    mod positive {
        use fresher_core::{CancelToken, EntryKind};

        use super::{candidate, holder};
        use crate::gathered::Gathered;

        #[test]
        fn a_folder_holder_holds_its_sign() {
            let folder = holder(0, EntryKind::Folder);

            assert_eq!(folder.sign(), Some(&folder.sign));
        }

        #[test]
        fn the_candidates_are_handed_sorted_by_path_with_the_holders() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add_candidate(&token, candidate(4, "/home/.cargo/registry/src"));
            gathered.add_holder(&token, holder(1, EntryKind::Folder));
            gathered.add_candidate(&token, candidate(3, "/home/.cargo/registry/cache"));

            let scan = gathered.take(&token);

            let paths: Vec<_> = scan.candidates.iter().map(|candidate| candidate.row).collect();
            assert_eq!(paths, [3, 4]);
            assert_eq!(scan.holder(1), Some(&holder(1, EntryKind::Folder)));
        }
    }
}
