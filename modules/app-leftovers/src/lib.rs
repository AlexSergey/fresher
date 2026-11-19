//! The `app-leftovers` module: the caches, settings and data that apps which are no longer installed left in the
//! user's library.
//!
//! It offers a leftover only when its name is a bundle id, the id is proven to be an app's, no installed or running
//! app has it, Spotlight finds no app with it, and none of its files changed for the setting's days; everything
//! else it recognized it keeps and reports as left out, with the reason. It never claims a name that is not an id,
//! anything of `com.apple.`, group containers or launch agents, and it runs no command. It does not read app
//! containers: they neither prove an id nor are offered (see `places.toml`).

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod exclude;
mod gathered;
mod name;
mod places;

use fresher_core::{
    BundleId, CategoryDescriptor, CategoryId, DaySetting, DescriptorError, EntryKind, FindingSink, IdError, Interest,
    InterestError, LeftOutItem, Locales, Match, Module, ModuleChoice, ModuleContext, ModuleDescriptor, ModuleError,
    ModuleGroup, ModuleId, NewFinding, Removal, SettingError, Subject, Text, TextArg, TextKey, TextKeyError, Timestamp,
};

use crate::categories::CATEGORIES;
use crate::decide::{Facts, IdVerdict, ItemVerdict, NoSystem, Reason, Search, decide_id, decide_item};
use crate::exclude::Exclude;
pub use crate::exclude::ExcludeError;
use crate::gathered::{Gathered, Leftover};
pub use crate::places::PlacesError;
use crate::places::{Form, Places};

/// The category the choice `keep-settings` selects: the caches, logs and saved state alone.
const CACHES: &str = "app-leftovers.caches";

/// `app-leftovers.uninstalled-after-days`. Never fewer than a week: a removed app's files are not offered the day
/// after.
const UNINSTALLED_AFTER: DaySetting = DaySetting::new("uninstalled-after-days", 7, 3650, 30);

/// Why the `app-leftovers` module could not be built (a bug in its own constants or tables).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppLeftoversError {
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
    #[error(transparent)]
    Setting(#[from] SettingError),
}

/// What a row's findings and left-outs carry besides the match: its category and its title.
#[derive(Debug)]
struct RowTexts {
    category: CategoryDescriptor,
    title: TextKey,
}

/// The `app-leftovers` module.
#[derive(Debug)]
pub struct AppLeftoversModule {
    descriptor: ModuleDescriptor,
    places: Places,
    exclude: Exclude,
    /// One interest per distinct place of `places.toml`, in its order.
    interests: Vec<Interest>,
    /// Per row of `places`, in its order.
    rows: Vec<RowTexts>,
    reason: TextKey,
    /// The key of each reason, in the order of `Reason::ALL`.
    left_out: Vec<(Reason, TextKey)>,
    /// The leftovers of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl AppLeftoversModule {
    /// The module with its descriptor, its tables and its interests.
    ///
    /// # Errors
    ///
    /// An [`AppLeftoversError`] when one of its own constants or tables is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, AppLeftoversError> {
        let id = ModuleId::new("app-leftovers")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, category)?, danger)))
            .collect::<Result<Vec<_>, _>>()?;
        let everything = categories.iter().map(CategoryDescriptor::id).collect();
        let keep_settings = categories.iter().map(CategoryDescriptor::id).filter(|id| id.as_str() == CACHES).collect();
        let left_out = Reason::ALL
            .into_iter()
            .map(|reason| {
                Ok::<_, TextKeyError>((reason, TextKey::new(format!("app-leftovers.left-out.{}", reason.key()))?))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let settings = vec![UNINSTALLED_AFTER.descriptor(id)?];
        let choices =
            vec![ModuleChoice::new("everything", everything), ModuleChoice::new("keep-settings", keep_settings)];
        // A bad table fails here, when the module is built.
        let places = Places::shipped()?;
        let exclude = Exclude::shipped()?;
        let rows = places
            .rows()
            .iter()
            .map(|row| {
                let category = categories.iter().find(|category| category.id().as_str() == row.category).cloned();
                let category = category.ok_or_else(|| PlacesError::UnknownCategory {
                    id: row.id.clone(),
                    category: row.category.into(),
                })?;
                let title = TextKey::new(format!("app-leftovers.title.{}", row.id))?;
                Ok::<_, AppLeftoversError>(RowTexts { category, title })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Apps, categories, settings, locales)?
            .with_problems(left_out.iter().map(|(_, key)| key.clone()).collect())?
            .with_choices(choices)?;
        let interests = places.interests()?;
        Ok(Self {
            descriptor,
            places,
            exclude,
            interests,
            rows,
            reason: TextKey::new("app-leftovers.finding.reason")?,
            left_out,
            gathered: Gathered::default(),
        })
    }

    /// The key of `reason`.
    fn key_of(&self, reason: Reason) -> Option<&TextKey> {
        self.left_out.iter().find(|(known, _)| *known == reason).map(|(_, key)| key)
    }

    /// Whether every place of the rows that is not optional was seen in full by the walk.
    fn places_complete(&self, cx: &ModuleContext<'_>) -> bool {
        (0..self.interests.len()).all(|interest| {
            let optional = self.places.rows_of(interest).iter().any(|(_, row)| row.optional);
            optional || cx.place_seen(interest).is_complete()
        })
    }

    /// Reports every one of `leftovers` as left out with `reason`, in its row's category.
    fn leave_out(&self, leftovers: &[Leftover], reason: Reason, out: &mut dyn FindingSink) {
        let Some(key) = self.key_of(reason) else { return };
        for leftover in leftovers {
            let Some(texts) = self.rows.get(leftover.row) else { continue };
            out.left_out(LeftOutItem { category: texts.category.id(), reason: key.clone(), sizes: leftover.sizes });
        }
    }

    /// The finding that offers `leftover`, of the app `id`, to the Trash.
    fn finding(&self, id: &BundleId, leftover: &Leftover, days: u32) -> Option<NewFinding> {
        let (row, texts) = (self.places.row(leftover.row)?, self.rows.get(leftover.row)?);
        let name = leftover.path.file_name()?.to_string_lossy();
        let title = Text::new(texts.title.clone()).with("id", TextArg::Text(id.as_str().to_owned()));
        let reason = Text::new(self.reason.clone()).with("days", TextArg::Count(u64::from(days)));
        let identity = format!("{}/{name}", row.id);
        let subject = Subject::Path(leftover.path.clone());
        let removal = Removal::MoveToTrash(leftover.path.clone());
        let finding = NewFinding::new(&texts.category, identity, subject, leftover.sizes, title, reason, removal);
        Some(finding.for_app(id.clone()))
    }
}

impl Module for AppLeftoversModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a folder or a file named after an id for the scan's `finish`, under the first row of its place whose kind
    /// is the match's and whose suffix gives an id; a link, an `Other` entry, a name that gives no id, or a match of
    /// no row is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let kind = match matched.kind {
            EntryKind::Folder => Form::Folder,
            EntryKind::File => Form::File,
            EntryKind::Symlink | EntryKind::Other => return Ok(()),
        };
        let Some(name) = matched.path.file_name() else { return Ok(()) };
        let found = self
            .places
            .rows_of(matched.interest)
            .into_iter()
            .filter(|(_, row)| row.kind == kind)
            .find_map(|(index, row)| name::app_id(name, row.suffix.as_deref()).map(|id| (index, id)));
        let Some((row, id)) = found else { return Ok(()) };
        let leftover = Leftover {
            row,
            id,
            path: matched.path.clone(),
            kind,
            sizes: matched.sizes,
            modified: matched.modified,
            changed: matched.changed,
            inside: matched.inside,
            facts: matched.facts,
        };
        self.gathered.add(cx.cancel(), leftover);
        Ok(())
    }

    /// Decides the leftovers of the scan, id by id in order, once every place is walked: offered one by one, every
    /// one left out with the reason, or not claimed; a leftover of a row that is not offered only takes part in the
    /// decision. Spotlight is asked last, once per id that reaches it.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let by_id = self.gathered.take(cx.cancel());
        if by_id.is_empty() {
            return Ok(());
        }
        let system = cx.system();
        let days = UNINSTALLED_AFTER.days(cx.settings());
        let facts = Facts {
            installed: system.installed_apps().map_err(|_| NoSystem),
            complete: system.installed_apps_complete().is_ok(),
            running: system.running_apps().map_err(|_| NoSystem),
            restored: cx.is_restored(),
            places_complete: self.places_complete(cx),
            now: Timestamp::from_system_time(cx.clock().now()),
            days,
        };
        // Spotlight is asked for the id and every shorter id it extends: a helper of an app kept outside the apps
        // folders finds its app.
        let mut search = |id: &BundleId| match system.find_apps(&name::search_ids(id), cx.cancel()) {
            Err(_) => Search::Unread,
            Ok(found) if !found.is_empty() => Search::Found,
            Ok(_) => Search::NotFound,
        };
        for (id, leftovers) in by_id {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let verdict = decide_id(&id, &leftovers, self.places.rows(), &self.exclude, &facts, &mut search);
            // A leftover of a row that is not offered proves its id and counts for its age, and is never reported.
            let shown: Vec<Leftover> = leftovers
                .into_iter()
                .filter(|leftover| self.places.row(leftover.row).is_some_and(|row| row.offered))
                .collect();
            match verdict {
                IdVerdict::NotClaimed => {}
                IdVerdict::LeftOut(reason) => self.leave_out(&shown, reason, out),
                IdVerdict::Offer => {
                    for leftover in &shown {
                        match decide_item(leftover) {
                            ItemVerdict::LeftOut(reason) => self.leave_out(std::slice::from_ref(leftover), reason, out),
                            ItemVerdict::Offered => {
                                if let Some(finding) = self.finding(&id, leftover, days) {
                                    out.push(finding);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
