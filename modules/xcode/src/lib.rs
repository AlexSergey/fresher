//! The `xcode` module: the device support files, previews, simulator caches and archives Xcode keeps in the user's
//! Library.
//!
//! It offers, one finding per folder, the device support folders Xcode copies from a connected device (Low), its
//! `SwiftUI` previews and the simulators' shared caches (Safe, emptied and kept), and the archives of builds (High).
//! It never claims a simulator device or runtime, nor `DerivedData` (that is `dev-caches`'), nothing while Xcode or the
//! Simulator runs, and no folder whose name or sign it does not recognise. It runs no command.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod name;
mod places;
mod report;

use std::cell::OnceCell;

use fresher_core::{
    AppIdError, CategoryDescriptor, CategoryId, DescriptorError, FindingSink, IdError, Interest, InterestError,
    Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, RunningSet,
    TextKeyError,
};

use crate::categories::CATEGORIES;
use crate::decide::{NoSystem, Verdict, decide, recent};
use crate::gathered::{Candidate, Gathered};
use crate::places::Places;
pub use crate::places::PlacesError;
use crate::report::Report;

/// Why the `xcode` module could not be built (a bug in its own constants or tables).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum XcodeError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    AppId(#[from] AppIdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Places(#[from] PlacesError),
}

/// The `xcode` module.
#[derive(Debug)]
pub struct XcodeModule {
    descriptor: ModuleDescriptor,
    places: Places,
    /// One interest per row of `places`, in its order.
    interests: Vec<Interest>,
    report: Report,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl XcodeModule {
    /// The module with its categories, its left-out reasons, no setting beyond its switch, its table and its
    /// interests.
    ///
    /// # Errors
    ///
    /// An [`XcodeError`] when one of its own constants or tables is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, XcodeError> {
        let id = ModuleId::new("xcode")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, category)?, danger)))
            .collect::<Result<Vec<_>, _>>()?;
        let report = Report::new(categories.clone())?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, categories, Vec::new(), locales)?
            .with_problems(report.problems())?;
        // A bad table fails here, when the module is built.
        let places = Places::shipped()?;
        let interests = places.interests()?;
        Ok(Self { descriptor, places, interests, report, gathered: Gathered::default() })
    }
}

impl Module for XcodeModule {
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
                holds: matched.holds.clone(),
                inside: matched.inside,
            };
            self.gathered.add(cx.cancel(), candidate);
        }
        Ok(())
    }

    /// Decides every folder of the scan, sorted by path: offered, left out with the reason, or not claimed; then
    /// marks the newest device support versions and reports. The running apps are asked at most once, and only when a
    /// folder reaches the question. No other port is asked and no command is run.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let candidates = self.gathered.take(cx.cancel());
        let running: OnceCell<Result<&RunningSet, NoSystem>> = OnceCell::new();
        let rows = self.places.rows();
        let mut decided: Vec<(&places::Row, &Candidate, Verdict)> = Vec::with_capacity(candidates.len());
        for candidate in &candidates {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let Some(row) = rows.get(candidate.row) else { continue };
            let ask = || *running.get_or_init(|| cx.system().running_apps().map_err(|_| NoSystem));
            decided.push((row, candidate, decide(candidate, row, ask)));
        }
        let recent = recent(decided.iter().map(|(row, _, verdict)| (*row, verdict)));
        for (row, candidate, verdict) in &decided {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            self.report.tell(row, candidate, verdict, &recent, out);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
