//! Test helpers of the matcher: a small walk driven by the matcher, compared with the reference walk on the same
//! fixture tree.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use fresher_core::{
    CancelToken, Claim, DiskWalk, EntryAccess, EntryKind, FolderListing, Interest, InterestSet, ModuleId, Pattern,
    RawEntry, Sizes, WalkMode, WalkRequest,
};
use fresher_testing::{FixtureTree, NaiveWalk};

use crate::matcher::Matcher;
use crate::matcher::compile::under_home;
use crate::matcher::state::TrieState;

/// One match as both walks report it: the interest, the path, the sibling names present, and whether the match
/// carries a marker check.
pub(crate) type Found = (usize, PathBuf, Vec<String>, bool);

pub(crate) fn interest(pattern: Pattern, claim: Claim, siblings: &[&str]) -> Interest {
    Interest::new(pattern, claim, siblings.iter().map(|&name| name.into()).collect()).unwrap()
}

pub(crate) fn set_of(interests: &[Interest]) -> InterestSet {
    let mut set = InterestSet::new();
    set.add(ModuleId::new("demo").unwrap(), interests);
    set
}

pub(crate) fn compile(interests: &InterestSet, home: &Path) -> Matcher {
    Matcher::compile(interests, &under_home(home))
}

/// A listing of `entries`, every one readable.
pub(crate) fn listing(entries: &[(&[u8], EntryKind)]) -> FolderListing {
    let mut listing = FolderListing::new();
    for &(name, kind) in entries {
        listing.push(name, raw(kind)).unwrap();
    }
    listing
}

pub(crate) const fn raw(kind: EntryKind) -> RawEntry {
    RawEntry {
        kind,
        device: 1,
        file_id: 1,
        links: 1,
        sizes: Sizes::ZERO,
        clone: None,
        modified: None,
        cloud_only: false,
        access: EntryAccess { owner: None, mode: 0, user_immutable: false, system_immutable: false, restricted: false },
        error: None,
    }
}

/// The reference walk's matches on `tree`, home at its root, sorted.
pub(crate) fn reference(tree: &FixtureTree, interests: &InterestSet) -> Vec<Found> {
    let roots = [tree.root().to_path_buf()];
    let request = WalkRequest { roots: &roots, interests, avoid: &[], mode: WalkMode::Full };
    let mut found = Vec::new();
    NaiveWalk::new(tree.root())
        .walk(
            &request,
            &CancelToken::new(),
            &mut |matched| found.push((matched.interest, matched.path, matched.siblings, matched.marker.is_some())),
            &mut |_| {},
        )
        .unwrap();
    found.sort();
    found
}

/// The matches of a walk driven by the matcher on `tree`, home at its root, sorted.
pub(crate) fn matched(tree: &FixtureTree, interests: &InterestSet) -> Vec<Found> {
    let matcher = compile(interests, tree.root());
    let mut found = Vec::new();
    walk(&matcher, interests, tree.root(), matcher.root_state(tree.root()), &mut found);
    found.sort();
    found
}

fn walk(matcher: &Matcher, interests: &InterestSet, folder: &Path, state: TrieState, found: &mut Vec<Found>) {
    let listing = read(folder);
    let facts = matcher.folder(&listing);
    for (name, entry) in listing.iter() {
        let path = folder.join(OsStr::from_bytes(name));
        let mut hits: Vec<(u32, bool)> =
            matcher.entry(state, name, entry.kind).as_slice().iter().map(|&hit| (hit, false)).collect();
        if entry.kind == EntryKind::Folder {
            hits.extend(matcher.folder(&read(&path)).markers().iter().map(|&hit| (hit, true)));
        }
        hits.sort_unstable();
        let mut whole = false;
        for (hit, marker) in hits {
            let siblings = interests.get(hit as usize).unwrap().siblings();
            let mask = facts.siblings_of(hit);
            let present = (0..).zip(siblings).filter(|&(bit, _)| mask & (1 << bit) != 0).map(|(_, name)| name.clone());
            let checked = marker && matcher.checks_cachedir_tag(hit);
            found.push((hit as usize, path.clone(), present.collect(), checked));
            whole |= matcher.claims_whole(hit);
        }
        if entry.kind == EntryKind::Folder && !whole {
            walk(matcher, interests, &path, matcher.child_state(state, name), found);
        }
    }
}

fn read(folder: &Path) -> FolderListing {
    let mut listing = FolderListing::new();
    for entry in fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.path().symlink_metadata().unwrap();
        let kind = if metadata.is_symlink() {
            EntryKind::Symlink
        } else if metadata.is_dir() {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        listing.push(entry.file_name().as_bytes(), raw(kind)).unwrap();
    }
    listing
}
