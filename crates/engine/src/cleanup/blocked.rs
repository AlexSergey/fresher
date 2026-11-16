//! The preconditions of a cleanup, checked when a plan is prepared and again when it is confirmed. They fail closed:
//! when one cannot be known, nothing is cleaned.

use fresher_core::{BlockedReason, CommandError, JournalState, SettingsFile};

use crate::engine::CleanupPorts;

/// The cleanup's ports when a cleanup may run: the settings file is `Loaded` (so the ignore list is known) and the
/// journal is `Ready`; otherwise `CleaningBlocked`, the settings first. An engine without cleanup ports has no
/// journal to write.
pub(crate) fn check(file: SettingsFile, ports: Option<&CleanupPorts>) -> Result<&CleanupPorts, CommandError> {
    if file != SettingsFile::Loaded {
        return Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings });
    }
    match ports {
        Some(ports) if ports.journal.state() == JournalState::Ready => Ok(ports),
        _ => Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }),
    }
}
