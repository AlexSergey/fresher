//! The first-level entries of Downloads of one scan, gathered while the walk runs and decided in `finish`: a `.part`
//! is decided with the file beside it, and matches come in no order. Nothing here reads the disk.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryKind, Inside, Match, Sizes, Timestamp};

use crate::unfinished::Form;

/// The interest that takes every first-level file.
pub(crate) const FILES: usize = 0;
/// The interest that sums the bundles whole.
pub(crate) const BUNDLES: usize = 1;

/// A first-level entry of Downloads: a file of interest 0 or a bundle of interest 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) kind: Form,
    /// A bundle's: everything inside it.
    pub(crate) sizes: Sizes,
    pub(crate) modified: Option<Timestamp>,
    pub(crate) changed: Option<Timestamp>,
    /// What a bundle holds; `None` for a file.
    pub(crate) inside: Option<Inside>,
    /// Which of `Info.plist` and `Contents` a bundle itself holds.
    pub(crate) holds: Vec<String>,
}

impl Entry {
    /// The entry `matched` is, when it is a file of interest 0 or a folder of interest 1, right in `downloads`, with
    /// a name of text; anything else (a link, an `Other` entry, a folder of interest 0, a match below the first
    /// level) is no entry.
    pub(crate) fn first_level(matched: &Match, downloads: &Path) -> Option<Self> {
        let kind = match (matched.interest, matched.kind) {
            (FILES, EntryKind::File) => Form::File,
            (BUNDLES, EntryKind::Folder) => Form::Bundle,
            _ => return None,
        };
        if matched.path.parent() != Some(downloads) {
            return None;
        }
        let name = matched.path.file_name()?.to_str()?.to_owned();
        Some(Self {
            name,
            path: matched.path.clone(),
            kind,
            sizes: matched.sizes,
            modified: matched.modified,
            changed: matched.changed,
            inside: matched.inside,
            holds: matched.holds.clone(),
        })
    }

    /// The newest of `modified` and `changed`; `None` when either is unknown.
    pub(crate) fn newest_change(&self) -> Option<Timestamp> {
        self.modified.zip(self.changed).map(|(modified, changed)| modified.max(changed))
    }

    /// Whether the bundle itself holds `name`.
    pub(crate) fn holds(&self, name: &str) -> bool {
        self.holds.iter().any(|held| held == name)
    }
}

/// The entries gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves
/// its entries behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Vec<Entry>)>>,
}

impl Gathered {
    /// Keeps `entry` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add(&self, token: &CancelToken, entry: Entry) {
        let mut scan = self.lock();
        match scan.as_mut() {
            Some((kept, entries)) if kept.is_same(token) => entries.push(entry),
            _ => *scan = Some((token.clone(), vec![entry])),
        }
    }

    /// The entries gathered for the scan of `token`, sorted by name then kind; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Vec<Entry> {
        let mut entries = match self.lock().take() {
            Some((kept, entries)) if kept.is_same(token) => entries,
            _ => Vec::new(),
        };
        entries.sort_by(|first, second| (&first.name, first.kind).cmp(&(&second.name, second.kind)));
        entries.dedup();
        entries
    }

    /// The lock, even after a panic while it was held: what it guards is a plain list.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Vec<Entry>)>> {
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

    const DOWNLOADS: &str = "/home/Downloads";

    mod negative {
        use std::path::Path;

        use fresher_core::{CancelToken, EntryKind};

        use super::{DOWNLOADS, matched};
        use crate::gathered::{BUNDLES, Entry, FILES, Gathered};

        #[test]
        fn a_link_an_other_entry_or_a_folder_of_the_files_interest_is_no_entry() {
            let downloads = Path::new(DOWNLOADS);
            for stray in [
                matched(FILES, "/home/Downloads/a.part", EntryKind::Symlink),
                matched(FILES, "/home/Downloads/a.part", EntryKind::Other),
                matched(FILES, "/home/Downloads/photos", EntryKind::Folder),
                matched(BUNDLES, "/home/Downloads/b.download", EntryKind::File),
                matched(2, "/home/Downloads/a.part", EntryKind::File),
            ] {
                assert_eq!(Entry::first_level(&stray, downloads), None, "{stray:?}");
            }
        }

        #[test]
        fn a_match_below_the_first_level_is_dropped() {
            let deep = matched(FILES, "/home/Downloads/photos/big.mov", EntryKind::File);

            assert_eq!(Entry::first_level(&deep, Path::new(DOWNLOADS)), None);
        }

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            let entry = |name: &str| {
                let path = format!("{DOWNLOADS}/{name}");
                Entry::first_level(&matched(FILES, &path, EntryKind::File), Path::new(DOWNLOADS)).unwrap()
            };
            gathered.add(&cancelled, entry("a.part"));

            assert_eq!(gathered.take(&next), []);

            gathered.add(&cancelled, entry("a.part"));
            gathered.add(&next, entry("b.part"));
            let names: Vec<String> = gathered.take(&next).into_iter().map(|entry| entry.name).collect();
            assert_eq!(names, ["b.part"]);
            assert_eq!(gathered.take(&next), []);
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{CancelToken, EntryKind};

        use super::{DOWNLOADS, matched};
        use crate::gathered::{BUNDLES, Entry, FILES, Gathered};
        use crate::unfinished::Form;

        #[test]
        fn the_entries_of_one_scan_come_back_sorted_by_name() {
            let (gathered, scan) = (Gathered::default(), CancelToken::new());
            for (interest, name, kind) in [
                (FILES, "z.dmg", EntryKind::File),
                (BUNDLES, "b.download", EntryKind::Folder),
                (FILES, "a.part", EntryKind::File),
            ] {
                let path = format!("{DOWNLOADS}/{name}");
                gathered.add(&scan, Entry::first_level(&matched(interest, &path, kind), Path::new(DOWNLOADS)).unwrap());
            }

            let entries = gathered.take(&scan);

            let names: Vec<(&str, Form)> = entries.iter().map(|entry| (entry.name.as_str(), entry.kind)).collect();
            assert_eq!(names, [("a.part", Form::File), ("b.download", Form::Bundle), ("z.dmg", Form::File)]);
        }
    }
}
