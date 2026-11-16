//! How a finding would be removed: described by a module as data, executed only by the cleaner.

use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

use crate::ToolId;

/// How the cleaner removes a finding once the user confirmed it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum Removal {
    /// The file or folder goes to the Trash.
    MoveToTrash(PathBuf),
    /// The folder's contents go to the Trash and the folder stays, for apps that expect it to exist.
    EmptyFolder(PathBuf),
    /// An external tool removes it (`docker image rm <id>`).
    ToolCommand(ToolCommand),
}

/// A call of an external tool: the tool and a fixed list of arguments, never a shell string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct ToolCommand {
    tool: ToolId,
    args: Vec<String>,
}

/// Why a removal was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RemovalError {
    #[error("argument {index} of the tool command holds a NUL byte")]
    NulByte { index: usize },
}

impl ToolCommand {
    pub fn new(tool: ToolId, args: Vec<String>) -> Result<Self, RemovalError> {
        if let Some(index) = args.iter().position(|arg| arg.contains('\0')) {
            return Err(RemovalError::NulByte { index });
        }
        Ok(Self { tool, args })
    }

    pub const fn tool(&self) -> ToolId {
        self.tool
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Whether one of the arguments tells the tool to force: `-f`, `--force`, `--force=…`, or `f` among short flags
    /// written together (`-rf`). A forced tool is never run; the plan and the cleaner both ask this one rule.
    pub fn forces(&self) -> bool {
        self.args.iter().any(|arg| {
            let short = arg.strip_prefix('-').filter(|flags| !flags.starts_with('-') && !flags.is_empty());
            arg == "--force"
                || arg.starts_with("--force=")
                || short
                    .is_some_and(|flags| flags.chars().all(|flag| flag.is_ascii_alphabetic()) && flags.contains('f'))
        })
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{RemovalError, ToolCommand, ToolId};

        #[test]
        fn rejects_a_tool_command_argument_with_a_nul_byte() {
            let docker = ToolId::new("docker").unwrap();

            let error = ToolCommand::new(docker, vec!["image".into(), "rm\0-f".into()]);

            assert_eq!(error, Err(RemovalError::NulByte { index: 1 }));
        }

        #[test]
        fn arguments_that_do_not_force_are_not_forcing() {
            let rm = ToolId::new("rm").unwrap();

            for args in [&["a"][..], &["--filter", "x"], &["-q"], &["-"], &["--forced-x"], &["-r1"]] {
                let command = ToolCommand::new(rm, args.iter().map(|arg| (*arg).to_owned()).collect()).unwrap();
                assert!(!command.forces(), "{args:?}");
            }
        }
    }

    mod positive {
        use serde_json::json;

        #[test]
        fn every_form_of_force_is_forcing() {
            let rm = ToolId::new("rm").unwrap();

            for args in [&["-f", "a"][..], &["--force"], &["--force=true"], &["-rf", "a"], &["-fr"], &["-Rf"]] {
                let command = ToolCommand::new(rm, args.iter().map(|arg| (*arg).to_owned()).collect()).unwrap();
                assert!(command.forces(), "{args:?}");
            }
        }

        use crate::{Removal, ToolCommand, ToolId};

        #[test]
        fn keeps_the_arguments_as_a_list() {
            let docker = ToolId::new("docker").unwrap();
            let command = ToolCommand::new(docker, vec!["image".into(), "rm".into(), "a b".into()]).unwrap();

            assert_eq!(command.tool(), docker);
            assert_eq!(command.args(), ["image", "rm", "a b"]);
            assert_eq!(
                serde_json::to_value(Removal::ToolCommand(command)).unwrap(),
                json!({ "kind": "tool-command", "value": { "tool": "docker", "args": ["image", "rm", "a b"] } })
            );
        }
    }
}
