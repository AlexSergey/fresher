//! What the reference walk saw of each anchored interest's place, from the folders it listed.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use fresher_core::{InterestSet, KnownFolders, Pattern, PlaceSeen};

/// What the reference walk saw of one folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Listing {
    /// Listed, every entry read.
    Read,
    /// Listed, an entry not read.
    Incomplete,
    /// Not listed: it could not be read.
    Unreadable,
    /// Not entered on purpose: avoided, another volume, another user's, the system's own items.
    NotEntered,
    /// Claimed whole: summed, not kept; its own listing is not known to the interests of its place.
    Whole,
}

/// The walked roots and every folder the walk saw below them, outside the folders claimed whole.
#[derive(Debug, Default)]
pub(crate) struct Listings {
    roots: Vec<PathBuf>,
    folders: HashMap<PathBuf, Listing>,
}

impl Listings {
    pub(crate) fn root(&mut self, root: &Path) {
        self.roots.push(root.to_path_buf());
    }

    pub(crate) fn saw(&mut self, folder: &Path, listing: Listing) {
        self.folders.insert(folder.to_path_buf(), listing);
    }

    /// What the walk saw of the place of every anchored interest of `interests` that has one, by interest: the
    /// pattern's folder without its last segment, when no segment but the last is `*`.
    pub(crate) fn places(&self, interests: &InterestSet, folders: &dyn KnownFolders) -> BTreeMap<usize, PlaceSeen> {
        let mut places = BTreeMap::new();
        for (index, interest) in interests.iter() {
            let Pattern::Under(folder, relative) = interest.pattern() else { continue };
            let Some(anchor) = folders.locate(*folder) else { continue };
            let segments: Vec<&str> = relative.split('/').collect();
            let Some((_, before)) = segments.split_last() else { continue };
            if before.contains(&"*") {
                continue;
            }
            let place = before.iter().fold(anchor, |path, name| path.join(name));
            places.insert(index, self.seen(&place));
        }
        places
    }

    /// Walks the place's path down from the deepest root that holds it.
    fn seen(&self, place: &Path) -> PlaceSeen {
        let root = self.roots.iter().filter(|root| place.starts_with(root)).max_by_key(|root| root.as_os_str().len());
        let Some(root) = root else { return PlaceSeen::NotScanned };
        let mut folder = root.clone();
        let mut names = place.strip_prefix(root).unwrap_or(place).iter();
        loop {
            let Some(&listing) = self.folders.get(&folder) else { return PlaceSeen::Absent };
            let name = names.next();
            match (listing, name) {
                (Listing::NotEntered, _) => return PlaceSeen::NotScanned,
                (Listing::Unreadable | Listing::Incomplete | Listing::Whole, _) => return PlaceSeen::Unread,
                (Listing::Read, None) => return PlaceSeen::Read,
                (Listing::Read, Some(name)) => folder.push(name),
            }
        }
    }
}
