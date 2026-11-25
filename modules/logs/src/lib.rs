//! The `logs` module: the old logs and crash reports in the user's `~/Library/Logs`.
//!
//! It offers files under `~/Library/Logs` that nothing has changed for more than the setting's days, crash reports
//! apart from the logs of apps. It never claims a folder or a link, nothing under `JetBrains` or under a top-level
//! folder shaped as a bundle id (other modules' places), no state or lock file, and nothing outside `Logs`. A file
//! goes to the Trash; its folders stay.

#![forbid(unsafe_code)]

mod decide;
mod names;

use fresher_core::{
    CategoryDescriptor, CategoryIcon, CategoryId, Claim, DangerLevel, DaySetting, DescriptorError, FindingSink,
    IdError, Interest, InterestError, KnownFolder, LeftOutItem, LeftOutKind, Locales, Match, Module, ModuleContext,
    ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, NewFinding, Pattern, Removal, SettingError, Subject, Text,
    TextArg, TextKey, TextKeyError, Timestamp,
};

use crate::decide::{Item, Reason, Scan, Verdict, decide};
use crate::names::{Located, Place, folder_text, identity, is_state_or_lock, locate};

/// The category of the logs of apps.
const APP_LOGS: &str = "logs.app-logs";

/// The category of the crash reports.
const CRASH_REPORTS: &str = "logs.crash-reports";

/// `logs.older-than-days`. The check can only become stricter: never fewer than a week.
const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 90, 7);

/// How many names deep below `Logs` a file is looked for.
const MAX_DEPTH: usize = 6;

/// Why the `logs` module could not be built (a bug in its own constants).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum LogsError {
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

/// The `logs` module.
#[derive(Debug)]
pub struct LogsModule {
    descriptor: ModuleDescriptor,
    app_logs: CategoryDescriptor,
    crash_reports: CategoryDescriptor,
    /// One interest per depth, from one name below `Logs` to `MAX_DEPTH`.
    interests: Vec<Interest>,
    /// The key of each left-out reason, in the order of `Reason::ALL`.
    left_out: Vec<(Reason, TextKey)>,
    title: TextKey,
    reason_top: TextKey,
    reason_folder: TextKey,
    reason_report: TextKey,
}

/// The finding's identity: the place's prefix and the names below the place's folder.
fn finding_identity(located: &Located<'_>) -> String {
    let prefix = match located.place {
        Place::AppLogs => "app-logs",
        Place::CrashReports => "crash-reports",
    };
    format!("{prefix}/{}", identity(&located.below))
}

impl LogsModule {
    /// The module, with its two categories, its setting, its left-out reasons and an interest per depth.
    ///
    /// # Errors
    ///
    /// A [`LogsError`] when one of its own constants is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, LogsError> {
        let id = ModuleId::new("logs")?;
        let app_logs =
            CategoryDescriptor::new(CategoryId::new(id, APP_LOGS)?, DangerLevel::Safe).with_icon(CategoryIcon::Log);
        let crash_reports = CategoryDescriptor::new(CategoryId::new(id, CRASH_REPORTS)?, DangerLevel::Safe)
            .with_icon(CategoryIcon::Log);
        let locales = Locales { en: include_str!("../locales/en.json") };
        let settings = vec![OLDER_THAN.descriptor(id)?];
        let left_out = Reason::ALL
            .into_iter()
            .map(|reason| TextKey::new(reason.key()).map(|key| (reason, key)))
            .collect::<Result<Vec<_>, _>>()?;
        let problems = left_out.iter().map(|(_, key)| key.clone()).collect();
        let kept = left_out
            .iter()
            .filter(|(reason, _)| reason.kind() == LeftOutKind::Kept)
            .map(|(_, key)| key.clone())
            .collect();
        let descriptor = ModuleDescriptor::new(
            id,
            ModuleGroup::System,
            vec![app_logs.clone(), crash_reports.clone()],
            settings,
            locales,
        )?
        .with_problems(problems)?
        .with_kept(kept)?;
        let interests = (1..=MAX_DEPTH)
            .map(|depth| {
                Interest::new(Pattern::Under(KnownFolder::Logs, vec!["*"; depth].join("/")), Claim::Item, vec![])
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            descriptor,
            app_logs,
            crash_reports,
            interests,
            left_out,
            title: TextKey::new("logs.title")?,
            reason_top: TextKey::new("logs.reason.top")?,
            reason_folder: TextKey::new("logs.reason.folder")?,
            reason_report: TextKey::new("logs.reason.report")?,
        })
    }

    /// The finding's reason: a crash report's, a log's in a folder below `Logs`, or a log's at the top of `Logs`.
    fn reason(&self, located: &Located<'_>, age_days: u64) -> Text {
        let days = TextArg::Count(age_days);
        match (located.place, folder_text(&located.below)) {
            (Place::CrashReports, _) => Text::new(self.reason_report.clone()).with("days", days),
            (Place::AppLogs, Some(folder)) => {
                Text::new(self.reason_folder.clone()).with("folder", TextArg::Text(folder)).with("days", days)
            }
            (Place::AppLogs, None) => Text::new(self.reason_top.clone()).with("days", days),
        }
    }
}

impl Module for LogsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Offers the file, keeps it as left out with the reason, or leaves it unclaimed: a folder, a link, a state or lock
    /// file, another module's place or anything outside `Logs` is not this module's.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let system = cx.system();
        let Some(logs) = system.locate(KnownFolder::Logs) else { return Ok(()) };
        let Some(located) = locate(&logs, &matched.path) else { return Ok(()) };
        let Some(name) = located.below.last().copied() else { return Ok(()) };
        let item = Item {
            kind: matched.kind,
            facts: matched.facts,
            modified: matched.modified,
            changed: matched.changed,
            state_or_lock: is_state_or_lock(name),
        };
        let scan = Scan { now: Timestamp::from_system_time(cx.clock().now()), days: OLDER_THAN.days(cx.settings()) };
        let category = match located.place {
            Place::AppLogs => &self.app_logs,
            Place::CrashReports => &self.crash_reports,
        };
        let title = || Text::new(self.title.clone()).with("name", TextArg::Text(name.to_string_lossy().into_owned()));
        match decide(&item, &scan) {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(kept) => {
                if let Some((_, reason)) = self.left_out.iter().find(|(reason, _)| *reason == kept) {
                    out.left_out(LeftOutItem {
                        category: category.id(),
                        reason: reason.clone(),
                        sizes: matched.sizes,
                        path: matched.path.clone(),
                        title: title(),
                    });
                }
            }
            Verdict::Offered { age_days } => {
                let title = title();
                let path = matched.path.clone();
                let finding = NewFinding::new(
                    category,
                    finding_identity(&located),
                    Subject::Path(path.clone()),
                    matched.sizes,
                    title,
                    self.reason(&located, age_days),
                    Removal::MoveToTrash(path),
                );
                out.push(finding);
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
