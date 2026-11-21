//! What the module tells the engine about a folder it decided: a finding to the Trash, or a left-out line in its
//! category with its sizes. The text keys are built once, with the module.

use std::path::Path;

use fresher_core::{
    CategoryDescriptor, FindingSink, LeftOutItem, NewFinding, Removal, Subject, Text, TextArg, TextKey, TextKeyError,
};

use crate::decide::{self, Reason, Verdict};
use crate::gathered::Candidate;
use crate::stores::{Store, Tool};

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    not_yours: TextKey,
    special: TextKey,
    app_open: TextKey,
    no_system: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |reason: Reason| TextKey::new(reason.key());
        Ok(Self {
            not_yours: key(Reason::NotYours)?,
            special: key(Reason::Special)?,
            app_open: key(Reason::AppOpen)?,
            no_system: key(Reason::NoSystem)?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::NotYours => &self.not_yours,
            Reason::Special => &self.special,
            Reason::AppOpen => &self.app_open,
            Reason::NoSystem => &self.no_system,
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
