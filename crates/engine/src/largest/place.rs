//! The Folders page: the place view of one folder of the home, with the way back to the home and a judgement per row.

use std::path::PathBuf;

use fresher_core::{CommandError, Descent, NodeId, NodeInfo, NodeKind, OwnPlace, OwnRow, kind_of};

use crate::largest::Ground;
use crate::storage::place::place_view;

/// The page of `limit` rows from `offset` of `folder`, a folder of the home that opens (the home itself always does),
/// without the picks a cleanup removed. `StaleNode` for a node not in the state, `NotOwnItem` for a folder outside the
/// home, one that does not open or one a cleanup removed.
pub(crate) fn own_place(
    ground: &Ground<'_>,
    folder: NodeId,
    offset: u32,
    limit: u32,
) -> Result<OwnPlace, CommandError> {
    let state = ground.state;
    let node = state.node(folder).ok_or(CommandError::StaleNode)?;
    let path = state.path(folder).ok_or(CommandError::StaleNode)?;
    if !path.starts_with(ground.home) {
        return Err(CommandError::NotOwnItem);
    }
    if ground.is_gone(&path) || (path != ground.home && !ground.judge_reached(&node, &path).1) {
        return Err(CommandError::NotOwnItem);
    }
    let gone = ground.gone_children(&path);
    // Removed rows leave the list before it is paged, so the offsets the screen asks for stay consistent.
    let (from, take) = if gone == 0 { (offset, limit) } else { (0, offset.saturating_add(limit).saturating_add(gone)) };
    let view = place_view(state, ground.shares, folder, Descent::Stay, from, take).ok_or(CommandError::StaleNode)?;
    let skip = if gone == 0 { 0 } else { usize::try_from(offset).unwrap_or(usize::MAX) };
    let rows = view
        .rows
        .into_iter()
        .filter_map(|row| {
            let path = state.path(row.node.id).filter(|path| !ground.is_gone(path))?;
            let (offer, opens) = ground.judge(&row.node, &path);
            let folder = matches!(row.node.kind, NodeKind::Folder | NodeKind::Collapsed);
            Some(OwnRow {
                kind: kind_of(&row.node.name, folder),
                explained: row.explained,
                offer,
                opens,
                picked: ground.picked(&path),
                node: row.node,
            })
        })
        .skip(skip)
        .take(usize::try_from(limit).unwrap_or(usize::MAX))
        .collect();
    Ok(OwnPlace { path: way(ground, &path), rows, offset, total: view.total.saturating_sub(gone) })
}

/// The folders from the home (first) to `path` (last).
fn way(ground: &Ground<'_>, path: &std::path::Path) -> Vec<NodeInfo> {
    let mut at = PathBuf::from(ground.home);
    let mut way: Vec<NodeInfo> = ground.state.find(&at).and_then(|id| ground.state.node(id)).into_iter().collect();
    for part in path.strip_prefix(ground.home).into_iter().flat_map(std::path::Path::components) {
        at.push(part);
        way.extend(ground.state.find(&at).and_then(|id| ground.state.node(id)));
    }
    way
}
