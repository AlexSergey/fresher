//! The preconditions of a cleanup, checked when a plan is prepared and again when it is confirmed. They fail closed:
//! when one cannot be known, nothing is cleaned. In order:
//!
//! 1. the settings file is `Loaded`, so the ignore list is known (`Settings`);
//! 2. the current version of the agreement is accepted (`Agreement`);
//! 3. the cleanup journal is `Ready` (`Journal`).

use fresher_core::{AGREEMENT_VERSION, BlockedReason, CommandError, JournalState, SettingsFile};

use crate::engine::CleanupPorts;
use crate::settings::Settings;

/// The cleanup's ports when a cleanup may run; otherwise `CleaningBlocked` with the first precondition that fails. An
/// engine without cleanup ports has no journal to write.
pub(crate) fn check<'ports>(
    settings: &Settings,
    ports: Option<&'ports CleanupPorts>,
) -> Result<&'ports CleanupPorts, CommandError> {
    if settings.file() != SettingsFile::Loaded {
        return Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings });
    }
    if settings.agreement().map(|acceptance| acceptance.version) != Some(AGREEMENT_VERSION) {
        return Err(CommandError::CleaningBlocked { reason: BlockedReason::Agreement });
    }
    match ports {
        Some(ports) if ports.journal.state() == JournalState::Ready => Ok(ports),
        _ => Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }),
    }
}
