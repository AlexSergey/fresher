//! What a folder claimed whole holds, gathered by the walk in the same pass that sums its sizes: how recently anything
//! in it changed or was read, and whether anything in it is special, someone else's or unread.

use crate::{EntryAccess, EntryKind, RawEntry, Timestamp};

/// The inside of a folder claimed whole. A tool can set a file's modification time back, and a folder's own times do
/// not move when a file deep inside is written: the newest change is taken over everything inside, status changes
/// included. Access times are lazy and folders' move when they are listed: only files' count, and only as a reason
/// to keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent facts the walk gathers, each a plain yes or no")]
pub struct Inside {
    /// The newest of `modified` and `changed` over the folder and everything inside it.
    pub newest_change: Option<Timestamp>,
    /// The newest `accessed` of the files inside (folders' access times move when the scan lists them).
    pub newest_file_access: Option<Timestamp>,
    /// A socket, a pipe or a device somewhere inside.
    pub special: bool,
    /// An entry inside that is not the user's, or carries a lock or a system flag.
    pub foreign: bool,
    /// An entry inside named as a program's lock (`*.lock`, `*.pid`, `LCK..*`, `.#*`): something in the folder may be
    /// in use.
    pub lock_named: bool,
    /// Every folder inside was read.
    pub complete: bool,
}

impl Inside {
    /// The inside of an empty folder that was read: nothing changed, nothing special, nothing foreign.
    pub const EMPTY: Self = Self {
        newest_change: None,
        newest_file_access: None,
        special: false,
        foreign: false,
        lock_named: false,
        complete: true,
    };

    /// Adds one entry inside, as the reader saw it; `user` is the user the scan is for (`None`: nobody, so every
    /// entry is foreign). An entry that could not be read makes the inside incomplete and adds nothing else.
    pub fn join(&mut self, entry: &RawEntry, user: Option<u32>) {
        if entry.error.is_some() {
            self.complete = false;
            return;
        }
        self.join_change(entry.modified.max(entry.changed));
        if entry.kind == EntryKind::File {
            self.newest_file_access = self.newest_file_access.max(entry.accessed);
        }
        self.special |= entry.kind == EntryKind::Other;
        self.foreign |= is_foreign(entry.access, user);
    }

    /// Counts the name of one entry inside, readable or not.
    pub fn join_name(&mut self, name: &[u8]) {
        self.lock_named |= is_lock_file_name(name);
    }

    /// Counts a change at `time`: the folder's own, which is not inside it.
    pub fn join_change(&mut self, time: Option<Timestamp>) {
        self.newest_change = self.newest_change.max(time);
    }

    /// Adds the inside of another part of the same folder.
    pub fn merge(&mut self, other: &Self) {
        self.join_change(other.newest_change);
        self.newest_file_access = self.newest_file_access.max(other.newest_file_access);
        self.special |= other.special;
        self.foreign |= other.foreign;
        self.lock_named |= other.lock_named;
        self.complete &= other.complete;
    }
}

/// The suffixes and prefixes of the names programs give their lock files, compared ignoring ASCII case.
const LOCK_SUFFIXES: [&[u8]; 2] = [b".lock", b".pid"];
const LOCK_PREFIXES: [&[u8]; 2] = [b"LCK..", b".#"];

/// Whether `name` is a program's lock file name: `*.lock`, `*.pid`, `LCK..*` or an editor's `.#*`.
fn is_lock_file_name(name: &[u8]) -> bool {
    let ends = |suffix: &[u8]| {
        name.len()
            .checked_sub(suffix.len())
            .and_then(|start| name.get(start..))
            .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
    };
    let starts = |prefix: &[u8]| name.get(..prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(prefix));
    LOCK_SUFFIXES.into_iter().any(ends) || LOCK_PREFIXES.into_iter().any(starts)
}

/// Not owned by `user`, or locked by the user or the system, or guarded by the system's integrity protection.
const fn is_foreign(access: EntryAccess, user: Option<u32>) -> bool {
    let owned = matches!((access.owner, user), (Some(owner), Some(user)) if owner == user);
    !owned || access.user_immutable || access.system_immutable || access.restricted
}

#[cfg(test)]
mod tests {
    use crate::{EntryAccess, EntryKind, RawEntry, Sizes, Timestamp};

    const ME: Option<u32> = Some(501);

    #[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
    fn at(secs: u64) -> Option<Timestamp> {
        Some(Timestamp::from_secs(secs))
    }

    fn entry(kind: EntryKind, modified: u64, changed: u64, accessed: u64) -> RawEntry {
        RawEntry {
            kind,
            device: 1,
            file_id: 2,
            links: 1,
            sizes: Sizes::ZERO,
            clone: None,
            modified: at(modified),
            changed: at(changed),
            accessed: at(accessed),
            cloud_only: false,
            access: EntryAccess { owner: ME, mode: 0o644, ..EntryAccess::default() },
            error: None,
        }
    }

    mod negative {
        use super::{ME, at, entry};
        use crate::{EntryAccess, EntryKind, Inside, ReadError};

        #[test]
        fn an_old_modification_time_does_not_hide_a_new_status_change() {
            let mut inside = Inside::EMPTY;

            inside.join(&entry(EntryKind::File, 1_546_300_800, 1_700_000_000, 1_546_300_800), ME);

            assert_eq!(inside.newest_change, at(1_700_000_000));
        }

        #[test]
        fn a_folders_access_time_never_counts() {
            let mut inside = Inside::EMPTY;

            inside.join(&entry(EntryKind::Folder, 10, 10, 1_700_000_000), ME);
            inside.join(&entry(EntryKind::Symlink, 10, 10, 1_700_000_000), ME);
            inside.join(&entry(EntryKind::Other, 10, 10, 1_700_000_000), ME);

            assert_eq!(inside.newest_file_access, None);
        }

        #[test]
        fn an_unreadable_entry_makes_it_incomplete_and_adds_nothing_else() {
            let mut inside = Inside::EMPTY;
            let unreadable = crate::RawEntry {
                error: Some(ReadError::NoPermission),
                access: EntryAccess::default(),
                ..entry(EntryKind::Other, 99, 99, 99)
            };

            inside.join(&unreadable, ME);

            assert_eq!(inside, Inside { complete: false, ..Inside::EMPTY });
        }

        #[test]
        fn a_pipe_a_socket_or_a_device_is_special() {
            let mut inside = Inside::EMPTY;

            inside.join(&entry(EntryKind::Other, 1, 1, 1), ME);

            assert!(inside.special);
        }

        #[test]
        fn an_entry_of_someone_else_or_with_a_lock_or_a_system_flag_is_foreign() {
            let mine = entry(EntryKind::File, 1, 1, 1);
            let theirs = EntryAccess { owner: Some(0), ..mine.access };
            let unknown = EntryAccess { owner: None, ..mine.access };
            let locked = EntryAccess { user_immutable: true, ..mine.access };
            let system = EntryAccess { system_immutable: true, ..mine.access };
            let restricted = EntryAccess { restricted: true, ..mine.access };

            for access in [theirs, unknown, locked, system, restricted] {
                let mut inside = Inside::EMPTY;
                inside.join(&crate::RawEntry { access, ..mine }, ME);
                assert!(inside.foreign, "{access:?}");
            }
            let mut nobody = Inside::EMPTY;
            nobody.join(&mine, None);
            assert!(nobody.foreign);
        }

        #[test]
        fn a_merge_keeps_every_doubt() {
            let mut inside = Inside::EMPTY;

            inside.merge(&Inside { special: true, foreign: true, lock_named: true, complete: false, ..Inside::EMPTY });
            inside.merge(&Inside::EMPTY);

            assert!(inside.special && inside.foreign && inside.lock_named && !inside.complete);
        }

        #[test]
        fn a_name_like_but_not_a_lock_files_is_not_counted() {
            let mut inside = Inside::EMPTY;

            for name in ["lockfile", "x.locked", "LCK", "LCK.x", "#x#", ".x#", "pid", "a.pidx", ""] {
                inside.join_name(name.as_bytes());
            }

            assert!(!inside.lock_named);
        }
    }

    mod positive {
        use super::{ME, at, entry};
        use crate::{EntryKind, Inside};

        #[test]
        fn an_empty_inside_is_complete_and_knows_no_time() {
            let inside = Inside::EMPTY;

            assert_eq!((inside.newest_change, inside.newest_file_access), (None, None));
            assert!(inside.complete && !inside.special && !inside.foreign && !inside.lock_named);
        }

        #[test]
        fn a_lock_files_name_inside_is_counted() {
            for name in ["app.lock", "server.pid", "LCK..ttyS0", ".#notes.txt", "DB.LOCK"] {
                let mut inside = Inside::EMPTY;

                inside.join_name(name.as_bytes());

                assert!(inside.lock_named, "{name}");
            }
        }

        #[test]
        fn the_newest_change_and_the_newest_file_access_win() {
            let mut inside = Inside::EMPTY;

            inside.join(&entry(EntryKind::File, 300, 100, 50), ME);
            inside.join(&entry(EntryKind::Folder, 100, 400, 900), ME);
            inside.join(&entry(EntryKind::File, 100, 200, 70), ME);

            assert_eq!(inside.newest_change, at(400));
            assert_eq!(inside.newest_file_access, at(70));
            assert!(inside.complete && !inside.special && !inside.foreign);
        }

        #[test]
        fn a_merge_takes_the_newest_times() {
            let mut left = Inside { newest_change: at(5), newest_file_access: at(9), ..Inside::EMPTY };

            left.merge(&Inside { newest_change: at(7), newest_file_access: at(3), ..Inside::EMPTY });
            left.join_change(at(6));

            assert_eq!((left.newest_change, left.newest_file_access), (at(7), at(9)));
        }
    }
}
