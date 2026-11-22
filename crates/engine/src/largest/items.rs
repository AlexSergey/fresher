//! The items of one state and minimum size: computed once by the scanner's items query and kept beside the state.

use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{ByteSize, DiskState, ItemKind, ItemRule, MAX_ITEMS, NodeId, NodeKind, Timestamp, kind_of};

use crate::error::{EngineError, lock};

/// One item of the Files list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) id: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) kind: ItemKind,
    pub(crate) modified: Option<Timestamp>,
    /// The folder directly under the home that holds the item; `None` for an item directly in the home.
    pub(crate) top: Option<NodeId>,
}

/// The items of one state at least `min` large, the largest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Items {
    pub(crate) min: ByteSize,
    pub(crate) list: Vec<Item>,
    /// More than [`MAX_ITEMS`] were found: the smallest were left out.
    pub(crate) truncated: bool,
}

/// The items kept beside a state: the last minimum's, replaced when the minimum changes, forgotten with the state.
pub(crate) type ItemsSlot = Arc<Mutex<Option<Arc<Items>>>>;

impl Items {
    /// The items of `state` below `home` that `rule` calls items, at least `min` large. Nothing for a home of another
    /// state.
    pub(crate) fn of(state: &dyn DiskState, home: NodeId, rule: &dyn ItemRule, min: ByteSize) -> Self {
        let found = state.items(home, rule, min, MAX_ITEMS.saturating_add(1)).unwrap_or_default();
        let kept = usize::try_from(MAX_ITEMS).unwrap_or(usize::MAX);
        let truncated = found.len() > kept;
        let home_path = state.path(home);
        let items = found
            .into_iter()
            .take(kept)
            .filter_map(|id| {
                let node = state.node(id)?;
                let path = state.path(id)?;
                let folder = matches!(node.kind, NodeKind::Folder | NodeKind::Collapsed);
                let top = home_path.as_deref().and_then(|home| top_folder(state, home, &path));
                Some(Item { id, kind: kind_of(&node.name, folder), modified: node.modified, top, path })
            })
            .collect();
        Self { min, list: items, truncated }
    }

    /// The items `slot` keeps for `min`, or those `build` makes, kept in their place.
    pub(crate) fn cached(
        slot: &Mutex<Option<Arc<Self>>>,
        min: ByteSize,
        build: impl FnOnce() -> Self,
    ) -> Result<Arc<Self>, EngineError> {
        let mut kept = lock(slot)?;
        if let Some(items) = kept.as_ref().filter(|items| items.min == min) {
            return Ok(Arc::clone(items));
        }
        let items = Arc::new(build());
        *kept = Some(Arc::clone(&items));
        Ok(items)
    }
}

/// The folder directly under `home` that holds `path`, when `path` is deeper than that.
fn top_folder(state: &dyn DiskState, home: &Path, path: &Path) -> Option<NodeId> {
    let mut inside = path.strip_prefix(home).ok()?.components();
    let first = inside.next()?;
    inside.next()?;
    match first {
        Component::Normal(name) => state.find(&home.join(name)),
        _ => None,
    }
}
