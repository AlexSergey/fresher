//! A category of findings: its id and danger level; its texts are found by convention.

use serde::Serialize;
use ts_rs::TS;

use crate::{CategoryId, DangerLevel, TextKey};

/// A category every finding belongs to; the floor of its findings' danger level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct CategoryDescriptor {
    id: CategoryId,
    danger: DangerLevel,
}

/// The five texts every category has (rule 3 of docs/rules.md), keyed `<category>.<text>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CategoryText {
    /// `name`: what the category is called.
    Name,
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
    pub const ALL: [Self; 5] = [Self::Name, Self::What, Self::MadeBy, Self::IfRemoved, Self::ComesBack];

    /// The last segment of the text's key.
    pub const fn segment(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::What => "what",
            Self::MadeBy => "made-by",
            Self::IfRemoved => "if-removed",
            Self::ComesBack => "comes-back",
        }
    }
}

impl CategoryDescriptor {
    pub const fn new(id: CategoryId, danger: DangerLevel) -> Self {
        Self { id, danger }
    }

    pub const fn id(&self) -> CategoryId {
        self.id
    }

    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }

    /// The key of one of the category's texts: `<category>.<segment>`.
    pub fn text_key(&self, text: CategoryText) -> TextKey {
        TextKey::from_valid(format!("{}.{}", self.id, text.segment()))
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use crate::{CategoryDescriptor, CategoryId, CategoryText, DangerLevel, ModuleId};

        #[test]
        fn derives_the_five_category_text_keys() {
            let module = ModuleId::new("demo").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(module, "demo.caches").unwrap(), DangerLevel::Low);

            let keys = CategoryText::ALL.map(|text| category.text_key(text).to_string());

            assert_eq!(
                keys,
                [
                    "demo.caches.name",
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
