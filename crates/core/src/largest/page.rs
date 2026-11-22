//! What the Largest screen asks and what it is answered: a query, a page of rows, and the folder view's rows.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ItemKind;
use crate::{ByteSize, FindingId, NodeId, NodeInfo, Sizes, TextKey, Timestamp};

/// The most rows one page holds; the engine caps a larger `limit`.
pub const MAX_PAGE: u32 = 100;

/// The most items the engine keeps per state: the largest ones (the answer says when more were left out).
pub const MAX_ITEMS: u32 = 10_000;

const DAY: u64 = 86_400;

/// How long an item has gone unchanged, as a filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Unchanged {
    Any,
    Month,
    HalfYear,
    Year,
    TwoYears,
}

impl Unchanged {
    /// Every choice, in the order of the enum.
    pub const ALL: [Self; 5] = [Self::Any, Self::Month, Self::HalfYear, Self::Year, Self::TwoYears];

    /// The choice's label: `core.largest.unchanged.<kebab>`.
    pub fn text_key(self) -> TextKey {
        let segment = match self {
            Self::Any => "any",
            Self::Month => "month",
            Self::HalfYear => "half-year",
            Self::Year => "year",
            Self::TwoYears => "two-years",
        };
        TextKey::from_valid(format!("core.largest.unchanged.{segment}"))
    }

    /// The time an item must be older than: 30, 182, 365 or 730 days before `taken` (the scan's time, never the
    /// clock); `None` for `Any`. Saturates at the epoch.
    pub fn before(self, taken: Timestamp) -> Option<Timestamp> {
        let days = match self {
            Self::Any => return None,
            Self::Month => 30,
            Self::HalfYear => 182,
            Self::Year => 365,
            Self::TwoYears => 730,
        };
        Some(Timestamp::from_secs(taken.secs().saturating_sub(days * DAY)))
    }
}

/// One page of the Files list: what to keep, and where to start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LargestQuery {
    /// Only items of this kind.
    pub kind: Option<ItemKind>,
    /// Only items inside this folder.
    pub place: Option<NodeId>,
    pub unchanged: Unchanged,
    pub offset: u32,
    /// At most `MAX_PAGE`; the engine caps a larger one.
    pub limit: u32,
}

/// Why a row cannot be chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Withheld {
    /// A module explains it on Scan.
    Explained,
    /// The home's `Library`.
    Library,
    /// The home's Trash.
    Trash,
    /// A hidden name, or inside a hidden folder.
    Hidden,
    /// An app bundle.
    App,
    /// A link.
    Link,
    /// Fresher could not look inside.
    NotSeen,
    /// A standard folder of the Mac: open it to choose what is inside.
    StandardFolder,
    /// A place macOS or Fresher protects.
    Protected,
    /// Part of it is explained on Scan.
    HoldsExplained,
    /// Its size is not complete.
    ReadInPart,
    /// At or below a path the user ignores.
    Ignored,
}

impl Withheld {
    /// Every reason, in the order of the enum.
    pub const ALL: [Self; 12] = [
        Self::Explained,
        Self::Library,
        Self::Trash,
        Self::Hidden,
        Self::App,
        Self::Link,
        Self::NotSeen,
        Self::StandardFolder,
        Self::Protected,
        Self::HoldsExplained,
        Self::ReadInPart,
        Self::Ignored,
    ];

    /// The reason in plain words: `core.largest.withheld.<kebab>`.
    pub fn text_key(self) -> TextKey {
        let segment = match self {
            Self::Explained => "explained",
            Self::Library => "library",
            Self::Trash => "trash",
            Self::Hidden => "hidden",
            Self::App => "app",
            Self::Link => "link",
            Self::NotSeen => "not-seen",
            Self::StandardFolder => "standard-folder",
            Self::Protected => "protected",
            Self::HoldsExplained => "holds-explained",
            Self::ReadInPart => "read-in-part",
            Self::Ignored => "ignored",
        };
        TextKey::from_valid(format!("core.largest.withheld.{segment}"))
    }
}

/// Whether a row can be chosen, as the engine judges it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "reason", rename_all = "kebab-case")]
pub enum Offer {
    Offered,
    Withheld(Withheld),
}

/// One row of the Files list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LargestRow {
    pub node: NodeInfo,
    /// The folder the item is in, as a path from the home, for display.
    pub place: String,
    pub kind: ItemKind,
    pub offer: Offer,
    /// The name of a folder beside an archive that may hold what was unpacked from it.
    pub unpacked: Option<String>,
    /// The finding the user already picked this item as.
    pub picked: Option<FindingId>,
}

/// How many items of one kind the filters leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub kind: ItemKind,
    pub count: u32,
}

/// How many items a folder holds under the filters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlaceCount {
    pub folder: NodeInfo,
    pub count: u32,
}

/// The answer to a `LargestQuery`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LargestPage {
    pub rows: Vec<LargestRow>,
    pub offset: u32,
    /// How many items the filters leave, in all.
    pub total: u32,
    /// The sizes of every item the filters leave.
    pub sizes: Sizes,
    /// The kinds present under the other filters, with their counts.
    pub kinds: Vec<KindCount>,
    /// The folders that hold the most items under the other filters.
    pub places: Vec<PlaceCount>,
    /// More items than `MAX_ITEMS` were found: the smallest were left out.
    pub truncated: bool,
    /// The smallest size listed (the setting's value).
    pub min_size: ByteSize,
}

/// One row of the folder view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OwnRow {
    pub node: NodeInfo,
    /// The part of it a module explains.
    pub explained: Sizes,
    pub kind: ItemKind,
    pub offer: Offer,
    /// Whether the row opens to its children.
    pub opens: bool,
    pub picked: Option<FindingId>,
}

/// One page of the folder view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OwnPlace {
    /// The way from the home (first) to the opened folder (last).
    pub path: Vec<NodeInfo>,
    pub rows: Vec<OwnRow>,
    pub offset: u32,
    pub total: u32,
}

#[cfg(test)]
mod tests {
    mod negative {
        use super::super::{LargestQuery, Unchanged};
        use crate::Timestamp;

        #[test]
        fn the_page_and_item_caps_are_pinned() {
            assert_eq!((super::super::MAX_PAGE, super::super::MAX_ITEMS), (100, 10_000));
        }

        #[test]
        fn an_unknown_field_of_a_query_is_refused() {
            let json = r#"{"kind":null,"place":null,"unchanged":"any","offset":0,"limit":10,"extra":1}"#;

            assert!(serde_json::from_str::<LargestQuery>(json).is_err());
        }

        #[test]
        fn any_has_no_cutoff() {
            assert_eq!(Unchanged::Any.before(Timestamp::from_secs(1_000_000)), None);
        }

        #[test]
        fn a_cutoff_saturates_at_the_epoch() {
            for choice in [Unchanged::Month, Unchanged::HalfYear, Unchanged::Year, Unchanged::TwoYears] {
                assert_eq!(choice.before(Timestamp::from_secs(5)), Some(Timestamp::from_secs(0)), "{choice:?}");
            }
        }
    }

    mod positive {
        use serde_json::json;

        use super::super::{LargestQuery, LargestRow, Offer, Unchanged, Withheld};
        use crate::{
            ByteSize, FindingId, ItemKind, NodeId, NodeInfo, NodeKind, Sizes, StateId, Timestamp, core_dictionary,
        };

        fn node() -> NodeInfo {
            NodeInfo {
                id: NodeId::new(StateId::new(1), 4),
                name: "Movie.mov".into(),
                kind: NodeKind::File,
                sizes: Sizes::complete(ByteSize::new(10), ByteSize::new(4096)),
                files: 0,
                folders: 0,
                owner: None,
                modified: Some(Timestamp::from_secs(100)),
                not_scanned: None,
            }
        }

        #[test]
        fn unchanged_counts_back_from_the_scan_s_time() {
            let taken = Timestamp::from_secs(100_000_000);
            let day = 86_400;
            let cutoffs: Vec<_> = [Unchanged::Month, Unchanged::HalfYear, Unchanged::Year, Unchanged::TwoYears]
                .map(|choice| choice.before(taken).map(|time| (taken.secs() - time.secs()) / day))
                .into();

            assert_eq!(cutoffs, [Some(30), Some(182), Some(365), Some(730)]);
        }

        #[test]
        fn a_query_is_read_in_camel_case() {
            let json = r#"{"kind":"video","place":null,"unchanged":"half-year","offset":20,"limit":50}"#;

            let query: LargestQuery = serde_json::from_str(json).unwrap();

            assert_eq!(query.kind, Some(ItemKind::Video));
            assert_eq!(query.unchanged, Unchanged::HalfYear);
            assert_eq!((query.offset, query.limit), (20, 50));
        }

        #[test]
        fn an_offer_serializes_with_its_reason() {
            assert_eq!(
                serde_json::to_value(Offer::Withheld(Withheld::StandardFolder)).unwrap(),
                json!({ "kind": "withheld", "reason": "standard-folder" })
            );
            assert_eq!(serde_json::to_value(Offer::Offered).unwrap(), json!({ "kind": "offered" }));
        }

        #[test]
        fn a_row_serializes_in_camel_case() {
            let row = LargestRow {
                node: node(),
                place: "Downloads".into(),
                kind: ItemKind::Video,
                offer: Offer::Offered,
                unpacked: Some("Movie".into()),
                picked: Some(FindingId::new(3)),
            };

            let json = serde_json::to_value(row).unwrap();

            assert_eq!(json["kind"], "video");
            assert_eq!(json["place"], "Downloads");
            assert_eq!(json["unpacked"], "Movie");
            assert_eq!(json["picked"], 3);
            assert_eq!(json["offer"], json!({ "kind": "offered" }));
            assert_eq!(json["node"]["name"], "Movie.mov");
        }

        #[test]
        fn every_withheld_reason_has_its_text() {
            let texts = core_dictionary().unwrap();

            for reason in Withheld::ALL {
                let key = reason.text_key();
                assert!(texts.get(key.as_str()).is_some_and(|text| !text.is_empty()), "{key}");
            }
        }

        #[test]
        fn every_choice_of_unchanged_has_its_text() {
            let texts = core_dictionary().unwrap();

            for choice in Unchanged::ALL {
                let key = choice.text_key();
                assert!(texts.get(key.as_str()).is_some_and(|text| !text.is_empty()), "{key}");
            }
        }
    }
}
