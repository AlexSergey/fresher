//! Whether Ollama's store, and then each of its models, is offered, kept or not claimed: the rows of plan 029's
//! design, section 7, in their order, on plain values. Nothing here reads the disk: the caller reads the manifests.
//!
//! The store's sizes, on a left-out of the whole store, are the sum of its manifests, layers and partial downloads:
//! the store folder's own match claims only its entry (`Claim::Item`), so its sizes are the folder's alone.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, Completeness, EntryFacts, EntryKind, MAX_PARTS, Parts, Sizes, Timestamp};

use crate::decide::Reason;
use crate::gathered::Candidate;
use crate::name::OllamaTag;
use crate::ollama::layers::{self, Layout};
use crate::ollama::manifest::{BlobName, Digest, Manifest, Unreadable, blob_kind};
use crate::stores::{OLLAMA_LAYOUT, OllamaInterest};

/// What one scan matched of an Ollama store, sorted out: the store folder, the entries of its manifests' tree above a
/// tag (and which of them are not folders), its manifests (files at a tag's depth whose path is a tag) and its layers
/// and partial downloads. A blob of another name and anything else of another shape are neither counted nor claimed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entries<'a> {
    store: Option<&'a Candidate>,
    folders: Vec<&'a Candidate>,
    /// The entries of the manifests' tree above a tag that may hide manifests the scan never read: anything but a
    /// folder of a plain name (UTF-8, not starting with a dot) or a regular file of a hidden name. The walk never
    /// enters a link, and a folder of another name holds no manifest the module reads, but Ollama may read either.
    hiding: Vec<&'a Candidate>,
    /// In name order.
    manifests: Vec<(OllamaTag, &'a Candidate)>,
    layers: BTreeMap<Digest, &'a Candidate>,
    partials: Vec<&'a Candidate>,
}

impl<'a> Entries<'a> {
    /// The entries of `candidates`, the matches of one Ollama row, in any order.
    pub(crate) fn new(candidates: impl IntoIterator<Item = &'a Candidate>) -> Self {
        let mut entries = Self {
            store: None,
            folders: Vec::new(),
            hiding: Vec::new(),
            manifests: Vec::new(),
            layers: BTreeMap::new(),
            partials: Vec::new(),
        };
        for candidate in candidates {
            match candidate.ollama {
                Some(OllamaInterest::Store) => entries.store = entries.store.or(Some(candidate)),
                Some(OllamaInterest::Hosts | OllamaInterest::Namespaces | OllamaInterest::Models) => {
                    entries.folders.push(candidate);
                    if may_hide_manifests(candidate) {
                        entries.hiding.push(candidate);
                    }
                }
                Some(OllamaInterest::Manifests) if candidate.kind != EntryKind::Folder => {
                    if let Some(tag) = OllamaTag::read(&candidate.path) {
                        entries.manifests.push((tag, candidate));
                    }
                }
                Some(OllamaInterest::Blobs) => match candidate.path.file_name().map(blob_kind) {
                    Some(BlobName::Layer(digest)) => {
                        entries.layers.insert(digest, candidate);
                    }
                    Some(BlobName::Partial(_)) => entries.partials.push(candidate),
                    Some(BlobName::Other) | None => {}
                },
                Some(OllamaInterest::Manifests) | None => {}
            }
        }
        entries.folders.sort_by(|first, second| first.path.cmp(&second.path));
        entries.partials.sort_by(|first, second| first.path.cmp(&second.path));
        entries.manifests.sort_by(|(first, at), (second, other)| (first, &at.path).cmp(&(second, &other.path)));
        entries
    }

    /// Whether the store folder was matched holding `manifests` and `blobs`: when not, nothing of the store is
    /// claimed, and no manifest needs reading.
    pub(crate) fn has_layout(&self) -> bool {
        self.store.is_some_and(|found| OLLAMA_LAYOUT.iter().all(|name| found.holds.iter().any(|held| held == name)))
    }

    /// The store folder, when it was matched.
    pub(crate) fn store_path(&self) -> Option<&'a Path> {
        self.store.map(|store| store.path.as_path())
    }

    /// The manifests to read, in name order: each tag and the match of its file.
    pub(crate) fn manifests(&self) -> &[(OllamaTag, &'a Candidate)] {
        &self.manifests
    }

    /// The store's sizes: its manifests, layers and partial downloads.
    pub(crate) fn sizes(&self) -> Sizes {
        let manifests = self.manifests.iter().map(|(_, candidate)| *candidate);
        sum(manifests.chain(self.layers.values().copied()).chain(self.partials.iter().copied()))
    }

    fn manifest(&self, path: &Path) -> Option<&'a Candidate> {
        self.manifests.iter().find(|(_, candidate)| candidate.path == path).map(|(_, candidate)| *candidate)
    }
}

/// What becomes of the whole store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StoreVerdict {
    /// No store with its layout: nothing is said.
    NotClaimed,
    /// Nothing of the store is offered, for the reason, with the store's sizes.
    LeftOut(Reason, Sizes),
    /// Every manifest read: its models are decided one by one.
    Open(Layout),
}

/// The first of the store's rows that needs no manifest read and applies to `entries`, if any:
///
/// 1. no store folder, or it does not hold both `manifests` and `blobs`: not claimed;
/// 2. a folder of the manifests' tree, a manifest, a layer or a partial download is not the user's (another owner, a
///    system or user lock, restricted): `NotYours`;
/// 3. a partial download: `OllamaDownload`;
/// 4. (first half) an entry of the manifests' tree above a tag may hide manifests the scan never read (a link, a file
///    of a plain name, a folder of a hidden or non-UTF-8 name): `OllamaUnreadable`.
///
/// `None`: the store can still be offered, once its manifests are read.
pub(crate) fn kept(entries: &Entries<'_>) -> Option<StoreVerdict> {
    if !entries.has_layout() {
        return Some(StoreVerdict::NotClaimed);
    }
    let sizes = entries.sizes();
    let mut matched = entries.folders.iter().copied().chain(entries.manifests.iter().map(|(_, candidate)| *candidate));
    let foreign = matched.any(|candidate| !is_yours(candidate.facts))
        || entries.layers.values().chain(&entries.partials).any(|candidate| !is_yours(candidate.facts));
    if foreign {
        return Some(StoreVerdict::LeftOut(Reason::NotYours, sizes));
    }
    if !entries.partials.is_empty() {
        return Some(StoreVerdict::LeftOut(Reason::OllamaDownload, sizes));
    }
    if !entries.hiding.is_empty() {
        return Some(StoreVerdict::LeftOut(Reason::OllamaUnreadable, sizes));
    }
    None
}

/// The first of the store's rows that applies, `read` being the manifests of `entries` read, in their order: the rows
/// of [`kept`], then
///
/// 4. (second half) a manifest not read: `OllamaUnreadable`.
///
/// Otherwise the store is open, with its layout.
pub(crate) fn store(entries: &Entries<'_>, read: Vec<Result<Manifest, Unreadable>>) -> StoreVerdict {
    if let Some(verdict) = kept(entries) {
        return verdict;
    }
    let sizes = entries.sizes();
    if read.len() != entries.manifests.len() {
        return StoreVerdict::LeftOut(Reason::OllamaUnreadable, sizes);
    }
    let mut manifests = Vec::with_capacity(read.len());
    for ((tag, candidate), manifest) in entries.manifests.iter().zip(read) {
        let Ok(manifest) = manifest else { return StoreVerdict::LeftOut(Reason::OllamaUnreadable, sizes) };
        manifests.push((tag.clone(), candidate.path.clone(), manifest));
    }
    StoreVerdict::Open(layers::layout(manifests))
}

/// One model offered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Offer {
    /// Its manifests in name order, then the layers it frees by name.
    pub(crate) parts: Parts,
    /// What its parts take.
    pub(crate) freed: Sizes,
    /// The logical size of every layer it names, shared ones included: what a pull downloads.
    pub(crate) listed: ByteSize,
    /// The first tag's title of every other model that names one of its layers, in name order.
    pub(crate) sharers: Vec<String>,
    /// The newest of its manifests' last change and its freed layers' last access.
    pub(crate) date: Timestamp,
}

/// What becomes of one model of an open store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ModelVerdict {
    /// Not offered and nothing said.
    NotClaimed,
    /// Kept for the reason, with the sizes of the parts it has.
    LeftOut(Reason, Sizes),
    Offered(Offer),
}

/// The first of a model's rows that applies to the model at `index` of `layout`, the layout of `entries`:
///
/// 5. a layer it names is not among the matches, is not a regular file, or its logical size is not the one a
///    manifest lists: `OllamaIncomplete`, with the sizes of its manifests and the layers it would free that are there;
/// 6. more than `MAX_PARTS` parts, or fewer than two (one tag and every layer shared), or no time known of its
///    manifests and freed layers: not claimed;
/// 7. otherwise offered.
///
/// An incomplete model's layers still count as named, so they are never another model's parts.
pub(crate) fn model(entries: &Entries<'_>, layout: &Layout, index: usize) -> ModelVerdict {
    let Some(model) = layout.models().get(index) else { return ModelVerdict::NotClaimed };
    let Some(manifests) = model.manifests.iter().map(|path| entries.manifest(path)).collect::<Option<Vec<_>>>() else {
        return ModelVerdict::NotClaimed;
    };
    let freed: Vec<&Candidate> =
        layout.parts(index).iter().filter_map(|digest| entries.layers.get(digest).copied()).collect();
    let whole = model.listed.iter().all(|(digest, size)| {
        entries
            .layers
            .get(digest)
            .is_some_and(|layer| layer.kind == EntryKind::File && layer.sizes.logical.bytes() == *size)
    });
    if !whole {
        return ModelVerdict::LeftOut(Reason::OllamaIncomplete, sum(manifests.iter().chain(&freed).copied()));
    }
    let paths: Vec<PathBuf> = manifests.iter().chain(&freed).map(|candidate| candidate.path.clone()).collect();
    if paths.len() > MAX_PARTS {
        return ModelVerdict::NotClaimed;
    }
    let Ok(parts) = Parts::new(paths) else { return ModelVerdict::NotClaimed };
    let Some(freed_sizes) = checked_sum(manifests.iter().chain(&freed).copied()) else {
        return ModelVerdict::NotClaimed;
    };
    let listed = model
        .digests
        .iter()
        .try_fold(ByteSize::ZERO, |total, digest| total.checked_add(entries.layers.get(digest)?.sizes.logical));
    let Some(listed) = listed else { return ModelVerdict::NotClaimed };
    let changed = manifests.iter().map(|manifest| manifest.modified);
    let Some(date) = changed.chain(freed.iter().map(|layer| layer.accessed)).flatten().max() else {
        return ModelVerdict::NotClaimed;
    };
    let sharers = layout
        .sharers(index)
        .into_iter()
        .filter_map(|other| layout.models().get(other)?.tags.first().map(OllamaTag::title))
        .collect();
    ModelVerdict::Offered(Offer { parts, freed: freed_sizes, listed, sharers, date })
}

/// Whether `candidate`, an entry of the manifests' tree above a tag, may hide manifests: anything but a folder of a
/// plain name, whose manifests the scan reads, or a regular file of a hidden name (`.DS_Store`), which holds none.
fn may_hide_manifests(candidate: &Candidate) -> bool {
    let name = candidate.path.file_name().and_then(|name| name.to_str());
    let hidden = name.is_some_and(|name| name.starts_with('.'));
    match candidate.kind {
        EntryKind::Folder => name.is_none() || hidden,
        EntryKind::File => !hidden,
        EntryKind::Symlink | EntryKind::Other => true,
    }
}

/// The user's own, with no lock and no restriction.
const fn is_yours(facts: EntryFacts) -> bool {
    facts.owned_by_user && !facts.system_immutable && !facts.restricted && !facts.user_immutable
}

/// The sizes of `candidates` added; `None` when a sum does not fit.
fn checked_sum<'c>(candidates: impl IntoIterator<Item = &'c Candidate>) -> Option<Sizes> {
    candidates.into_iter().try_fold(Sizes::ZERO, |total, candidate| total.checked_add(candidate.sizes))
}

/// The sizes of `candidates` added, unknown when a sum does not fit: what a left-out shows.
fn sum<'c>(candidates: impl IntoIterator<Item = &'c Candidate>) -> Sizes {
    checked_sum(candidates).unwrap_or(Sizes { completeness: Completeness::Unknown, ..Sizes::ZERO })
}

#[cfg(test)]
mod tests;
