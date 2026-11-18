//! Compiling an interest set into a matcher.

use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use fresher_core::{CACHEDIR_TAG, Claim, InterestSet, KnownFolder, Pattern};

use crate::matcher::fingerprint::fingerprint;
use crate::matcher::small_list::SmallList;
use crate::matcher::trie::{Segment, TrieBuilder};
use crate::matcher::{InterestFacts, Matcher, SiblingSlot};

/// The most sibling names one interest may have: they are a 64-bit mask in each hit.
const MAX_SIBLINGS: usize = 64;

/// Resolves an anchored pattern to its absolute anchor folder and the segments below it; `None` when it does not
/// resolve.
pub(crate) type Anchor<'a> = &'a dyn Fn(&Pattern) -> Option<(PathBuf, Vec<Segment>)>;

/// The resolver of the home folder alone (tests and benches): `Pattern::Under(Home, …)` against `home`.
pub(crate) fn under_home(home: &Path) -> impl Fn(&Pattern) -> Option<(PathBuf, Vec<Segment>)> + '_ {
    move |pattern| match pattern {
        Pattern::Under(KnownFolder::Home, relative) => Some((home.to_path_buf(), Segment::parse(relative))),
        Pattern::Under(..) | Pattern::FolderName(_) | Pattern::MarkerFile(_) | Pattern::Extension(_) => None,
    }
}

impl Matcher {
    /// Compiles every interest of `interests`. An anchored interest whose anchor does not resolve, and an interest
    /// with more sibling names than a hit can hold, are skipped and logged: they match nothing.
    pub(crate) fn compile(interests: &InterestSet, anchor: Anchor<'_>) -> Self {
        let mut matcher = Self::default();
        let mut trie = TrieBuilder::default();
        let mut anchors = Vec::new();
        for (index, interest) in interests.iter() {
            let Ok(id) = u32::try_from(index) else { break };
            if interest.siblings().len() > MAX_SIBLINGS {
                tracing::warn!(interest = index, "an interest has more than 64 sibling names; it matches nothing");
                matcher.interests.push(InterestFacts::default());
                continue;
            }
            let mut facts = InterestFacts { whole: interest.claim() == Claim::Whole, ..InterestFacts::default() };
            match interest.pattern() {
                Pattern::FolderName(name) => add(&mut matcher.folder_names, name, id),
                Pattern::Extension(extension) => add(&mut matcher.extensions, extension, id),
                Pattern::MarkerFile(marker) => {
                    facts.cachedir_tag = marker == CACHEDIR_TAG;
                    add(&mut matcher.markers, marker, id);
                }
                Pattern::Under(..) => {
                    let Some(mut segments) = anchor(interest.pattern()).and_then(|(at, below)| absolute(&at, below))
                    else {
                        tracing::warn!(interest = index, "an anchored interest does not resolve; it matches nothing");
                        matcher.interests.push(InterestFacts::default());
                        continue;
                    };
                    if interest.matches_only_bundle_ids()
                        && let Some(last @ Segment::Any) = segments.last_mut()
                    {
                        *last = Segment::BundleId;
                    }
                    trie.insert(&segments, id);
                    anchors.push((id, segments));
                    facts.holds = !interest.holds().is_empty();
                    for (bit, name) in (0_u8..).zip(interest.holds()) {
                        if name == CACHEDIR_TAG {
                            facts.held_tag = Some(bit);
                        }
                        let slots = matcher.held.entry(name.as_bytes().into()).or_default();
                        slots.push(SiblingSlot { interest: id, bit });
                    }
                }
            }
            for (bit, sibling) in (0_u8..).zip(interest.siblings()) {
                let slots = matcher.siblings.entry(sibling.as_bytes().into()).or_default();
                slots.push(SiblingSlot { interest: id, bit });
            }
            matcher.interests.push(facts);
        }
        matcher.trie = trie.build();
        matcher.fingerprint = fingerprint(interests, &anchors);
        matcher.anchors = anchors;
        matcher
    }
}

fn add(map: &mut HashMap<Box<[u8]>, SmallList>, name: &str, interest: u32) {
    map.entry(name.as_bytes().into()).or_default().push(interest);
}

/// The segments from the file system root: the anchor's names, then the pattern's. `None` for an anchor that is not
/// an absolute path of plain names.
fn absolute(anchor: &Path, below: Vec<Segment>) -> Option<Vec<Segment>> {
    let mut components = anchor.components();
    if components.next() != Some(Component::RootDir) {
        return None;
    }
    let mut segments = Vec::with_capacity(below.len() + 4);
    for component in components {
        let Component::Normal(name) = component else { return None };
        segments.push(Segment::Literal(name.as_bytes().into()));
    }
    segments.extend(below);
    Some(segments)
}
