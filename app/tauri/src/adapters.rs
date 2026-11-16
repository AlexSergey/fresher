//! The app's own stand-ins for the core's ports, until the crates of later plans provide the real ones.

use std::path::{Path, PathBuf};

use fresher_core::{
    Access, AccessProbe, AccessStatus, Capability, CapabilityState, GuardedPlace, SystemUi, SystemUiError,
};

/// Answers `Unknown` for Full Disk Access and `NotAsked` for every other permission, knows no guarded place and
/// never tries one, until plan 006/03 asks macOS (`docs/hacks/`).
#[derive(Debug)]
pub(crate) struct UnknownAccess;

impl AccessProbe for UnknownAccess {
    fn status(&self) -> AccessStatus {
        let state = |capability| {
            let access = if capability == Capability::AllFiles { Access::Unknown } else { Access::NotAsked };
            CapabilityState::new(capability, access)
        };
        AccessStatus { capabilities: Capability::SYSTEM.into_iter().map(state).collect() }
    }

    fn guarded_places(&self, _roots: &[PathBuf]) -> Vec<GuardedPlace> {
        Vec::new()
    }

    fn try_place(&self, _place: &GuardedPlace) -> Access {
        Access::Unknown
    }
}

/// Answers `Unsupported` to everything, until plan 006/03 opens System Settings and Finder (`docs/hacks/`).
#[derive(Debug)]
pub(crate) struct NoSystemUi;

impl SystemUi for NoSystemUi {
    fn open_access_settings(&self, _capability: &Capability) -> Result<(), SystemUiError> {
        Err(SystemUiError::Unsupported)
    }

    fn reveal(&self, _path: &Path) -> Result<(), SystemUiError> {
        Err(SystemUiError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedFolder, GuardedPlace, SystemUi, SystemUiError};

        use crate::adapters::{NoSystemUi, UnknownAccess};

        #[test]
        fn the_placeholders_know_nothing() {
            let roots = [PathBuf::from("~")];
            let access = UnknownAccess.status();
            assert_eq!(access.access(&Capability::AllFiles), Access::Unknown);
            assert_eq!(access.access(&Capability::Folder(GuardedFolder::Desktop)), Access::NotAsked);
            assert_eq!(access.capabilities.len(), Capability::SYSTEM.len());
            let place = GuardedPlace { path: "/home/Desktop".into(), capability: Capability::AllFiles };
            assert_eq!(UnknownAccess.guarded_places(&roots), []);
            assert_eq!(UnknownAccess.try_place(&place), Access::Unknown);
            assert_eq!(NoSystemUi.open_access_settings(&Capability::AllFiles), Err(SystemUiError::Unsupported));
            assert_eq!(NoSystemUi.reveal(&place.path), Err(SystemUiError::Unsupported));
        }
    }
}
