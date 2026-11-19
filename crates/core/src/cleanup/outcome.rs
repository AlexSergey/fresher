//! Why an item was not removed, how each item of a cleanup ended, and the texts of both.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cleanup::Method;
use crate::{ByteSize, Capability, DETAIL_ARG, FindingId, Removability, Sizes, Text, TextArg, TextKey};

/// Why an item is left out of a plan or skipped by the cleaner: every check that can fail, each with its text.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SkipReason {
    /// A development build or a test removes only below a sandbox marker.
    NotInSandbox,
    /// Not strictly inside one of the plan's roots that is still a scan root.
    OutsideRoots,
    /// A protected place, inside one, or an ancestor of one.
    Protected,
    /// One of the user's own folders (or an ancestor of one), which never goes as a whole.
    WholeFolder,
    /// On the user's ignore list, inside an ignored path, or an ancestor of one.
    Ignored,
    /// A component of its path is a link.
    ThroughLink,
    /// It is no longer there.
    Vanished,
    /// On another volume than its root, or a folder with another volume inside.
    OtherVolume,
    /// Another file or folder is in its place now.
    Replaced,
    /// A file became a folder, or the other way round.
    KindChanged,
    /// The app cannot remove it now.
    NotRemovable { removability: Removability },
    /// The place or the tool needs a capability that is not allowed.
    MissingCapability { capability: Capability },
    /// A process of the user's holds it, or something inside it, open.
    InUse { app: String },
    /// A folder grew past the confirmed size by more than the margin.
    Grown,
    /// Its volume has no Trash; it is never deleted instead.
    NoTrash,
    /// Inside another item of the plan: it goes with that one.
    Covered,
    /// The removal names a path other than the finding's subject.
    NotTheSubject,
    /// The tool's arguments ask it to force the removal.
    ForcedTool,
    /// A development build on the real Mac: every check passed and nothing was done.
    DryRun,
    /// A check could not tell (an error other than "not found"): fail closed.
    CheckFailed,
}

/// A skip reason without its details: what the journal keeps and the history shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum SkipCode {
    NotInSandbox,
    OutsideRoots,
    Protected,
    WholeFolder,
    Ignored,
    ThroughLink,
    Vanished,
    OtherVolume,
    Replaced,
    KindChanged,
    NotRemovable,
    MissingCapability,
    InUse,
    Grown,
    NoTrash,
    Covered,
    NotTheSubject,
    ForcedTool,
    DryRun,
    CheckFailed,
}

impl SkipReason {
    pub const fn code(&self) -> SkipCode {
        match self {
            Self::NotInSandbox => SkipCode::NotInSandbox,
            Self::OutsideRoots => SkipCode::OutsideRoots,
            Self::Protected => SkipCode::Protected,
            Self::WholeFolder => SkipCode::WholeFolder,
            Self::Ignored => SkipCode::Ignored,
            Self::ThroughLink => SkipCode::ThroughLink,
            Self::Vanished => SkipCode::Vanished,
            Self::OtherVolume => SkipCode::OtherVolume,
            Self::Replaced => SkipCode::Replaced,
            Self::KindChanged => SkipCode::KindChanged,
            Self::NotRemovable { .. } => SkipCode::NotRemovable,
            Self::MissingCapability { .. } => SkipCode::MissingCapability,
            Self::InUse { .. } => SkipCode::InUse,
            Self::Grown => SkipCode::Grown,
            Self::NoTrash => SkipCode::NoTrash,
            Self::Covered => SkipCode::Covered,
            Self::NotTheSubject => SkipCode::NotTheSubject,
            Self::ForcedTool => SkipCode::ForcedTool,
            Self::DryRun => SkipCode::DryRun,
            Self::CheckFailed => SkipCode::CheckFailed,
        }
    }

    /// What happened: `core.cleanup.skip.<code>`.
    pub fn text(&self) -> Text {
        Text::new(self.code().text_key())
    }

    /// What the user can do about it, if anything: the removability's own instruction for `NotRemovable`, "Quit
    /// {app} and try again." for `InUse`, else `core.cleanup.skip.<code>.how`.
    pub fn how(&self) -> Option<Text> {
        match self {
            Self::NotRemovable { removability } => removability.how_key().map(Text::new),
            Self::InUse { app } => {
                self.code().how_key().map(|key| Text::new(key).with("app", TextArg::Text(app.clone())))
            }
            _ => self.code().how_key().map(Text::new),
        }
    }
}

impl SkipCode {
    pub const ALL: [Self; 20] = [
        Self::NotInSandbox,
        Self::OutsideRoots,
        Self::Protected,
        Self::WholeFolder,
        Self::Ignored,
        Self::ThroughLink,
        Self::Vanished,
        Self::OtherVolume,
        Self::Replaced,
        Self::KindChanged,
        Self::NotRemovable,
        Self::MissingCapability,
        Self::InUse,
        Self::Grown,
        Self::NoTrash,
        Self::Covered,
        Self::NotTheSubject,
        Self::ForcedTool,
        Self::DryRun,
        Self::CheckFailed,
    ];

    /// What happened: `core.cleanup.skip.<code>`, with no placeholder, so the history can show it from the code alone.
    pub fn text_key(self) -> TextKey {
        TextKey::from_valid(format!("core.cleanup.skip.{}", self.id()))
    }

    /// The general instruction, where there is one: `core.cleanup.skip.<code>.how`. `NotRemovable` has none of its own
    /// (the removability's is used); the one of `InUse` names the app (`{app}`).
    pub fn how_key(self) -> Option<TextKey> {
        let has_how = matches!(
            self,
            Self::Replaced
                | Self::KindChanged
                | Self::MissingCapability
                | Self::InUse
                | Self::Grown
                | Self::NoTrash
                | Self::CheckFailed
        );
        has_how.then(|| TextKey::from_valid(format!("core.cleanup.skip.{}.how", self.id())))
    }

    const fn id(self) -> &'static str {
        match self {
            Self::NotInSandbox => "not-in-sandbox",
            Self::OutsideRoots => "outside-roots",
            Self::Protected => "protected",
            Self::WholeFolder => "whole-folder",
            Self::Ignored => "ignored",
            Self::ThroughLink => "through-link",
            Self::Vanished => "vanished",
            Self::OtherVolume => "other-volume",
            Self::Replaced => "replaced",
            Self::KindChanged => "kind-changed",
            Self::NotRemovable => "not-removable",
            Self::MissingCapability => "missing-capability",
            Self::InUse => "in-use",
            Self::Grown => "grown",
            Self::NoTrash => "no-trash",
            Self::Covered => "covered",
            Self::NotTheSubject => "not-the-subject",
            Self::ForcedTool => "forced-tool",
            Self::DryRun => "dry-run",
            Self::CheckFailed => "check-failed",
        }
    }
}

/// Why the act on an item failed after its checks passed: the text of `ItemOutcome::Failed`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Failure {
    /// The system refused to move it to the Trash.
    TrashRefused,
    /// What landed in the Trash is not the item that was checked; it can be put back.
    LandedElsewhere,
    /// The system refused to delete it, or it is still there afterwards.
    DeleteFailed,
    /// The tool ran and failed; `detail` is the first line of what it wrote, shown only on demand.
    ToolFailed { detail: String },
    /// The tool could not be started.
    ToolMissing,
}

impl Failure {
    /// `core.cleanup.failed.<failure>`; the tool's line as the `DETAIL_ARG` arg, which the sentence never names.
    pub fn text(&self) -> Text {
        let key = |id: &str| Text::new(TextKey::from_valid(format!("core.cleanup.failed.{id}")));
        match self {
            Self::TrashRefused => key("trash-refused"),
            Self::LandedElsewhere => key("landed-elsewhere"),
            Self::DeleteFailed => key("delete-failed"),
            Self::ToolFailed { detail } => key("tool-failed").with(DETAIL_ARG, TextArg::Text(detail.clone())),
            Self::ToolMissing => key("tool-missing"),
        }
    }
}

/// How one item of a cleanup ended, as the events and the report carry it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ItemOutcome {
    /// Moved to the Trash.
    Moved,
    /// The folder's entries went (to the Trash, or deleted), `skipped` of them stayed; the folder stays.
    Emptied { moved: u32, skipped: u32 },
    /// Deleted permanently.
    Deleted,
    /// Its tool removed it.
    ToolDone,
    /// A check failed: nothing was done to it.
    Skipped { reason: SkipReason },
    /// The checks passed but the act failed; `reason` is a `Failure`'s text.
    Failed { reason: Text },
    /// The cleanup was cancelled or stopped before it.
    NotReached,
}

impl ItemOutcome {
    /// `core.cleanup.outcome.<kind>`; a skip's and a failure's reasons have texts of their own.
    pub fn text(&self) -> Text {
        outcome_text(match self {
            Self::Moved => "moved",
            Self::Emptied { .. } => "emptied",
            Self::Deleted => "deleted",
            Self::ToolDone => "tool-done",
            Self::Skipped { .. } => "skipped",
            Self::Failed { .. } => "failed",
            Self::NotReached => "not-reached",
        })
    }

    /// What the journal keeps of it: the skip reason's code, a tool's exit `status` for a failure.
    pub fn recorded(&self, status: Option<i32>) -> RecordedOutcome {
        match self {
            Self::Moved => RecordedOutcome::Moved,
            Self::Emptied { moved, skipped } => RecordedOutcome::Emptied { moved: *moved, skipped: *skipped },
            Self::Deleted => RecordedOutcome::Deleted,
            Self::ToolDone => RecordedOutcome::ToolDone,
            Self::Skipped { reason } => RecordedOutcome::Skipped { reason: reason.code() },
            Self::Failed { .. } => RecordedOutcome::Failed { status },
            Self::NotReached => RecordedOutcome::NotReached,
        }
    }
}

/// How an item ended, as the journal keeps it and the history shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RecordedOutcome {
    Moved,
    Emptied {
        moved: u32,
        skipped: u32,
    },
    Deleted,
    ToolDone,
    Skipped {
        reason: SkipCode,
    },
    /// The act failed; a tool's exit status when a tool failed.
    Failed {
        status: Option<i32>,
    },
    NotReached,
    /// No outcome was recorded: the app stopped while the cleanup was on it or before it. It may be in the Trash.
    Unknown,
}

impl RecordedOutcome {
    /// `core.cleanup.outcome.<kind>`.
    pub fn text(&self) -> Text {
        outcome_text(match self {
            Self::Moved => "moved",
            Self::Emptied { .. } => "emptied",
            Self::Deleted => "deleted",
            Self::ToolDone => "tool-done",
            Self::Skipped { .. } => "skipped",
            Self::Failed { .. } => "failed",
            Self::NotReached => "not-reached",
            Self::Unknown => "unknown",
        })
    }
}

fn outcome_text(kind: &str) -> Text {
    Text::new(TextKey::from_valid(format!("core.cleanup.outcome.{kind}")))
}

/// One item's outcome as the events carry it: its index in the plan and its finding.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct ItemReport {
    pub index: u32,
    pub finding: FindingId,
    pub outcome: ItemOutcome,
}

/// How a cleanup ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Ended {
    /// Every item was reached.
    Completed,
    /// The user stopped it; the item in flight finished.
    Cancelled,
    /// The journal could not be written: it stopped after the item in flight, so nothing goes unrecorded.
    Stopped,
    /// Only in the history: the journal has no end, the app stopped mid-way.
    Interrupted,
}

impl Ended {
    pub const ALL: [Self; 4] = [Self::Completed, Self::Cancelled, Self::Stopped, Self::Interrupted];

    /// `core.cleanup.ended.<ended>`.
    pub fn text(self) -> Text {
        let id = match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Stopped => "stopped",
            Self::Interrupted => "interrupted",
        };
        Text::new(TextKey::from_valid(format!("core.cleanup.ended.{id}")))
    }
}

/// How many items ended each way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeCounts {
    pub moved: u32,
    pub emptied: u32,
    pub deleted: u32,
    pub tool_done: u32,
    pub skipped: u32,
    pub failed: u32,
    pub not_reached: u32,
    /// Only in the history: items a crash left without an outcome.
    pub unknown: u32,
}

impl OutcomeCounts {
    /// Counts one more item that ended as `outcome`.
    pub const fn add(&mut self, outcome: &RecordedOutcome) {
        let count = match outcome {
            RecordedOutcome::Moved => &mut self.moved,
            RecordedOutcome::Emptied { .. } => &mut self.emptied,
            RecordedOutcome::Deleted => &mut self.deleted,
            RecordedOutcome::ToolDone => &mut self.tool_done,
            RecordedOutcome::Skipped { .. } => &mut self.skipped,
            RecordedOutcome::Failed { .. } => &mut self.failed,
            RecordedOutcome::NotReached => &mut self.not_reached,
            RecordedOutcome::Unknown => &mut self.unknown,
        };
        *count = count.saturating_add(1);
    }
}

/// What a cleanup did: how it ended, the items by outcome, the sizes moved to the Trash and deleted (estimates from
/// the plan), and the space measured on the volumes (`None` when a volume could not be read).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct CleanupReport {
    pub method: Method,
    pub ended: Ended,
    pub counts: OutcomeCounts,
    pub moved: Sizes,
    pub deleted: Sizes,
    pub measured: Option<ByteSize>,
}

impl CleanupReport {
    /// The report of a cleanup by `method` before any item: completed, nothing counted, nothing measured.
    pub const fn new(method: Method) -> Self {
        Self {
            method,
            ended: Ended::Completed,
            counts: OutcomeCounts {
                moved: 0,
                emptied: 0,
                deleted: 0,
                tool_done: 0,
                skipped: 0,
                failed: 0,
                not_reached: 0,
                unknown: 0,
            },
            moved: Sizes::ZERO,
            deleted: Sizes::ZERO,
            measured: None,
        }
    }

    /// Counts an item of `sizes` that ended as `outcome`.
    pub fn count(&mut self, outcome: &ItemOutcome, sizes: Sizes) {
        self.count_recorded(&outcome.recorded(None), sizes);
    }

    /// Counts an item of `sizes` as the journal recorded it: a moved item adds to `moved`, a deleted one or a tool's
    /// to `deleted`, an emptied folder to the one of the method.
    pub fn count_recorded(&mut self, outcome: &RecordedOutcome, sizes: Sizes) {
        self.counts.add(outcome);
        let total = match (outcome, self.method) {
            (RecordedOutcome::Moved, _) | (RecordedOutcome::Emptied { .. }, Method::Trash) => &mut self.moved,
            (RecordedOutcome::Deleted | RecordedOutcome::ToolDone, _)
            | (RecordedOutcome::Emptied { .. }, Method::Permanent) => &mut self.deleted,
            (
                RecordedOutcome::Skipped { .. }
                | RecordedOutcome::Failed { .. }
                | RecordedOutcome::NotReached
                | RecordedOutcome::Unknown,
                _,
            ) => return,
        };
        *total = saturating_add(*total, sizes);
    }
}

/// `a + b`, each number stopping at its largest value (marked partial then) instead of failing.
fn saturating_add(a: Sizes, b: Sizes) -> Sizes {
    a.checked_add(b).unwrap_or_else(|| Sizes {
        logical: ByteSize::new(a.logical.bytes().saturating_add(b.logical.bytes())),
        allocated: ByteSize::new(a.allocated.bytes().saturating_add(b.allocated.bytes())),
        reclaimable: ByteSize::new(a.reclaimable.bytes().saturating_add(b.reclaimable.bytes())),
        completeness: a.completeness.max(b.completeness).max(crate::Completeness::Partial),
    })
}

/// Where an item moved to the Trash is now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum TrashState {
    /// Still in the Trash, the same item: it can be put back.
    InTrash,
    /// Not in the Trash any more: the Trash was emptied, or the item was moved.
    Gone,
    /// Something else is at its place in the Trash.
    Replaced,
}

impl TrashState {
    pub const ALL: [Self; 3] = [Self::InTrash, Self::Gone, Self::Replaced];

    /// `core.cleanup.trash.<state>`.
    pub fn text(self) -> Text {
        let id = match self {
            Self::InTrash => "in-trash",
            Self::Gone => "gone",
            Self::Replaced => "replaced",
        };
        Text::new(TextKey::from_valid(format!("core.cleanup.trash.{id}")))
    }
}

/// How putting an item back ended. Put back never replaces anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PutBackOutcome {
    /// It is at its original place again.
    PutBack,
    /// It never went to the Trash: deleted, removed by its tool, or not removed.
    NotTrashed,
    /// It is no longer in the Trash.
    Gone,
    /// Something else is at its place in the Trash.
    Replaced,
    /// Something is at its original place now; the item stays in the Trash.
    Occupied,
    /// The folder it was in is gone.
    NoParent,
    /// Its original place fails a check of the guard today.
    Refused { reason: SkipCode },
    /// The system refused to move it.
    Failed,
}

impl PutBackOutcome {
    /// `core.cleanup.put-back.<outcome>`.
    pub fn text(self) -> Text {
        let id = match self {
            Self::PutBack => "put-back",
            Self::NotTrashed => "not-trashed",
            Self::Gone => "gone",
            Self::Replaced => "replaced",
            Self::Occupied => "occupied",
            Self::NoParent => "no-parent",
            Self::Refused { .. } => "refused",
            Self::Failed => "failed",
        };
        Text::new(TextKey::from_valid(format!("core.cleanup.put-back.{id}")))
    }
}

/// Why no cleanup can start: the whole command is refused before any item (fail closed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum BlockedReason {
    /// The settings file is not loaded, so the ignore list cannot be known.
    Settings,
    /// The cleanup journal cannot be written.
    Journal,
    /// The disk is too full to write even the journal's first record.
    DiskFull,
    /// The files other apps hold open could not be listed.
    OpenFiles,
}

impl BlockedReason {
    pub const ALL: [Self; 4] = [Self::Settings, Self::Journal, Self::DiskFull, Self::OpenFiles];

    /// `core.cleanup.blocked.<reason>`: what is wrong and what to do.
    pub fn text(self) -> Text {
        let id = match self {
            Self::Settings => "settings",
            Self::Journal => "journal",
            Self::DiskFull => "disk-full",
            Self::OpenFiles => "open-files",
        };
        Text::new(TextKey::from_valid(format!("core.cleanup.blocked.{id}")))
    }
}

#[cfg(test)]
pub(crate) mod sample {
    use crate::cleanup::{Failure, ItemOutcome, PutBackOutcome, RecordedOutcome, SkipCode, SkipReason};
    use crate::{Capability, Removability};

    /// One skip reason of every variant; the `match` has no wildcard, so a new variant fails to compile until it is
    /// here.
    pub(crate) fn skip_reasons() -> Vec<SkipReason> {
        let sample = |reason: SkipReason| match reason {
            SkipReason::NotInSandbox
            | SkipReason::OutsideRoots
            | SkipReason::Protected
            | SkipReason::WholeFolder
            | SkipReason::Ignored
            | SkipReason::ThroughLink
            | SkipReason::Vanished
            | SkipReason::OtherVolume
            | SkipReason::Replaced
            | SkipReason::KindChanged
            | SkipReason::NotRemovable { .. }
            | SkipReason::MissingCapability { .. }
            | SkipReason::InUse { .. }
            | SkipReason::Grown
            | SkipReason::NoTrash
            | SkipReason::Covered
            | SkipReason::NotTheSubject
            | SkipReason::ForcedTool
            | SkipReason::DryRun
            | SkipReason::CheckFailed => reason,
        };
        vec![
            sample(SkipReason::NotInSandbox),
            sample(SkipReason::OutsideRoots),
            sample(SkipReason::Protected),
            sample(SkipReason::WholeFolder),
            sample(SkipReason::Ignored),
            sample(SkipReason::ThroughLink),
            sample(SkipReason::Vanished),
            sample(SkipReason::OtherVolume),
            sample(SkipReason::Replaced),
            sample(SkipReason::KindChanged),
            sample(SkipReason::NotRemovable { removability: Removability::Locked }),
            sample(SkipReason::MissingCapability { capability: Capability::AllFiles }),
            sample(SkipReason::InUse { app: "Safari".into() }),
            sample(SkipReason::Grown),
            sample(SkipReason::NoTrash),
            sample(SkipReason::Covered),
            sample(SkipReason::NotTheSubject),
            sample(SkipReason::ForcedTool),
            sample(SkipReason::DryRun),
            sample(SkipReason::CheckFailed),
        ]
    }

    pub(crate) fn failures() -> Vec<Failure> {
        let sample = |failure: Failure| match failure {
            Failure::TrashRefused
            | Failure::LandedElsewhere
            | Failure::DeleteFailed
            | Failure::ToolFailed { .. }
            | Failure::ToolMissing => failure,
        };
        vec![
            sample(Failure::TrashRefused),
            sample(Failure::LandedElsewhere),
            sample(Failure::DeleteFailed),
            sample(Failure::ToolFailed { detail: "image is in use".into() }),
            sample(Failure::ToolMissing),
        ]
    }

    pub(crate) fn item_outcomes() -> Vec<ItemOutcome> {
        let sample = |outcome: ItemOutcome| match outcome {
            ItemOutcome::Moved
            | ItemOutcome::Emptied { .. }
            | ItemOutcome::Deleted
            | ItemOutcome::ToolDone
            | ItemOutcome::Skipped { .. }
            | ItemOutcome::Failed { .. }
            | ItemOutcome::NotReached => outcome,
        };
        vec![
            sample(ItemOutcome::Moved),
            sample(ItemOutcome::Emptied { moved: 3, skipped: 1 }),
            sample(ItemOutcome::Deleted),
            sample(ItemOutcome::ToolDone),
            sample(ItemOutcome::Skipped { reason: SkipReason::Grown }),
            sample(ItemOutcome::Failed { reason: Failure::TrashRefused.text() }),
            sample(ItemOutcome::NotReached),
        ]
    }

    pub(crate) fn recorded_outcomes() -> Vec<RecordedOutcome> {
        let sample = |outcome: RecordedOutcome| match outcome {
            RecordedOutcome::Moved
            | RecordedOutcome::Emptied { .. }
            | RecordedOutcome::Deleted
            | RecordedOutcome::ToolDone
            | RecordedOutcome::Skipped { .. }
            | RecordedOutcome::Failed { .. }
            | RecordedOutcome::NotReached
            | RecordedOutcome::Unknown => outcome,
        };
        vec![
            sample(RecordedOutcome::Moved),
            sample(RecordedOutcome::Emptied { moved: 3, skipped: 1 }),
            sample(RecordedOutcome::Deleted),
            sample(RecordedOutcome::ToolDone),
            sample(RecordedOutcome::Skipped { reason: SkipCode::InUse }),
            sample(RecordedOutcome::Failed { status: Some(1) }),
            sample(RecordedOutcome::NotReached),
            sample(RecordedOutcome::Unknown),
        ]
    }

    pub(crate) fn put_back_outcomes() -> Vec<PutBackOutcome> {
        let sample = |outcome: PutBackOutcome| match outcome {
            PutBackOutcome::PutBack
            | PutBackOutcome::NotTrashed
            | PutBackOutcome::Gone
            | PutBackOutcome::Replaced
            | PutBackOutcome::Occupied
            | PutBackOutcome::NoParent
            | PutBackOutcome::Refused { .. }
            | PutBackOutcome::Failed => outcome,
        };
        vec![
            sample(PutBackOutcome::PutBack),
            sample(PutBackOutcome::NotTrashed),
            sample(PutBackOutcome::Gone),
            sample(PutBackOutcome::Replaced),
            sample(PutBackOutcome::Occupied),
            sample(PutBackOutcome::NoParent),
            sample(PutBackOutcome::Refused { reason: SkipCode::OutsideRoots }),
            sample(PutBackOutcome::Failed),
        ]
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::cleanup::outcome::sample;
        use crate::cleanup::{CleanupReport, ItemOutcome, Method, SkipCode, SkipReason};
        use crate::{ByteSize, Completeness, Removability, Sizes, TextArg};

        #[test]
        fn every_skip_reason_has_its_code() {
            let codes: Vec<SkipCode> = sample::skip_reasons().iter().map(SkipReason::code).collect();

            assert_eq!(codes, SkipCode::ALL);
            for (reason, code) in sample::skip_reasons().iter().zip(SkipCode::ALL) {
                let tag = serde_json::to_value(reason).unwrap()["kind"].clone();
                assert_eq!(tag, serde_json::to_value(code).unwrap(), "{reason:?}");
            }
        }

        #[test]
        fn a_skip_and_a_failure_count_no_size() {
            let mut report = CleanupReport::new(Method::Trash);
            let sizes = Sizes::complete(ByteSize::new(1), ByteSize::new(4096));

            report.count(&ItemOutcome::Skipped { reason: SkipReason::Grown }, sizes);
            report.count(&ItemOutcome::NotReached, sizes);

            assert_eq!((report.moved, report.deleted), (Sizes::ZERO, Sizes::ZERO));
            assert_eq!((report.counts.skipped, report.counts.not_reached), (1, 1));
        }

        #[test]
        fn sizes_that_do_not_fit_stop_at_the_largest_and_are_partial() {
            let mut report = CleanupReport::new(Method::Permanent);
            let huge = Sizes::complete(ByteSize::new(u64::MAX), ByteSize::new(u64::MAX));

            report.count(&ItemOutcome::Deleted, huge);
            report.count(&ItemOutcome::Deleted, huge);

            assert_eq!(report.deleted.allocated, ByteSize::new(u64::MAX));
            assert_eq!(report.deleted.completeness, Completeness::Partial);
        }

        #[test]
        fn a_not_removable_skip_tells_what_stands_in_the_way() {
            let locked = SkipReason::NotRemovable { removability: Removability::Locked };
            let removable = SkipReason::NotRemovable { removability: Removability::Removable };

            assert_eq!(locked.how().unwrap().key(), &Removability::Locked.how_key().unwrap());
            assert_eq!(removable.how(), None);
            assert_eq!(SkipReason::Covered.how(), None);
            let in_use = SkipReason::InUse { app: "Safari".into() }.how().unwrap();
            assert_eq!(in_use.args().get("app"), Some(&TextArg::Text("Safari".into())));
        }
    }

    mod positive {
        use serde_json::json;

        use crate::cleanup::Failure;
        use crate::cleanup::outcome::sample;
        use crate::cleanup::{
            CleanupReport, Ended, ItemOutcome, ItemReport, Method, RecordedOutcome, SkipCode, SkipReason,
        };
        use crate::{ByteSize, Capability, DETAIL_ARG, FindingId, Sizes, TextArg};

        #[test]
        fn a_tools_failure_carries_its_line_as_the_detail() {
            let text = Failure::ToolFailed { detail: "Error: image is in use".into() }.text();

            assert_eq!(text.key().as_str(), "core.cleanup.failed.tool-failed");
            assert_eq!(text.args().len(), 1);
            assert_eq!(text.args().get(DETAIL_ARG), Some(&TextArg::Text("Error: image is in use".into())));
        }

        #[test]
        fn outcomes_serialize_with_kebab_case_tags() {
            let report = ItemReport {
                index: 2,
                finding: FindingId::new(9),
                outcome: ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Safari".into() } },
            };

            assert_eq!(
                serde_json::to_value(report).unwrap(),
                json!({
                    "index": 2,
                    "finding": 9,
                    "outcome": { "kind": "skipped", "reason": { "kind": "in-use", "app": "Safari" } },
                })
            );
            assert_eq!(
                serde_json::to_value(SkipReason::MissingCapability { capability: Capability::AllFiles }).unwrap(),
                json!({ "kind": "missing-capability", "capability": { "kind": "all-files" } })
            );
            assert_eq!(serde_json::to_value(ItemOutcome::ToolDone).unwrap(), json!({ "kind": "tool-done" }));
            assert_eq!(
                serde_json::to_value(RecordedOutcome::Skipped { reason: SkipCode::NoTrash }).unwrap(),
                json!({ "kind": "skipped", "reason": "no-trash" })
            );
            assert_eq!(serde_json::from_value::<SkipCode>(json!("check-failed")).unwrap(), SkipCode::CheckFailed);
            assert_eq!(serde_json::to_value(Ended::Interrupted).unwrap(), "interrupted");
        }

        #[test]
        fn the_report_counts_by_outcome() {
            let sizes = |bytes| Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes));
            let mut trash = CleanupReport::new(Method::Trash);
            let mut permanent = CleanupReport::new(Method::Permanent);

            for (outcome, bytes) in sample::item_outcomes().iter().zip([1, 2, 4, 8, 16, 32, 64]) {
                trash.count(outcome, sizes(bytes));
                permanent.count(outcome, sizes(bytes));
            }

            assert_eq!(trash.moved, sizes(1 + 2));
            assert_eq!(trash.deleted, sizes(4 + 8));
            assert_eq!(permanent.moved, sizes(1));
            assert_eq!(permanent.deleted, sizes(2 + 4 + 8));
            let counts = trash.counts;
            assert_eq!(
                [counts.moved, counts.emptied, counts.deleted, counts.tool_done, counts.skipped, counts.failed],
                [1; 6]
            );
            assert_eq!(counts.not_reached, 1);
            assert_eq!(serde_json::to_value(counts).unwrap()["toolDone"], 1);
            assert_eq!(trash.ended, Ended::Completed);
            assert_eq!(trash.measured, None);
        }

        #[test]
        fn the_journal_keeps_the_code_and_the_status() {
            let recorded: Vec<RecordedOutcome> =
                sample::item_outcomes().iter().map(|outcome| outcome.recorded(Some(125))).collect();

            assert_eq!(
                recorded,
                [
                    RecordedOutcome::Moved,
                    RecordedOutcome::Emptied { moved: 3, skipped: 1 },
                    RecordedOutcome::Deleted,
                    RecordedOutcome::ToolDone,
                    RecordedOutcome::Skipped { reason: SkipCode::Grown },
                    RecordedOutcome::Failed { status: Some(125) },
                    RecordedOutcome::NotReached,
                ]
            );
        }

        #[test]
        fn the_dry_run_skip_has_its_code_and_text_and_no_instruction() {
            let reason = SkipReason::DryRun;

            assert_eq!(reason.code(), SkipCode::DryRun);
            assert_eq!(reason.text().key().as_str(), "core.cleanup.skip.dry-run");
            assert_eq!(reason.how(), None);
            assert_eq!(serde_json::to_value(&reason).unwrap(), json!({ "kind": "dry-run" }));
        }

        #[test]
        fn texts_follow_the_kind() {
            assert_eq!(ItemOutcome::Moved.text().key().as_str(), "core.cleanup.outcome.moved");
            assert_eq!(RecordedOutcome::Unknown.text().key().as_str(), "core.cleanup.outcome.unknown");
            assert_eq!(SkipReason::Grown.text().key().as_str(), "core.cleanup.skip.grown");
            assert_eq!(SkipCode::NoTrash.how_key().unwrap().as_str(), "core.cleanup.skip.no-trash.how");
        }
    }
}
