//! The managers whose versions the module offers, as data: `toolchains.toml`, parsed and checked when the module is
//! built, and the interests built from it.

use std::collections::BTreeSet;

use fresher_core::{Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../toolchains.toml");

/// The name a versions place ends in.
const STAR: &str = "*";

/// The pin file several managers share, so it is read for the keys of each row.
pub(crate) const TOOL_VERSIONS: &str = ".tool-versions";

/// Why a toolchains table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ToolchainsError {
    #[error("the toolchains table is not valid TOML: {0}")]
    Format(String),
    #[error("the toolchains table lists no row")]
    Empty,
    #[error("`{id}` is not a row id in kebab case")]
    BadId { id: String },
    #[error("the row id `{id}` is listed twice")]
    DuplicateId { id: String },
    #[error("the row `{id}` names `{category}`, which is not a category of the module")]
    UnknownCategory { id: String, category: String },
    #[error("the category `{category}` has no row")]
    CategoryWithoutRow { category: String },
    #[error("the row `{id}` names `{path}`, which is not a relative path of plain names")]
    NotRelative { id: String, path: String },
    #[error("the versions place `{path}` of the row `{id}` does not end in `*`")]
    NoStar { id: String, path: String },
    #[error("`{name}` in the row `{id}` is not a single name")]
    NotAName { id: String, name: String },
    #[error("the pin `{name}` of the row `{id}` names no keys")]
    KeysMissing { id: String, name: String },
    #[error("the pin `{name}` of the row `{id}` names the key `{key}` twice")]
    DuplicateKey { id: String, name: String, key: String },
    #[error("the row `{id}` names the switch `{switch}`, which is not a setting of the module")]
    UnknownSwitch { id: String, switch: String },
}

/// The manager whose files a row's pointers and pins are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Manager {
    Rustup,
    Nvm,
}

/// How a version's folder is named.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Shape {
    /// `<channel>[-<date>]-<triple>`.
    RustupToolchain,
    /// `v<major>.<minor>.<patch>`.
    NodeVersion,
}

/// A place of the manager's own that points at a version.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct PointerPlace {
    /// Relative to the home.
    pub(crate) place: String,
    /// The one name the place must hold, when it is a folder that is its own sign.
    #[serde(default)]
    pub(crate) holds: Option<String>,
}

/// A file a project keeps that pins a version.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct Pin {
    pub(crate) name: String,
    /// For `.tool-versions`: the tools of the file this manager reads.
    #[serde(default)]
    pub(crate) keys: Vec<String>,
}

/// One row of the table: a manager, where its versions are, what points at them, its category and its switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) manager: Manager,
    pub(crate) versions: String,
    pub(crate) shape: Shape,
    pub(crate) pointers: Vec<PointerPlace>,
    pub(crate) pins: Vec<Pin>,
    pub(crate) category: &'static str,
    pub(crate) switch: Option<&'static str>,
}

/// What an interest of the module found: its index in `Toolchains::interests` indexes `Toolchains::kinds`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InterestKind {
    /// The versions of the row (its index).
    Versions(usize),
    /// A pointer place (row index, place index in the row).
    Pointer(usize, usize),
    /// A pin file by its name, shared by every row that lists it.
    Pin(String),
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Toolchains {
    rows: Vec<Row>,
    kinds: Vec<InterestKind>,
}

impl Toolchains {
    pub(crate) fn shipped() -> Result<Self, ToolchainsError> {
        Self::parse(SHIPPED, &crate::categories::CATEGORIES, &[crate::settings::NODE_VERSIONS])
    }

    /// A table read from `text` and checked against the module's `categories` and `settings` (names after the
    /// module's id): at least one row; row ids in kebab case, unique; `versions` a relative path of plain names ending
    /// in `*`; pointer places relative, plain, with `holds` and pin names single names; the keys of `.tool-versions`
    /// present and each once; each row's category one of `categories`, and each of `categories` used by a row; a
    /// switch one of `settings`.
    pub(crate) fn parse(
        text: &str,
        categories: &[(&'static str, DangerLevel)],
        settings: &[&'static str],
    ) -> Result<Self, ToolchainsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| ToolchainsError::Format(error.to_string()))?;
        if table.row.is_empty() {
            return Err(ToolchainsError::Empty);
        }
        let mut rows: Vec<Row> = Vec::with_capacity(table.row.len());
        for raw in table.row {
            let row = check_row(raw, categories, settings)?;
            if rows.iter().any(|known| known.id == row.id) {
                return Err(ToolchainsError::DuplicateId { id: row.id });
            }
            rows.push(row);
        }
        if let Some((category, _)) = categories.iter().find(|(id, _)| !rows.iter().any(|row| row.category == *id)) {
            return Err(ToolchainsError::CategoryWithoutRow { category: (*category).into() });
        }
        let kinds = kinds(&rows);
        Ok(Self { rows, kinds })
    }

    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// What each interest of [`interests`](Self::interests) is, by its index.
    pub(crate) fn kinds(&self) -> &[InterestKind] {
        &self.kinds
    }

    /// The interests, in the order of [`kinds`](Self::kinds): per row, its versions claimed whole and its pointers as
    /// items; then one marker file per pin name, each name once.
    pub(crate) fn interests(&self) -> Result<Vec<Interest>, InterestError> {
        self.kinds
            .iter()
            .map(|kind| match kind {
                InterestKind::Versions(row) => Interest::new(
                    Pattern::Under(KnownFolder::Home, self.rows[*row].versions.clone()),
                    Claim::Whole,
                    vec![],
                ),
                InterestKind::Pointer(row, place) => {
                    let pointer = &self.rows[*row].pointers[*place];
                    let interest =
                        Interest::new(Pattern::Under(KnownFolder::Home, pointer.place.clone()), Claim::Item, vec![])?;
                    match &pointer.holds {
                        Some(name) => interest.holding(vec![name.clone()]),
                        None => Ok(interest),
                    }
                }
                InterestKind::Pin(name) => Interest::new(Pattern::MarkerFile(name.clone()), Claim::Item, vec![]),
            })
            .collect()
    }
}

/// The kinds in the design's order: per row its versions then its pointers; then each pin name once.
fn kinds(rows: &[Row]) -> Vec<InterestKind> {
    let mut kinds = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        kinds.push(InterestKind::Versions(index));
        kinds.extend((0..row.pointers.len()).map(|place| InterestKind::Pointer(index, place)));
    }
    let mut seen = BTreeSet::new();
    for pin in rows.iter().flat_map(|row| &row.pins) {
        if seen.insert(pin.name.as_str()) {
            kinds.push(InterestKind::Pin(pin.name.clone()));
        }
    }
    kinds
}

/// One row checked on its own.
fn check_row(
    raw: RawRow,
    categories: &[(&'static str, DangerLevel)],
    settings: &[&'static str],
) -> Result<Row, ToolchainsError> {
    let id = raw.id;
    if !is_row_id(&id) {
        return Err(ToolchainsError::BadId { id });
    }
    check_path(&id, &raw.versions)?;
    if raw.versions.rsplit('/').next() != Some(STAR) {
        return Err(ToolchainsError::NoStar { id, path: raw.versions });
    }
    for pointer in &raw.pointers {
        check_path(&id, &pointer.place)?;
        if let Some(name) = &pointer.holds {
            check_name(&id, name)?;
        }
    }
    for pin in &raw.pins {
        check_name(&id, &pin.name)?;
        check_keys(&id, pin)?;
    }
    let Some(&(category, _)) = categories.iter().find(|(known, _)| *known == raw.category) else {
        return Err(ToolchainsError::UnknownCategory { id, category: raw.category });
    };
    let switch = match raw.switch {
        None => None,
        Some(name) => match settings.iter().find(|known| **known == name) {
            Some(&switch) => Some(switch),
            None => return Err(ToolchainsError::UnknownSwitch { id, switch: name }),
        },
    };
    Ok(Row {
        id,
        manager: raw.manager,
        versions: raw.versions,
        shape: raw.shape,
        pointers: raw.pointers,
        pins: raw.pins,
        category,
        switch,
    })
}

/// A `.tool-versions` pin names its keys, each once; other pins name none that matter.
fn check_keys(id: &str, pin: &Pin) -> Result<(), ToolchainsError> {
    if pin.name != TOOL_VERSIONS {
        return Ok(());
    }
    if pin.keys.is_empty() {
        return Err(ToolchainsError::KeysMissing { id: id.into(), name: pin.name.clone() });
    }
    let mut seen = BTreeSet::new();
    for key in &pin.keys {
        check_name(id, key)?;
        if !seen.insert(key.as_str()) {
            return Err(ToolchainsError::DuplicateKey { id: id.into(), name: pin.name.clone(), key: key.clone() });
        }
    }
    Ok(())
}

fn check_path(id: &str, path: &str) -> Result<(), ToolchainsError> {
    if path.split('/').all(is_plain_name) {
        Ok(())
    } else {
        Err(ToolchainsError::NotRelative { id: id.into(), path: path.into() })
    }
}

fn check_name(id: &str, name: &str) -> Result<(), ToolchainsError> {
    if is_plain_name(name) && !name.contains(' ') {
        Ok(())
    } else {
        Err(ToolchainsError::NotAName { id: id.into(), name: name.into() })
    }
}

/// A row id: lowercase letters and digits in segments joined by single dashes.
fn is_row_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .split('-')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    row: Vec<RawRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawRow {
    id: String,
    manager: Manager,
    versions: String,
    shape: Shape,
    #[serde(default)]
    pointers: Vec<PointerPlace>,
    #[serde(default)]
    pins: Vec<Pin>,
    category: String,
    switch: Option<String>,
}

#[cfg(test)]
mod tests;
