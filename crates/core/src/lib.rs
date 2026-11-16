//! The shared core of Fresher: domain types, the module contract, ports, events, commands, errors and text keys.
//!
//! The core does no IO. The ports to the outside world (the clock, the Trash, external commands, open files, folders,
//! the change journal, volumes, the scan cache, the disk state, the access probe, the system's interface, the cleaner
//! and the cleanup journal) are traits here, so every other crate can be tested with the fakes of `fresher-testing`.

#![forbid(unsafe_code)]

mod access;
mod cachedir;
mod cancel;
mod category;
mod cleanup;
mod command;
mod coverage;
mod danger;
mod dictionary;
mod disk_state;
mod error;
mod event;
mod finding;
mod folder;
mod hash;
mod ids;
mod interest;
mod journal;
mod module;
mod ports;
mod removability;
mod removal;
mod scan_cache;
mod settings;
mod size;
mod store;
mod subject;
mod text;
mod texts;
mod volume;
mod walk;

pub use access::{
    Access, AccessStatus, Availability, Capability, CapabilityState, CapabilityText, GuardedFolder, GuardedPlace, Need,
    NeedKind, VolumeKind,
};
pub use cachedir::{CACHEDIR_SIGNATURE, CACHEDIR_TAG, is_valid_cachedir_tag};
pub use cancel::CancelToken;
pub use category::{CategoryDescriptor, CategoryText};
pub use cleanup::{
    Act, Action, ActionKind, BeginRecord, BlockedReason, CategoryPick, Cleaner, CleanupContext, CleanupDetail,
    CleanupId, CleanupItemRef, CleanupItemView, CleanupJournal, CleanupPlan, CleanupPlanView, CleanupReport,
    CleanupSelection, CleanupSummary, CleanupsPage, CleanupsQuery, ConfirmError, Confirmation, ConfirmedMethod,
    ConfirmedPlan, Depth, EchoField, EndRecord, Ended, Failure, ItemDone, ItemIdentity, ItemOutcome, ItemRecord,
    ItemReport, JournalError, JournalItem, JournalPage, JournalProblem, JournalState, LeftOut, MAX_PLAN_ITEMS, Method,
    OutcomeCounts, PLAN_LIFETIME, PermanentConsent, PlanError, PlanGroup, PlanId, PlanItem, PlanItemView, PlanSummary,
    Protected, Protection, ProtectionError, PutBackItem, PutBackOutcome, PutBackRecord, PutBackReport, PutBackRequest,
    RecordedCleanup, RecordedOutcome, Rehearsal, SANDBOX_MARKER, SANDBOX_SIGNATURE, SkipCode, SkipReason, TrashState,
    TrashedItem, VolumeSpace,
};
pub use command::{
    CategoryInfo, CategoryTotal, EngineStatus, FindingOrder, FindingsPage, FindingsQuery, IgnoreTarget, IgnoredCount,
    IgnoredEntry, LastScan, Locale, ModuleInfo, Operation, Overview, OwnData, SettingUpdate, SettingsFile,
    SettingsView,
};
pub use coverage::{Coverage, NotEntered, UnreadableCount};
pub use danger::DangerLevel;
pub use dictionary::{Dictionary, DictionaryError};
pub use disk_state::{
    ChildOrder, DiskState, NodeId, NodeInfo, NodeKind, NodePage, NotScanned, StateId, UnreadablePlace,
};
pub use error::{CommandError, Refusal};
pub use event::{Event, ModuleOutcome, ScanKind, ScanOutcome};
pub use finding::{Finding, NewFinding};
pub use folder::{EntryAccess, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError};
pub use hash::stable_hash;
pub use ids::{CategoryId, FindingId, FindingKey, IdError, ModuleId, ScanId, Timestamp, ToolId};
pub use interest::{Claim, EntryKind, Interest, InterestError, InterestSet, MarkerCheck, Match, Pattern};
pub use journal::{ChangeJournal, Changes, JournalGap, JournalPosition};
pub use module::{DescriptorError, FindingSink, Locales, Module, ModuleContext, ModuleDescriptor, ModuleError};
pub use ports::{
    AccessProbe, Clock, CommandOutput, CommandRunError, CommandRunner, Landed, OpenFiles, OpenFilesError, OpenSet,
    SystemUi, SystemUiError, Trash, TrashError,
};
pub use removability::{EntryFacts, Removability};
pub use removal::{Removal, RemovalError, ToolCommand};
pub use scan_cache::{ScanCache, ScopeKey};
pub use settings::{
    IGNORE_CATEGORIES, IGNORE_PATHS, ModuleSettings, ResolvedSettings, SCAN_ROOTS, SchemaError, SettingDescriptor,
    SettingEffect, SettingError, SettingGroup, SettingKey, SettingKind, SettingProblem, SettingValue, SettingsSchema,
    Unit,
};
pub use size::{ByteSize, Completeness, Sizes};
pub use store::{FormatVersion, SettingsStore, StoreError, StoredSettings, VersionPolicy};
pub use subject::Subject;
pub use text::{Text, TextArg, TextKey, TextKeyError};
pub use texts::core_dictionary;
pub use volume::{VolumeFacts, VolumeId, Volumes};
pub use walk::{
    ClaimedPath, DiskWalk, FullScanReason, Unreadable, WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
};
