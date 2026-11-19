use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use fresher_core::{
    Access, AccessProbe, AccessStatus, Capability, CapabilityState, GuardedPlace, SystemUi, SystemUiError,
};

use crate::fakes::lock;

/// Any state of any capability, guarded places under a home folder (a fixture tree's root), and a script of what
/// trying each place answers. Records every `try_place` call.
#[derive(Debug, Default)]
pub struct FakeAccess {
    states: Mutex<Vec<CapabilityState>>,
    home: PathBuf,
    places: Vec<GuardedPlace>,
    answers: HashMap<PathBuf, Access>,
    tried: Mutex<Vec<GuardedPlace>>,
}

impl FakeAccess {
    /// A system with exactly these capabilities in these states; any other is `NotApplicable`.
    pub fn with(states: &[(Capability, Access)]) -> Self {
        let states = states.iter().map(|&(capability, access)| CapabilityState::new(capability, access)).collect();
        Self { states: Mutex::new(states), ..Self::default() }
    }

    /// The home folder the guarded places are under: a fixture tree's root.
    #[must_use]
    pub fn home(mut self, home: impl Into<PathBuf>) -> Self {
        self.home = home.into();
        self
    }

    /// Guards `relative` (under the home folder) with `capability`.
    #[must_use]
    pub fn guard(mut self, relative: impl AsRef<Path>, capability: Capability) -> Self {
        self.places.push(GuardedPlace { path: self.home.join(relative), capability });
        self
    }

    /// Trying the place `relative` (under the home folder) answers `access`; an unscripted place answers `Unknown`.
    #[must_use]
    pub fn answering(mut self, relative: impl AsRef<Path>, access: Access) -> Self {
        self.answers.insert(self.home.join(relative), access);
        self
    }

    /// Sets the state of `capability` from now on, as a grant in the system's settings would.
    pub fn set(&self, capability: Capability, access: Access) {
        let mut states = lock(&self.states);
        match states.iter_mut().find(|state| state.capability == capability) {
            Some(state) => state.access = access,
            None => states.push(CapabilityState::new(capability, access)),
        }
    }

    /// The places tried so far, in order.
    pub fn tried(&self) -> Vec<GuardedPlace> {
        lock(&self.tried).clone()
    }
}

impl AccessProbe for FakeAccess {
    fn status(&self) -> AccessStatus {
        AccessStatus { capabilities: lock(&self.states).clone() }
    }

    /// The guarded places inside any of `roots`.
    fn guarded_places(&self, roots: &[PathBuf]) -> Vec<GuardedPlace> {
        let inside = |place: &&GuardedPlace| roots.iter().any(|root| place.path.starts_with(root));
        self.places.iter().filter(inside).cloned().collect()
    }

    /// Records the try and, as the system's probe remembers every answer of its tries, sets the capability's state to
    /// the answer, `Unknown` included.
    fn try_place(&self, place: &GuardedPlace) -> Access {
        lock(&self.tried).push(place.clone());
        let answer = self.answers.get(&place.path).copied().unwrap_or(Access::Unknown);
        self.set(place.capability, answer);
        answer
    }
}

/// One call of the system's interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemUiCall {
    OpenAccessSettings(Capability),
    Reveal(PathBuf),
}

/// Records every call of the system's interface; it can be told to fail them all.
#[derive(Debug, Default)]
pub struct FakeSystemUi {
    calls: Mutex<Vec<SystemUiCall>>,
    error: Option<SystemUiError>,
}

impl FakeSystemUi {
    /// Every call is recorded and answers `error`.
    pub fn failing(error: SystemUiError) -> Self {
        Self { error: Some(error), ..Self::default() }
    }

    /// The calls so far, in order.
    pub fn calls(&self) -> Vec<SystemUiCall> {
        lock(&self.calls).clone()
    }

    fn record(&self, call: SystemUiCall) -> Result<(), SystemUiError> {
        lock(&self.calls).push(call);
        self.error.clone().map_or(Ok(()), Err)
    }
}

impl SystemUi for FakeSystemUi {
    fn open_access_settings(&self, capability: &Capability) -> Result<(), SystemUiError> {
        self.record(SystemUiCall::OpenAccessSettings(*capability))
    }

    fn reveal(&self, path: &Path) -> Result<(), SystemUiError> {
        self.record(SystemUiCall::Reveal(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Capability, GuardedFolder};

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);
    const DOCUMENTS: Capability = Capability::Folder(GuardedFolder::Documents);

    mod negative {
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedPlace, SystemUi, SystemUiError};

        use super::DESKTOP;
        use crate::{FakeAccess, FakeSystemUi, FixtureTree, SystemUiCall};

        #[test]
        fn an_unlisted_capability_is_not_applicable() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);

            assert_eq!(access.status().access(&DESKTOP), Access::NotApplicable);
            assert_eq!(FakeAccess::default().status().access(&Capability::AllFiles), Access::NotApplicable);
        }

        #[test]
        fn try_place_on_an_unscripted_place_is_unknown_and_remembered() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"Desktop\"\n").unwrap();
            let access = FakeAccess::with(&[(DESKTOP, Access::NotAsked)]).home(tree.root()).guard("Desktop", DESKTOP);
            let place = GuardedPlace { path: tree.root().join("Desktop"), capability: DESKTOP };

            assert_eq!(access.try_place(&place), Access::Unknown);
            assert_eq!(access.status().access(&DESKTOP), Access::Unknown);
        }

        #[test]
        fn a_place_outside_every_root_is_not_listed() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"Desktop\"\n").unwrap();
            let access = FakeAccess::with(&[]).home(tree.root()).guard("Desktop", DESKTOP);

            assert_eq!(access.guarded_places(&[tree.root().join("Projects")]), []);
            assert_eq!(access.guarded_places(&[]), []);
        }

        #[test]
        fn a_failing_system_ui_returns_its_error() {
            let error = SystemUiError::Failed { reason: "refused by the test".into() };
            let ui = FakeSystemUi::failing(error.clone());

            assert_eq!(ui.open_access_settings(&Capability::AllFiles), Err(error.clone()));
            assert_eq!(ui.reveal(&PathBuf::from("/home/a")), Err(error));
            assert_eq!(ui.calls().len(), 2);
            assert_eq!(ui.calls()[0], SystemUiCall::OpenAccessSettings(Capability::AllFiles));
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedPlace, SystemUi};

        use super::{DESKTOP, DOCUMENTS};
        use crate::{FakeAccess, FakeSystemUi, FixtureTree, SystemUiCall};

        #[test]
        fn set_changes_the_state_seen_by_the_next_status() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let before = access.status();

            access.set(Capability::AllFiles, Access::Granted);
            access.set(DESKTOP, Access::NotAsked);

            assert_eq!(before.access(&Capability::AllFiles), Access::Denied);
            assert_eq!(access.status().access(&Capability::AllFiles), Access::Granted);
            assert_eq!(access.status().access(&DESKTOP), Access::NotAsked);
        }

        #[test]
        fn try_place_calls_are_recorded_in_order() {
            let tree =
                FixtureTree::from_toml_str("[[dir]]\npath = \"Desktop\"\n\n[[dir]]\npath = \"Documents\"\n").unwrap();
            let access = FakeAccess::with(&[])
                .home(tree.root())
                .guard("Desktop", DESKTOP)
                .guard("Documents", DOCUMENTS)
                .answering("Desktop", Access::Granted)
                .answering("Documents", Access::Denied);
            let place = |name: &str, capability| GuardedPlace { path: tree.root().join(name), capability };

            let answers =
                [access.try_place(&place("Documents", DOCUMENTS)), access.try_place(&place("Desktop", DESKTOP))];

            assert_eq!(answers, [Access::Denied, Access::Granted]);
            assert_eq!(access.tried(), [place("Documents", DOCUMENTS), place("Desktop", DESKTOP)]);
        }

        #[test]
        fn a_granted_or_denied_try_is_remembered_in_the_status() {
            let tree =
                FixtureTree::from_toml_str("[[dir]]\npath = \"Desktop\"\n\n[[dir]]\npath = \"Documents\"\n").unwrap();
            let access = FakeAccess::with(&[(DESKTOP, Access::NotAsked), (DOCUMENTS, Access::NotAsked)])
                .home(tree.root())
                .guard("Desktop", DESKTOP)
                .guard("Documents", DOCUMENTS)
                .answering("Desktop", Access::Granted)
                .answering("Documents", Access::Denied);

            access.try_place(&GuardedPlace { path: tree.root().join("Desktop"), capability: DESKTOP });
            access.try_place(&GuardedPlace { path: tree.root().join("Documents"), capability: DOCUMENTS });

            assert_eq!(access.status().access(&DESKTOP), Access::Granted);
            assert_eq!(access.status().access(&DOCUMENTS), Access::Denied);
        }

        #[test]
        fn guarded_places_are_on_the_fixture_tree() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"Desktop\"\n").unwrap();
            let access = FakeAccess::with(&[]).home(tree.root()).guard("Desktop", DESKTOP);

            let places = access.guarded_places(&[tree.root().to_path_buf()]);

            assert_eq!(places, [GuardedPlace { path: tree.root().join("Desktop"), capability: DESKTOP }]);
            assert!(places[0].path.is_dir());
        }

        #[test]
        fn system_ui_calls_are_recorded() {
            let ui = FakeSystemUi::default();

            ui.open_access_settings(&DESKTOP).unwrap();
            ui.reveal(&PathBuf::from("/home/a")).unwrap();

            assert_eq!(
                ui.calls(),
                [SystemUiCall::OpenAccessSettings(DESKTOP), SystemUiCall::Reveal(PathBuf::from("/home/a"))]
            );
        }
    }
}
