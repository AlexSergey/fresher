//! Showing a finding in Finder: the way the user removes, by hand, an item the app may not remove.

use fresher_core::{CommandError, FindingId, Subject};

use crate::Engine;
use crate::error::EngineError;

impl Engine {
    /// Shows finding `id` in Finder, selected: the path stays in the core, the UI passes only the id. Refused for a
    /// finding of another scan or an ignored one (`UnknownFinding`) and for a finding without a path of its own
    /// (`NothingToReveal`); a system that cannot show it is an internal error.
    pub fn reveal_finding(&self, id: FindingId) -> Result<(), CommandError> {
        let finding = self.finding(id)?;
        let Subject::Path(path) = finding.finding().subject() else {
            return Err(CommandError::NothingToReveal);
        };
        self.shared.ports.system_ui.reveal(path).map_err(|error| EngineError::SystemUi(error).into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{DangerLevel, NewFinding, Subject, ToolId};
    use fresher_testing::FakeSystemUi;

    use crate::Engine;
    use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, category, finding, scan};

    /// After one scan: finding 1 is the folder `/home/Caches`, finding 2 a Docker image (no path of its own).
    fn scanned(system_ui: FakeSystemUi) -> (Engine, Arc<FakeSystemUi>) {
        let images = category("alpha.images", DangerLevel::Low);
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .category("alpha.images", DangerLevel::Low)
            .folder("Caches")
            .on_finish(move |_, out| {
                let path = finding(&images, "/unused", bytes(9));
                let docker = ToolId::new("docker").unwrap();
                out.push(NewFinding::new(
                    &images,
                    "image",
                    Subject::External { tool: docker, id: "sha256:1".into() },
                    path.sizes(),
                    path.title().clone(),
                    path.reason().clone(),
                    path.removal().clone(),
                ));
                Ok(())
            });
        let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(10));
        let fixture = Fixture::new(walk).module(module).system_ui(system_ui);
        let system_ui = Arc::clone(&fixture.system_ui);
        let engine = fixture.engine();
        scan(&engine);
        (engine, system_ui)
    }

    mod negative {
        use fresher_core::{CommandError, FindingId, SystemUiError};
        use fresher_testing::{FakeSystemUi, SystemUiCall};

        use super::scanned;

        #[test]
        fn reveal_of_an_unknown_id_is_refused() {
            let (engine, system_ui) = scanned(FakeSystemUi::default());

            let refused = engine.reveal_finding(FindingId::new(99));

            assert_eq!(refused, Err(CommandError::UnknownFinding { id: FindingId::new(99) }));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn reveal_of_an_external_subject_is_refused() {
            let (engine, system_ui) = scanned(FakeSystemUi::default());

            assert_eq!(engine.reveal_finding(FindingId::new(2)), Err(CommandError::NothingToReveal));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn a_failed_reveal_is_reported() {
            let failure = SystemUiError::Failed { reason: "open exited with 1".into() };
            let (engine, system_ui) = scanned(FakeSystemUi::failing(failure));

            assert_eq!(engine.reveal_finding(FindingId::new(1)), Err(CommandError::Internal));
            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal("/home/Caches".into())]);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::FindingId;
        use fresher_testing::{FakeSystemUi, SystemUiCall};

        use super::scanned;

        #[test]
        fn show_in_finder_reveals_the_findings_path() {
            let (engine, system_ui) = scanned(FakeSystemUi::default());

            engine.reveal_finding(FindingId::new(1)).unwrap();

            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal(PathBuf::from("/home/Caches"))]);
        }
    }
}
