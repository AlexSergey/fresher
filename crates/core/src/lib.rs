//! The shared core of Fresher: domain types, the module contract, ports, events, commands, errors and text keys.
//!
//! The core does no IO. The ports to the outside world (the clock, the Trash, external commands, folders, the change
//! journal, volumes, the scan cache, the disk state) are traits here, so every other crate can be tested with the
//! fakes of `fresher-testing`.

#![forbid(unsafe_code)]

mod cachedir;
mod cancel;
mod category;
mod command;
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

pub use cachedir::{CACHEDIR_SIGNATURE, CACHEDIR_TAG, is_valid_cachedir_tag};
pub use cancel::CancelToken;
pub use category::{CategoryDescriptor, CategoryText};
pub use command::{
    CategoryTotal, EngineStatus, FindingOrder, FindingsPage, FindingsQuery, LastScan, Locale, ModuleInfo, Operation,
    Overview, SettingUpdate, SettingsFile, SettingsView,
};
pub use danger::DangerLevel;
pub use dictionary::{Dictionary, DictionaryError};
pub use disk_state::{
    ChildOrder, DiskState, NodeId, NodeInfo, NodeKind, NodePage, NotScanned, StateId, UnreadablePlace,
};
pub use error::{CommandError, Refusal};
pub use event::{Event, ModuleOutcome, ScanKind, ScanOutcome};
pub use finding::{Finding, NewFinding};
pub use folder::{FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError};
pub use hash::stable_hash;
pub use ids::{CategoryId, FindingId, FindingKey, IdError, ModuleId, ScanId, Timestamp, ToolId};
pub use interest::{Claim, EntryKind, Interest, InterestError, InterestSet, MarkerCheck, Match, Pattern};
pub use journal::{ChangeJournal, Changes, JournalGap, JournalPosition};
pub use module::{DescriptorError, FindingSink, Locales, Module, ModuleContext, ModuleDescriptor, ModuleError};
pub use ports::{
    Access, AccessProbe, AccessStatus, Clock, CommandOutput, CommandRunError, CommandRunner, Trash, TrashError,
};
pub use removal::{Removal, RemovalError, ToolCommand};
pub use scan_cache::{ScanCache, ScopeKey};
pub use settings::{
    ModuleSettings, ResolvedSettings, SchemaError, SettingDescriptor, SettingError, SettingKey, SettingKind,
    SettingProblem, SettingValue, SettingsSchema, Unit,
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
