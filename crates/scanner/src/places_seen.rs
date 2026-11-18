//! What the walk saw of each anchored interest's place, read from its tree once the walk is done.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use fresher_core::PlaceSeen;

use crate::matcher::trie::Segment;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// The flags of a folder not entered on purpose.
const NOT_ENTERED: [Flag; 3] = [Flag::Avoided, Flag::OtherVolume, Flag::OthersFolder];

/// What `tree` shows of the place of every interest of `anchors` (its compiled absolute segments) that has one, by
/// interest. Only the folders on the way and the place's own flags count, never `Partial`: an unreadable folder below
/// the place is inside a match, not the place's.
pub(crate) fn places_seen(tree: &Tree, anchors: &[(u32, Vec<Segment>)]) -> BTreeMap<usize, PlaceSeen> {
    anchors
        .iter()
        .filter_map(|(interest, segments)| {
            let interest = usize::try_from(*interest).ok()?;
            Some((interest, seen(tree, &place_of(segments)?)))
        })
        .collect()
}

/// The place of an interest: its absolute path without the last segment; `None` when a segment before the last is a
/// `*`, plain or shaped (the interest matches in more than one folder).
fn place_of(segments: &[Segment]) -> Option<PathBuf> {
    let (_, folders) = segments.split_last()?;
    let mut place = PathBuf::from("/");
    for segment in folders {
        let Segment::Literal(name) = segment else { return None };
        place.push(OsStr::from_bytes(name));
    }
    Some(place)
}

/// Walks the place's path down from the root that holds it, one folder at a time.
fn seen(tree: &Tree, place: &Path) -> PlaceSeen {
    let Some((mut folder, below)) = root_of(tree, place) else { return PlaceSeen::NotScanned };
    let mut names = below.components();
    loop {
        let Some(node) = tree.node(folder) else { return PlaceSeen::Unread };
        let flags = node.flags;
        if NOT_ENTERED.into_iter().any(|flag| flags.has(flag)) {
            return PlaceSeen::NotScanned;
        }
        if flags.unreadable().is_some() || flags.has(Flag::Incomplete) {
            return PlaceSeen::Unread;
        }
        // Claimed whole: nothing inside is a node, so neither the place's own listing nor the next name is known.
        if flags.has(Flag::Opaque) {
            return PlaceSeen::Unread;
        }
        let Some(name) = names.next() else { return PlaceSeen::Read };
        // Folded for the cache: the place itself was read in full, but the next name below it is not known.
        if flags.has(Flag::Collapsed) {
            return PlaceSeen::Unread;
        }
        let Component::Normal(name) = name else { return PlaceSeen::Unread };
        let child = tree.children(folder).find(|&child| tree.name(child) == name.as_bytes());
        match child {
            Some(child) if tree.node(child).is_some_and(|child| child.flags.has(Flag::Folder)) => folder = child,
            _ => return PlaceSeen::Absent,
        }
    }
}

/// The root that holds `place` (the deepest, should two), and the place's path below it.
fn root_of<'a>(tree: &Tree, place: &'a Path) -> Option<(NodeIndex, &'a Path)> {
    tree.roots()
        .iter()
        .filter_map(|root| {
            let below = place.strip_prefix(OsStr::from_bytes(tree.name(root.node))).ok()?;
            Some((root.node, below))
        })
        .min_by_key(|(_, below)| below.components().count())
}

#[cfg(test)]
#[path = "places_seen_tests.rs"]
mod tests;
