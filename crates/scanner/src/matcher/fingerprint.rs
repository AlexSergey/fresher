//! The fingerprint of a compiled interest set, stored with the cache: another fingerprint means a full scan.

use fresher_core::{Claim, InterestSet, Pattern, stable_hash};

use crate::matcher::Matcher;
use crate::matcher::trie::Segment;

impl Matcher {
    /// A stable 64-bit hash of every interest and every resolved anchor: the same on every run for the same modules,
    /// different after any change of an interest.
    pub(crate) const fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

/// The hash of a canonical encoding: each interest's owner, local index, pattern (with its known folder and the shape
/// of its last `*`, its suffixes for a suffixed one), claim, siblings and held names, then each anchored interest's
/// resolved segments. Every string and list is length-prefixed, so no two sets encode alike.
pub(super) fn fingerprint(interests: &InterestSet, anchors: &[(u32, Vec<Segment>)]) -> u64 {
    let mut bytes = Vec::new();
    for (index, interest) in interests.iter() {
        let (owner, local) = interests.owner(index).map_or(("", 0), |(owner, local)| (owner.as_str(), local));
        put_bytes(&mut bytes, owner.as_bytes());
        put_number(&mut bytes, local);
        let (kind, folder, value) = match interest.pattern() {
            Pattern::FolderName(value) => (0, "", value),
            Pattern::Under(folder, value) if interest.matches_only_bundle_ids() => (4, folder.as_str(), value),
            Pattern::Under(folder, value) if !interest.name_suffixes().is_empty() => (5, folder.as_str(), value),
            Pattern::Under(folder, value) => (1, folder.as_str(), value),
            Pattern::MarkerFile(value) => (2, "", value),
            Pattern::Extension(value) => (3, "", value),
        };
        bytes.push(kind);
        if kind == 5 {
            put_number(&mut bytes, interest.name_suffixes().len());
            for suffix in interest.name_suffixes() {
                put_bytes(&mut bytes, suffix.as_bytes());
            }
        }
        put_bytes(&mut bytes, folder.as_bytes());
        put_bytes(&mut bytes, value.as_bytes());
        bytes.push(match interest.claim() {
            Claim::Whole => 0,
            Claim::Item => 1,
        });
        put_number(&mut bytes, interest.siblings().len());
        for sibling in interest.siblings() {
            put_bytes(&mut bytes, sibling.as_bytes());
        }
        put_number(&mut bytes, interest.holds().len());
        for held in interest.holds() {
            put_bytes(&mut bytes, held.as_bytes());
        }
    }
    put_number(&mut bytes, anchors.len());
    for (interest, segments) in anchors {
        put_number(&mut bytes, *interest as usize);
        put_number(&mut bytes, segments.len());
        for segment in segments {
            match segment {
                Segment::Literal(name) => {
                    bytes.push(0);
                    put_bytes(&mut bytes, name);
                }
                Segment::Any => bytes.push(1),
                Segment::BundleId => bytes.push(2),
                Segment::Suffixes(suffixes) => {
                    bytes.push(3);
                    put_number(&mut bytes, suffixes.len());
                    for suffix in suffixes {
                        put_bytes(&mut bytes, suffix);
                    }
                }
            }
        }
    }
    stable_hash(&bytes)
}

fn put_number(bytes: &mut Vec<u8>, number: usize) {
    bytes.extend_from_slice(&(number as u64).to_le_bytes());
}

fn put_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    put_number(bytes, value.len());
    bytes.extend_from_slice(value);
}
