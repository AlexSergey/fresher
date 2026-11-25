//! What the module tells the engine about a folder it decided (design, section 5): a finding that moves the folder to
//! the Trash or empties it, counted in Xcode's line of the map, or a left-out line in its category with its sizes. The
//! text keys are built once, with the module.

use std::collections::BTreeSet;

use fresher_core::{
    BundleId, CategoryDescriptor, FindingSink, LeftOutItem, LeftOutKind, NewFinding, Removal, Subject, Text, TextArg,
    TextKey,
};

use crate::XcodeError;
use crate::decide::{Named, Reason, Verdict, XCODE_APPS};
use crate::gathered::Candidate;
use crate::name::Version;
use crate::places::{Row, RowRemoval};

/// The category of the previews, emptied as one fixed folder.
const PREVIEWS: &str = "xcode.previews";
/// The category of the simulators' caches, emptied as one fixed folder.
const SIMULATOR_CACHES: &str = "xcode.simulator-caches";

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    special: TextKey,
    not_yours: TextKey,
    no_system: TextKey,
    app_open: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, XcodeError> {
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

/// The title and the reason of one kind of finding.
#[derive(Debug)]
struct Texts {
    title: TextKey,
    reason: TextKey,
}

impl Texts {
    fn new(title: &str, reason: &str) -> Result<Self, XcodeError> {
        Ok(Self { title: TextKey::new(title)?, reason: TextKey::new(reason)? })
    }
}

/// The module's categories, the app every finding is counted in, and the keys of every text a finding or a left-out
/// carries.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Vec<CategoryDescriptor>,
    xcode: BundleId,
    device_support: Texts,
    device_support_recent: TextKey,
    previews: Texts,
    simulator_caches: Texts,
    archive: Texts,
    left_out: LeftOutKeys,
}

impl Report {
    /// The texts of the module whose categories are `categories`.
    pub(crate) fn new(categories: Vec<CategoryDescriptor>) -> Result<Self, XcodeError> {
        Ok(Self {
            categories,
            xcode: BundleId::new(XCODE_APPS[0])?,
            device_support: Texts::new("xcode.title.device-support", "xcode.reason.device-support")?,
            device_support_recent: TextKey::new("xcode.reason.device-support-recent")?,
            previews: Texts::new("xcode.title.previews", "xcode.reason.previews")?,
            simulator_caches: Texts::new("xcode.title.simulator-caches", "xcode.reason.simulator-caches")?,
            archive: Texts::new("xcode.title.archive", "xcode.reason.archive")?,
            left_out: LeftOutKeys::new()?,
        })
    }

    /// The keys of the left-out reasons, in the order the module declares them.
    pub(crate) fn problems(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.left_out.of(reason).clone()).collect()
    }

    /// The keys of the reasons that keep a folder for good.
    pub(crate) fn kept(&self) -> Vec<TextKey> {
        let kept = Reason::ALL.into_iter().filter(|reason| reason.kind() == LeftOutKind::Kept);
        kept.map(|reason| self.left_out.of(reason).clone()).collect()
    }

    /// Reports `verdict` on `candidate`, a folder of `row`'s place: a finding, a left-out in the row's category, or
    /// nothing. A device support folder whose platform and version are in `recent` has the recent reason.
    pub(crate) fn tell(
        &self,
        row: &Row,
        candidate: &Candidate,
        verdict: &Verdict,
        recent: &BTreeSet<(String, Version)>,
        out: &mut dyn FindingSink,
    ) {
        // Every row names a category of the module: `Places::parse` refuses a table where one does not.
        let Some(category) = self.categories.iter().find(|category| category.id().as_str() == row.category) else {
            return;
        };
        let (named, left_out) = match verdict {
            Verdict::NotClaimed => return,
            Verdict::LeftOut(reason, named) => (named, Some(*reason)),
            Verdict::Offered(named) => (named, None),
        };
        // A candidate is a child of a place, so its path has a name; a name that is not UTF-8 was not read as a shape.
        let Some(folder) = candidate.path.file_name().and_then(|name| name.to_str()) else { return };
        let Some((identity, title, reason)) = self.texts(row, folder, named, candidate, recent) else { return };
        if let Some(left_out) = left_out {
            out.left_out(LeftOutItem {
                category: category.id(),
                reason: self.left_out.of(left_out).clone(),
                sizes: candidate.sizes,
                path: candidate.path.clone(),
                title,
            });
            return;
        }
        let path = candidate.path.clone();
        let removal = match row.removal {
            RowRemoval::Trash => Removal::MoveToTrash(path.clone()),
            RowRemoval::EmptyFolder => Removal::EmptyFolder(path.clone()),
        };
        let finding = NewFinding::new(category, identity, Subject::Path(path), candidate.sizes, title, reason, removal);
        out.push(finding.for_app(self.xcode.clone()));
    }

    /// The identity, the title and the reason of `folder`, a candidate of `row` whose name read as `named`; `None` for
    /// a fixed row of a category with no fixed place.
    fn texts(
        &self,
        row: &Row,
        folder: &str,
        named: &Named,
        candidate: &Candidate,
        recent: &BTreeSet<(String, Version)>,
    ) -> Option<(String, Text, Text)> {
        let size = TextArg::Size(candidate.sizes.logical);
        let texts = match named {
            Named::DeviceSupport(name) => {
                // A device support row always has a platform: `Places::parse` requires it for that shape.
                let platform = row.platform.clone().unwrap_or_default();
                let key = if recent.contains(&(platform.clone(), name.version.clone())) {
                    &self.device_support_recent
                } else {
                    &self.device_support.reason
                };
                let (platform, version) = (TextArg::Text(platform), TextArg::Text(name.version.to_string()));
                let title = Text::new(self.device_support.title.clone())
                    .with("platform", platform.clone())
                    .with("version", version.clone())
                    .with("build", TextArg::Text(name.build.clone()));
                let reason =
                    Text::new(key.clone()).with("platform", platform).with("version", version).with("size", size);
                (format!("{}/{folder}", row.id), title, reason)
            }
            Named::Fixed => {
                let texts = match row.category {
                    PREVIEWS => &self.previews,
                    SIMULATOR_CACHES => &self.simulator_caches,
                    // A fixed row is of one of these two categories: the module has no other fixed place.
                    _ => return None,
                };
                let reason = Text::new(texts.reason.clone()).with("size", size);
                (row.id.clone(), Text::new(texts.title.clone()), reason)
            }
            Named::Archive(name) => {
                let title = Text::new(self.archive.title.clone()).with("name", TextArg::Text(name.name.clone()));
                let reason = Text::new(self.archive.reason.clone())
                    .with("date", TextArg::Text(name.date.clone()))
                    .with("size", size);
                (format!("{}/{}/{folder}", row.id, name.date), title, reason)
            }
        };
        Some(texts)
    }
}
