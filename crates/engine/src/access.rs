//! The engine's view of access: the last answer of the access probe, merged with the states of the tools the modules
//! need. The tools are checked by the engine itself, through the `CommandRunner` port: running a program is not a
//! permission of the system.

use std::sync::{Arc, Mutex};

use fresher_core::{
    Access, AccessProbe, AccessStatus, Capability, CapabilityState, CommandRunError, CommandRunner, ToolCommand,
};

use crate::error::lock_anyway;

/// What the app may see and do, as the engine last learnt it.
pub(crate) struct AccessView {
    probe: Arc<dyn AccessProbe>,
    commands: Arc<dyn CommandRunner>,
    known: Mutex<Known>,
}

/// The probe's last answer, and one state per tool any module needs, in registration order.
struct Known {
    system: AccessStatus,
    tools: Vec<CapabilityState>,
}

impl AccessView {
    /// Probes the system once (a probe never prompts); every tool of `checks` is `NotAsked` until its check runs.
    pub(crate) fn new<'a>(
        probe: Arc<dyn AccessProbe>,
        commands: Arc<dyn CommandRunner>,
        checks: impl Iterator<Item = &'a ToolCommand>,
    ) -> Self {
        let mut tools: Vec<CapabilityState> = Vec::new();
        for check in checks {
            let tool = Capability::Tool(check.tool());
            if !tools.iter().any(|known| known.capability == tool) {
                tools.push(CapabilityState::new(tool, Access::NotAsked));
            }
        }
        let system = probe.status();
        Self { probe, commands, known: Mutex::new(Known { system, tools }) }
    }

    /// The system's capabilities, then the tools'.
    pub(crate) fn status(&self) -> AccessStatus {
        let known = lock_anyway(&self.known);
        let capabilities = known.system.capabilities.iter().chain(&known.tools).cloned().collect();
        AccessStatus { capabilities }
    }

    /// Probes the system again; `true` when anything changed. The tools are not checked again.
    pub(crate) fn refresh(&self) -> bool {
        let system = self.probe.status();
        let mut known = lock_anyway(&self.known);
        let changed = known.system != system;
        known.system = system;
        changed
    }

    /// Takes the answers of the guarded places tried in a scan: each capability's system state becomes its answer
    /// (added when the probe did not list it). `true` when a state changed. The probe may remember its tries itself
    /// (`MacAccess` does); the engine does not rely on it, so a module needing a place runs in the scan that tried it.
    pub(crate) fn learn(&self, tried: &[(Capability, Access)]) -> bool {
        let mut known = lock_anyway(&self.known);
        let mut changed = false;
        for &(capability, access) in tried {
            match known.system.capabilities.iter_mut().find(|state| state.capability == capability) {
                Some(state) if state.access == access => {}
                Some(state) => {
                    state.access = access;
                    changed = true;
                }
                None => {
                    known.system.capabilities.push(CapabilityState::new(capability, access));
                    changed = true;
                }
            }
        }
        changed
    }

    /// Runs each check once: success → `Granted`; not installed → `Denied` with `core.access.tool.not-installed`; any
    /// other answer → `Denied` with `core.access.tool.not-ready`; a check stopped by `cancel` changes nothing. `true`
    /// when a state changed.
    pub(crate) fn check_tools<'a>(
        &self,
        checks: impl Iterator<Item = &'a ToolCommand>,
        cancel: &fresher_core::CancelToken,
    ) -> bool {
        let mut changed = false;
        for check in checks {
            let Some(state) = self.check(check, cancel) else { continue };
            let mut known = lock_anyway(&self.known);
            match known.tools.iter_mut().find(|known| known.capability == state.capability) {
                Some(known) if *known == state => {}
                Some(known) => {
                    *known = state;
                    changed = true;
                }
                None => {
                    known.tools.push(state);
                    changed = true;
                }
            }
        }
        changed
    }

    /// The tool's state; `None` when the check was cancelled.
    fn check(&self, check: &ToolCommand, cancel: &fresher_core::CancelToken) -> Option<CapabilityState> {
        let tool = check.tool();
        let args: Vec<&str> = check.args().iter().map(String::as_str).collect();
        match self.commands.run(tool.as_str(), &args, cancel) {
            Ok(output) if output.status == 0 => Some(CapabilityState::new(Capability::Tool(tool), Access::Granted)),
            Err(CommandRunError::NotInstalled { .. }) => Some(CapabilityState::tool_not_installed(tool)),
            Ok(_) | Err(CommandRunError::Failed { .. }) => Some(CapabilityState::tool_not_ready(tool)),
            Err(CommandRunError::Cancelled { .. }) => None,
        }
    }
}

impl std::fmt::Debug for AccessView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("AccessView").field("status", &self.status()).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Capability, DangerLevel, Event, ModuleId, ModuleOutcome, Need, ToolId};

    use crate::testing::{ScriptedWalk, TestModule, TestModuleBuilder, bytes};

    fn docker() -> Capability {
        Capability::Tool(ToolId::new("docker").unwrap())
    }

    /// A module that needs Docker to find anything, checked with `docker info`.
    fn docker_module() -> TestModuleBuilder {
        TestModule::builder("docker")
            .needing("docker.images", DangerLevel::Low, vec![Need::required(docker())])
            .tool_check("docker", &["info"])
            .folder("Images")
    }

    fn walk() -> ScriptedWalk {
        ScriptedWalk::new().folder("Images", "/home/Images", bytes(10))
    }

    fn module_outcome(events: &[Event]) -> Option<ModuleOutcome> {
        events.iter().find_map(|event| match event {
            Event::ModuleFinished { outcome, .. } => Some(outcome.clone()),
            _ => None,
        })
    }

    fn docker_id() -> ModuleId {
        ModuleId::new("docker").unwrap()
    }

    mod negative {
        use fresher_core::{
            Access, Capability, CapabilityState, CommandError, DangerLevel, Event, GuardedFolder, ModuleOutcome, Need,
            ScanOutcome, SettingValue, SystemUiError, ToolId,
        };
        use fresher_testing::{FakeAccess, FakeCommandRunner, FakeSystemUi};

        use super::{docker, docker_module, module_outcome, walk};
        use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

        #[test]
        fn a_module_whose_tool_is_not_installed_is_not_run() {
            let module = docker_module();
            let seen = module.seen();
            let engine = Fixture::new(walk()).module(module).engine();

            let events = scan(&engine);

            assert_eq!(seen.lock().unwrap().len(), 0);
            assert_eq!(module_outcome(&events), Some(ModuleOutcome::Unavailable { missing: vec![docker()] }));
            assert!(matches!(events.last(), Some(Event::ScanFinished { outcome: ScanOutcome::Completed, .. })));
            let tool = ToolId::new("docker").unwrap();
            let status = engine.access_status();
            assert!(status.capabilities.contains(&CapabilityState::tool_not_installed(tool)));
            assert!(events.contains(&Event::AccessChanged { status }));
        }

        #[test]
        fn a_tool_that_answers_non_zero_is_not_ready() {
            let commands = FakeCommandRunner::default().exiting("docker info", 1);
            let engine = Fixture::new(walk()).module(docker_module()).commands(commands).engine();

            let events = scan(&engine);

            let tool = ToolId::new("docker").unwrap();
            assert!(engine.access_status().capabilities.contains(&CapabilityState::tool_not_ready(tool)));
            assert_eq!(module_outcome(&events), Some(ModuleOutcome::Unavailable { missing: vec![docker()] }));
        }

        #[test]
        fn a_tool_is_not_asked_before_the_first_scan() {
            let engine = Fixture::new(walk()).module(docker_module()).engine();

            assert_eq!(engine.access_status().access(&docker()), Access::NotAsked);
        }

        #[test]
        fn refresh_without_a_change_sends_no_event() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let engine = Fixture::new(ScriptedWalk::new()).access(access).engine();
            let events = engine.subscribe();

            assert!(!engine.refresh_access());

            assert_eq!(events.try_recv().ok(), None);
        }

        #[test]
        fn the_focus_re_check_runs_no_tool_check() {
            let fixture = Fixture::new(walk()).module(docker_module());
            let runner = std::sync::Arc::clone(&fixture.commands);
            let engine = fixture.engine();

            assert!(!engine.refresh_access());

            assert_eq!(runner.calls(), Vec::<String>::new());
            assert_eq!(engine.access_status().access(&docker()), Access::NotAsked);
        }

        #[test]
        fn checking_again_without_a_change_sends_no_event() {
            let commands = FakeCommandRunner::default().answering("docker info", "27.0");
            let engine = Fixture::new(walk()).module(docker_module()).commands(commands).engine();
            assert!(engine.check_access_again());
            let events = engine.subscribe();

            assert!(!engine.check_access_again());

            assert_eq!(events.try_recv().ok(), None);
        }

        #[test]
        fn opening_settings_for_a_capability_this_system_lacks_is_refused() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let fixture = Fixture::new(ScriptedWalk::new()).access(access);
            let system_ui = std::sync::Arc::clone(&fixture.system_ui);
            let engine = fixture.engine();
            let desktop = Capability::Folder(GuardedFolder::Desktop);

            assert_eq!(
                engine.open_access_settings(&desktop),
                Err(CommandError::NoSettingsPane { capability: desktop })
            );
            assert_eq!(
                engine.open_access_settings(&docker()),
                Err(CommandError::NoSettingsPane { capability: docker() })
            );
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn a_system_without_the_pane_or_failing_is_refused() {
            let access = || FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let unsupported = FakeSystemUi::failing(SystemUiError::Unsupported);
            let failing = FakeSystemUi::failing(SystemUiError::Failed { reason: "broken".into() });
            let without = Fixture::new(ScriptedWalk::new()).access(access()).system_ui(unsupported).engine();
            let broken = Fixture::new(ScriptedWalk::new()).access(access()).system_ui(failing).engine();

            assert_eq!(
                without.open_access_settings(&Capability::AllFiles),
                Err(CommandError::NoSettingsPane { capability: Capability::AllFiles })
            );
            assert_eq!(broken.open_access_settings(&Capability::AllFiles), Err(CommandError::Internal));
        }

        #[test]
        fn a_limited_category_has_partial_totals() {
            let module = TestModule::builder("alpha")
                .needing("alpha.caches", DangerLevel::Safe, vec![Need::sees_more(Capability::AllFiles)])
                .folder("Caches");
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(10));
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let engine = Fixture::new(walk).module(module).access(access).engine();

            let events = scan(&engine);

            let totals = engine.overview().unwrap().totals;
            assert_eq!(totals.len(), 1);
            assert_eq!(totals[0].sizes.completeness, fresher_core::Completeness::Partial);
            assert_eq!(module_outcome(&events), Some(ModuleOutcome::Completed));
        }

        #[test]
        fn the_opt_in_switches_are_off_by_default() {
            let engine = Fixture::new(ScriptedWalk::new()).engine();

            let view = engine.settings().unwrap();

            let opt_ins: Vec<_> =
                view.values.iter().filter(|(key, _)| key.as_str().starts_with("core.access.")).collect();
            assert_eq!(opt_ins.len(), 4);
            assert!(opt_ins.iter().all(|(_, value)| **value == SettingValue::Switch(false)));
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{
            Access, Availability, Capability, Completeness, DangerLevel, Event, GuardedFolder, ModuleOutcome, Need,
        };
        use fresher_testing::{FakeAccess, FakeCommandRunner, SystemUiCall};

        use super::{docker, docker_id, docker_module, module_outcome, walk};
        use crate::access::AccessView;
        use crate::testing::{Fixture, ScriptedWalk, TestModule, scan};

        #[test]
        fn refresh_after_a_grant_sends_access_changed() {
            let fixture =
                Fixture::new(ScriptedWalk::new()).access(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]));
            let access = Arc::clone(&fixture.access);
            let engine = fixture.engine();
            let events = engine.subscribe();

            access.set(Capability::AllFiles, Access::Granted);

            assert!(engine.refresh_access());
            assert_eq!(engine.access_status().access(&Capability::AllFiles), Access::Granted);
            assert_eq!(events.try_recv().ok(), Some(Event::AccessChanged { status: engine.access_status() }));
        }

        #[test]
        fn checking_again_runs_every_tool_check_and_sends_access_changed() {
            let commands = FakeCommandRunner::default().answering("docker info", "27.0");
            let fixture = Fixture::new(walk()).module(docker_module()).commands(commands);
            let runner = Arc::clone(&fixture.commands);
            let engine = fixture.engine();
            let events = engine.subscribe();

            assert!(engine.check_access_again());

            assert_eq!(runner.calls(), ["docker info"]);
            assert_eq!(engine.access_status().access(&docker()), Access::Granted);
            assert_eq!(events.try_recv().ok(), Some(Event::AccessChanged { status: engine.access_status() }));
            assert_eq!(events.try_recv().ok(), None);
        }

        #[test]
        fn a_ready_tool_lets_its_module_run() {
            let module = docker_module();
            let seen = module.seen();
            let commands = FakeCommandRunner::default().answering("docker info", "27.0");
            let fixture = Fixture::new(walk()).module(module).commands(commands);
            let runner = Arc::clone(&fixture.commands);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(seen.lock().unwrap().len(), 1);
            assert_eq!(module_outcome(&events), Some(ModuleOutcome::Completed));
            assert_eq!(engine.access_status().access(&docker()), Access::Granted);
            assert_eq!(runner.calls(), ["docker info"]);
            assert_eq!(engine.overview().unwrap().totals[0].sizes.completeness, Completeness::Complete);
        }

        #[test]
        fn open_access_settings_reaches_the_system_ui_port() {
            let desktop = Capability::Folder(GuardedFolder::Desktop);
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (desktop, Access::NotAsked)]);
            let fixture = Fixture::new(ScriptedWalk::new()).access(access);
            let system_ui = Arc::clone(&fixture.system_ui);
            let engine = fixture.engine();

            engine.open_access_settings(&Capability::AllFiles).unwrap();
            engine.open_access_settings(&desktop).unwrap();

            assert_eq!(
                system_ui.calls(),
                [SystemUiCall::OpenAccessSettings(Capability::AllFiles), SystemUiCall::OpenAccessSettings(desktop)]
            );
        }

        #[test]
        fn learn_records_an_answer_the_probe_did_not_list() {
            let downloads = Capability::Folder(GuardedFolder::Downloads);
            let probe = Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]));
            let view = AccessView::new(probe, Arc::new(FakeCommandRunner::default()), std::iter::empty());

            assert!(view.learn(&[(downloads, Access::Granted)]));

            assert_eq!(view.status().access(&downloads), Access::Granted);
            assert_eq!(view.status().access(&Capability::AllFiles), Access::Denied);
            assert!(!view.learn(&[(downloads, Access::Granted)]));
        }

        #[test]
        fn modules_report_the_availability_of_each_category() {
            let alpha = TestModule::builder("alpha")
                .needing("alpha.caches", DangerLevel::Safe, vec![Need::sees_more(Capability::AllFiles)])
                .category("alpha.logs", DangerLevel::Safe);
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
            let engine = Fixture::new(walk()).module(alpha).module(docker_module()).access(access).engine();

            let modules = engine.modules();

            let availability: Vec<Vec<Availability>> = modules
                .iter()
                .map(|module| module.categories.iter().map(|category| category.availability.clone()).collect())
                .collect();
            assert_eq!(
                availability,
                [
                    vec![Availability::Limited { missing: vec![Capability::AllFiles] }, Availability::Ready],
                    vec![Availability::Unavailable { missing: vec![docker()] }],
                ]
            );
            assert_eq!(modules[1].id, docker_id());
            assert!(format!("{engine:?}").contains("AccessView"));
        }
    }
}
