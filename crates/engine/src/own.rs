//! A row of the Largest screen the user ticks, made a finding of the core's own category `core.own-file` (High): held
//! apart from the scan's findings, so the plan, the confirmation, the cleaner, the journal and put back take it as any
//! other finding. The engine never picks a row itself; a pick is judged again when a plan is prepared and confirmed.

use std::path::Path;

use fresher_core::{
    CommandError, Finding, FindingId, NewFinding, NodeId, NodeInfo, PlanItem, Removability, Removal, Subject, own_file,
    own_file_reason, own_file_title,
};

use crate::Engine;
use crate::engine::Shared;
use crate::error::{EngineError, lock};
use crate::largest::pickable;
use crate::results::Committed;

impl Engine {
    /// Makes the row `node` of the last state a finding the user can review and remove: refused `Busy` while a scan or
    /// a cleanup runs, `NoScan` before any scan, `StaleNode` for a node of another state, and `NotOwnItem` unless the
    /// row is offered now (as Files and Folders judge it, with the ignore list and the removed picks as they are now).
    /// While High is withheld every pick is refused with `HighWithheld` (a pick is always High).
    /// The same row answers the same finding; a new one takes the session's next free id.
    pub fn pick_own_item(&self, node: NodeId) -> Result<Finding, CommandError> {
        // Held to the end: no scan or cleanup starts while the row is judged and kept.
        let machine = lock(&self.shared.machine)?;
        machine.idle()?;
        if !self.shared.high.offers(own_file().danger()) {
            return Err(CommandError::HighWithheld);
        }
        let committed = self.committed_for_largest()?;
        let state = &*committed.state;
        if node.state() != state.id() {
            return Err(CommandError::StaleNode);
        }
        let info = state.node(node).ok_or(CommandError::StaleNode)?;
        let path = state.path(node).ok_or(CommandError::StaleNode)?;
        let home = pickable(&self.shared, &committed, &info, &path).ok_or(CommandError::NotOwnItem)?;
        let mut results = lock(&self.shared.results)?;
        if results.state().is_none_or(|kept| kept.id() != state.id()) {
            return Err(CommandError::StaleNode);
        }
        if let Some(picked) = results.own_by_path(&path) {
            return Ok(picked.clone());
        }
        let id = results.take_id().ok_or(EngineError::FindingIdsExhausted)?;
        let finding = own_finding(id, &home, &path, &info);
        results.pick(finding.clone());
        Ok(finding)
    }
}

/// The picks of the last state that are still offered now, in id order: those a plan may take.
pub(crate) fn offered_picks(shared: &Shared) -> Result<Vec<Finding>, CommandError> {
    let Some(committed) = lock(&shared.results)?.committed() else { return Ok(Vec::new()) };
    Ok(committed.own.iter().filter(|finding| still_offered(shared, &committed, finding)).cloned().collect())
}

/// `PlanGone` when an item of the core's own category is not a pick still offered now.
pub(crate) fn check_picks(shared: &Shared, items: &[PlanItem]) -> Result<(), CommandError> {
    let own = own_file().id();
    let picked: Vec<FindingId> = items.iter().filter(|item| item.category() == own).map(PlanItem::finding).collect();
    if picked.is_empty() {
        return Ok(());
    }
    let offered = offered_picks(shared)?;
    let all_offered = picked.iter().all(|id| offered.iter().any(|finding| finding.id() == *id));
    if all_offered { Ok(()) } else { Err(CommandError::PlanGone) }
}

fn still_offered(shared: &Shared, committed: &Committed, finding: &Finding) -> bool {
    let Subject::Path(path) = finding.finding().subject() else { return false };
    let state = &*committed.state;
    let Some(node) = state.find(path).and_then(|id| state.node(id)) else { return false };
    pickable(shared, committed, &node, path).is_some()
}

/// The finding of the row `node` at `path`, picked as `id`: the core's own category, keyed by its path from `home`,
/// moved to the Trash, `Removable` (the cleaner's rehearsal reads the disk and refuses what it must).
fn own_finding(id: FindingId, home: &Path, path: &Path, node: &NodeInfo) -> Finding {
    let from_home = path.strip_prefix(home).unwrap_or(path);
    let new = NewFinding::new(
        &own_file(),
        format!("own/{}", from_home.to_string_lossy()),
        Subject::Path(path.to_path_buf()),
        node.sizes,
        own_file_title(&node.name),
        own_file_reason(node.kind),
        Removal::MoveToTrash(path.to_path_buf()),
    );
    Finding::new(id, new, Removability::Removable)
}

#[cfg(test)]
#[path = "own_tests.rs"]
mod tests;
