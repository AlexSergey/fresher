use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The wall clock, injected so that tests never read the real time.
pub trait Clock {
    fn now(&self) -> SystemTime;
}

/// The macOS Trash: the only way the cleaner removes a file by default.
pub trait Trash {
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
pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CommandError>;
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CommandError {
    #[error("{program} is not installed")]
    NotInstalled { program: String },
    #[error("{program} could not be started: {reason}")]
    Failed { program: String, reason: String },
}
