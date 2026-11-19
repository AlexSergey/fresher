//! The app's own stand-in for the core's access port, where the Mac's real access must not be asked.

use std::path::PathBuf;

use fresher_core::{Access, AccessProbe, AccessStatus, Capability, CapabilityState, GuardedPlace};

/// Answers `Unknown` for Full Disk Access and `NotAsked` for every other permission, knows no guarded place and
/// never tries one: sandbox mode's access, whose world is not the Mac's, and the app's tests' (`docs/hacks/`).
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

#[cfg(test)]
mod tests {
    mod positive {
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedFolder, GuardedPlace};

        use crate::adapters::UnknownAccess;

        #[test]
        fn the_placeholder_knows_nothing() {
            let roots = [PathBuf::from("~")];
            let access = UnknownAccess.status();
            assert_eq!(access.access(&Capability::AllFiles), Access::Unknown);
            assert_eq!(access.access(&Capability::Folder(GuardedFolder::Desktop)), Access::NotAsked);
            assert_eq!(access.capabilities.len(), Capability::SYSTEM.len());
            let place = GuardedPlace { path: "/home/Desktop".into(), capability: Capability::AllFiles };
            assert_eq!(UnknownAccess.guarded_places(&roots), []);
            assert_eq!(UnknownAccess.try_place(&place), Access::Unknown);
        }
    }
}
