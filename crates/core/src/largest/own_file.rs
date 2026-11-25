//! The core's own category: the files and folders the user picks on the Largest screen.

use crate::{CategoryDescriptor, CategoryIcon, CategoryId, DangerLevel, NodeKind, Text, TextArg, TextKey};

/// The id of the core's own category.
pub const OWN_FILE_ID: &str = "core.own-file";

/// The core's own category, `core.own-file`: items the user chose by hand, never found by a scan. Always High: Fresher
/// cannot tell what such an item holds. It has no module, no switch and no card, and the generic icon: it can be
/// anything.
pub fn own_file() -> CategoryDescriptor {
    CategoryDescriptor::new(CategoryId::own_file(), DangerLevel::High).with_icon(CategoryIcon::Generic)
}

/// The title of a picked item: its name.
pub fn own_file_title(name: &str) -> Text {
    Text::new(TextKey::from_valid("core.own-file.title".to_owned())).with("name", TextArg::Text(name.to_owned()))
}

/// Why a picked item of `kind` is offered: a folder (a collapsed one too) reads as a folder, anything else as a file.
pub fn own_file_reason(kind: NodeKind) -> Text {
    let segment = match kind {
        NodeKind::Folder | NodeKind::Collapsed => "folder",
        NodeKind::File | NodeKind::Link => "file",
    };
    Text::new(TextKey::from_valid(format!("core.own-file.reason.{segment}")))
}

#[cfg(test)]
mod tests {
    mod positive {
        use super::super::{OWN_FILE_ID, own_file, own_file_reason, own_file_title};
        use crate::{CategoryIcon, DangerLevel, ModuleId, NodeKind, TextArg};

        #[test]
        fn the_title_of_a_pick_names_it() {
            let title = own_file_title("Movie.mov");

            assert_eq!(title.key().as_str(), "core.own-file.title");
            assert_eq!(title.args().get("name"), Some(&TextArg::Text("Movie.mov".into())));
        }

        #[test]
        fn a_folder_reads_as_a_folder_and_anything_else_as_a_file() {
            let reason = |kind| own_file_reason(kind).key().as_str().to_owned();

            assert_eq!(reason(NodeKind::Folder), "core.own-file.reason.folder");
            assert_eq!(reason(NodeKind::Collapsed), "core.own-file.reason.folder");
            assert_eq!(reason(NodeKind::File), "core.own-file.reason.file");
        }

        #[test]
        fn the_own_file_category_is_the_core_s_and_high() {
            let category = own_file();

            assert_eq!(category.id().as_str(), OWN_FILE_ID);
            assert_eq!(category.id().module(), ModuleId::system());
            assert_eq!(category.danger(), DangerLevel::High);
            assert_eq!(category.needs(), []);
            assert_eq!(category.icon(), CategoryIcon::Generic);
        }
    }
}
