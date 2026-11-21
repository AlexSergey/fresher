//! How a finding would be removed: described by a module as data, executed only by the cleaner.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

use crate::ToolId;
use crate::cleanup::is_plain;

/// The most paths one item may name.
pub const MAX_PARTS: usize = 64;

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
    /// Several files or folders that leave together, as one item the user confirms: the first is the finding's
    /// subject and goes first; put back brings it back last.
    MoveTogether(#[ts(as = "Vec<PathBuf>")] Parts),
}

/// The paths of a `MoveTogether`: two to `MAX_PARTS`, each absolute and plain, none twice, none inside another.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Parts(Vec<PathBuf>);

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
    #[error("an item of several paths names fewer than two")]
    TooFewParts,
    #[error("an item of several paths names {count}, more than {MAX_PARTS}")]
    TooManyParts { count: usize },
    #[error("part {index} is not an absolute path of plain names")]
    NotPlain { index: usize },
    #[error("part {index} names a path an earlier part names")]
    PartTwice { index: usize },
    #[error("part {inner} is inside part {outer}")]
    PartInsidePart { inner: usize, outer: usize },
}

impl Parts {
    /// The parts of one item, in the order they leave: two to `MAX_PARTS` absolute paths with no `.` or `..`, none
    /// the same as another or inside another (by components, never by text).
    pub fn new(paths: Vec<PathBuf>) -> Result<Self, RemovalError> {
        if paths.len() < 2 {
            return Err(RemovalError::TooFewParts);
        }
        if paths.len() > MAX_PARTS {
            return Err(RemovalError::TooManyParts { count: paths.len() });
        }
        if let Some(index) = paths.iter().position(|path| !is_plain(path)) {
            return Err(RemovalError::NotPlain { index });
        }
        for (later, path) in paths.iter().enumerate() {
            for (earlier, other) in paths[..later].iter().enumerate() {
                if path == other {
                    return Err(RemovalError::PartTwice { index: later });
                }
                if path.starts_with(other) {
                    return Err(RemovalError::PartInsidePart { inner: later, outer: earlier });
                }
                if other.starts_with(path) {
                    return Err(RemovalError::PartInsidePart { inner: earlier, outer: later });
                }
            }
        }
        Ok(Self(paths))
    }

    /// The first part: the finding's subject, the part that goes first.
    pub fn subject(&self) -> &Path {
        &self.0[0]
    }

    /// Every part, the subject first.
    pub fn all(&self) -> &[PathBuf] {
        &self.0
    }

    /// The parts after the subject.
    pub fn rest(&self) -> &[PathBuf] {
        &self.0[1..]
    }
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
        use std::path::PathBuf;

        use crate::{MAX_PARTS, Parts, RemovalError, ToolCommand, ToolId};

        fn parts(paths: &[&str]) -> Result<Parts, RemovalError> {
            Parts::new(paths.iter().map(PathBuf::from).collect())
        }

        #[test]
        fn parts_refuse_fewer_than_two() {
            assert_eq!(parts(&[]), Err(RemovalError::TooFewParts));
            assert_eq!(parts(&["/home/a"]), Err(RemovalError::TooFewParts));
        }

        #[test]
        fn parts_refuse_more_than_the_limit() {
            let paths: Vec<PathBuf> = (0..=MAX_PARTS).map(|index| PathBuf::from(format!("/home/{index}"))).collect();

            assert_eq!(paths.len(), 65);
            assert_eq!(Parts::new(paths), Err(RemovalError::TooManyParts { count: 65 }));
        }

        #[test]
        fn parts_refuse_a_relative_or_dotted_path() {
            assert_eq!(parts(&["/home/a", "home/b"]), Err(RemovalError::NotPlain { index: 1 }));
            assert_eq!(parts(&["/home/./a", "/home/b"]), Err(RemovalError::NotPlain { index: 0 }));
            assert_eq!(parts(&["/home/a", "/home/x/../b"]), Err(RemovalError::NotPlain { index: 1 }));
            assert_eq!(parts(&["/home/a", "/home/b/."]), Err(RemovalError::NotPlain { index: 1 }));
        }

        #[test]
        fn parts_refuse_a_path_twice() {
            assert_eq!(parts(&["/home/a", "/home/b", "/home/a"]), Err(RemovalError::PartTwice { index: 2 }));
            assert_eq!(parts(&["/home/a", "/home//a/"]), Err(RemovalError::PartTwice { index: 1 }));
        }

        #[test]
        fn parts_refuse_a_part_inside_another() {
            assert_eq!(parts(&["/home/a", "/home/a/b"]), Err(RemovalError::PartInsidePart { inner: 1, outer: 0 }));
            assert_eq!(
                parts(&["/home/x", "/home/a/b", "/home/a"]),
                Err(RemovalError::PartInsidePart { inner: 1, outer: 2 })
            );
            assert!(parts(&["/home/a/b", "/home/a/bc"]).is_ok(), "compared by components, not by text");
        }

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

        use std::path::{Path, PathBuf};

        use crate::{Parts, Removal, ToolCommand, ToolId};

        #[test]
        fn parts_keep_their_order_and_the_first_is_the_subject() {
            let paths: Vec<PathBuf> = ["/home/m/latest", "/home/b/2", "/home/b/1"].map(PathBuf::from).to_vec();

            let parts = Parts::new(paths.clone()).unwrap();

            assert_eq!(parts.subject(), Path::new("/home/m/latest"));
            assert_eq!(parts.all(), paths);
            assert_eq!(parts.rest(), &paths[1..]);
        }

        #[test]
        fn move_together_serializes_as_a_list() {
            let parts = Parts::new(vec!["/home/a".into(), "/home/b".into()]).unwrap();

            assert_eq!(
                serde_json::to_value(Removal::MoveTogether(parts)).unwrap(),
                json!({ "kind": "move-together", "value": ["/home/a", "/home/b"] })
            );
        }

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
