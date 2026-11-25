//! The Apps list and an app's page as the UI receives them.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Access, ByteSize, Capability, CapabilityText, CategoryId, DangerLevel, FootprintKind, KeptReason, Manager,
    NotOffered, Sizes, StateId, Text, TextArg, TextKey, Timestamp, UninstallChoice,
};

/// An app of one state's list; another state refuses it, like a `NodeId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct AppId {
    state: StateId,
    index: u32,
}

impl AppId {
    pub const fn new(state: StateId, index: u32) -> Self {
        Self { state, index }
    }

    pub const fn state(self) -> StateId {
        self.state
    }

    pub const fn index(self) -> u32 {
        self.index
    }
}

/// Whether an app may be uninstalled from its page.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AppOffer {
    Offered,
    /// Shown with its size and the reason's sentence, never offered.
    NotOffered {
        reason: NotOffered,
        text: Text,
    },
}

impl AppOffer {
    /// Not offered for `reason`, with its sentence. An app a manager installed is made by `installed_by`, which names
    /// the manager.
    pub fn not_offered(reason: NotOffered) -> Self {
        Self::NotOffered { reason, text: Text::new(reason.text_key()) }
    }

    /// Not offered: `manager` installed it.
    pub fn installed_by(manager: Manager) -> Self {
        let reason = NotOffered::InstalledBy;
        let text = Text::new(reason.text_key()).with("manager", TextArg::Text(manager.name().into()));
        Self::NotOffered { reason, text }
    }
}

/// What the parts of one kind take.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct PartSizes {
    pub kind: FootprintKind,
    pub sizes: Sizes,
}

/// What is known of an app's last use, for its summary sentence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LastUse {
    /// Not used since `since`, long enough ago to say so.
    Unused { since: Timestamp },
    /// Used lately.
    Used,
    /// Not known.
    Unknown,
}

impl LastUse {
    /// The app's sentence: `core.apps.summary.<unused|used|unknown>`, naming `app` and `size` (and `date`).
    pub fn summary(self, app: &str, size: ByteSize) -> Text {
        let text = |segment: &str| {
            Text::new(TextKey::from_valid(format!("core.apps.summary.{segment}")))
                .with("app", TextArg::Text(app.into()))
                .with("size", TextArg::Size(size))
        };
        match self {
            Self::Unused { since } => text("unused").with("date", TextArg::Date(since)),
            Self::Used => text("used"),
            Self::Unknown => text("unknown"),
        }
    }
}

/// One row of the Apps list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppSummary {
    pub app: AppId,
    pub name: String,
    pub version: Option<String>,
    /// The bundle and every part, kept parts too; while High is withheld, no part of a High kind.
    pub sizes: Sizes,
    /// The sizes by kind, kept parts too; while High is withheld, no High kind.
    pub parts: Vec<PartSizes>,
    pub last_used: Option<Timestamp>,
    pub summary: Text,
    pub offer: AppOffer,
}

/// Why a part stays, with its sentence.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct KeptItem {
    pub reason: KeptReason,
    pub text: Text,
}

impl KeptItem {
    pub fn new(reason: KeptReason) -> Self {
        Self { reason, text: reason.text() }
    }
}

/// One file or folder of an app's page.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FootprintItem {
    pub kind: FootprintKind,
    pub path: PathBuf,
    pub sizes: Sizes,
    pub danger: DangerLevel,
    /// Whether it goes with `KeepData`.
    pub keep_data: bool,
    /// Whether it goes with `Everything`.
    pub everything: bool,
    pub kept: Option<KeptItem>,
}

impl FootprintItem {
    /// A part of `kind`: its danger is the kind's, and it goes with the choices its kind goes with unless it is kept.
    pub fn new(kind: FootprintKind, path: PathBuf, sizes: Sizes, kept: Option<KeptReason>) -> Self {
        let goes = |choice| kept.is_none() && kind.goes_with(choice);
        Self {
            kind,
            path,
            sizes,
            danger: kind.danger(),
            keep_data: goes(UninstallChoice::KeepData),
            everything: goes(UninstallChoice::Everything),
            kept: kept.map(KeptItem::new),
        }
    }
}

/// One uninstall choice of an app's page: what goes under it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct ChoiceView {
    pub choice: UninstallChoice,
    pub label: Text,
    pub description: Text,
    /// The bundle and the parts that go.
    pub sizes: Sizes,
    /// How many paths go, the bundle included.
    pub items: u32,
}

impl ChoiceView {
    pub fn new(choice: UninstallChoice, sizes: Sizes, items: u32) -> Self {
        Self {
            choice,
            label: Text::new(choice.label_key()),
            description: Text::new(choice.description_key()),
            sizes,
            items,
        }
    }
}

/// An app's page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AppDetail {
    pub summary: AppSummary,
    /// The bundle first, then its parts.
    pub items: Vec<FootprintItem>,
    pub choices: Vec<ChoiceView>,
    /// The app's cleanable categories on Scan.
    pub categories: Vec<CategoryId>,
    /// A package manager's uninstall command, for the user to copy; never run.
    pub command: Option<String>,
    /// A line about the permission an uninstall may need.
    pub notice: Option<Text>,
}

impl AppDetail {
    /// The page's line about App Management in state `access`: that macOS may ask, before the first uninstall
    /// (`NotAsked`); what to do, once macOS refused a move (`Denied`, the capability's `denied` text); none otherwise.
    pub fn management_notice(access: Access) -> Option<Text> {
        match access {
            Access::NotAsked => Some(Text::new(TextKey::from_valid("core.apps.management.not-asked".into()))),
            Access::Denied => Capability::AppManagement.text_key(CapabilityText::Denied).map(Text::new),
            Access::Granted | Access::Unknown | Access::NotApplicable => None,
        }
    }
}

/// How long an app has not been used, for the list's filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum UnusedFor {
    ThreeMonths,
    SixMonths,
    AYear,
}

impl UnusedFor {
    pub const ALL: [Self; 3] = [Self::ThreeMonths, Self::SixMonths, Self::AYear];

    pub const fn days(self) -> u64 {
        match self {
            Self::ThreeMonths => 90,
            Self::SixMonths => 182,
            Self::AYear => 365,
        }
    }
}

/// What the Apps list holds: every app, or only those not used for a while.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppsQuery {
    unused: Option<UnusedFor>,
}

impl AppsQuery {
    pub const fn new(unused: Option<UnusedFor>) -> Self {
        Self { unused }
    }

    pub const fn unused(self) -> Option<UnusedFor> {
        self.unused
    }
}

/// The Apps list, largest first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AppsView {
    pub apps: Vec<AppSummary>,
    pub notice: Option<Text>,
}

impl AppsView {
    /// The installed apps could not be listed: nothing is shown, and the notice says why.
    pub fn unlisted() -> Self {
        Self { apps: Vec::new(), notice: Some(Text::new(TextKey::from_valid("core.apps.no-apps".into()))) }
    }

    /// The apps listed, with the line that files kept under other names are not.
    pub fn listed(apps: Vec<AppSummary>) -> Self {
        Self { apps, notice: Some(Text::new(TextKey::from_valid("core.apps.other-names".into()))) }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{Access, AppDetail, AppsQuery, ByteSize, FootprintItem, FootprintKind, KeptReason, LastUse, Sizes};

        #[test]
        fn an_apps_query_with_an_unknown_field_is_refused() {
            assert!(serde_json::from_str::<AppsQuery>(r#"{ "unused": null, "deep": true }"#).is_err());
            assert!(serde_json::from_str::<AppsQuery>(r#"{ "unused": "a-week" }"#).is_err());
        }

        #[test]
        fn a_kept_part_goes_with_no_choice() {
            let item = FootprintItem::new(FootprintKind::Caches, "/c".into(), Sizes::ZERO, Some(KeptReason::Shared));

            assert!(!item.keep_data);
            assert!(!item.everything);
            assert_eq!(item.kept.unwrap().text.key().as_str(), "core.apps.kept.shared");
        }

        #[test]
        fn no_management_notice_once_granted_or_not_guarded() {
            for access in [Access::Granted, Access::Unknown, Access::NotApplicable] {
                assert_eq!(AppDetail::management_notice(access), None, "{access:?}");
            }
        }

        #[test]
        fn an_unknown_last_use_names_no_date() {
            let text = LastUse::Unknown.summary("Editor", ByteSize::new(1));

            assert_eq!(text.key().as_str(), "core.apps.summary.unknown");
            assert!(!text.args().contains_key("date"));
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{
            AppDetail, AppId, AppOffer, AppSummary, AppsQuery, AppsView, ByteSize, ChoiceView, DangerLevel,
            FootprintItem, FootprintKind, LastUse, Manager, NotOffered, PartSizes, Sizes, StateId, Text, TextArg,
            Timestamp, UninstallChoice, UnusedFor,
        };

        #[test]
        fn the_management_notice_says_macos_may_ask_then_what_to_do_once_denied() {
            let key = |access| AppDetail::management_notice(access).map(|text| text.key().as_str().to_owned());

            assert_eq!(key(crate::Access::NotAsked).as_deref(), Some("core.apps.management.not-asked"));
            assert_eq!(key(crate::Access::Denied).as_deref(), Some("core.access.app-management.denied"));
        }

        fn summary() -> AppSummary {
            AppSummary {
                app: AppId::new(StateId::new(2), 7),
                name: "Editor".into(),
                version: Some("2.1".into()),
                sizes: Sizes::ZERO,
                parts: vec![PartSizes { kind: FootprintKind::SavedState, sizes: Sizes::ZERO }],
                last_used: Some(Timestamp::from_secs(9)),
                summary: LastUse::Used.summary("Editor", ByteSize::new(4096)),
                offer: AppOffer::not_offered(NotOffered::Open),
            }
        }

        #[test]
        fn an_app_id_round_trips_through_json() {
            let id = AppId::new(StateId::new(3), 12);

            let json = serde_json::to_value(id).unwrap();

            assert_eq!(json, json!({ "state": 3, "index": 12 }));
            assert_eq!(serde_json::from_value::<AppId>(json).unwrap(), id);
            assert_eq!((id.state(), id.index()), (StateId::new(3), 12));
        }

        #[test]
        fn the_ipc_types_serialize_in_camel_case() {
            let item = FootprintItem::new(FootprintKind::Settings, "/p/Editor.plist".into(), Sizes::ZERO, None);
            let detail = AppDetail {
                summary: summary(),
                items: vec![item],
                choices: vec![ChoiceView::new(UninstallChoice::KeepData, Sizes::ZERO, 1)],
                categories: Vec::new(),
                command: Some("brew uninstall --cask editor".into()),
                notice: None,
            };

            let json = serde_json::to_value(&detail).unwrap();

            assert_eq!(json["summary"]["lastUsed"], 9);
            assert_eq!(json["summary"]["parts"][0]["kind"], "saved-state");
            assert_eq!(json["summary"]["offer"]["kind"], "not-offered");
            assert_eq!(json["summary"]["offer"]["reason"], "open");
            assert_eq!(json["items"][0]["keepData"], false);
            assert_eq!(json["items"][0]["everything"], true);
            assert_eq!(json["items"][0]["danger"], "medium");
            assert_eq!(json["choices"][0]["choice"], "keep-data");
            assert_eq!(json["choices"][0]["label"]["key"], "core.uninstall.choice.keep-data.label");
            assert_eq!(serde_json::to_value(AppOffer::Offered).unwrap(), json!({ "kind": "offered" }));
        }

        #[test]
        fn an_apps_query_reads_its_filter() {
            let query: AppsQuery = serde_json::from_str(r#"{ "unused": "six-months" }"#).unwrap();

            assert_eq!(query.unused(), Some(UnusedFor::SixMonths));
            assert_eq!(serde_json::from_str::<AppsQuery>("{}").unwrap(), AppsQuery::default());
            assert_eq!(AppsQuery::new(Some(UnusedFor::AYear)).unused(), Some(UnusedFor::AYear));
            assert_eq!(UnusedFor::ALL.map(UnusedFor::days), [90, 182, 365]);
        }

        #[test]
        fn a_part_takes_its_kind_s_danger_and_choices() {
            let item = FootprintItem::new(FootprintKind::Logs, "/l".into(), Sizes::ZERO, None);

            assert_eq!(item.danger, DangerLevel::Safe);
            assert!(item.keep_data && item.everything && item.kept.is_none());
        }

        #[test]
        fn an_unused_app_s_sentence_names_the_date() {
            let text = LastUse::Unused { since: Timestamp::from_secs(5) }.summary("Editor", ByteSize::new(1));

            assert_eq!(text.key().as_str(), "core.apps.summary.unused");
            assert_eq!(text.args().get("date"), Some(&TextArg::Date(Timestamp::from_secs(5))));
            assert_eq!(text.args().get("app"), Some(&TextArg::Text("Editor".into())));
        }

        #[test]
        fn an_app_installed_by_a_manager_names_it() {
            let offer = AppOffer::installed_by(Manager::Homebrew);

            let text = Text::new(NotOffered::InstalledBy.text_key()).with("manager", TextArg::Text("Homebrew".into()));
            assert_eq!(offer, AppOffer::NotOffered { reason: NotOffered::InstalledBy, text });
        }

        #[test]
        fn the_list_says_why_it_is_empty_or_what_it_leaves_out() {
            let notice = |view: AppsView| view.notice.unwrap().key().as_str().to_owned();

            assert_eq!(notice(AppsView::unlisted()), "core.apps.no-apps");
            assert_eq!(notice(AppsView::listed(vec![summary()])), "core.apps.other-names");
        }
    }
}
