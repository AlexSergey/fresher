use std::sync::Arc;

use fresher_core::{
    Access, AppId, Capability, CleanupPlanView, DangerLevel, Event, FindingId, InstalledApp, NodeKind, StateId,
};
use fresher_testing::{FakeAccess, FakeCleaner, FakeInstalledApps};

use super::super::fixture::{app, at, facts, idle, state, system};
use crate::Engine;
use crate::cleanup::testing::{echo, wait_for_cleanup};
use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

const EDITOR: AppId = AppId::new(StateId::new(1), 0);
const VIEWER: AppId = AppId::new(StateId::new(1), 1);
const CACHES: &str = "/home/Library/Caches/com.example.Editor";
/// The uninstall's finding: the next id after the scan's one finding.
const UNINSTALL: FindingId = FindingId::new(2);

fn editor() -> InstalledApp {
    app("com.example.Editor", "Editor")
}

fn viewer() -> InstalledApp {
    app("com.example.Viewer", "Viewer")
}

/// The editor and the viewer installed, 100 bytes each, the editor's caches found by module `alpha` as finding 1;
/// `cleaner` cleans. After one scan.
fn scanned(cleaner: FakeCleaner) -> (Engine, Arc<FakeCleaner>) {
    let apps = FakeInstalledApps::new(vec![editor(), viewer()])
        .with_bundle(editor().path, Ok(facts(100, None)))
        .with_bundle(viewer().path, Ok(facts(100, None)));
    let home = state(1, &[(CACHES, NodeKind::Folder, 10)]);
    let walk = ScriptedWalk::new().folder("com.example.Editor", CACHES, bytes(10)).committing(vec![Arc::new(home)]);
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("com.example.Editor");
    let cleaner = Arc::new(cleaner);
    let engine = Fixture::new(walk)
        .system(system(Arc::new(apps), idle()))
        .access(FakeAccess::with(&[(Capability::OtherAppsData, Access::Granted)]))
        .module(alpha)
        .cleaner(Arc::clone(&cleaner))
        .engine();
    scan(&engine);
    (engine, cleaner)
}

/// The editor installed in the user's own apps folder, 100 bytes, `cleaner` cleaning; after one scan.
fn in_own_apps(cleaner: FakeCleaner) -> Engine {
    let editor = at(editor(), "/home/Applications/Editor.app");
    let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(editor.path, Ok(facts(100, None)));
    let home = state(1, &[(CACHES, NodeKind::Folder, 10)]);
    let walk = ScriptedWalk::new().folder("com.example.Editor", CACHES, bytes(10)).committing(vec![Arc::new(home)]);
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("com.example.Editor");
    let engine = Fixture::new(walk)
        .system(system(Arc::new(apps), idle()))
        .access(FakeAccess::with(&[(Capability::OtherAppsData, Access::Granted)]))
        .module(alpha)
        .cleaner(Arc::new(cleaner))
        .engine();
    scan(&engine);
    engine
}

fn management(engine: &Engine) -> Access {
    engine.access_status().access(&Capability::AppManagement)
}

/// Confirms `view` to the Trash; the events up to the cleanup's end.
fn confirm(engine: &Engine, view: &CleanupPlanView) -> Vec<Event> {
    let events = engine.subscribe();
    engine.confirm_cleanup(&echo(view, "trash")).unwrap();
    wait_for_cleanup(&events)
}

/// The App Management states the `AccessChanged` events of `events` carry, in order.
fn learned(events: &[Event]) -> Vec<Access> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::AccessChanged { status } => Some(status.access(&Capability::AppManagement)),
            _ => None,
        })
        .collect()
}

mod negative {
    use fresher_core::{Access, CommandError, Failure, FindingId, ItemOutcome, PartlyWay, SkipReason, UninstallChoice};
    use fresher_testing::FakeCleaner;

    use super::super::Management;
    use super::{EDITOR, UNINSTALL, confirm, in_own_apps, learned, management, scanned};
    use crate::cleanup::testing::{echo, select};

    #[test]
    fn it_is_not_asked_until_an_uninstall() {
        let (engine, cleaner) = scanned(FakeCleaner::default());

        engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        assert_eq!(Management::default().access(), Access::NotAsked);
        assert_eq!(management(&engine), Access::NotAsked);
        assert!(cleaner.executed().is_empty(), "nothing is tried to tell it");
    }

    #[test]
    fn a_skipped_or_cancelled_uninstall_teaches_nothing() {
        let outcomes = [
            ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Editor".into() } },
            ItemOutcome::NotReached,
            ItemOutcome::Deleted,
            ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() },
            ItemOutcome::Failed { reason: Failure::DeleteFailed.text() },
            ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 3, way: PartlyWay::Trash }.text() },
        ];
        let known = Management::default();
        for outcome in &outcomes {
            assert!(!known.learn(outcome, None), "{outcome:?}");
        }
        let partly = ItemOutcome::Failed { reason: Failure::Partly { done: 0, of: 3, way: PartlyWay::Trash }.text() };
        assert!(!known.learn(&partly, Some(0)), "the bundle did not move");
        assert_eq!(known.access(), Access::NotAsked);

        let (skipped, _) = scanned(FakeCleaner::default().skipping(UNINSTALL, SkipReason::Vanished));
        let view = skipped.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        assert_eq!(skipped.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::NothingToClean));
        assert_eq!(management(&skipped), Access::NotAsked);

        let (cancelled, _) = scanned(FakeCleaner::default().ending(UNINSTALL, ItemOutcome::NotReached));
        let view = cancelled.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        assert_eq!(learned(&confirm(&cancelled, &view)), Vec::<Access>::new());
        assert_eq!(management(&cancelled), Access::NotAsked);
    }

    #[test]
    fn an_uninstall_from_the_users_own_apps_folder_teaches_nothing() {
        let refused = ItemOutcome::Failed { reason: Failure::TrashRefused.text() };
        for cleaner in [FakeCleaner::default(), FakeCleaner::default().ending(UNINSTALL, refused)] {
            let engine = in_own_apps(cleaner);
            let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
            assert_eq!(view.groups[0].items[0].finding, UNINSTALL);

            assert_eq!(learned(&confirm(&engine, &view)), Vec::<Access>::new());
            assert_eq!(management(&engine), Access::NotAsked);
        }
    }

    #[test]
    fn an_ordinary_cleanup_teaches_nothing() {
        let refused = ItemOutcome::Failed { reason: Failure::TrashRefused.text() };
        let (refusing, _) = scanned(FakeCleaner::default().ending(FindingId::new(1), refused));
        let (moving, _) = scanned(FakeCleaner::default());

        for engine in [&refusing, &moving] {
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();

            assert_eq!(learned(&confirm(engine, &view)), Vec::<Access>::new());
            assert_eq!(management(engine), Access::NotAsked);
        }
    }
}

mod positive {
    use fresher_core::{Access, Event, Failure, ItemOutcome, PartlyWay, UninstallChoice};
    use fresher_testing::FakeCleaner;

    use super::super::Management;
    use super::{EDITOR, UNINSTALL, VIEWER, confirm, learned, management, scanned};

    #[test]
    fn a_refused_move_of_the_bundle_is_denied() {
        let refused = ItemOutcome::Failed { reason: Failure::TrashRefused.text() };
        let (engine, cleaner) = scanned(FakeCleaner::default().ending(UNINSTALL, refused.clone()));
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        assert_eq!(view.groups[0].items[0].finding, UNINSTALL);

        let events = confirm(&engine, &view);

        assert_eq!(learned(&events), [Access::Denied]);
        assert_eq!(management(&engine), Access::Denied);
        assert_eq!(cleaner.executed().len(), 1, "the refused move is never tried again");
        assert!(Management::default().learn(&refused, None));
    }

    #[test]
    fn a_moved_bundle_is_granted() {
        let (engine, _) = scanned(FakeCleaner::default());
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        confirm(&engine, &view);

        assert_eq!(management(&engine), Access::Granted);
    }

    #[test]
    fn a_partly_done_uninstall_whose_bundle_moved_is_granted() {
        let (engine, _) = scanned(FakeCleaner::default().partly(UNINSTALL, 1));
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let events = confirm(&engine, &view);

        assert_eq!(learned(&events), [Access::Granted]);
        assert_eq!(management(&engine), Access::Granted);
        let partly = ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 3, way: PartlyWay::Trash }.text() };
        assert!(Management::default().learn(&partly, Some(1)));
    }

    #[test]
    fn the_change_is_published_once() {
        let (engine, _) = scanned(FakeCleaner::default());
        let first = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        let events = confirm(&engine, &first);
        let changed = events.iter().position(|event| matches!(event, Event::AccessChanged { .. }));
        let finished = events.iter().position(|event| matches!(event, Event::CleanupFinished { .. }));

        let second = engine.prepare_uninstall(VIEWER, UninstallChoice::KeepData).unwrap();
        let again = confirm(&engine, &second);

        assert_eq!(learned(&events), [Access::Granted]);
        assert!(changed < finished, "the state is known before the cleanup ends");
        assert_eq!(learned(&again), Vec::<Access>::new());
        assert_eq!(management(&engine), Access::Granted);
    }
}
