//! A manifest and the names of a store's blobs, read as plain values (design, section 6). Nothing here reads the
//! disk: the manifest's text is given, read through the small-file reader by the caller.

use std::collections::BTreeMap;
use std::ffi::OsStr;

use fresher_core::SmallFile;
use serde::Deserialize;

/// The only manifest schema Ollama writes.
const SCHEMA_VERSION: u32 = 2;
/// The start of a digest as a manifest writes it, and of a blob's name as the store does.
const DIGEST_PREFIX: &str = "sha256:";
const BLOB_PREFIX: &str = "sha256-";
/// The digits of a digest, lowercase.
#[cfg(test)]
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
/// How many hex digits a digest has.
const HEX_LEN: usize = 64;
/// The end of the name of a download not finished, with or without a number after a dash.
const PARTIAL: &str = "-partial";

/// A layer's SHA-256 digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Digest([u8; 32]);

impl Digest {
    /// The digest `sha256:<64 lowercase hex>` names; `None` for anything else.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        Self::from_hex(text.strip_prefix(DIGEST_PREFIX)?)
    }

    fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != HEX_LEN || !hex.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')) {
            return None;
        }
        let digit = |byte: u8| if byte <= b'9' { byte - b'0' } else { byte - b'a' + 10 };
        let mut bytes = [0; 32];
        for (out, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
            *out = digit(pair[0]) << 4 | digit(pair[1]);
        }
        Some(Self(bytes))
    }

    /// The name of the layer's file under `blobs`: `sha256-<hex>` (the module finds layers by their matches' names,
    /// so only tests write one).
    #[cfg(test)]
    pub(crate) fn blob_name(&self) -> String {
        let mut name = String::from(BLOB_PREFIX);
        for byte in self.0 {
            name.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
            name.push(char::from(HEX_DIGITS[usize::from(byte & 15)]));
        }
        name
    }
}

/// The layers a model is made of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Manifest {
    pub(crate) config: Digest,
    /// The layers, each once, in the manifest's order.
    pub(crate) layers: Vec<Digest>,
    /// The size each digest, the config's included, is listed with.
    pub(crate) sizes: BTreeMap<Digest, u64>,
}

impl Manifest {
    /// The config and the layers, each once.
    pub(crate) fn digests(&self) -> Vec<Digest> {
        let mut digests = vec![self.config];
        digests.extend(self.layers.iter().filter(|layer| **layer != self.config));
        digests
    }
}

/// Why a manifest is not read: anything but a manifest of the known schema makes it unreadable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unreadable {
    /// The file was not read: a link, a long file, not text, gone or refused.
    NotRead(SmallFile),
    NotJson,
    /// JSON, but not `schemaVersion` 2 with a `config` and `layers` that each have a `digest` and a `size`.
    Schema,
    /// A digest off the shape `sha256:<64 lowercase hex>`.
    Digest,
    NoLayers,
    /// A digest listed twice with two sizes.
    SizeMismatch,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Raw {
    schema_version: u32,
    config: RawLayer,
    layers: Vec<RawLayer>,
}

#[derive(Deserialize)]
struct RawLayer {
    digest: String,
    size: u64,
}

/// The manifest `file` holds. Unknown fields are allowed: Ollama adds them.
pub(crate) fn read(file: SmallFile) -> Result<Manifest, Unreadable> {
    let SmallFile::Text(text) = file else { return Err(Unreadable::NotRead(file)) };
    let raw: Raw = serde_json::from_str(&text).map_err(|error| match error.classify() {
        serde_json::error::Category::Data => Unreadable::Schema,
        _ => Unreadable::NotJson,
    })?;
    if raw.schema_version != SCHEMA_VERSION {
        return Err(Unreadable::Schema);
    }
    if raw.layers.is_empty() {
        return Err(Unreadable::NoLayers);
    }
    let mut sizes = BTreeMap::new();
    let mut note = |layer: &RawLayer| {
        let digest = Digest::parse(&layer.digest).ok_or(Unreadable::Digest)?;
        match sizes.insert(digest, layer.size) {
            Some(before) if before != layer.size => Err(Unreadable::SizeMismatch),
            _ => Ok(digest),
        }
    };
    let config = note(&raw.config)?;
    let mut layers: Vec<Digest> = Vec::with_capacity(raw.layers.len());
    for layer in &raw.layers {
        let digest = note(layer)?;
        if !layers.contains(&digest) {
            layers.push(digest);
        }
    }
    Ok(Manifest { config, layers, sizes })
}

/// What a file's name under `blobs` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BlobName {
    /// `sha256-<hex>`: a layer.
    Layer(Digest),
    /// `sha256-<hex>-partial` or `sha256-<hex>-partial-<n>`: a download not finished.
    Partial(Digest),
    Other,
}

/// What the name of an entry of `blobs` is.
pub(crate) fn blob_kind(name: &OsStr) -> BlobName {
    let Some(rest) = name.to_str().and_then(|name| name.strip_prefix(BLOB_PREFIX)) else { return BlobName::Other };
    let (Some(hex), Some(tail)) = (rest.get(..HEX_LEN), rest.get(HEX_LEN..)) else { return BlobName::Other };
    let Some(digest) = Digest::from_hex(hex) else { return BlobName::Other };
    match tail.strip_prefix(PARTIAL) {
        _ if tail.is_empty() => BlobName::Layer(digest),
        Some("") => BlobName::Partial(digest),
        Some(number) => match number.strip_prefix('-') {
            Some(count) if !count.is_empty() && count.bytes().all(|byte| byte.is_ascii_digit()) => {
                BlobName::Partial(digest)
            }
            _ => BlobName::Other,
        },
        None => BlobName::Other,
    }
}

#[cfg(test)]
mod tests;
