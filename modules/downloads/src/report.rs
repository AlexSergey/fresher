//! What the module tells the engine about an entry it decided: a finding to the Trash, or a left-out line in its
//! category with its sizes. The text keys are built once, with the module.

use fresher_core::{
    CategoryDescriptor, FindingSink, IdError, LeftOutItem, LeftOutKind, NewFinding, Removal, Subject, Text, TextArg,
    TextKey, TextKeyError,
};

use crate::categories::Category;
use crate::decide::{OfferReason, Reason, Verdict};
use crate::gathered::Entry;

/// The module's categories, one per `Category`.
#[derive(Debug)]
pub(crate) struct Categories {
    unfinished: CategoryDescriptor,
    safari_resumable: CategoryDescriptor,
    installers: CategoryDescriptor,
    macos_installers: CategoryDescriptor,
    large_untouched: CategoryDescriptor,
}

impl Categories {
    /// Each category as `make` builds it.
    pub(crate) fn new(mut make: impl FnMut(Category) -> Result<CategoryDescriptor, IdError>) -> Result<Self, IdError> {
        Ok(Self {
            unfinished: make(Category::Unfinished)?,
            safari_resumable: make(Category::SafariResumable)?,
            installers: make(Category::Installers)?,
            macos_installers: make(Category::MacosInstallers)?,
            large_untouched: make(Category::LargeUntouched)?,
        })
    }

    fn get(&self, category: Category) -> &CategoryDescriptor {
        match category {
            Category::Unfinished => &self.unfinished,
            Category::SafariResumable => &self.safari_resumable,
            Category::Installers => &self.installers,
            Category::MacosInstallers => &self.macos_installers,
            Category::LargeUntouched => &self.large_untouched,
        }
    }
}

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    no_system: TextKey,
    apps_unread: TextKey,
    browser_open: TextKey,
    recent: TextKey,
    not_older: TextKey,
    newer_macos: TextKey,
    scan_again: TextKey,
    special: TextKey,
    not_yours: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |reason: Reason| TextKey::new(reason.key());
        Ok(Self {
            no_system: key(Reason::NoSystem)?,
            apps_unread: key(Reason::AppsUnread)?,
            browser_open: key(Reason::BrowserOpen)?,
            recent: key(Reason::Recent)?,
            not_older: key(Reason::NotOlder)?,
            newer_macos: key(Reason::NewerMacos)?,
            scan_again: key(Reason::ScanAgain)?,
            special: key(Reason::Special)?,
            not_yours: key(Reason::NotYours)?,
        })
    }

    fn get(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::NoSystem => &self.no_system,
            Reason::AppsUnread => &self.apps_unread,
            Reason::BrowserOpen => &self.browser_open,
            Reason::Recent => &self.recent,
            Reason::NotOlder => &self.not_older,
            Reason::NewerMacos => &self.newer_macos,
            Reason::ScanAgain => &self.scan_again,
            Reason::Special => &self.special,
            Reason::NotYours => &self.not_yours,
        }
    }
}

/// The module's categories and text keys.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Categories,
    title: TextKey,
    left_out: LeftOutKeys,
    unfinished: TextKey,
    placeholder: TextKey,
    safari_resumable: TextKey,
    installer: TextKey,
    installer_no_version: TextKey,
    installer_receipt: TextKey,
    macos_installer: TextKey,
    large_untouched: TextKey,
}

impl Report {
    /// The keys, and the module's `categories`.
    pub(crate) fn new(categories: Categories) -> Result<Self, TextKeyError> {
        let reason = |name: &str| TextKey::new(format!("downloads.reason.{name}"));
        Ok(Self {
            categories,
            title: TextKey::new("downloads.title")?,
            left_out: LeftOutKeys::new()?,
            unfinished: reason("unfinished")?,
            placeholder: reason("placeholder")?,
            safari_resumable: reason("safari-resumable")?,
            installer: reason("installer")?,
            installer_no_version: reason("installer-no-version")?,
            installer_receipt: reason("installer-receipt")?,
            macos_installer: reason("macos-installer")?,
            large_untouched: reason("large-untouched")?,
        })
    }

    /// The categories in the order of `Category::ALL`, for the module's descriptor.
    pub(crate) fn categories(&self) -> Vec<CategoryDescriptor> {
        Category::ALL.into_iter().map(|category| self.categories.get(category).clone()).collect()
    }

    /// The keys of the left-out reasons in the order of `Reason::ALL`, as the module declares them.
    pub(crate) fn problems(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.left_out.get(reason).clone()).collect()
    }

    /// The keys of the reasons that keep a file for good.
    pub(crate) fn kept(&self) -> Vec<TextKey> {
        let kept = Reason::ALL.into_iter().filter(|reason| reason.kind() == LeftOutKind::Kept);
        kept.map(|reason| self.left_out.get(reason).clone()).collect()
    }

    /// Reports `entry` of `category` as `verdict` says: nothing when it is not claimed.
    pub(crate) fn tell(&self, entry: &Entry, category: Category, verdict: Verdict, out: &mut dyn FindingSink) {
        let descriptor = self.categories.get(category);
        match verdict {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(reason) => out.left_out(LeftOutItem {
                category: descriptor.id(),
                reason: self.left_out.get(reason).clone(),
                sizes: entry.sizes,
                path: entry.path.clone(),
                title: self.title(entry),
            }),
            Verdict::Offered { reason } => {
                let (text, app) = self.reason(reason);
                let finding = self.finding(descriptor, category, entry, text);
                out.push(match app {
                    Some(app) => finding.for_app(app),
                    None => finding,
                });
            }
        }
    }

    /// Offers `placeholder`, the empty file beside an offered `.part`, in the unfinished category.
    pub(crate) fn placeholder(&self, placeholder: &Entry, out: &mut dyn FindingSink) {
        let descriptor = self.categories.get(Category::Unfinished);
        let reason = Text::new(self.placeholder.clone());
        out.push(self.finding(descriptor, Category::Unfinished, placeholder, reason));
    }

    /// The title of `entry`, offered or left out: its name.
    fn title(&self, entry: &Entry) -> Text {
        Text::new(self.title.clone()).with("name", TextArg::Text(entry.name.clone()))
    }

    /// The finding that moves `entry` to the Trash, identified as `<category name>/<file name>`.
    fn finding(&self, descriptor: &CategoryDescriptor, category: Category, entry: &Entry, reason: Text) -> NewFinding {
        let title = self.title(entry);
        let identity = format!("{}/{}", category.name(), entry.name);
        let (subject, removal) = (Subject::Path(entry.path.clone()), Removal::MoveToTrash(entry.path.clone()));
        NewFinding::new(descriptor, identity, subject, entry.sizes, title, reason, removal)
    }

    /// The reason's text, and the app an installer proven by name installed.
    fn reason(&self, reason: OfferReason) -> (Text, Option<fresher_core::BundleId>) {
        let days = |key: &TextKey, days: u32| Text::new(key.clone()).with("days", TextArg::Count(u64::from(days)));
        match reason {
            OfferReason::Unfinished { days: count } => (days(&self.unfinished, count), None),
            OfferReason::SafariResumable { days: count } => (days(&self.safari_resumable, count), None),
            OfferReason::Installer { app, name, version: Some(version) } => {
                let text = Text::new(self.installer.clone())
                    .with("app", TextArg::Text(name))
                    .with("version", TextArg::Text(version));
                (text, Some(app))
            }
            OfferReason::Installer { app, name, version: None } => {
                (Text::new(self.installer_no_version.clone()).with("app", TextArg::Text(name)), Some(app))
            }
            OfferReason::InstallerReceipt { installed } => {
                (Text::new(self.installer_receipt.clone()).with("date", TextArg::Date(installed)), None)
            }
            OfferReason::MacosInstaller { running } => {
                let version = format!("{}.{}", running.major, running.minor);
                (Text::new(self.macos_installer.clone()).with("running", TextArg::Text(version)), None)
            }
            OfferReason::LargeUntouched { size, since } => {
                let text = Text::new(self.large_untouched.clone())
                    .with("size", TextArg::Size(size))
                    .with("date", TextArg::Date(since));
                (text, None)
            }
        }
    }
}
