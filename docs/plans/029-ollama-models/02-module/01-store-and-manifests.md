# 029/02/01 — The store and the manifests

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, docs

The Ollama row of the stores table, its six interests, the gathering of its matches, and the pure reading of a
manifest and of a model's name. Nothing is offered yet. The design: [`../00-design.md`](../00-design.md), sections 5
and 6.

## Files

- `modules/local-ai-models/stores.toml` — the row `ollama` (design section 5) and its lines in the head comment (the
  shape `ollama-store`: the store folder itself, its layout known by the shape).
- `modules/local-ai-models/src/stores.rs` — `Shape::OllamaStore`; the table refuses, one error each: an `ollama-store`
  path ending in `*`, a `sign`, an `app`, kinds, or no `category`; `interests()` gives the row's six interests in the
  design's order (a to f), the first holding `manifests` and `blobs`, every one `Claim::Item`.
- `modules/local-ai-models/src/gathered.rs` — the Ollama candidates by interest: the store folder (its `holds`), the
  manifests' folders (path, facts), the manifests and the blob entries (path, kind, sizes, `modified`, `accessed`,
  facts).
- `modules/local-ai-models/src/ollama/mod.rs` (new) — the submodule; `ollama/manifest.rs` (new):
  - `pub(crate) struct Digest([u8; 32])` with `parse("sha256:<64 lowercase hex>")` and `blob_name()` →
    `sha256-<hex>`;
  - `pub(crate) struct Manifest { config: Digest, layers: Vec<Digest>, sizes: BTreeMap<Digest, u64> }` with
    `digests()` (config and layers, each once);
  - `pub(crate) enum Unreadable { NotRead(SmallFile), NotJson, Schema, Digest, NoLayers, SizeMismatch }` and
    `pub(crate) fn read(file: SmallFile) -> Result<Manifest, Unreadable>`: `serde_json`, unknown fields allowed,
    `schemaVersion` 2, every digest of the shape, at least one layer, a digest listed twice with two sizes refused.
  - `pub(crate) fn blob_kind(name: &OsStr) -> BlobName`: `Layer(Digest)`, `Partial(Digest)` (`-partial` or
    `-partial-<n>`), `Other`.
- `modules/local-ai-models/src/name.rs` — `pub(crate) struct OllamaTag { host, namespace, model, tag }` read from the
  last four components under `manifests/` (valid UTF-8, none starting with `.`), its `title()` (design section 6) and
  its `identity()` (`ollama/<host>/<namespace>/<model>/<tag>`).
- `modules/local-ai-models/src/lib.rs` — `on_match` gathers the Ollama interests; nothing is reported for them yet.
- `modules/local-ai-models/Cargo.toml` — `serde_json` from dev-dependencies to dependencies; `docs/dependencies.md` —
  the module in `serde_json`'s row.

## Produces

The Ollama row and its gathered candidates, `Manifest`, `Digest`, `BlobName`, `OllamaTag`, for 02.

## Checks

- `stores.rs`, negative first: one test per new refusal; positive: the shipped table pinned: four rows and nine
  interests (three of plan 023, six of the Ollama row).
- `ollama/manifest.rs`, negative first: `a_missing_link_or_long_file_is_not_read` (each `SmallFile` but `Text`),
  `text_that_is_not_json_is_unreadable`, `another_schema_version_is_unreadable`, `a_digest_off_the_shape_is_unreadable`
  (upper case, short, `sha512:`, a path in it), `a_manifest_without_layers_is_unreadable`,
  `a_digest_twice_with_two_sizes_is_unreadable`; positive: `reads_the_config_and_the_layers`,
  `unknown_fields_are_allowed`, `a_digest_listed_twice_counts_once`, `a_manifest_of_4096_bytes_is_read` (with 024's
  `SmallFile::Text` at the limit).
- `blob_kind`: `Other` for `.DS_Store`, `sha256-short`, `sha256-<hex>.tmp`; `Partial` for both partial names;
  `Layer` for the plain name.
- `name.rs`: refused paths (three components, a hidden component, not UTF-8); positive: the three title forms.
- `cargo nextest run -p fresher-module-local-ai-models`; clippy; fmt.

## Conclusions

Built, with one change of order. The pure parts (the Ollama shape and tool in `stores.rs`, `OllamaTag`, the
manifest reader, `BlobName`) were built and tested on a test table without the shipped `stores.toml` row (preflight
F4): the row, the category and the fixtures arrived with step 02 and 03 in one commit, because the module's contract
test goes green only with all three. Manifests are read through `cx.system().small_file` (4 KiB) in `finish`; the
`serde_json` dependency moved from dev to normal (the dependencies table says so). A manifest that is not JSON or has
a digest that is not 64 lowercase hex characters is unreadable. Review: approved.
