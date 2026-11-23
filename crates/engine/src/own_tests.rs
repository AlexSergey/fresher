use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CategoryId, DangerLevel, DiskState, Finding, ModuleId, NodeId, NodeKind, NotScanned, Sizes, StateId,
    Timestamp,
};
use fresher_testing::FixedDiskState;

use crate::Engine;
use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

const MB: u64 = 1_000_000;

fn caches() -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), "alpha.caches").unwrap()
}

fn mb(size: u64) -> Sizes {
    Sizes::complete(ByteSize::new(size * MB), ByteSize::new(size * MB))
}

/// A state `id` of `/home`: a claimed `Caches`, a folder holding a claimed one, Documents, Library, a hidden file,
/// Downloads with a movie, and a project folder with an archive in it.
fn home(id: u32) -> FixedDiskState {
    let nodes = [
        ("/home", NodeKind::Folder, 2_000),
        ("/home/Caches", NodeKind::Folder, 300),
        ("/home/Caches/blob.bin", NodeKind::File, 300),
        ("/home/Work", NodeKind::Folder, 200),
        ("/home/Work/node_modules", NodeKind::Folder, 200),
        ("/home/Documents", NodeKind::Folder, 100),
        ("/home/Documents/report.pdf", NodeKind::File, 100),
        ("/home/Library", NodeKind::Folder, 400),
        ("/home/Library/big.bin", NodeKind::File, 400),
        ("/home/.big", NodeKind::File, 150),
        ("/home/Downloads", NodeKind::Folder, 500),
        ("/home/Downloads/movie.mov", NodeKind::File, 500),
        ("/home/Projects", NodeKind::Folder, 350),
        ("/home/Projects/old", NodeKind::Folder, 350),
        ("/home/Projects/old/build.tar", NodeKind::File, 350),
    ];
    FixedDiskState::new(
        StateId::new(id),
        Timestamp::from_secs(1_000),
        nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, mb(size))),
    )
    .with_owner(Path::new("/home/Caches"), caches())
    .with_owner(Path::new("/home/Work/node_modules"), caches())
}

/// A state of `/home` with a folder that holds a mount point of another volume, which the scan did not enter.
fn home_with_volume() -> FixedDiskState {
    let nodes = [
        ("/home", NodeKind::Folder, 900),
        ("/home/Mount", NodeKind::Folder, 900),
        ("/home/Mount/vol", NodeKind::Folder, 0),
        ("/home/Mount/note.bin", NodeKind::File, 900),
    ];
    FixedDiskState::new(
        StateId::new(1),
        Timestamp::from_secs(1_000),
        nodes.map(|(path, kind, size)| (PathBuf::from(path), kind, mb(size))),
    )
    .with_not_scanned(Path::new("/home/Mount/vol"), NotScanned::OtherVolume)
}

/// The module `alpha` claiming every `Caches` folder, over a walk that finds `/home/Caches` (the scan's one finding)
/// and commits `states` in turn.
fn fixture(states: Vec<Arc<dyn DiskState>>) -> Fixture {
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
    let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(10)).committing(states);
    Fixture::new(walk).module(alpha)
}

/// An engine of `fixture` that scanned `home(1)` once.
fn scanned_fixture(fixture: Fixture) -> Engine {
    let engine = fixture.engine();
    scan(&engine);
    engine
}

fn scanned() -> Engine {
    scanned_fixture(fixture(vec![Arc::new(home(1))]))
}

/// The node at `path` of the engine's last state.
fn node(engine: &Engine, path: &str) -> NodeId {
    let state = engine.shared.results.lock().unwrap().state().unwrap();
    state.find(Path::new(path)).unwrap()
}

fn pick(engine: &Engine, path: &str) -> Finding {
    engine.pick_own_item(node(engine, path)).unwrap()
}

mod negative {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

    use fresher_core::{CommandError, Event, IgnoreTarget, NodeId, OWN_FILE_ID, Operation, Protection, StateId};
    use serde_json::json;

    use super::{fixture, home, home_with_volume, node, pick, scanned, scanned_fixture};
    use crate::cleanup::testing::{echo, select};
    use crate::testing::{Fixture, MemorySettings, Protecting, ScriptedWalk, engine, scan};

    #[test]
    fn nothing_is_picked_before_a_scan() {
        let engine = engine(ScriptedWalk::new());

        assert_eq!(engine.pick_own_item(NodeId::new(StateId::new(1), 0)), Err(CommandError::NoScan));
    }

    #[test]
    fn nothing_is_picked_while_a_scan_runs() {
        let (walk, gate) = ScriptedWalk::new().committing(vec![Arc::new(home(1))]).hold();
        gate.release();
        let engine = Fixture::new(walk).engine();
        scan(&engine);
        let movie = node(&engine, "/home/Downloads/movie.mov");
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        while !matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanStarted { .. }) {}

        assert_eq!(engine.pick_own_item(movie), Err(CommandError::Busy { running: Operation::Scan }));
        engine.cancel_scan().unwrap();
    }

    #[test]
    fn a_node_of_another_state_is_stale() {
        let engine = scanned();

        assert_eq!(engine.pick_own_item(NodeId::new(StateId::new(9), 1)), Err(CommandError::StaleNode));
        assert_eq!(engine.pick_own_item(NodeId::new(StateId::new(1), 9_999)), Err(CommandError::StaleNode));
    }

    #[test]
    fn a_claimed_node_cannot_be_picked() {
        let engine = scanned();

        for path in ["/home/Caches", "/home/Caches/blob.bin", "/home/Work/node_modules"] {
            assert_eq!(engine.pick_own_item(node(&engine, path)), Err(CommandError::NotOwnItem), "{path}");
        }
    }

    #[test]
    fn a_standard_folder_cannot_be_picked() {
        let protection = Protection::new(Vec::new(), vec![PathBuf::from("/home/Documents")], Vec::new()).unwrap();
        let engine = scanned_fixture(fixture(vec![Arc::new(home(1))]).cleaner(Arc::new(Protecting::new(protection))));

        assert_eq!(engine.pick_own_item(node(&engine, "/home/Documents")), Err(CommandError::NotOwnItem));
        assert!(engine.pick_own_item(node(&engine, "/home/Documents/report.pdf")).is_ok());
    }

    #[test]
    fn a_folder_holding_another_volume_is_refused_through_protection() {
        let protection = Protection::new(vec![PathBuf::from("/home/Mount/vol")], Vec::new(), Vec::new()).unwrap();
        let fixture = fixture(vec![Arc::new(home_with_volume())]).cleaner(Arc::new(Protecting::new(protection)));
        let engine = scanned_fixture(fixture);

        assert_eq!(engine.pick_own_item(node(&engine, "/home/Mount")), Err(CommandError::NotOwnItem));
        assert_eq!(engine.pick_own_item(node(&engine, "/home/Mount/vol")), Err(CommandError::NotOwnItem));
    }

    #[test]
    fn the_home_itself_cannot_be_picked() {
        let engine = scanned();

        assert_eq!(engine.pick_own_item(node(&engine, "/home")), Err(CommandError::NotOwnItem));
    }

    #[test]
    fn the_library_cannot_be_picked() {
        let engine = scanned();

        for path in ["/home/Library", "/home/Library/big.bin"] {
            assert_eq!(engine.pick_own_item(node(&engine, path)), Err(CommandError::NotOwnItem), "{path}");
        }
    }

    #[test]
    fn a_hidden_file_cannot_be_picked() {
        let engine = scanned();

        assert_eq!(engine.pick_own_item(node(&engine, "/home/.big")), Err(CommandError::NotOwnItem));
    }

    #[test]
    fn an_ignored_place_cannot_be_picked() {
        let settings = MemorySettings::with(&[("core.ignore.paths", json!(["/home/Projects/old"]))]);
        let engine = scanned_fixture(fixture(vec![Arc::new(home(1))]).settings(settings));

        for path in ["/home/Projects", "/home/Projects/old", "/home/Projects/old/build.tar"] {
            assert_eq!(engine.pick_own_item(node(&engine, path)), Err(CommandError::NotOwnItem), "{path}");
        }
    }

    #[test]
    fn a_folder_holding_explained_bytes_cannot_be_picked() {
        let engine = scanned();

        assert_eq!(engine.pick_own_item(node(&engine, "/home/Work")), Err(CommandError::NotOwnItem));
    }

    #[test]
    fn a_pick_ignored_since_it_was_made_is_not_planned() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        engine.ignore(&IgnoreTarget::Finding { finding: movie.id() }).unwrap();

        assert_eq!(
            engine.prepare_cleanup(&select(&[movie.id().get()], &[])),
            Err(CommandError::UnknownFinding { id: movie.id() })
        );
    }

    #[test]
    fn a_pick_no_longer_offered_at_the_confirmation_is_refused() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        let view = engine.prepare_cleanup(&select(&[movie.id().get()], &[])).unwrap();
        engine.shared.results.lock().unwrap().remove(&[movie.id()], &[movie.id()], &[]);

        assert_eq!(engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
    }

    #[test]
    fn an_ignore_added_after_the_prepare_refuses_the_confirmation() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        let view = engine.prepare_cleanup(&select(&[movie.id().get()], &[])).unwrap();
        engine.ignore(&IgnoreTarget::Finding { finding: movie.id() }).unwrap();

        assert_eq!(engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
    }

    #[test]
    fn an_ignored_pick_is_not_shown_by_its_id() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        engine.ignore(&IgnoreTarget::Finding { finding: movie.id() }).unwrap();

        assert_eq!(engine.finding(movie.id()), Err(CommandError::UnknownFinding { id: movie.id() }));
    }

    #[test]
    fn a_category_pick_of_the_own_category_is_unknown() {
        let engine = scanned();
        pick(&engine, "/home/Downloads/movie.mov");

        assert_eq!(
            engine.prepare_cleanup(&select(&[], &[(OWN_FILE_ID, &[])])),
            Err(CommandError::UnknownCategory { id: OWN_FILE_ID.into() })
        );
    }

    #[test]
    fn a_removed_row_cannot_be_picked_again() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        engine.shared.results.lock().unwrap().remove(&[movie.id()], &[movie.id()], &[]);

        assert_eq!(engine.pick_own_item(node(&engine, "/home/Downloads/movie.mov")), Err(CommandError::NotOwnItem));
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::Arc;

    use std::path::Path;

    use fresher_core::{
        DangerLevel, Event, FindingId, LargestQuery, ModuleId, NodeId, NodeInfo, NodeKind, OWN_FILE_ID, Removability,
        Removal, SkipReason, StateId, Subject, TextArg, Unchanged,
    };

    use super::{fixture, home, mb, node, pick, scanned, scanned_fixture};
    use crate::cleanup::testing::{echo, items, select, wait_for_cleanup};
    use crate::own::own_finding;
    use crate::testing::scan;

    const fn query() -> LargestQuery {
        LargestQuery { kind: None, place: None, unchanged: Unchanged::Any, offset: 0, limit: 100 }
    }

    #[test]
    fn picking_a_largest_row_needs_no_acceptance() {
        let engine = scanned_fixture(fixture(vec![Arc::new(home(1))]).unaccepted());

        let movie = pick(&engine, "/home/Downloads/movie.mov");

        assert_eq!(movie.finding().danger(), DangerLevel::High);
        assert!(!engine.agreement().unwrap().is_accepted());
    }

    #[test]
    fn the_finding_is_built_from_plain_values() {
        let node = NodeInfo {
            id: NodeId::new(StateId::new(1), 4),
            name: "Trip".into(),
            kind: NodeKind::Folder,
            sizes: mb(42),
            files: 3,
            folders: 1,
            owner: None,
            modified: None,
            not_scanned: None,
        };

        let trip = own_finding(FindingId::new(7), Path::new("/u"), Path::new("/u/Movies/Trip"), &node);

        assert_eq!(trip.id(), FindingId::new(7));
        assert_eq!(trip.finding().key().identity(), "own/Movies/Trip");
        assert_eq!(trip.finding().sizes(), mb(42));
        assert_eq!(trip.finding().title().args().get("name"), Some(&TextArg::Text("Trip".into())));
        assert_eq!(trip.finding().reason().key().as_str(), "core.own-file.reason.folder");
        assert_eq!(trip.finding().danger(), DangerLevel::High);
    }

    #[test]
    fn a_picked_row_is_marked_in_files_and_folders() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");

        let files = engine.largest(&query()).unwrap();
        let folder = engine.largest_place(Some(node(&engine, "/home/Downloads")), 0, 10).unwrap();

        let marked = files.rows.iter().find(|row| row.node.name == "movie.mov").map(|row| row.picked);
        assert_eq!(marked, Some(Some(movie.id())));
        assert_eq!(folder.rows.iter().map(|row| row.picked).collect::<Vec<_>>(), [Some(movie.id())]);
        assert!(files.rows.iter().filter(|row| row.node.name != "movie.mov").all(|row| row.picked.is_none()));
    }

    #[test]
    fn a_picked_row_goes_through_the_plan_at_high() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");

        let view = engine.prepare_cleanup(&select(&[movie.id().get()], &[])).unwrap();

        assert_eq!(view.summary.highest, DangerLevel::High);
        assert_eq!(view.groups.len(), 1);
        assert_eq!(view.groups[0].category.as_str(), OWN_FILE_ID);
        assert_eq!(items(&view), [movie.id().get()]);
    }

    #[test]
    fn a_picked_row_beside_a_scan_finding_goes_in_one_plan() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");

        let view = engine.prepare_cleanup(&select(&[1, movie.id().get()], &[])).unwrap();

        let mut planned = items(&view);
        planned.sort_unstable();
        assert_eq!(planned, [1, movie.id().get()]);
        assert_eq!(view.summary.highest, DangerLevel::High);
    }

    #[test]
    fn a_picked_row_inside_a_picked_folder_is_covered() {
        let engine = scanned();
        let old = pick(&engine, "/home/Projects/old");
        let tar = pick(&engine, "/home/Projects/old/build.tar");

        let view = engine.prepare_cleanup(&select(&[tar.id().get(), old.id().get()], &[])).unwrap();

        assert_eq!(items(&view), [old.id().get()]);
        assert_eq!(view.left_out.len(), 1);
        assert_eq!(view.left_out[0].finding(), tar.id());
        assert_eq!(view.left_out[0].reason(), &SkipReason::Covered);
    }

    #[test]
    fn a_removed_pick_is_named_by_findings_removed() {
        let engine = scanned();
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        let view = engine.prepare_cleanup(&select(&[movie.id().get()], &[])).unwrap();
        let events = engine.subscribe();

        engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
        let seen = wait_for_cleanup(&events);

        let removed = seen.iter().find_map(|event| match event {
            Event::FindingsRemoved { findings, .. } => Some(findings.clone()),
            _ => None,
        });
        assert_eq!(removed, Some(vec![movie.id()]));
        assert!(engine.largest(&query()).unwrap().rows.iter().all(|row| row.node.name != "movie.mov"));
        let downloads = engine.largest_place(Some(node(&engine, "/home/Downloads")), 0, 10).unwrap();
        assert_eq!((downloads.rows.len(), downloads.total), (0, 0));
        assert_eq!(engine.finding(movie.id()), Err(fresher_core::CommandError::UnknownFinding { id: movie.id() }));
    }

    #[test]
    fn a_folder_with_removed_rows_pages_without_a_repeat_or_a_gap() {
        let mut nodes =
            vec![("/home".to_string(), NodeKind::Folder, 9_000), ("/home/Many".into(), NodeKind::Folder, 9_000)];
        nodes.extend((0..250).map(|n| (format!("/home/Many/f{n:03}.bin"), NodeKind::File, 1_000 - n)));
        let state = fresher_testing::FixedDiskState::new(
            StateId::new(1),
            fresher_core::Timestamp::from_secs(1_000),
            nodes.into_iter().map(|(path, kind, size)| (PathBuf::from(path), kind, mb(size))),
        );
        let engine = scanned_fixture(fixture(vec![Arc::new(state)]));
        let many = node(&engine, "/home/Many");
        let removed = [pick(&engine, "/home/Many/f003.bin"), pick(&engine, "/home/Many/f150.bin")];
        let ids: Vec<_> = removed.iter().map(fresher_core::Finding::id).collect();
        engine.shared.results.lock().unwrap().remove(&ids, &ids, &[]);

        let mut names = Vec::new();
        let total = loop {
            let page = engine.largest_place(Some(many), u32::try_from(names.len()).unwrap(), 100).unwrap();
            if page.rows.is_empty() {
                break page.total;
            }
            names.extend(page.rows.into_iter().map(|row| row.node.name));
        };

        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!((names.len(), unique.len(), total), (248, 248, 248));
        assert!(!names.contains(&"f003.bin".to_string()) && !names.contains(&"f150.bin".to_string()));
    }

    #[test]
    fn a_cleanup_of_a_pick_records_its_category_in_the_journal() {
        let fixture = fixture(vec![Arc::new(home(1))]);
        let journal = Arc::clone(&fixture.journal);
        let engine = fixture.engine();
        scan(&engine);
        let movie = pick(&engine, "/home/Downloads/movie.mov");
        let view = engine.prepare_cleanup(&select(&[movie.id().get()], &[])).unwrap();
        let events = engine.subscribe();

        engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
        wait_for_cleanup(&events);

        let recorded = &journal.recorded()[0];
        assert_eq!(recorded.begin.items.len(), 1);
        let item = &recorded.begin.items[0];
        assert_eq!(item.category, OWN_FILE_ID);
        assert_eq!(item.danger, DangerLevel::High);
        assert_eq!(item.key, "own/Downloads/movie.mov");
        assert_eq!(item.path.as_deref(), Some(Path::new("/home/Downloads/movie.mov")));
    }

    #[test]
    fn a_picked_file_is_an_own_file_finding_at_high() {
        let engine = scanned();

        let movie = pick(&engine, "/home/Downloads/movie.mov");

        let path = PathBuf::from("/home/Downloads/movie.mov");
        let new = movie.finding();
        assert_eq!(movie.module(), ModuleId::system());
        assert_eq!(movie.removability(), Removability::Removable);
        assert_eq!(new.category().as_str(), OWN_FILE_ID);
        assert_eq!(new.danger(), DangerLevel::High);
        assert_eq!(new.key().identity(), "own/Downloads/movie.mov");
        assert_eq!(new.subject(), &Subject::Path(path.clone()));
        assert_eq!(new.removal(), &Removal::MoveToTrash(path));
        assert_eq!(new.sizes(), mb(500));
        assert_eq!(new.title().key().as_str(), "core.own-file.title");
        assert_eq!(new.title().args().get("name"), Some(&TextArg::Text("movie.mov".into())));
        assert_eq!(new.reason().key().as_str(), "core.own-file.reason.file");
    }

    #[test]
    fn a_picked_folder_reads_as_a_folder() {
        let engine = scanned();

        let old = pick(&engine, "/home/Projects/old");

        assert_eq!(old.finding().reason().key().as_str(), "core.own-file.reason.folder");
        assert_eq!(old.finding().removal(), &Removal::MoveToTrash(PathBuf::from("/home/Projects/old")));
    }

    #[test]
    fn picking_twice_answers_one_finding() {
        let engine = scanned();

        let first = pick(&engine, "/home/Downloads/movie.mov");
        let second = pick(&engine, "/home/Downloads/movie.mov");

        assert_eq!(first, second);
        assert_eq!(engine.shared.results.lock().unwrap().own().len(), 1);
    }

    #[test]
    fn a_picked_id_follows_the_scan_s_ids() {
        let engine = scanned();

        let movie = pick(&engine, "/home/Downloads/movie.mov");
        let tar = pick(&engine, "/home/Projects/old/build.tar");

        assert_eq!((movie.id(), tar.id()), (FindingId::new(2), FindingId::new(3)));
        assert_eq!(engine.finding(FindingId::new(2)), Ok(movie));
    }

    #[test]
    fn the_next_scan_s_ids_follow_the_picks() {
        let engine = fixture(vec![Arc::new(home(1)), Arc::new(home(2))]).engine();
        scan(&engine);
        pick(&engine, "/home/Downloads/movie.mov");

        scan(&engine);

        let (findings, own) = {
            let results = engine.shared.results.lock().unwrap();
            (results.findings().iter().map(|finding| finding.id().get()).collect::<Vec<_>>(), results.own().len())
        };
        assert_eq!((findings, own), (vec![3], 0));
        assert_eq!(pick(&engine, "/home/Downloads/movie.mov").id(), FindingId::new(4));
    }
}
