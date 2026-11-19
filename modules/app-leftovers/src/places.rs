//! The places the module looks in, as data: `places.toml`, parsed and checked when the module is built.

use fresher_core::{Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern, TextKey};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../places.toml");

/// The last name of a `path`: it matches any name shaped as a bundle id.
const ANY_NAME: &str = "*";

/// The only known folders a row may be under: the user's own app data, never a place the scan walks as its own, the
/// home or the apps themselves.
const LIBRARY_PLACES: [KnownFolder; 7] = [
    KnownFolder::Library,
    KnownFolder::AppCaches,
    KnownFolder::AppSupport,
    KnownFolder::Preferences,
    KnownFolder::Logs,
    KnownFolder::Containers,
    KnownFolder::SavedState,
];

/// The group containers' name under the library: never a row's place (the user's decision 1).
const GROUP_CONTAINERS: &str = "Group Containers";

/// The launch agents' name under the library: what starts a program, never a leftover to offer.
const LAUNCH_AGENTS: &str = "LaunchAgents";

/// The names under the library that have a known folder of their own: a row names that known folder instead.
const OWN_KNOWN_FOLDERS: [&str; 6] =
    ["Caches", "Application Support", "Preferences", "Logs", "Containers", "Saved Application State"];

/// Why a places table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlacesError {
    #[error("the places table is not valid TOML: {0}")]
    Format(String),
    #[error("the places table lists no place")]
    Empty,
    #[error("`{id}` is not a row id in kebab case")]
    BadId { id: String },
    #[error("the row id `{id}` is listed twice")]
    DuplicateId { id: String },
    #[error("the row `{id}` names `{category}`, which is not a category of the module")]
    UnknownCategory { id: String, category: String },
    #[error("the category `{category}` has no row")]
    CategoryWithoutRow { category: String },
    #[error("the row `{id}` is under `{folder}`, which is not one of the user's library places")]
    WalkedPlace { id: String, folder: KnownFolder },
    #[error("the row `{id}` is under the group containers, which the module leaves alone")]
    GroupContainers { id: String },
    #[error("the row `{id}` names `{path}`, which has a known folder of its own: the row must be under it")]
    OwnKnownFolder { id: String, path: String },
    #[error("the row `{id}` names `{path}`, which is not plain folder names followed by `*`")]
    NotAStarPath { id: String, path: String },
    #[error("the rows `{first}` and `{second}` look for the same kind of name at the same place")]
    SameForm { first: String, second: String },
    #[error("the row `{id}` names the suffix `{suffix}`, which is not a dot and a plain name's end")]
    BadSuffix { id: String, suffix: String },
    #[error("the rows `{first}` and `{second}` share a place but disagree on `optional`")]
    OptionalDiffers { first: String, second: String },
    #[error("no row is a proof that an id belongs to an app")]
    NoProof,
    #[error("the row `{id}` is neither offered nor a proof: it would only cost a walk")]
    RowWithoutUse { id: String },
}

/// What a row's leftover is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Form {
    Folder,
    File,
}

/// One row of the table: a form of leftover the module offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) category: &'static str,
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
    pub(crate) kind: Form,
    pub(crate) suffix: Option<String>,
    pub(crate) proof: bool,
    pub(crate) optional: bool,
    /// Its leftovers are offered (findings or left-out items); a row that is not offered is read only as a proof and
    /// for the age of its id.
    pub(crate) offered: bool,
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Places {
    rows: Vec<Row>,
}

impl Places {
    pub(crate) fn shipped() -> Result<Self, PlacesError> {
        Self::parse(SHIPPED, &crate::categories::CATEGORIES)
    }

    /// A table read from `text` and checked against the module's `categories`: at least one row; row ids in kebab
    /// case, unique; each row in one of `categories`, and each of `categories` with a row; each row under a library
    /// place and never the group containers; `path` plain names then `*`; a suffix that is a dot and a plain name's
    /// end; each row offered or a proof; no two rows of one place with the same kind and suffix, and all of one place
    /// agreeing on `optional`; at least one proof.
    pub(crate) fn parse(text: &str, categories: &[(&'static str, DangerLevel)]) -> Result<Self, PlacesError> {
        let table: RawTable = toml::from_str(text).map_err(|error| PlacesError::Format(error.to_string()))?;
        if table.place.is_empty() {
            return Err(PlacesError::Empty);
        }
        let mut rows: Vec<Row> = Vec::with_capacity(table.place.len());
        for raw in table.place {
            let row = check_row(raw, categories)?;
            check_against_others(&row, &rows)?;
            rows.push(row);
        }
        if let Some((category, _)) = categories.iter().find(|(id, _)| !rows.iter().any(|row| row.category == *id)) {
            return Err(PlacesError::CategoryWithoutRow { category: (*category).into() });
        }
        if !rows.iter().any(|row| row.proof) {
            return Err(PlacesError::NoProof);
        }
        Ok(Self { rows })
    }

    /// The interests, in the table's order of first use: one per distinct `(folder, path)`, its names claimed whole
    /// and only when shaped as a bundle id.
    pub(crate) fn interests(&self) -> Result<Vec<Interest>, InterestError> {
        self.distinct_places()
            .into_iter()
            .map(|(folder, path)| {
                let pattern = Pattern::Under(folder, path.to_owned());
                Interest::new(pattern, Claim::Whole, Vec::new())?.only_bundle_ids()
            })
            .collect()
    }

    /// Every row, in the table's order: a leftover names its row by its index here.
    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The row at `index`.
    pub(crate) fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// The rows of the interest `interest` (an index in [`Self::interests`]), with their indexes, in the table's
    /// order; none for an index of no interest.
    pub(crate) fn rows_of(&self, interest: usize) -> Vec<(usize, &Row)> {
        let Some(&(folder, path)) = self.distinct_places().get(interest) else { return Vec::new() };
        self.rows.iter().enumerate().filter(|(_, row)| row.folder == folder && row.path == path).collect()
    }

    /// Each distinct `(folder, path)` of the rows, in the table's order of first use: one interest each.
    fn distinct_places(&self) -> Vec<(KnownFolder, &str)> {
        let mut places: Vec<(KnownFolder, &str)> = Vec::new();
        for row in &self.rows {
            if !places.contains(&(row.folder, row.path.as_str())) {
                places.push((row.folder, row.path.as_str()));
            }
        }
        places
    }
}

/// One row checked on its own.
fn check_row(raw: RawRow, categories: &[(&'static str, DangerLevel)]) -> Result<Row, PlacesError> {
    let id = raw.id;
    if !is_row_id(&id) {
        return Err(PlacesError::BadId { id });
    }
    let Some(&(category, _)) = categories.iter().find(|(known, _)| *known == raw.category) else {
        return Err(PlacesError::UnknownCategory { id, category: raw.category });
    };
    if raw.folder == KnownFolder::GroupContainers {
        return Err(PlacesError::GroupContainers { id });
    }
    if !LIBRARY_PLACES.contains(&raw.folder) {
        return Err(PlacesError::WalkedPlace { id, folder: raw.folder });
    }
    if raw.folder == KnownFolder::Library {
        check_library_path(&id, &raw.path)?;
    }
    if !is_star_path(&raw.path) {
        return Err(PlacesError::NotAStarPath { id, path: raw.path });
    }
    if let Some(suffix) = raw.suffix.as_deref().filter(|suffix| !is_suffix(suffix)) {
        return Err(PlacesError::BadSuffix { id, suffix: suffix.into() });
    }
    if !raw.offered && !raw.proof {
        return Err(PlacesError::RowWithoutUse { id });
    }
    Ok(Row {
        id,
        category,
        folder: raw.folder,
        path: raw.path,
        kind: raw.kind,
        suffix: raw.suffix,
        proof: raw.proof,
        optional: raw.optional,
        offered: raw.offered,
    })
}

/// A path under the library whose first name is a place the module must never reach that way: the group containers,
/// the launch agents, or a folder with a known folder of its own (`Caches` is `app-caches`), case ignored as the file
/// system does.
fn check_library_path(id: &str, path: &str) -> Result<(), PlacesError> {
    let first = path.split('/').next().unwrap_or_default();
    let is = |name: &str| first.eq_ignore_ascii_case(name);
    if is(GROUP_CONTAINERS) {
        return Err(PlacesError::GroupContainers { id: id.into() });
    }
    if is(LAUNCH_AGENTS) {
        return Err(PlacesError::WalkedPlace { id: id.into(), folder: KnownFolder::Library });
    }
    if OWN_KNOWN_FOLDERS.into_iter().any(is) {
        return Err(PlacesError::OwnKnownFolder { id: id.into(), path: path.into() });
    }
    Ok(())
}

/// `row` against the rows before it: a new id, a new form at its place, and the same `optional` as its place.
fn check_against_others(row: &Row, before: &[Row]) -> Result<(), PlacesError> {
    if before.iter().any(|other| other.id == row.id) {
        return Err(PlacesError::DuplicateId { id: row.id.clone() });
    }
    for other in before.iter().filter(|other| other.folder == row.folder && other.path == row.path) {
        let (first, second) = (other.id.clone(), row.id.clone());
        if other.kind == row.kind && other.suffix == row.suffix {
            return Err(PlacesError::SameForm { first, second });
        }
        if other.optional != row.optional {
            return Err(PlacesError::OptionalDiffers { first, second });
        }
    }
    Ok(())
}

/// A row id is one kebab-case segment of its title's key, `app-leftovers.title.<id>`.
fn is_row_id(id: &str) -> bool {
    !id.contains('.') && TextKey::new(format!("app-leftovers.title.{id}")).is_ok()
}

/// Plain folder names, then `*`: no empty name, `.`, `..` or `/` inside a name, and no `*` but the last.
fn is_star_path(path: &str) -> bool {
    let plain = |name: &&str| !name.is_empty() && *name != "." && *name != ".." && !name.contains(['\0', '*']);
    let mut names: Vec<&str> = path.split('/').collect();
    names.pop() == Some(ANY_NAME) && names.iter().all(plain)
}

/// A dot and then names of letters, digits, `-` or `_`, each name not empty, joined by dots (`.savedState`).
fn is_suffix(suffix: &str) -> bool {
    let Some(names) = suffix.strip_prefix('.') else { return false };
    names.split('.').all(|name| {
        !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    place: Vec<RawRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawRow {
    id: String,
    category: String,
    folder: KnownFolder,
    path: String,
    kind: Form,
    suffix: Option<String>,
    #[serde(default)]
    proof: bool,
    #[serde(default)]
    optional: bool,
    #[serde(default = "offered_by_default")]
    offered: bool,
}

/// A row's leftovers are offered unless it says `offered = false`.
const fn offered_by_default() -> bool {
    true
}

#[cfg(test)]
mod tests;
