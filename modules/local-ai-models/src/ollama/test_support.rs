//! Hand-made manifests, digests and tags for the tests of Ollama's layout and decision.

use std::path::PathBuf;

use crate::name::OllamaTag;
use crate::ollama::manifest::{Digest, Manifest};

/// The digest whose 32 bytes are all `byte`.
pub(crate) fn digest(byte: u8) -> Digest {
    Digest::parse(&format!("sha256:{}", format!("{byte:02x}").repeat(32))).unwrap()
}

/// The size a manifest of [`manifest`] lists the digest of `byte` with: 100 bytes per unit.
pub(crate) fn listed(byte: u8) -> u64 {
    u64::from(byte) * 100
}

/// A manifest of the config `config` and the layers `layers`, each listed with [`listed`] bytes.
pub(crate) fn manifest(config: u8, layers: &[u8]) -> Manifest {
    let sizes = std::iter::once(config).chain(layers.iter().copied()).map(|byte| (digest(byte), listed(byte)));
    Manifest {
        config: digest(config),
        layers: layers.iter().map(|byte| digest(*byte)).collect(),
        sizes: sizes.collect(),
    }
}

/// The tag `model:tag` of Ollama's own registry.
pub(crate) fn tag(model: &str, tag: &str) -> OllamaTag {
    OllamaTag { host: "registry.ollama.ai".into(), namespace: "library".into(), model: model.into(), tag: tag.into() }
}

/// The store at `/home/.ollama/models`, or the entry `place` in it.
pub(crate) fn in_store(place: &str) -> PathBuf {
    let store = PathBuf::from("/home/.ollama/models");
    if place.is_empty() { store } else { store.join(place) }
}

/// The manifest of `model:tag` in the store.
pub(crate) fn manifest_path(model: &str, tag: &str) -> PathBuf {
    in_store(&format!("manifests/registry.ollama.ai/library/{model}/{tag}"))
}
