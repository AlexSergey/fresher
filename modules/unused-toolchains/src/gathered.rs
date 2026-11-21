//! The matches of one scan, gathered while the walk runs and decided in `finish`, once: matches come in no order, and
//! the pointers and pins of a version may be seen after it. Nothing here reads the disk.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryFacts, EntryKind, Inside, Sizes};

/// A match of a row's versions place: a folder the module may offer as one version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    /// The index of the row in the toolchains table.
    pub(crate) row: usize,
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
    pub(crate) sizes: Sizes,
    pub(crate) facts: EntryFacts,
    pub(crate) inside: Option<Inside>,
}

/// A match of one of a row's pointer places: the `.rustup` folder, an nvm alias file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PointerMatch {
    /// The index of the row in the toolchains table.
    pub(crate) row: usize,
    /// The index of the place among the row's pointers.
    pub(crate) place: usize,
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
    /// Which names of the place's `holds` the folder holds.
    pub(crate) holds: Vec<String>,
}

/// Everything one scan matched, as `finish` reads it: candidates and pointers sorted by path, pin folders by path
/// with the pin names each holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Matches {
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) pointers: Vec<PointerMatch>,
    pub(crate) pins: BTreeMap<PathBuf, BTreeSet<String>>,
}

/// The matches gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves
/// its matches behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Matches)>>,
}

impl Gathered {
    pub(crate) fn add_candidate(&self, token: &CancelToken, candidate: Candidate) {
        self.with(token, |matches| matches.candidates.push(candidate));
    }

    pub(crate) fn add_pointer(&self, token: &CancelToken, pointer: PointerMatch) {
        self.with(token, |matches| matches.pointers.push(pointer));
    }

    /// Notes that the folder `folder` holds the pin file `name`.
    pub(crate) fn add_pin(&self, token: &CancelToken, folder: PathBuf, name: &str) {
        self.with(token, |matches| {
            matches.pins.entry(folder).or_default().insert(name.to_owned());
        });
    }

    /// The matches gathered for the scan of `token`, candidates and pointers sorted by path; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Matches {
        let mut matches = match self.lock().take() {
            Some((kept, matches)) if kept.is_same(token) => matches,
            _ => Matches::default(),
        };
        matches.candidates.sort_by(|first, second| (&first.path, first.row).cmp(&(&second.path, second.row)));
        matches.pointers.sort_by(|first, second| {
            (&first.path, first.row, first.place).cmp(&(&second.path, second.row, second.place))
        });
        matches
    }

    /// Adds to the matches of the scan of `token`, dropping what an earlier scan left.
    fn with(&self, token: &CancelToken, add: impl FnOnce(&mut Matches)) {
        let mut kept = self.lock();
        match kept.as_mut() {
            Some((owner, matches)) if owner.is_same(token) => add(matches),
            _ => {
                let mut matches = Matches::default();
                add(&mut matches);
                *kept = Some((token.clone(), matches));
            }
        }
    }

    /// The lock, even after a panic while it was held: what it guards is plain lists.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Matches)>> {
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
            gathered.add_candidate(&cancelled, candidate(0, "/home/.rustup/toolchains/1.70.0-aarch64-apple-darwin"));
            gathered.add_pin(&cancelled, "/home/dev/app".into(), ".nvmrc");

            gathered.add_candidate(&next, candidate(1, "/home/.nvm/versions/node/v18.17.0"));

            let matches = gathered.take(&next);
            assert_eq!(matches.candidates, [candidate(1, "/home/.nvm/versions/node/v18.17.0")]);
            assert!(matches.pins.is_empty());
        }

        #[test]
        fn another_scans_token_takes_nothing_and_matches_are_taken_once() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add_candidate(&token, candidate(0, "/home/.rustup/toolchains/1.70.0-aarch64-apple-darwin"));

            assert_eq!(gathered.take(&CancelToken::new()).candidates, []);
            gathered.add_candidate(&token, candidate(0, "/home/.rustup/toolchains/1.70.0-aarch64-apple-darwin"));
            assert_eq!(gathered.take(&token).candidates.len(), 1);
            assert_eq!(gathered.take(&token).candidates, []);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{CancelToken, EntryKind};

        use super::candidate;
        use crate::gathered::{Gathered, PointerMatch};

        fn pointer(place: usize, path: &str) -> PointerMatch {
            PointerMatch { row: 1, place, path: path.into(), kind: EntryKind::File, holds: Vec::new() }
        }

        #[test]
        fn candidates_and_pointers_are_handed_sorted_by_path() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add_candidate(&token, candidate(1, "/home/.nvm/versions/node/v20.0.0"));
            gathered.add_candidate(&token, candidate(1, "/home/.nvm/versions/node/v18.0.0"));
            gathered.add_pointer(&token, pointer(1, "/home/.nvm/alias/lts/iron"));
            gathered.add_pointer(&token, pointer(0, "/home/.nvm/alias/default"));

            let matches = gathered.take(&token);

            let candidates: Vec<&str> = matches.candidates.iter().filter_map(|found| found.path.to_str()).collect();
            assert_eq!(candidates, ["/home/.nvm/versions/node/v18.0.0", "/home/.nvm/versions/node/v20.0.0"]);
            let pointers: Vec<usize> = matches.pointers.iter().map(|found| found.place).collect();
            assert_eq!(pointers, [0, 1]);
        }

        #[test]
        fn a_pin_folder_keeps_every_pin_name_it_holds() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add_pin(&token, "/home/dev/app".into(), ".tool-versions");
            gathered.add_pin(&token, "/home/dev/app".into(), ".nvmrc");

            let matches = gathered.take(&token);

            let names: Vec<&str> = matches.pins[&PathBuf::from("/home/dev/app")].iter().map(String::as_str).collect();
            assert_eq!(names, [".nvmrc", ".tool-versions"]);
        }
    }
}
