//! Check 7: the item is still the one the plan holds — the same volume, file id and kind — read through its handle.

use std::fs::Metadata;
use std::os::unix::fs::MetadataExt as _;

use fresher_core::{EntryKind, ItemIdentity, SkipReason};

use crate::guard::Purpose;

/// The identity of the entry `metadata` describes; a link is its own entry.
pub(crate) fn of(metadata: &Metadata) -> ItemIdentity {
    ItemIdentity::new(metadata.dev(), metadata.ino(), kind(metadata))
}

pub(crate) fn kind(metadata: &Metadata) -> EntryKind {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Folder
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

/// Check 7: `found` against the plan's identity. A socket, a pipe or a device is never acted on, whatever was
/// confirmed. A rehearsal of a new plan has none yet and takes the one found; an act without one acts on nothing.
pub(super) fn check(
    planned: Option<ItemIdentity>,
    found: ItemIdentity,
    purpose: Purpose,
) -> Result<ItemIdentity, SkipReason> {
    if found.kind() == EntryKind::Other {
        return Err(SkipReason::KindChanged);
    }
    let Some(planned) = planned else {
        return match purpose {
            Purpose::Rehearse => Ok(found),
            Purpose::Act => Err(SkipReason::CheckFailed),
        };
    };
    if planned.kind() != found.kind() {
        Err(SkipReason::KindChanged)
    } else if planned != found {
        Err(SkipReason::Replaced)
    } else {
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{EntryKind, ItemIdentity};

    const FILE: ItemIdentity = ItemIdentity::new(1, 2, EntryKind::File);

    mod negative {
        use fresher_core::{EntryKind, ItemIdentity, SkipReason};

        use super::FILE;
        use crate::guard::Purpose;
        use crate::guard::identity::check;

        #[test]
        fn another_file_id_is_replaced() {
            let other = ItemIdentity::new(1, 3, EntryKind::File);

            assert_eq!(check(Some(FILE), other, Purpose::Act), Err(SkipReason::Replaced));
        }

        #[test]
        fn another_volume_is_replaced() {
            let other = ItemIdentity::new(9, 2, EntryKind::File);

            assert_eq!(check(Some(FILE), other, Purpose::Rehearse), Err(SkipReason::Replaced));
        }

        #[test]
        fn another_kind_has_changed_kind() {
            let folder = ItemIdentity::new(1, 2, EntryKind::Folder);
            let link = ItemIdentity::new(1, 7, EntryKind::Symlink);

            assert_eq!(check(Some(FILE), folder, Purpose::Act), Err(SkipReason::KindChanged));
            assert_eq!(check(Some(FILE), link, Purpose::Act), Err(SkipReason::KindChanged));
        }

        #[test]
        fn a_socket_a_pipe_or_a_device_is_refused_whatever_was_confirmed() {
            let other = ItemIdentity::new(1, 2, EntryKind::Other);

            assert_eq!(check(Some(FILE), other, Purpose::Act), Err(SkipReason::KindChanged));
            assert_eq!(check(Some(other), other, Purpose::Act), Err(SkipReason::KindChanged));
            assert_eq!(check(None, other, Purpose::Rehearse), Err(SkipReason::KindChanged));
        }

        #[test]
        fn an_act_without_an_identity_acts_on_nothing() {
            assert_eq!(check(None, FILE, Purpose::Act), Err(SkipReason::CheckFailed));
        }
    }

    mod positive {
        use super::FILE;
        use crate::guard::Purpose;
        use crate::guard::identity::check;

        #[test]
        fn the_same_item_passes() {
            assert_eq!(check(Some(FILE), FILE, Purpose::Act), Ok(FILE));
        }

        #[test]
        fn a_rehearsal_of_a_new_plan_takes_the_identity_found() {
            assert_eq!(check(None, FILE, Purpose::Rehearse), Ok(FILE));
        }
    }
}
