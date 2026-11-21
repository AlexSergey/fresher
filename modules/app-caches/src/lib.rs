//! The `app-caches` module: the caches installed apps keep in the user's Library.
//!
//! It offers, one finding per folder, the folders `~/Library/Caches/<bundle id>` that an installed app provably owns:
//! the app's own id, a helper's, or an id under one of them. They are offered only while the app and its helpers are
//! closed, at Low danger, and removing one empties the folder into the Trash and keeps the folder. It never claims a
//! name that is not a bundle id, anything of Apple's or Fresher's own, a folder whose owner is not an installed app
//! (a removed app's cache is `app-leftovers`' own), nor any folder while its app runs. It runs no command.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod exclude;
mod gathered;
mod owner;
mod places;
mod report;

use std::cell::OnceCell;

use fresher_core::{
    CategoryDescriptor, CategoryId, DescriptorError, FindingSink, IdError, Interest, InterestError, Locales, Match,
    Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, RunningSet, TextKeyError,
};

use crate::categories::CATEGORIES;
use crate::decide::{NoSystem, decide};
use crate::exclude::Exclude;
pub use crate::exclude::ExcludeError;
use crate::gathered::{Candidate, Gathered};
use crate::places::Places;
pub use crate::places::PlacesError;
use crate::report::Report;

/// Why the `app-caches` module could not be built (a bug in its own constants or tables).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppCachesError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Places(#[from] PlacesError),
    #[error(transparent)]
    Exclude(#[from] ExcludeError),
}

/// The `app-caches` module.
#[derive(Debug)]
pub struct AppCachesModule {
    descriptor: ModuleDescriptor,
    places: Places,
    exclude: Exclude,
    /// One interest per row of `places`, in its order.
    interests: Vec<Interest>,
    report: Report,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl AppCachesModule {
    /// The module with its category, its left-out reasons, no setting beyond its switch, its tables and its
    /// interests.
    ///
    /// # Errors
    ///
    /// An [`AppCachesError`] when one of its own constants or tables is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, AppCachesError> {
        let id = ModuleId::new("app-caches")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, category)?, danger)))
            .collect::<Result<Vec<_>, _>>()?;
        let report = Report::new(categories.clone())?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::System, categories, Vec::new(), locales)?
            .with_problems(report.problems())?;
        // A bad table fails here, when the module is built.
        let places = Places::shipped()?;
        let exclude = Exclude::shipped()?;
        let interests = places.interests()?;
        Ok(Self { descriptor, places, exclude, interests, report, gathered: Gathered::default() })
    }
}

impl Module for AppCachesModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a match of a place for the scan's `finish`; a match of no interest is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if matched.interest < self.places.rows().len() {
            let candidate = Candidate {
                row: matched.interest,
                path: matched.path.clone(),
                kind: matched.kind,
                sizes: matched.sizes,
                facts: matched.facts,
                inside: matched.inside,
            };
            self.gathered.add(cx.cancel(), candidate);
        }
        Ok(())
    }

    /// Decides every folder of the scan, sorted by path: offered, left out with the reason, or not claimed. The
    /// installed apps are asked once, and only when something was gathered; the running apps at most once, and only
    /// when a folder reaches the question.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let candidates = self.gathered.take(cx.cancel());
        if candidates.is_empty() {
            return Ok(());
        }
        let apps = cx.system().installed_apps().map_err(|_| NoSystem);
        let running: OnceCell<Result<&RunningSet, NoSystem>> = OnceCell::new();
        let rows = self.places.rows();
        for candidate in &candidates {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let Some(row) = rows.get(candidate.row) else { continue };
            let ask = || *running.get_or_init(|| cx.system().running_apps().map_err(|_| NoSystem));
            self.report.tell(row, candidate, decide(candidate, &self.exclude, apps, ask), out);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
