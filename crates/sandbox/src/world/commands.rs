//! The programs of a sandbox's world: the answers its case files state (`[[case.tool]]`), and the runner that gives
//! them in sandbox mode and records every call they do not state, so no program of the Mac is ever run.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use fresher_core::{CancelToken, CommandOutput, CommandRunError, CommandRunner};

/// In a stated argument, the place of the canonical sandbox folder: a call that names a path of the sandbox (the
/// Docker read's socket) is stated as `unix://{sandbox}/home/...`.
pub(crate) const SANDBOX_TOKEN: &str = "{sandbox}";

/// The longest output a stated tool may give, in bytes.
pub(crate) const MAX_TOOL_OUTPUT: usize = 64 * 1024;

/// A call a case file answers: the program, relative to the sandbox, its exact arguments, its exit status and its
/// output (`stderr` is always empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StatedTool {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) status: i32,
    pub(crate) stdout: String,
}

/// Why a stated tool cannot be kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolProblem {
    /// The program is absolute, empty, or has a component that is empty, `.`, `..` or holds a NUL.
    Program,
    /// An argument holds a NUL.
    Argument,
    /// The output is over [`MAX_TOOL_OUTPUT`].
    Output,
}

impl StatedTool {
    /// The tool as stated, or why it is refused.
    pub(crate) fn new(program: &str, args: Vec<String>, status: i32, stdout: String) -> Result<Self, ToolProblem> {
        if !is_plain_program(program) {
            return Err(ToolProblem::Program);
        }
        if args.iter().any(|arg| arg.contains('\0')) {
            return Err(ToolProblem::Argument);
        }
        if stdout.len() > MAX_TOOL_OUTPUT {
            return Err(ToolProblem::Output);
        }
        Ok(Self { program: PathBuf::from(program), args, status, stdout })
    }

    /// Whether `other` is the same call: the same program and arguments, whatever it answers.
    pub(crate) fn same_call(&self, other: &Self) -> bool {
        self.program == other.program && self.args == other.args
    }
}

/// `true` for a relative path whose every component, split at `/`, is a plain name: not empty, `.` or `..`, no NUL.
fn is_plain_program(program: &str) -> bool {
    !program.is_empty()
        && program.split('/').all(|name| !name.is_empty() && name != "." && name != ".." && !name.contains('\0'))
}

/// Joins `tools` in order: a call stated twice with the same answer is kept once; with another answer, the program of
/// the second is returned.
pub(crate) fn join_tools(tools: impl IntoIterator<Item = StatedTool>) -> Result<Vec<StatedTool>, PathBuf> {
    let mut joined: Vec<StatedTool> = Vec::new();
    for tool in tools {
        match joined.iter().find(|kept| kept.same_call(&tool)) {
            Some(kept) if *kept == tool => {}
            Some(_) => return Err(tool.program),
            None => joined.push(tool),
        }
    }
    Ok(joined)
}

/// The programs of a sandbox's world as sandbox mode runs them: a stated call answers as its case file says, and
/// every other call runs nothing, answers `NotInstalled` and is recorded, so a check can fail on it.
#[derive(Debug)]
pub struct WorldCommands {
    /// The stated calls, each program at its full path below the sandbox.
    tools: Vec<StatedTool>,
    /// Grows with every unstated call and is never trimmed: a case run or a manual sandbox session makes few (the
    /// trial's and Finder's reads of each scan), so it stays tiny.
    unanswered: Mutex<Vec<String>>,
}

impl WorldCommands {
    /// The runner of `tools`, their programs below `sandbox` (the canonical sandbox folder), each
    /// [`SANDBOX_TOKEN`] of their arguments that folder.
    pub(crate) fn new(sandbox: &Path, tools: &[StatedTool]) -> Self {
        let folder = sandbox.to_string_lossy();
        let tools = tools
            .iter()
            .map(|tool| StatedTool {
                program: sandbox.join(&tool.program),
                args: tool.args.iter().map(|arg| arg.replace(SANDBOX_TOKEN, &folder)).collect(),
                ..tool.clone()
            })
            .collect();
        Self { tools, unanswered: Mutex::new(Vec::new()) }
    }

    /// The calls no case file states, in the order they were made: the program and its arguments joined by spaces.
    pub fn unanswered(&self) -> Vec<String> {
        self.unanswered.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl CommandRunner for WorldCommands {
    /// Runs nothing. A stated call answers its status and output; a set token answers `Cancelled`; every call no case
    /// states is recorded (cancelled or not) and answers `NotInstalled`.
    fn run(&self, program: &str, args: &[&str], cancel: &CancelToken) -> Result<CommandOutput, CommandRunError> {
        let stated = self.tools.iter().find(|tool| {
            tool.program == Path::new(program)
                && tool.args.len() == args.len()
                && tool.args.iter().zip(args).all(|(stated, given)| stated == given)
        });
        if stated.is_none() {
            let call = std::iter::once(program).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
            self.unanswered.lock().unwrap_or_else(PoisonError::into_inner).push(call);
        }
        if cancel.is_cancelled() {
            return Err(CommandRunError::Cancelled { program: program.into() });
        }
        match stated {
            Some(tool) => Ok(CommandOutput { status: tool.status, stdout: tool.stdout.clone(), stderr: String::new() }),
            None => Err(CommandRunError::NotInstalled { program: program.into() }),
        }
    }
}
