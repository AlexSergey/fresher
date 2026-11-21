//! The matches of one scan, gathered while the walk runs and decided in `finish`, once: the running apps are asked
//! for at most once a scan, and matches come in no order. Nothing here reads the disk.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryFacts, EntryKind, Inside, Sizes, Timestamp};

use crate::stores::OllamaInterest;

/// A match of a store's place: a folder the module may offer as one model, or an entry of an Ollama store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    /// The index of the row in the stores table.
    pub(crate) row: usize,
    /// For a match of an Ollama row, which of its six interests; `None` for any other row.
    pub(crate) ollama: Option<OllamaInterest>,
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
    pub(crate) sizes: Sizes,
    pub(crate) modified: Option<Timestamp>,
    pub(crate) accessed: Option<Timestamp>,
    pub(crate) facts: EntryFacts,
    /// Which names of the row's sign the folder holds.
    pub(crate) holds: Vec<String>,
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

    /// The candidates gathered for the scan of `token`, sorted by path then row; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Vec<Candidate> {
        let mut candidates = match self.lock().take() {
            Some((kept, candidates)) if kept.is_same(token) => candidates,
            _ => Vec::new(),
        };
        candidates.sort_by(|first, second| (&first.path, first.row).cmp(&(&second.path, second.row)));
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

    use crate::gathered::Candidate;

    fn candidate(row: usize, path: &str) -> Candidate {
        Candidate {
            row,
            ollama: None,
            path: path.into(),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            modified: None,
            accessed: None,
            facts: EntryFacts::REMOVABLE,
            holds: Vec::new(),
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
            gathered.add(&cancelled, candidate(0, "/home/.cache/huggingface/hub/models--gpt2"));

            gathered.add(&next, candidate(1, "/home/.lmstudio/models/a/b"));

            assert_eq!(gathered.take(&next), [candidate(1, "/home/.lmstudio/models/a/b")]);
        }

        #[test]
        fn another_scans_token_takes_nothing_and_candidates_are_taken_once() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, candidate(0, "/home/.cache/huggingface/hub/models--gpt2"));

            assert_eq!(gathered.take(&CancelToken::new()), []);
            gathered.add(&token, candidate(0, "/home/.cache/huggingface/hub/models--gpt2"));
            assert_eq!(gathered.take(&token).len(), 1);
            assert_eq!(gathered.take(&token), []);
        }
    }

    mod positive {
        use fresher_core::CancelToken;

        use super::candidate;
        use crate::gathered::Gathered;

        #[test]
        fn the_candidates_are_handed_sorted_by_path_then_row() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, candidate(2, "/home/.lmstudio/models/b/m"));
            gathered.add(&token, candidate(1, "/home/.lmstudio/models/a/m"));
            gathered.add(&token, candidate(0, "/home/.lmstudio/models/a/m"));

            let rows: Vec<usize> = gathered.take(&token).iter().map(|candidate| candidate.row).collect();

            assert_eq!(rows, [0, 1, 2]);
        }
    }
}
