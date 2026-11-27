//! Folding one folder's listing: small files become a count and three sums; folders, large files and matched entries
//! are kept as their own records.

use std::ops::Range;

use fresher_core::{EntryKind, FolderListing, RawEntry, Sizes};

use crate::matcher::Hits;
use crate::matcher::entry::FolderFacts;
use crate::tree::access::NodeAccess;
use crate::tree::node::{Hit, Node, NodeIndex, Sums, time_bits};
use crate::tree::times::Times;

/// A file at or over this allocated size is kept as its own record (1 MiB, confirmed by plan 004/01).
pub(crate) const KEEP_THRESHOLD: u64 = 1 << 20;

/// The Finder writes `.DS_Store` when the user only looks at a folder, so it is not a change.
const DS_STORE: &[u8] = b".DS_Store";

/// One kept entry of a batch: where its name and hits lie in the batch's buffers, what the reader saw, and its access
/// for the scan's user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Kept {
    pub(crate) name: Range<usize>,
    pub(crate) hits: Range<usize>,
    pub(crate) entry: RawEntry,
    pub(crate) access: NodeAccess,
}

/// A folded file whose bytes may be shared: a hard link (more than one link) or a clone (a clone id). Its bytes are
/// counted once per group when the tree is frozen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SharedFile {
    pub(crate) device: u64,
    pub(crate) file_id: u64,
    pub(crate) links: u32,
    pub(crate) clone: Option<u64>,
    pub(crate) sizes: Sizes,
}

impl SharedFile {
    pub(crate) const fn of(entry: &RawEntry) -> Self {
        Self {
            device: entry.device,
            file_id: entry.file_id,
            links: entry.links,
            clone: entry.clone,
            sizes: entry.sizes,
        }
    }
}

/// What a worker sends for one folder: everything the aggregator needs to grow the tree, in a few flat buffers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FolderBatch {
    pub(crate) folder: NodeIndex,
    /// The plain small files and the links nobody matched.
    pub(crate) folded: Sums,
    /// The small files whose bytes may be shared.
    pub(crate) shared: Vec<SharedFile>,
    /// Folders, large files and matched entries, in listing order; its folders are the ones to queue.
    pub(crate) kept: Vec<Kept>,
    pub(crate) names: Vec<u8>,
    pub(crate) hits: Vec<Hit>,
    /// The hits of the folder itself (its marker interests), added by the walker.
    pub(crate) own_hits: Vec<Hit>,
    /// The folder's own times, from its parent's listing: recorded when it has hits of its own.
    pub(crate) own_times: Times,
    /// The newest change (the later of `modified` and `changed`) of the listing's files, links and special entries
    /// that were read, `.DS_Store` left out; `i64::MIN` for none.
    pub(crate) newest: i64,
    /// Some entry could not be read.
    pub(crate) incomplete: bool,
}

impl FolderBatch {
    pub(crate) const fn new(folder: NodeIndex) -> Self {
        Self {
            folder,
            folded: Sums::ZERO,
            shared: Vec::new(),
            kept: Vec::new(),
            names: Vec::new(),
            hits: Vec::new(),
            own_hits: Vec::new(),
            own_times: Times::NONE,
            newest: Node::NO_TIME,
            incomplete: false,
        }
    }

    pub(crate) fn name(&self, kept: &Kept) -> &[u8] {
        self.names.get(kept.name.clone()).unwrap_or_default()
    }

    pub(crate) fn hits_of(&self, kept: &Kept) -> &[Hit] {
        self.hits.get(kept.hits.clone()).unwrap_or_default()
    }
}

/// The time an entry last changed: the later of its `modified` and `changed`; `i64::MIN` when neither is known.
fn change_of(entry: &RawEntry) -> i64 {
    time_bits(entry.modified.max(entry.changed))
}

/// Folds the listing of `folder`. `hits` are the matcher's hits of each entry, in listing order; `facts` the
/// listing's facts, for the sibling masks; `user` the user a kept entry's access is read for. A file is kept when its allocated size is at least `threshold` or an
/// interest hit it; a small hard link or clone is folded and listed in `shared`. A link is kept (its own size) when
/// an interest hit it, otherwise folded as a zero-size entry. An entry that could not be read makes the batch
/// incomplete and counts for nothing.
pub(crate) fn fold(
    folder: NodeIndex,
    listing: &FolderListing,
    facts: &FolderFacts,
    hits: &[Hits],
    threshold: u64,
    user: Option<u32>,
) -> FolderBatch {
    let mut batch = FolderBatch::new(folder);
    for (position, (name, entry)) in listing.iter().enumerate() {
        let found = hits.get(position).map(Hits::as_slice).unwrap_or_default();
        if entry.error.is_some() {
            batch.incomplete = true;
            continue;
        }
        if entry.kind != EntryKind::Folder && name != DS_STORE {
            batch.newest = batch.newest.max(change_of(entry));
        }
        let keep = match entry.kind {
            EntryKind::Folder => true,
            EntryKind::File => !found.is_empty() || entry.sizes.allocated.bytes() >= threshold,
            EntryKind::Symlink | EntryKind::Other => !found.is_empty(),
        };
        if keep {
            keep_entry(&mut batch, name, entry, found, facts, user);
        } else if entry.kind == EntryKind::File && (entry.links > 1 || entry.clone.is_some()) {
            batch.shared.push(SharedFile::of(entry));
        } else {
            let sizes = if entry.kind == EntryKind::File { entry.sizes } else { Sizes::ZERO };
            let file = Sums::file(sizes);
            batch.folded = batch.folded.checked_add(file).unwrap_or_else(|| {
                batch.incomplete = true;
                batch.folded.saturating_add(file)
            });
        }
    }
    batch
}

fn keep_entry(
    batch: &mut FolderBatch,
    name: &[u8],
    entry: &RawEntry,
    found: &[u32],
    facts: &FolderFacts,
    user: Option<u32>,
) {
    let name_start = batch.names.len();
    batch.names.extend_from_slice(name);
    let hits_start = batch.hits.len();
    batch.hits.extend(found.iter().map(|&interest| Hit {
        interest,
        marker: None,
        siblings: facts.siblings_of(interest),
        holds: 0,
    }));
    batch.kept.push(Kept {
        name: name_start..batch.names.len(),
        hits: hits_start..batch.hits.len(),
        entry: *entry,
        access: NodeAccess::of(entry.access, user),
    });
}

#[cfg(test)]
#[path = "fold_tests.rs"]
mod tests;
