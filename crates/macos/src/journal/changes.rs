//! The events of a replay in the core's words: the folders that changed, the subtrees to read whole, or a gap.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

use fresher_core::{Changes, JournalGap, JournalPosition};

use crate::ffi::fsevents::{Event, HISTORY_DONE};

/// `kFSEventStreamEventFlag…` values of `<CoreServices/.../FSEvents.h>`.
const MUST_SCAN_SUBDIRS: u32 = 0x1;
const USER_DROPPED: u32 = 0x2;
const KERNEL_DROPPED: u32 = 0x4;
const EVENT_IDS_WRAPPED: u32 = 0x8;
const ROOT_CHANGED: u32 = 0x20;

/// Each event's folder, once; a gap flag on any event is the answer instead (the first one met).
pub(super) fn from_events(events: &[Event], end: JournalPosition) -> Result<Changes, JournalGap> {
    let mut folders = BTreeSet::new();
    let mut subtrees = BTreeSet::new();
    for event in events {
        if event.flags & (USER_DROPPED | KERNEL_DROPPED) != 0 {
            return Err(JournalGap::Dropped);
        }
        if event.flags & EVENT_IDS_WRAPPED != 0 {
            return Err(JournalGap::Wrapped);
        }
        if event.flags & ROOT_CHANGED != 0 {
            return Err(JournalGap::RootChanged);
        }
        if event.flags & HISTORY_DONE != 0 {
            continue;
        }
        let path = folder(&event.path);
        if event.flags & MUST_SCAN_SUBDIRS != 0 {
            subtrees.insert(path);
        } else {
            folders.insert(path);
        }
    }
    Ok(Changes { folders: folders.into_iter().collect(), subtrees: subtrees.into_iter().collect(), end })
}

/// The event's path without the `/` a folder event may end with (the root `/` stays).
fn folder(path: &[u8]) -> PathBuf {
    let trimmed = path.strip_suffix(b"/").filter(|rest| !rest.is_empty()).unwrap_or(path);
    PathBuf::from(OsStr::from_bytes(trimmed))
}

#[cfg(test)]
mod tests {
    use fresher_core::JournalPosition;

    use crate::ffi::fsevents::Event;

    pub(super) const END: JournalPosition = JournalPosition { journal: 7, cursor: 100 };

    pub(super) fn event(path: &str, flags: u32) -> Event {
        Event { path: path.as_bytes().to_vec(), flags, id: 1 }
    }

    mod negative {
        use fresher_core::JournalGap;

        use super::{END, event};
        use crate::journal::changes::{
            EVENT_IDS_WRAPPED, KERNEL_DROPPED, MUST_SCAN_SUBDIRS, ROOT_CHANGED, USER_DROPPED, from_events,
        };

        #[test]
        fn a_dropped_event_is_a_gap() {
            for flags in [USER_DROPPED | MUST_SCAN_SUBDIRS, KERNEL_DROPPED | MUST_SCAN_SUBDIRS] {
                let events = [event("/a/", 0), event("/", flags)];

                assert_eq!(from_events(&events, END), Err(JournalGap::Dropped));
            }
        }

        #[test]
        fn wrapped_ids_and_a_changed_root_are_gaps() {
            assert_eq!(from_events(&[event("/", EVENT_IDS_WRAPPED)], END), Err(JournalGap::Wrapped));
            assert_eq!(from_events(&[event("/a", ROOT_CHANGED)], END), Err(JournalGap::RootChanged));
        }

        #[test]
        fn the_end_of_the_history_is_not_a_change() {
            let changes = from_events(&[event("", crate::ffi::fsevents::HISTORY_DONE)], END).unwrap();

            assert!(changes.folders.is_empty() && changes.subtrees.is_empty());
        }
    }

    mod positive {
        use std::path::PathBuf;

        use super::{END, event};
        use crate::journal::changes::{MUST_SCAN_SUBDIRS, from_events};

        #[test]
        fn folders_are_deduplicated_and_trimmed() {
            let events = [event("/a/b/", 0x1000), event("/a/b", 0x100), event("/a/", 0x200), event("/", 0x400)];

            let changes = from_events(&events, END).unwrap();

            assert_eq!(changes.folders, [PathBuf::from("/"), PathBuf::from("/a"), PathBuf::from("/a/b")]);
            assert_eq!(changes.subtrees, [] as [PathBuf; 0]);
            assert_eq!(changes.end, END);
        }

        #[test]
        fn a_subtree_to_scan_whole_is_reported_apart() {
            let changes = from_events(&[event("/a/b/", MUST_SCAN_SUBDIRS), event("/a/", 0)], END).unwrap();

            assert_eq!(changes.subtrees, [PathBuf::from("/a/b")]);
            assert_eq!(changes.folders, [PathBuf::from("/a")]);
        }
    }
}
