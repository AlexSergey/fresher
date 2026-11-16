//! Every way an item leaves its place: to the Trash, emptied, deleted at the single deletion point, or by its tool.
//! Each takes a `Verified` (consumed, so one check serves one act) and an `Act`, which only a confirmed plan lends;
//! nothing here is reachable from outside the crate.

mod empty;
mod permanent;
mod tool;
mod trash;

use std::fs;
use std::path::Path;

use fresher_core::{ItemIdentity, ItemOutcome, TrashedItem};

pub(crate) use empty::{Way, empty};
pub(crate) use permanent::{delete, delete_entry};
pub(crate) use tool::run_tool;
pub(crate) use trash::to_trash;

use crate::guard::identity;

/// How one removal ended: its outcome, what went to the Trash, and a tool's exit status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Removed {
    pub(crate) outcome: ItemOutcome,
    pub(crate) trashed: Vec<TrashedItem>,
    pub(crate) status: Option<i32>,
}

impl Removed {
    pub(crate) const fn ended(outcome: ItemOutcome) -> Self {
        Self { outcome, trashed: Vec::new(), status: None }
    }
}

/// The identity of what is at `path` now, looked at without following a link at its end.
fn identity_at(path: &Path) -> Option<ItemIdentity> {
    fs::symlink_metadata(path).ok().map(|metadata| identity::of(&metadata))
}
