//! Matching one entry, and the facts of one folder's listing.

use fresher_core::{EntryKind, FolderListing};

use crate::matcher::state::TrieState;
use crate::matcher::{Hits, Matcher};

/// What one folder's listing holds for the interests: the marker interests whose marker file it holds (they match
/// the folder itself), and per interest with sibling names, which of them are present (as a mask, bit `i` for the
/// interest's `i`-th sibling). Both sorted by interest; empty, with no allocation, for most folders.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FolderFacts {
    markers: Vec<u32>,
    siblings: Vec<(u32, u64)>,
}

impl FolderFacts {
    pub(crate) fn markers(&self) -> &[u32] {
        &self.markers
    }

    /// The sibling mask of `interest` for an entry of this folder.
    pub(crate) fn siblings_of(&self, interest: u32) -> u64 {
        sibling_mask(&self.siblings, interest)
    }

    /// Every interest's sibling mask, sorted by interest: what a subfolder's job carries for the folder's own marker
    /// hits.
    pub(crate) fn siblings(&self) -> &[(u32, u64)] {
        &self.siblings
    }
}

/// The mask of `interest` in `masks`, sorted by interest; 0 when it has none.
pub(crate) fn sibling_mask(masks: &[(u32, u64)], interest: u32) -> u64 {
    masks
        .binary_search_by_key(&interest, |&(found, _)| found)
        .ok()
        .and_then(|position| masks.get(position))
        .map_or(0, |&(_, mask)| mask)
}

impl Matcher {
    /// The interests the entry `name` of kind `kind`, in a folder at `state`, matches by its name, its extension or
    /// its anchored path. A folder name matches a folder or a link; an extension a file or a link; an anchored path
    /// any entry. Markers are matched on the folder's own listing ([`Matcher::folder`]).
    pub(crate) fn entry(&self, state: TrieState, name: &[u8], kind: EntryKind) -> Hits {
        let mut hits = Hits::default();
        if kind != EntryKind::File
            && let Some(found) = self.folder_names.get(name)
        {
            hits.extend_from_slice(found.as_slice());
        }
        if kind != EntryKind::Folder
            && !self.extensions.is_empty()
            && let Some(found) = extension(name).and_then(|extension| self.extensions.get(extension))
        {
            hits.extend_from_slice(found.as_slice());
        }
        if state.is_live()
            && let Some(node) = self.trie_node(self.child_state(state, name))
        {
            hits.extend_from_slice(node.ends.as_slice());
        }
        hits.sort_dedup();
        hits
    }

    /// The markers and sibling names `listing` holds, in one pass over its names. A marker counts only as a regular
    /// file; an entry that could not be read counts as absent.
    pub(crate) fn folder(&self, listing: &FolderListing) -> FolderFacts {
        let mut facts = FolderFacts::default();
        if self.markers.is_empty() && self.siblings.is_empty() {
            return facts;
        }
        for (name, entry) in listing.iter() {
            if entry.error.is_some() {
                continue;
            }
            if entry.kind == EntryKind::File
                && let Some(found) = self.markers.get(name)
            {
                facts.markers.extend_from_slice(found.as_slice());
            }
            for slot in self.siblings.get(name).map(Vec::as_slice).unwrap_or_default() {
                facts.siblings.push((slot.interest, 1 << slot.bit));
            }
        }
        facts.markers.sort_unstable();
        facts.markers.dedup();
        facts.siblings.sort_unstable_by_key(|&(interest, _)| interest);
        facts.siblings.dedup_by(|later, earlier| {
            let same = later.0 == earlier.0;
            if same {
                earlier.1 |= later.1;
            }
            same
        });
        facts
    }
}

/// The extension of a file name as `std::path::Path::extension` sees it: after the last dot, none for a name whose
/// only dot leads it.
fn extension(name: &[u8]) -> Option<&[u8]> {
    let dot = name.iter().rposition(|&byte| byte == b'.')?;
    if dot == 0 { None } else { name.get(dot + 1..) }
}
