//! Which layers each model of an Ollama store names, which of them it frees and which other models share them
//! (design, section 7). Pure: on the manifests already read, never the disk.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::name::OllamaTag;
use crate::ollama::manifest::{Digest, Manifest};

/// One model: the manifests that name exactly the same digests, one per tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Model {
    /// Its tags, in name order.
    pub(crate) tags: Vec<OllamaTag>,
    /// The manifest of each tag, in the same order.
    pub(crate) manifests: Vec<PathBuf>,
    /// The config and the layers its manifests name.
    pub(crate) digests: BTreeSet<Digest>,
    /// Every size a manifest of the model lists a digest with.
    pub(crate) listed: BTreeSet<(Digest, u64)>,
}

/// Every model of a store, in name order, and which models name each digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Layout {
    models: Vec<Model>,
    /// The models that name each digest, by their index in `models`, in name order.
    named_by: BTreeMap<Digest, Vec<usize>>,
}

/// The models of `manifests` (each tag, its manifest's path and the manifest read): manifests that name the same set
/// of digests are one model. The order of `manifests` does not matter.
pub(crate) fn layout(mut manifests: Vec<(OllamaTag, PathBuf, Manifest)>) -> Layout {
    manifests.sort_by(|(tag, path, _), (other_tag, other_path, _)| (tag, path).cmp(&(other_tag, other_path)));
    let mut models: Vec<Model> = Vec::new();
    for (tag, path, manifest) in manifests {
        let digests: BTreeSet<Digest> = manifest.digests().into_iter().collect();
        let listed = manifest.sizes.iter().map(|(digest, size)| (*digest, *size));
        match models.iter_mut().find(|model| model.digests == digests) {
            Some(model) => {
                model.tags.push(tag);
                model.manifests.push(path);
                model.listed.extend(listed);
            }
            None => models.push(Model { tags: vec![tag], manifests: vec![path], digests, listed: listed.collect() }),
        }
    }
    let mut named_by: BTreeMap<Digest, Vec<usize>> = BTreeMap::new();
    for (index, model) in models.iter().enumerate() {
        for digest in &model.digests {
            named_by.entry(*digest).or_default().push(index);
        }
    }
    Layout { models, named_by }
}

impl Layout {
    /// Every model, in name order.
    pub(crate) fn models(&self) -> &[Model] {
        &self.models
    }

    /// The digests of the model at `model` that no other model names, in digest order: the layers it frees.
    pub(crate) fn parts(&self, model: usize) -> Vec<Digest> {
        let Some(found) = self.models.get(model) else { return Vec::new() };
        found
            .digests
            .iter()
            .filter(|digest| self.named_by.get(*digest).is_some_and(|by| by == &[model]))
            .copied()
            .collect()
    }

    /// The other models that name a digest of the model at `model`, in name order.
    pub(crate) fn sharers(&self, model: usize) -> Vec<usize> {
        let Some(found) = self.models.get(model) else { return Vec::new() };
        let sharers: BTreeSet<usize> = found
            .digests
            .iter()
            .filter_map(|digest| self.named_by.get(digest))
            .flatten()
            .copied()
            .filter(|other| *other != model)
            .collect();
        sharers.into_iter().collect()
    }
}

#[cfg(test)]
mod tests;
