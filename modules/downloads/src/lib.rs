//! The `downloads` module: what piles up in the user's Downloads folder.
//!
//! It offers unfinished downloads, Safari's resumable downloads, the installers of apps that are installed, the
//! macOS installers of a version this Mac has passed, and big files nobody has opened for a long time. It reads only
//! the first level of `~/Downloads` (plus the bundles it sums whole), and only once the user allowed the folder. It
//! never reads Downloads without that access, never looks below the first level, never mounts a disk image, expands
//! an archive or runs an installer, and never claims a `.dmg` or `.pkg` that no installed app or receipt proves.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod installers;
mod macos;
mod name;
mod report;
mod unfinished;

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};

use fresher_core::{
    Capability, CategoryDescriptor, CategoryIcon, CategoryId, Claim, DaySetting, DescriptorError, FindingSink,
    GuardedFolder, IdError, InstalledApp, Interest, InterestError, KnownFolder, Locales, Match, Module, ModuleContext,
    ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, Need, Pattern, Receipt, RunningApp, RunningSet, SettingError,
    SystemFacts, TextKeyError, Timestamp,
};

use crate::categories::Category;
use crate::decide::{AppsState, Package, Proof, Scan, Verdict};
use crate::gathered::{Entry, FILES, Gathered};
use crate::installers::Words;
pub use crate::installers::WordsError;
pub use crate::macos::MacosError;
use crate::macos::MacosInstallers;
use crate::report::{Categories, Report};
pub use crate::unfinished::SuffixesError;
use crate::unfinished::{Form, Suffixes};

/// `downloads.older-than-days`. Never fewer than a week: a download is not offered the day after it stopped.
const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 3650, 30);

/// The name of a macOS installer, which is always a folder ending with this.
const APP_SUFFIX: &str = ".app";

/// Why the `downloads` module could not be built (a bug in its own constants or tables).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DownloadsError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Setting(#[from] SettingError),
    #[error(transparent)]
    Suffixes(#[from] SuffixesError),
    #[error(transparent)]
    Words(#[from] WordsError),
    #[error(transparent)]
    Macos(#[from] MacosError),
}

/// The `downloads` module.
#[derive(Debug)]
pub struct DownloadsModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
    tables: Tables,
    report: Report,
    /// The entries of the scan, waiting for its `finish`.
    gathered: Gathered,
}

/// The module's three data tables.
#[derive(Debug)]
struct Tables {
    suffixes: Suffixes,
    words: Words,
    macos: MacosInstallers,
}

impl DownloadsModule {
    /// The module with its descriptor, its tables and its interests.
    ///
    /// # Errors
    ///
    /// A [`DownloadsError`] when one of its own constants or tables is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, DownloadsError> {
        let id = ModuleId::new("downloads")?;
        let needs = || vec![Need::required(Capability::Folder(GuardedFolder::Downloads))];
        let categories = Categories::new(|category| {
            let id = CategoryId::new(id, category.id())?;
            Ok(CategoryDescriptor::new(id, category.danger()).with_icon(CategoryIcon::Download).with_needs(needs()))
        })?;
        let report = Report::new(categories)?;
        let (categories, problems, kept) = (report.categories(), report.problems(), report.kept());
        let locales = Locales { en: include_str!("../locales/en.json") };
        let settings = vec![OLDER_THAN.descriptor(id)?];
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Files, categories, settings, locales)?
            .with_problems(problems)?
            .with_kept(kept)?;
        // A bad table fails here, when the module is built.
        let tables =
            Tables { suffixes: Suffixes::shipped()?, words: Words::shipped()?, macos: MacosInstallers::shipped()? };
        let interests = interests(&tables.suffixes)?;
        Ok(Self { descriptor, interests, tables, report, gathered: Gathered::default() })
    }

    /// The category of `entry` and its verdict: an unfinished file by its row; then a bundle (resumable, a macOS
    /// installer, or not claimed); then a `.dmg` or `.pkg`; then a big file. The system is asked only for what the
    /// entry needs.
    fn decide(&self, entry: &Entry, scan: &Scan, system: &Asked<'_>) -> (Category, Verdict) {
        let suffixes = &self.tables.suffixes;
        if entry.kind == Form::Bundle {
            if let Some(row) = suffixes.row_of(&entry.name, Form::Bundle) {
                return (Category::SafariResumable, decide::resumable(entry, row, system.running(), scan));
            }
            let Some(version) = self.tables.macos.version_of(&entry.name) else {
                return (Category::MacosInstallers, Verdict::NotClaimed);
            };
            let running_os = system.facts.system_version().map_err(|_| ());
            return (Category::MacosInstallers, decide::macos_installer(entry, version, running_os, scan));
        }
        if let Some(row) = suffixes.row_of(&entry.name, Form::File) {
            return (Category::Unfinished, decide::unfinished(entry, row, system.running(), scan));
        }
        if let Some((stem, package)) = decide::package_of(&entry.name) {
            let receipt = if package == Package::Flat {
                system.receipts().and_then(|receipts| decide::receipt_proof(entry, receipts))
            } else {
                None
            };
            // A receipt proves the package without the installed apps, which are then not asked.
            let verdict = if let Some(installed) = receipt {
                decide::installer(entry, Proof::Receipt { installed }, AppsState::Unlisted, scan)
            } else {
                let (apps, listed) = system.apps();
                let named = name::names_app(stem, listed, &self.tables.words);
                decide::installer(entry, Proof::Name(named), apps, scan)
            };
            return (Category::Installers, verdict);
        }
        (Category::LargeUntouched, decide::large(entry, scan))
    }

    /// The placeholders offered with their `.part`, by the name of their `.part`.
    fn placeholders<'a>(&self, entries: &'a [Entry], scan: &Scan, system: &Asked<'_>) -> BTreeMap<&'a str, &'a Entry> {
        let mut placeholders = BTreeMap::new();
        for part in entries.iter().filter(|entry| entry.kind == Form::File) {
            let Some(row) = self.tables.suffixes.row_of(&part.name, Form::File) else { continue };
            let Some(name) = decide::placeholder_name(part, row) else { continue };
            let offered = matches!(decide::unfinished(part, row, system.running(), scan), Verdict::Offered { .. });
            let sibling = file_named(entries, name);
            if let Some(sibling) = sibling.filter(|_| decide::placeholder(offered, sibling, scan)) {
                placeholders.insert(part.name.as_str(), sibling);
            }
        }
        placeholders
    }
}

/// The file named `name` among `entries`, which are sorted by name.
fn file_named<'a>(entries: &'a [Entry], name: &str) -> Option<&'a Entry> {
    let at = entries.partition_point(|entry| entry.name.as_str() < name);
    entries.get(at..)?.iter().take_while(|entry| entry.name == name).find(|entry| entry.kind == Form::File)
}

/// What `finish` asks the system, each on first need.
struct Asked<'a> {
    facts: &'a SystemFacts,
    receipts: OnceCell<Option<&'a [Receipt]>>,
}

impl<'a> Asked<'a> {
    fn new(facts: &'a SystemFacts) -> Self {
        Self { facts, receipts: OnceCell::new() }
    }

    fn running(&self) -> Result<&'a [RunningApp], ()> {
        self.facts.running_apps().map(RunningSet::apps).map_err(|_| ())
    }

    /// The installed apps' state, and the apps the name rule looks among (the partial list too).
    fn apps(&self) -> (AppsState<'a>, &'a [InstalledApp]) {
        match self.facts.installed_apps() {
            Err(_) => (AppsState::Unlisted, &[]),
            Ok(listed) => match self.facts.installed_apps_complete() {
                Ok(apps) => (AppsState::Complete(apps), apps),
                Err(_) => (AppsState::Partial, listed),
            },
        }
    }

    /// The receipts; receipts that cannot be read prove nothing, and that is logged once (the reason, never a path).
    fn receipts(&self) -> Option<&'a [Receipt]> {
        *self.receipts.get_or_init(|| match self.facts.receipts() {
            Ok(receipts) => Some(receipts),
            Err(error) => {
                tracing::warn!(reason = %error.reason, "the installer receipts could not be read; packages are proven by name only");
                None
            }
        })
    }
}

/// Interest 0 takes every first-level file; interest 1 sums the bundles whole: Safari's (the table's bundle suffix)
/// and the macOS installers.
fn interests(suffixes: &Suffixes) -> Result<Vec<Interest>, InterestError> {
    let anything = || Pattern::Under(KnownFolder::Downloads, "*".into());
    let bundles = suffixes.bundle_suffix().into_iter().map(str::to_owned).chain([APP_SUFFIX.to_owned()]).collect();
    Ok(vec![
        Interest::new(anything(), Claim::Item, Vec::new())?,
        Interest::new(anything(), Claim::Whole, Vec::new())?
            .only_suffixes(bundles)?
            .holding(vec!["Info.plist".into(), "Contents".into()])?,
    ])
}

impl Module for DownloadsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a first-level file of interest 0 or a bundle of interest 1 for the scan's `finish`; anything else is
    /// not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let Some(downloads) = cx.system().locate(KnownFolder::Downloads) else { return Ok(()) };
        if let Some(entry) = Entry::first_level(matched, &downloads) {
            self.gathered.add(cx.cancel(), entry);
        }
        Ok(())
    }

    /// Decides the entries of the scan by name, each once: offered, left out with the reason, or not claimed. An
    /// empty placeholder offered with its `.part` is told right after it.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let entries = self.gathered.take(cx.cancel());
        if entries.is_empty() {
            return Ok(());
        }
        let scan = Scan {
            now: Timestamp::from_system_time(cx.clock().now()),
            days: OLDER_THAN.days(cx.settings()),
            restored: cx.is_restored(),
            place_complete: cx.place_seen(FILES).is_complete(),
        };
        let system = Asked::new(cx.system());
        let placeholders = self.placeholders(&entries, &scan, &system);
        let taken: BTreeSet<&str> = placeholders.values().map(|placeholder| placeholder.name.as_str()).collect();
        for entry in &entries {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            if entry.kind == Form::File && taken.contains(entry.name.as_str()) {
                continue;
            }
            let (category, verdict) = self.decide(entry, &scan, &system);
            self.report.tell(entry, category, verdict, out);
            if let Some(placeholder) = placeholders.get(entry.name.as_str()) {
                self.report.placeholder(placeholder, out);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
