//! The kinds of an app's footprint, the two uninstall choices, and why a part is kept or an app not offered.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CategoryId, DangerLevel, Removability, Text, TextKey};

/// What a part of an app's footprint is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum FootprintKind {
    /// The app's bundle itself.
    Bundle,
    Caches,
    Logs,
    /// The windows it reopens.
    SavedState,
    /// The agents that start it, or its helpers, at login.
    LaunchAgents,
    /// Its website data and sign-ins.
    WebData,
    /// Its preferences.
    Settings,
    /// The files it keeps for the user.
    Support,
    /// Its sandbox folder.
    Container,
    /// The folder it shares with its maker's other apps.
    GroupContainer,
}

impl FootprintKind {
    pub const ALL: [Self; 10] = [
        Self::Bundle,
        Self::Caches,
        Self::Logs,
        Self::SavedState,
        Self::LaunchAgents,
        Self::WebData,
        Self::Settings,
        Self::Support,
        Self::Container,
        Self::GroupContainer,
    ];

    /// Its name: `core.apps.part.<kind>`.
    pub fn name_key(self) -> TextKey {
        TextKey::from_valid(format!("core.apps.part.{}", self.id()))
    }

    /// The danger shown for a part of this kind.
    pub const fn danger(self) -> DangerLevel {
        match self {
            Self::Logs => DangerLevel::Safe,
            Self::Caches | Self::SavedState => DangerLevel::Low,
            Self::Bundle | Self::LaunchAgents | Self::WebData | Self::Settings => DangerLevel::Medium,
            Self::Support | Self::Container | Self::GroupContainer => DangerLevel::High,
        }
    }

    /// Whether a part of this kind goes with `choice`: what the app rebuilds goes with both, what it keeps for the
    /// user only with `Everything`.
    pub const fn goes_with(self, choice: UninstallChoice) -> bool {
        match self {
            Self::Bundle | Self::Caches | Self::Logs | Self::SavedState | Self::LaunchAgents => true,
            Self::WebData | Self::Settings | Self::Support | Self::Container | Self::GroupContainer => {
                matches!(choice, UninstallChoice::Everything)
            }
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::Bundle => "bundle",
            Self::Caches => "caches",
            Self::Logs => "logs",
            Self::SavedState => "saved-state",
            Self::LaunchAgents => "launch-agents",
            Self::WebData => "web-data",
            Self::Settings => "settings",
            Self::Support => "support",
            Self::Container => "container",
            Self::GroupContainer => "group-container",
        }
    }
}

/// How much of an app an uninstall takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum UninstallChoice {
    /// The app and what it rebuilds; its settings and files stay. The default.
    KeepData,
    /// The app and everything of it Fresher found.
    Everything,
}

impl UninstallChoice {
    pub const ALL: [Self; 2] = [Self::KeepData, Self::Everything];

    /// The category of the uninstall finding: `core.uninstall-keep-data` or `core.uninstall-everything`.
    pub const fn category(self) -> CategoryId {
        match self {
            Self::KeepData => CategoryId::uninstall_keep_data(),
            Self::Everything => CategoryId::uninstall_everything(),
        }
    }

    /// `core.uninstall.choice.<choice>.label`.
    pub fn label_key(self) -> TextKey {
        TextKey::from_valid(format!("core.uninstall.choice.{}.label", self.id()))
    }

    /// `core.uninstall.choice.<choice>.description`.
    pub fn description_key(self) -> TextKey {
        TextKey::from_valid(format!("core.uninstall.choice.{}.description", self.id()))
    }

    /// The uninstall finding's reason, `core.uninstall.reason.<choice>`, with the args `app` and `size`.
    pub fn reason_key(self) -> TextKey {
        TextKey::from_valid(format!("core.uninstall.reason.{}", self.id()))
    }

    const fn id(self) -> &'static str {
        match self {
            Self::KeepData => "keep-data",
            Self::Everything => "everything",
        }
    }
}

/// Why a part is listed but never in an uninstall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum KeptReason {
    /// Another installed bundle names the same id.
    Shared,
    /// A group container another installed app of the same maker may use, or the app has no team id.
    TeamShared,
    /// The entry is a link.
    Link,
    /// A container while other apps' data is not allowed.
    NeedsPermission,
    /// The scan did not read it in full.
    NotScanned,
    /// The user ignored it, something inside it or a folder holding it.
    Ignored,
}

impl KeptReason {
    pub const ALL: [Self; 6] =
        [Self::Shared, Self::TeamShared, Self::Link, Self::NeedsPermission, Self::NotScanned, Self::Ignored];

    /// Why it stays: `core.apps.kept.<reason>`.
    pub fn text(self) -> Text {
        let id = match self {
            Self::Shared => "shared",
            Self::TeamShared => "team-shared",
            Self::Link => "link",
            Self::NeedsPermission => "needs-permission",
            Self::NotScanned => "not-scanned",
            Self::Ignored => "ignored",
        };
        Text::new(TextKey::from_valid(format!("core.apps.kept.{id}")))
    }
}

/// Why an app is listed but not offered for uninstall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum NotOffered {
    /// Its bundle's facts could not be read.
    Unmeasured,
    /// A package manager installed it.
    InstalledBy,
    /// It belongs to another user or the system, or its folder is not the user's to write.
    NeedsAdministrator,
    /// The user locked it.
    Locked,
    /// The system protects it.
    Protected,
    /// It, or one of its helpers, runs now.
    Open,
    /// Its bundle and parts are more paths than one item may hold.
    TooManyFiles,
}

impl NotOffered {
    pub const ALL: [Self; 7] = [
        Self::Unmeasured,
        Self::InstalledBy,
        Self::NeedsAdministrator,
        Self::Locked,
        Self::Protected,
        Self::Open,
        Self::TooManyFiles,
    ];

    /// The reason for a bundle the user may not move; `None` for a removable one.
    pub const fn of(removability: Removability) -> Option<Self> {
        match removability {
            Removability::Removable => None,
            Removability::NeedsAdministrator => Some(Self::NeedsAdministrator),
            Removability::Locked => Some(Self::Locked),
            Removability::ProtectedBySystem => Some(Self::Protected),
        }
    }

    /// The code it is serialized with.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unmeasured => "unmeasured",
            Self::InstalledBy => "installed-by",
            Self::NeedsAdministrator => "needs-administrator",
            Self::Locked => "locked",
            Self::Protected => "protected",
            Self::Open => "open",
            Self::TooManyFiles => "too-many-files",
        }
    }

    /// What stands in the way and what to do: `core.apps.not-offered.<code>`; `InstalledBy`'s names `{manager}`.
    pub fn text_key(self) -> TextKey {
        TextKey::from_valid(format!("core.apps.not-offered.{}", self.code()))
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{NotOffered, Removability};

        #[test]
        fn a_removable_bundle_is_not_a_reason() {
            assert_eq!(NotOffered::of(Removability::Removable), None);
        }
    }

    mod positive {
        use crate::{DangerLevel, FootprintKind, NotOffered, Removability, UninstallChoice};

        #[test]
        fn every_kind_has_its_danger_and_its_choices() {
            let table: Vec<_> = FootprintKind::ALL
                .into_iter()
                .map(|kind| {
                    let goes = UninstallChoice::ALL.map(|choice| kind.goes_with(choice));
                    (kind.name_key().as_str().to_owned(), kind.danger(), goes)
                })
                .collect();

            let row = |key: &str, danger, goes| (format!("core.apps.part.{key}"), danger, goes);
            assert_eq!(
                table,
                [
                    row("bundle", DangerLevel::Medium, [true, true]),
                    row("caches", DangerLevel::Low, [true, true]),
                    row("logs", DangerLevel::Safe, [true, true]),
                    row("saved-state", DangerLevel::Low, [true, true]),
                    row("launch-agents", DangerLevel::Medium, [true, true]),
                    row("web-data", DangerLevel::Medium, [false, true]),
                    row("settings", DangerLevel::Medium, [false, true]),
                    row("support", DangerLevel::High, [false, true]),
                    row("container", DangerLevel::High, [false, true]),
                    row("group-container", DangerLevel::High, [false, true]),
                ]
            );
        }

        #[test]
        fn each_choice_names_its_texts() {
            let keys = |choice: UninstallChoice| {
                [choice.label_key(), choice.description_key(), choice.reason_key()].map(|key| key.as_str().to_owned())
            };

            assert_eq!(
                keys(UninstallChoice::KeepData),
                [
                    "core.uninstall.choice.keep-data.label",
                    "core.uninstall.choice.keep-data.description",
                    "core.uninstall.reason.keep-data",
                ]
            );
            assert_eq!(
                keys(UninstallChoice::Everything),
                [
                    "core.uninstall.choice.everything.label",
                    "core.uninstall.choice.everything.description",
                    "core.uninstall.reason.everything",
                ]
            );
        }

        #[test]
        fn each_removability_maps_to_its_reason() {
            assert_eq!(NotOffered::of(Removability::NeedsAdministrator), Some(NotOffered::NeedsAdministrator));
            assert_eq!(NotOffered::of(Removability::Locked), Some(NotOffered::Locked));
            assert_eq!(NotOffered::of(Removability::ProtectedBySystem), Some(NotOffered::Protected));
        }

        #[test]
        fn each_reason_s_code_is_its_serialized_name_and_its_key() {
            for reason in NotOffered::ALL {
                assert_eq!(serde_json::to_value(reason).unwrap(), reason.code());
                assert_eq!(reason.text_key().as_str(), format!("core.apps.not-offered.{}", reason.code()));
            }
        }
    }
}
