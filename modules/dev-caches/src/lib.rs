//! The `dev-caches` module: the download caches of package managers, Go's build cache and Xcode's derived data.
//!
//! Every place it offers is a tool's own cache folder at its default place (`tools.toml`), recognized only by the sign
//! the tool writes there; a cache moved by an environment variable or the tool's config is not found, since the module
//! reads neither. It never runs a tool's command: what the user confirms goes to the Trash, and each tool makes its
//! cache again when it needs it.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
#[cfg(test)]
mod home;
mod name;
mod tools;

use fresher_core::{
    CategoryDescriptor, CategoryIcon, CategoryId, DescriptorError, FindingSink, IdError, Interest, InterestError,
    LeftOutItem, LeftOutKind, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup,
    ModuleId, NewFinding, Removal, Subject, Text, TextArg, TextKey, TextKeyError,
};

use crate::categories::CATEGORIES;
use crate::decide::{NoSystem, Reason, Verdict, decide};
use crate::gathered::{Candidate, Gathered, Holder, Sign};
pub use crate::tools::ToolsError;
use crate::tools::{Cache, ComesBack, InterestMap, Role, Tools};

/// The keys of the reasons the module may give for keeping a cache it recognized (design, section 4).
#[derive(Debug)]
struct LeftOutKeys {
    not_yours: TextKey,
    special: TextKey,
    app_open: TextKey,
    no_system: TextKey,
}

impl LeftOutKeys {
    fn new() -> Result<Self, TextKeyError> {
        Ok(Self {
            not_yours: TextKey::new("dev-caches.left-out.not-yours")?,
            special: TextKey::new("dev-caches.left-out.special")?,
            app_open: TextKey::new("dev-caches.left-out.app-open")?,
            no_system: TextKey::new("dev-caches.left-out.no-system")?,
        })
    }

    const fn of(&self, reason: Reason) -> &TextKey {
        match reason {
            Reason::NotYours => &self.not_yours,
            Reason::Special => &self.special,
            Reason::AppOpen => &self.app_open,
            Reason::NoSystem => &self.no_system,
        }
    }

    /// Every key, in the order the module declares them.
    fn all(&self) -> Vec<TextKey> {
        Reason::ALL.into_iter().map(|reason| self.of(reason).clone()).collect()
    }

    /// The keys of the reasons that keep a cache for good.
    fn kept(&self) -> Vec<TextKey> {
        let kept = Reason::ALL.into_iter().filter(|reason| reason.kind() == LeftOutKind::Kept);
        kept.map(|reason| self.of(reason).clone()).collect()
    }
}

/// Why the `dev-caches` module could not be built (a bug in its own constants or tools table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DevCachesError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Tools(#[from] ToolsError),
}

/// What a row's findings and left-outs carry besides the match: its category and its title.
#[derive(Debug)]
struct RowTexts {
    category: CategoryDescriptor,
    title: TextKey,
}

/// The `dev-caches` module.
#[derive(Debug)]
pub struct DevCachesModule {
    descriptor: ModuleDescriptor,
    tools: Tools,
    /// The interests of `tools.toml` (one per row, then one per sign place) and the role of each.
    map: InterestMap,
    /// Per row of `tools`, in its order.
    rows: Vec<RowTexts>,
    reason_download: TextKey,
    reason_build: TextKey,
    left_out: LeftOutKeys,
    /// The matches of the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl DevCachesModule {
    /// The module, with its nine categories, its left-out reasons, no setting beyond its switch, and the interests
    /// of its tools table.
    ///
    /// # Errors
    ///
    /// A [`DevCachesError`] when one of its own constants or its tools table is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, DevCachesError> {
        let id = ModuleId::new("dev-caches")?;
        let categories = CATEGORIES
            .into_iter()
            .map(|(category, danger)| {
                Ok::<_, IdError>(
                    CategoryDescriptor::new(CategoryId::new(id, category)?, danger).with_icon(icon(category)),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let tools = Tools::shipped()?;
        let rows = tools
            .caches()
            .iter()
            .map(|row| {
                let category = categories.iter().find(|category| category.id().as_str() == row.category).cloned();
                let category = category
                    .ok_or_else(|| ToolsError::UnknownCategory { id: row.id.clone(), category: row.category.into() })?;
                let title = TextKey::new(format!("dev-caches.title.{}", row.id))?;
                Ok::<_, DevCachesError>(RowTexts { category, title })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let left_out = LeftOutKeys::new()?;
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, categories, Vec::new(), locales)?
            .with_problems(left_out.all())?
            .with_kept(left_out.kept())?;
        let map = tools.interests()?;
        Ok(Self {
            descriptor,
            tools,
            map,
            rows,
            reason_download: TextKey::new("dev-caches.finding.reason-download")?,
            reason_build: TextKey::new("dev-caches.finding.reason-build")?,
            left_out,
            gathered: Gathered::default(),
        })
    }

    /// The finding that offers `candidate`, the folder of `row`.
    fn finding(&self, candidate: &Candidate, row: &Cache, texts: &RowTexts) -> NewFinding {
        let name = candidate.path.file_name().unwrap_or_default();
        let title = title(candidate, row, texts);
        let reason = match row.comes_back {
            ComesBack::Download => &self.reason_download,
            ComesBack::Build => &self.reason_build,
        };
        let reason = Text::new(reason.clone()).with("tool", TextArg::Text(row.tool.clone()));
        let identity = format!("{}/{}", row.id, name.to_string_lossy());
        let subject = Subject::Path(candidate.path.clone());
        let removal = Removal::MoveToTrash(candidate.path.clone());
        let finding = NewFinding::new(&texts.category, identity, subject, candidate.sizes, title, reason, removal);
        match &row.app {
            Some(app) => finding.for_app(app.clone()),
            None => finding,
        }
    }
}

impl Module for DevCachesModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.map.interests
    }

    /// Keeps a row's folder or a sign place for the scan's `finish`; a match of no interest is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let sign = || Sign { holds: matched.holds.clone(), marker: matched.marker };
        match self.map.roles.get(matched.interest) {
            Some(&Role::Cache(row)) => {
                let candidate = Candidate {
                    row,
                    path: matched.path.clone(),
                    kind: matched.kind,
                    sizes: matched.sizes,
                    facts: matched.facts,
                    inside: matched.inside,
                    sign: sign(),
                };
                self.gathered.add_candidate(cx.cancel(), candidate);
            }
            Some(&Role::SignHolder(index)) => {
                self.gathered.add_holder(cx.cancel(), Holder { index, kind: matched.kind, sign: sign() });
            }
            None => {}
        }
        Ok(())
    }

    /// Decides every folder of the scan, sorted by path, once its sign holders are seen: offered, left out with the
    /// reason, or not claimed.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let scan = self.gathered.take(cx.cancel());
        let caches = self.tools.caches();
        let names_app = |candidate: &Candidate| caches.get(candidate.row).is_some_and(|row| row.app.is_some());
        // Read once, and only when a row that names an app has a folder. Otherwise it is not read, and `decide`, which
        // reads it only for such a row, never looks at the value given in its place.
        let running = if scan.candidates.iter().any(names_app) {
            cx.system().running_apps().map_err(|_| NoSystem)
        } else {
            Err(NoSystem)
        };
        for candidate in &scan.candidates {
            let (Some(row), Some(texts)) = (caches.get(candidate.row), self.rows.get(candidate.row)) else { continue };
            let sign = match self.map.holder_of(candidate.row) {
                None => Some(&candidate.sign),
                Some(index) => scan.holder(index).and_then(Holder::sign),
            };
            match decide(candidate, row, sign, running) {
                Verdict::NotClaimed => {}
                Verdict::LeftOut(reason) => out.left_out(LeftOutItem {
                    category: texts.category.id(),
                    reason: self.left_out.of(reason).clone(),
                    sizes: candidate.sizes,
                    path: candidate.path.clone(),
                    title: title(candidate, row, texts),
                }),
                Verdict::Offered => out.push(self.finding(candidate, row, texts)),
            }
        }
        Ok(())
    }
}

/// The title of `candidate`, a folder of `row`, offered or left out: the project's name for a row that names one.
fn title(candidate: &Candidate, row: &Cache, texts: &RowTexts) -> Text {
    let name = candidate.path.file_name().unwrap_or_default();
    let title = Text::new(texts.title.clone());
    match row.name.and_then(|_| name::project(name)) {
        Some(project) => title.with("project", TextArg::Text(project.to_owned())),
        None => title,
    }
}

/// The icon of each category; one it does not list is generic, which the module contract refuses.
fn icon(category: &str) -> CategoryIcon {
    match category {
        "dev-caches.npm"
        | "dev-caches.yarn"
        | "dev-caches.homebrew"
        | "dev-caches.cargo"
        | "dev-caches.cocoapods"
        | "dev-caches.pip" => CategoryIcon::Download,
        "dev-caches.gradle" | "dev-caches.go-build" | "dev-caches.xcode-derived-data" => CategoryIcon::Build,
        _ => CategoryIcon::Generic,
    }
}

#[cfg(test)]
mod tests;
