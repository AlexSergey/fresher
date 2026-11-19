//! The named groups modules belong to.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::TextKey;

/// The named group a module belongs to; the sidebar and the overview show modules under it. The order of the variants
/// is the screen's order, `Debug` last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleGroup {
    /// What macOS and the user's apps keep for themselves: temporary files.
    System,
    /// What apps leave behind: the files of apps you removed.
    Apps,
    /// Files you downloaded or keep yourself: old installers, big downloads.
    Files,
    /// Tools for building software: IDEs, package managers, containers, project folders.
    Development,
    /// The dev-only modules of a development build (`demo`, `trial`); a release build refuses them.
    Debug,
}

impl ModuleGroup {
    /// Every group, in the screen's order.
    pub const ALL: [Self; 5] = [Self::System, Self::Apps, Self::Files, Self::Development, Self::Debug];

    /// The group's name: `core.group.<group>.name`.
    pub fn name_key(self) -> TextKey {
        TextKey::from_valid(format!("core.group.{}.name", self.as_str()))
    }

    /// What the group holds, in a sentence: `core.group.<group>.description`.
    pub fn description_key(self) -> TextKey {
        TextKey::from_valid(format!("core.group.{}.description", self.as_str()))
    }

    /// The group's serialized name: `system`, `apps`, `files`, `development`, `debug`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Apps => "apps",
            Self::Files => "files",
            Self::Development => "development",
            Self::Debug => "debug",
        }
    }

    /// The group serialized as `name`; `None` for a name no group has.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|group| group.as_str() == name)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::ModuleGroup;

        #[test]
        fn an_unknown_group_is_refused_when_read() {
            assert!(serde_json::from_str::<ModuleGroup>("\"downloads\"").is_err());
            assert!(serde_json::from_str::<ModuleGroup>("\"Development\"").is_err());
            assert_eq!(ModuleGroup::parse("downloads"), None);
            assert_eq!(ModuleGroup::parse("Debug"), None);
        }
    }

    mod positive {
        use crate::ModuleGroup;

        #[test]
        fn groups_are_in_the_screens_order_with_debug_last() {
            assert!(ModuleGroup::ALL.is_sorted());
            assert_eq!(ModuleGroup::ALL.iter().max(), Some(&ModuleGroup::Debug));
        }

        #[test]
        fn serializes_as_kebab_case_matching_its_keys() {
            for group in ModuleGroup::ALL {
                let name = serde_json::to_value(group).unwrap();
                assert_eq!(group.name_key().as_str(), format!("core.group.{}.name", name.as_str().unwrap()));
                assert_eq!(name.as_str(), Some(group.as_str()));
                assert_eq!(ModuleGroup::parse(group.as_str()), Some(group));
                assert_eq!(serde_json::from_value::<ModuleGroup>(name).unwrap(), group);
            }
            assert_eq!(ModuleGroup::Debug.description_key().as_str(), "core.group.debug.description");
        }

        #[test]
        fn lists_every_group_once_with_system_first() {
            assert_eq!(
                ModuleGroup::ALL,
                [
                    ModuleGroup::System,
                    ModuleGroup::Apps,
                    ModuleGroup::Files,
                    ModuleGroup::Development,
                    ModuleGroup::Debug
                ]
            );
        }

        #[test]
        fn apps_come_right_after_system_and_read_back() {
            assert!(ModuleGroup::System < ModuleGroup::Apps && ModuleGroup::Apps < ModuleGroup::Development);
            assert_eq!(ModuleGroup::parse("apps"), Some(ModuleGroup::Apps));
            assert_eq!(serde_json::from_str::<ModuleGroup>("\"apps\"").unwrap(), ModuleGroup::Apps);
        }

        #[test]
        fn files_come_after_apps_and_before_development_and_read_back() {
            assert!(ModuleGroup::Apps < ModuleGroup::Files && ModuleGroup::Files < ModuleGroup::Development);
            assert_eq!(ModuleGroup::parse("files"), Some(ModuleGroup::Files));
            assert_eq!(ModuleGroup::Files.name_key().as_str(), "core.group.files.name");
            assert_eq!(serde_json::from_str::<ModuleGroup>("\"files\"").unwrap(), ModuleGroup::Files);
        }
    }
}
