//! The plan as the confirmation shows it.

use fresher_core::{ActionKind, CleanupPlan, CleanupPlanView, PlanError, PlanGroup, PlanItemView, PlanSummary};

use crate::error::EngineError;

/// The view of `plan`: its items in groups of one category that go the same way, in plan order, each group with its
/// summary; the left-out list; the plan's own summary. The permanent choice applies when an item is not a tool's.
pub(crate) fn of(plan: &CleanupPlan) -> Result<CleanupPlanView, EngineError> {
    let same_group = |a: &fresher_core::PlanItem, b: &fresher_core::PlanItem| {
        a.category() == b.category() && a.action().kind() == b.action().kind()
    };
    let groups = plan
        .items()
        .chunk_by(same_group)
        .filter_map(|items| items.first().map(|first| (first, items)))
        .map(|(first, items)| {
            Ok(PlanGroup {
                category: first.category(),
                action: first.action().kind(),
                summary: PlanSummary::of(items).ok_or(EngineError::Plan(PlanError::Overflow))?,
                items: items.iter().map(PlanItemView::of).collect(),
            })
        })
        .collect::<Result<Vec<_>, EngineError>>()?;
    Ok(CleanupPlanView {
        plan: plan.id(),
        summary: plan.summary(),
        groups,
        left_out: plan.left_out().to_vec(),
        permanent_available: plan.items().iter().any(|item| item.action().kind() != ActionKind::Tool),
    })
}
