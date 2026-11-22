//! The `docker` module: what Docker keeps on the Mac, described and never cleaned.
//!
//! For each Docker runtime of its table (`runtimes.toml`) it describes, in an insight, the runtime's disk file and,
//! while the runtime runs, what is inside it and the Docker command that would free each part. It removes nothing and
//! offers nothing: no finding, no left-out item, no command of its own to run for the user.
//!
//! It never starts a runtime. The only program it runs is the runtime's own CLI, `--context <context> system df
//! --format json`, and only while the runtime's app runs and its socket is present; never in a restored scan.

#![forbid(unsafe_code)]

mod answer;
mod decide;
mod gathered;
mod report;
mod runtimes;

use std::cell::OnceCell;

use fresher_core::{
    DescriptorError, FindingSink, IdError, InstalledApp, Interest, InterestError, Locales, Match, Module,
    ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, PlaceSeen, RunningSet,
};

use crate::decide::{Disk, NoSystem, State, Verdict, answered, decide};
use crate::gathered::{Gathered, Seen, Spotted};
pub use crate::runtimes::RuntimesError;
use crate::runtimes::{Runtime, Runtimes};

/// The arguments of the only call the module makes, after `--context <context>`.
const READ: [&str; 4] = ["system", "df", "--format", "json"];

/// Why the `docker` module could not be built (a bug in its own constants or table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DockerError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Runtimes(#[from] RuntimesError),
}

/// The `docker` module.
#[derive(Debug)]
pub struct DockerModule {
    descriptor: ModuleDescriptor,
    runtimes: Runtimes,
    /// Two per runtime, in the table's order: its socket, then its disk file.
    interests: Vec<Interest>,
    /// What the matches of the scan showed, waiting for its `finish`.
    gathered: Gathered,
}

impl DockerModule {
    /// The module with no category, no setting beyond its switch and no problem, and its table's interests.
    ///
    /// # Errors
    ///
    /// A [`DockerError`] when one of its own constants or its table is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, DockerError> {
        let id = ModuleId::new("docker")?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, Vec::new(), Vec::new(), locales)?;
        // A bad table fails here, when the module is built.
        let runtimes = Runtimes::shipped()?;
        let interests = runtimes.interests()?;
        Ok(Self { descriptor, runtimes, interests, gathered: Gathered::default() })
    }
}

impl Module for DockerModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a runtime's socket or disk file for the scan's `finish`; anything else at their places is not seen.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let Some((_, watched)) = self.runtimes.of_interest(matched.interest) else { return Ok(()) };
        if let Some(spotted) = Spotted::of(matched.interest / 2, watched, matched.kind, matched.sizes) {
            self.gathered.add(cx.cancel(), spotted);
        }
        Ok(())
    }

    /// Decides each runtime in the table's order and describes its card; asks a runtime only when the decision says
    /// so. It never pushes a finding or a left-out.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let seen = self.gathered.take(cx.cancel());
        let installed: OnceCell<Result<&[InstalledApp], NoSystem>> = OnceCell::new();
        let running: OnceCell<Result<&RunningSet, NoSystem>> = OnceCell::new();
        for (index, row) in self.runtimes.rows().iter().enumerate() {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let apps = *installed.get_or_init(|| cx.system().installed_apps().map_err(|_| NoSystem));
            let app = match apps.map(|apps| apps.iter().find(|app| app.bundle_id == row.app)) {
                // Not found in a partial listing is not "not installed": a left-behind card needs the whole list.
                Ok(None) if cx.system().installed_apps_complete().is_err() => Err(NoSystem),
                found => found,
            };
            let is_running = || {
                let set = *running.get_or_init(|| cx.system().running_apps().map_err(|_| NoSystem));
                set.map(|set| set.apps().iter().any(|running| running.bundle_ids.contains(&row.app)))
            };
            let disk = disk(&seen, index, cx);
            let state = match decide(app, is_running, seen.socket(index), disk, cx.is_restored()) {
                Verdict::Silent => continue,
                Verdict::Describe(state) => state,
                Verdict::Ask => match app {
                    Ok(Some(app)) => ask(row, app, cx)?,
                    Ok(None) | Err(NoSystem) => continue,
                },
            };
            // A card that cannot be built is a bug in the module's own table, which its tests catch: it is skipped.
            if let Ok(insight) = report::insight(row, &state, &disk) {
                out.describe(insight);
            }
        }
        Ok(())
    }
}

/// What the walk saw of the disk file of row `index`: seen when matched; otherwise unseen when its place could not be
/// read or was not entered, absent when it was read or is absent (or not known, in a restored scan).
fn disk(seen: &Seen, index: usize, cx: &ModuleContext<'_>) -> Disk {
    if let Some(sizes) = seen.disk(index) {
        return Disk::Seen(sizes);
    }
    match cx.place_seen(index * 2 + 1) {
        PlaceSeen::Unread | PlaceSeen::NotScanned => Disk::Unseen,
        PlaceSeen::Read | PlaceSeen::Absent | PlaceSeen::Unknown => Disk::Absent,
    }
}

/// Asks the runtime of `row`, whose app is `app`, what it keeps: its own CLI inside the app bundle, with exactly
/// `--context <context> system df --format json`. A program path that is not UTF-8 is no answer.
fn ask(row: &Runtime, app: &InstalledApp, cx: &ModuleContext<'_>) -> Result<State, ModuleError> {
    let program = app.path.join(&row.cli);
    let Some(program) = program.to_str() else { return Ok(State::NoAnswer) };
    let args: Vec<&str> = ["--context", row.context.as_str()].into_iter().chain(READ).collect();
    answered(cx.commands().run(program, &args, cx.cancel())).map_err(|_| ModuleError::Cancelled)
}

#[cfg(test)]
mod tests;
