//! The cache folders the `dev-caches` module offers and the signs it checks, as data: `tools.toml`, parsed and
//! checked when the module is built.

use std::collections::BTreeSet;

use fresher_core::{AppIdError, BundleId, Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern, TextKey};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../tools.toml");

/// The last name of a `path` whose matches may have any name.
const ANY_NAME: &str = "*";

/// The most names a sign may have: the walk reports them as a 64-bit mask.
const MAX_SIGNS: usize = 64;

/// Why a tools table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ToolsError {
    #[error("the tools table is not valid TOML: {0}")]
    Format(String),
    #[error("the tools table lists no cache")]
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
    #[error("the row `{id}` names `{path}`, which is not a relative path of plain names")]
    NotRelative { id: String, path: String },
    #[error("the sign of the row `{id}` is at `{sign_at}`, which is not above its folder `{path}`")]
    SignNotAbove { id: String, sign_at: String, path: String },
    #[error("the rows `{first}` and `{second}` name the same place")]
    SamePlace { first: String, second: String },
    #[error("the rows `{first}` and `{second}` look for different signs at the same place")]
    SignsDiffer { first: String, second: String },
    #[error("the row `{id}` names no sign")]
    EmptySign { id: String },
    #[error("the sign `{name}` of the row `{id}` is not a single name")]
    NotAName { id: String, name: String },
    #[error("the row `{id}` names the sign `{name}` twice")]
    DuplicateSign { id: String, name: String },
    #[error("the row `{id}` names {count} signs; at most {MAX_SIGNS}")]
    TooManySigns { id: String, count: usize },
    #[error(transparent)]
    BundleId(#[from] AppIdError),
    #[error("the row `{id}` gives a name shape but its path does not end in `*`")]
    ShapeWithoutStar { id: String },
}

/// How a tool makes its folder again, which decides the finding's reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ComesBack {
    Download,
    Build,
}

/// The shape every matched name of a row must have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NameShape {
    ProjectHash,
}

/// One row of the table: a cache folder the module offers, where its tool's sign is, and how it comes back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cache {
    pub(crate) id: String,
    pub(crate) category: &'static str,
    pub(crate) tool: String,
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
    pub(crate) sign_at: Option<String>,
    pub(crate) sign: Vec<String>,
    pub(crate) comes_back: ComesBack,
    pub(crate) app: Option<BundleId>,
    pub(crate) name: Option<NameShape>,
}

/// What a match of one of the module's interests is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// A folder the row with this index offers.
    Cache(usize),
    /// The folder that holds the sign of the rows with a `sign-at`: the n-th distinct sign place, in the table's
    /// order.
    SignHolder(usize),
}

/// The module's interests and, at the same index, the role of each one's matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InterestMap {
    pub(crate) interests: Vec<Interest>,
    pub(crate) roles: Vec<Role>,
    holder_of: Vec<Option<usize>>,
}

impl InterestMap {
    /// The sign holder (the index of [`Role::SignHolder`]) that holds the sign of the row `row`, when its sign is
    /// not in its own folder.
    pub(crate) fn holder_of(&self, row: usize) -> Option<usize> {
        self.holder_of.get(row).copied().flatten()
    }
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tools {
    caches: Vec<Cache>,
}

impl Tools {
    pub(crate) fn shipped() -> Result<Self, ToolsError> {
        Self::parse(SHIPPED, &crate::categories::CATEGORIES)
    }

    /// A table read from `text` and checked against the module's `categories`: at least one row; row ids in kebab
    /// case, unique; each row in one of `categories`, and each of `categories` with a row; no temporary folder;
    /// `path` and `sign-at` relative paths of plain names, `sign-at` strictly above `path`; no two rows at one place;
    /// one sign per sign place; a sign of 1 to 64 distinct single names; a valid app; a name shape only for a `path`
    /// ending in `*`.
    pub(crate) fn parse(text: &str, categories: &[(&'static str, DangerLevel)]) -> Result<Self, ToolsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| ToolsError::Format(error.to_string()))?;
        if table.cache.is_empty() {
            return Err(ToolsError::Empty);
        }
        let mut caches: Vec<Cache> = Vec::with_capacity(table.cache.len());
        for raw in table.cache {
            let cache = check_row(raw, categories)?;
            check_against_others(&cache, &caches)?;
            caches.push(cache);
        }
        if let Some((category, _)) = categories.iter().find(|(id, _)| !caches.iter().any(|row| row.category == *id)) {
            return Err(ToolsError::CategoryWithoutRow { category: (*category).into() });
        }
        Ok(Self { caches })
    }

    pub(crate) fn caches(&self) -> &[Cache] {
        &self.caches
    }

    /// The interests, in the table's order: per row, its folder claimed whole, looking for the sign when it is the
    /// folder's own; then per distinct sign place, that folder as an item, looking for the sign.
    pub(crate) fn interests(&self) -> Result<InterestMap, InterestError> {
        let mut interests = Vec::with_capacity(self.caches.len());
        let mut roles = Vec::with_capacity(self.caches.len());
        for (row, cache) in self.caches.iter().enumerate() {
            let interest = Interest::new(Pattern::Under(cache.folder, cache.path.clone()), Claim::Whole, Vec::new())?;
            let interest = if cache.sign_at.is_none() { interest.holding(cache.sign.clone())? } else { interest };
            interests.push(interest);
            roles.push(Role::Cache(row));
        }
        let mut holders: Vec<(KnownFolder, &str)> = Vec::new();
        let mut holder_of = Vec::with_capacity(self.caches.len());
        for cache in &self.caches {
            let Some(at) = cache.sign_at.as_deref() else {
                holder_of.push(None);
                continue;
            };
            let index = if let Some(index) = holders.iter().position(|holder| *holder == (cache.folder, at)) {
                index
            } else {
                let pattern = Pattern::Under(cache.folder, at.to_owned());
                interests.push(Interest::new(pattern, Claim::Item, Vec::new())?.holding(cache.sign.clone())?);
                roles.push(Role::SignHolder(holders.len()));
                holders.push((cache.folder, at));
                holders.len() - 1
            };
            holder_of.push(Some(index));
        }
        Ok(InterestMap { interests, roles, holder_of })
    }
}

/// One row checked on its own.
fn check_row(raw: RawCache, categories: &[(&'static str, DangerLevel)]) -> Result<Cache, ToolsError> {
    let id = raw.id;
    if !is_row_id(&id) {
        return Err(ToolsError::BadId { id });
    }
    let Some(&(category, _)) = categories.iter().find(|(known, _)| *known == raw.category) else {
        return Err(ToolsError::UnknownCategory { id, category: raw.category });
    };
    if raw.folder.is_walked_place() {
        return Err(ToolsError::WalkedPlace { id, folder: raw.folder });
    }
    if !is_plain_path(&raw.path) {
        return Err(ToolsError::NotRelative { id, path: raw.path });
    }
    if let Some(sign_at) = &raw.sign_at {
        if !is_plain_path(sign_at) {
            return Err(ToolsError::NotRelative { id, path: sign_at.clone() });
        }
        if !is_strictly_above(sign_at, &raw.path) {
            return Err(ToolsError::SignNotAbove { id, sign_at: sign_at.clone(), path: raw.path });
        }
    }
    check_sign(&id, &raw.sign)?;
    let app = raw.app.map(BundleId::new).transpose()?;
    if raw.name.is_some() && raw.path.rsplit('/').next() != Some(ANY_NAME) {
        return Err(ToolsError::ShapeWithoutStar { id });
    }
    Ok(Cache {
        id,
        category,
        tool: raw.tool,
        folder: raw.folder,
        path: raw.path,
        sign_at: raw.sign_at,
        sign: raw.sign,
        comes_back: raw.comes_back,
        app,
        name: raw.name,
    })
}

/// `cache` against the rows before it: a new id, a new place, and the same sign at a sign place already named.
fn check_against_others(cache: &Cache, before: &[Cache]) -> Result<(), ToolsError> {
    if before.iter().any(|row| row.id == cache.id) {
        return Err(ToolsError::DuplicateId { id: cache.id.clone() });
    }
    if let Some(row) = before.iter().find(|row| row.folder == cache.folder && row.path == cache.path) {
        return Err(ToolsError::SamePlace { first: row.id.clone(), second: cache.id.clone() });
    }
    let same_holder =
        |row: &&Cache| row.folder == cache.folder && row.sign_at.is_some() && row.sign_at == cache.sign_at;
    if let Some(row) = before.iter().filter(same_holder).find(|row| sign_names(row) != sign_names(cache)) {
        return Err(ToolsError::SignsDiffer { first: row.id.clone(), second: cache.id.clone() });
    }
    Ok(())
}

/// The names of a row's sign, in no order.
fn sign_names(row: &Cache) -> BTreeSet<&str> {
    row.sign.iter().map(String::as_str).collect()
}

fn check_sign(id: &str, sign: &[String]) -> Result<(), ToolsError> {
    if sign.is_empty() {
        return Err(ToolsError::EmptySign { id: id.into() });
    }
    if sign.len() > MAX_SIGNS {
        return Err(ToolsError::TooManySigns { id: id.into(), count: sign.len() });
    }
    let mut seen = BTreeSet::new();
    for name in sign {
        if !is_plain_name(name) {
            return Err(ToolsError::NotAName { id: id.into(), name: name.clone() });
        }
        if !seen.insert(name.as_str()) {
            return Err(ToolsError::DuplicateSign { id: id.into(), name: name.clone() });
        }
    }
    Ok(())
}

/// A row id is one kebab-case segment of its title's key, `dev-caches.title.<id>`.
fn is_row_id(id: &str) -> bool {
    !id.contains('.') && TextKey::new(format!("dev-caches.title.{id}")).is_ok()
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

/// A relative path of plain names, written once: no leading, trailing or doubled `/`, no `.` or `..`.
fn is_plain_path(path: &str) -> bool {
    path.split('/').all(is_plain_name)
}

fn is_strictly_above(ancestor: &str, path: &str) -> bool {
    let (above, below): (Vec<&str>, Vec<&str>) = (ancestor.split('/').collect(), path.split('/').collect());
    above.len() < below.len() && below.starts_with(&above)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    cache: Vec<RawCache>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawCache {
    id: String,
    category: String,
    tool: String,
    folder: KnownFolder,
    path: String,
    sign_at: Option<String>,
    sign: Vec<String>,
    comes_back: ComesBack,
    app: Option<String>,
    name: Option<NameShape>,
}

#[cfg(test)]
mod tests;
