//! Ollama's store: what its manifests and its layers are, read as plain values, and what the module decides of them.

pub(crate) mod decide;
pub(crate) mod layers;
pub(crate) mod manifest;
#[cfg(test)]
pub(crate) mod test_support;
