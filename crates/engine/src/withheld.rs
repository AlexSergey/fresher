//! What the High switch leaves out: while High is withheld, no finding at High leaves the engine.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fresher_core::{CategoryDescriptor, CategoryId, DangerLevel, Finding, HighLevel, Subject};

/// The switch, the registered categories whose descriptor is High (empty when High is offered) and, for a committed
/// state, the paths of its findings raised to High in a category that is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Withheld {
    high: HighLevel,
    categories: BTreeSet<CategoryId>,
    paths: Vec<PathBuf>,
}

impl Withheld {
    /// What `high` leaves out of the registered `categories`.
    pub(crate) fn new<'a>(high: HighLevel, categories: impl Iterator<Item = &'a CategoryDescriptor>) -> Self {
        let categories =
            categories.filter(|category| !high.offers(category.danger())).map(CategoryDescriptor::id).collect();
        Self { high, categories, paths: Vec::new() }
    }

    /// Nothing left out.
    pub(crate) const fn offered() -> Self {
        Self { high: HighLevel::Offered, categories: BTreeSet::new(), paths: Vec::new() }
    }

    /// Whether `finding` is left out: its own level is not offered, or its category is withheld.
    pub(crate) fn hides(&self, finding: &Finding) -> bool {
        let new = finding.finding();
        !self.high.offers(new.danger()) || self.hides_category(new.category())
    }

    /// Whether something told at `level` (an insight's part) is left out.
    pub(crate) const fn hides_level(&self, level: DangerLevel) -> bool {
        !self.high.offers(level)
    }

    /// Whether `category` is withheld whole.
    pub(crate) fn hides_category(&self, category: CategoryId) -> bool {
        self.categories.contains(&category)
    }

    /// This set with the paths of `findings` (a state's) raised to a level that is not offered, in a category that is
    /// not withheld whole.
    pub(crate) fn with_raised(&self, findings: &[Finding]) -> Self {
        let paths = findings
            .iter()
            .filter(|finding| self.hides(finding) && !self.hides_category(finding.finding().category()))
            .filter_map(|finding| match finding.finding().subject() {
                Subject::Path(path) => Some(path.clone()),
                Subject::External { .. } => None,
            })
            .collect();
        Self { paths, ..self.clone() }
    }

    /// Whether an owner of `category` at `path` is left out of the shares: its category is withheld, or a finding
    /// raised to High is at `path`.
    pub(crate) fn hides_owner(&self, category: CategoryId, path: &Path) -> bool {
        self.hides_category(category) || self.paths.iter().any(|raised| raised == path)
    }
}
