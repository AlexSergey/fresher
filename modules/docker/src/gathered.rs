//! The matches of one scan, gathered while the walk runs and decided in `finish`: per runtime, whether its socket was
//! seen and its disk file's sizes. Nothing here reads the disk.

use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, EntryKind, Sizes};

use crate::runtimes::Watched;

/// What one match showed of a runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Spotted {
    /// The socket of the runtime of row `row`.
    Socket { row: usize },
    /// The disk file of the runtime of row `row`, with its sizes.
    Disk { row: usize, sizes: Sizes },
}

impl Spotted {
    /// What a match of `kind` at a place `watched` of row `row` shows: a socket only as a special entry, a disk file
    /// only as a file; a link or a folder at either place is not seen.
    pub(crate) fn of(row: usize, watched: Watched, kind: EntryKind, sizes: Sizes) -> Option<Self> {
        match (watched, kind) {
            (Watched::Socket, EntryKind::Other) => Some(Self::Socket { row }),
            (Watched::Disk, EntryKind::File) => Some(Self::Disk { row, sizes }),
            _ => None,
        }
    }
}

/// What the matches of one scan showed, taken by `finish`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Seen {
    spotted: Vec<Spotted>,
}

impl Seen {
    /// Whether the walk saw the socket of row `row`.
    pub(crate) fn socket(&self, row: usize) -> bool {
        self.spotted.contains(&Spotted::Socket { row })
    }

    /// The sizes of the disk file of row `row`, when the walk saw it.
    pub(crate) fn disk(&self, row: usize) -> Option<Sizes> {
        self.spotted.iter().find_map(|spotted| match spotted {
            Spotted::Disk { row: of, sizes } if *of == row => Some(*sizes),
            _ => None,
        })
    }
}

/// What was spotted for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves its
/// matches behind, and the next scan, with another token, drops them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Vec<Spotted>)>>,
}

impl Gathered {
    /// Keeps `spotted` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add(&self, token: &CancelToken, spotted: Spotted) {
        let mut kept = self.lock();
        match kept.as_mut() {
            Some((owner, list)) if owner.is_same(token) => list.push(spotted),
            _ => *kept = Some((token.clone(), vec![spotted])),
        }
    }

    /// What was spotted for the scan of `token`; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> Seen {
        match self.lock().take() {
            Some((kept, spotted)) if kept.is_same(token) => Seen { spotted },
            _ => Seen::default(),
        }
    }

    /// The lock, even after a panic while it was held: what it guards is a plain list.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Vec<Spotted>)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{ByteSize, Sizes};

    fn sizes() -> Sizes {
        Sizes::complete(ByteSize::new(100), ByteSize::new(40))
    }

    mod negative {
        use fresher_core::{CancelToken, EntryKind};

        use super::sizes;
        use crate::gathered::{Gathered, Spotted};
        use crate::runtimes::Watched;

        #[test]
        fn a_link_or_a_folder_at_either_place_is_not_seen() {
            for kind in [EntryKind::Symlink, EntryKind::Folder, EntryKind::File] {
                assert_eq!(Spotted::of(0, Watched::Socket, kind, sizes()), None, "{kind:?}");
            }
            for kind in [EntryKind::Symlink, EntryKind::Folder, EntryKind::Other] {
                assert_eq!(Spotted::of(0, Watched::Disk, kind, sizes()), None, "{kind:?}");
            }
        }

        #[test]
        fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add(&cancelled, Spotted::Socket { row: 0 });
            gathered.add(&next, Spotted::Socket { row: 1 });

            let seen = gathered.take(&next);

            assert!(!seen.socket(0));
            assert!(seen.socket(1));
            assert!(!gathered.take(&next).socket(1));
            assert!(!gathered.take(&CancelToken::new()).socket(1));
        }
    }

    mod positive {
        use fresher_core::{CancelToken, EntryKind};

        use super::sizes;
        use crate::gathered::{Gathered, Spotted};
        use crate::runtimes::Watched;

        #[test]
        fn a_socket_and_a_disk_file_are_seen_per_row() {
            let (gathered, token) = (Gathered::default(), CancelToken::new());
            for spotted in [
                Spotted::of(0, Watched::Socket, EntryKind::Other, sizes()),
                Spotted::of(1, Watched::Disk, EntryKind::File, sizes()),
            ] {
                gathered.add(&token, spotted.unwrap());
            }

            let seen = gathered.take(&token);

            assert!(seen.socket(0));
            assert!(!seen.socket(1));
            assert_eq!(seen.disk(0), None);
            assert_eq!(seen.disk(1), Some(sizes()));
        }
    }
}
