//! A folder claimed whole: its totals are kept, nothing inside it becomes a node.

use crate::tree::TreeError;
use crate::tree::arena::Tree;
use crate::tree::fold::SharedFile;
use crate::tree::node::{Flag, NodeIndex, Sums};

impl Tree {
    /// Stores the totals of the subtree below `folder` — the plain files' `sums`, the `folders` inside it and the
    /// files whose bytes may be shared — and flags it opaque. Its insides were read to the end but are not kept.
    pub(crate) fn apply_opaque(
        &mut self,
        folder: NodeIndex,
        sums: Sums,
        folders: u32,
        shared: &[SharedFile],
    ) -> Result<(), TreeError> {
        if self.children(folder).next().is_some() {
            return Err(TreeError::NotOpaque);
        }
        self.add_own(folder, sums)?;
        self.add_hidden_folders(folder, folders)?;
        for file in shared {
            self.place_file(folder, file, true)?;
        }
        self.mark(folder, Flag::Opaque)
    }
}
