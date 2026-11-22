//! Cleanups: the plan the user confirms, the confirmation that alone leads to a removal, what came of each item, and
//! what the cleanup commands answer.

mod confirm;
mod journal;
mod outcome;
mod plan;
mod port;
mod protection;
mod sandbox;
mod trash;
mod view;

pub use confirm::{Act, ConfirmError, Confirmation, ConfirmedMethod, ConfirmedPlan, EchoField, PermanentConsent};
pub use journal::{
    BeginRecord, CleanupJournal, EndRecord, ItemRecord, JournalError, JournalItem, JournalPage, JournalPart,
    JournalProblem, JournalState, PutBackRecord, RecordedCleanup, VolumeSpace,
};
pub use outcome::{
    BlockedReason, CleanupReport, Ended, Failure, ItemOutcome, ItemReport, OutcomeCounts, PartlyWay, PutBackOutcome,
    RecordedOutcome, SkipCode, SkipReason, TrashState,
};
pub use plan::{
    Action, ActionKind, CategoryPick, CleanupId, CleanupPlan, CleanupSelection, ItemIdentity, LeftOut, MAX_PLAN_ITEMS,
    Method, PLAN_LIFETIME, PlanError, PlanId, PlanItem, PlanSummary,
};
pub use port::{Cleaner, CleanupContext, Depth, ItemDone, Rehearsal, TrashedItem};
pub(crate) use protection::is_plain;
pub use protection::{Protected, Protection, ProtectionError};
pub use sandbox::{SANDBOX_MARKER, SANDBOX_SIGNATURE};
pub use trash::{TrashLine, TrashView};
pub use view::{
    CleanupDetail, CleanupItemRef, CleanupItemView, CleanupPlanView, CleanupSummary, CleanupsPage, CleanupsQuery,
    PlanGroup, PlanItemView, PutBackItem, PutBackReport, PutBackRequest,
};

#[cfg(test)]
pub(crate) use outcome::sample as outcome_sample;
