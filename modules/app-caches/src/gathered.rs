//! The matches of one scan, gathered while the walk runs and decided in `finish`, once: the installed and the running
//! apps are asked for at most once a scan, and matches come in no order. Nothing here reads the disk.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryFacts, EntryKind, Inside, Sizes};

/// What one interest of the module is: a row of `places.toml`, or one cache of a row of `apps.toml`, by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Source {
    Place { row: usize },
    App { row: usize, cache: usize },
}

/// A match of an interest: an entry the module may offer as one app's cache.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    /// The interest's source: the row of a table it matched.
    pub(crate) source: Source,
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
    pub(crate) sizes: Sizes,
    pub(crate) facts: EntryFacts,
    pub(crate) inside: Option<Inside>,
}

/// The candidates gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish`
/// leaves its candidates behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Vec<Candidate>)>>,
}

impl Gathered {
    /// Keeps `candidate` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add(&self, token: &CancelToken, candidate: Candidate) {
        let mut kept = self.lock();
        match kept.as_mut() {
            Some((owner, candidates)) if owner.is_same(token) => candidates.push(candidate),
            _ => *kept = Some((token.clone(), vec![candidate])),
        }
    }

    /// The candidates gathered for the scan of `token`, sorted by path then source; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Vec<Candidate> {
        let mut candidates = match self.lock().take() {
            Some((kept, candidates)) if kept.is_same(token) => candidates,
            _ => Vec::new(),
        };
        candidates.sort_by(|first, second| (&first.path, first.source).cmp(&(&second.path, second.source)));
        candidates
    }

    /// The lock, even after a panic while it was held: what it guards is a plain list.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Vec<Candidate>)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{EntryFacts, EntryKind, Inside, Sizes};

    use crate::gathered::{Candidate, Source};

    fn candidate(row: usize, path: &str) -> Candidate {
        Candidate {
            source: Source::Place { row },
            path: path.into(),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            facts: EntryFacts::REMOVABLE,
            inside: Some(Inside::EMPTY),
        }
    }

    mod negative {
        use fresher_core::CancelToken;

        use super::candidate;
        use crate::gathered::Gathered;

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add(&cancelled, candidate(0, "/home/Library/Caches/com.example.Editor"));

            gathered.add(&next, candidate(0, "/home/Library/Caches/com.example.Viewer"));

            assert_eq!(gathered.take(&next), [candidate(0, "/home/Library/Caches/com.example.Viewer")]);
        }

        #[test]
        fn another_scans_token_takes_nothing_and_candidates_are_taken_once() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, candidate(0, "/home/Library/Caches/com.example.Editor"));

            assert_eq!(gathered.take(&CancelToken::new()), []);
            gathered.add(&token, candidate(0, "/home/Library/Caches/com.example.Editor"));
            assert_eq!(gathered.take(&token).len(), 1);
            assert_eq!(gathered.take(&token), []);
        }
    }

    mod positive {
        use fresher_core::CancelToken;

        use super::candidate;
        use crate::gathered::{Gathered, Source};

        #[test]
        fn the_candidates_are_handed_sorted_by_path_then_source() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, candidate(2, "/home/Library/Caches/com.example.B"));
            gathered.add(&token, candidate(1, "/home/Library/Caches/com.example.A"));
            gathered.add(&token, candidate(0, "/home/Library/Caches/com.example.A"));

            let sources: Vec<Source> = gathered.take(&token).iter().map(|candidate| candidate.source).collect();

            assert_eq!(sources, [Source::Place { row: 0 }, Source::Place { row: 1 }, Source::Place { row: 2 }]);
        }
    }
}
