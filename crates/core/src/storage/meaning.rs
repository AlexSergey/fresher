//! The map by meaning: the apps, the explained categories, the named places and the map's other parts.

use serde::Serialize;
use ts_rs::TS;

use crate::{BundleId, CategoryId, MapPart, NodeId, Sizes, TextKey};

/// One group of the by-meaning view, largest first within its kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum MeaningGroup {
    /// An app: the findings attributed to it, by category, and its data folders no module claims.
    App { app: AppRef, sizes: Sizes, categories: Vec<CategoryShare>, data: Vec<PlaceShare> },
    /// An explained category not tied to an app.
    Category(CategoryShare),
    /// A named place or one of the largest of the user's folders.
    Place(PlaceShare),
    /// One of the map's parts that is not a folder: the system's, the remainder, Free.
    Part(MapPart),
}

/// The app a group is about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AppRef {
    #[ts(type = "string")]
    pub id: BundleId,
    pub name: String,
    /// Whether the app is still installed: the data of a removed app stays behind.
    pub installed: bool,
}

/// The bytes one category owns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CategoryShare {
    pub category: CategoryId,
    pub sizes: Sizes,
}

/// The bytes of one folder no module claims.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PlaceShare {
    pub node: NodeId,
    pub name: PlaceName,
    pub sizes: Sizes,
}

/// What a place is called: its folder's own name, or a place the app names.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum PlaceName {
    Folder(String),
    /// The user's Trash: never cleaned by the app; emptying it gives the space back.
    Trash,
}

impl PlaceName {
    /// The named place's name: `core.storage.place.<place>.name`; `None` for a folder, which shows its own.
    pub fn name_key(&self) -> Option<TextKey> {
        self.key("name")
    }

    /// What the named place is and what the user can do: `core.storage.place.<place>.description`.
    pub fn description_key(&self) -> Option<TextKey> {
        self.key("description")
    }

    fn key(&self, text: &str) -> Option<TextKey> {
        match self {
            Self::Folder(_) => None,
            Self::Trash => Some(TextKey::from_valid(format!("core.storage.place.trash.{text}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::PlaceName;

        #[test]
        fn a_folder_has_no_text_of_its_own() {
            let folder = PlaceName::Folder("Movies".into());

            assert_eq!((folder.name_key(), folder.description_key()), (None, None));
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{
            AppRef, BundleId, ByteSize, CategoryId, CategoryShare, MapPart, MeaningGroup, ModuleId, NodeId, PartKind,
            PlaceName, PlaceShare, Sizes, StateId,
        };

        fn sizes() -> Sizes {
            Sizes::complete(ByteSize::new(4), ByteSize::new(8))
        }

        #[test]
        fn a_meaning_group_serializes_by_kind() {
            let category = CategoryShare {
                category: CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap(),
                sizes: sizes(),
            };
            let trash = PlaceShare { node: NodeId::new(StateId::new(1), 4), name: PlaceName::Trash, sizes: sizes() };
            let folder = PlaceShare {
                node: NodeId::new(StateId::new(1), 5),
                name: PlaceName::Folder("Movies".into()),
                sizes: sizes(),
            };
            let app = MeaningGroup::App {
                app: AppRef { id: BundleId::new("com.example.App").unwrap(), name: "Example".into(), installed: false },
                sizes: sizes(),
                categories: vec![category.clone()],
                data: vec![folder],
            };
            let part = MeaningGroup::Part(MapPart { kind: PartKind::Free, sizes: sizes() });
            let sizes = json!({ "logical": 4, "allocated": 8, "reclaimable": 8, "completeness": "complete" });
            let category_json = json!({ "category": "demo.caches", "sizes": sizes });

            let groups = [app, MeaningGroup::Category(category), MeaningGroup::Place(trash), part]
                .map(|group| serde_json::to_value(group).unwrap());

            assert_eq!(
                groups,
                [
                    json!({ "kind": "app", "value": {
                        "app": { "id": "com.example.App", "name": "Example", "installed": false },
                        "sizes": sizes,
                        "categories": [category_json],
                        "data": [{
                            "node": { "state": 1, "index": 5 },
                            "name": { "kind": "folder", "value": "Movies" },
                            "sizes": sizes,
                        }],
                    }}),
                    json!({ "kind": "category", "value": category_json }),
                    json!({ "kind": "place", "value": {
                        "node": { "state": 1, "index": 4 },
                        "name": { "kind": "trash" },
                        "sizes": sizes,
                    }}),
                    json!({ "kind": "part", "value": { "kind": { "kind": "free" }, "sizes": sizes } }),
                ]
            );
        }

        #[test]
        fn the_trash_has_its_texts() {
            assert_eq!(PlaceName::Trash.name_key().unwrap().as_str(), "core.storage.place.trash.name");
            assert_eq!(PlaceName::Trash.description_key().unwrap().as_str(), "core.storage.place.trash.description");
        }
    }
}
