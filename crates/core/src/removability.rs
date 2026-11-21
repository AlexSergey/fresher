//! Whether the app itself can remove a finding. The app never runs as an administrator: an item it cannot remove is
//! shown with its size and an instruction, never cleaned. Worked out by the engine from what the walk read about the
//! entry, never declared by a module.

use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use crate::{Removal, TextKey};

/// Whether the app can remove an item, and if not, what stands in the way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Removability {
    /// The app can move it to the Trash, or its tool can remove it.
    Removable,
    /// It belongs to another user or the system, or its folder is not writable by the app.
    NeedsAdministrator,
    /// The user locked it in the file manager.
    Locked,
    /// The system's integrity protection guards it.
    ProtectedBySystem,
}

/// What the walk read about an entry, as plain facts. The default is what nothing is known of, and is not removable:
/// a producer that cannot tell leaves a fact at its default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent facts the walk reads, each a plain yes or no")]
pub struct EntryFacts {
    /// It belongs to the user the app runs as.
    pub owned_by_user: bool,
    /// The app may write to the folder that holds it (to move it out).
    pub parent_writable: bool,
    /// A folder the app may write to (to empty it, or to move it with its link to its parent); `true` for a file or a
    /// link, which need no such write.
    pub folder_writable: bool,
    /// The user's immutable flag (locked in the file manager).
    pub user_immutable: bool,
    /// The system's immutable flag: only the system can clear it.
    pub system_immutable: bool,
    /// Restricted: the system's integrity protection guards it.
    pub restricted: bool,
}

impl EntryFacts {
    /// The facts of an entry the app can remove: the user's own, in a writable folder, with no flag.
    pub const REMOVABLE: Self = Self {
        owned_by_user: true,
        parent_writable: true,
        folder_writable: true,
        user_immutable: false,
        system_immutable: false,
        restricted: false,
    };
}

impl Removability {
    pub const ALL: [Self; 4] = [Self::Removable, Self::NeedsAdministrator, Self::Locked, Self::ProtectedBySystem];

    /// The removability of a path, from its facts: protected by the system first, then locked, then needs an
    /// administrator; removable only when every fact allows it.
    pub const fn of(facts: EntryFacts) -> Self {
        if facts.restricted || facts.system_immutable {
            Self::ProtectedBySystem
        } else if facts.user_immutable {
            Self::Locked
        } else if !facts.owned_by_user || !facts.parent_writable || !facts.folder_writable {
            Self::NeedsAdministrator
        } else {
            Self::Removable
        }
    }

    /// The removability of a finding removed by `removal`, given the facts of the matched entry each path names: a
    /// tool's removal is removable here (its need is the tool); a path with no facts is not (fail closed). An item of
    /// several paths is the removability of its first part, in order, that is not removable.
    pub fn of_removal(removal: &Removal, facts: impl Fn(&Path) -> Option<EntryFacts>) -> Self {
        let of_path = |path: &Path| facts(path).map_or(Self::NeedsAdministrator, Self::of);
        match removal {
            Removal::ToolCommand(_) => Self::Removable,
            Removal::MoveToTrash(path) | Removal::EmptyFolder(path) => of_path(path),
            Removal::MoveTogether(parts) => parts
                .all()
                .iter()
                .map(|part| of_path(part))
                .find(|state| !state.is_removable())
                .unwrap_or(Self::Removable),
        }
    }

    pub const fn is_removable(self) -> bool {
        matches!(self, Self::Removable)
    }

    /// A short label: `core.removability.<state>.name`.
    pub fn name_key(self) -> TextKey {
        TextKey::from_valid(format!("core.removability.{}.name", self.id()))
    }

    /// What the user can do about it, or why not: `core.removability.<state>.how`; `None` for `Removable`.
    pub fn how_key(self) -> Option<TextKey> {
        (!self.is_removable()).then(|| TextKey::from_valid(format!("core.removability.{}.how", self.id())))
    }

    const fn id(self) -> &'static str {
        match self {
            Self::Removable => "removable",
            Self::NeedsAdministrator => "needs-administrator",
            Self::Locked => "locked",
            Self::ProtectedBySystem => "protected-by-system",
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::{EntryFacts, Parts, Removability, Removal};

        #[test]
        fn nothing_known_is_not_removable() {
            assert_eq!(Removability::of(EntryFacts::default()), Removability::NeedsAdministrator);
        }

        #[test]
        fn an_item_of_another_user_needs_an_administrator() {
            let facts = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };

            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn an_item_in_a_folder_the_app_cannot_write_needs_an_administrator() {
            let facts = EntryFacts { parent_writable: false, ..EntryFacts::REMOVABLE };

            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn a_folder_the_app_cannot_write_needs_an_administrator() {
            let facts = EntryFacts { folder_writable: false, ..EntryFacts::REMOVABLE };

            assert_eq!(Removability::of(facts), Removability::NeedsAdministrator);
        }

        #[test]
        fn a_locked_item_is_locked() {
            let facts = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };

            assert_eq!(Removability::of(facts), Removability::Locked);
            let of_another_user = EntryFacts { owned_by_user: false, ..facts };
            assert_eq!(Removability::of(of_another_user), Removability::Locked);
        }

        #[test]
        fn a_restricted_item_is_protected_by_the_system_even_if_writable() {
            let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
            let system_immutable = EntryFacts { system_immutable: true, user_immutable: true, ..EntryFacts::REMOVABLE };

            assert_eq!(Removability::of(restricted), Removability::ProtectedBySystem);
            assert_eq!(Removability::of(system_immutable), Removability::ProtectedBySystem);
        }

        #[test]
        fn a_path_removal_without_facts_is_not_removable() {
            for removal in [Removal::MoveToTrash("/home/a".into()), Removal::EmptyFolder("/home/b".into())] {
                assert_eq!(Removability::of_removal(&removal, |_| None), Removability::NeedsAdministrator);
            }
        }

        #[test]
        fn a_together_removal_is_the_first_part_that_cannot_go() {
            let parts = Parts::new(vec!["/home/a".into(), "/home/b".into(), "/home/c".into()]).unwrap();
            let facts = |path: &Path| {
                Some(match path.to_str() {
                    Some("/home/b") => EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE },
                    Some("/home/c") => EntryFacts { restricted: true, ..EntryFacts::REMOVABLE },
                    _ => EntryFacts::REMOVABLE,
                })
            };

            assert_eq!(Removability::of_removal(&Removal::MoveTogether(parts), facts), Removability::Locked);
        }

        #[test]
        fn a_part_without_facts_needs_the_administrator() {
            let parts = Parts::new(vec!["/home/a".into(), "/home/b".into()]).unwrap();
            let facts = |path: &Path| (path == Path::new("/home/a")).then_some(EntryFacts::REMOVABLE);

            assert_eq!(
                Removability::of_removal(&Removal::MoveTogether(parts), facts),
                Removability::NeedsAdministrator
            );
        }

        #[test]
        fn removable_has_no_instruction() {
            assert_eq!(Removability::Removable.how_key(), None);
        }
    }

    mod positive {
        use proptest::prelude::*;
        use serde_json::json;

        use crate::{EntryFacts, Parts, Removability, Removal, ToolCommand, ToolId};

        #[test]
        fn a_users_own_writable_item_is_removable() {
            assert_eq!(Removability::of(EntryFacts::REMOVABLE), Removability::Removable);
        }

        #[test]
        fn a_tool_removal_is_removable() {
            let docker = ToolId::new("docker").unwrap();
            let removal = Removal::ToolCommand(ToolCommand::new(docker, vec!["image".into(), "rm".into()]).unwrap());

            assert_eq!(Removability::of_removal(&removal, |_| None), Removability::Removable);
        }

        #[test]
        fn a_path_removal_takes_the_facts_of_its_entry() {
            let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
            let removal = Removal::EmptyFolder("/home/Library/Caches/a".into());

            assert_eq!(Removability::of_removal(&removal, |_| Some(EntryFacts::REMOVABLE)), Removability::Removable);
            assert_eq!(Removability::of_removal(&removal, |_| Some(locked)), Removability::Locked);
        }

        #[test]
        fn a_together_removal_of_removable_parts_is_removable() {
            let parts = Parts::new(vec!["/home/a".into(), "/home/b".into()]).unwrap();

            assert_eq!(
                Removability::of_removal(&Removal::MoveTogether(parts), |_| Some(EntryFacts::REMOVABLE)),
                Removability::Removable
            );
        }

        #[test]
        fn removability_serializes_in_kebab_case() {
            let names: Vec<_> = Removability::ALL.iter().map(|state| serde_json::to_value(state).unwrap()).collect();

            assert_eq!(
                names,
                [json!("removable"), json!("needs-administrator"), json!("locked"), json!("protected-by-system")]
            );
        }

        #[test]
        fn every_state_but_removable_has_an_instruction() {
            for state in Removability::ALL {
                assert!(state.name_key().as_str().starts_with("core.removability."));
                assert_eq!(state.how_key().is_some(), state != Removability::Removable, "{state:?}");
            }
        }

        proptest! {
            #[test]
            fn removable_only_when_every_fact_allows_it(
                owned_by_user: bool,
                parent_writable: bool,
                folder_writable: bool,
                user_immutable: bool,
                system_immutable: bool,
                restricted: bool,
            ) {
                let facts = EntryFacts {
                    owned_by_user, parent_writable, folder_writable, user_immutable, system_immutable, restricted,
                };
                let allowed = owned_by_user && parent_writable && folder_writable
                    && !user_immutable && !system_immutable && !restricted;

                prop_assert_eq!(Removability::of(facts).is_removable(), allowed);
            }
        }
    }
}
