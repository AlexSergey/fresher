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
    }

    mod positive {
        use serde_json::json;

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
