//! The engine: the one object the app drives. Commands return at once; long work reports through the bus.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fresher_core::{
    Access, AccessProbe, AccessStatus, CancelToken, Capability, Cleaner, CleanupDetail, CleanupId, CleanupJournal,
    CleanupPlanView, CleanupSelection, CleanupsPage, Clock, CommandError, CommandRunner, Confirmation, Dictionary,
    DiskState, DiskWalk, EngineStatus, Event, Finding, FindingId, FindingsPage, FindingsQuery, KnownFolder, ModuleInfo,
    Operation, Overview, PlanId, PutBackReport, ScanId, SchemaError, SettingEffect, SettingKey, SettingUpdate,
    SettingsStore, SettingsView, SystemUi, SystemUiError, TextKey, Volumes, WalkMode,
};

use crate::access::AccessView;
use crate::cleanup::{self, Pending};
use crate::coverage::{OPT_INS, opt_in_key};
use crate::error::{EngineError, lock, lock_anyway};
use crate::places::walked_places;
use crate::results::{Filter, Results};
use crate::session::{self, Planned, ScanPlan};
use crate::settings::{self as settings_rules, Settings};
use crate::state::{Machine, OperationThread, Running};
use crate::storage::roots::{RootVolumes, expand_home};
use crate::{EventBus, Registry};

/// How long `shutdown` waits for the cancelled operation's thread before it gives up on it.
const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// The ports the engine uses, implemented outside the core and injected by the app.
pub struct Ports {
    pub walk: Arc<dyn DiskWalk>,
    pub settings: Arc<dyn SettingsStore>,
    pub clock: Arc<dyn Clock>,
    pub commands: Arc<dyn CommandRunner>,
    pub access: Arc<dyn AccessProbe>,
    pub system_ui: Arc<dyn SystemUi>,
    /// The system's facts: known folders, installed and running apps. Each scan gathers them anew, when asked.
    pub system: fresher_core::SystemPorts,
}

impl std::fmt::Debug for Ports {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Ports").finish_non_exhaustive()
    }
}

/// The ports a cleanup uses: the cleaner (the only code that removes), the journal of cleanups, and the volumes whose
/// available space a cleanup measures.
#[derive(Clone)]
pub struct CleanupPorts {
    pub cleaner: Arc<dyn Cleaner>,
    pub journal: Arc<dyn CleanupJournal>,
    pub volumes: Arc<dyn Volumes>,
}

impl std::fmt::Debug for CleanupPorts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("CleanupPorts").finish_non_exhaustive()
    }
}

/// How a scan the user starts reads the disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScanMode {
    /// Only what changed since the last scan, when the walk can tell; every folder otherwise.
    Auto,
    /// Every folder, whatever the last scan left.
    Full,
}

/// Runs the registered modules, one operation at a time. Dropping it shuts it down.
#[derive(Debug)]
pub struct Engine {
    pub(crate) shared: Arc<Shared>,
}

/// What the engine and its scan thread share.
#[derive(Debug)]
pub(crate) struct Shared {
    pub(crate) registry: Registry,
    pub(crate) ports: Ports,
    pub(crate) bus: EventBus,
    pub(crate) machine: Mutex<Machine>,
    pub(crate) results: Mutex<Results>,
    pub(crate) settings: Mutex<Settings>,
    pub(crate) access: AccessView,
    /// `None`: the engine cannot clean, and refuses every cleanup (`CleaningBlocked`).
    pub(crate) cleanup: Option<CleanupPorts>,
    pub(crate) pending: Mutex<Pending>,
}

impl Shared {
    /// Probes the system's permissions again; publishes `AccessChanged` and answers `true` when something changed.
    pub(crate) fn refresh_access(&self) -> bool {
        let changed = self.access.refresh();
        if changed {
            self.bus.publish(&Event::AccessChanged { status: self.access.status() });
        }
        changed
    }

    /// The home folder, where `~` in a scan root leads, as the walk fills it in.
    pub(crate) fn home(&self) -> Option<PathBuf> {
        self.ports.system.known_folders.locate(KnownFolder::Home)
    }

    /// Reads the volumes of the absolute `roots` for the storage map, the home's first: through the volumes of the
    /// cleanup ports, or without facts when the engine has none.
    pub(crate) fn read_volumes(&self, roots: &[PathBuf]) -> RootVolumes {
        RootVolumes::read(self.cleanup.as_ref().map(|ports| &*ports.volumes), self.home().as_deref(), roots)
    }
}

impl Engine {
    /// An idle engine over the registered modules that cannot clean: every cleanup is refused with
    /// `CleaningBlocked`. The settings are loaded through the port here. It fails only if the core's own settings
    /// repeat a key.
    pub fn new(registry: Registry, ports: Ports) -> Result<Self, SchemaError> {
        Self::build(registry, ports, None)
    }

    /// An idle engine over the registered modules that cleans through `cleanup`.
    pub fn with_cleanup(registry: Registry, ports: Ports, cleanup: CleanupPorts) -> Result<Self, SchemaError> {
        Self::build(registry, ports, Some(cleanup))
    }

    fn build(registry: Registry, ports: Ports, cleanup: Option<CleanupPorts>) -> Result<Self, SchemaError> {
        let settings = Settings::load(registry.schema()?, &*ports.settings);
        let access = AccessView::new(Arc::clone(&ports.access), Arc::clone(&ports.commands), registry.tool_checks());
        let mut results = Results::new();
        results.set_ignored(settings.ignored(&registry));
        let shared = Shared {
            registry,
            ports,
            bus: EventBus::new(),
            machine: Mutex::new(Machine::new()),
            results: Mutex::new(results),
            settings: Mutex::new(settings),
            access,
            cleanup,
            pending: Mutex::new(Pending::default()),
        };
        Ok(Self { shared: Arc::new(shared) })
    }

    /// A receiver of every event published from now on.
    pub fn subscribe(&self) -> Receiver<Event> {
        self.shared.bus.subscribe()
    }

    pub fn status(&self) -> Result<EngineStatus, CommandError> {
        Ok(lock(&self.shared.machine)?.status())
    }

    /// Starts a scan on its own thread; it reports through the bus and ends with `ScanFinished`. `Auto` reads only
    /// what changed since the last scan when the walk can tell; `Full` reads every folder.
    pub fn start_scan(&self, mode: ScanMode) -> Result<ScanId, CommandError> {
        let walk = match mode {
            ScanMode::Auto => WalkMode::Auto,
            ScanMode::Full => WalkMode::Full,
        };
        self.start(walk)
    }

    /// Shows the last scan again without reading the disk, as an operation like a scan (refused while one runs): the
    /// walk restores its cached state and the enabled modules that declare interests run over its matches; the
    /// modules without interests (they ask an external tool) do not run. With nothing to restore it ends quietly: no
    /// event, and the engine stays without a last scan.
    pub fn restore(&self) -> Result<ScanId, CommandError> {
        self.start(WalkMode::Restore)
    }

    fn start(&self, mode: WalkMode) -> Result<ScanId, CommandError> {
        let mut machine = lock(&self.shared.machine)?;
        let scan = machine.next_scan()?;
        let plan = self.plan(mode)?;
        // The findings a pending cleanup plan was made of are about to be replaced.
        lock(&self.shared.pending)?.kill();
        let cancel = CancelToken::new();
        let shared = Arc::clone(&self.shared);
        let token = cancel.clone();
        let (done, ended) = mpsc::channel::<()>();
        let handle = thread::Builder::new()
            .name(format!("fresher-scan-{}", scan.get()))
            .spawn(move || {
                // Dropped when the thread ends, on a return or a panic: `shutdown` waits for that, not for ever.
                let _done = done;
                session::run(&shared, scan, &token, &plan);
            })
            .map_err(|error| EngineError::Spawn { kind: error.kind() })?;
        machine.start(scan, Running { cancel, thread: Some(OperationThread { handle, done: ended }) });
        Ok(scan)
    }

    /// Asks the running scan to stop; it still ends with `ScanFinished`, marked partial.
    pub fn cancel_scan(&self) -> Result<(), CommandError> {
        lock(&self.shared.machine)?.cancel(Operation::Scan)
    }

    /// Cancels the running operation (a scan, or a cleanup after the item in flight) and waits for its thread, at most
    /// 5 s: the app calls this before it quits. A thread still running past that is left behind (detached), and the
    /// app's exit ends it.
    pub fn shutdown(&self) {
        self.shutdown_within(SHUTDOWN_DEADLINE);
    }

    fn shutdown_within(&self, deadline: Duration) {
        let Some(thread) = lock_anyway(&self.shared.machine).stop() else { return };
        match thread.done.recv_timeout(deadline) {
            Err(RecvTimeoutError::Timeout) => {
                tracing::error!(?deadline, "the operation's thread did not stop in time; it is left to the app's exit");
            }
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                if thread.handle.join().is_err() {
                    tracing::error!("the operation's thread panicked");
                }
            }
        }
    }

    /// The state, the last scan, one total per category of the current findings and all of them summed.
    pub fn overview(&self) -> Result<Overview, CommandError> {
        let status = lock(&self.shared.machine)?.status();
        let results = lock(&self.shared.results)?;
        let (totals, found) = results.totals(self.shared.registry.categories());
        let ignored = results.ignored_count();
        let hidden_modules = results.hidden_modules(self.shared.registry.module_groups());
        Ok(Overview { status, last_scan: results.last(), totals, found, ignored, hidden_modules })
    }

    /// A page of the current findings, by module and category. An unknown module is refused; a category nobody
    /// declared matches nothing.
    pub fn findings(&self, query: &FindingsQuery) -> Result<FindingsPage, CommandError> {
        let module = match query.module() {
            None => None,
            Some(id) => {
                Some(self.shared.registry.module_id(id).ok_or_else(|| CommandError::UnknownModule { id: id.into() })?)
            }
        };
        let filter = Filter {
            module,
            category: query.category(),
            order: query.order(),
            offset: query.offset(),
            limit: query.limit(),
        };
        Ok(lock(&self.shared.results)?.page(filter))
    }

    /// One finding of the current scan; an id from another scan is unknown.
    pub fn finding(&self, id: FindingId) -> Result<Finding, CommandError> {
        lock(&self.shared.results)?.finding(id)
    }

    /// The disk state of the last committed scan; `None` before any scan committed one.
    pub fn disk_state(&self) -> Result<Option<Arc<dyn DiskState>>, CommandError> {
        Ok(lock(&self.shared.results)?.state())
    }

    /// Every setting, its value in use, the problems found in the file and whether the settings are read-only.
    pub fn settings(&self) -> Result<SettingsView, CommandError> {
        Ok(lock(&self.shared.settings)?.view())
    }

    /// Checks the value, saves it through the port, then publishes `SettingChanged`. A running scan keeps the
    /// settings it started with.
    pub fn update_setting(&self, update: SettingUpdate) -> Result<(), CommandError> {
        let mut settings = lock(&self.shared.settings)?;
        let changed = settings.update(update, &*self.shared.ports.settings)?;
        self.take_changes(&settings, &[changed])
    }

    /// Takes in saved changes of `settings` (held locked, so the events go out in the order of the saves), then
    /// publishes them: the ignore lists hide what they name at once; a change that applies at the next scan marks the
    /// last scan ("Rescan to apply"); a change of what is scanned or ignored kills the pending cleanup plan.
    pub(crate) fn take_changes(&self, settings: &Settings, changes: &[Event]) -> Result<(), CommandError> {
        let keys: Vec<&SettingKey> = changes
            .iter()
            .filter_map(|event| match event {
                Event::SettingChanged { key, .. } => Some(key),
                _ => None,
            })
            .collect();
        {
            let mut results = lock(&self.shared.results)?;
            results.set_ignored(settings.ignored(&self.shared.registry));
            if keys.iter().any(|key| settings.effect(key.as_str()) == Some(SettingEffect::NextScan)) {
                results.mark_settings_changed();
            }
        }
        if keys.iter().any(|key| settings_rules::changes_what_is_scanned(key) || settings_rules::is_ignore_list(key)) {
            lock(&self.shared.pending)?.kill();
        }
        for change in changes {
            self.shared.bus.publish(change);
        }
        Ok(())
    }

    /// The registered modules for the UI, in registration order, each category with its availability.
    pub fn modules(&self) -> Vec<ModuleInfo> {
        self.shared.registry.modules(&self.shared.access.status())
    }

    /// Every problem the registered modules declare: the reasons a module may give for leaving an item out.
    pub fn problems(&self) -> Vec<TextKey> {
        self.shared.registry.problems().cloned().collect()
    }

    /// The merged texts of the core and every module.
    pub fn dictionary(&self) -> &Dictionary {
        self.shared.registry.dictionary()
    }

    /// What the app may see and do: the system's permissions as last probed, and the tools as last checked.
    pub fn access_status(&self) -> AccessStatus {
        self.shared.access.status()
    }

    /// Probes the system's permissions again (the tools are checked at the start of a scan); publishes
    /// `AccessChanged` and answers `true` when something changed.
    pub fn refresh_access(&self) -> bool {
        self.shared.refresh_access()
    }

    /// Opens the system's settings at the pane that grants `capability`. Refused for a tool, for a capability this
    /// system lacks, and when the system has no such pane.
    pub fn open_access_settings(&self, capability: &Capability) -> Result<(), CommandError> {
        let no_pane = || CommandError::NoSettingsPane { capability: *capability };
        if matches!(capability, Capability::Tool(_)) || self.access_status().access(capability) == Access::NotApplicable
        {
            return Err(no_pane());
        }
        self.shared.ports.system_ui.open_access_settings(capability).map_err(|error| match error {
            SystemUiError::Unsupported => no_pane(),
            SystemUiError::Failed { .. } => {
                tracing::warn!(%error, capability = capability.id(), "the settings could not be opened");
                CommandError::Internal
            }
        })
    }
}

/// Cleanups: the plan, its confirmation and the cleanup it starts. Nothing is removed but from a plan the user
/// confirmed.
impl Engine {
    /// Turns `selection` (ids only) into a frozen plan, rehearsed with every check of the cleaner and nothing done,
    /// and keeps it as the pending plan in place of any earlier one; answers it as the confirmation shows it.
    /// Refused while an operation runs, before any scan, for an id the scan did not produce, a category nobody
    /// registered, a selection that picks nothing or more than 10 000 items, and when cleaning is blocked.
    pub fn prepare_cleanup(&self, selection: &CleanupSelection) -> Result<CleanupPlanView, CommandError> {
        cleanup::prepare(&self.shared, selection)
    }

    /// Drops the pending plan `plan` (the confirmation's Cancel); any other id changes nothing.
    pub fn discard_cleanup(&self, plan: PlanId) -> Result<(), CommandError> {
        lock(&self.shared.pending)?.discard(plan);
        Ok(())
    }

    /// Confirms the pending plan with the summary the user saw and starts its cleanup on its own thread; answers the
    /// cleanup's id. It reports through the bus: `CleanupStarted`, `CleanupItems`, `FindingsRemoved`,
    /// `CleanupFinished`. Refused while an operation runs, for a plan that is gone or expired, for a summary that
    /// differs from the plan's (the plan dies: the user prepares again), and when cleaning is blocked; nothing is
    /// acted on then.
    pub fn confirm_cleanup(&self, echo: &Confirmation) -> Result<CleanupId, CommandError> {
        cleanup::confirm(&self.shared, echo)
    }

    /// Asks the running cleanup to stop after the item in flight; it still ends with `CleanupFinished`, cancelled.
    pub fn cancel_cleanup(&self) -> Result<(), CommandError> {
        lock(&self.shared.machine)?.cancel(Operation::Clean)
    }

    /// The window is closing: while a cleanup runs, publishes `QuitRequested` (the UI asks the user) and answers
    /// `true`, and the app holds the close; otherwise answers `false`.
    pub fn request_quit(&self) -> bool {
        let cleaning = matches!(lock_anyway(&self.shared.machine).status(), EngineStatus::Cleaning { .. });
        if cleaning {
            self.shared.bus.publish(&Event::QuitRequested);
        }
        cleaning
    }

    /// The user chose to quit during a cleanup: it stops after the item in flight, its journal is closed, and this
    /// returns once its thread ended (like [`Engine::shutdown`]).
    pub fn quit_after_cleanup(&self) {
        self.shutdown();
    }

    /// A page of the history of cleanups, newest first, at most 100; each with how many of its items are still in
    /// the Trash, and the size of those summed over the page. A journal that cannot be written says so; one that
    /// cannot be read answers an empty page.
    pub fn cleanups(&self, offset: u32, limit: u32) -> CleanupsPage {
        cleanup::page(&self.shared, offset, limit)
    }

    /// One cleanup of the journal with every item: its outcome or skip reason, and where it is now.
    pub fn cleanup(&self, id: CleanupId) -> Result<CleanupDetail, CommandError> {
        cleanup::detail(&self.shared, id)
    }

    /// Puts `items` of cleanup `id` back from the Trash — every item still there when `None` — never replacing
    /// anything; each attempt is journalled. Refused while a scan or a cleanup runs, and for an item the cleanup does
    /// not have.
    pub fn put_back(&self, id: CleanupId, items: Option<&[u32]>) -> Result<PutBackReport, CommandError> {
        cleanup::put_back(&self.shared, id, items)
    }

    /// Shows item `item` of cleanup `id` in the Trash, selected; refused when it is not in the Trash now.
    pub fn reveal_in_trash(&self, id: CleanupId, item: u32) -> Result<(), CommandError> {
        cleanup::reveal(&self.shared, id, item)
    }
}

impl Engine {
    /// What the next scan runs in `mode`: the enabled modules, and for a restore only those with interests. The
    /// results stay as they are: the scan's thread empties them when it starts.
    fn plan(&self, mode: WalkMode) -> Result<ScanPlan, EngineError> {
        let settings = lock(&self.shared.settings)?;
        let modules = self.shared.registry.registered().iter().filter(|module| {
            settings.is_enabled(module.descriptor().id())
                && (mode != WalkMode::Restore || !module.interests().is_empty())
        });
        let modules = modules
            .map(|module| Planned { module: Arc::clone(module), settings: settings.module(module.descriptor().id()) })
            .collect();
        let opt_ins = OPT_INS.into_iter().filter(|&capability| settings.core_switch(&opt_in_key(capability))).collect();
        let roots = settings.scan_roots();
        let protection = self.shared.cleanup.as_ref().map(|cleanup| cleanup.cleaner.protection()).unwrap_or_default();
        let places = walked_places(
            &self.shared.registry,
            &*self.shared.ports.system.known_folders,
            &expand_home(&roots, self.shared.home().as_deref()),
            &protection,
        );
        Ok(ScanPlan { roots, places, opt_ins, modules, mode, ignored: settings.ignored(&self.shared.registry) })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::time::Duration;

        use fresher_core::{
            CommandError, DangerLevel, EngineStatus, Event, FindingId, Operation, ScanKind, ScanOutcome,
        };

        use crate::testing::{
            Fixture, ScriptedWalk, TestModule, bytes, engine, poison, scan, wait_for_finish, wait_until_idle,
        };

        #[test]
        fn refuses_a_scan_while_one_is_running() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            assert_eq!(engine.start_scan(crate::ScanMode::Auto), Err(CommandError::Busy { running: Operation::Scan }));

            engine.cancel_scan().unwrap();
            wait_for_finish(&events);
        }

        #[test]
        fn a_restore_during_a_scan_is_refused() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            assert_eq!(engine.restore(), Err(CommandError::Busy { running: Operation::Scan }));

            engine.cancel_scan().unwrap();
            wait_for_finish(&events);
        }

        #[test]
        fn a_restore_runs_no_module_without_interests() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicUsize, Ordering};

            let finished = Arc::new(AtomicUsize::new(0));
            let counted = Arc::clone(&finished);
            let tool = TestModule::builder("tool").category("tool.images", DangerLevel::Low).on_finish(move |_, _| {
                counted.fetch_add(1, Ordering::Relaxed);
                Ok(())
            });
            let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/Caches", bytes(1))
                .kind(ScanKind::Restored { taken: fresher_core::Timestamp::from_secs(7) });
            let engine = Fixture::new(walk).module(tool).module(alpha).engine();
            let events = engine.subscribe();

            let scan = engine.restore().unwrap();

            let seen = wait_for_finish(&events);
            assert_eq!(finished.load(Ordering::Relaxed), 0);
            let alpha = fresher_core::ModuleId::new("alpha").unwrap();
            assert_eq!(seen.first(), Some(&Event::ScanStarted { scan, modules: vec![alpha] }));
        }

        #[test]
        fn nothing_to_restore_leaves_the_engine_idle_without_a_last_scan() {
            let walk = ScriptedWalk::new().failing(fresher_core::WalkError::NothingToRestore);
            let modes = walk.modes();
            let engine = engine(walk);
            let events = engine.subscribe();

            engine.restore().unwrap();

            wait_until_idle(&engine);
            assert_eq!(events.try_iter().count(), 0);
            assert_eq!(engine.overview().unwrap().last_scan, None);
            assert_eq!(*modes.lock().unwrap(), [fresher_core::WalkMode::Restore]);
        }

        #[test]
        fn refuses_to_cancel_when_idle() {
            let engine = engine(ScriptedWalk::new());

            assert_eq!(engine.cancel_scan(), Err(CommandError::NotRunning));
        }

        #[test]
        fn a_cancelled_scan_ends_partial() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            let scan = engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.cancel_scan().unwrap();

            let finished = wait_for_finish(&events);
            assert_eq!(
                finished.last(),
                Some(&Event::ScanFinished { scan, outcome: ScanOutcome::Partial, kind: ScanKind::Full })
            );
        }

        #[test]
        fn planning_a_scan_keeps_the_previous_findings() {
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let engine = Fixture::new(walk).module(module).engine();
            scan(&engine);

            // What a scan whose thread could not be started leaves behind: the plan, and no thread.
            engine.plan(fresher_core::WalkMode::Auto).unwrap();

            assert_eq!(engine.finding(FindingId::new(1)).unwrap().module().as_str(), "alpha");
            assert_eq!(engine.overview().unwrap().totals.len(), 1);
        }

        #[test]
        fn shutdown_gives_up_on_a_scan_that_does_not_stop() {
            let (walk, gate) = ScriptedWalk::new().deaf().hold();
            let engine = engine(walk);
            let events = engine.subscribe();
            let scan = engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.shutdown_within(Duration::from_millis(10));

            assert_eq!(engine.status(), Ok(EngineStatus::Scanning { scan }));
            gate.release();
            wait_for_finish(&events);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
        }

        #[test]
        fn a_poisoned_state_is_an_internal_error() {
            let engine = engine(ScriptedWalk::new());
            poison(&engine.shared.machine);

            assert_eq!(engine.status(), Err(CommandError::Internal));
            assert_eq!(engine.start_scan(crate::ScanMode::Auto), Err(CommandError::Internal));
            assert_eq!(engine.cancel_scan(), Err(CommandError::Internal));
        }
    }

    mod positive {
        use fresher_core::{Access, Capability, EngineStatus, Event, ScanId, ScanKind, ScanOutcome};

        use fresher_testing::FakeAccess;

        use crate::testing::{Fixture, ScriptedWalk, engine, wait_for_finish};

        #[test]
        fn a_scan_goes_idle_scanning_idle() {
            let (walk, gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));

            let scan = engine.start_scan(crate::ScanMode::Auto).unwrap();
            assert_eq!(engine.status(), Ok(EngineStatus::Scanning { scan }));

            gate.release();
            let finished = wait_for_finish(&events);
            assert_eq!(finished.first(), Some(&Event::ScanStarted { scan, modules: vec![] }));
            assert_eq!(
                finished.last(),
                Some(&Event::ScanFinished { scan, outcome: ScanOutcome::Completed, kind: ScanKind::Full })
            );
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
        }

        #[test]
        fn a_module_knows_a_restore_from_a_walk() {
            use std::sync::{Arc, Mutex};

            use fresher_core::{DangerLevel, Timestamp};

            use crate::testing::{Fixture, TestModule, bytes};

            let seen = Arc::new(Mutex::new(Vec::new()));
            let record = Arc::clone(&seen);
            let module = TestModule::builder("alpha")
                .category("alpha.caches", DangerLevel::Safe)
                .folder("Caches")
                .on_finish(move |cx, _| {
                    record.lock().unwrap().push(cx.is_restored());
                    Ok(())
                });
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/Caches", bytes(5))
                .kind(ScanKind::Restored { taken: Timestamp::from_secs(7) });
            let engine = Fixture::new(walk).module(module).engine();
            let events = engine.subscribe();

            engine.restore().unwrap();
            wait_for_finish(&events);
            engine.start_scan(crate::ScanMode::Full).unwrap();
            wait_for_finish(&events);

            assert_eq!(*seen.lock().unwrap(), [true, false]);
        }

        #[test]
        fn a_module_knows_in_finish_whether_the_walk_read_every_folder() {
            use std::sync::{Arc, Mutex};

            use fresher_core::DangerLevel;

            use crate::testing::{Fixture, TestModule, bytes};

            for (walk, expected) in [
                (ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(5)), [true]),
                (ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(5)).unreadable(1), [false]),
            ] {
                let seen = Arc::new(Mutex::new(Vec::new()));
                let record = Arc::clone(&seen);
                let module = TestModule::builder("alpha")
                    .category("alpha.caches", DangerLevel::Safe)
                    .folder("Caches")
                    .on_finish(move |cx, _| {
                        record.lock().unwrap().push(cx.is_walk_complete());
                        Ok(())
                    });
                let engine = Fixture::new(walk).module(module).engine();
                let events = engine.subscribe();

                engine.start_scan(crate::ScanMode::Full).unwrap();
                wait_for_finish(&events);

                assert_eq!(*seen.lock().unwrap(), expected);
            }
        }

        #[test]
        fn a_restore_shows_the_last_findings_with_their_date() {
            use fresher_core::{DangerLevel, Timestamp, WalkMode};

            use crate::testing::{Fixture, TestModule, bytes};

            let taken = Timestamp::from_secs(1_763_000_000);
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let walk =
                ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(5)).kind(ScanKind::Restored { taken });
            let (modes, commits) = (walk.modes(), walk.commits());
            let engine = Fixture::new(walk).module(module).engine();
            let events = engine.subscribe();

            let scan = engine.restore().unwrap();

            let finished = wait_for_finish(&events);
            let kind = ScanKind::Restored { taken };
            assert_eq!(finished.last(), Some(&Event::ScanFinished { scan, outcome: ScanOutcome::Completed, kind }));
            let overview = engine.overview().unwrap();
            assert_eq!(overview.last_scan.map(|last| last.kind), Some(kind));
            assert_eq!(overview.totals[0].sizes.logical.bytes(), 5);
            assert_eq!(*modes.lock().unwrap(), [WalkMode::Restore]);
            assert_eq!(commits.lock().unwrap().len(), 1);
        }

        #[test]
        fn a_scan_passes_its_mode_to_the_walk() {
            use fresher_core::WalkMode;

            let walk = ScriptedWalk::new();
            let modes = walk.modes();
            let engine = engine(walk);
            let events = engine.subscribe();

            engine.start_scan(crate::ScanMode::Auto).unwrap();
            wait_for_finish(&events);
            engine.start_scan(crate::ScanMode::Full).unwrap();
            wait_for_finish(&events);

            assert_eq!(*modes.lock().unwrap(), [WalkMode::Auto, WalkMode::Full]);
        }

        #[test]
        fn shutdown_cancels_and_joins_the_running_scan() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            let scan = engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.shutdown();

            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            let last = events.try_iter().last();
            assert_eq!(last, Some(Event::ScanFinished { scan, outcome: ScanOutcome::Partial, kind: ScanKind::Full }));
        }

        #[test]
        fn scan_ids_increase_per_scan() {
            let engine = engine(ScriptedWalk::new());
            let events = engine.subscribe();

            let first = engine.start_scan(crate::ScanMode::Auto).unwrap();
            wait_for_finish(&events);
            let second = engine.start_scan(crate::ScanMode::Auto).unwrap();
            wait_for_finish(&events);

            assert_eq!((first, second), (ScanId::new(1), ScanId::new(2)));
        }

        #[test]
        fn answers_the_access_status_of_its_port() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)]);
            let engine = Fixture::new(ScriptedWalk::new()).access(access).engine();

            assert_eq!(engine.access_status().access(&Capability::AllFiles), Access::Unknown);
        }

        #[test]
        fn answers_the_modules_and_texts_of_its_registry() {
            let engine = engine(ScriptedWalk::new());

            assert_eq!(engine.modules(), vec![]);
            assert!(engine.dictionary().get("core.error.busy").is_some());
            assert!(format!("{engine:?}").starts_with("Engine"));
        }
    }
}
