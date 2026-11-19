//! The `temp` module: the old items apps left in the user's temporary folder and in the shared one.
//!
//! Each top-level item of the two places is one match, claimed whole, and decided on its own (`decide.rs`): the answer
//! depends only on the item, the clock, the setting and the running apps, so the walk's order never matters and
//! `finish` has nothing to do.

#![forbid(unsafe_code)]

mod decide;
mod names;

use fresher_core::{
    CategoryDescriptor, CategoryId, Claim, DangerLevel, DaySetting, DescriptorError, FindingSink, IdError, Interest,
    InterestError, KnownFolder, LeftOutItem, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError,
    ModuleGroup, ModuleId, NewFinding, Pattern, Removal, SettingError, Subject, Text, TextArg, TextKey, TextKeyError,
    Timestamp,
};

use crate::decide::{Item, NoSystem, Reason, Scan, Verdict, decide};

/// The category of the old temporary items (design, section 7).
const OLD_FILES: &str = "temp.old-files";

/// `temp.older-than-days`. The check can only become stricter: never fewer than a week.
const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 30, 7);

/// The keys of the reasons the module may give for keeping an item (design, section 8).
#[derive(Debug)]
struct LeftOutKeys {
    system: TextKey,
    locked: TextKey,
    app_open: TextKey,
    special: TextKey,
    lock: TextKey,
    recent: TextKey,
    scan_again: TextKey,
    no_system: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        Ok(Self {
            system: TextKey::new("temp.left-out.system")?,
            locked: TextKey::new("temp.left-out.locked")?,
            app_open: TextKey::new("temp.left-out.app-open")?,
            special: TextKey::new("temp.left-out.special")?,
            lock: TextKey::new("temp.left-out.lock")?,
            recent: TextKey::new("temp.left-out.recent")?,
            scan_again: TextKey::new("temp.left-out.scan-again")?,
            no_system: TextKey::new("temp.left-out.no-system")?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::System => &self.system,
            Reason::Locked => &self.locked,
            Reason::AppOpen => &self.app_open,
            Reason::Special => &self.special,
            Reason::Lock => &self.lock,
            Reason::Recent => &self.recent,
            Reason::ScanAgain => &self.scan_again,
            Reason::NoSystem => &self.no_system,
        }
    }

    /// Every key, in the order the module declares them.
    fn all(&self) -> Vec<TextKey> {
        [
            &self.system,
            &self.locked,
            &self.app_open,
            &self.special,
            &self.lock,
            &self.recent,
            &self.scan_again,
            &self.no_system,
        ]
        .into_iter()
        .cloned()
        .collect()
    }
}

/// One of the module's places: the folder it walks, the prefix of its findings' identities and their reason.
#[derive(Clone, Copy, Debug)]
struct Place {
    folder: KnownFolder,
    identity: &'static str,
    reason: &'static str,
}

/// The two places, in the order of the module's interests.
const PLACES: [Place; 2] = [
    Place { folder: KnownFolder::UserTemp, identity: "user-temp", reason: "temp.finding.reason-user" },
    Place { folder: KnownFolder::SharedTemp, identity: "shared-temp", reason: "temp.finding.reason-shared" },
];

/// Why the `temp` module could not be built (a bug in its own constants).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum TempError {
    /// The module's or its category's id is not a valid id.
    #[error(transparent)]
    Id(#[from] IdError),
    /// One of its text keys is not a valid key.
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    /// Its descriptor does not hold together: a category, a setting or a problem key listed twice or outside its
    /// namespace.
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    /// One of its interests is not a valid interest.
    #[error(transparent)]
    Interest(#[from] InterestError),
    /// Its setting is not a valid setting.
    #[error(transparent)]
    Setting(#[from] SettingError),
}

/// The `temp` module.
#[derive(Debug)]
pub struct TempModule {
    descriptor: ModuleDescriptor,
    category: CategoryDescriptor,
    /// One interest per place, in the order of `PLACES`.
    interests: Vec<Interest>,
    left_out: LeftOutKeys,
    title: TextKey,
    /// The finding's reason of each place, in the order of `PLACES`.
    reasons: Vec<TextKey>,
}

impl TempModule {
    /// The module, with its one category, its setting, its left-out reasons and an interest per place.
    ///
    /// # Errors
    ///
    /// A [`TempError`] when one of its own constants is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, TempError> {
        let id = ModuleId::new("temp")?;
        let category = CategoryDescriptor::new(CategoryId::new(id, OLD_FILES)?, DangerLevel::Safe);
        let locales = Locales { en: include_str!("../locales/en.json") };
        let settings = vec![OLDER_THAN.descriptor(id)?];
        let left_out = LeftOutKeys::new()?;
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::System, vec![category.clone()], settings, locales)?
            .with_problems(left_out.all())?;
        let interests = PLACES
            .into_iter()
            .map(|place| Interest::new(Pattern::Under(place.folder, "*".into()), Claim::Whole, vec![]))
            .collect::<Result<_, _>>()?;
        let reasons = PLACES.into_iter().map(|place| TextKey::new(place.reason)).collect::<Result<_, _>>()?;
        Ok(Self { descriptor, category, interests, left_out, title: TextKey::new("temp.finding.title")?, reasons })
    }
}

impl Module for TempModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Offers the item, keeps it as left out with the reason, or leaves it unclaimed when the walk could not read it
    /// all; a match of no place, or with no name, is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let (Some(place), Some(reason)) = (PLACES.get(matched.interest), self.reasons.get(matched.interest)) else {
            return Ok(());
        };
        let Some(name) = matched.path.file_name() else { return Ok(()) };
        let item = Item {
            name,
            kind: matched.kind,
            facts: matched.facts,
            modified: matched.modified,
            changed: matched.changed,
            accessed: matched.accessed,
            inside: matched.inside,
        };
        let system = cx.system();
        let scan = Scan {
            restored: cx.is_restored(),
            now: Timestamp::from_system_time(cx.clock().now()),
            days: OLDER_THAN.days(cx.settings()),
            running: system.running_apps().map_err(|_| NoSystem),
        };
        match decide(&item, &scan) {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(kept) => {
                let reason = self.left_out.of(kept).clone();
                out.left_out(LeftOutItem { category: self.category.id(), reason, sizes: matched.sizes });
            }
            Verdict::Offered { age_days } => {
                let title =
                    Text::new(self.title.clone()).with("name", TextArg::Text(name.to_string_lossy().into_owned()));
                let reason = Text::new(reason.clone()).with("days", TextArg::Count(age_days));
                let path = matched.path.clone();
                let subject = Subject::Path(path.clone());
                let removal = Removal::MoveToTrash(path);
                let identity = format!("{}/{}", place.identity, names::identity(name));
                let finding = NewFinding::new(&self.category, identity, subject, matched.sizes, title, reason, removal);
                let app = system.installed_apps().ok().and_then(|apps| {
                    apps.iter().find(|app| names::named_after(name, &app.bundle_id)).map(|app| app.bundle_id.clone())
                });
                out.push(match app {
                    Some(app) => finding.for_app(app),
                    None => finding,
                });
            }
        }
        Ok(())
    }

    fn finish(&self, _: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
