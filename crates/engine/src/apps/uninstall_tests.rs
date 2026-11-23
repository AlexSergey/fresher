use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    Access, AppId, BundleFacts, CancelToken, Capability, Cleaner, CleanupContext, CleanupPlanView, ConfirmedPlan,
    DangerLevel, Depth, Ended, FindingId, InstalledApp, ItemDone, NodeKind, PlanItem, Protected, PutBackOutcome,
    Rehearsal, RunningApp, RunningApps, StateId, TrashState, TrashedItem,
};
use fresher_testing::{FakeAccess, FakeCleaner, FakeInstalledApps, FixedDiskState, running_app};

use super::super::fixture::{app, facts, idle, state, system};
use crate::Engine;
use crate::cleanup::testing::{echo, wait_for_cleanup};
use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

const EDITOR: AppId = AppId::new(StateId::new(1), 0);
const BUNDLE: &str = "/Applications/Editor.app";
const CACHES: &str = "/home/Library/Caches/com.example.Editor";

fn editor() -> InstalledApp {
    app("com.example.Editor", "Editor")
}

/// The editor's files, one of each kind but the group container, and `more`.
fn home(more: &[(&str, NodeKind, u64)]) -> FixedDiskState {
    let mut entries = vec![
        (CACHES, NodeKind::Folder, 10),
        ("/home/Library/Logs/com.example.Editor", NodeKind::Folder, 1),
        ("/home/Library/Saved Application State/com.example.Editor.savedState", NodeKind::Folder, 2),
        ("/home/Library/LaunchAgents/com.example.Editor.plist", NodeKind::File, 1),
        ("/home/Library/HTTPStorages/com.example.Editor", NodeKind::Folder, 4),
        ("/home/Library/Preferences/com.example.Editor.plist", NodeKind::File, 3),
        ("/home/Library/Application Support/com.example.Editor", NodeKind::Folder, 20),
        ("/home/Library/Containers/com.example.Editor", NodeKind::Folder, 30),
    ];
    entries.extend_from_slice(more);
    state(1, &entries)
}

/// The editor installed, its bundle's facts `bundle`.
fn installed(bundle: BundleFacts) -> FakeInstalledApps {
    FakeInstalledApps::new(vec![editor()]).with_bundle(editor().path, Ok(bundle))
}

/// What an uninstall test is made of.
struct World {
    state: FixedDiskState,
    apps: FakeInstalledApps,
    running: Arc<dyn RunningApps>,
    cleaner: Arc<dyn Cleaner>,
    /// A module `alpha` finds the editor's caches folder as `alpha.caches`, finding 1.
    caches_finding: bool,
}

impl World {
    /// The editor's home, its bundle of 100 bytes, nothing running, the fake cleaner.
    fn new() -> Self {
        Self {
            state: home(&[]),
            apps: installed(facts(100, None)),
            running: idle(),
            cleaner: Arc::new(FakeCleaner::default()),
            caches_finding: false,
        }
    }

    fn fixture(self) -> Fixture {
        let mut walk = ScriptedWalk::new();
        if self.caches_finding {
            walk = walk.folder("com.example.Editor", CACHES, bytes(10));
        }
        let walk = walk.committing(vec![Arc::new(self.state)]);
        let mut fixture = Fixture::new(walk)
            .system(system(Arc::new(self.apps), self.running))
            .access(FakeAccess::with(&[(Capability::OtherAppsData, Access::Granted)]))
            .cleaner(Arc::new(Wrapped(self.cleaner)));
        if self.caches_finding {
            let alpha =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("com.example.Editor");
            fixture = fixture.module(alpha);
        }
        fixture
    }

    /// The engine of the world after one scan.
    fn scanned(self) -> Engine {
        scanned(self.fixture())
    }
}

fn scanned(fixture: Fixture) -> Engine {
    let engine = fixture.engine();
    scan(&engine);
    engine
}

/// A cleaner the test keeps a handle on.
struct Wrapped(Arc<dyn Cleaner>);

impl Cleaner for Wrapped {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        self.0.rehearse(roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        self.0.execute(plan, cx, cancel, out)
    }

    fn protection(&self) -> fresher_core::Protection {
        self.0.protection()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.0.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.0.put_back(item, cx)
    }
}

/// The fake cleaner, noting what the context's protection answers for the editor's bundle and the context's roots, at
/// each rehearsal and each execution.
#[derive(Debug, Default)]
struct Probe {
    inner: FakeCleaner,
    answers: Mutex<Vec<Result<(), Protected>>>,
    roots: Mutex<Vec<Vec<PathBuf>>>,
}

impl Probe {
    fn note(&self, cx: &CleanupContext<'_>) {
        self.answers.lock().unwrap().push(cx.protection.check(Path::new(BUNDLE)));
        self.roots.lock().unwrap().push(cx.roots.to_vec());
    }

    fn answers(&self) -> Vec<Result<(), Protected>> {
        self.answers.lock().unwrap().clone()
    }
}

impl Cleaner for Probe {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        self.note(cx);
        self.inner.rehearse(roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        self.note(cx);
        self.inner.execute(plan, cx, cancel, out)
    }

    fn protection(&self) -> fresher_core::Protection {
        self.inner.protection()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.inner.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.inner.put_back(item, cx)
    }
}

/// The paths of the pending plan's one item, the bundle first.
fn planned(engine: &Engine) -> Vec<PathBuf> {
    let pending = engine.shared.pending.lock().unwrap();
    let (plan, _) = pending.pending().unwrap();
    assert_eq!(plan.items().len(), 1, "an uninstall is one item");
    plan.items()[0].action().parts().to_vec()
}

fn has_pending(engine: &Engine) -> bool {
    engine.shared.pending.lock().unwrap().pending().is_some()
}

/// The paths below `/home/Library`.
fn library(paths: &[&str]) -> Vec<PathBuf> {
    paths.iter().map(|path| PathBuf::from("/home/Library").join(path)).collect()
}

/// Confirms `view` to the Trash and waits for the cleanup's end.
fn confirm(engine: &Engine, view: &CleanupPlanView) {
    let events = engine.subscribe();
    engine.confirm_cleanup(&echo(view, "trash")).unwrap();
    wait_for_cleanup(&events);
}

/// The ids of the findings on Scan.
fn shown(engine: &Engine) -> Vec<FindingId> {
    engine.shared.results.lock().unwrap().visible().iter().map(fresher_core::Finding::id).collect()
}

/// The names of the Apps list's rows.
fn listed(engine: &Engine) -> Vec<String> {
    engine.apps(fresher_core::AppsQuery::default()).unwrap().apps.into_iter().map(|row| row.name).collect()
}

fn opened(ids: &[&str]) -> Vec<RunningApp> {
    vec![running_app("Editor", ids).unwrap()]
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{
        Access, AppId, AppOffer, BundleFacts, Capability, CommandError, EntryFacts, Failure, FindingId, InstalledBy,
        ItemOutcome, KeptReason, Manager, NodeKind, NotOffered, Operation, Protected, SkipReason, StateId,
        UninstallChoice,
    };
    use fresher_testing::{FakeAccess, FakeCleaner, FakeInstalledApps, FakeRunningApps};
    use serde_json::json;

    use super::super::super::fixture::facts;
    use super::{
        BUNDLE, CACHES, EDITOR, Probe, World, confirm, editor, has_pending, home, installed, library, listed, opened,
        planned, scanned, shown, state,
    };
    use crate::cleanup::testing::{echo, select};
    use crate::testing::{MemorySettings, ScriptedWalk, engine, scan, wait_for_finish};

    #[test]
    fn prepare_uninstall_while_scanning_is_busy() {
        let (walk, gate) = ScriptedWalk::held();
        let engine = engine(walk);
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        let error = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);

        assert_eq!(error, Err(CommandError::Busy { running: Operation::Scan }));
        gate.release();
        wait_for_finish(&events);
    }

    #[test]
    fn an_unknown_app_is_refused() {
        let engine = World::new().scanned();

        for id in [AppId::new(StateId::new(9), 0), AppId::new(StateId::new(1), 1)] {
            assert_eq!(engine.prepare_uninstall(id, UninstallChoice::KeepData), Err(CommandError::UnknownApp { id }));
        }
        assert!(!has_pending(&engine));
    }

    #[test]
    fn an_app_not_offered_is_refused_with_its_reason() {
        let homebrew = Some(InstalledBy::new(Manager::Homebrew, "editor").unwrap());
        let root_owned = BundleFacts { entry: EntryFacts::default(), ..facts(100, None) };
        let cases = [
            (FakeInstalledApps::new(vec![editor()]), NotOffered::Unmeasured),
            (installed(BundleFacts { installed_by: homebrew, ..facts(100, None) }), NotOffered::InstalledBy),
            (installed(root_owned), NotOffered::NeedsAdministrator),
        ];
        for (apps, reason) in cases {
            let engine = World { apps, ..World::new() }.scanned();

            let refused = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);

            assert_eq!(refused, Err(CommandError::UninstallRefused { reason }));
            assert!(!has_pending(&engine));
        }
    }

    #[test]
    fn an_uninstall_without_the_agreement_is_blocked_and_holds_no_plan() {
        let engine = scanned(World::new().fixture().unaccepted());

        let refused = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);
        let unknown = AppId::new(StateId::new(9), 0);

        assert_eq!(refused, Err(CommandError::CleaningBlocked { reason: fresher_core::BlockedReason::Agreement }));
        assert!(!has_pending(&engine));
        assert_eq!(
            engine.prepare_uninstall(unknown, UninstallChoice::KeepData),
            Err(CommandError::UnknownApp { id: unknown })
        );
        let unmeasured =
            scanned(World { apps: FakeInstalledApps::new(vec![editor()]), ..World::new() }.fixture().unaccepted());
        assert_eq!(
            unmeasured.prepare_uninstall(EDITOR, UninstallChoice::KeepData),
            Err(CommandError::UninstallRefused { reason: NotOffered::Unmeasured })
        );
    }

    #[test]
    fn an_app_opened_since_the_list_is_refused_open() {
        let running = FakeRunningApps::default().then(opened(&["com.example.Editor"]));
        let engine = World { running: Arc::new(running), ..World::new() }.scanned();
        let rows = engine.apps(fresher_core::AppsQuery::default()).unwrap().apps;
        assert_eq!(rows[0].offer, AppOffer::Offered);

        let refused = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);

        assert_eq!(refused, Err(CommandError::UninstallRefused { reason: NotOffered::Open }));
    }

    #[test]
    fn an_app_opened_since_the_plan_is_refused_at_its_confirmation() {
        let cleaner = Arc::new(FakeCleaner::default());
        let running = FakeRunningApps::default().then(opened(&["com.example.Editor"]));
        let world = World { running: Arc::new(running), cleaner: Arc::clone(&cleaner) as _, ..World::new() };
        let engine = world.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let refused = engine.confirm_cleanup(&echo(&view, "trash"));

        assert_eq!(refused, Err(CommandError::UninstallRefused { reason: NotOffered::Open }));
        assert_eq!(cleaner.executed(), []);
        assert!(!has_pending(&engine));
        assert_eq!(listed(&engine), ["Editor"]);
    }

    #[test]
    fn a_permanent_uninstall_is_never_offered_and_refused_with_nothing_executed() {
        let cleaner = Arc::new(FakeCleaner::default());
        let engine = World { cleaner: Arc::clone(&cleaner) as _, ..World::new() }.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::Everything).unwrap();
        assert!(!view.permanent_available, "an uninstall always goes to the Trash");

        let refused = engine.confirm_cleanup(&echo(&view, "permanent"));

        assert_eq!(refused, Err(CommandError::PlanMismatch));
        assert_eq!(cleaner.executed(), []);
        assert!(!has_pending(&engine));
        assert_eq!(listed(&engine), ["Editor"]);
        assert_eq!(engine.cleanups(0, 10).total, 0);
    }

    #[test]
    fn a_bundle_not_on_disk_as_listed_is_refused() {
        let apps = installed(facts(100, None)).with_stray_bundle(BUNDLE);
        let engine = World { apps, ..World::new() }.scanned();

        let refused = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);

        assert_eq!(refused, Err(CommandError::UninstallRefused { reason: NotOffered::Unmeasured }));
        assert!(!has_pending(&engine));
    }

    #[test]
    fn more_than_64_paths_is_refused_too_many_files() {
        let names: Vec<String> =
            (0..60).map(|n| format!("/home/Library/LaunchAgents/com.example.Editor.agent{n:02}.plist")).collect();
        let more: Vec<(&str, NodeKind, u64)> = names.iter().map(|name| (name.as_str(), NodeKind::File, 1)).collect();
        let engine = World { state: home(&more), ..World::new() }.scanned();

        // 1 bundle, 4 parts and 60 more agents: 65 paths.
        let refused = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData);

        assert_eq!(refused, Err(CommandError::UninstallRefused { reason: NotOffered::TooManyFiles }));
        assert!(!has_pending(&engine));
    }

    #[test]
    fn a_kept_part_is_never_in_the_plan() {
        let partial = FakeInstalledApps::partial(vec![editor()], &["/Applications/Broken.app"])
            .with_bundle(editor().path, Ok(facts(100, None)));
        let ignore = MemorySettings::with(&[("core.ignore.paths", json!([CACHES]))]);
        let denied = FakeAccess::with(&[(Capability::OtherAppsData, Access::Denied)]);
        let engines = [
            (World { apps: partial, ..World::new() }.scanned(), KeptReason::Shared),
            (World { state: state(1, &[(CACHES, NodeKind::Link, 10)]), ..World::new() }.scanned(), KeptReason::Link),
            (scanned(World::new().fixture().settings(ignore)), KeptReason::Ignored),
            (scanned(World::new().fixture().access(denied)), KeptReason::NeedsPermission),
        ];
        for (engine, reason) in engines {
            let items = engine.app(EDITOR).unwrap().items;
            let is_kept =
                |item: &&fresher_core::FootprintItem| item.kept.as_ref().is_some_and(|kept| kept.reason == reason);
            let kept: Vec<_> = items.iter().filter(is_kept).map(|item| item.path.clone()).collect();
            assert_ne!(kept, Vec::<std::path::PathBuf>::new(), "{reason:?}");

            engine.prepare_uninstall(EDITOR, UninstallChoice::Everything).unwrap();

            let planned = planned(&engine);
            assert!(kept.iter().all(|path| !planned.contains(path)), "{reason:?}: {planned:?}");
            assert_eq!(planned[0], std::path::PathBuf::from(BUNDLE));
        }
    }

    #[test]
    fn keep_data_never_takes_settings_or_support() {
        let engine = World::new().scanned();

        engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let planned = planned(&engine);
        for path in library(&[
            "HTTPStorages/com.example.Editor",
            "Preferences/com.example.Editor.plist",
            "Application Support/com.example.Editor",
            "Containers/com.example.Editor",
        ]) {
            assert!(!planned.contains(&path), "{}", path.display());
        }
    }

    #[test]
    fn a_part_the_rehearsal_refuses_leaves_the_whole_app_out() {
        let uninstall = FindingId::new(1);
        let cleaner = FakeCleaner::default().skipping(uninstall, SkipReason::Vanished);
        let engine = World { cleaner: Arc::new(cleaner), ..World::new() }.scanned();

        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        assert_eq!(view.groups, []);
        assert_eq!(view.left_out.len(), 1);
        assert_eq!(view.left_out[0].finding(), uninstall);
        assert_eq!(view.left_out[0].reason(), &SkipReason::Vanished);
        assert_eq!(engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::NothingToClean));
    }

    #[test]
    fn a_new_scan_kills_the_uninstall_plan() {
        let engine = World::new().scanned();
        engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        scan(&engine);

        assert!(!has_pending(&engine));
    }

    #[test]
    fn the_uninstall_finding_is_never_in_the_results() {
        let engine = World { caches_finding: true, ..World::new() }.scanned();
        let (overview, findings) = (engine.overview().unwrap(), shown(&engine));

        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let uninstall = view.groups[0].items[0].finding;
        assert_eq!(engine.overview().unwrap(), overview);
        assert_eq!(shown(&engine), findings);
        assert_eq!(engine.finding(uninstall), Err(CommandError::UnknownFinding { id: uninstall }));
        let selected = engine.prepare_cleanup(&select(&[uninstall.get()], &[]));
        assert_eq!(selected, Err(CommandError::UnknownFinding { id: uninstall }));
    }

    #[test]
    fn an_ordinary_cleanup_is_rehearsed_and_cleaned_with_the_apps_folder_refused() {
        let probe = Arc::new(Probe::default());
        let engine = World { caches_finding: true, cleaner: Arc::clone(&probe) as _, ..World::new() }.scanned();
        let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();

        confirm(&engine, &view);

        assert_eq!(probe.answers(), [Err(Protected::Never), Err(Protected::Never)]);
    }

    #[test]
    fn a_failed_uninstall_keeps_the_app_and_its_findings() {
        let failed = ItemOutcome::Failed { reason: Failure::TrashRefused.text() };
        let cleaner = FakeCleaner::default().ending(FindingId::new(2), failed);
        let engine = World { caches_finding: true, cleaner: Arc::new(cleaner), ..World::new() }.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        assert_eq!(view.groups[0].items[0].finding, FindingId::new(2));

        confirm(&engine, &view);

        assert_eq!(shown(&engine), [FindingId::new(1)]);
        assert_eq!(listed(&engine), ["Editor"]);
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{Action, ActionKind, CommandError, DangerLevel, FindingId, NodeKind, UninstallChoice};

    use super::{BUNDLE, CACHES, EDITOR, Probe, World, confirm, home, library, listed, planned, shown, state};

    #[test]
    fn keep_data_moves_the_bundle_caches_logs_saved_state_and_agents_together() {
        let engine = World::new().scanned();

        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let mut expected = vec![PathBuf::from(BUNDLE)];
        expected.extend(library(&[
            "Caches/com.example.Editor",
            "Logs/com.example.Editor",
            "Saved Application State/com.example.Editor.savedState",
            "LaunchAgents/com.example.Editor.plist",
        ]));
        assert_eq!(planned(&engine), expected);
        assert_eq!(view.groups.len(), 1);
        assert_eq!(view.groups[0].category.as_str(), "core.uninstall-keep-data");
        assert_eq!(view.groups[0].action, ActionKind::Remove);
        assert_eq!(view.summary.count, 1);
        assert_eq!(view.summary.sizes.logical.bytes(), 114);
        let item = &view.groups[0].items[0];
        assert_eq!(item.title.key().as_str(), "core.uninstall.title");
        assert_eq!(item.reason.key().as_str(), "core.uninstall.reason.keep-data");
        assert_eq!(item.path, Some(PathBuf::from(BUNDLE)));
    }

    #[test]
    fn everything_adds_settings_web_data_and_support() {
        let engine = World::new().scanned();

        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::Everything).unwrap();

        let mut expected = vec![PathBuf::from(BUNDLE)];
        expected.extend(library(&[
            "Caches/com.example.Editor",
            "Logs/com.example.Editor",
            "Saved Application State/com.example.Editor.savedState",
            "LaunchAgents/com.example.Editor.plist",
            "HTTPStorages/com.example.Editor",
            "Preferences/com.example.Editor.plist",
            "Application Support/com.example.Editor",
            "Containers/com.example.Editor",
        ]));
        assert_eq!(planned(&engine), expected);
        assert_eq!(view.summary.sizes.logical.bytes(), 171);
        assert_eq!(view.groups[0].category.as_str(), "core.uninstall-everything");
    }

    #[test]
    fn an_app_with_no_part_is_one_path() {
        let engine = World { state: state(1, &[("/home", NodeKind::Folder, 0)]), ..World::new() }.scanned();

        engine.prepare_uninstall(EDITOR, UninstallChoice::Everything).unwrap();

        let pending = engine.shared.pending.lock().unwrap();
        let (plan, _) = pending.pending().unwrap();
        assert_eq!(plan.items()[0].action(), &Action::Remove(PathBuf::from(BUNDLE)));
    }

    #[test]
    fn the_finding_is_medium_for_keep_data_and_high_for_everything() {
        let engine = World::new().scanned();

        let keep = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        let everything = engine.prepare_uninstall(EDITOR, UninstallChoice::Everything).unwrap();

        assert_eq!(keep.summary.highest, DangerLevel::Medium);
        assert_eq!(everything.summary.highest, DangerLevel::High);
    }

    #[test]
    fn the_plan_s_roots_name_the_apps_folder() {
        let engine = World::new().scanned();

        engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        let pending = engine.shared.pending.lock().unwrap();
        let (plan, purpose) = pending.pending().unwrap();
        assert_eq!(plan.roots(), [PathBuf::from("~"), PathBuf::from("/Applications")]);
        assert!(matches!(purpose, crate::cleanup::Purpose::Uninstall { id: EDITOR, .. }));
    }

    #[test]
    fn the_finding_id_is_past_every_scan_id() {
        let engine = World { caches_finding: true, ..World::new() }.scanned();
        assert_eq!(shown(&engine), [FindingId::new(1)]);

        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        let uninstall = view.groups[0].items[0].finding;
        crate::testing::scan(&engine);

        assert_eq!(uninstall, FindingId::new(2));
        let next = shown(&engine);
        assert_ne!(next, []);
        assert!(next.iter().all(|id| *id > uninstall));
    }

    #[test]
    fn an_uninstall_confirms_with_the_uninstall_context() {
        let probe = Arc::new(Probe::default());
        let engine = World { cleaner: Arc::clone(&probe) as _, ..World::new() }.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        confirm(&engine, &view);

        assert_eq!(probe.answers(), [Ok(()), Ok(())]);
        let roots = probe.roots.lock().unwrap().clone();
        assert!(roots.iter().all(|roots| roots.contains(&PathBuf::from("/Applications"))));
    }

    #[test]
    fn every_finding_below_a_part_that_went_leaves_the_results() {
        let engine = World { caches_finding: true, ..World::new() }.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        confirm(&engine, &view);

        assert_eq!(shown(&engine), []);
        let committed = engine.shared.results.lock().unwrap().committed().unwrap();
        assert!(committed.gone.contains(&PathBuf::from(BUNDLE)));
        assert!(committed.gone.contains(&PathBuf::from(CACHES)));
    }

    #[test]
    fn sixty_four_paths_are_one_item() {
        let names: Vec<String> =
            (0..59).map(|n| format!("/home/Library/LaunchAgents/com.example.Editor.agent{n:02}.plist")).collect();
        let more: Vec<(&str, NodeKind, u64)> = names.iter().map(|name| (name.as_str(), NodeKind::File, 1)).collect();
        let engine = World { state: home(&more), ..World::new() }.scanned();

        engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        assert_eq!(planned(&engine).len(), fresher_core::MAX_PARTS);
    }

    #[test]
    fn the_parts_that_went_of_a_partly_done_uninstall_leave() {
        let cleaner = fresher_testing::FakeCleaner::default().partly(FindingId::new(2), 1);
        let engine = World { caches_finding: true, cleaner: Arc::new(cleaner), ..World::new() }.scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        confirm(&engine, &view);

        assert_eq!(listed(&engine), Vec::<String>::new());
        assert_eq!(shown(&engine), [FindingId::new(1)]);
    }

    #[test]
    fn an_uninstalled_app_leaves_the_list() {
        let engine = World::new().scanned();
        let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();

        confirm(&engine, &view);

        assert_eq!(listed(&engine), Vec::<String>::new());
        assert_eq!(engine.app(EDITOR), Err(CommandError::UnknownApp { id: EDITOR }));
    }
}
