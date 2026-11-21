//! What the module tells the engine about a folder it decided: a finding to the Trash, or a left-out line in its
//! category with its sizes; and about Ollama's store and models: a finding whose parts go together, or a left-out.
//! The text keys are built once, with the module.

use std::path::Path;

use fresher_core::{
    CategoryDescriptor, FindingSink, LeftOutItem, NewFinding, Removal, Sizes, Subject, Text, TextArg, TextKey,
    TextKeyError,
};

use crate::decide::{self, Reason, Verdict};
use crate::gathered::Candidate;
use crate::name::OllamaTag;
use crate::ollama::decide::{ModelVerdict, Offer};
use crate::ollama::layers::Model;
use crate::stores::{Categories, Store, Tool};

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    not_yours: TextKey,
    special: TextKey,
    app_open: TextKey,
    no_system: TextKey,
    ollama_unreadable: TextKey,
    ollama_download: TextKey,
    ollama_incomplete: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |reason: Reason| TextKey::new(reason.key());
        Ok(Self {
            not_yours: key(Reason::NotYours)?,
            special: key(Reason::Special)?,
            app_open: key(Reason::AppOpen)?,
            no_system: key(Reason::NoSystem)?,
            ollama_unreadable: key(Reason::OllamaUnreadable)?,
            ollama_download: key(Reason::OllamaDownload)?,
            ollama_incomplete: key(Reason::OllamaIncomplete)?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::NotYours => &self.not_yours,
            Reason::Special => &self.special,
            Reason::AppOpen => &self.app_open,
            Reason::NoSystem => &self.no_system,
            Reason::OllamaUnreadable => &self.ollama_unreadable,
            Reason::OllamaDownload => &self.ollama_download,
            Reason::OllamaIncomplete => &self.ollama_incomplete,
        }
    }
}

/// The module's categories and the keys of every text a finding or a left-out carries.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Vec<CategoryDescriptor>,
    title: TextKey,
    reason_hugging_face: TextKey,
    reason_lm_studio: TextKey,
    reason_ollama: TextKey,
    reason_ollama_shared: TextKey,
    reason_ollama_shared_more: TextKey,
    left_out: LeftOutKeys,
}

impl Report {
    /// The texts of the module whose categories are `categories`.
    pub(crate) fn new(categories: Vec<CategoryDescriptor>) -> Result<Self, TextKeyError> {
        Ok(Self {
            categories,
            title: TextKey::new("local-ai-models.title")?,
            reason_hugging_face: TextKey::new("local-ai-models.reason.hugging-face")?,
            reason_lm_studio: TextKey::new("local-ai-models.reason.lm-studio")?,
            reason_ollama: TextKey::new("local-ai-models.reason.ollama")?,
            reason_ollama_shared: TextKey::new("local-ai-models.reason.ollama-shared")?,
            reason_ollama_shared_more: TextKey::new("local-ai-models.reason.ollama-shared-more")?,
            left_out: LeftOutKeys::new()?,
        })
    }

    /// The keys of the left-out reasons, in the order the module declares them.
    pub(crate) fn problems(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.left_out.of(reason).clone()).collect()
    }

    /// Reports `verdict` on `candidate`, a folder of `row`'s store: a finding, a left-out in the category of its name,
    /// or nothing.
    pub(crate) fn tell(&self, row: &Store, candidate: &Candidate, verdict: Verdict, out: &mut dyn FindingSink) {
        match verdict {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(reason) => {
                let category = decide::model(row, &candidate.path).and_then(|(category, _)| self.category(category));
                if let Some(category) = category {
                    let reason = self.left_out.of(reason).clone();
                    out.left_out(LeftOutItem { category: category.id(), reason, sizes: candidate.sizes });
                }
            }
            Verdict::Offered { category, name, seen } => {
                let Some(category) = self.category(category) else { return };
                let title = Text::new(self.title.clone()).with("name", TextArg::Text(name.title));
                let reason = match row.tool {
                    Tool::HuggingFace => &self.reason_hugging_face,
                    Tool::LmStudio => &self.reason_lm_studio,
                    // Ollama's store is told by `tell_ollama_store` and `tell_ollama_model`, never here.
                    Tool::Ollama => return,
                };
                let reason = Text::new(reason.clone())
                    .with("date", TextArg::Date(seen))
                    .with("size", TextArg::Size(candidate.sizes.logical));
                let identity = format!("{}/{}", row.id, folder_names(&candidate.path, row.shape.stars()));
                let subject = Subject::Path(candidate.path.clone());
                let removal = Removal::MoveToTrash(candidate.path.clone());
                out.push(NewFinding::new(category, identity, subject, candidate.sizes, title, reason, removal));
            }
        }
    }

    /// Reports that nothing of the Ollama store of `row` is offered, for `reason`, with the store's `sizes`.
    pub(crate) fn tell_ollama_store(&self, row: &Store, reason: Reason, sizes: Sizes, out: &mut dyn FindingSink) {
        if let Some(category) = self.ollama_category(row) {
            out.left_out(LeftOutItem { category: category.id(), reason: self.left_out.of(reason).clone(), sizes });
        }
    }

    /// Reports `verdict` on `model`, a model of the Ollama store of `row`: a finding whose parts go together, a
    /// left-out with the sizes of its parts, or nothing.
    pub(crate) fn tell_ollama_model(
        &self,
        row: &Store,
        model: &Model,
        verdict: ModelVerdict,
        out: &mut dyn FindingSink,
    ) {
        let Some(category) = self.ollama_category(row) else { return };
        let offer = match verdict {
            ModelVerdict::NotClaimed => return,
            ModelVerdict::LeftOut(reason, sizes) => {
                let reason = self.left_out.of(reason).clone();
                out.left_out(LeftOutItem { category: category.id(), reason, sizes });
                return;
            }
            ModelVerdict::Offered(offer) => offer,
        };
        let (Some(first), Some(manifest)) = (model.tags.first(), model.manifests.first()) else { return };
        let titles: Vec<String> = model.tags.iter().map(OllamaTag::title).collect();
        let title = Text::new(self.title.clone()).with("name", TextArg::Text(titles.join(", ")));
        let reason = self.ollama_reason(first, &offer);
        let subject = Subject::Path(manifest.clone());
        let Offer { parts, freed, .. } = offer;
        let removal = Removal::MoveTogether(parts);
        out.push(NewFinding::new(category, first.identity(), subject, freed, title, reason, removal));
    }

    /// The reason of an offered Ollama model, by how many other models share its layers: its date and freed size;
    /// with no sharer its first tag's title; with a sharer its whole size and the first sharer's title; with more,
    /// how many others.
    fn ollama_reason(&self, first: &OllamaTag, offer: &Offer) -> Text {
        let key = match offer.sharers.len() {
            0 => &self.reason_ollama,
            1 => &self.reason_ollama_shared,
            _ => &self.reason_ollama_shared_more,
        };
        let text = Text::new(key.clone())
            .with("date", TextArg::Date(offer.date))
            .with("size", TextArg::Size(offer.freed.logical));
        let Some((other, rest)) = offer.sharers.split_first() else {
            return text.with("name", TextArg::Text(first.title()));
        };
        let text = text.with("listed", TextArg::Size(offer.listed)).with("other", TextArg::Text(other.clone()));
        if rest.is_empty() {
            return text;
        }
        text.with("count", TextArg::Count(u64::try_from(rest.len()).unwrap_or(u64::MAX)))
    }

    /// The category of the Ollama store of `row`.
    fn ollama_category(&self, row: &Store) -> Option<&CategoryDescriptor> {
        match &row.categories {
            Categories::One(category) => self.category(category),
            Categories::ByKind(_) => None,
        }
    }

    fn category(&self, id: &str) -> Option<&CategoryDescriptor> {
        self.categories.iter().find(|category| category.id().as_str() == id)
    }
}

/// The last `count` names of `path`, joined by `/`: what tells a model apart in its store (two publishers may each
/// have a model of the same folder name).
fn folder_names(path: &Path, count: usize) -> String {
    let mut names: Vec<_> = path.iter().rev().take(count).map(|name| name.to_string_lossy()).collect();
    names.reverse();
    names.join("/")
}
