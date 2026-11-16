//! The engine's read-only queries over the disk state of the last committed scan.
//!
//! Each query takes the state's `Arc` under the results lock and lets the lock go before it reads: a long query never
//! blocks a scan from starting or committing, and a state replaced by a new scan lives until its last reader drops it.

use std::sync::Arc;

use fresher_core::{ChildOrder, CommandError, DiskState, FindingsQuery, NodeId, NodeInfo, NodePage, UnreadablePlace};

use crate::Engine;

/// The most nodes one query answers, like a page of findings.
const MAX_LIMIT: u32 = FindingsQuery::MAX_LIMIT;

impl Engine {
    /// The scan roots of the last state, in the order scanned.
    pub fn disk_roots(&self) -> Result<Vec<NodeInfo>, CommandError> {
        let state = self.last_state()?;
        Ok(state.roots().into_iter().filter_map(|id| state.node(id)).collect())
    }

    /// One node of the last state; a node of an older state is `StaleNode`.
    pub fn disk_node(&self, id: NodeId) -> Result<NodeInfo, CommandError> {
        let state = self.state_of(id)?;
        state.node(id).ok_or(CommandError::StaleNode)
    }

    /// One page of the children of `id`, in `order`; at most 500 a page.
    pub fn disk_children(
        &self,
        id: NodeId,
        order: ChildOrder,
        offset: u32,
        limit: u32,
    ) -> Result<NodePage, CommandError> {
        let state = self.state_of(id)?;
        state.children(id, order, offset, limit.min(MAX_LIMIT)).ok_or(CommandError::StaleNode)
    }

    /// The largest files of the last state, the largest first; at most 500.
    pub fn largest_files(&self, limit: u32) -> Result<Vec<NodeInfo>, CommandError> {
        Ok(self.last_state()?.largest_files(limit.min(MAX_LIMIT)))
    }

    /// The folders and large files whose name contains `query`, case ignored, the largest first; at most 500.
    pub fn search_names(&self, query: &str, limit: u32) -> Result<Vec<NodeInfo>, CommandError> {
        Ok(self.last_state()?.search(query, limit.min(MAX_LIMIT)))
    }

    /// The top-most folders the last scan did not read, with why and how many folders each hides; at most 500.
    pub fn unreadable_places(&self, limit: u32) -> Result<Vec<UnreadablePlace>, CommandError> {
        Ok(self.last_state()?.unreadable(limit.min(MAX_LIMIT)))
    }

    /// The last committed state; `NoScan` before any.
    fn last_state(&self) -> Result<Arc<dyn DiskState>, CommandError> {
        self.disk_state()?.ok_or(CommandError::NoScan)
    }

    /// The last state when `id` is one of its nodes' ids; `StaleNode` when it is of another state.
    fn state_of(&self, id: NodeId) -> Result<Arc<dyn DiskState>, CommandError> {
        let state = self.last_state()?;
        if id.state() == state.id() { Ok(state) } else { Err(CommandError::StaleNode) }
    }
}

#[cfg(test)]
#[path = "disk_tests.rs"]
mod tests;
