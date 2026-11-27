//! Entry points for the hand-run measurements (`crates/e2e/tests/storage_budget.rs`, `largest_budget.rs`): they reach
//! crate-private parts. Not part of the API.

use std::mem::size_of;
use std::path::Path;

use fresher_core::{ByteSize, DiskState, FolderRow, FoldersPage, FoldersQuery, ItemRule, MAX_PAGE, Timestamp};

use crate::largest::folders::{Listed, folders_page, left_alone};
use crate::largest::{HomeItems, Item, Items, with_bare_ground};
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

/// The Folders list of a state and its first page, built as `Engine::largest_folders` builds them.
#[derive(Debug)]
pub struct FoldersBench {
    listed: Vec<Listed>,
    page: FoldersPage,
}

impl FoldersBench {
    /// The folders below the node at `home` of at least `min` large that nothing changed in since `before`, by the
    /// engine's rule with no `Library` or Trash under the home, over a ground that ignores and protects nothing, and
    /// the page of the first `MAX_PAGE`. Nothing when the state does not hold `home`.
    pub fn build(state: &dyn DiskState, home: &Path, min: ByteSize, before: Timestamp) -> Self {
        let Some(root) = state.find(home) else {
            let page = crate::largest::folders::empty_folders(0, min, 0);
            return Self { listed: Vec::new(), page };
        };
        with_bare_ground(state, home, |ground| {
            let listed = left_alone(ground, root, &HomeItems::new(None, None), min, before);
            let sizes = state.node(root).map(|node| node.sizes).unwrap_or_default();
            let page = folders_page(&listed, FoldersQuery { offset: 0, limit: MAX_PAGE }, ground, sizes, min, 0);
            Self { listed, page }
        })
    }

    /// How many folders are listed.
    pub fn len(&self) -> usize {
        self.listed.len()
    }

    /// Whether no folder is listed.
    pub fn is_empty(&self) -> bool {
        self.listed.is_empty()
    }

    /// The bytes the list and its page hold on the heap: the list, each folder's path and name, and the rows.
    pub fn heap_bytes(&self) -> usize {
        let each: usize = self.listed.iter().map(|(node, path, _)| path.capacity() + node.name.len()).sum();
        let rows: usize = self.page.rows.iter().map(|row| row.place.capacity() + row.node.name.len()).sum();
        self.listed.capacity() * size_of::<Listed>() + each + self.page.rows.capacity() * size_of::<FolderRow>() + rows
    }
}
