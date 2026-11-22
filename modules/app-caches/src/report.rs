//! What the module tells the engine about a folder it decided: a finding that empties the folder into the Trash, or a
//! left-out line in its category with its sizes. The text keys are built once, with the module.

use std::path::Path;

use fresher_core::{
    CategoryDescriptor, FindingSink, LeftOutItem, NewFinding, Removal, Subject, Text, TextArg, TextKey, TextKeyError,
};

use crate::apps::AppRow;
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
    title_profile: TextKey,
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
            title_profile: TextKey::new("app-caches.title.profile")?,
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
    pub(crate) fn tell_place(&self, row: &Row, candidate: &Candidate, verdict: Verdict<'_>, out: &mut dyn FindingSink) {
        let Some(category) = self.category(row.category) else { return };
        let owner = match verdict {
            Verdict::NotClaimed => return,
            Verdict::LeftOut(reason) => return self.tell_left_out(category, reason, candidate, out),
            Verdict::Offered(owner) => owner,
        };
        let Some(folder) = candidate.path.file_name().and_then(|name| name.to_str()) else { return };
        let app = TextArg::Text(owner.app.name.clone());
        let title = if owner.own {
            Text::new(self.title_own.clone()).with("app", app)
        } else {
            Text::new(self.title_part.clone()).with("app", app).with("folder", TextArg::Text(folder.to_owned()))
        };
        let identity = format!("{}/{folder}", row.id);
        let finding = self.finding(category, identity, candidate, title, &owner.app.name);
        out.push(finding.for_app(owner.app.bundle_id.clone()));
    }

    /// Reports `verdict` on `candidate`, a cache folder of `row` of the apps table: a finding titled by the app, the
    /// profile (for a row with one) and the cache's name, a left-out in the row's category, or nothing.
    pub(crate) fn tell_app(
        &self,
        row: &AppRow,
        candidate: &Candidate,
        verdict: Verdict<'_>,
        out: &mut dyn FindingSink,
    ) {
        let Some(category) = self.category(row.category) else { return };
        let owner = match verdict {
            Verdict::NotClaimed => return,
            Verdict::LeftOut(reason) => return self.tell_left_out(category, reason, candidate, out),
            Verdict::Offered(owner) => owner,
        };
        let utf8 = |path: &Path| path.file_name().and_then(|name| name.to_str()).map(str::to_owned);
        let Some(cache) = utf8(&candidate.path) else { return };
        let app = TextArg::Text(owner.app.name.clone());
        let (identity, title) = match row.profile {
            None => {
                let title =
                    Text::new(self.title_part.clone()).with("app", app).with("folder", TextArg::Text(cache.clone()));
                (format!("{}/{cache}", row.id), title)
            }
            Some(index) => {
                // The profile is the name at the `*`'s place, counted back from the cache folder.
                let Some(back) = row.path.split('/').count().checked_sub(index) else { return };
                let Some(profile) = candidate.path.ancestors().nth(back).and_then(utf8) else { return };
                let title = Text::new(self.title_profile.clone())
                    .with("app", app)
                    .with("profile", TextArg::Text(profile.clone()))
                    .with("folder", TextArg::Text(cache.clone()));
                (format!("{}/{profile}/{cache}", row.id), title)
            }
        };
        let finding = self.finding(category, identity, candidate, title, &owner.app.name);
        out.push(finding.for_app(row.bundle_id.clone()));
    }

    /// The category of the id `category`, one of the module's.
    fn category(&self, category: &str) -> Option<&CategoryDescriptor> {
        self.categories.iter().find(|known| known.id().as_str() == category)
    }

    /// A left-out line in `category` for `reason`, with the candidate's sizes.
    fn tell_left_out(
        &self,
        category: &CategoryDescriptor,
        reason: Reason,
        candidate: &Candidate,
        out: &mut dyn FindingSink,
    ) {
        let reason = self.left_out.of(reason).clone();
        out.left_out(LeftOutItem { category: category.id(), reason, sizes: candidate.sizes });
    }

    /// A finding in `category` that empties the candidate's folder, with its sizes, `title` and the reason naming
    /// `app`.
    fn finding(
        &self,
        category: &CategoryDescriptor,
        identity: String,
        candidate: &Candidate,
        title: Text,
        app: &str,
    ) -> NewFinding {
        let reason = Text::new(self.reason.clone())
            .with("app", TextArg::Text(app.to_owned()))
            .with("size", TextArg::Size(candidate.sizes.logical));
        let subject = Subject::Path(candidate.path.clone());
        let removal = Removal::EmptyFolder(candidate.path.clone());
        NewFinding::new(category, identity, subject, candidate.sizes, title, reason, removal)
    }
}
