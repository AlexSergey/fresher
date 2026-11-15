//! The shared core of Fresher: domain types, the module contract, ports, events, commands, errors and text keys.
//!
//! The core does no IO. The ports to the outside world (the clock, the Trash, external commands) are traits here, so
//! every other crate can be tested with the fakes of `fresher-testing`.

mod cancel;
mod category;
mod command;
mod danger;
mod dictionary;
mod error;
mod event;
mod finding;
mod ids;
mod interest;
mod module;
mod ports;
mod removal;
mod settings;
mod size;
mod store;
mod subject;
mod text;
mod texts;
mod walk;

pub use cancel::CancelToken;
pub use category::{CategoryDescriptor, CategoryText};
pub use command::{
    CategoryTotal, EngineStatus, FindingsPage, FindingsQuery, LastScan, Locale, ModuleInfo, Operation, Overview,
    SettingUpdate, SettingsFile, SettingsView,
};
pub use danger::DangerLevel;
pub use dictionary::{Dictionary, DictionaryError};
pub use error::{CommandError, Refusal};
pub use event::{Event, ModuleOutcome, ScanOutcome};
pub use finding::{Finding, NewFinding};
pub use ids::{CategoryId, FindingId, FindingKey, IdError, ModuleId, ScanId, Timestamp, ToolId};
pub use interest::{Claim, EntryKind, Interest, InterestError, InterestSet, Match, Pattern};
pub use module::{DescriptorError, FindingSink, Locales, Module, ModuleContext, ModuleDescriptor, ModuleError};
pub use ports::{
    Access, AccessProbe, AccessStatus, Clock, CommandOutput, CommandRunError, CommandRunner, Trash, TrashError,
};
pub use removal::{Removal, RemovalError, ToolCommand};
pub use settings::{
    ModuleSettings, ResolvedSettings, SchemaError, SettingDescriptor, SettingError, SettingKey, SettingKind,
    SettingProblem, SettingValue, SettingsSchema, Unit,
};
pub use size::{ByteSize, Completeness, Sizes};
pub use store::{FormatVersion, SettingsStore, StoreError, StoredSettings, VersionPolicy};
pub use subject::Subject;
pub use text::{Text, TextArg, TextKey, TextKeyError};
pub use texts::core_dictionary;
pub use walk::{DiskWalk, Unreadable, WalkError, WalkProgress, WalkSummary};
