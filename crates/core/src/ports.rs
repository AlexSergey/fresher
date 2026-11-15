use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;
use ts_rs::TS;

/// The wall clock, injected so that tests never read the real time.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

/// The macOS Trash: the only way the cleaner removes a file by default.
pub trait Trash: Send + Sync {
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError>;
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum TrashError {
    #[error("{path} does not exist")]
    NotFound { path: PathBuf },
    #[error("{path} could not be moved to the Trash: {reason}")]
    Refused { path: PathBuf, reason: String },
}

/// The result of an external program (the Docker CLI, `tmutil`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs external programs, injected so that tests use a fake instead of the real Docker CLI.
pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CommandRunError>;
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CommandRunError {
    #[error("{program} is not installed")]
    NotInstalled { program: String },
    #[error("{program} could not be started: {reason}")]
    Failed { program: String, reason: String },
}

/// Whether the app may read the folders macOS protects.
pub trait AccessProbe: Send + Sync {
    fn status(&self) -> AccessStatus;
}

/// What the app may read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AccessStatus {
    /// Full Disk Access: without it the protected folders of the home folder are unreadable.
    pub full_disk_access: Access,
}

/// One permission: granted, denied, or not known (the probe could not tell).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    Granted,
    Denied,
    Unknown,
}
