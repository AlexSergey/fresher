//! The cache folders of browsers and web-engine apps, as data: `apps.toml`, parsed and checked when the module is
//! built.

use fresher_core::{BundleId, Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern, is_bundle_id_shaped};
use serde::Deserialize;

use crate::places::is_row_id;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../apps.toml");

/// The name of a `path` that stands for a profile.
const PROFILE: &str = "*";

/// Why an apps table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppsError {
    #[error("the apps table is not valid TOML: {0}")]
    Format(String),
    #[error("the apps table lists no app")]
    Empty,
    #[error("`{id}` is not a row id in kebab case")]
    BadId { id: String },
    #[error("the row id `{id}` is listed twice")]
    DuplicateId { id: String },
    #[error("the row `{id}` names `{bundle_id}`, which is not a bundle id with a dot")]
    BadBundleId { id: String, bundle_id: String },
    #[error("the row `{id}` names `{category}`, which is not a category of the apps table")]
    UnknownCategory { id: String, category: String },
    #[error("the category `{category}` has no row")]
    CategoryWithoutRow { category: String },
    #[error("the row `{id}` is under `{folder}`, a place the scan walks as its own")]
    WalkedPlace { id: String, folder: KnownFolder },
    #[error("the row `{id}` names `{path}`, which is not plain folder names with at most one `*`")]
    BadPath { id: String, path: String },
    #[error("the row `{id}` names `{path}`, whose first name is `*`")]
    StarFirst { id: String, path: String },
    #[error("the row `{id}` names `{path}`, whose first name is shaped as a bundle id")]
    IdShapedName { id: String, path: String },
    #[error("the row `{id}` lists no cache")]
    NoCaches { id: String },
    #[error("the row `{id}` lists `{cache}`, which is not a plain name or is listed twice")]
    BadCache { id: String, cache: String },
    #[error("the rows `{first}` and `{second}` can name the same place")]
    OverlappingPlaces { first: String, second: String },
}

/// One row of the table: an app and the exact names of its cache folders in one place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppRow {
    pub(crate) id: String,
    pub(crate) category: &'static str,
    pub(crate) bundle_id: BundleId,
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
    /// The index of the `*` among the path's names (`Some(2)` for `Google/Chrome/*`), `None` without one.
    pub(crate) profile: Option<usize>,
    pub(crate) caches: Vec<String>,
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Apps {
    rows: Vec<AppRow>,
}

impl Apps {
    pub(crate) fn shipped() -> Result<Self, AppsError> {
        Self::parse(SHIPPED, &crate::categories::APP_CATEGORIES)
    }

    /// A table read from `text` and checked against the table's `categories`: at least one row; row ids in kebab case,
    /// unique; a bundle id with a dot; each row in one of `categories`, and each of `categories` with a row; no row
    /// under a place the scan walks as its own; `path` plain names with at most one `*`, neither the first name;
    /// `caches` plain names, listed once; no two rows of one folder whose paths can name the same folder.
    pub(crate) fn parse(text: &str, categories: &[(&'static str, DangerLevel)]) -> Result<Self, AppsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| AppsError::Format(error.to_string()))?;
        if table.app.is_empty() {
            return Err(AppsError::Empty);
        }
        let mut rows: Vec<AppRow> = Vec::with_capacity(table.app.len());
        for raw in table.app {
            let row = check_row(raw, categories)?;
            check_against_others(&row, &rows)?;
            rows.push(row);
        }
        if let Some((category, _)) = categories.iter().find(|(id, _)| !rows.iter().any(|row| row.category == *id)) {
            return Err(AppsError::CategoryWithoutRow { category: (*category).into() });
        }
        Ok(Self { rows })
    }

    /// The interests, in the table's order: for each row, for each of its caches, `<path>/<cache>` claimed whole,
    /// with the indexes of its row and its cache.
    pub(crate) fn interests(&self) -> Result<Vec<(Interest, usize, usize)>, InterestError> {
        let mut interests = Vec::new();
        for (row_index, row) in self.rows.iter().enumerate() {
            for (cache_index, cache) in row.caches.iter().enumerate() {
                let pattern = Pattern::Under(row.folder, format!("{}/{cache}", row.path));
                interests.push((Interest::new(pattern, Claim::Whole, Vec::new())?, row_index, cache_index));
            }
        }
        Ok(interests)
    }

    /// Every row, in the table's order.
    pub(crate) fn rows(&self) -> &[AppRow] {
        &self.rows
    }
}

/// One row checked on its own.
fn check_row(raw: RawRow, categories: &[(&'static str, DangerLevel)]) -> Result<AppRow, AppsError> {
    let id = raw.id;
    if !is_row_id(&id) {
        return Err(AppsError::BadId { id });
    }
    let Some(bundle_id) = BundleId::new(raw.bundle_id.as_str()).ok().filter(|_| raw.bundle_id.contains('.')) else {
        return Err(AppsError::BadBundleId { id, bundle_id: raw.bundle_id });
    };
    let Some(&(category, _)) = categories.iter().find(|(known, _)| *known == raw.category) else {
        return Err(AppsError::UnknownCategory { id, category: raw.category });
    };
    if raw.folder.is_walked_place() {
        return Err(AppsError::WalkedPlace { id, folder: raw.folder });
    }
    let profile = match check_path(&raw.path) {
        Ok(profile) => profile,
        Err(refusal) => return Err(refusal.into_error(id, raw.path)),
    };
    if raw.caches.is_empty() {
        return Err(AppsError::NoCaches { id });
    }
    for (index, cache) in raw.caches.iter().enumerate() {
        if !is_plain_name(cache) || raw.caches[..index].contains(cache) {
            return Err(AppsError::BadCache { id, cache: cache.clone() });
        }
    }
    Ok(AppRow { id, category, bundle_id, folder: raw.folder, path: raw.path, profile, caches: raw.caches })
}

/// `row` against the rows before it: a new id and a place no row of its folder can name.
fn check_against_others(row: &AppRow, before: &[AppRow]) -> Result<(), AppsError> {
    if before.iter().any(|other| other.id == row.id) {
        return Err(AppsError::DuplicateId { id: row.id.clone() });
    }
    if let Some(other) = before.iter().find(|other| other.folder == row.folder && overlap(&other.path, &row.path)) {
        return Err(AppsError::OverlappingPlaces { first: other.id.clone(), second: row.id.clone() });
    }
    Ok(())
}

/// Whether two checked paths can name the same folder: as many names, each pair equal or one of them `*`.
fn overlap(first: &str, second: &str) -> bool {
    let (first, second): (Vec<&str>, Vec<&str>) = (first.split('/').collect(), second.split('/').collect());
    first.len() == second.len()
        && first.iter().zip(&second).all(|(one, other)| one == other || *one == PROFILE || *other == PROFILE)
}

/// What is wrong with a path, before the row's id and path are put in the error.
enum PathRefusal {
    Bad,
    StarFirst,
    IdShaped,
}

impl PathRefusal {
    fn into_error(self, id: String, path: String) -> AppsError {
        match self {
            Self::Bad => AppsError::BadPath { id, path },
            Self::StarFirst => AppsError::StarFirst { id, path },
            Self::IdShaped => AppsError::IdShapedName { id, path },
        }
    }
}

/// The index of the path's `*`, if any: every name plain or `*` itself, at most one `*`, and the first name neither
/// `*` nor shaped as a bundle id.
fn check_path(path: &str) -> Result<Option<usize>, PathRefusal> {
    let names: Vec<&str> = path.split('/').collect();
    if !names.iter().all(|name| *name == PROFILE || is_plain_name(name)) {
        return Err(PathRefusal::Bad);
    }
    let mut stars = names.iter().enumerate().filter(|(_, name)| **name == PROFILE).map(|(index, _)| index);
    let profile = stars.next();
    if stars.next().is_some() {
        return Err(PathRefusal::Bad);
    }
    if profile == Some(0) {
        return Err(PathRefusal::StarFirst);
    }
    if names.first().is_some_and(|first| is_bundle_id_shaped(first.as_bytes())) {
        return Err(PathRefusal::IdShaped);
    }
    Ok(profile)
}

/// One plain folder name: not empty, not `.` or `..`, and no `/`, `*` or NUL in it.
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '*', '\0'])
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    app: Vec<RawRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawRow {
    id: String,
    category: String,
    bundle_id: String,
    folder: KnownFolder,
    path: String,
    caches: Vec<String>,
}

#[cfg(test)]
mod tests;
