//! Entry points for the hand-run measurements (`crates/e2e/tests/storage_budget.rs`, `largest_budget.rs`): they reach
//! crate-private parts. Not part of the API.

use std::mem::size_of;
use std::path::Path;

use fresher_core::{ByteSize, DiskState, ItemRule};

use crate::largest::{HomeItems, Item, Items};
use crate::settings::Ignored;
use crate::storage::shares::Shares;
use crate::withheld::Withheld;

/// The shares of a state, built as the first map query after a commit builds them, with nothing ignored.
#[derive(Debug)]
pub struct SharesBench {
    shares: Shares,
}

impl SharesBench {
    /// Builds the shares of `state`, nothing ignored.
    pub fn build(state: &dyn DiskState) -> Self {
        Self { shares: Shares::of(state, &Ignored::default(), &Withheld::offered()) }
    }

    /// The bytes the shares hold on the heap.
    pub fn heap_bytes(&self) -> usize {
        self.shares.heap_bytes()
    }
}

/// The Files list's items of a state, built as the first Largest query after a commit builds them.
#[derive(Debug)]
pub struct LargestBench {
    items: Items,
}

impl LargestBench {
    /// The items of `state` below the node at `home` at least `min` large, by the engine's rule with no `Library` or
    /// Trash under the home; nothing when the state does not hold `home`.
    pub fn build(state: &dyn DiskState, home: &Path, min: ByteSize) -> Self {
        Self::build_with(state, home, &HomeItems::new(None, None), min)
    }

    /// The same with `rule` in place of the engine's: a rule that calls more nodes items fills the list to its cap.
    pub fn build_with(state: &dyn DiskState, home: &Path, rule: &dyn ItemRule, min: ByteSize) -> Self {
        let items = state.find(home).map_or_else(
            || Items { min, list: Vec::new(), truncated: false },
            |home| Items::of(state, home, rule, min),
        );
        Self { items }
    }

    /// How many items are kept.
    pub fn len(&self) -> usize {
        self.items.list.len()
    }

    /// Whether no item is kept.
    pub fn is_empty(&self) -> bool {
        self.items.list.is_empty()
    }

    /// The bytes the kept items hold on the heap: the list and each item's path.
    pub fn heap_bytes(&self) -> usize {
        let paths: usize = self.items.list.iter().map(|item| item.path.capacity()).sum();
        self.items.list.capacity() * size_of::<Item>() + paths
    }
}
