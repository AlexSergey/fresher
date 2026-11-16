//! What a scan saw and what it did not: the guarded places it did not enter and the folders it could not read, as
//! counts and capabilities. No path is held: the UI says what was not seen and what would show it.

use serde::Serialize;
use ts_rs::TS;

use crate::{Access, Capability, Unreadable};

/// How much of the scan roots one scan saw.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    /// The guarded places the scan did not enter, once per capability.
    pub not_entered: Vec<NotEntered>,
    /// The folders that could not be read, counted by why.
    pub unreadable: Vec<UnreadableCount>,
}

impl Coverage {
    /// `true` when every place was entered and every folder read.
    pub fn is_complete(&self) -> bool {
        self.not_entered.is_empty() && self.unreadable.iter().all(|count| count.folders == 0)
    }
}

/// Places guarded by `capability` that the scan did not enter, with the capability's state that kept it out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct NotEntered {
    pub capability: Capability,
    pub access: Access,
}

/// How many folders could not be read for one reason, and the capability that would let the app read them, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UnreadableCount {
    pub reason: Unreadable,
    #[ts(type = "number")]
    pub folders: u64,
    pub unlocked_by: Option<Capability>,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{Access, Capability, Coverage, GuardedFolder, NotEntered, Unreadable, UnreadableCount};

        #[test]
        fn a_place_not_entered_makes_the_coverage_incomplete() {
            let desktop = Capability::Folder(GuardedFolder::Desktop);
            let coverage = Coverage {
                not_entered: vec![NotEntered { capability: desktop, access: Access::NotAsked }],
                ..Coverage::default()
            };

            assert!(!coverage.is_complete());
        }

        #[test]
        fn an_unreadable_folder_makes_the_coverage_incomplete() {
            let unreadable = vec![UnreadableCount {
                reason: Unreadable::Guarded,
                folders: 2,
                unlocked_by: Some(Capability::AllFiles),
            }];
            let coverage = Coverage { unreadable, ..Coverage::default() };

            assert!(!coverage.is_complete());
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{Access, Capability, Coverage, GuardedFolder, NotEntered, Unreadable, UnreadableCount};

        #[test]
        fn an_empty_coverage_is_complete() {
            assert!(Coverage::default().is_complete());
        }

        #[test]
        fn coverage_serializes_in_camel_case_with_kebab_case_values() {
            let coverage = Coverage {
                not_entered: vec![NotEntered {
                    capability: Capability::Folder(GuardedFolder::Downloads),
                    access: Access::Denied,
                }],
                unreadable: vec![UnreadableCount { reason: Unreadable::NoPermission, folders: 3, unlocked_by: None }],
            };

            assert_eq!(
                serde_json::to_value(coverage).unwrap(),
                json!({
                    "notEntered": [{ "capability": { "kind": "folder", "value": "downloads" }, "access": "denied" }],
                    "unreadable": [{ "reason": "no-permission", "folders": 3, "unlockedBy": null }],
                })
            );
        }
    }
}
