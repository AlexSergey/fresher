//! The engine: the one object the app drives. Commands return at once; long work reports through the bus.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fresher_core::{
    AccessProbe, AccessStatus, CancelToken, Clock, CommandError, CommandRunner, Dictionary, DiskWalk, EngineStatus,
    Event, Finding, FindingId, FindingsPage, FindingsQuery, ModuleInfo, Overview, ScanId, SchemaError, SettingUpdate,
    SettingsStore, SettingsView,
};

use crate::error::{EngineError, lock, lock_anyway};
use crate::results::{Filter, Results};
use crate::session::{self, Planned, ScanPlan};
use crate::settings::Settings;
use crate::state::{Machine, Running, ScanThread};
use crate::{EventBus, Registry};

/// How long `shutdown` waits for the cancelled scan's thread before it gives up on it.
const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// The ports the engine uses, implemented outside the core and injected by the app.
pub struct Ports {
    pub walk: Arc<dyn DiskWalk>,
    pub settings: Arc<dyn SettingsStore>,
    pub clock: Arc<dyn Clock>,
    pub commands: Arc<dyn CommandRunner>,
    pub access: Arc<dyn AccessProbe>,
}

impl std::fmt::Debug for Ports {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Ports").finish_non_exhaustive()
    }
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
}

impl Engine {
    /// An idle engine over the registered modules; the settings are loaded through the port here. It fails only if
    /// the core's own settings repeat a key.
    pub fn new(registry: Registry, ports: Ports) -> Result<Self, SchemaError> {
        let settings = Settings::load(registry.schema()?, &*ports.settings);
        let shared = Shared {
            registry,
            ports,
            bus: EventBus::new(),
            machine: Mutex::new(Machine::new()),
            results: Mutex::new(Results::new()),
            settings: Mutex::new(settings),
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

    /// Starts a scan on its own thread; it reports through the bus and ends with `ScanFinished`.
    pub fn start_scan(&self) -> Result<ScanId, CommandError> {
        let mut machine = lock(&self.shared.machine)?;
        let scan = machine.next_scan()?;
        let plan = self.plan()?;
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
        machine.start(Running { scan, cancel, thread: Some(ScanThread { handle, done: ended }) });
        Ok(scan)
    }

    /// Asks the running scan to stop; it still ends with `ScanFinished`, marked partial.
    pub fn cancel_scan(&self) -> Result<(), CommandError> {
        lock(&self.shared.machine)?.cancel()
    }

    /// Cancels the running operation and waits for its thread, at most 5 s: the app calls this before it quits. A
    /// thread still running past that is left behind (detached), and the app's exit ends it.
    pub fn shutdown(&self) {
        self.shutdown_within(SHUTDOWN_DEADLINE);
    }

    fn shutdown_within(&self, deadline: Duration) {
        let Some(thread) = lock_anyway(&self.shared.machine).stop() else { return };
        match thread.done.recv_timeout(deadline) {
            Err(RecvTimeoutError::Timeout) => {
                tracing::error!(?deadline, "the scan thread did not stop in time; it is left to the app's exit");
            }
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                if thread.handle.join().is_err() {
                    tracing::error!("the scan thread panicked");
                }
            }
        }
    }

    /// The state, the last scan and one total per category of the current findings.
    pub fn overview(&self) -> Result<Overview, CommandError> {
        let status = lock(&self.shared.machine)?.status();
        let results = lock(&self.shared.results)?;
        Ok(Overview { status, last_scan: results.last(), totals: results.totals(self.shared.registry.categories()) })
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
        let filter = Filter { module, category: query.category(), offset: query.offset(), limit: query.limit() };
        Ok(lock(&self.shared.results)?.page(filter))
    }

    /// One finding of the current scan; an id from another scan is unknown.
    pub fn finding(&self, id: FindingId) -> Result<Finding, CommandError> {
        lock(&self.shared.results)?.finding(id)
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
        self.shared.bus.publish(&changed);
        Ok(())
    }

    /// The registered modules for the UI, in registration order.
    pub fn modules(&self) -> Vec<ModuleInfo> {
        self.shared.registry.modules()
    }

    /// The merged texts of the core and every module.
    pub fn dictionary(&self) -> &Dictionary {
        self.shared.registry.dictionary()
    }

    pub fn access_status(&self) -> AccessStatus {
        self.shared.ports.access.status()
    }
}

impl Engine {
    /// What the next scan runs. The results stay as they are: the scan's thread empties them when it starts.
    fn plan(&self) -> Result<ScanPlan, EngineError> {
        let settings = lock(&self.shared.settings)?;
        let modules =
            self.shared.registry.registered().iter().filter(|module| settings.is_enabled(module.descriptor().id()));
        let modules = modules
            .map(|module| Planned { module: Arc::clone(module), settings: settings.module(module.descriptor().id()) })
            .collect();
        Ok(ScanPlan { roots: settings.scan_roots(), modules })
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

        use fresher_core::{CommandError, DangerLevel, EngineStatus, Event, FindingId, Operation, ScanOutcome};

        use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, engine, poison, scan, wait_for_finish};

        #[test]
        fn refuses_a_scan_while_one_is_running() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            engine.start_scan().unwrap();

            assert_eq!(engine.start_scan(), Err(CommandError::Busy { running: Operation::Scan }));

            engine.cancel_scan().unwrap();
            wait_for_finish(&events);
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
            let scan = engine.start_scan().unwrap();

            engine.cancel_scan().unwrap();

            let finished = wait_for_finish(&events);
            assert_eq!(finished.last(), Some(&Event::ScanFinished { scan, outcome: ScanOutcome::Partial }));
        }

        #[test]
        fn planning_a_scan_keeps_the_previous_findings() {
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let engine = Fixture::new(walk).module(module).engine();
            scan(&engine);

            // What a scan whose thread could not be started leaves behind: the plan, and no thread.
            engine.plan().unwrap();

            assert_eq!(engine.finding(FindingId::new(1)).unwrap().module().as_str(), "alpha");
            assert_eq!(engine.overview().unwrap().totals.len(), 1);
        }

        #[test]
        fn shutdown_gives_up_on_a_scan_that_does_not_stop() {
            let (walk, gate) = ScriptedWalk::new().deaf().hold();
            let engine = engine(walk);
            let events = engine.subscribe();
            let scan = engine.start_scan().unwrap();

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
            assert_eq!(engine.start_scan(), Err(CommandError::Internal));
            assert_eq!(engine.cancel_scan(), Err(CommandError::Internal));
        }
    }

    mod positive {
        use fresher_core::{Access, AccessStatus, EngineStatus, Event, ScanId, ScanOutcome};

        use crate::testing::{ScriptedWalk, engine, wait_for_finish};

        #[test]
        fn a_scan_goes_idle_scanning_idle() {
            let (walk, gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));

            let scan = engine.start_scan().unwrap();
            assert_eq!(engine.status(), Ok(EngineStatus::Scanning { scan }));

            gate.release();
            let finished = wait_for_finish(&events);
            assert_eq!(finished.first(), Some(&Event::ScanStarted { scan, modules: vec![] }));
            assert_eq!(finished.last(), Some(&Event::ScanFinished { scan, outcome: ScanOutcome::Completed }));
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
        }

        #[test]
        fn shutdown_cancels_and_joins_the_running_scan() {
            let (walk, _gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            let scan = engine.start_scan().unwrap();

            engine.shutdown();

            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            let last = events.try_iter().last();
            assert_eq!(last, Some(Event::ScanFinished { scan, outcome: ScanOutcome::Partial }));
        }

        #[test]
        fn scan_ids_increase_per_scan() {
            let engine = engine(ScriptedWalk::new());
            let events = engine.subscribe();

            let first = engine.start_scan().unwrap();
            wait_for_finish(&events);
            let second = engine.start_scan().unwrap();
            wait_for_finish(&events);

            assert_eq!((first, second), (ScanId::new(1), ScanId::new(2)));
        }

        #[test]
        fn answers_the_access_status_of_its_port() {
            let engine = engine(ScriptedWalk::new());

            assert_eq!(engine.access_status(), AccessStatus { full_disk_access: Access::Unknown });
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
