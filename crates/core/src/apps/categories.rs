//! The two categories of an uninstall: the core's own, offered only from an app's page, never on Scan.

use crate::{CategoryDescriptor, CategoryIcon, DangerLevel, Text, TextArg, TextKey, UninstallChoice};

/// `core.uninstall-keep-data` at Medium and `core.uninstall-everything` at High, in `UninstallChoice::ALL`'s order.
/// No registry holds them: no scan finds them.
pub fn uninstall_categories() -> [CategoryDescriptor; 2] {
    UninstallChoice::ALL.map(uninstall_category)
}

/// The category of an uninstall with `choice`, with the app icon.
pub fn uninstall_category(choice: UninstallChoice) -> CategoryDescriptor {
    let danger = match choice {
        UninstallChoice::KeepData => DangerLevel::Medium,
        UninstallChoice::Everything => DangerLevel::High,
    };
    CategoryDescriptor::new(choice.category(), danger).with_icon(CategoryIcon::App)
}

/// The title of an uninstall finding: the app's name.
pub fn uninstall_title(app: &str) -> Text {
    Text::new(TextKey::from_valid("core.uninstall.title".to_owned())).with("app", TextArg::Text(app.to_owned()))
}

#[cfg(test)]
mod tests {
    mod positive {
        use crate::{
            CategoryIcon, DangerLevel, ModuleId, TextArg, UninstallChoice, uninstall_categories, uninstall_title,
        };

        #[test]
        fn each_choice_names_its_category() {
            let categories = uninstall_categories();

            let rows: Vec<_> = categories.iter().map(|category| (category.id().as_str(), category.danger())).collect();

            assert_eq!(
                rows,
                [("core.uninstall-keep-data", DangerLevel::Medium), ("core.uninstall-everything", DangerLevel::High)]
            );
            for (choice, category) in UninstallChoice::ALL.into_iter().zip(&categories) {
                assert_eq!(choice.category(), category.id());
                assert_eq!(category.id().module(), ModuleId::system());
                assert_eq!(category.needs(), []);
                assert_eq!(category.icon(), CategoryIcon::App);
            }
        }

        #[test]
        fn the_title_names_the_app() {
            let title = uninstall_title("Editor");

            assert_eq!(title.key().as_str(), "core.uninstall.title");
            assert_eq!(title.args().get("app"), Some(&TextArg::Text("Editor".into())));
        }
    }
}
