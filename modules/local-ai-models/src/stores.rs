//! The stores of local AI models the module offers, as data: `stores.toml`, parsed and checked when the module is
//! built.

use std::collections::BTreeSet;

use fresher_core::{AppIdError, BundleId, Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../stores.toml");

/// The names an Ollama store folder holds: its layout.
pub(crate) const OLLAMA_LAYOUT: [&str; 2] = ["manifests", "blobs"];

/// The name a place's last part must be: any name.
const STAR: &str = "*";

/// Why a stores table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum StoresError {
    #[error("the stores table is not valid TOML: {0}")]
    Format(String),
    #[error("the stores table lists no store")]
    Empty,
    #[error("`{id}` is not a row id in kebab case")]
    BadId { id: String },
    #[error("the row id `{id}` is listed twice")]
    DuplicateId { id: String },
    #[error("the row `{id}` names `{category}`, which is not a category of the module")]
    UnknownCategory { id: String, category: String },
    #[error("the row `{id}` names no category")]
    CategoryMissing { id: String },
    #[error("the hub repository row `{id}` names a category; only its kinds may")]
    CategoryNotAllowed { id: String },
    #[error("the category `{category}` has no row")]
    CategoryWithoutRow { category: String },
    #[error("the place `{path}` of the Ollama row `{id}` has a `*`; its place is the store folder itself")]
    OllamaStar { id: String, path: String },
    #[error("the Ollama row `{id}` names a sign; its layout is known by its shape")]
    OllamaSign { id: String },
    #[error("the Ollama row `{id}` names an app")]
    OllamaApp { id: String },
    #[error("the row `{id}` names `{path}`, which is not a relative path of plain names")]
    NotRelative { id: String, path: String },
    #[error("the place `{path}` of the row `{id}` does not end in `*`")]
    NoStar { id: String, path: String },
    #[error("the place `{path}` of the row `{id}` does not fit its shape")]
    ShapeMismatch { id: String, path: String },
    #[error("the row `{id}` is a hub repository store and names no kinds")]
    KindsMissing { id: String },
    #[error("the row `{id}` names kinds, which only a hub repository store may")]
    KindsNotAllowed { id: String },
    #[error("the row `{id}` names the kind `{kind}` twice")]
    DuplicateKind { id: String, kind: String },
    #[error("`{name}` in the row `{id}` is not a single name")]
    NotAName { id: String, name: String },
    #[error(transparent)]
    BundleId(#[from] AppIdError),
}

/// The tool that keeps a store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Tool {
    HuggingFace,
    LmStudio,
    Ollama,
}

/// How a model's folder is named in a store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Shape {
    /// `<kind>--<owner>--<name>` in one folder: one `*`.
    HubRepo,
    /// `<publisher>/<model>`: two `*`.
    PublisherModel,
    /// The store folder itself, no `*`: `manifests/<host>/<namespace>/<model>/<tag>` and `blobs/sha256-<hex>`.
    OllamaStore,
}

impl Shape {
    /// How many names of a place are `*`, all of them the last ones.
    pub(crate) const fn stars(self) -> usize {
        match self {
            Self::HubRepo => 1,
            Self::PublisherModel => 2,
            Self::OllamaStore => 0,
        }
    }
}

/// Which of the six interests of an Ollama row a match is, in the order of the design's section 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OllamaInterest {
    /// The store folder, holding `manifests` and `blobs`.
    Store,
    /// A host's folder under `manifests`.
    Hosts,
    /// A namespace's folder.
    Namespaces,
    /// A model's folder.
    Models,
    /// A manifest, one file per tag.
    Manifests,
    /// A layer, or a partial download, under `blobs`.
    Blobs,
}

impl OllamaInterest {
    /// The six, in the order of their interests.
    const ALL: [Self; 6] = [Self::Store, Self::Hosts, Self::Namespaces, Self::Models, Self::Manifests, Self::Blobs];

    /// The place of the interest under the store folder `store`: the folder itself, or a depth of `manifests`, or
    /// `blobs`.
    fn place(self, store: &str) -> String {
        match self {
            Self::Store => store.to_owned(),
            Self::Hosts => format!("{store}/manifests/*"),
            Self::Namespaces => format!("{store}/manifests/*/*"),
            Self::Models => format!("{store}/manifests/*/*/*"),
            Self::Manifests => format!("{store}/manifests/*/*/*/*"),
            Self::Blobs => format!("{store}/blobs/*"),
        }
    }
}

/// What one interest of the module stands for: a row of the table and, for an Ollama row, which of its six.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) row: usize,
    pub(crate) ollama: Option<OllamaInterest>,
}

/// The categories of a row's models.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Categories {
    /// Every model is in this one.
    One(&'static str),
    /// A model is in the category of its name's kind, in the order the row lists them.
    ByKind(Vec<(String, &'static str)>),
}

/// One row of the table: a store of models, its shape, its sign and its app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Store {
    pub(crate) id: String,
    pub(crate) tool: Tool,
    pub(crate) folder: KnownFolder,
    pub(crate) path: String,
    pub(crate) shape: Shape,
    pub(crate) sign: Vec<String>,
    pub(crate) app: Option<BundleId>,
    pub(crate) categories: Categories,
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Stores {
    stores: Vec<Store>,
}

impl Stores {
    pub(crate) fn shipped() -> Result<Self, StoresError> {
        Self::parse(SHIPPED, &crate::categories::CATEGORIES)
    }

    /// A table read from `text` and checked against the module's `categories`: at least one row; row ids in kebab
    /// case, unique; `path` a relative path of plain names whose last names are `*` as its shape says; a hub
    /// repository row with its kinds, each once, and any other row with none; a sign of single names; a valid app;
    /// each row's category one of `categories`, and each of `categories` used by a row.
    pub(crate) fn parse(text: &str, categories: &[(&'static str, DangerLevel)]) -> Result<Self, StoresError> {
        let table: RawTable = toml::from_str(text).map_err(|error| StoresError::Format(error.to_string()))?;
        if table.store.is_empty() {
            return Err(StoresError::Empty);
        }
        let mut stores: Vec<Store> = Vec::with_capacity(table.store.len());
        for raw in table.store {
            let store = check_row(raw, categories)?;
            if stores.iter().any(|row| row.id == store.id) {
                return Err(StoresError::DuplicateId { id: store.id });
            }
            stores.push(store);
        }
        let uses = |id: &str| stores.iter().any(|store| store.categories.contains(id));
        if let Some((category, _)) = categories.iter().find(|(id, _)| !uses(id)) {
            return Err(StoresError::CategoryWithoutRow { category: (*category).into() });
        }
        Ok(Self { stores })
    }

    pub(crate) fn stores(&self) -> &[Store] {
        &self.stores
    }

    /// The interests, in the table's order: per row, its place claimed whole, looking for the sign when it has one; an
    /// Ollama row gives six, each claiming its entry alone (`Claim::Item`), the first holding `manifests` and `blobs`.
    pub(crate) fn interests(&self) -> Result<Vec<Interest>, InterestError> {
        let mut interests = Vec::new();
        for store in &self.stores {
            if store.shape == Shape::OllamaStore {
                for which in OllamaInterest::ALL {
                    let interest =
                        Interest::new(Pattern::Under(store.folder, which.place(&store.path)), Claim::Item, Vec::new())?;
                    interests.push(if which == OllamaInterest::Store {
                        interest.holding(OLLAMA_LAYOUT.map(str::to_owned).to_vec())?
                    } else {
                        interest
                    });
                }
            } else {
                let interest =
                    Interest::new(Pattern::Under(store.folder, store.path.clone()), Claim::Whole, Vec::new())?;
                interests.push(if store.sign.is_empty() { interest } else { interest.holding(store.sign.clone())? });
            }
        }
        Ok(interests)
    }

    /// What each interest of [`interests`](Self::interests) stands for, in the same order.
    pub(crate) fn slots(&self) -> Vec<Slot> {
        let mut slots = Vec::new();
        for (row, store) in self.stores.iter().enumerate() {
            if store.shape == Shape::OllamaStore {
                slots.extend(OllamaInterest::ALL.map(|which| Slot { row, ollama: Some(which) }));
            } else {
                slots.push(Slot { row, ollama: None });
            }
        }
        slots
    }
}

impl Categories {
    fn contains(&self, id: &str) -> bool {
        match self {
            Self::One(category) => *category == id,
            Self::ByKind(kinds) => kinds.iter().any(|(_, category)| *category == id),
        }
    }
}

/// One row checked on its own.
fn check_row(raw: RawStore, categories: &[(&'static str, DangerLevel)]) -> Result<Store, StoresError> {
    let id = raw.id;
    if !is_row_id(&id) {
        return Err(StoresError::BadId { id });
    }
    if !is_plain_path(&raw.path) {
        return Err(StoresError::NotRelative { id, path: raw.path });
    }
    check_place(&id, &raw.path, raw.shape)?;
    if raw.shape == Shape::OllamaStore {
        if !raw.sign.is_empty() {
            return Err(StoresError::OllamaSign { id });
        }
        if raw.app.is_some() {
            return Err(StoresError::OllamaApp { id });
        }
    }
    if let Some(name) = raw.sign.iter().find(|name| !is_plain_name(name)) {
        return Err(StoresError::NotAName { id, name: name.clone() });
    }
    let app = raw.app.map(BundleId::new).transpose()?;
    let categories = check_categories(&id, raw.shape, raw.category, raw.kind, categories)?;
    Ok(Store {
        id,
        tool: raw.tool,
        folder: raw.folder,
        path: raw.path,
        shape: raw.shape,
        sign: raw.sign,
        app,
        categories,
    })
}

/// An Ollama place has no `*`; any other ends in `*`, with as many `*` names as the shape has, and none before them.
fn check_place(id: &str, path: &str, shape: Shape) -> Result<(), StoresError> {
    if shape == Shape::OllamaStore {
        return if path.contains('*') {
            Err(StoresError::OllamaStar { id: id.into(), path: path.into() })
        } else {
            Ok(())
        };
    }
    let names: Vec<&str> = path.split('/').collect();
    if names.last() != Some(&STAR) {
        return Err(StoresError::NoStar { id: id.into(), path: path.into() });
    }
    let stars = names.iter().rev().take_while(|name| **name == STAR).count();
    let before = &names[..names.len() - stars];
    if stars != shape.stars() || before.is_empty() || before.iter().any(|name| name.contains('*')) {
        return Err(StoresError::ShapeMismatch { id: id.into(), path: path.into() });
    }
    Ok(())
}

fn check_categories(
    id: &str,
    shape: Shape,
    category: Option<String>,
    kinds: Vec<RawKind>,
    known: &[(&'static str, DangerLevel)],
) -> Result<Categories, StoresError> {
    let find = |name: String| match known.iter().find(|(known, _)| *known == name) {
        Some(&(category, _)) => Ok(category),
        None => Err(StoresError::UnknownCategory { id: id.into(), category: name }),
    };
    match shape {
        Shape::HubRepo => {
            if kinds.is_empty() {
                return Err(StoresError::KindsMissing { id: id.into() });
            }
            if category.is_some() {
                return Err(StoresError::CategoryNotAllowed { id: id.into() });
            }
            let mut seen = BTreeSet::new();
            let mut by_kind = Vec::with_capacity(kinds.len());
            for raw in kinds {
                if !is_plain_name(&raw.kind) || raw.kind.contains("--") {
                    return Err(StoresError::NotAName { id: id.into(), name: raw.kind });
                }
                if !seen.insert(raw.kind.clone()) {
                    return Err(StoresError::DuplicateKind { id: id.into(), kind: raw.kind });
                }
                by_kind.push((raw.kind, find(raw.category)?));
            }
            Ok(Categories::ByKind(by_kind))
        }
        Shape::PublisherModel | Shape::OllamaStore => {
            if !kinds.is_empty() {
                return Err(StoresError::KindsNotAllowed { id: id.into() });
            }
            let category = category.ok_or_else(|| StoresError::CategoryMissing { id: id.into() })?;
            find(category).map(Categories::One)
        }
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

/// A relative path of plain names, written once: no leading, trailing or doubled `/`, no `.` or `..`.
fn is_plain_path(path: &str) -> bool {
    path.split('/').all(is_plain_name)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    store: Vec<RawStore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawStore {
    id: String,
    tool: Tool,
    folder: KnownFolder,
    path: String,
    shape: Shape,
    #[serde(default)]
    sign: Vec<String>,
    app: Option<String>,
    category: Option<String>,
    #[serde(default)]
    kind: Vec<RawKind>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawKind {
    kind: String,
    category: String,
}

#[cfg(test)]
mod tests;
