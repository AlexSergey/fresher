//! What the app may see and do: the capabilities a system grants, their states, what a category needs, and what a
//! category can do with the current access. OS-neutral: the names say what is unlocked, not which switch unlocks it.

use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::{Text, TextArg, TextKey, ToolId};

/// Something the app may be allowed to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum Capability {
    /// Every file the user can open (on macOS: Full Disk Access).
    AllFiles,
    /// One of the user's folders the system guards.
    Folder(GuardedFolder),
    /// Other apps' data (their containers).
    OtherAppsData,
    /// A removable or network volume.
    Volume(VolumeKind),
    /// An external program, installed and ready to answer.
    Tool(ToolId),
}

/// A folder of the user's that the system guards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum GuardedFolder {
    Desktop,
    Documents,
    Downloads,
}

/// A volume the system guards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum VolumeKind {
    Removable,
    Network,
}

/// A capability as the UI sends it back to a command. Only the system's capabilities are read: a tool is named by
/// the modules, never by the UI, so one sent from outside is refused.
#[derive(Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
enum CapabilityInput {
    AllFiles,
    Folder(GuardedFolder),
    OtherAppsData,
    Volume(VolumeKind),
    Tool(serde::de::IgnoredAny),
}

impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match CapabilityInput::deserialize(deserializer)? {
            CapabilityInput::AllFiles => Ok(Self::AllFiles),
            CapabilityInput::Folder(folder) => Ok(Self::Folder(folder)),
            CapabilityInput::OtherAppsData => Ok(Self::OtherAppsData),
            CapabilityInput::Volume(kind) => Ok(Self::Volume(kind)),
            CapabilityInput::Tool(_) => Err(serde::de::Error::custom("a tool cannot be named from outside the app")),
        }
    }
}

/// The texts every capability the system grants has, keyed `core.access.<id>.<text>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CapabilityText {
    /// `name`: what the permission is called.
    Name,
    /// `unlocks`: what it lets the app see or do.
    Unlocks,
    /// `how`: the numbered steps that grant it.
    How,
    /// `missing`: the one sentence a section without it shows.
    Missing,
}

impl CapabilityText {
    pub const ALL: [Self; 4] = [Self::Name, Self::Unlocks, Self::How, Self::Missing];

    /// The last segment of the text's key.
    pub const fn segment(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Unlocks => "unlocks",
            Self::How => "how",
            Self::Missing => "missing",
        }
    }
}

impl Capability {
    /// Every capability the system grants: all but the tools, which the modules name.
    pub const SYSTEM: [Self; 7] = [
        Self::AllFiles,
        Self::Folder(GuardedFolder::Desktop),
        Self::Folder(GuardedFolder::Documents),
        Self::Folder(GuardedFolder::Downloads),
        Self::OtherAppsData,
        Self::Volume(VolumeKind::Removable),
        Self::Volume(VolumeKind::Network),
    ];

    /// A kebab-case name: `all-files`, `desktop`, `removable-volume`, or the tool's id.
    pub const fn id(self) -> &'static str {
        match self {
            Self::AllFiles => "all-files",
            Self::Folder(GuardedFolder::Desktop) => "desktop",
            Self::Folder(GuardedFolder::Documents) => "documents",
            Self::Folder(GuardedFolder::Downloads) => "downloads",
            Self::OtherAppsData => "other-apps-data",
            Self::Volume(VolumeKind::Removable) => "removable-volume",
            Self::Volume(VolumeKind::Network) => "network-volume",
            Self::Tool(tool) => tool.as_str(),
        }
    }

    /// The key of one of its texts in the core's dictionary: `core.access.<id>.<segment>`. A tool has none: its
    /// state carries a note instead.
    pub fn text_key(self, text: CapabilityText) -> Option<TextKey> {
        match self {
            Self::Tool(_) => None,
            _ => Some(TextKey::from_valid(format!("core.access.{}.{}", self.id(), text.segment()))),
        }
    }
}

/// The state of one capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    Granted,
    Denied,
    /// The app has not looked, because looking could make the system ask the user.
    NotAsked,
    /// The check could not tell.
    Unknown,
    /// This system has no such permission: nothing guards what it would unlock.
    NotApplicable,
}

impl Access {
    pub const ALL: [Self; 5] = [Self::Granted, Self::Denied, Self::NotAsked, Self::Unknown, Self::NotApplicable];

    /// Whether what the capability unlocks can be used: granted, or not guarded at all.
    pub const fn allows(self) -> bool {
        matches!(self, Self::Granted | Self::NotApplicable)
    }

    /// The key of its name: `core.access.state.<state>`.
    pub fn text_key(self) -> TextKey {
        let state = match self {
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::NotAsked => "not-asked",
            Self::Unknown => "unknown",
            Self::NotApplicable => "not-applicable",
        };
        TextKey::from_valid(format!("core.access.state.{state}"))
    }
}

/// One capability and its state, with a plain-English detail when the state needs one ("Docker is installed but not
/// running").
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct CapabilityState {
    pub capability: Capability,
    pub access: Access,
    pub note: Option<Text>,
}

impl CapabilityState {
    /// `capability` in state `access`, with no note.
    pub const fn new(capability: Capability, access: Access) -> Self {
        Self { capability, access, note: None }
    }

    /// `tool` is not installed: denied, with the note `core.access.tool.not-installed`.
    pub fn tool_not_installed(tool: ToolId) -> Self {
        Self::tool_denied(tool, "core.access.tool.not-installed")
    }

    /// `tool` is installed but did not answer its check: denied, with the note `core.access.tool.not-ready`.
    pub fn tool_not_ready(tool: ToolId) -> Self {
        Self::tool_denied(tool, "core.access.tool.not-ready")
    }

    fn tool_denied(tool: ToolId, key: &str) -> Self {
        let note = Text::new(TextKey::from_valid(key.into())).with("tool", TextArg::Text(tool.as_str().into()));
        Self { capability: Capability::Tool(tool), access: Access::Denied, note: Some(note) }
    }
}

/// What the app may see and do: every capability this system has, with its state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, TS)]
pub struct AccessStatus {
    pub capabilities: Vec<CapabilityState>,
}

impl AccessStatus {
    /// The state of `capability`; one that is not listed is `NotApplicable`.
    pub fn access(&self, capability: &Capability) -> Access {
        self.capabilities
            .iter()
            .find(|state| &state.capability == capability)
            .map_or(Access::NotApplicable, |state| state.access)
    }

    /// `Ok` when every need is allowed, else every capability missing, once each, in the order of `needs`.
    pub fn check(&self, needs: &[Need]) -> Result<(), Vec<Capability>> {
        let missing = missing(needs.iter(), self);
        if missing.is_empty() { Ok(()) } else { Err(missing) }
    }
}

/// A place the system guards, and the capability that opens it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GuardedPlace {
    pub path: PathBuf,
    pub capability: Capability,
}

/// How much a category depends on a capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum NeedKind {
    /// Without it the category cannot be found at all.
    Required,
    /// Without it the category is found, but part of it may be hidden.
    SeesMore,
}

/// What a category needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct Need {
    pub capability: Capability,
    pub kind: NeedKind,
}

impl Need {
    pub const fn required(capability: Capability) -> Self {
        Self { capability, kind: NeedKind::Required }
    }

    pub const fn sees_more(capability: Capability) -> Self {
        Self { capability, kind: NeedKind::SeesMore }
    }
}

/// What a category can do with the current access.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Availability {
    Ready,
    /// Found, but part of it may be hidden without `missing`: its totals are partial.
    Limited {
        missing: Vec<Capability>,
    },
    /// Cannot be found without `missing`, the required capabilities not allowed.
    Unavailable {
        missing: Vec<Capability>,
    },
}

impl Availability {
    /// `Unavailable` when a required need is not allowed, `Limited` when only a `SeesMore` one is not, else `Ready`.
    pub fn of(needs: &[Need], status: &AccessStatus) -> Self {
        let of_kind = |kind: NeedKind| needs.iter().filter(move |need| need.kind == kind);
        let required = missing(of_kind(NeedKind::Required), status);
        if !required.is_empty() {
            return Self::Unavailable { missing: required };
        }
        let more = missing(of_kind(NeedKind::SeesMore), status);
        if more.is_empty() { Self::Ready } else { Self::Limited { missing: more } }
    }
}

/// The capabilities of `needs` that `status` does not allow, once each, in order.
fn missing<'a>(needs: impl Iterator<Item = &'a Need>, status: &AccessStatus) -> Vec<Capability> {
    let mut missing: Vec<Capability> = Vec::new();
    for need in needs {
        if !status.access(&need.capability).allows() && !missing.contains(&need.capability) {
            missing.push(need.capability);
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use crate::{Access, AccessStatus, Capability, CapabilityState, GuardedFolder, ToolId};

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);

    fn docker() -> Capability {
        Capability::Tool(ToolId::new("docker").unwrap())
    }

    fn status(states: &[(Capability, Access)]) -> AccessStatus {
        let capabilities = states.iter().map(|&(capability, access)| CapabilityState::new(capability, access));
        AccessStatus { capabilities: capabilities.collect() }
    }

    mod negative {
        use super::{DESKTOP, docker, status};
        use crate::{Access, Availability, Capability, CapabilityText, Need};

        #[test]
        fn a_capability_that_is_not_listed_is_not_applicable() {
            let status = status(&[(Capability::AllFiles, Access::Denied)]);

            assert_eq!(status.access(&DESKTOP), Access::NotApplicable);
            assert_eq!(status.access(&docker()), Access::NotApplicable);
            assert_eq!(status.access(&Capability::AllFiles), Access::Denied);
        }

        #[test]
        fn a_required_need_that_is_denied_makes_the_category_unavailable() {
            let status = status(&[(docker(), Access::Denied), (Capability::AllFiles, Access::Unknown)]);
            let needs = [Need::required(docker()), Need::sees_more(Capability::AllFiles)];

            assert_eq!(Availability::of(&needs, &status), Availability::Unavailable { missing: vec![docker()] });
        }

        #[test]
        fn a_need_that_is_not_asked_or_unknown_is_missing() {
            for access in [Access::NotAsked, Access::Unknown, Access::Denied] {
                let status = status(&[(DESKTOP, access)]);

                assert_eq!(status.check(&[Need::required(DESKTOP)]), Err(vec![DESKTOP]), "{access:?}");
                assert!(!access.allows());
            }
        }

        #[test]
        fn check_reports_every_missing_capability() {
            let status = status(&[
                (Capability::AllFiles, Access::Denied),
                (DESKTOP, Access::NotAsked),
                (docker(), Access::Granted),
            ]);
            let needs = [
                Need::sees_more(Capability::AllFiles),
                Need::required(docker()),
                Need::required(DESKTOP),
                Need::required(Capability::AllFiles),
            ];

            assert_eq!(status.check(&needs), Err(vec![Capability::AllFiles, DESKTOP]));
        }

        #[test]
        fn a_tool_sent_from_outside_is_refused() {
            let error = serde_json::from_value::<Capability>(serde_json::json!({ "kind": "tool", "value": "docker" }));
            let unknown =
                serde_json::from_value::<Capability>(serde_json::json!({ "kind": "folder", "value": "music" }));

            assert!(error.unwrap_err().to_string().contains("a tool cannot be named"));
            assert!(unknown.is_err());
        }

        #[test]
        fn a_tool_has_no_texts_in_the_core() {
            for text in CapabilityText::ALL {
                assert_eq!(docker().text_key(text), None);
            }
        }
    }

    mod positive {
        use serde_json::json;
        use ts_rs::{Config, TS};

        use super::{DESKTOP, docker, status};
        use crate::{
            Access, AccessStatus, Availability, Capability, CapabilityState, CapabilityText, GuardedFolder, Need,
            NeedKind, VolumeKind,
        };

        #[test]
        fn a_sees_more_need_that_is_not_granted_limits_the_category() {
            let status = status(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)]);
            let needs = [Need::sees_more(Capability::AllFiles), Need::sees_more(DESKTOP)];

            assert_eq!(
                Availability::of(&needs, &status),
                Availability::Limited { missing: vec![Capability::AllFiles, DESKTOP] }
            );
        }

        #[test]
        fn granted_needs_make_the_category_ready() {
            let status = status(&[(Capability::AllFiles, Access::Granted), (docker(), Access::Granted)]);
            let needs = [Need::required(docker()), Need::sees_more(Capability::AllFiles), Need::sees_more(DESKTOP)];

            assert_eq!(Availability::of(&needs, &status), Availability::Ready);
            assert_eq!(Availability::of(&[], &AccessStatus::default()), Availability::Ready);
            assert_eq!(status.check(&needs), Ok(()));
        }

        #[test]
        fn capabilities_serialize_with_kebab_case_tags() {
            assert_eq!(serde_json::to_value(DESKTOP).unwrap(), json!({ "kind": "folder", "value": "desktop" }));
            assert_eq!(serde_json::to_value(Capability::AllFiles).unwrap(), json!({ "kind": "all-files" }));
            assert_eq!(
                serde_json::to_value(Capability::Volume(VolumeKind::Removable)).unwrap(),
                json!({ "kind": "volume", "value": "removable" })
            );
            assert_eq!(serde_json::to_value(docker()).unwrap(), json!({ "kind": "tool", "value": "docker" }));
            assert_eq!(
                serde_json::to_value(status(&[(Capability::OtherAppsData, Access::NotApplicable)])).unwrap(),
                json!({ "capabilities": [
                    { "capability": { "kind": "other-apps-data" }, "access": "not-applicable", "note": null },
                ] })
            );
            assert_eq!(
                serde_json::to_value(Availability::Limited { missing: vec![Capability::AllFiles] }).unwrap(),
                json!({ "kind": "limited", "missing": [{ "kind": "all-files" }] })
            );
            assert_eq!(
                serde_json::to_value(Need::sees_more(DESKTOP)).unwrap(),
                json!({ "capability": { "kind": "folder", "value": "desktop" }, "kind": "sees-more" })
            );
        }

        #[test]
        fn exports_the_access_types_to_typescript() {
            let config = Config::new();

            let capability = Capability::decl(&config);
            assert!(capability.contains(r#"{ "kind": "folder", "value": GuardedFolder }"#), "{capability}");
            assert!(capability.contains(r#"{ "kind": "all-files" }"#), "{capability}");
            assert!(Access::decl(&config).contains(r#""not-asked""#));
            assert!(AccessStatus::decl(&config).contains("capabilities: Array<CapabilityState>"));
            assert!(CapabilityState::decl(&config).contains("note: Text | null"));
            assert!(Availability::decl(&config).contains(r#""kind": "unavailable", missing: Array<Capability>"#));
            assert!(NeedKind::decl(&config).contains(r#""sees-more""#));
            assert!(GuardedFolder::decl(&config).contains(r#""downloads""#));
            assert!(VolumeKind::decl(&config).contains(r#""network""#));
        }

        #[test]
        fn names_every_system_capability_and_its_texts() {
            let ids = Capability::SYSTEM.map(Capability::id);

            assert_eq!(
                ids,
                [
                    "all-files",
                    "desktop",
                    "documents",
                    "downloads",
                    "other-apps-data",
                    "removable-volume",
                    "network-volume"
                ]
            );
            assert_eq!(DESKTOP.text_key(CapabilityText::Missing).unwrap().as_str(), "core.access.desktop.missing");
            assert_eq!(docker().id(), "docker");
            assert_eq!(Access::NotAsked.text_key().as_str(), "core.access.state.not-asked");
            assert_eq!(Capability::Folder(GuardedFolder::Downloads).id(), "downloads");
        }

        #[test]
        fn a_tool_that_is_not_ready_is_denied_with_a_note_naming_it() {
            let tool = crate::ToolId::new("docker").unwrap();

            let missing = CapabilityState::tool_not_installed(tool);
            let stopped = CapabilityState::tool_not_ready(tool);

            assert_eq!((missing.capability, missing.access), (docker(), Access::Denied));
            assert_eq!(
                serde_json::to_value(&missing.note).unwrap(),
                json!({ "key": "core.access.tool.not-installed", "args": { "tool": { "kind": "text", "value": "docker" } } })
            );
            assert_eq!(stopped.note.unwrap().key().as_str(), "core.access.tool.not-ready");
        }

        #[test]
        fn every_system_capability_reads_back_what_it_serializes() {
            for capability in Capability::SYSTEM {
                let json = serde_json::to_value(capability).unwrap();

                assert_eq!(serde_json::from_value::<Capability>(json).unwrap(), capability);
            }
        }

        #[test]
        fn granted_and_not_applicable_allow() {
            assert!(Access::Granted.allows());
            assert!(Access::NotApplicable.allows());
            assert_eq!(CapabilityState::new(DESKTOP, Access::Granted).note, None);
        }
    }
}
