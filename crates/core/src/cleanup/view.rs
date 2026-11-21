//! What the cleanup commands take from and return to the UI. No path crosses the IPC but the display paths of an
//! item; the UI names items by their ids and indexes.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cleanup::{
    Action, ActionKind, BlockedReason, CleanupId, Ended, LeftOut, Method, OutcomeCounts, PlanId, PlanItem, PlanSummary,
    PutBackOutcome, RecordedOutcome, TrashState,
};
use crate::{ByteSize, CategoryId, DangerLevel, FindingId, Sizes, Text, Timestamp};

/// The answer of `prepare_cleanup`: the plan exactly as the confirmation shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanView {
    pub plan: PlanId,
    /// What the user confirms, and what the confirmation sends back.
    pub summary: PlanSummary,
    /// The items by category and by how they go, in the order they are acted on.
    pub groups: Vec<PlanGroup>,
    /// What will not be removed, each with its reason; not in the summary.
    pub left_out: Vec<LeftOut>,
    /// Whether the permanent choice applies to any item (a tool's items go the same way either way).
    pub permanent_available: bool,
}

/// The items of one category that go the same way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PlanGroup {
    pub category: CategoryId,
    pub action: ActionKind,
    pub summary: PlanSummary,
    pub items: Vec<PlanItemView>,
}

/// One item as the confirmation lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PlanItemView {
    pub finding: FindingId,
    pub title: Text,
    pub reason: Text,
    pub danger: DangerLevel,
    pub sizes: Sizes,
    /// Where it is, to show; `None` for a tool's item.
    pub path: Option<PathBuf>,
    /// The other paths that leave with it, to show: the parts after the subject of an item of several paths, empty
    /// for any other item.
    pub more: Vec<PathBuf>,
}

impl PlanItemView {
    pub fn of(item: &PlanItem) -> Self {
        Self {
            finding: item.finding(),
            title: item.title().clone(),
            reason: item.reason().clone(),
            danger: item.danger(),
            sizes: item.sizes(),
            path: item.action().path().map(PathBuf::from),
            more: match item.action() {
                Action::RemoveTogether(parts) => parts.rest().to_vec(),
                Action::Remove(_) | Action::Empty(_) | Action::Tool(_) => Vec::new(),
            },
        }
    }
}

/// The input of `get_cleanups`: a page of the history, newest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CleanupsQuery {
    offset: u32,
    limit: u32,
}

impl CleanupsQuery {
    /// The largest page the engine returns.
    pub const MAX_LIMIT: u32 = 100;

    pub const fn new(offset: u32, limit: u32) -> Self {
        Self { offset, limit }
    }

    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// The page size asked for, capped at `MAX_LIMIT`.
    pub const fn limit(self) -> u32 {
        if self.limit > Self::MAX_LIMIT { Self::MAX_LIMIT } else { self.limit }
    }
}

/// The answer of `get_cleanups`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CleanupsPage {
    pub cleanups: Vec<CleanupSummary>,
    pub offset: u32,
    /// How many cleanups the history holds.
    pub total: u32,
    /// The sizes of every item of every cleanup still in the Trash: "Waiting in the Trash from cleanups".
    pub waiting: ByteSize,
    /// Why no new cleanup can be recorded now, when the journal is read-only; the history is still shown.
    pub blocked: Option<BlockedReason>,
}

/// One cleanup of the history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupSummary {
    pub cleanup: CleanupId,
    pub started: Timestamp,
    pub method: Method,
    pub ended: Ended,
    /// How many items it had.
    pub count: u32,
    pub counts: OutcomeCounts,
    pub moved: Sizes,
    pub deleted: Sizes,
    /// How many of its items are still in the Trash.
    pub in_trash: u32,
}

/// The answer of `get_cleanup`: one cleanup with every item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CleanupDetail {
    pub summary: CleanupSummary,
    pub items: Vec<CleanupItemView>,
}

/// One item of a past cleanup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItemView {
    /// Its place in the cleanup: what `put_back` and `reveal_in_trash` name it by.
    pub index: u32,
    /// Where it was, to show; `None` for a tool's item.
    pub path: Option<PathBuf>,
    pub category: String,
    pub danger: DangerLevel,
    pub sizes: Sizes,
    pub action: ActionKind,
    pub outcome: RecordedOutcome,
    /// Where it is now, for an item moved to the Trash.
    pub trash: Option<TrashState>,
    /// How the last attempt to put it back ended, if there was one.
    pub put_back: Option<PutBackOutcome>,
}

/// The input of `put_back`: items of one cleanup, or (`None`) every item of it still in the Trash.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PutBackRequest {
    cleanup: CleanupId,
    items: Option<Vec<u32>>,
}

impl PutBackRequest {
    pub const fn cleanup(&self) -> CleanupId {
        self.cleanup
    }

    pub fn items(&self) -> Option<&[u32]> {
        self.items.as_deref()
    }
}

/// One item of a past cleanup, as `reveal_in_trash` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CleanupItemRef {
    cleanup: CleanupId,
    item: u32,
}

impl CleanupItemRef {
    pub const fn cleanup(self) -> CleanupId {
        self.cleanup
    }

    pub const fn item(self) -> u32 {
        self.item
    }
}

/// The answer of `put_back`: how each item ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PutBackReport {
    pub items: Vec<PutBackItem>,
}

/// How putting one item back ended, with its text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PutBackItem {
    pub index: u32,
    pub outcome: PutBackOutcome,
    pub text: Text,
}

impl PutBackItem {
    pub fn new(index: u32, outcome: PutBackOutcome) -> Self {
        Self { index, outcome, text: outcome.text() }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    /// Every string of `json` that looks like an absolute path, with the key it is under.
    fn paths(json: &Value, key: &str, found: &mut Vec<(String, String)>) {
        match json {
            Value::String(text) if text.starts_with('/') => found.push((key.to_owned(), text.clone())),
            Value::Array(values) => values.iter().for_each(|value| paths(value, key, found)),
            Value::Object(fields) => fields.iter().for_each(|(name, value)| paths(value, name, found)),
            _ => {}
        }
    }

    mod negative {
        use super::paths;
        use crate::cleanup::plan::sample;
        use crate::cleanup::{
            ActionKind, CleanupDetail, CleanupId, CleanupItemView, CleanupPlanView, CleanupSummary, CleanupsQuery,
            Ended, LeftOut, Method, OutcomeCounts, PlanGroup, PlanItem, PlanItemView, PutBackRequest, RecordedOutcome,
            SkipReason, TrashState,
        };
        use crate::{DangerLevel, Sizes, Timestamp};

        #[test]
        fn no_view_holds_a_path_but_the_display_ones() {
            let plan = sample::plan();
            let left = LeftOut::new(&sample::trash(9, DangerLevel::Low, 1), SkipReason::Covered);
            let view = CleanupPlanView {
                plan: plan.id(),
                summary: plan.summary(),
                groups: vec![PlanGroup {
                    category: plan.items()[0].category(),
                    action: ActionKind::Remove,
                    summary: plan.summary(),
                    items: plan
                        .items()
                        .iter()
                        .map(PlanItemView::of)
                        .chain([PlanItemView::of(&PlanItem::new(&sample::together(4, DangerLevel::Low, 1)))])
                        .collect(),
                }],
                left_out: vec![left],
                permanent_available: true,
            };
            let summary = CleanupSummary {
                cleanup: CleanupId::new(1),
                started: Timestamp::from_secs(1),
                method: Method::Trash,
                ended: Ended::Completed,
                count: 1,
                counts: OutcomeCounts::default(),
                moved: Sizes::ZERO,
                deleted: Sizes::ZERO,
                in_trash: 1,
            };
            let detail = CleanupDetail {
                summary,
                items: vec![CleanupItemView {
                    index: 0,
                    path: Some("/home/cache-1".into()),
                    category: "demo.caches".into(),
                    danger: DangerLevel::Low,
                    sizes: Sizes::ZERO,
                    action: ActionKind::Remove,
                    outcome: RecordedOutcome::Moved,
                    trash: Some(TrashState::InTrash),
                    put_back: None,
                }],
            };

            let mut found = Vec::new();
            paths(&serde_json::to_value(view).unwrap(), "", &mut found);
            paths(&serde_json::to_value(detail).unwrap(), "", &mut found);

            assert!(found.iter().all(|(key, _)| key == "path" || key == "more"), "{found:?}");
            assert_eq!(
                found.len(),
                5,
                "the three path items, the together item's other part, the history's: {found:?}"
            );
        }

        #[test]
        fn a_history_query_over_the_limit_is_capped() {
            let query: CleanupsQuery = serde_json::from_str(r#"{ "offset": 0, "limit": 100000 }"#).unwrap();

            assert_eq!(query.limit(), CleanupsQuery::MAX_LIMIT);
        }

        #[test]
        fn a_put_back_request_holds_no_path() {
            let json = r#"{ "cleanup": 1, "items": [0], "path": "/" }"#;

            assert!(serde_json::from_str::<PutBackRequest>(json).is_err());
        }
    }

    mod positive {
        use std::path::PathBuf;

        use serde_json::json;

        use crate::cleanup::plan::sample;
        use crate::cleanup::{
            CleanupId, CleanupItemRef, CleanupsQuery, PlanItem, PlanItemView, PutBackItem, PutBackOutcome,
            PutBackRequest,
        };

        #[test]
        fn a_plan_item_shows_its_path_and_a_tool_item_none() {
            let trash = PlanItemView::of(&PlanItem::new(&sample::trash(1, crate::DangerLevel::Low, 1)));
            let tool = PlanItemView::of(&PlanItem::new(&sample::tool(2)));

            assert_eq!(trash.path, Some(PathBuf::from("/home/cache-1")));
            assert_eq!(tool.path, None);
            assert!(trash.more.is_empty() && tool.more.is_empty());
        }

        #[test]
        fn a_together_item_shows_its_subject_and_the_parts_after_it() {
            let view = PlanItemView::of(&PlanItem::new(&sample::together(3, crate::DangerLevel::Medium, 1)));

            assert_eq!(view.path, Some(PathBuf::from("/home/model-3")));
            assert_eq!(view.more, [PathBuf::from("/home/layer-3")]);
        }

        #[test]
        fn the_inputs_read_ids_only() {
            let all: PutBackRequest = serde_json::from_value(json!({ "cleanup": 3, "items": null })).unwrap();
            let some: PutBackRequest = serde_json::from_value(json!({ "cleanup": 3, "items": [1, 2] })).unwrap();
            let item: CleanupItemRef = serde_json::from_value(json!({ "cleanup": 3, "item": 1 })).unwrap();
            let query = CleanupsQuery::new(20, 50);

            assert_eq!((all.cleanup(), all.items()), (CleanupId::new(3), None));
            assert_eq!(some.items(), Some(&[1, 2][..]));
            assert_eq!((item.cleanup(), item.item()), (CleanupId::new(3), 1));
            assert_eq!((query.offset(), query.limit()), (20, 50));
        }

        #[test]
        fn exports_the_cleanup_types_to_typescript() {
            use ts_rs::{Config, TS};

            use crate::cleanup::{CleanupItemView, CleanupPlanView, CleanupSummary, Confirmation, ItemOutcome};

            let config = Config::new();
            let view = CleanupPlanView::decl(&config);
            let echo = Confirmation::decl(&config);

            assert!(view.contains("leftOut: Array<LeftOut>"), "{view}");
            assert!(view.contains("permanentAvailable: boolean"), "{view}");
            assert!(echo.contains("allocated: ByteSize") && echo.contains("method: Method"), "{echo}");
            assert!(ItemOutcome::decl(&config).contains("\"kind\": \"not-reached\""));
            assert!(CleanupSummary::decl(&config).contains("inTrash: number"));
            assert!(CleanupItemView::decl(&config).contains("putBack: PutBackOutcome | null"));
        }

        #[test]
        fn a_put_back_item_carries_its_text() {
            let item = PutBackItem::new(2, PutBackOutcome::Occupied);

            assert_eq!(item.text.key().as_str(), "core.cleanup.put-back.occupied");
            assert_eq!(
                serde_json::to_value(PutBackOutcome::Refused { reason: crate::cleanup::SkipCode::Protected }).unwrap(),
                json!({ "kind": "refused", "reason": "protected" })
            );
        }
    }
}
