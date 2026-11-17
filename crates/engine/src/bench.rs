//! Entry points for the hand-run measurements (`crates/e2e/tests/storage_budget.rs`): they reach crate-private parts.
//! Not part of the API.

use fresher_core::DiskState;

use crate::settings::Ignored;
use crate::storage::shares::Shares;

/// The shares of a state, built as the first map query after a commit builds them, with nothing ignored.
#[derive(Debug)]
pub struct SharesBench {
    shares: Shares,
}

impl SharesBench {
    /// Builds the shares of `state`, nothing ignored.
    pub fn build(state: &dyn DiskState) -> Self {
        Self { shares: Shares::of(state, &Ignored::default()) }
    }

    /// The bytes the shares hold on the heap.
    pub fn heap_bytes(&self) -> usize {
        self.shares.heap_bytes()
    }
}
