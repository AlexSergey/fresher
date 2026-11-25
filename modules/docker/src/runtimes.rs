//! The Docker runtimes the module describes, as data: `runtimes.toml`, parsed and checked when the module is built.

use std::path::PathBuf;

use fresher_core::{BundleId, Claim, Interest, InterestError, KnownFolder, Pattern, TeamId, TextKey};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../runtimes.toml");

/// The longest runtime name taken, in characters.
const MAX_NAME: usize = 40;

/// Why a runtimes table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RuntimesError {
    #[error("the runtimes table is not valid TOML: {0}")]
    Format(String),
    #[error("the runtimes table lists no runtime")]
    Empty,
    #[error("`{id}` is not a row id in kebab case")]
    BadId { id: String },
    #[error("the row id `{id}` is listed twice")]
    DuplicateId { id: String },
    #[error("the row `{id}` has a name that is empty, longer than {MAX_NAME} characters or holds a control character")]
    BadName { id: String },
    #[error("the row `{id}` names `{app}`, which is not a bundle id")]
    BadApp { id: String, app: String },
    #[error("the row `{id}` names the team `{team}`, which is not a developer's team id")]
    BadTeam { id: String, team: String },
    #[error("the row `{id}` names the CLI `{cli}`, which is not plain folder names relative to the app")]
    BadCli { id: String, cli: String },
    #[error("the row `{id}` names the context `{context}`, which is not lowercase letters, digits and `-`")]
    BadContext { id: String, context: String },
    #[error("the row `{id}` names the place `{path}`, which is not a plain relative path under a known folder")]
    BadPlace { id: String, path: String },
    #[error("the row `{id}` names a place that another place of the table already names")]
    DuplicatePlace { id: String },
}

/// Whether the disk file's length is the size limit the user set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Maximum {
    /// It is: the card shows it as the largest the disk file may grow to.
    Shown,
    /// It is not: the card does not show it.
    Hidden,
}

/// A place under a known folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) folder: KnownFolder,
    pub(crate) path: PathBuf,
}

/// One row of the table: a runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Runtime {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) app: BundleId,
    /// The developer whose signature the CLI must carry before it is run: any bundle may claim the bundle id.
    pub(crate) team: TeamId,
    /// The runtime's own Docker CLI, relative to its app bundle.
    pub(crate) cli: PathBuf,
    pub(crate) context: String,
    pub(crate) socket: Place,
    pub(crate) disk: Place,
    pub(crate) maximum: Maximum,
}

/// What an interest of the module watches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Watched {
    Socket,
    Disk,
}

/// Every row of the table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Runtimes {
    rows: Vec<Runtime>,
}

impl Runtimes {
    pub(crate) fn shipped() -> Result<Self, RuntimesError> {
        Self::parse(SHIPPED)
    }

    /// A table read from `text`: at least one row; ids in kebab case, unique; names, apps, CLIs, contexts and places
    /// as the refusals say; no place named twice.
    pub(crate) fn parse(text: &str) -> Result<Self, RuntimesError> {
        let table: RawTable = toml::from_str(text).map_err(|error| RuntimesError::Format(error.to_string()))?;
        if table.runtime.is_empty() {
            return Err(RuntimesError::Empty);
        }
        let mut rows: Vec<Runtime> = Vec::with_capacity(table.runtime.len());
        for raw in table.runtime {
            let row = check_row(raw)?;
            if rows.iter().any(|other| other.id == row.id) {
                return Err(RuntimesError::DuplicateId { id: row.id });
            }
            let named = |place: &Place| rows.iter().any(|other| other.socket == *place || other.disk == *place);
            if named(&row.socket) || named(&row.disk) || row.socket == row.disk {
                return Err(RuntimesError::DuplicatePlace { id: row.id });
            }
            rows.push(row);
        }
        Ok(Self { rows })
    }

    /// Every row, in the table's order.
    pub(crate) fn rows(&self) -> &[Runtime] {
        &self.rows
    }

    /// The interests, in the table's order: per row the socket, then the disk file, each one item.
    pub(crate) fn interests(&self) -> Result<Vec<Interest>, InterestError> {
        let mut interests = Vec::with_capacity(self.rows.len() * 2);
        for row in &self.rows {
            for place in [&row.socket, &row.disk] {
                let path = place.path.to_string_lossy().into_owned();
                interests.push(Interest::new(Pattern::Under(place.folder, path), Claim::Item, Vec::new())?);
            }
        }
        Ok(interests)
    }

    /// The row an interest belongs to and what it watches: index `2 × row` is the socket, `2 × row + 1` the disk file.
    pub(crate) fn of_interest(&self, index: usize) -> Option<(&Runtime, Watched)> {
        let watched = if index.is_multiple_of(2) { Watched::Socket } else { Watched::Disk };
        self.rows.get(index / 2).map(|row| (row, watched))
    }
}

/// One row checked on its own.
fn check_row(raw: RawRow) -> Result<Runtime, RuntimesError> {
    let id = raw.id;
    if id.contains('.') || TextKey::new(format!("docker.runtime.{id}.gives-back")).is_err() {
        return Err(RuntimesError::BadId { id });
    }
    let name = raw.name;
    if name.is_empty() || name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
        return Err(RuntimesError::BadName { id });
    }
    let Ok(app) = BundleId::new(raw.app.as_str()) else {
        return Err(RuntimesError::BadApp { id, app: raw.app });
    };
    let Ok(team) = TeamId::new(raw.team.as_str()) else {
        return Err(RuntimesError::BadTeam { id, team: raw.team });
    };
    if !plain_names(&raw.cli) {
        return Err(RuntimesError::BadCli { id, cli: raw.cli });
    }
    let context = raw.context;
    if context.is_empty() || !context.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
        return Err(RuntimesError::BadContext { id, context });
    }
    let socket = check_place(&id, raw.socket)?;
    let disk = check_place(&id, raw.disk)?;
    Ok(Runtime { id, name, app, team, cli: PathBuf::from(raw.cli), context, socket, disk, maximum: raw.maximum })
}

fn check_place(id: &str, raw: RawPlace) -> Result<Place, RuntimesError> {
    if raw.folder.is_walked_place() || !plain_names(&raw.path) {
        return Err(RuntimesError::BadPlace { id: id.into(), path: raw.path });
    }
    Ok(Place { folder: raw.folder, path: PathBuf::from(raw.path) })
}

/// A relative path of plain names: none empty, `.`, `..` or holding `*`, so never absolute.
fn plain_names(path: &str) -> bool {
    path.split('/').all(|name| !name.is_empty() && name != "." && name != ".." && !name.contains(['\0', '*']))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    runtime: Vec<RawRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawRow {
    id: String,
    name: String,
    app: String,
    team: String,
    cli: String,
    context: String,
    socket: RawPlace,
    disk: RawPlace,
    maximum: Maximum,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawPlace {
    folder: KnownFolder,
    path: String,
}

#[cfg(test)]
mod tests;
