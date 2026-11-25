//! A category of findings: its id, danger level, needs and icon; its texts are found by convention.

use serde::Serialize;
use ts_rs::TS;

use crate::{CategoryId, DangerLevel, Need, TextKey};

/// A category every finding belongs to; the floor of its findings' danger level.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct CategoryDescriptor {
    id: CategoryId,
    danger: DangerLevel,
    /// The capabilities it needs to be found, or to be found whole.
    needs: Vec<Need>,
    /// The shape the UI draws beside its name.
    icon: CategoryIcon,
}

/// The icon a category names, one of a fixed set; the UI draws its shape. A module's category never names `Generic`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum CategoryIcon {
    Cache,
    Build,
    Download,
    Project,
    Toolchain,
    Log,
    Temporary,
    App,
    Model,
    Editor,
    Browser,
    Device,
    Archive,
    /// No particular shape: only the core's own categories name it.
    Generic,
}

impl CategoryIcon {
    pub const ALL: [Self; 14] = [
        Self::Cache,
        Self::Build,
        Self::Download,
        Self::Project,
        Self::Toolchain,
        Self::Log,
        Self::Temporary,
        Self::App,
        Self::Model,
        Self::Editor,
        Self::Browser,
        Self::Device,
        Self::Archive,
        Self::Generic,
    ];
}

/// The six texts every category has (rule 3 of docs/rules.md), keyed `<category>.<text>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CategoryText {
    /// `name`: what the category is called.
    Name,
    /// `line`: what its tile says under its name, in a few words.
    Line,
    /// `what`: what the files are.
    What,
    /// `made-by`: which app produces them.
    MadeBy,
    /// `if-removed`: what happens when they are removed.
    IfRemoved,
    /// `comes-back`: whether and how they come back.
    ComesBack,
}

impl CategoryText {
    pub const ALL: [Self; 6] = [Self::Name, Self::Line, Self::What, Self::MadeBy, Self::IfRemoved, Self::ComesBack];

    /// The last segment of the text's key.
    pub const fn segment(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Line => "line",
            Self::What => "what",
            Self::MadeBy => "made-by",
            Self::IfRemoved => "if-removed",
            Self::ComesBack => "comes-back",
        }
    }
}

impl CategoryDescriptor {
    /// A category that needs nothing, with the generic icon.
    pub const fn new(id: CategoryId, danger: DangerLevel) -> Self {
        Self { id, danger, needs: Vec::new(), icon: CategoryIcon::Generic }
    }

    /// This category, with `icon`.
    #[must_use]
    pub fn with_icon(mut self, icon: CategoryIcon) -> Self {
        self.icon = icon;
        self
    }

    /// This category, needing `needs`.
    #[must_use]
    pub fn with_needs(mut self, needs: Vec<Need>) -> Self {
        self.needs = needs;
        self
    }

    pub const fn id(&self) -> CategoryId {
        self.id
    }

    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }

    pub fn needs(&self) -> &[Need] {
        &self.needs
    }

    pub const fn icon(&self) -> CategoryIcon {
        self.icon
    }

    /// The key of one of the category's texts: `<category>.<segment>`.
    pub fn text_key(&self, text: CategoryText) -> TextKey {
        TextKey::from_valid(format!("{}.{}", self.id, text.segment()))
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{CategoryDescriptor, CategoryIcon, CategoryId, DangerLevel, ModuleId};

        #[test]
        fn a_new_category_has_the_generic_icon() {
            let module = ModuleId::new("demo").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(module, "demo.caches").unwrap(), DangerLevel::Low);

            assert_eq!(category.icon(), CategoryIcon::Generic);
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{
            Capability, CategoryDescriptor, CategoryIcon, CategoryId, CategoryText, DangerLevel, ModuleId, Need,
        };

        #[test]
        fn a_category_serializes_its_needs() {
            let module = ModuleId::new("demo").unwrap();
            let id = CategoryId::new(module, "demo.caches").unwrap();
            let plain = CategoryDescriptor::new(id, DangerLevel::Low);
            let needy = plain.clone().with_needs(vec![Need::sees_more(Capability::AllFiles)]);

            assert_eq!(plain.needs(), []);
            assert_eq!(needy.needs(), [Need::sees_more(Capability::AllFiles)]);
            assert_eq!(
                serde_json::to_value(needy).unwrap(),
                json!({
                    "id": "demo.caches",
                    "danger": "low",
                    "needs": [{ "capability": { "kind": "all-files" }, "kind": "sees-more" }],
                    "icon": "generic",
                })
            );
        }

        #[test]
        fn with_icon_sets_it_and_it_serializes() {
            let module = ModuleId::new("demo").unwrap();
            let id = CategoryId::new(module, "demo.caches").unwrap();

            let names: Vec<_> = CategoryIcon::ALL
                .into_iter()
                .map(|icon| {
                    let category = CategoryDescriptor::new(id, DangerLevel::Low).with_icon(icon);
                    assert_eq!(category.icon(), icon);
                    serde_json::to_value(category).unwrap()["icon"].as_str().unwrap().to_owned()
                })
                .collect();

            assert_eq!(
                names,
                [
                    "cache",
                    "build",
                    "download",
                    "project",
                    "toolchain",
                    "log",
                    "temporary",
                    "app",
                    "model",
                    "editor",
                    "browser",
                    "device",
                    "archive",
                    "generic",
                ]
            );
        }

        #[test]
        fn derives_the_six_category_text_keys() {
            let module = ModuleId::new("demo").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(module, "demo.caches").unwrap(), DangerLevel::Low);

            let keys = CategoryText::ALL.map(|text| category.text_key(text).to_string());

            assert_eq!(
                keys,
                [
                    "demo.caches.name",
                    "demo.caches.line",
                    "demo.caches.what",
                    "demo.caches.made-by",
                    "demo.caches.if-removed",
                    "demo.caches.comes-back",
                ]
            );
            assert_eq!(category.danger(), DangerLevel::Low);
            assert_eq!(category.id().as_str(), "demo.caches");
        }
    }
}
