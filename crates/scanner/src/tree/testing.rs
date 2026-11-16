//! Test helpers of the tree: a canonical listing, so two trees built in different orders compare equal when they hold
//! the same thing.

use std::path::{Path, PathBuf};

use fresher_core::{Completeness, FolderIdentity};

use crate::tree::arena::Tree;
use crate::tree::node::{Hit, NodeFlags, NodeIndex, Sums};

/// One node as a test sees it: everything but its index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shape {
    pub(crate) path: PathBuf,
    pub(crate) flags: NodeFlags,
    pub(crate) sums: Sums,
    pub(crate) folders: u32,
    pub(crate) completeness: Completeness,
    pub(crate) hits: Vec<Hit>,
}

/// Every node depth-first, children in list order.
pub(crate) fn canonical(tree: &Tree) -> Vec<Shape> {
    let mut shapes = Vec::with_capacity(tree.len());
    let mut stack: Vec<NodeIndex> = tree.roots().iter().rev().map(|root| root.node).collect();
    while let Some(index) = stack.pop() {
        let node = tree.node(index).unwrap();
        shapes.push(Shape {
            path: tree.path(index).unwrap(),
            flags: node.flags,
            sums: node.sums(),
            folders: node.folders,
            completeness: tree.completeness(index).unwrap(),
            hits: tree.hits(index).to_vec(),
        });
        let children: Vec<_> = tree.children(index).collect();
        stack.extend(children.into_iter().rev());
    }
    shapes
}

/// A tree with one root at `path`.
pub(crate) fn rooted(path: &str) -> (Tree, NodeIndex) {
    let mut tree = Tree::new();
    let root = tree.add_root(Path::new(path), FolderIdentity { device: 1, file_id: 2 }).unwrap();
    (tree, root)
}

/// Sums of one file count and three sizes.
pub(crate) const fn sums(files: u32, logical: u64, allocated: u64, reclaimable: u64) -> Sums {
    Sums { files, logical, allocated, reclaimable }
}
