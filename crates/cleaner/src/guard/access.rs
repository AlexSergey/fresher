//! Check 9: what the item needs from the system is allowed — the capability of each guarded place it lies in or
//! holds, or its tool's.

use fresher_core::{Access, AccessStatus, Capability, GuardedPlace, Need, SkipReason, ToolId};

use crate::paths::Placed;

/// Check 9 for a path: every guarded place the item lies in, or that lies inside the item (it would go with it),
/// must be allowed by `access` (`AccessStatus::check`), by either form of the path.
pub(super) fn check(placed: &Placed, guarded: &[GuardedPlace], access: &AccessStatus) -> Result<(), SkipReason> {
    let touches = |place: &GuardedPlace| {
        [&placed.lexical, &placed.physical]
            .iter()
            .any(|path| path.starts_with(&place.path) || place.path.starts_with(path))
    };
    let needs: Vec<Need> =
        guarded.iter().filter(|place| touches(place)).map(|place| Need::required(place.capability)).collect();
    access.check(&needs).map_err(|missing| match missing.first() {
        Some(capability) => SkipReason::MissingCapability { capability: *capability },
        None => SkipReason::CheckFailed,
    })
}

/// Check 9 for a tool: the tool check of plan 006 must have found it ready (`Granted`); a tool the status does not
/// list is not allowed.
pub(super) fn check_tool(tool: ToolId, access: &AccessStatus) -> Result<(), SkipReason> {
    let capability = Capability::Tool(tool);
    match access.access(&capability) {
        Access::Granted => Ok(()),
        _ => Err(SkipReason::MissingCapability { capability }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{Access, AccessStatus, Capability, CapabilityState, GuardedFolder, GuardedPlace, ToolId};

    use crate::paths::Placed;

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);

    fn placed(path: &str) -> Placed {
        Placed {
            lexical: PathBuf::from(path),
            physical: PathBuf::from(format!("/private{path}")),
            root: PathBuf::from("/private/home"),
            device: 1,
            relative: Vec::new(),
        }
    }

    fn desktop() -> Vec<GuardedPlace> {
        vec![GuardedPlace { path: "/home/Desktop".into(), capability: DESKTOP }]
    }

    fn status(states: &[(Capability, Access)]) -> AccessStatus {
        AccessStatus {
            capabilities: states.iter().map(|&(capability, access)| CapabilityState::new(capability, access)).collect(),
        }
    }

    fn docker() -> ToolId {
        ToolId::new("docker").unwrap()
    }

    mod negative {
        use fresher_core::{Access, Capability, CapabilityState, GuardedPlace, SkipReason};

        use super::{DESKTOP, desktop, docker, placed, status};
        use crate::guard::access::{check, check_tool};

        #[test]
        fn an_item_in_a_guarded_place_without_its_capability_is_refused() {
            let denied = status(&[(DESKTOP, Access::Denied)]);

            let answer = check(&placed("/home/Desktop/a"), &desktop(), &denied);

            assert_eq!(answer, Err(SkipReason::MissingCapability { capability: DESKTOP }));
        }

        #[test]
        fn an_item_holding_a_guarded_place_needs_its_capability() {
            let unknown = status(&[(DESKTOP, Access::Unknown)]);

            assert_eq!(
                check(&placed("/home"), &desktop(), &unknown),
                Err(SkipReason::MissingCapability { capability: DESKTOP })
            );
        }

        #[test]
        fn a_guarded_place_named_by_its_physical_path_counts() {
            let physical = vec![GuardedPlace { path: "/private/home/Desktop".into(), capability: DESKTOP }];
            let denied = status(&[(DESKTOP, Access::NotAsked)]);

            assert!(check(&placed("/home/Desktop/a"), &physical, &denied).is_err());
        }

        #[test]
        fn a_tool_whose_capability_is_denied_is_refused() {
            let tool = Capability::Tool(docker());
            let denied = fresher_core::AccessStatus { capabilities: vec![CapabilityState::tool_not_ready(docker())] };

            assert_eq!(check_tool(docker(), &denied), Err(SkipReason::MissingCapability { capability: tool }));
        }

        #[test]
        fn a_tool_the_status_does_not_list_is_refused() {
            let tool = Capability::Tool(docker());

            assert_eq!(check_tool(docker(), &status(&[])), Err(SkipReason::MissingCapability { capability: tool }));
        }
    }

    mod positive {
        use fresher_core::{Access, Capability};

        use super::{DESKTOP, desktop, docker, placed, status};
        use crate::guard::access::{check, check_tool};

        #[test]
        fn a_guarded_place_with_its_capability_passes() {
            let granted = status(&[(DESKTOP, Access::Granted)]);

            assert_eq!(check(&placed("/home/Desktop/a"), &desktop(), &granted), Ok(()));
        }

        #[test]
        fn an_item_outside_every_guarded_place_needs_nothing() {
            let denied = status(&[(DESKTOP, Access::Denied)]);

            assert_eq!(check(&placed("/home/Desktopx/a"), &desktop(), &denied), Ok(()));
        }

        #[test]
        fn a_tool_found_ready_passes() {
            assert_eq!(check_tool(docker(), &status(&[(Capability::Tool(docker()), Access::Granted)])), Ok(()));
        }
    }
}
