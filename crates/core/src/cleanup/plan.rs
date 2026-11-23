//! The plan of a cleanup: what the user picked, each item as the core checked it, what was left out and why, and the
//! summary the user confirms.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cleanup::SkipReason;
use crate::{
    CategoryId, DangerLevel, EntryKind, Finding, FindingId, FindingKey, Parts, Removal, ScanId, Sizes, Text, Timestamp,
    ToolCommand,
};

/// The most items one plan holds: an exact list longer than this cannot be reviewed.
pub const MAX_PLAN_ITEMS: u32 = 10_000;

/// How long an unconfirmed plan lives; a confirmation of an older one is refused.
pub const PLAN_LIFETIME: Duration = Duration::from_mins(10);

/// The id of a plan in one session, assigned by the engine; one plan is pending at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct PlanId(u32);

impl PlanId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The id of a cleanup, stable across sessions: its sequence number in the cleanup journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct CleanupId(u32);

impl CleanupId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for CleanupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// What the user picked for removal, as the UI sends it: ids only, never a path.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CleanupSelection {
    findings: Vec<FindingId>,
    categories: Vec<CategoryPick>,
}

impl CleanupSelection {
    /// Findings picked one by one.
    pub fn findings(&self) -> &[FindingId] {
        &self.findings
    }

    /// Whole categories, each minus the findings ticked off in it.
    pub fn categories(&self) -> &[CategoryPick] {
        &self.categories
    }
}

/// A whole category picked for removal, minus `except`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CategoryPick {
    category: String,
    except: Vec<FindingId>,
}

impl CategoryPick {
    /// The category's id as the UI sent it; the engine checks it names a category.
    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn except(&self) -> &[FindingId] {
        &self.except
    }
}

/// How the items of a plan leave their place, chosen once for the whole plan in the confirmation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    /// To the Trash of their volume: they can be put back until the Trash is emptied.
    Trash,
    /// Deleted at once: the space comes back now, and the items cannot be put back.
    Permanent,
}

/// What an item was on the disk when the plan was made: checked again right before it is removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemIdentity {
    device: u64,
    file: u64,
    kind: EntryKind,
}

impl ItemIdentity {
    /// The identity of an entry: its device (`st_dev`), its file id (`st_ino`) and its kind.
    pub const fn new(device: u64, file: u64, kind: EntryKind) -> Self {
        Self { device, file, kind }
    }

    pub const fn device(self) -> u64 {
        self.device
    }

    pub const fn file(self) -> u64 {
        self.file
    }

    pub const fn kind(self) -> EntryKind {
        self.kind
    }
}

/// What the cleaner does to an item, taken from the finding's `Removal`; the plan's method decides Trash or delete.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// The file or folder leaves its place (`Removal::MoveToTrash`).
    Remove(PathBuf),
    /// The folder's contents leave, each entry on its own; the folder stays (`Removal::EmptyFolder`).
    Empty(PathBuf),
    /// The tool removes it, whatever the method; it can never be put back (`Removal::ToolCommand`).
    Tool(ToolCommand),
    /// Several files or folders leave together, the subject first, each the way a removed item goes; nothing of it
    /// leaves unless every part can (`Removal::MoveTogether`).
    RemoveTogether(Parts),
}

/// The kind of an action, without what it acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    Remove,
    Empty,
    Tool,
}

impl Action {
    /// The action a finding's removal asks for.
    pub fn from_removal(removal: &Removal) -> Self {
        match removal {
            Removal::MoveToTrash(path) => Self::Remove(path.clone()),
            Removal::EmptyFolder(path) => Self::Empty(path.clone()),
            Removal::ToolCommand(command) => Self::Tool(command.clone()),
            Removal::MoveTogether(parts) => Self::RemoveTogether(parts.clone()),
        }
    }

    pub const fn kind(&self) -> ActionKind {
        match self {
            Self::Remove(_) | Self::RemoveTogether(_) => ActionKind::Remove,
            Self::Empty(_) => ActionKind::Empty,
            Self::Tool(_) => ActionKind::Tool,
        }
    }

    /// The path it acts on, the subject of an item of several paths; `None` for a tool's.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Remove(path) | Self::Empty(path) => Some(path),
            Self::RemoveTogether(parts) => Some(parts.subject()),
            Self::Tool(_) => None,
        }
    }

    /// Every path it acts on, in order: its one path, every part of an item of several paths, none for a tool's.
    pub fn parts(&self) -> &[PathBuf] {
        match self {
            Self::Remove(path) | Self::Empty(path) => std::slice::from_ref(path),
            Self::RemoveTogether(parts) => parts.all(),
            Self::Tool(_) => &[],
        }
    }
}

/// One item of a plan: a finding of the current scan, as the user will see it, and what will be done to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanItem {
    finding: FindingId,
    key: FindingKey,
    category: CategoryId,
    danger: DangerLevel,
    sizes: Sizes,
    title: Text,
    reason: Text,
    action: Action,
    identity: Option<ItemIdentity>,
    part_identities: Vec<ItemIdentity>,
}

impl PlanItem {
    /// The item of `finding`, its action from the finding's removal, with no identity read yet.
    pub fn new(finding: &Finding) -> Self {
        let inner = finding.finding();
        Self {
            finding: finding.id(),
            key: inner.key().clone(),
            category: inner.category(),
            danger: inner.danger(),
            sizes: inner.sizes(),
            title: inner.title().clone(),
            reason: inner.reason().clone(),
            action: Action::from_removal(inner.removal()),
            identity: None,
            part_identities: Vec::new(),
        }
    }

    /// The item with the identity the rehearsal read; `None` for a tool's item.
    #[must_use]
    pub const fn with_identity(mut self, identity: Option<ItemIdentity>) -> Self {
        self.identity = identity;
        self
    }

    /// The item with the identity of every part the rehearsal read, in part order, for an item of several paths; its
    /// `identity()` is the first's, the subject's.
    #[must_use]
    pub fn with_part_identities(mut self, identities: Vec<ItemIdentity>) -> Self {
        self.identity = identities.first().copied();
        self.part_identities = identities;
        self
    }

    pub const fn finding(&self) -> FindingId {
        self.finding
    }

    pub const fn key(&self) -> &FindingKey {
        &self.key
    }

    pub const fn category(&self) -> CategoryId {
        self.category
    }

    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }

    pub const fn sizes(&self) -> Sizes {
        self.sizes
    }

    pub const fn title(&self) -> &Text {
        &self.title
    }

    pub const fn reason(&self) -> &Text {
        &self.reason
    }

    pub const fn action(&self) -> &Action {
        &self.action
    }

    pub const fn identity(&self) -> Option<ItemIdentity> {
        self.identity
    }

    /// The identity of every part, in part order, of an item of several paths; empty for every other item.
    pub fn part_identities(&self) -> &[ItemIdentity] {
        &self.part_identities
    }
}

/// A finding of the selection that will not be removed, and why; shown under "These will not be removed".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct LeftOut {
    finding: FindingId,
    title: Text,
    reason: SkipReason,
}

impl LeftOut {
    pub fn new(finding: &Finding, reason: SkipReason) -> Self {
        Self { finding: finding.id(), title: finding.finding().title().clone(), reason }
    }

    pub const fn finding(&self) -> FindingId {
        self.finding
    }

    pub const fn title(&self) -> &Text {
        &self.title
    }

    pub const fn reason(&self) -> &SkipReason {
        &self.reason
    }
}

/// What the user confirms: how many items, their sizes, the highest danger level among them, and how many a tool
/// removes (never put back).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct PlanSummary {
    pub count: u32,
    pub sizes: Sizes,
    pub highest: DangerLevel,
    pub irreversible: u32,
}

impl PlanSummary {
    /// The summary of `items`: their count, their sizes summed (the weakest completeness wins), the highest level
    /// (`Safe` for none) and the number of tool items; `None` when the count or a sum does not fit.
    pub fn of<'a>(items: impl IntoIterator<Item = &'a PlanItem>) -> Option<Self> {
        let mut summary = Self { count: 0, sizes: Sizes::ZERO, highest: DangerLevel::Safe, irreversible: 0 };
        for item in items {
            summary.count = summary.count.checked_add(1)?;
            summary.sizes = summary.sizes.checked_add(item.sizes)?;
            summary.highest = summary.highest.max(item.danger);
            if item.action.kind() == ActionKind::Tool {
                summary.irreversible = summary.irreversible.checked_add(1)?;
            }
        }
        Some(summary)
    }
}

/// Why a plan could not be made.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("a plan holds at most {limit} items")]
    TooManyItems { limit: u32 },
    #[error("finding {finding} is in the plan twice")]
    Duplicate { finding: FindingId },
    #[error("the sizes of the plan's items do not fit one number")]
    Overflow,
}

/// A frozen plan, waiting for the user's confirmation. It is never deserialized and never cloned: the only way to a
/// removal is [`CleanupPlan::confirm`], which consumes it.
#[derive(Debug, PartialEq, Eq)]
pub struct CleanupPlan {
    id: PlanId,
    made: Timestamp,
    scan: ScanId,
    roots: Vec<PathBuf>,
    items: Vec<PlanItem>,
    left_out: Vec<LeftOut>,
    summary: PlanSummary,
}

impl CleanupPlan {
    /// A plan of `items`, made at `made` from the findings of `scan` under `roots`. The summary is computed from the
    /// items, so no caller can give one that disagrees with them.
    pub fn new(
        id: PlanId,
        made: Timestamp,
        scan: ScanId,
        roots: Vec<PathBuf>,
        items: Vec<PlanItem>,
        left_out: Vec<LeftOut>,
    ) -> Result<Self, PlanError> {
        if items.len() > MAX_PLAN_ITEMS as usize {
            return Err(PlanError::TooManyItems { limit: MAX_PLAN_ITEMS });
        }
        let mut seen = HashSet::with_capacity(items.len());
        if let Some(twice) = items.iter().find(|item| !seen.insert(item.finding)) {
            return Err(PlanError::Duplicate { finding: twice.finding });
        }
        let summary = PlanSummary::of(&items).ok_or(PlanError::Overflow)?;
        Ok(Self { id, made, scan, roots, items, left_out, summary })
    }

    pub const fn id(&self) -> PlanId {
        self.id
    }

    pub const fn made(&self) -> Timestamp {
        self.made
    }

    pub const fn scan(&self) -> ScanId {
        self.scan
    }

    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The items, in the order the confirmation shows them and the cleaner acts on them.
    pub fn items(&self) -> &[PlanItem] {
        &self.items
    }

    pub fn left_out(&self) -> &[LeftOut] {
        &self.left_out
    }

    pub const fn summary(&self) -> PlanSummary {
        self.summary
    }
}

#[cfg(test)]
pub(crate) mod sample {
    use std::path::PathBuf;

    use crate::cleanup::{CleanupPlan, PlanId, PlanItem};
    use crate::{
        ByteSize, CategoryDescriptor, CategoryId, DangerLevel, Finding, FindingId, ModuleId, NewFinding, Parts,
        Removability, Removal, ScanId, Sizes, Subject, Text, TextKey, Timestamp, ToolCommand, ToolId,
    };

    /// The time every sample plan is made at.
    pub(crate) const MADE: u64 = 1_000;

    pub(crate) fn finding(id: u32, danger: DangerLevel, allocated: u64, removal: Removal) -> Finding {
        let demo = ModuleId::new("demo").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(demo, "demo.caches").unwrap(), danger);
        let text = |key: &str| Text::new(TextKey::new(key).unwrap());
        let subject = match &removal {
            Removal::MoveToTrash(path) | Removal::EmptyFolder(path) => Subject::Path(path.clone()),
            Removal::MoveTogether(parts) => Subject::Path(parts.subject().to_path_buf()),
            Removal::ToolCommand(command) => Subject::External { tool: command.tool(), id: format!("{id}") },
        };
        let sizes = Sizes::complete(ByteSize::new(allocated / 2), ByteSize::new(allocated));
        let new = NewFinding::new(
            &category,
            format!("item-{id}"),
            subject,
            sizes,
            text("demo.finding.title"),
            text("demo.finding.reason"),
            removal,
        );
        Finding::new(FindingId::new(id), new, Removability::Removable)
    }

    pub(crate) fn trash(id: u32, danger: DangerLevel, allocated: u64) -> Finding {
        finding(id, danger, allocated, Removal::MoveToTrash(PathBuf::from(format!("/home/cache-{id}"))))
    }

    /// A finding of two parts: `/home/model-<id>` (the subject) and `/home/layer-<id>`.
    pub(crate) fn together(id: u32, danger: DangerLevel, allocated: u64) -> Finding {
        let paths = vec![PathBuf::from(format!("/home/model-{id}")), PathBuf::from(format!("/home/layer-{id}"))];
        finding(id, danger, allocated, Removal::MoveTogether(Parts::new(paths).unwrap()))
    }

    pub(crate) fn tool(id: u32) -> Finding {
        let docker = ToolId::new("docker").unwrap();
        let command = ToolCommand::new(docker, vec!["image".into(), "rm".into(), format!("{id}")]).unwrap();
        finding(id, DangerLevel::Low, 100, Removal::ToolCommand(command))
    }

    /// A plan of three items: 4 KiB at `Low`, 8 KiB at `Medium`, and a tool's item at `Low`.
    pub(crate) fn plan() -> CleanupPlan {
        let findings = [trash(1, DangerLevel::Low, 4096), trash(2, DangerLevel::Medium, 8192), tool(3)];
        let items = findings.iter().map(PlanItem::new).collect();
        CleanupPlan::new(
            PlanId::new(7),
            Timestamp::from_secs(MADE),
            ScanId::new(1),
            vec!["/home".into()],
            items,
            vec![],
        )
        .unwrap()
    }

    /// A plan of two items: 4 KiB at `Low` and 16 KiB at `High`.
    pub(crate) fn high_plan() -> CleanupPlan {
        let findings = [trash(1, DangerLevel::Low, 4096), trash(2, DangerLevel::High, 16_384)];
        let items = findings.iter().map(PlanItem::new).collect();
        CleanupPlan::new(
            PlanId::new(7),
            Timestamp::from_secs(MADE),
            ScanId::new(1),
            vec!["/home".into()],
            items,
            vec![],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::cleanup::plan::sample;
        use crate::cleanup::{CleanupPlan, CleanupSelection, MAX_PLAN_ITEMS, PlanError, PlanId, PlanItem};
        use crate::{ByteSize, DangerLevel, ScanId, Timestamp};

        fn plan_of(items: Vec<PlanItem>) -> Result<CleanupPlan, PlanError> {
            CleanupPlan::new(PlanId::new(1), Timestamp::from_secs(0), ScanId::new(1), vec![], items, vec![])
        }

        #[test]
        fn a_selection_with_an_unknown_field_is_refused() {
            let json = r#"{ "findings": [1], "categories": [], "paths": ["/"] }"#;
            let pick =
                r#"{ "findings": [], "categories": [{ "category": "demo.caches", "except": [], "all": true }] }"#;

            assert!(serde_json::from_str::<CleanupSelection>(json).is_err());
            assert!(serde_json::from_str::<CleanupSelection>(pick).is_err());
        }

        #[test]
        fn a_plan_over_the_limit_is_refused() {
            let item = PlanItem::new(&sample::trash(1, DangerLevel::Safe, 1));
            let items = vec![item; MAX_PLAN_ITEMS as usize + 1];

            assert_eq!(plan_of(items), Err(PlanError::TooManyItems { limit: MAX_PLAN_ITEMS }));
        }

        #[test]
        fn a_finding_twice_in_a_plan_is_refused() {
            let item = PlanItem::new(&sample::trash(4, DangerLevel::Safe, 1));

            assert_eq!(
                plan_of(vec![item.clone(), item]),
                Err(PlanError::Duplicate { finding: crate::FindingId::new(4) })
            );
        }

        #[test]
        fn sizes_that_do_not_fit_are_refused() {
            let huge = |id| PlanItem::new(&sample::trash(id, DangerLevel::Safe, u64::MAX));

            assert_eq!(plan_of(vec![huge(1), huge(2)]), Err(PlanError::Overflow));
            assert_eq!(ByteSize::new(u64::MAX).checked_add(ByteSize::new(1)), None);
        }

        #[test]
        fn a_tool_action_has_no_path() {
            let item = PlanItem::new(&sample::tool(1));

            assert_eq!(item.action().path(), None);
            assert_eq!(item.action().parts(), [] as [std::path::PathBuf; 0]);
            assert_eq!(item.identity(), None);
        }

        #[test]
        fn only_an_item_of_several_paths_keeps_part_identities() {
            let item = PlanItem::new(&sample::trash(1, DangerLevel::Low, 1))
                .with_identity(Some(crate::cleanup::ItemIdentity::new(1, 2, crate::EntryKind::File)));

            assert_eq!(item.part_identities(), []);
            assert_eq!(PlanItem::new(&sample::together(2, DangerLevel::Low, 1)).part_identities(), []);
        }
    }

    mod positive {
        use std::path::Path;

        use serde_json::json;

        use crate::cleanup::plan::sample;
        use crate::cleanup::{
            Action, ActionKind, CleanupId, CleanupSelection, ItemIdentity, LeftOut, Method, PlanId, PlanItem,
            PlanSummary, SkipReason,
        };
        use crate::{ByteSize, DangerLevel, EntryKind, FindingId, Removal, ScanId, Sizes, Timestamp, ToolCommand};

        #[test]
        fn the_summary_is_computed_from_the_items() {
            let plan = sample::plan();

            let summary = plan.summary();

            assert_eq!(summary.count, 3);
            assert_eq!(
                summary.sizes,
                Sizes::complete(ByteSize::new(2048 + 4096 + 50), ByteSize::new(4096 + 8192 + 100))
            );
            assert_eq!(summary.highest, DangerLevel::Medium);
            assert_eq!(summary.irreversible, 1);
            assert_eq!(PlanSummary::of(plan.items()), Some(summary));
        }

        #[test]
        fn an_empty_plan_sums_to_nothing_at_safe() {
            let summary = PlanSummary::of(&[]).unwrap();

            assert_eq!(
                summary,
                PlanSummary { count: 0, sizes: Sizes::ZERO, highest: DangerLevel::Safe, irreversible: 0 }
            );
        }

        #[test]
        fn a_plan_keeps_what_it_was_made_of() {
            let plan = sample::plan();

            assert_eq!(plan.id(), PlanId::new(7));
            assert_eq!(plan.made(), Timestamp::from_secs(sample::MADE));
            assert_eq!(plan.scan(), ScanId::new(1));
            assert_eq!(plan.roots(), [Path::new("/home")]);
            assert_eq!(plan.items().iter().map(PlanItem::finding).collect::<Vec<_>>(), [1, 2, 3].map(FindingId::new));
            assert_eq!(plan.left_out(), []);
        }

        #[test]
        fn an_item_takes_its_finding() {
            let finding = sample::trash(5, DangerLevel::High, 4096);
            let identity = ItemIdentity::new(16, 99, EntryKind::Folder);

            let item = PlanItem::new(&finding).with_identity(Some(identity));

            assert_eq!(item.finding(), FindingId::new(5));
            assert_eq!(item.key(), finding.finding().key());
            assert_eq!(item.category(), finding.finding().category());
            assert_eq!(item.danger(), DangerLevel::High);
            assert_eq!(item.sizes(), finding.finding().sizes());
            assert_eq!(item.title(), finding.finding().title());
            assert_eq!(item.reason(), finding.finding().reason());
            assert_eq!(item.action().path(), Some(Path::new("/home/cache-5")));
            assert_eq!(item.identity(), Some(identity));
            assert_eq!((identity.device(), identity.file(), identity.kind()), (16, 99, EntryKind::Folder));
        }

        #[test]
        fn an_action_follows_the_removal() {
            let docker = crate::ToolId::new("docker").unwrap();
            let command = ToolCommand::new(docker, vec!["image".into(), "rm".into(), "a".into()]).unwrap();

            let remove = Action::from_removal(&Removal::MoveToTrash("/home/a".into()));
            let empty = Action::from_removal(&Removal::EmptyFolder("/home/b".into()));
            let tool = Action::from_removal(&Removal::ToolCommand(command.clone()));

            assert_eq!(remove, Action::Remove("/home/a".into()));
            assert_eq!(empty, Action::Empty("/home/b".into()));
            assert_eq!(tool, Action::Tool(command));
            assert_eq!(
                [remove.kind(), empty.kind(), tool.kind()],
                [ActionKind::Remove, ActionKind::Empty, ActionKind::Tool]
            );
            assert_eq!(empty.path(), Some(Path::new("/home/b")));
            assert_eq!(remove.parts(), [Path::new("/home/a")]);
            assert_eq!(empty.parts(), [Path::new("/home/b")]);
        }

        #[test]
        fn a_together_action_removes_every_part_with_the_subject_first() {
            let finding = sample::together(4, DangerLevel::Medium, 4096);
            let identities = vec![ItemIdentity::new(1, 10, EntryKind::File), ItemIdentity::new(1, 11, EntryKind::File)];

            let item = PlanItem::new(&finding).with_part_identities(identities.clone());

            assert!(matches!(item.action(), Action::RemoveTogether(_)));
            assert_eq!(item.action().kind(), ActionKind::Remove);
            assert_eq!(item.action().path(), Some(Path::new("/home/model-4")));
            assert_eq!(item.action().parts(), [Path::new("/home/model-4"), Path::new("/home/layer-4")]);
            assert_eq!(item.identity(), Some(identities[0]));
            assert_eq!(item.part_identities(), identities);
            assert_eq!(PlanSummary::of([&item]).map(|summary| (summary.count, summary.irreversible)), Some((1, 0)));
        }

        #[test]
        fn ids_serialize_as_numbers() {
            assert_eq!(serde_json::to_value(PlanId::new(3)).unwrap(), json!(3));
            assert_eq!(serde_json::to_value(CleanupId::new(9)).unwrap(), json!(9));
            assert_eq!(serde_json::from_value::<CleanupId>(json!(9)).unwrap().get(), 9);
            assert_eq!(PlanId::new(3).get(), 3);
            assert_eq!(CleanupId::new(9).to_string(), "9");
            assert_eq!(serde_json::to_value(Method::Permanent).unwrap(), "permanent");
            assert_eq!(serde_json::to_value(ActionKind::Tool).unwrap(), "tool");
        }

        #[test]
        fn a_selection_reads_ids_and_categories() {
            let selection: CleanupSelection = serde_json::from_value(json!({
                "findings": [1, 2],
                "categories": [{ "category": "demo.caches", "except": [3] }],
            }))
            .unwrap();

            assert_eq!(selection.findings(), [FindingId::new(1), FindingId::new(2)]);
            assert_eq!(selection.categories()[0].category(), "demo.caches");
            assert_eq!(selection.categories()[0].except(), [FindingId::new(3)]);
        }

        #[test]
        fn a_left_out_item_names_its_finding_and_reason() {
            let finding = sample::trash(8, DangerLevel::Low, 1);

            let left = LeftOut::new(&finding, SkipReason::Covered);

            assert_eq!(left.finding(), FindingId::new(8));
            assert_eq!(left.title(), finding.finding().title());
            assert_eq!(left.reason(), &SkipReason::Covered);
            assert_eq!(serde_json::to_value(&left).unwrap()["reason"], json!({ "kind": "covered" }));
        }
    }
}
