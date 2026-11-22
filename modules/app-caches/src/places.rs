//! The places the module looks in, as data: `places.toml`, parsed and checked when the module is built.

use fresher_core::{Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern, TextKey};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../places.toml");

/// The last name of a `path`: it matches any name shaped as a bundle id.
const ANY_NAME: &str = "*";

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
    #[error("the row `{id}` is under `{folder}`, a place the scan walks as its own")]
    WalkedPlace { id: String, folder: KnownFolder },
    #[error("the row `{id}` names `{path}`, which is not plain folder names followed by `*`")]
    NotPlainStar { id: String, path: String },
    #[error("the rows `{first}` and `{second}` name the same place")]
    DuplicatePlace { first: String, second: String },
}

/// One row of the table: a place whose id-named folders the module looks at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) category: &'static str,
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Places {
    rows: Vec<Row>,
}

impl Places {
    pub(crate) fn shipped() -> Result<Self, PlacesError> {
        Self::parse(SHIPPED, &crate::categories::PLACE_CATEGORIES)
    }

    /// A table read from `text` and checked against the module's `categories`: at least one row; row ids in kebab
    /// case, unique; each row in one of `categories`, and each of `categories` with a row; no row under a place the
    /// scan walks as its own; `path` plain folder names then `*`; no two rows of one `(folder, path)`.
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
        Ok(Self { rows })
    }

    /// The interests, in the table's order: one per row, its names claimed whole and only when shaped as a bundle id.
    pub(crate) fn interests(&self) -> Result<Vec<Interest>, InterestError> {
        self.rows
            .iter()
            .map(|row| {
                let pattern = Pattern::Under(row.folder, row.path.clone());
                Interest::new(pattern, Claim::Whole, Vec::new())?.only_bundle_ids()
            })
            .collect()
    }

    /// Every row, in the table's order: a match names its row by its interest's index.
    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
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
    if raw.folder.is_walked_place() {
        return Err(PlacesError::WalkedPlace { id, folder: raw.folder });
    }
    if !is_star_path(&raw.path) {
        return Err(PlacesError::NotPlainStar { id, path: raw.path });
    }
    Ok(Row { id, category, folder: raw.folder, path: raw.path })
}

/// `row` against the rows before it: a new id and a new place.
fn check_against_others(row: &Row, before: &[Row]) -> Result<(), PlacesError> {
    if before.iter().any(|other| other.id == row.id) {
        return Err(PlacesError::DuplicateId { id: row.id.clone() });
    }
    if let Some(other) = before.iter().find(|other| other.folder == row.folder && other.path == row.path) {
        return Err(PlacesError::DuplicatePlace { first: other.id.clone(), second: row.id.clone() });
    }
    Ok(())
}

/// A row id is one kebab-case segment of a text key.
pub(crate) fn is_row_id(id: &str) -> bool {
    !id.contains('.') && TextKey::new(format!("app-caches.row.{id}")).is_ok()
}

/// Plain folder names, then `*`: no empty name, `.`, `..` or `/` inside a name, and no `*` but the last.
fn is_star_path(path: &str) -> bool {
    let plain = |name: &&str| !name.is_empty() && *name != "." && *name != ".." && !name.contains(['\0', '*']);
    let mut names: Vec<&str> = path.split('/').collect();
    names.pop() == Some(ANY_NAME) && names.iter().all(plain)
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
}

#[cfg(test)]
mod tests;
