//! The record's types: what a run of a development build did, in the shape of the design's section 2.5.

use std::collections::BTreeMap;
use std::path::PathBuf;

use fresher_core::{
    Access, ActionKind, CapabilityState, CleanupId, Coverage, DangerLevel, Ended, Finding, FindingId, LeftOutCount,
    Method, ModuleGroup, ModuleId, ModuleOutcome, RecordedOutcome, ScanId, ScanKind, ScanOutcome, SettingKey, Sizes,
    SkipCode,
};
use serde::Serialize;

use super::BuildName;

/// The version of the record's shape, read by the checker.
pub(crate) const FORMAT: u32 = 1;

/// The whole record: the snapshot file holds it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Record {
    pub(crate) format: u32,
    pub(crate) session: Session,
    /// Every setting's value as the settings file stores it (plain, untagged).
    pub(crate) settings: BTreeMap<SettingKey, serde_json::Value>,
    pub(crate) access: Vec<CapabilityState>,
    pub(crate) actions: Vec<ActionRecord>,
    pub(crate) scans: Vec<ScanRecord>,
    pub(crate) clear: Vec<ClearRecord>,
}

/// Where and how the run happened.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Session {
    pub(crate) started: String,
    pub(crate) build: BuildName,
    pub(crate) mode: String,
    pub(crate) macos: Option<String>,
    pub(crate) home: PathBuf,
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) full_disk_access: Access,
}

/// A command the UI sent that changes something or reaches outside the app, with its arguments as sent.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionRecord {
    pub(crate) at: String,
    pub(crate) command: String,
    pub(crate) args: serde_json::Value,
}

/// One scan; the `Option`s are `null` while it runs.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanRecord {
    pub(crate) id: ScanId,
    pub(crate) kind: Option<ScanKind>,
    pub(crate) started: String,
    pub(crate) finished: Option<String>,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) outcome: Option<ScanOutcome>,
    pub(crate) coverage: Option<Coverage>,
    pub(crate) modules: BTreeMap<ModuleId, ModuleRecord>,
    /// The findings a cleanup removed from the results.
    pub(crate) removed: Vec<FindingId>,
}

/// One module's part of a scan.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleRecord {
    pub(crate) group: Option<ModuleGroup>,
    pub(crate) outcome: Option<ModuleOutcome>,
    pub(crate) findings: Vec<FindingRecord>,
    pub(crate) left_out: Vec<LeftOutRecord>,
}

/// A finding as the UI gets it, with its title and reason in English.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FindingRecord {
    #[serde(flatten)]
    pub(crate) finding: Finding,
    pub(crate) title_text: Option<String>,
    pub(crate) reason_text: Option<String>,
}

/// What a module recognized and did not offer, with the reason in English.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LeftOutRecord {
    #[serde(flatten)]
    pub(crate) count: LeftOutCount,
    pub(crate) reason_text: Option<String>,
}

/// One cleanup, its items by the kebab-case name of their module's group (`unknown` when no module claims the
/// category).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClearRecord {
    pub(crate) id: CleanupId,
    pub(crate) scan: Option<ScanId>,
    pub(crate) method: Method,
    pub(crate) dry_run: bool,
    pub(crate) ended: Option<Ended>,
    pub(crate) groups: BTreeMap<String, Vec<ClearItem>>,
}

/// One item of a cleanup and how it ended, or would have.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClearItem {
    pub(crate) module: String,
    pub(crate) category: String,
    pub(crate) danger: DangerLevel,
    pub(crate) path: Option<PathBuf>,
    pub(crate) sizes: Sizes,
    pub(crate) action: ActionKind,
    pub(crate) outcome: &'static str,
    pub(crate) reason: Option<SkipCode>,
}

/// The outcomes that mean the disk changed.
pub(crate) const ACTED: [&str; 4] = ["moved", "emptied", "deleted", "tool-done"];

/// How an item's `outcome` is written: a dry run's skip says what would have happened.
pub(crate) fn clear_outcome(
    outcome: &RecordedOutcome,
    action: ActionKind,
    method: Method,
) -> (&'static str, Option<SkipCode>) {
    match outcome {
        RecordedOutcome::Skipped { reason: SkipCode::DryRun } => {
            let would = match (action, method) {
                (ActionKind::Remove, Method::Trash) => "would-move",
                (ActionKind::Remove, Method::Permanent) => "would-delete",
                (ActionKind::Empty, _) => "would-empty",
                (ActionKind::Tool, _) => "would-run",
            };
            (would, None)
        }
        RecordedOutcome::Skipped { reason } => ("skipped", Some(*reason)),
        RecordedOutcome::Moved => ("moved", None),
        RecordedOutcome::Emptied { .. } => ("emptied", None),
        RecordedOutcome::Deleted => ("deleted", None),
        RecordedOutcome::ToolDone => ("tool-done", None),
        RecordedOutcome::Failed { .. } => ("failed", None),
        RecordedOutcome::NotReached => ("not-reached", None),
        RecordedOutcome::Unknown => ("unknown", None),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{ActionKind, Method, RecordedOutcome, SkipCode};

        use crate::dev_runs::record::clear_outcome;

        #[test]
        fn a_skip_other_than_the_dry_run_keeps_its_code() {
            let skipped = RecordedOutcome::Skipped { reason: SkipCode::InUse };

            assert_eq!(clear_outcome(&skipped, ActionKind::Remove, Method::Trash), ("skipped", Some(SkipCode::InUse)));
            assert_eq!(clear_outcome(&RecordedOutcome::Unknown, ActionKind::Tool, Method::Trash), ("unknown", None));
            assert_eq!(
                clear_outcome(&RecordedOutcome::Failed { status: Some(1) }, ActionKind::Tool, Method::Trash),
                ("failed", None)
            );
        }
    }

    mod positive {
        use fresher_core::{ActionKind, Method, RecordedOutcome, SkipCode};

        use crate::dev_runs::record::clear_outcome;

        #[test]
        fn a_dry_run_says_what_would_have_happened() {
            let dry = RecordedOutcome::Skipped { reason: SkipCode::DryRun };
            for (action, method, word) in [
                (ActionKind::Remove, Method::Trash, "would-move"),
                (ActionKind::Remove, Method::Permanent, "would-delete"),
                (ActionKind::Empty, Method::Permanent, "would-empty"),
                (ActionKind::Tool, Method::Trash, "would-run"),
            ] {
                assert_eq!(clear_outcome(&dry, action, method), (word, None), "{action:?} {method:?}");
            }
        }

        #[test]
        fn an_act_is_written_as_it_ended() {
            let emptied = RecordedOutcome::Emptied { moved: 1, skipped: 0 };

            assert_eq!(clear_outcome(&RecordedOutcome::Moved, ActionKind::Remove, Method::Trash), ("moved", None));
            assert_eq!(clear_outcome(&emptied, ActionKind::Empty, Method::Trash), ("emptied", None));
            assert_eq!(
                clear_outcome(&RecordedOutcome::Deleted, ActionKind::Remove, Method::Permanent),
                ("deleted", None)
            );
            assert_eq!(clear_outcome(&RecordedOutcome::ToolDone, ActionKind::Tool, Method::Trash), ("tool-done", None));
            assert_eq!(
                clear_outcome(&RecordedOutcome::NotReached, ActionKind::Remove, Method::Trash),
                ("not-reached", None)
            );
        }
    }
}
