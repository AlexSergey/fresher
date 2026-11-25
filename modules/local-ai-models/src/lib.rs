//! The `local-ai-models` module: the models Hugging Face, LM Studio and Ollama downloaded.
//!
//! It offers one finding per model, found in the default store of each tool (`stores.toml`): Hugging Face's hub cache,
//! LM Studio's models folder (the current place and the older one) and Ollama's `~/.ollama/models`. A store moved by an
//! environment variable or the tool's settings is not found, since the module reads neither. It runs no tool command
//! and sends no request. The only files it reads are Ollama's manifests its matches showed
//! (`manifests/<host>/<namespace>/<model>/<tag>`), each once a scan through the small-file reader, to tell which layer
//! files each model names: an Ollama model goes to the Trash as its manifests and the layers no other model names,
//! together. What the user confirms goes to the Trash, and most models come back by download.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod name;
mod ollama;
mod report;
mod stores;

use fresher_core::{
    CategoryDescriptor, CategoryIcon, CategoryId, DescriptorError, FindingSink, IdError, Interest, InterestError,
    Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, SmallFile,
    TextKeyError,
};

use crate::categories::CATEGORIES;
use crate::decide::{NoSystem, decide};
use crate::gathered::{Candidate, Gathered};
use crate::ollama::decide::{Entries, StoreVerdict};
use crate::report::Report;
pub use crate::stores::StoresError;
use crate::stores::{Shape, Slot, Store, Stores};

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
    /// One per row of `stores`, six for an Ollama row, in its order.
    interests: Vec<Interest>,
    /// What each of `interests` stands for, in the same order.
    slots: Vec<Slot>,
    report: Report,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl LocalAiModelsModule {
    /// The module, with its four categories, its left-out reasons, no setting beyond its switch, and the interests
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
            .map(|(category, danger)| {
                Ok::<_, IdError>(
                    CategoryDescriptor::new(CategoryId::new(id, category)?, danger).with_icon(CategoryIcon::Model),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let report = Report::new(categories.clone())?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, categories, Vec::new(), locales)?
            .with_problems(report.problems())?
            .with_kept(report.kept())?;
        let stores = Stores::shipped()?;
        let interests = stores.interests()?;
        let slots = stores.slots();
        Ok(Self { descriptor, stores, interests, slots, report, gathered: Gathered::default() })
    }
}

impl Module for LocalAiModelsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a store's folder, or an entry of an Ollama store, for the scan's `finish`; a match of no interest is not
    /// claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if let Some(slot) = self.slots.get(matched.interest) {
            let candidate = Candidate {
                row: slot.row,
                ollama: slot.ollama,
                path: matched.path.clone(),
                kind: matched.kind,
                sizes: matched.sizes,
                modified: matched.modified,
                accessed: matched.accessed,
                facts: matched.facts,
                holds: matched.holds.clone(),
                inside: matched.inside,
            };
            self.gathered.add(cx.cancel(), candidate);
        }
        Ok(())
    }

    /// Decides every folder of the scan, sorted by path: offered, left out with the reason, or not claimed; then
    /// each Ollama store, from its manifests read now.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let (ollama, candidates): (Vec<Candidate>, Vec<Candidate>) =
            self.gathered.take(cx.cancel()).into_iter().partition(|candidate| candidate.ollama.is_some());
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
        for (index, row) in rows.iter().enumerate().filter(|(_, row)| row.shape == Shape::OllamaStore) {
            self.finish_ollama(row, &Entries::new(ollama.iter().filter(|candidate| candidate.row == index)), cx, out)?;
        }
        Ok(())
    }
}

impl LocalAiModelsModule {
    /// Decides the store of `row` from `entries`, its matches, as far as it can without reading; when it can still be
    /// offered, reads every manifest through the small-file reader, then decides the store and, when it is open,
    /// each of its models in name order.
    fn finish_ollama(
        &self,
        row: &Store,
        entries: &Entries<'_>,
        cx: &ModuleContext<'_>,
        out: &mut dyn FindingSink,
    ) -> Result<(), ModuleError> {
        if let Some(verdict) = ollama::decide::kept(entries) {
            self.tell_ollama_store(row, entries, &verdict, out);
            return Ok(());
        }
        let mut read = Vec::with_capacity(entries.manifests().len());
        for (tag, candidate) in entries.manifests() {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let file = candidate
                .path
                .parent()
                .map_or(SmallFile::Unreadable, |folder| cx.system().small_file(folder, &tag.tag));
            read.push(ollama::manifest::read(file));
        }
        let layout = match ollama::decide::store(entries, read) {
            StoreVerdict::Open(layout) => layout,
            verdict => {
                self.tell_ollama_store(row, entries, &verdict, out);
                return Ok(());
            }
        };
        for (index, model) in layout.models().iter().enumerate() {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            self.report.tell_ollama_model(row, model, ollama::decide::model(entries, &layout, index), out);
        }
        Ok(())
    }

    /// Tells a verdict on the whole Ollama store of `row` that offers nothing: its left-out, or nothing.
    fn tell_ollama_store(&self, row: &Store, entries: &Entries<'_>, verdict: &StoreVerdict, out: &mut dyn FindingSink) {
        if let (StoreVerdict::LeftOut(reason, sizes), Some(path)) = (verdict, entries.store_path()) {
            self.report.tell_ollama_store(row, path, *reason, *sizes, out);
        }
    }
}

#[cfg(test)]
mod tests;
