//! What the module tells the engine about a folder it decided: a finding that empties the folder into the Trash, or a
//! left-out line in its category with its sizes. The text keys are built once, with the module.

use fresher_core::{
    CategoryDescriptor, FindingSink, LeftOutItem, NewFinding, Removal, Subject, Text, TextArg, TextKey, TextKeyError,
};

use crate::decide::{Reason, Verdict};
use crate::gathered::Candidate;
use crate::places::Row;

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    special: TextKey,
    not_yours: TextKey,
    no_system: TextKey,
    app_open: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |reason: Reason| TextKey::new(reason.key());
        Ok(Self {
            special: key(Reason::Special)?,
            not_yours: key(Reason::NotYours)?,
            no_system: key(Reason::NoSystem)?,
            app_open: key(Reason::AppOpen)?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::Special => &self.special,
            Reason::NotYours => &self.not_yours,
            Reason::NoSystem => &self.no_system,
            Reason::AppOpen => &self.app_open,
        }
    }
}

/// The module's categories and the keys of every text a finding or a left-out carries.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Vec<CategoryDescriptor>,
    title_own: TextKey,
    title_part: TextKey,
    reason: TextKey,
    left_out: LeftOutKeys,
}

impl Report {
    /// The texts of the module whose categories are `categories`.
    pub(crate) fn new(categories: Vec<CategoryDescriptor>) -> Result<Self, TextKeyError> {
        Ok(Self {
            categories,
            title_own: TextKey::new("app-caches.title.own")?,
            title_part: TextKey::new("app-caches.title.part")?,
            reason: TextKey::new("app-caches.reason")?,
            left_out: LeftOutKeys::new()?,
        })
    }

    /// The keys of the left-out reasons, in the order the module declares them.
    pub(crate) fn problems(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.left_out.of(reason).clone()).collect()
    }

    /// Reports `verdict` on `candidate`, a folder of `row`'s place: a finding, a left-out in the row's category, or
    /// nothing.
    pub(crate) fn tell(&self, row: &Row, candidate: &Candidate, verdict: Verdict<'_>, out: &mut dyn FindingSink) {
        let Some(category) = self.categories.iter().find(|category| category.id().as_str() == row.category) else {
            return;
        };
        match verdict {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(reason) => {
                let reason = self.left_out.of(reason).clone();
                out.left_out(LeftOutItem { category: category.id(), reason, sizes: candidate.sizes });
            }
            Verdict::Offered(owner) => {
                let Some(folder) = candidate.path.file_name().and_then(|name| name.to_str()) else { return };
                let app = TextArg::Text(owner.app.name.clone());
                let title = if owner.own {
                    Text::new(self.title_own.clone()).with("app", app.clone())
                } else {
                    Text::new(self.title_part.clone())
                        .with("app", app.clone())
                        .with("folder", TextArg::Text(folder.to_owned()))
                };
                let reason = Text::new(self.reason.clone())
                    .with("app", app)
                    .with("size", TextArg::Size(candidate.sizes.logical));
                let identity = format!("{}/{folder}", row.id);
                let subject = Subject::Path(candidate.path.clone());
                let removal = Removal::EmptyFolder(candidate.path.clone());
                let finding = NewFinding::new(category, identity, subject, candidate.sizes, title, reason, removal);
                out.push(finding.for_app(owner.app.bundle_id.clone()));
            }
        }
    }
}
