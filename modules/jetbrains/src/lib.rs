//! The `jetbrains` module: the settings, caches and logs that older versions of the vendor's IDEs left behind, and the
//! data of an IDE that is no longer installed.
//!
//! Each IDE keeps one folder per version (`WebStorm2025.3`) under `JetBrains/` in the app support, caches and logs
//! folders. The module recognizes a folder only by its exact name (`name.rs`) of a known product (`products.toml`), and
//! offers it only when an installed, closed IDE of that product uses a newer version, or when no app of the product is
//! installed, nothing of the vendor runs and none of the product's folders changed for the setting's days; everything
//! else it recognized it keeps and reports as left out, with the reason.
//!
//! Each folder of an installed product is decided on its own match (`decide.rs`): the answer depends only on the
//! folder's product and version and on the scan's installed and running apps. The folders of a product that is not
//! installed wait for `finish` (`uninstalled.rs`): their answer depends on every folder of the product, so the walk's
//! order never matters; they are kept with the scan's cancel token, so a scan cancelled before its `finish` leaves
//! nothing to the next one.

#![forbid(unsafe_code)]

mod decide;
mod name;
mod places;
mod products;
mod uninstalled;

use fresher_core::{
    CategoryDescriptor, CategoryIcon, CategoryId, DangerLevel, DaySetting, DescriptorError, EntryKind, FindingSink,
    IdError, Interest, InterestError, LeftOutItem, LeftOutKind, Locales, Match, Module, ModuleContext,
    ModuleDescriptor, ModuleError, ModuleGroup, ModuleId, NewFinding, Removal, RunningSet, SettingError, Subject, Text,
    TextArg, TextKey, TextKeyError, Timestamp,
};

use crate::decide::{Reason, Verdict, decide};
use crate::places::{PLACES, Place};
use crate::products::Products;
pub use crate::products::ProductsError;
use crate::uninstalled::{Age, Gathered, Waiting, age};

/// The category of the data of an IDE that is no longer installed (design, section 5).
const UNINSTALLED: &str = "jetbrains.uninstalled";

/// `jetbrains.uninstalled-after-days`. Never fewer than a week: a removed IDE's data is not offered the day after.
const UNINSTALLED_AFTER: DaySetting = DaySetting::new("uninstalled-after-days", 7, 3650, 30);

/// The keys of the reasons the module may give for keeping a folder it recognized (design, section 5).
#[derive(Debug)]
struct LeftOutKeys {
    current: TextKey,
    newer: TextKey,
    running: TextKey,
    unknown_version: TextKey,
    no_system: TextKey,
    used_recently: TextKey,
    unknown_ide: TextKey,
    installed_elsewhere: TextKey,
    vendor_app_open: TextKey,
    apps_unread: TextKey,
    spotlight_unread: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        Ok(Self {
            current: TextKey::new("jetbrains.left-out.current")?,
            newer: TextKey::new("jetbrains.left-out.newer")?,
            running: TextKey::new("jetbrains.left-out.running")?,
            unknown_version: TextKey::new("jetbrains.left-out.unknown-version")?,
            no_system: TextKey::new("jetbrains.left-out.no-system")?,
            used_recently: TextKey::new("jetbrains.left-out.used-recently")?,
            unknown_ide: TextKey::new("jetbrains.left-out.unknown-ide")?,
            installed_elsewhere: TextKey::new("jetbrains.left-out.installed-elsewhere")?,
            vendor_app_open: TextKey::new("jetbrains.left-out.vendor-app-open")?,
            apps_unread: TextKey::new("jetbrains.left-out.apps-unread")?,
            spotlight_unread: TextKey::new("jetbrains.left-out.spotlight-unread")?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::Current => &self.current,
            Reason::Newer => &self.newer,
            Reason::Running => &self.running,
            Reason::UnknownVersion => &self.unknown_version,
            Reason::NoSystem => &self.no_system,
            Reason::UsedRecently => &self.used_recently,
            Reason::UnknownIde => &self.unknown_ide,
            Reason::InstalledElsewhere => &self.installed_elsewhere,
            Reason::VendorAppOpen => &self.vendor_app_open,
            Reason::AppsUnread => &self.apps_unread,
            Reason::SpotlightUnread => &self.spotlight_unread,
        }
    }

    /// The keys of the reasons that keep a folder for good: it is in use, or an installed app may use it.
    fn kept(&self) -> Vec<TextKey> {
        Reason::ALL
            .into_iter()
            .filter(|reason| reason.kind() == LeftOutKind::Kept)
            .map(|reason| self.of(reason).clone())
            .collect()
    }

    /// Every key, in the order the module declares them.
    fn all(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.of(reason).clone()).collect()
    }
}

/// The titles of the findings of an IDE that is no longer installed, one per place.
#[derive(Debug)]
struct UninstalledTitles {
    settings: TextKey,
    caches: TextKey,
    logs: TextKey,
}

impl UninstalledTitles {
    fn new() -> Result<Self, TextKeyError> {
        Ok(Self {
            settings: TextKey::new(Place::Settings.uninstalled_title())?,
            caches: TextKey::new(Place::Caches.uninstalled_title())?,
            logs: TextKey::new(Place::Logs.uninstalled_title())?,
        })
    }

    const fn of(&self, place: Place) -> &TextKey {
        match place {
            Place::Settings => &self.settings,
            Place::Caches => &self.caches,
            Place::Logs => &self.logs,
        }
    }
}

/// Why the `jetbrains` module could not be built (a bug in its own constants or products table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum JetBrainsError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Products(#[from] ProductsError),
    #[error(transparent)]
    Setting(#[from] SettingError),
}

/// The `jetbrains` module.
#[derive(Debug)]
pub struct JetBrainsModule {
    descriptor: ModuleDescriptor,
    /// One interest per place, in the order of `PLACES`.
    interests: Vec<Interest>,
    /// The category of each place, in the order of `PLACES`.
    categories: Vec<CategoryDescriptor>,
    /// The category of the data of an IDE that is no longer installed.
    uninstalled: CategoryDescriptor,
    products: Products,
    title: TextKey,
    reason: TextKey,
    uninstalled_title: UninstalledTitles,
    uninstalled_reason: TextKey,
    left_out: LeftOutKeys,
    /// The folders of the products that are not installed, waiting for the scan's `finish`.
    gathered: Gathered,
}

impl JetBrainsModule {
    pub fn new() -> Result<Self, JetBrainsError> {
        let id = ModuleId::new("jetbrains")?;
        let category = |place: Place| {
            Ok::<_, IdError>(
                CategoryDescriptor::new(CategoryId::new(id, place.category())?, place.danger()).with_icon(icon(place)),
            )
        };
        let categories = PLACES.into_iter().map(category).collect::<Result<Vec<_>, _>>()?;
        let uninstalled = CategoryDescriptor::new(CategoryId::new(id, UNINSTALLED)?, DangerLevel::Medium)
            .with_icon(CategoryIcon::Editor);
        let mut listed = categories.clone();
        listed.push(uninstalled.clone());
        listed.sort_by_key(|category| category.id().as_str());
        let locales = Locales { en: include_str!("../locales/en.json") };
        let left_out = LeftOutKeys::new()?;
        let settings = vec![UNINSTALLED_AFTER.descriptor(id)?];
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, listed, settings, locales)?
            .with_problems(left_out.all())?
            .with_kept(left_out.kept())?;
        let interests = PLACES.into_iter().map(Place::interest).collect::<Result<_, _>>()?;
        Ok(Self {
            descriptor,
            interests,
            categories,
            uninstalled,
            products: Products::shipped()?,
            title: TextKey::new("jetbrains.finding.title")?,
            reason: TextKey::new("jetbrains.finding.reason")?,
            uninstalled_title: UninstalledTitles::new()?,
            uninstalled_reason: TextKey::new("jetbrains.finding.uninstalled-reason")?,
            left_out,
            gathered: Gathered::default(),
        })
    }

    /// Offers or keeps every folder of the product `product` that is not installed: kept after a walk that could not
    /// read every folder (an unseen one may have changed), when one changed within the setting's days, or when
    /// Spotlight knows an app of the product elsewhere or cannot tell; offered otherwise.
    fn decide_uninstalled(
        &self,
        product: &str,
        folders: Vec<Waiting>,
        cx: &ModuleContext<'_>,
        out: &mut dyn FindingSink,
    ) {
        // The folder name was read from this table when the folder was gathered.
        let Some(product) = self.products.by_folder(product) else { return };
        let now = Timestamp::from_system_time(cx.clock().now());
        let days = UNINSTALLED_AFTER.days(cx.settings());
        let age = if cx.is_walk_complete() {
            age(folders.iter().map(|folder| folder.modified), now, days)
        } else {
            Age::Recent
        };
        let kept = match age {
            Age::Recent => Some(Reason::UsedRecently),
            Age::Old => match cx.system().find_apps(product.bundle_ids(), cx.cancel()) {
                // Spotlight could not be asked, or its index is off: an app of the product may be anywhere.
                Err(_) => Some(Reason::SpotlightUnread),
                Ok(found) if !found.is_empty() => Some(Reason::InstalledElsewhere),
                Ok(_) => None,
            },
        };
        if let Some(reason) = kept {
            let reason = self.left_out.of(reason);
            for folder in folders {
                let title = Text::new(self.uninstalled_title.of(folder.place).clone())
                    .with("product", TextArg::Text(product.name().into()))
                    .with("version", TextArg::Text(folder.version.to_string()));
                let item = LeftOutItem {
                    category: self.uninstalled.id(),
                    reason: reason.clone(),
                    sizes: folder.sizes,
                    path: folder.path,
                    title,
                };
                out.left_out(item);
            }
            return;
        }
        // Every product has at least one bundle id (checked when the table is read); the release's is first.
        let Some(app) = product.bundle_ids().first() else { return };
        for folder in folders {
            let args = |text: Text| {
                text.with("product", TextArg::Text(product.name().into()))
                    .with("version", TextArg::Text(folder.version.to_string()))
            };
            let title = args(Text::new(self.uninstalled_title.of(folder.place).clone()));
            let reason = args(Text::new(self.uninstalled_reason.clone())).with("days", TextArg::Count(u64::from(days)));
            let name = folder.path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            let identity = format!("{}/{name}", folder.place.identity());
            let subject = Subject::Path(folder.path.clone());
            let removal = Removal::MoveToTrash(folder.path);
            let finding = NewFinding::new(&self.uninstalled, identity, subject, folder.sizes, title, reason, removal);
            out.push(finding.for_app_named(app.clone(), product.name()));
        }
    }
}

impl Module for JetBrainsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Decides the folder at once, or keeps it for `finish` when its product is not installed; a link, a file, a name
    /// that is not a version folder of a known product, or a match of no place is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if matched.kind != EntryKind::Folder {
            return Ok(());
        }
        let (Some(&place), Some(category)) = (PLACES.get(matched.interest), self.categories.get(matched.interest))
        else {
            return Ok(());
        };
        let Some(name) = matched.path.file_name() else { return Ok(()) };
        let Some((product, version)) = name::parse(name, &self.products) else { return Ok(()) };
        let system = cx.system();
        let running = system.running_apps().map(RunningSet::apps).map_err(|_| Reason::NoSystem);
        // A partial list is no list: an IDE whose bundle could not be read must not count as not installed.
        let installed = system
            .installed_apps_complete()
            .map_err(|_| if system.installed_apps().is_ok() { Reason::AppsUnread } else { Reason::NoSystem });
        let args = |text: Text| {
            text.with("product", TextArg::Text(product.name().into()))
                .with("version", TextArg::Text(version.to_string()))
        };
        match decide(&self.products, product, version, installed, running) {
            Verdict::Offer { current, app } => {
                let title = args(Text::new(self.title.clone()));
                let reason = args(Text::new(self.reason.clone())).with("current", TextArg::Text(current.to_string()));
                let identity = format!("{}/{}", place.identity(), name.to_string_lossy());
                let path = matched.path.clone();
                let subject = Subject::Path(path.clone());
                let removal = Removal::MoveToTrash(path);
                let finding = NewFinding::new(category, identity, subject, matched.sizes, title, reason, removal);
                out.push(finding.for_app(app));
            }
            Verdict::LeftOut(reason) => {
                let (category, title) = if reason.of_removed_ide() {
                    (&self.uninstalled, self.uninstalled_title.of(place))
                } else {
                    (category, &self.title)
                };
                out.left_out(LeftOutItem {
                    category: category.id(),
                    reason: self.left_out.of(reason).clone(),
                    sizes: matched.sizes,
                    path: matched.path.clone(),
                    title: args(Text::new(title.clone())),
                });
            }
            Verdict::Uninstalled => {
                let waiting = Waiting {
                    product: product.folder().to_owned(),
                    version,
                    place,
                    path: matched.path.clone(),
                    sizes: matched.sizes,
                    modified: matched.modified,
                };
                self.gathered.add(cx.cancel(), waiting);
            }
        }
        Ok(())
    }

    /// Decides the folders of the products that are not installed, each product's together, once every folder of the
    /// scan is seen.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        for (product, folders) in self.gathered.take(cx.cancel()) {
            self.decide_uninstalled(&product, folders, cx, out);
        }
        Ok(())
    }
}

/// The icon of the category of `place`.
const fn icon(place: Place) -> CategoryIcon {
    match place {
        Place::Logs => CategoryIcon::Log,
        Place::Settings | Place::Caches => CategoryIcon::Editor,
    }
}

#[cfg(test)]
mod tests;
