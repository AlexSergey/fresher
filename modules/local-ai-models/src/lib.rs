//! The `local-ai-models` module: the models Hugging Face and LM Studio downloaded.
//!
//! It offers one finding per model, found in the default store of each tool (`stores.toml`): Hugging Face's hub cache
//! and LM Studio's models folder, the current place and the older one. A store moved by an environment variable or the
//! tool's settings is not found, since the module reads neither. It runs no tool command and has nothing for Ollama,
//! and it reads no file: what the user confirms goes to the Trash, and a model comes back by download.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod name;
mod report;
mod stores;

use fresher_core::{
    CategoryDescriptor, CategoryId, DescriptorError, FindingSink, IdError, Interest, InterestError, Locales, Match,
    Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, TextKeyError,
};

use crate::categories::CATEGORIES;
use crate::decide::{NoSystem, decide};
use crate::gathered::{Candidate, Gathered};
use crate::report::Report;
use crate::stores::Stores;
pub use crate::stores::StoresError;

/// Why the `local-ai-models` module could not be built (a bug in its own constants or stores table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum LocalAiModelsError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Stores(#[from] StoresError),
}

/// The `local-ai-models` module.
#[derive(Debug)]
pub struct LocalAiModelsModule {
    descriptor: ModuleDescriptor,
    stores: Stores,
    /// One per row of `stores`, in its order.
    interests: Vec<Interest>,
    report: Report,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl LocalAiModelsModule {
    /// The module, with its three categories, its left-out reasons, no setting beyond its switch, and the interests
    /// of its stores table.
    ///
    /// # Errors
    ///
    /// A [`LocalAiModelsError`] when one of its own constants or its stores table is invalid: a bug, which its tests
    /// catch.
    pub fn new() -> Result<Self, LocalAiModelsError> {
        let id = ModuleId::new("local-ai-models")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, category)?, danger)))
            .collect::<Result<Vec<_>, _>>()?;
        let report = Report::new(categories.clone())?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, categories, Vec::new(), locales)?
            .with_problems(report.problems())?;
        let stores = Stores::shipped()?;
        let interests = stores.interests()?;
        Ok(Self { descriptor, stores, interests, report, gathered: Gathered::default() })
    }
}

impl Module for LocalAiModelsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a store's folder for the scan's `finish`; a match of no interest is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if matched.interest < self.stores.stores().len() {
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

    /// Decides every folder of the scan, sorted by path: offered, left out with the reason, or not claimed.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let candidates = self.gathered.take(cx.cancel());
        let rows = self.stores.stores();
        let names_app = |candidate: &Candidate| rows.get(candidate.row).is_some_and(|row| row.app.is_some());
        // Read once, and only when a row that names an app has a folder. Otherwise it is not read, and `decide`, which
        // reads it only for such a row, never looks at the value given in its place.
        let running = if candidates.iter().any(names_app) {
            cx.system().running_apps().map_err(|_| NoSystem)
        } else {
            Err(NoSystem)
        };
        for candidate in &candidates {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let Some(row) = rows.get(candidate.row) else { continue };
            self.report.tell(row, candidate, decide(candidate, row, running), out);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
