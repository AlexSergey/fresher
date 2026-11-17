//! What a module found, and the same finding once the engine published it.

use serde::Serialize;
use ts_rs::TS;

use crate::{
    BundleId, CategoryDescriptor, CategoryId, DangerLevel, FindingId, FindingKey, ModuleId, Removability, Removal,
    Sizes, Subject, Text,
};

/// A finding as a module produces it. It has no removability: only the engine sets that, from what the walk read, so a
/// module cannot make an item removable.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct NewFinding {
    key: FindingKey,
    category: CategoryId,
    subject: Subject,
    sizes: Sizes,
    danger: DangerLevel,
    title: Text,
    reason: Text,
    removal: Removal,
    /// The app the finding belongs to, when the module knows it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, as = "Option<String>")]
    app: Option<BundleId>,
}

impl NewFinding {
    /// A finding of `category`, at the category's danger level. `identity` names the item across scans within the
    /// category's module; `title` names the item, `reason` says why it matched.
    pub fn new(
        category: &CategoryDescriptor,
        identity: impl Into<String>,
        subject: Subject,
        sizes: Sizes,
        title: Text,
        reason: Text,
        removal: Removal,
    ) -> Self {
        Self {
            key: FindingKey::new(category.id().module(), identity),
            category: category.id(),
            subject,
            sizes,
            danger: category.danger(),
            title,
            reason,
            removal,
            app: None,
        }
    }

    /// The finding belongs to `app`: the storage map counts it in the app's line.
    #[must_use]
    pub fn for_app(self, app: BundleId) -> Self {
        Self { app: Some(app), ..self }
    }

    /// Raises the danger level to `level`; a lower level changes nothing, so a finding is never less dangerous than
    /// its category.
    pub fn raise_danger(&mut self, level: DangerLevel) {
        self.danger = self.danger.max(level);
    }

    pub const fn key(&self) -> &FindingKey {
        &self.key
    }

    pub const fn category(&self) -> CategoryId {
        self.category
    }

    pub const fn subject(&self) -> &Subject {
        &self.subject
    }

    pub const fn sizes(&self) -> Sizes {
        self.sizes
    }

    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }

    pub const fn title(&self) -> &Text {
        &self.title
    }

    pub const fn reason(&self) -> &Text {
        &self.reason
    }

    pub const fn removal(&self) -> &Removal {
        &self.removal
    }

    /// The app the finding belongs to; `None` when the module did not say.
    pub const fn app(&self) -> Option<&BundleId> {
        self.app.as_ref()
    }
}

/// A finding the engine published, with the id the UI names it by in this session and whether the app can remove
/// it. It serializes flat: the id, the module and the removability next to the fields of the module's finding.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct Finding {
    id: FindingId,
    module: ModuleId,
    removability: Removability,
    #[serde(flatten)]
    inner: NewFinding,
}

impl Finding {
    pub const fn new(id: FindingId, finding: NewFinding, removability: Removability) -> Self {
        Self { id, module: finding.key.module(), removability, inner: finding }
    }

    pub const fn removability(&self) -> Removability {
        self.removability
    }

    pub const fn id(&self) -> FindingId {
        self.id
    }

    pub const fn module(&self) -> ModuleId {
        self.module
    }

    pub const fn finding(&self) -> &NewFinding {
        &self.inner
    }
}

#[cfg(test)]
pub(crate) mod sample {
    use crate::{
        CategoryDescriptor, CategoryId, DangerLevel, ModuleId, NewFinding, Removal, Sizes, Subject, Text, TextKey,
    };

    pub(crate) fn demo() -> ModuleId {
        ModuleId::new("demo").unwrap()
    }

    pub(crate) fn caches(danger: DangerLevel) -> CategoryDescriptor {
        CategoryDescriptor::new(CategoryId::new(demo(), "demo.caches").unwrap(), danger)
    }

    pub(crate) fn finding(category: &CategoryDescriptor) -> NewFinding {
        let text = |key: &str| Text::new(TextKey::new(key).unwrap());
        NewFinding::new(
            category,
            "Library/Caches/a",
            Subject::Path("/home/Library/Caches/a".into()),
            Sizes::ZERO,
            text("demo.finding.title"),
            text("demo.finding.reason"),
            Removal::MoveToTrash("/home/Library/Caches/a".into()),
        )
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::DangerLevel;
        use crate::finding::sample;

        #[test]
        fn a_module_cannot_set_removability() {
            let finding = sample::finding(&sample::caches(DangerLevel::Safe));

            let json = serde_json::to_value(&finding).unwrap();

            assert!(json.get("removability").is_none(), "{json}");
        }

        #[test]
        fn a_finding_names_no_app_unless_told() {
            let finding = sample::finding(&sample::caches(DangerLevel::Safe));

            let json = serde_json::to_value(&finding).unwrap();

            assert_eq!(finding.app(), None);
            assert!(json.get("app").is_none(), "{json}");
        }

        #[test]
        fn never_lowers_the_category_danger_level() {
            let mut finding = sample::finding(&sample::caches(DangerLevel::Medium));

            finding.raise_danger(DangerLevel::Safe);

            assert_eq!(finding.danger(), DangerLevel::Medium);
        }
    }

    mod positive {
        use crate::finding::sample;
        use crate::{BundleId, DangerLevel, Finding, FindingId, Removability, Removal, Sizes, Subject};

        #[test]
        fn starts_at_the_category_danger_level() {
            let category = sample::caches(DangerLevel::Low);
            let finding = sample::finding(&category);

            assert_eq!(finding.danger(), DangerLevel::Low);
            assert_eq!(finding.category(), category.id());
            assert_eq!(finding.key().module(), sample::demo());
            assert_eq!(finding.key().identity(), "Library/Caches/a");
            assert_eq!(finding.subject(), &Subject::Path("/home/Library/Caches/a".into()));
            assert_eq!(finding.sizes(), Sizes::ZERO);
            assert_eq!(finding.title().key().as_str(), "demo.finding.title");
            assert_eq!(finding.reason().key().as_str(), "demo.finding.reason");
            assert_eq!(finding.removal(), &Removal::MoveToTrash("/home/Library/Caches/a".into()));
        }

        #[test]
        fn a_finding_for_an_app_names_it() {
            let app = BundleId::new("com.example.App").unwrap();

            let finding = sample::finding(&sample::caches(DangerLevel::Safe)).for_app(app.clone());

            assert_eq!(finding.app(), Some(&app));
            assert_eq!(serde_json::to_value(&finding).unwrap()["app"], "com.example.App");
        }

        #[test]
        fn raises_the_danger_level_for_one_finding() {
            let mut finding = sample::finding(&sample::caches(DangerLevel::Low));

            finding.raise_danger(DangerLevel::High);

            assert_eq!(finding.danger(), DangerLevel::High);
        }

        #[test]
        fn a_published_finding_names_its_module() {
            let new = sample::finding(&sample::caches(DangerLevel::Safe));
            let finding = Finding::new(FindingId::new(4), new, Removability::Locked);

            assert_eq!(finding.id(), FindingId::new(4));
            assert_eq!(finding.removability(), Removability::Locked);
            assert_eq!(finding.module(), sample::demo());
            assert_eq!(finding.finding().danger(), DangerLevel::Safe);
            let json = serde_json::to_value(&finding).unwrap();
            assert_eq!(json["id"], 4);
            assert_eq!(json["module"], "demo");
            assert_eq!(json["danger"], "safe");
            assert_eq!(json["removability"], "locked");
        }
    }
}
