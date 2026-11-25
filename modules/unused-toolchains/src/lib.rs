//! The `unused-toolchains` module: the toolchain versions a version manager installed and nothing points at.
//!
//! It offers one finding per version, found in the default place of each manager (`toolchains.toml`): the rustup
//! toolchains and, behind a switch that is off by default, the nvm Node versions. A version is offered only when
//! nothing it can see points at it: the manager's default and its other pointers, and the pin files of the scanned
//! folders. It runs no tool command, since a version goes to the Trash and comes back with the manager's install
//! command; it looks only at the default places, so a place moved by an environment variable is not found; and it
//! reads only the files its matches showed, each through the small-file reader and only when a scan finishes.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod pointers;
mod read;
mod report;
mod settings;
mod toolchains;

use std::path::Path;

use fresher_core::{
    CategoryDescriptor, CategoryIcon, CategoryId, DescriptorError, EntryKind, FindingSink, IdError, Interest,
    InterestError, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId,
    SettingError, TextKeyError,
};

use crate::categories::{CATEGORIES, needs};
use crate::decide::{Name, Scan, Version, decide};
use crate::gathered::{Candidate, Gathered, Matches, PointerMatch};
use crate::pointers::{Pointers, Resolved};
use crate::read::{Installed, Reader};
use crate::report::Report;
pub use crate::toolchains::ToolchainsError;
use crate::toolchains::{InterestKind, Row, Toolchains};

/// Why the `unused-toolchains` module could not be built (a bug in its own constants or table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum UnusedToolchainsError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Setting(#[from] SettingError),
    #[error(transparent)]
    Toolchains(#[from] ToolchainsError),
}

/// The `unused-toolchains` module.
#[derive(Debug)]
pub struct UnusedToolchainsModule {
    descriptor: ModuleDescriptor,
    toolchains: Toolchains,
    /// One per interest of `toolchains`, in the order of its kinds.
    interests: Vec<Interest>,
    report: Report,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl UnusedToolchainsModule {
    /// The module, with its two categories and their needs, its switch for Node versions, its left-out reasons and
    /// the interests of its table.
    ///
    /// # Errors
    ///
    /// An [`UnusedToolchainsError`] when one of its own constants or its table is invalid: a bug, which its tests
    /// catch.
    pub fn new() -> Result<Self, UnusedToolchainsError> {
        let id = ModuleId::new("unused-toolchains")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| {
                Ok::<_, IdError>(
                    CategoryDescriptor::new(CategoryId::new(id, category)?, danger)
                        .with_icon(CategoryIcon::Toolchain)
                        .with_needs(needs()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let report = Report::new(categories.clone())?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(
            id,
            ModuleGroup::Development,
            categories,
            vec![settings::node_versions_setting(id)?],
            locales,
        )?
        .with_problems(report.problems())?
        .with_kept(report.kept())?;
        let toolchains = Toolchains::shipped()?;
        let interests = toolchains.interests()?;
        Ok(Self { descriptor, toolchains, interests, report, gathered: Gathered::default() })
    }
}

impl Module for UnusedToolchainsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a version folder, a pointer or a pin folder for the scan's `finish`; a match of no interest is dropped.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let token = cx.cancel();
        match self.toolchains.kinds().get(matched.interest) {
            Some(InterestKind::Versions(row)) => self.gathered.add_candidate(
                token,
                Candidate {
                    row: *row,
                    path: matched.path.clone(),
                    kind: matched.kind,
                    sizes: matched.sizes,
                    facts: matched.facts,
                    inside: matched.inside,
                },
            ),
            Some(InterestKind::Pointer(row, place)) => self.gathered.add_pointer(
                token,
                PointerMatch {
                    row: *row,
                    place: *place,
                    path: matched.path.clone(),
                    kind: matched.kind,
                    holds: matched.holds.clone(),
                },
            ),
            Some(InterestKind::Pin(name)) => self.gathered.add_pin(token, matched.path.clone(), name),
            None => {}
        }
        Ok(())
    }

    /// Decides every version folder of the scan, in path order: offered, left out with the reason, or not claimed.
    /// Nothing is claimed before the projects are known; a row whose switch is off has no file read.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let matches = self.gathered.take(cx.cancel());
        let Some(projects) = cx.system().projects() else { return Ok(()) };
        let artifacts: Vec<&Path> =
            projects.iter().flat_map(|project| &project.artifacts).map(|artifact| artifact.path.as_path()).collect();
        let mut reader = Reader::new(cx.system());
        let rows: Vec<Decided<'_>> = self
            .toolchains
            .rows()
            .iter()
            .enumerate()
            .map(|(index, row)| Decided::new(index, row, &matches, cx, &mut reader, &artifacts))
            .collect();
        let mut seen = vec![0; rows.len()];
        for candidate in &matches.candidates {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let (Some(decided), Some(at)) = (rows.get(candidate.row), seen.get_mut(candidate.row)) else { continue };
            let version = decided.installed.versions.get(*at).and_then(Option::as_ref);
            *at += 1;
            let newest = version.is_some_and(|version| decided.newest == Some(version.index));
            let verdict = decide(candidate, decided.row, version, newest, &decided.pointers, decided.scan);
            self.report.tell(decided.row, candidate, verdict, out);
        }
        Ok(())
    }
}

/// What `finish` knows of one row before it decides its versions.
struct Decided<'a> {
    row: &'a Row,
    /// The row's candidates in path order, as `matches.candidates` lists them.
    installed: Installed<'a>,
    /// The index of the newest installed Node version.
    newest: Option<usize>,
    pointers: Pointers,
    scan: Scan,
}

impl<'a> Decided<'a> {
    /// The row at `index`: its installed versions, and its pointers read only when its switch is on.
    fn new(
        index: usize,
        row: &'a Row,
        matches: &'a Matches,
        cx: &ModuleContext<'_>,
        reader: &mut Reader<'_>,
        artifacts: &[&Path],
    ) -> Self {
        let own: Vec<_> = matches.candidates.iter().filter(|candidate| candidate.row == index).collect();
        let names: Vec<Option<&str>> =
            own.iter().map(|candidate| candidate.path.file_name().and_then(|name| name.to_str())).collect();
        let versions = names
            .iter()
            .enumerate()
            .map(|(at, name)| name.and_then(|name| Version::read(row.shape, at, name)))
            .collect();
        let installed = Installed { names, versions };
        let newest = installed
            .versions
            .iter()
            .flatten()
            .filter_map(|version| match version.name {
                Name::Node(numbers)
                    if own.get(version.index).is_some_and(|candidate| candidate.kind != EntryKind::File) =>
                {
                    Some((numbers, version.index))
                }
                _ => None,
            })
            .max()
            .map(|(_, at)| at);
        let switch_on = row.switch.is_none_or(|switch| settings::is_on(cx.settings(), switch));
        let pointers = if switch_on {
            read::pointers(index, row, &installed, matches, reader, artifacts)
        } else {
            Pointers { default: Resolved::Nothing, others: Vec::new() }
        };
        let scan = Scan { walk_complete: cx.is_walk_complete(), switch_on };
        Self { row, installed, newest, pointers, scan }
    }
}

#[cfg(test)]
mod tests;
