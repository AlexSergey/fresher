//! The map by place: one page of a folder's children, largest first, each with the bytes the modules explain, opened
//! through the folders that are almost all one child.

use fresher_core::{ChildOrder, Descent, DiskState, NodeId, NodeInfo, NodeKind, PlaceRow, PlaceView};

use crate::storage::shares::Shares;

/// The share of a folder's allocated size, in percent, its largest child must hold for the view to open through it.
/// A chain of folders that each hold little else (`Library › Containers › com.example.app`) says nothing on its own:
/// the user wants the folder at its end.
const DESCENT_SHARE: u128 = 90;

/// The most folders one opening goes through, so a deep chain still shows where it leads within a few clicks back.
const DESCENT_LIMIT: usize = 8;

/// The page of `limit` rows from `offset` of the node `id` of `state`, with `shares` of that state. At `offset` 0 with
/// [`Descent::Through`] the view first goes down through every folder that is almost all one child; a next page
/// continues the folder it is given, and [`Descent::Stay`] opens the folder asked. A node that is not a folder opens
/// as itself, with no rows. `None` for a node of another state.
pub(crate) fn place_view(
    state: &dyn DiskState,
    shares: &Shares,
    id: NodeId,
    descent: Descent,
    offset: u32,
    limit: u32,
) -> Option<PlaceView> {
    if id.state() != state.id() {
        return None;
    }
    let opened = state.node(id)?;
    let mut path = vec![opened];
    if offset == 0 && descent == Descent::Through {
        while path.len() <= DESCENT_LIMIT {
            let Some(next) = path.last().and_then(|node| dominant_child(state, node)) else { break };
            path.push(next);
        }
    }
    let Some(last) = path.last().filter(|node| node.kind == NodeKind::Folder) else {
        return Some(PlaceView { path, rows: Vec::new(), offset, total: 0 });
    };
    let page = state.children(last.id, ChildOrder::BySize, offset, limit)?;
    let rows =
        page.nodes.into_iter().map(|node| PlaceRow { explained: shares.explained(state, node.id), node }).collect();
    Some(PlaceView { path, rows, offset: page.offset, total: page.total })
}

/// The child of the folder `node` the view opens through: its largest, when that is a scanned folder holding at least
/// [`DESCENT_SHARE`] percent of `node`'s allocated size.
fn dominant_child(state: &dyn DiskState, node: &NodeInfo) -> Option<NodeInfo> {
    if !is_open(node) || node.sizes.allocated.bytes() == 0 {
        return None;
    }
    let largest = state.children(node.id, ChildOrder::BySize, 0, 1)?.nodes.into_iter().next()?;
    let share = u128::from(largest.sizes.allocated.bytes()) * 100;
    let needed = u128::from(node.sizes.allocated.bytes()) * DESCENT_SHARE;
    (is_open(&largest) && share >= needed).then_some(largest)
}

/// A folder the scan went into: not unreadable, avoided or on another volume.
fn is_open(node: &NodeInfo) -> bool {
    node.kind == NodeKind::Folder && node.not_scanned.is_none()
}

#[cfg(test)]
#[path = "place_tests.rs"]
mod tests;
