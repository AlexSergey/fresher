//! What the module tells the engine about a folder it decided: a finding to the Trash, or a left-out line in its
//! category with its sizes. The categories and text keys are built once, with the module.

use std::path::Path;

use fresher_core::{
    CategoryDescriptor, FindingSink, IdError, LeftOutItem, NewFinding, Removal, Sizes, Subject, Text, TextArg, TextKey,
    TextKeyError,
};

use crate::CACHE_UNUSED_DAYS;
use crate::categories::Category;
use crate::decide::{Reason, Staleness, Verdict};

/// The module's categories, one per `Category`.
#[derive(Debug)]
pub(crate) struct Categories {
    dependencies: CategoryDescriptor,
    build_output: CategoryDescriptor,
    python_environments: CategoryDescriptor,
    rust_build_cache: CategoryDescriptor,
}

impl Categories {
    /// Each category as `make` builds it.
    pub(crate) fn new(mut make: impl FnMut(Category) -> Result<CategoryDescriptor, IdError>) -> Result<Self, IdError> {
        Ok(Self {
            dependencies: make(Category::Dependencies)?,
            build_output: make(Category::BuildOutput)?,
            python_environments: make(Category::PythonEnvironments)?,
            rust_build_cache: make(Category::RustBuildCache)?,
        })
    }

    fn get(&self, category: Category) -> &CategoryDescriptor {
        match category {
            Category::Dependencies => &self.dependencies,
            Category::BuildOutput => &self.build_output,
            Category::PythonEnvironments => &self.python_environments,
            Category::RustBuildCache => &self.rust_build_cache,
        }
    }
}

/// The keys of the left-out reasons, one per `Reason`.
#[derive(Debug)]
struct LeftOutKeys {
    scan_again: TextKey,
    unread: TextKey,
    recent: TextKey,
    cache_recent: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |reason: Reason| TextKey::new(reason.key());
        Ok(Self {
            scan_again: key(Reason::ScanAgain)?,
            unread: key(Reason::Unread)?,
            recent: key(Reason::Recent)?,
            cache_recent: key(Reason::CacheRecent)?,
        })
    }

    fn get(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::ScanAgain => &self.scan_again,
            Reason::Unread => &self.unread,
            Reason::Recent => &self.recent,
            Reason::CacheRecent => &self.cache_recent,
        }
    }
}

/// The keys of the offer reasons, one per `Category`.
#[derive(Debug)]
struct ReasonKeys {
    dependencies: TextKey,
    build_output: TextKey,
    python_environments: TextKey,
    rust_build_cache: TextKey,
}

impl ReasonKeys {
    fn new() -> Result<Self, TextKeyError> {
        let key = |category: Category| TextKey::new(format!("stale-projects.reason.{}", category.name()));
        Ok(Self {
            dependencies: key(Category::Dependencies)?,
            build_output: key(Category::BuildOutput)?,
            python_environments: key(Category::PythonEnvironments)?,
            rust_build_cache: key(Category::RustBuildCache)?,
        })
    }

    fn get(&self, category: Category) -> &TextKey {
        match category {
            Category::Dependencies => &self.dependencies,
            Category::BuildOutput => &self.build_output,
            Category::PythonEnvironments => &self.python_environments,
            Category::RustBuildCache => &self.rust_build_cache,
        }
    }
}

/// Where a decided folder is, for its identity and its title.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Where<'a> {
    pub(crate) home: &'a Path,
    /// The root of the group's outermost project, which names every folder of the group.
    pub(crate) outermost: &'a Path,
}

/// A decided folder: an artifact of a group, or a cache.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Item<'a> {
    pub(crate) category: Category,
    pub(crate) path: &'a Path,
    pub(crate) sizes: Sizes,
    /// The command its reason names.
    pub(crate) command: &'a str,
    /// For an artifact, its group's staleness, whose time an offer names; `None` for a cache.
    pub(crate) group: Option<Staleness>,
}

/// The module's categories and text keys.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Categories,
    title: TextKey,
    reasons: ReasonKeys,
    left_out: LeftOutKeys,
}

impl Report {
    /// The keys, and the module's `categories`.
    pub(crate) fn new(categories: Categories) -> Result<Self, TextKeyError> {
        Ok(Self {
            categories,
            title: TextKey::new("stale-projects.title")?,
            reasons: ReasonKeys::new()?,
            left_out: LeftOutKeys::new()?,
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

    /// Reports `item` as `verdict` says: nothing when it is not claimed, nor when an artifact is offered for a group
    /// that is not stale (which the decision never gives).
    pub(crate) fn tell(&self, item: &Item<'_>, place: Where<'_>, verdict: Verdict, out: &mut dyn FindingSink) {
        let descriptor = self.categories.get(item.category);
        match verdict {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(reason) => out.left_out(LeftOutItem {
                category: descriptor.id(),
                reason: self.left_out.get(reason).clone(),
                sizes: item.sizes,
                path: item.path.to_path_buf(),
                title: self.title(item, place),
            }),
            Verdict::Offered => {
                let Some(reason) = self.reason(item) else { return };
                out.push(self.finding(descriptor, item, place, reason));
            }
        }
    }

    /// The reason of an offer: the group's last activity and the command for an artifact, the days and the command
    /// for a cache; `None` for an artifact of a group that is not stale.
    fn reason(&self, item: &Item<'_>) -> Option<Text> {
        let text = Text::new(self.reasons.get(item.category).clone());
        let text = match item.group {
            None => text.with("days", TextArg::Count(u64::from(CACHE_UNUSED_DAYS))),
            Some(Staleness::Stale { since }) => text.with("date", TextArg::Date(since)),
            // Unreachable: an artifact is offered only for a stale group; nothing is offered without a reason.
            Some(Staleness::Restored | Staleness::Active | Staleness::Unread) => return None,
        };
        Some(text.with("command", TextArg::Text(item.command.to_owned())))
    }

    /// The finding that moves `item` to the Trash, identified as `<category name>/<path relative to the home>`.
    fn finding(&self, descriptor: &CategoryDescriptor, item: &Item<'_>, place: Where<'_>, reason: Text) -> NewFinding {
        let relative = item.path.strip_prefix(place.home).unwrap_or(item.path);
        let identity = format!("{}/{}", item.category.name(), relative.to_string_lossy());
        let title = self.title(item, place);
        let (subject, removal) =
            (Subject::Path(item.path.to_path_buf()), Removal::MoveToTrash(item.path.to_path_buf()));
        NewFinding::new(descriptor, identity, subject, item.sizes, title, reason, removal)
    }

    /// The title of `item`, offered or left out: its outermost project's name and its folder in that project.
    fn title(&self, item: &Item<'_>, place: Where<'_>) -> Text {
        let project = place.outermost.file_name().map_or_else(|| place.outermost.as_os_str(), |name| name);
        let folder = item.path.strip_prefix(place.outermost).unwrap_or(item.path);
        Text::new(self.title.clone())
            .with("project", TextArg::Text(project.to_string_lossy().into_owned()))
            .with("folder", TextArg::Text(folder.to_string_lossy().into_owned()))
    }
}
