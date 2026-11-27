//! Growing the tree by one folder's batch: the only way the tree grows during a walk.

use fresher_core::EntryKind;

use crate::tree::TreeError;
use crate::tree::arena::{Shared, Tree};
use crate::tree::fold::{FolderBatch, SharedFile};
use crate::tree::node::{Flag, NodeIndex, Sums};
use crate::tree::times::Times;

impl Tree {
    /// Adds the batch's folded bytes to its folder and its kept entries as the folder's children, with their hits; a
    /// node with hits keeps its entry's times.
    /// `created` receives the node of each kept entry, in batch order: the walker queues the folders among them.
    pub(crate) fn apply(&mut self, batch: &FolderBatch, created: &mut Vec<NodeIndex>) -> Result<(), TreeError> {
        let folder = batch.folder;
        self.add_own(folder, batch.folded)?;
        self.join_newest(folder, batch.newest)?;
        if batch.incomplete {
            self.mark(folder, Flag::Incomplete)?;
        }
        self.add_hits(folder, &batch.own_hits)?;
        if !batch.own_hits.is_empty() {
            self.set_times(folder, batch.own_times)?;
        }
        for file in &batch.shared {
            self.place_file(folder, file, true)?;
        }
        for kept in &batch.kept {
            let entry = &kept.entry;
            let node = self.add_child(folder, batch.name(kept), entry.kind)?;
            self.set_modified(node, entry.modified)?;
            self.set_access(node, kept.access)?;
            let hits = batch.hits_of(kept);
            self.add_hits(node, hits)?;
            if !hits.is_empty() {
                self.set_times(node, Times::of(entry))?;
            }
            match entry.kind {
                EntryKind::File => {
                    self.place_file(node, &SharedFile::of(entry), false)?;
                }
                EntryKind::Symlink | EntryKind::Other => {
                    self.add_own(node, Sums { files: 0, ..Sums::file(entry.sizes) })?;
                }
                EntryKind::Folder => {}
            }
            created.push(node);
        }
        Ok(())
    }

    /// Puts a file's bytes in the tree: its own part on `holder`, a hard link's bytes in the link groups and a
    /// clone's shared part in the clone groups, each counted once at `freeze`. `folded` counts the file at `holder`
    /// (a kept file's node counts itself).
    pub(super) fn place_file(&mut self, holder: NodeIndex, file: &SharedFile, folded: bool) -> Result<(), TreeError> {
        let SharedFile { device, file_id, links, clone, sizes } = *file;
        let (logical, allocated, reclaimable) =
            (sizes.logical.bytes(), sizes.allocated.bytes(), sizes.reclaimable.bytes());
        let shared = if clone.is_some() { allocated.saturating_sub(reclaimable) } else { 0 };
        let own = Sums { files: 0, logical, allocated: allocated - shared, reclaimable };
        let mut counted = Sums { files: u32::from(folded), ..Sums::ZERO };
        if links > 1 {
            self.add_shared(Shared::HardLink, device, file_id, holder, own)?;
        } else {
            counted = Sums { files: counted.files, ..own };
        }
        if let (Some(clone), true) = (clone, shared > 0) {
            self.add_shared(Shared::Clone, device, clone, holder, Sums { allocated: shared, ..Sums::ZERO })?;
        }
        self.add_own(holder, counted)
    }
}
