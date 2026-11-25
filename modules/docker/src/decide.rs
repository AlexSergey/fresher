//! Whether a runtime is asked and what its card says: the design's section 3, rows 1 to 6 in their order, on plain
//! values. Nothing here runs a program or asks the system but through `running`.

use fresher_core::{CommandOutput, CommandRunError, InstalledApp, Sizes};

use crate::answer::{Usage, parse};

/// One of the system's lists (the installed or the running apps) could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// The scan was cancelled while the runtime was asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cancelled;

/// What the walk saw of a runtime's disk file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Disk {
    /// The file, with its sizes.
    Seen(Sizes),
    /// Not there: its place was read, or is itself absent.
    Absent,
    /// Its place could not be read, or was not entered.
    Unseen,
}

/// What a runtime's card says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum State {
    /// The app is not installed any more; its disk file is left.
    Left,
    /// The running apps could not be listed, so it was not asked.
    Unknown,
    /// Its app is not running, or the walk saw no socket.
    Stopped,
    /// It was asked and did not answer.
    NoAnswer,
    /// It answered something the module cannot read.
    Unreadable,
    /// It answered.
    Answered(Usage),
}

impl State {
    /// The key of the card's summary.
    pub(crate) const fn key(&self) -> &'static str {
        match self {
            Self::Left => "docker.summary.left",
            Self::Unknown => "docker.summary.unknown",
            Self::Stopped => "docker.summary.stopped",
            Self::NoAnswer => "docker.summary.no-answer",
            Self::Unreadable => "docker.summary.unreadable",
            Self::Answered(_) => "docker.summary.answered",
        }
    }
}

/// What becomes of one runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Nothing is said.
    Silent,
    /// Its card is described without asking it.
    Describe(State),
    /// Its CLI is asked.
    Ask,
}

/// The first row that applies to one runtime:
///
/// 1. the installed apps cannot be listed: `Silent`;
/// 2. its app is not installed and its disk file was not seen: `Silent`;
/// 3. its app is not installed, its disk file was seen: `Left`;
/// 4. the running apps cannot be listed: `Unknown`;
/// 5. its app is not running, or the walk saw no socket: `Stopped`;
/// 6. otherwise: `Ask`.
///
/// `running` tells whether the app runs, and is called only when row 4 is reached.
pub(crate) fn decide(
    installed: Result<Option<&InstalledApp>, NoSystem>,
    running: impl FnOnce() -> Result<bool, NoSystem>,
    socket_seen: bool,
    disk: Disk,
) -> Verdict {
    let Ok(installed) = installed else { return Verdict::Silent };
    if installed.is_none() {
        return match disk {
            Disk::Seen(_) => Verdict::Describe(State::Left),
            Disk::Absent | Disk::Unseen => Verdict::Silent,
        };
    }
    match running() {
        Err(NoSystem) => Verdict::Describe(State::Unknown),
        Ok(false) => Verdict::Describe(State::Stopped),
        Ok(true) if !socket_seen => Verdict::Describe(State::Stopped),
        Ok(true) => Verdict::Ask,
    }
}

/// The state the runtime's answer gives: `NotInstalled`, `Failed` or a status other than 0 is no answer; status 0 is
/// answered when the output reads, unreadable otherwise.
pub(crate) fn answered(result: Result<CommandOutput, CommandRunError>) -> Result<State, Cancelled> {
    match result {
        Err(CommandRunError::Cancelled { .. }) => Err(Cancelled),
        Err(CommandRunError::NotInstalled { .. } | CommandRunError::Failed { .. }) => Ok(State::NoAnswer),
        Ok(output) if output.status != 0 => Ok(State::NoAnswer),
        // The reason of an unreadable answer is dropped: the module has no log, and the card shows no number.
        Ok(output) => Ok(parse(&output.stdout).map_or(State::Unreadable, State::Answered)),
    }
}

#[cfg(test)]
mod tests;
