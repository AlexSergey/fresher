//! The only code of the module that reads a file: the pointer and pin files its matches showed, each through the
//! small-file reader and at most once a scan, resolved per manager into what keeps each installed version.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fresher_core::{EntryKind, SmallFile, SystemFacts};

use crate::decide::{Name, Version};
use crate::gathered::{Matches, PointerMatch};
use crate::pointers::rustup::{self, FileRef, Override};
use crate::pointers::tool_versions::{self, ToolVersion};
use crate::pointers::{Pointers, Resolved, nvm};
use crate::toolchains::{Manager, Pin, Row, TOOL_VERSIONS};

/// The name of nvm's default alias, under its alias folder.
const NVM_DEFAULT: &str = "default";

/// Reads the small files of one scan, each once: a pin two managers read is read for the first and kept for the
/// second.
pub(crate) struct Reader<'a> {
    system: &'a SystemFacts,
    read: BTreeMap<(PathBuf, String), SmallFile>,
}

impl<'a> Reader<'a> {
    pub(crate) const fn new(system: &'a SystemFacts) -> Self {
        Self { system, read: BTreeMap::new() }
    }

    /// The file `name` in `folder`, as its first read in this scan found it.
    fn file(&mut self, folder: &Path, name: &str) -> Read {
        let system = self.system;
        let file =
            self.read.entry((folder.to_path_buf(), name.to_owned())).or_insert_with(|| system.small_file(folder, name));
        Read::of(file)
    }

    /// The file `path` names, read as [`file`](Self::file) reads it; a path without a parent or a name is unknown.
    fn path(&mut self, path: &Path) -> Read {
        match (path.parent(), path.file_name().and_then(|name| name.to_str())) {
            (Some(folder), Some(name)) => self.file(folder, name),
            _ => Read::Unknown,
        }
    }
}

/// What a read file gives a pointer.
enum Read {
    Text(String),
    /// Gone since the walk saw it.
    Gone,
    /// It is there, but not a regular text file of at most the reader's limit, or it could not be read.
    Unknown,
}

impl Read {
    fn of(file: &SmallFile) -> Self {
        match file {
            SmallFile::Text(text) => Self::Text(text.clone()),
            SmallFile::Missing => Self::Gone,
            SmallFile::NotAFile | SmallFile::TooLong | SmallFile::NotText | SmallFile::Unreadable => Self::Unknown,
        }
    }
}

/// The installed versions of one row, by their index: every match of its versions place, in path order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Installed<'a> {
    /// The folder's name, `None` when it is not UTF-8.
    pub(crate) names: Vec<Option<&'a str>>,
    /// The name read by the row's shape, `None` when off it.
    pub(crate) versions: Vec<Option<Version>>,
}

/// The resolved pointers of the row at `index` in the toolchains table: its manager's default and other pointers,
/// and the pins of `matches` whose folder is not inside one of `artifacts`.
pub(crate) fn pointers(
    index: usize,
    row: &Row,
    installed: &Installed<'_>,
    matches: &Matches,
    reader: &mut Reader<'_>,
    artifacts: &[&Path],
) -> Pointers {
    let own: Vec<&PointerMatch> = matches.pointers.iter().filter(|pointer| pointer.row == index).collect();
    let pins = pins(row, matches, reader, artifacts);
    match row.manager {
        Manager::Rustup => rustup_pointers(row, installed, &own, &pins, reader),
        Manager::Nvm => nvm_pointers(row, installed, &own, &pins, reader),
    }
}

/// The pins of `row` the scan's matches showed, read: none inside an artifact folder, none gone since the walk.
fn pins<'r>(row: &'r Row, matches: &Matches, reader: &mut Reader<'_>, artifacts: &[&Path]) -> Vec<(&'r Pin, Read)> {
    let mut read = Vec::new();
    for (folder, names) in &matches.pins {
        if artifacts.iter().any(|artifact| folder.starts_with(artifact)) {
            continue;
        }
        for pin in row.pins.iter().filter(|pin| names.contains(&pin.name)) {
            match reader.file(folder, &pin.name) {
                Read::Gone => {}
                file => read.push((pin, file)),
            }
        }
    }
    read
}

/// What each of `pins` keeps, `resolve` reading the reference of a pin file of the manager's own format.
fn resolve_pins(pins: &[(&Pin, Read)], resolve: impl Fn(&Pin, &str) -> Resolved) -> Vec<Resolved> {
    let mut resolved = Vec::new();
    for (pin, file) in pins {
        let Read::Text(text) = file else {
            resolved.push(Resolved::Unknown);
            continue;
        };
        if pin.name != TOOL_VERSIONS {
            resolved.push(resolve(pin, text));
            continue;
        }
        let keys: Vec<&str> = pin.keys.iter().map(String::as_str).collect();
        resolved.extend(tool_versions::references(text, &keys).into_iter().map(|reference| match reference {
            ToolVersion::Version(version) => resolve(pin, version),
            ToolVersion::Nothing => Resolved::Nothing,
            ToolVersion::Unknown => Resolved::Unknown,
        }));
    }
    resolved
}

/// rustup: the default and the overrides of `settings.toml` (read only when the `.rustup` match holds it), then the
/// pins.
fn rustup_pointers(
    row: &Row,
    installed: &Installed<'_>,
    own: &[&PointerMatch],
    pins: &[(&Pin, Read)],
    reader: &mut Reader<'_>,
) -> Pointers {
    let settings = own.iter().find_map(|pointer| {
        let name = row.pointers.get(pointer.place)?.holds.as_ref()?;
        pointer.holds.contains(name).then_some((pointer, name))
    });
    let (default, overrides) = match settings.map(|(pointer, name)| reader.file(&pointer.path, name)) {
        Some(Read::Text(text)) => rustup::settings(&text),
        _ => (None, Vec::new()),
    };
    let default = match default.map(|default| toolchain(&default, installed)) {
        Some(Resolved::Keeps(kept)) if !kept.is_empty() => Resolved::Keeps(kept),
        _ => Resolved::Unknown,
    };
    let mut others: Vec<Resolved> = overrides
        .iter()
        .map(|value| match value {
            Override::Reference(reference) => toolchain(reference, installed),
            Override::Unreadable => Resolved::Unknown,
        })
        .collect();
    others.extend(resolve_pins(pins, |pin, text| {
        if pin.name == TOOL_VERSIONS {
            return toolchain(text, installed);
        }
        match rustup::toolchain_file(text) {
            FileRef::Channel(channel) => toolchain(&channel, installed),
            FileRef::Path => Resolved::Nothing,
            FileRef::Unknown => Resolved::Unknown,
        }
    }));
    Pointers { default, others }
}

/// What the rustup reference `text` keeps: the folder of that whole name, and every folder the reference names;
/// `Unknown` when it is no folder's name and does not parse.
fn toolchain(text: &str, installed: &Installed<'_>) -> Resolved {
    let mut kept: BTreeSet<usize> =
        installed.names.iter().enumerate().filter(|(_, name)| **name == Some(text)).map(|(index, _)| index).collect();
    match rustup::reference(text) {
        Some(reference) => kept.extend(installed.versions.iter().flatten().filter_map(|version| match &version.name {
            Name::Toolchain(folder) if rustup::keeps(&reference, folder) => Some(version.index),
            _ => None,
        })),
        None if kept.is_empty() => return Resolved::Unknown,
        None => {}
    }
    Resolved::Keeps(kept)
}

/// nvm: the alias files (`default` is the default, every other a pointer), then the pins.
fn nvm_pointers(
    row: &Row,
    installed: &Installed<'_>,
    own: &[&PointerMatch],
    pins: &[(&Pin, Read)],
    reader: &mut Reader<'_>,
) -> Pointers {
    let mut aliases = BTreeMap::new();
    let mut unknown = Vec::new();
    for pointer in own.iter().filter(|pointer| pointer.kind != EntryKind::Folder) {
        let key = alias_key(row, pointer);
        match (reader.path(&pointer.path), key) {
            (Read::Gone, _) => {}
            (Read::Text(text), Some(key)) => {
                aliases.insert(key, text);
            }
            (_, key) => unknown.push(key),
        }
    }
    let (positions, numbers): (Vec<usize>, Vec<[u32; 3]>) = installed
        .versions
        .iter()
        .flatten()
        .filter_map(|version| match version.name {
            Name::Node(numbers) => Some((version.index, numbers)),
            Name::Toolchain(_) => None,
        })
        .unzip();
    let resolve = |text: &str| match nvm::resolve(text, &aliases, &numbers) {
        Resolved::Keeps(kept) => Resolved::Keeps(kept.iter().filter_map(|at| positions.get(*at).copied()).collect()),
        other => other,
    };
    let default = if unknown.iter().any(|key| key.as_deref() == Some(NVM_DEFAULT)) {
        Resolved::Unknown
    } else {
        aliases.get(NVM_DEFAULT).map_or(Resolved::Nothing, |text| resolve(text))
    };
    let mut others: Vec<Resolved> =
        aliases.iter().filter(|(key, _)| key.as_str() != NVM_DEFAULT).map(|(_, text)| resolve(text)).collect();
    others.extend(unknown.iter().filter(|key| key.as_deref() != Some(NVM_DEFAULT)).map(|_| Resolved::Unknown));
    others.extend(resolve_pins(pins, |_, text| resolve(text)));
    Pointers { default, others }
}

/// The alias an alias file is, by its path under the alias folder (`default`, `lts/iron`): the last names of its
/// path, as many as its place has names after the folder of the row's first place. `None` when a name is not UTF-8.
fn alias_key(row: &Row, pointer: &PointerMatch) -> Option<String> {
    let base = row.pointers.first()?.place.split('/').count().checked_sub(1)?;
    let depth = row.pointers.get(pointer.place)?.place.split('/').count().checked_sub(base)?;
    let names: Vec<&str> =
        pointer.path.iter().rev().take(depth).map(|name| name.to_str()).collect::<Option<Vec<_>>>()?;
    (names.len() == depth).then(|| names.into_iter().rev().collect::<Vec<_>>().join("/"))
}
