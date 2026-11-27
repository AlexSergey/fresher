//! Showing a finding in Finder: the way the user removes, by hand, an item the app may not remove.

use fresher_core::{CommandError, FindingId, NodeId, Subject};

use crate::Engine;
use crate::error::EngineError;
use crate::largest::revealable;

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

    /// Shows the row `node` of the last state in Finder, selected (Show in Finder of a Largest row): the UI names the
    /// id, the engine holds the path. `NoScan` before any scan, `Busy` while one runs, `StaleNode` for a node of
    /// another state or none, `NotOwnItem` for the home, a path outside it or one a cleanup removed since the scan; a
    /// system that cannot show it is an internal error.
    pub fn reveal_node(&self, node: NodeId) -> Result<(), CommandError> {
        let committed = self.committed_for_largest()?;
        let state = &*committed.state;
        if node.state() != state.id() {
            return Err(CommandError::StaleNode);
        }
        let path = state.path(node).ok_or(CommandError::StaleNode)?;
        if !revealable(&self.shared, &committed, &path) {
            return Err(CommandError::NotOwnItem);
        }
        self.shared.ports.system_ui.reveal(&path).map_err(|error| EngineError::SystemUi(error).into())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::{
        ByteSize, DangerLevel, NewFinding, NodeId, NodeKind, Sizes, StateId, Subject, Timestamp, ToolId,
    };
    use fresher_testing::{FakeSystemUi, FixedDiskState};

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

    /// An engine that scanned a state of `/home` (a folder `Docs` with `a.bin`, and `/other`) and the node at `path`.
    fn with_home(system_ui: FakeSystemUi) -> (Engine, Arc<FakeSystemUi>) {
        let nodes = [
            ("/home", NodeKind::Folder, 100),
            ("/home/Docs", NodeKind::Folder, 100),
            ("/home/Docs/a.bin", NodeKind::File, 100),
            ("/other", NodeKind::Folder, 0),
        ];
        let sizes = |size: u64| Sizes::complete(ByteSize::new(size), ByteSize::new(size));
        let state = FixedDiskState::new(
            StateId::new(1),
            Timestamp::from_secs(1_000),
            nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, sizes(size))),
        );
        let walk = ScriptedWalk::new().committing(vec![Arc::new(state)]);
        let fixture = Fixture::new(walk).system_ui(system_ui);
        let system_ui = Arc::clone(&fixture.system_ui);
        let engine = fixture.engine();
        scan(&engine);
        (engine, system_ui)
    }

    fn node(engine: &Engine, path: &str) -> NodeId {
        let state = engine.shared.results.lock().unwrap().state().unwrap();
        state.find(Path::new(path)).unwrap()
    }

    mod negative {
        use std::sync::Arc;
        use std::time::Duration;

        use fresher_core::{CommandError, Event, FindingId, NodeId, Operation, StateId, SystemUiError};
        use fresher_testing::{FakeSystemUi, SystemUiCall};

        use super::{node, scanned, with_home};
        use crate::testing::{Fixture, ScriptedWalk, engine, scan};

        #[test]
        fn reveal_of_a_node_before_a_scan_is_refused() {
            let engine = engine(ScriptedWalk::new());

            assert_eq!(engine.reveal_node(NodeId::new(StateId::new(1), 0)), Err(CommandError::NoScan));
        }

        #[test]
        fn reveal_of_a_node_while_scanning_is_refused() {
            let (walk, gate) = ScriptedWalk::new().hold();
            gate.release();
            let fixture = Fixture::new(walk);
            let system_ui = Arc::clone(&fixture.system_ui);
            let engine = fixture.engine();
            scan(&engine);
            gate.close();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();
            while !matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanStarted { .. }) {}

            let refused = engine.reveal_node(NodeId::new(StateId::new(1), 0));

            assert_eq!(refused, Err(CommandError::Busy { running: Operation::Scan }));
            assert_eq!(system_ui.calls(), []);
            engine.cancel_scan().unwrap();
        }

        #[test]
        fn reveal_of_a_node_of_another_state_is_refused() {
            let (engine, system_ui) = with_home(FakeSystemUi::default());

            assert_eq!(engine.reveal_node(NodeId::new(StateId::new(9), 1)), Err(CommandError::StaleNode));
            assert_eq!(engine.reveal_node(NodeId::new(StateId::new(1), 9_999)), Err(CommandError::StaleNode));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn reveal_of_the_home_or_outside_it_is_refused() {
            let (engine, system_ui) = with_home(FakeSystemUi::default());

            assert_eq!(engine.reveal_node(node(&engine, "/home")), Err(CommandError::NotOwnItem));
            assert_eq!(engine.reveal_node(node(&engine, "/other")), Err(CommandError::NotOwnItem));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn reveal_of_a_removed_path_is_refused() {
            let (engine, system_ui) = with_home(FakeSystemUi::default());
            let file = node(&engine, "/home/Docs/a.bin");
            let picked = engine.pick_own_item(file).unwrap();
            engine.shared.results.lock().unwrap().remove(&[picked.id()], &[picked.id()], &[]);

            assert_eq!(engine.reveal_node(file), Err(CommandError::NotOwnItem));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn a_failed_reveal_of_a_node_is_reported() {
            let failure = SystemUiError::Failed { reason: "open exited with 1".into() };
            let (engine, system_ui) = with_home(FakeSystemUi::failing(failure));

            assert_eq!(engine.reveal_node(node(&engine, "/home/Docs")), Err(CommandError::Internal));
            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal("/home/Docs".into())]);
        }

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

        use super::{node, scanned, with_home};

        #[test]
        fn show_in_finder_reveals_the_nodes_path() {
            let (engine, system_ui) = with_home(FakeSystemUi::default());

            engine.reveal_node(node(&engine, "/home/Docs")).unwrap();

            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal(PathBuf::from("/home/Docs"))]);
        }

        #[test]
        fn show_in_finder_reveals_the_findings_path() {
            let (engine, system_ui) = scanned(FakeSystemUi::default());

            engine.reveal_finding(FindingId::new(1)).unwrap();

            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal(PathBuf::from("/home/Caches"))]);
        }
    }
}
