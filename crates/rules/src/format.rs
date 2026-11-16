//! The TOML schema of a module's `rules.toml`, as written; [`crate::RuleSet::parse`] validates it.

use fresher_core::{DangerLevel, KnownFolder};
use serde::Deserialize;

/// A whole `rules.toml`: a list of `[[rule]]` tables.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleFile {
    #[serde(default)]
    pub(crate) rule: Vec<RuleSpec>,
}

/// One `[[rule]]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct RuleSpec {
    /// Unique in the file; findings name it.
    pub(crate) id: String,
    /// A category of the owning module.
    pub(crate) category: String,
    /// Never below the category's.
    pub(crate) danger: DangerLevel,
    pub(crate) removal: RemovalSpec,
    pub(crate) look: LookSpec,
    #[serde(default)]
    pub(crate) recognize: RecognizeSpec,
    /// Required, at least one; a missing list is caught by the validation, not by the parser.
    #[serde(default)]
    pub(crate) must_not_match: Vec<NameSpec>,
}

/// How a claimed item is removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RemovalSpec {
    /// The item goes to the Trash.
    Trash,
    /// The folder's contents go to the Trash; the folder stays.
    EmptyFolder,
}

/// Where the walk looks: exactly one key of `[rule.look]`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LookSpec {
    /// `under = { folder = "app-caches", path = "JetBrains/*" }`: a path under a known folder.
    Under(UnderSpec),
    FolderName(String),
    Marker(String),
    Extension(String),
}

/// A path under a known folder: a module never writes an OS path.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnderSpec {
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
}

/// How the item is recognized: every part given must hold.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct RecognizeSpec {
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) min_age_days: Option<u32>,
    pub(crate) sibling: Option<String>,
}

/// One `[[rule.must-not-match]]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NameSpec {
    pub(crate) name: String,
}
