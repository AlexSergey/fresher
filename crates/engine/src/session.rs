//! One scan: the shared walk for every enabled module, each match to the module that asked for it, then every
//! module's `finish`. Runs on the scan's own thread and reports only through the bus.

use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{
    CancelToken, Event, Finding, FindingId, FindingSink, InterestSet, LastScan, Match, Module, ModuleContext,
    ModuleError, ModuleId, ModuleOutcome, ModuleSettings, NewFinding, ScanId, ScanOutcome, Timestamp, WalkError,
    WalkProgress,
};

use crate::batch::Batcher;
use crate::engine::Shared;
use crate::error::lock_anyway;
use crate::throttle::Throttle;

/// What a scan runs, fixed when it starts: later changes to the settings wait for the next scan.
pub(crate) struct ScanPlan {
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) modules: Vec<Planned>,
}

/// An enabled module and its settings.
pub(crate) struct Planned {
    pub(crate) module: Arc<dyn Module>,
    pub(crate) settings: ModuleSettings,
}

/// Runs `plan` to its end: `ScanStarted`, progress and findings, one `ModuleFinished` per module, `ScanFinished`.
///
/// A module that panics is caught and counts as failed; any other panic still ends the scan `Partial` (see
/// [`Output`]'s `Drop`), so the engine never stays `Scanning`.
pub(crate) fn run(shared: &Shared, scan: ScanId, cancel: &CancelToken, plan: &ScanPlan) {
    let ids: Vec<ModuleId> = plan.modules.iter().map(|planned| planned.module.descriptor().id()).collect();
    // The previous scan's findings are forgotten only now that this scan runs; `None` when the ids ran out.
    let first_id = lock_anyway(&shared.results).begin();
    shared.bus.publish(&Event::ScanStarted { scan, modules: ids.clone() });

    let mut interests = InterestSet::new();
    for planned in &plan.modules {
        interests.add(planned.module.descriptor().id(), planned.module.interests());
    }
    let (clock, commands) = (&*shared.ports.clock, &*shared.ports.commands);
    let contexts: Vec<ModuleContext<'_>> =
        plan.modules.iter().map(|planned| ModuleContext::new(clock, commands, &planned.settings, cancel)).collect();
    let mut outcomes: Vec<Option<ModuleOutcome>> = vec![None; plan.modules.len()];
    let mut output =
        Output { shared, scan, next_id: first_id, batcher: Batcher::default(), exhausted: false, ended: false };

    let mut throttle = Throttle::default();
    let publish_progress = |progress: WalkProgress| {
        shared.bus.publish(&Event::ScanProgress { scan, entries: progress.entries, bytes: progress.bytes });
    };
    let walked = {
        let mut on_match = |found: Match| {
            let Some((owner, local)) = interests.owner(found.interest) else { return };
            let Some(index) = ids.iter().position(|&id| id == owner) else { return };
            if outcomes[index].is_some() {
                return;
            }
            let found = Match { interest: local, ..found };
            let module = &plan.modules[index].module;
            let called = isolated(|| module.on_match(&found, &contexts[index], &mut output));
            if let Err(outcome) = called {
                outcomes[index] = Some(output.module_finished(owner, outcome));
            }
        };
        let mut on_progress = |progress: WalkProgress| {
            if let Some(due) = throttle.offer(progress, clock.now()) {
                publish_progress(due);
            }
        };
        shared.ports.walk.walk(&plan.roots, &interests, cancel, &mut on_match, &mut on_progress)
    };
    if let Some(last) = throttle.take() {
        publish_progress(last);
    }
    let walk_complete = match walked {
        Ok(summary) => summary.is_complete(),
        Err(WalkError::Cancelled) => false,
        Err(error @ WalkError::RootUnreadable { .. }) => {
            tracing::warn!(%error, "the walk stopped early");
            false
        }
    };

    for (index, planned) in plan.modules.iter().enumerate() {
        if outcomes[index].is_some() {
            continue;
        }
        let outcome = if cancel.is_cancelled() {
            ModuleOutcome::Cancelled
        } else {
            isolated(|| planned.module.finish(&contexts[index], &mut output))
                .map_or_else(|outcome| outcome, |()| ModuleOutcome::Completed)
        };
        outcomes[index] = Some(output.module_finished(ids[index], outcome));
    }

    let every_module_completed = outcomes.iter().all(|outcome| outcome == &Some(ModuleOutcome::Completed));
    let complete = walk_complete && every_module_completed && !output.exhausted && !cancel.is_cancelled();
    let outcome = if complete { ScanOutcome::Completed } else { ScanOutcome::Partial };
    output.end(outcome);
}

/// Calls a module; its error or its panic becomes the outcome it ends with. The panic does not reach the scan: what
/// the module left behind is only the findings it already pushed, which are kept like those of a failing module.
fn isolated(call: impl FnOnce() -> Result<(), ModuleError>) -> Result<(), ModuleOutcome> {
    match panic::catch_unwind(AssertUnwindSafe(call)) {
        Ok(result) => result.map_err(outcome_of),
        Err(_) => Err(ModuleOutcome::panicked()),
    }
}

fn outcome_of(error: ModuleError) -> ModuleOutcome {
    match error {
        ModuleError::Cancelled => ModuleOutcome::Cancelled,
        ModuleError::Failed { reason } => ModuleOutcome::Failed { reason },
    }
}

/// Where the modules' findings go: numbered, batched, kept in the results and published. It also ends the scan, and
/// if it is dropped before (a panic outside the modules), its `Drop` ends the scan `Partial`.
struct Output<'a> {
    shared: &'a Shared,
    scan: ScanId,
    next_id: Option<u32>,
    batcher: Batcher,
    exhausted: bool,
    ended: bool,
}

impl Output<'_> {
    /// Records the last scan, publishes `ScanFinished` and lets the engine go `Idle`, all under the state's lock.
    fn end(&mut self, outcome: ScanOutcome) {
        self.ended = true;
        let finished = Timestamp::from_system_time(self.shared.ports.clock.now());
        let mut machine = lock_anyway(&self.shared.machine);
        let last = LastScan { scan: self.scan, finished, outcome: outcome.clone() };
        lock_anyway(&self.shared.results).finish(last, self.next_id);
        self.shared.bus.publish(&Event::ScanFinished { scan: self.scan, outcome });
        machine.finish(self.scan);
    }

    /// Sends what the module left in the open batch, then its `ModuleFinished`.
    fn module_finished(&mut self, module: ModuleId, outcome: ModuleOutcome) -> ModuleOutcome {
        if let Some(batch) = self.batcher.take() {
            self.send(batch);
        }
        if let ModuleOutcome::Failed { reason } = &outcome {
            tracing::warn!(module = module.as_str(), reason = reason.key().as_str(), "a module failed");
        }
        self.shared.bus.publish(&Event::ModuleFinished { scan: self.scan, module, outcome: outcome.clone() });
        outcome
    }

    fn send(&self, batch: Vec<Finding>) {
        lock_anyway(&self.shared.results).append(&batch);
        self.shared.bus.publish(&Event::FindingsAdded { scan: self.scan, findings: batch });
    }
}

impl Drop for Output<'_> {
    fn drop(&mut self) {
        if !self.ended {
            tracing::error!(scan = self.scan.get(), "the scan stopped on a panic");
            self.end(ScanOutcome::Partial);
        }
    }
}

impl FindingSink for Output<'_> {
    fn push(&mut self, finding: NewFinding) {
        let Some(id) = self.next_id else {
            self.exhausted = true;
            return;
        };
        self.next_id = id.checked_add(1);
        let now = self.shared.ports.clock.now();
        if let Some(batch) = self.batcher.push(Finding::new(FindingId::new(id), finding), now) {
            self.send(batch);
        }
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Event, FindingId, ModuleId, ModuleOutcome, ScanOutcome};

    /// The outcome of every `ModuleFinished` in `events`, by module.
    fn module_outcomes(events: &[Event]) -> Vec<(ModuleId, ModuleOutcome)> {
        let finished = events.iter().filter_map(|event| match event {
            Event::ModuleFinished { module, outcome, .. } => Some((*module, outcome.clone())),
            _ => None,
        });
        finished.collect()
    }

    fn scan_outcome(events: &[Event]) -> Option<ScanOutcome> {
        match events.last() {
            Some(Event::ScanFinished { outcome, .. }) => Some(outcome.clone()),
            _ => None,
        }
    }

    fn ids(events: &[Event]) -> Vec<u32> {
        crate::testing::batches(events).iter().flatten().map(|finding| finding.id().get()).collect()
    }

    fn id(id: u32) -> FindingId {
        FindingId::new(id)
    }

    mod negative {
        use fresher_core::{
            CommandError, DangerLevel, EngineStatus, Event, FindingsQuery, ModuleError, ModuleId, ModuleOutcome,
            ScanOutcome, WalkError,
        };
        use serde_json::json;

        use super::{id, ids, module_outcomes, scan_outcome};
        use crate::testing::{
            Fixture, MemorySettings, ScriptedWalk, TestModule, bytes, category, finding, panic_now, scan, text,
            wait_for_finish,
        };

        fn alpha() -> ModuleId {
            ModuleId::new("alpha").unwrap()
        }

        fn beta() -> ModuleId {
            ModuleId::new("beta").unwrap()
        }

        #[test]
        fn a_failing_module_does_not_fail_the_scan() {
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/a/Caches", bytes(1))
                .folder("Caches", "/home/b/Caches", bytes(1))
                .folder("node_modules", "/home/app/node_modules", bytes(2));
            let failing = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let failing = failing.failing_on_match();
            let seen = failing.seen();
            let working = TestModule::builder("beta").category("beta.deps", DangerLevel::Low).folder("node_modules");
            let engine = Fixture::new(walk).module(failing).module(working).engine();

            let events = scan(&engine);

            assert_eq!(seen.lock().unwrap().len(), 1);
            let reason = text("test.module.failed");
            assert_eq!(
                module_outcomes(&events),
                vec![(alpha(), ModuleOutcome::Failed { reason }), (beta(), ModuleOutcome::Completed)]
            );
            assert_eq!(ids(&events), [1]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
        }

        #[test]
        fn a_module_panicking_on_a_match_fails_alone() {
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/a/Caches", bytes(1))
                .folder("Caches", "/home/b/Caches", bytes(1))
                .folder("node_modules", "/home/app/node_modules", bytes(2));
            let panicking = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let panicking = panicking.panicking_on_match();
            let seen = panicking.seen();
            let working = TestModule::builder("beta").category("beta.deps", DangerLevel::Low).folder("node_modules");
            let engine = Fixture::new(walk).module(panicking).module(working).engine();

            let events = scan(&engine);

            assert_eq!(seen.lock().unwrap().len(), 1);
            assert_eq!(
                module_outcomes(&events),
                vec![(alpha(), ModuleOutcome::panicked()), (beta(), ModuleOutcome::Completed)]
            );
            assert_eq!(ids(&events), [1]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
        }

        #[test]
        fn a_module_panicking_in_finish_fails_alone_and_keeps_what_it_found() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let panicking =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/demo/before", bytes(1)));
                    panic_now("a module panics in finish")
                });
            let engine =
                Fixture::new(ScriptedWalk::new()).module(panicking).module(TestModule::builder("beta")).engine();

            let events = scan(&engine);

            assert_eq!(
                module_outcomes(&events),
                vec![(alpha(), ModuleOutcome::panicked()), (beta(), ModuleOutcome::Completed)]
            );
            assert_eq!(ids(&events), [1]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
        }

        #[test]
        fn a_panic_outside_the_modules_still_ends_the_scan_partial() {
            let walk = ScriptedWalk::new().panicking();
            let engine = Fixture::new(walk).module(TestModule::builder("alpha")).engine();

            let events = scan(&engine);

            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Partial));
            assert!(engine.start_scan().is_ok());
        }

        #[test]
        fn holds_walk_progress_back_while_the_clock_stands_still() {
            let walk = (1..=5).fold(ScriptedWalk::new(), |walk, entries| walk.progress(entries, entries * 10));
            let engine = Fixture::new(walk).engine();

            let events = scan(&engine);

            let progress: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    Event::ScanProgress { entries, .. } => Some(*entries),
                    _ => None,
                })
                .collect();
            assert_eq!(progress, [1, 5]);
        }

        #[test]
        fn a_module_failing_or_cancelled_in_finish_is_reported() {
            let failing = TestModule::builder("alpha")
                .on_finish(|_, _| Err(ModuleError::Failed { reason: text("alpha.error.broken") }));
            let cancelled = TestModule::builder("beta").on_finish(|_, _| Err(ModuleError::Cancelled));
            let engine = Fixture::new(ScriptedWalk::new()).module(failing).module(cancelled).engine();

            let events = scan(&engine);

            let reason = text("alpha.error.broken");
            assert_eq!(
                module_outcomes(&events),
                vec![(alpha(), ModuleOutcome::Failed { reason }), (beta(), ModuleOutcome::Cancelled)]
            );
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
        }

        #[test]
        fn a_disabled_module_gets_no_matches() {
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let disabled = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let seen = disabled.seen();
            let enabled = TestModule::builder("beta");
            let settings = MemorySettings::with(&[("alpha.enabled", json!(false))]);
            let engine = Fixture::new(walk).module(disabled).module(enabled).settings(settings).engine();

            let events = scan(&engine);

            assert!(seen.lock().unwrap().is_empty());
            assert_eq!(
                events.first(),
                Some(&Event::ScanStarted { scan: fresher_core::ScanId::new(1), modules: vec![beta()] })
            );
            assert_eq!(module_outcomes(&events), vec![(beta(), ModuleOutcome::Completed)]);
        }

        #[test]
        fn refuses_an_unknown_finding_id() {
            let engine = Fixture::new(ScriptedWalk::new()).engine();

            assert_eq!(engine.finding(id(999)), Err(CommandError::UnknownFinding { id: id(999) }));
        }

        #[test]
        fn an_id_from_the_previous_scan_is_unknown() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/demo/cache", bytes(1)));
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

            let first = ids(&scan(&engine));
            let second = ids(&scan(&engine));

            assert_eq!((first, second), (vec![1], vec![2]));
            assert_eq!(engine.finding(id(1)), Err(CommandError::UnknownFinding { id: id(1) }));
            assert_eq!(engine.finding(id(2)).unwrap().id(), id(2));
        }

        #[test]
        fn a_cancelled_scan_still_publishes_scan_finished_partial() {
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let (walk, _gate) = walk.hold();
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let engine = Fixture::new(walk).module(module).engine();
            let events = engine.subscribe();
            engine.start_scan().unwrap();

            engine.cancel_scan().unwrap();

            let events = wait_for_finish(&events);
            assert_eq!(module_outcomes(&events), vec![(alpha(), ModuleOutcome::Cancelled)]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
            assert_eq!(engine.finding(id(1)).unwrap().module(), alpha());
        }

        #[test]
        fn a_walk_that_could_not_read_everything_ends_partial() {
            let unreadable = Fixture::new(ScriptedWalk::new().unreadable(3)).engine();
            let root = WalkError::RootUnreadable { index: 0, kind: std::io::ErrorKind::PermissionDenied };
            let stopped = Fixture::new(ScriptedWalk::new().failing(root)).module(TestModule::builder("alpha")).engine();

            let unreadable = scan(&unreadable);
            let stopped = scan(&stopped);

            assert_eq!(scan_outcome(&unreadable), Some(ScanOutcome::Partial));
            assert_eq!(scan_outcome(&stopped), Some(ScanOutcome::Partial));
            assert_eq!(module_outcomes(&stopped), vec![(alpha(), ModuleOutcome::Completed)]);
        }

        #[test]
        fn findings_past_the_last_id_are_dropped_and_the_scan_is_partial() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/demo/last", bytes(1)));
                    out.push(finding(&caches, "/demo/dropped", bytes(1)));
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();
            engine.shared.results.lock().unwrap().set_next_id(Some(u32::MAX));

            let events = scan(&engine);

            assert_eq!(ids(&events), [u32::MAX]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Partial));
        }

        #[test]
        fn refuses_a_query_for_an_unknown_module() {
            let engine = Fixture::new(ScriptedWalk::new()).module(TestModule::builder("alpha")).engine();
            let query = FindingsQuery::new(Some("gamma".into()), None, 0, 10);

            assert_eq!(engine.findings(&query), Err(CommandError::UnknownModule { id: "gamma".into() }));
        }

        #[test]
        fn a_category_nobody_declared_matches_nothing() {
            let engine = Fixture::new(ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1)))
                .module(TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches"))
                .engine();
            scan(&engine);

            let page = engine.findings(&FindingsQuery::new(None, Some("alpha.unknown".into()), 0, 10)).unwrap();

            assert_eq!((page.findings.len(), page.total), (0, 0));
        }
    }

    mod positive {
        use std::time::Duration;

        use fresher_core::{
            ByteSize, Completeness, DangerLevel, EngineStatus, Event, FindingsQuery, LastScan, ScanId, ScanOutcome,
            Sizes, Timestamp,
        };

        use super::{id, ids, scan_outcome};
        use crate::testing::{
            Fixture, MemorySettings, START, ScriptedWalk, TestModule, batches, bytes, category, finding, scan,
        };

        #[test]
        fn sends_each_match_to_the_module_that_asked_for_it() {
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/Caches", bytes(1))
                .folder("node_modules", "/home/app/node_modules", bytes(2))
                .folder("target", "/home/app/target", bytes(3));
            let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let alpha_seen = alpha.seen();
            let beta = TestModule::builder("beta").category("beta.builds", DangerLevel::Low);
            let beta = beta.folder("node_modules").folder("target");
            let beta_seen = beta.seen();
            let engine = Fixture::new(walk).module(alpha).module(beta).engine();

            let events = scan(&engine);

            let alpha_seen: Vec<_> = alpha_seen.lock().unwrap().iter().map(|m| (m.interest, m.path.clone())).collect();
            let beta_seen: Vec<_> = beta_seen.lock().unwrap().iter().map(|m| (m.interest, m.path.clone())).collect();
            assert_eq!(alpha_seen, [(0, "/home/Caches".into())]);
            assert_eq!(beta_seen, [(0, "/home/app/node_modules".into()), (1, "/home/app/target".into())]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Completed));
        }

        #[test]
        fn assigns_finding_ids_in_order() {
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/a/Caches", bytes(1))
                .folder("Caches", "/home/b/Caches", bytes(1))
                .folder("Caches", "/home/c/Caches", bytes(1));
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            assert_eq!(ids(&events), [1, 2, 3]);
            let path = engine.finding(id(3)).unwrap().finding().subject().clone();
            assert_eq!(path, fresher_core::Subject::Path("/home/c/Caches".into()));
        }

        #[test]
        fn batches_findings_instead_of_one_event_each() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    for index in 0..1_000 {
                        out.push(finding(&caches, format!("/demo/{index}"), bytes(1)));
                    }
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

            let events = scan(&engine);

            let sizes: Vec<_> = batches(&events).iter().map(Vec::len).collect();
            assert_eq!(sizes, [256, 256, 256, 232]);
        }

        #[test]
        fn flushes_a_batch_when_the_clock_moves_50_ms() {
            let fixture = Fixture::new(ScriptedWalk::new());
            let clock = std::sync::Arc::clone(&fixture.clock);
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/demo/1", bytes(1)));
                    clock.advance(Duration::from_millis(50));
                    out.push(finding(&caches, "/demo/2", bytes(1)));
                    out.push(finding(&caches, "/demo/3", bytes(1)));
                    Ok(())
                });
            let engine = fixture.module(module).engine();

            let events = scan(&engine);

            let sizes: Vec<_> = batches(&events).iter().map(Vec::len).collect();
            assert_eq!(sizes, [2, 1]);
        }

        #[test]
        fn overview_sums_sizes_and_takes_the_highest_danger() {
            let caches = category("alpha.caches", DangerLevel::Low);
            let logs = category("alpha.logs", DangerLevel::Safe);
            let module = TestModule::builder("alpha")
                .category("alpha.caches", DangerLevel::Low)
                .category("alpha.logs", DangerLevel::Safe)
                .category("alpha.empty", DangerLevel::Safe)
                .on_finish(move |_, out| {
                    out.push(finding(&logs, "/demo/log", bytes(5)));
                    out.push(finding(&caches, "/demo/a", bytes(100)));
                    let mut risky = finding(&caches, "/demo/b", bytes(200));
                    risky.raise_danger(DangerLevel::High);
                    out.push(risky);
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();
            scan(&engine);

            let overview = engine.overview().unwrap();

            assert_eq!(overview.status, EngineStatus::Idle);
            let finished = Timestamp::from_secs(START);
            let last = LastScan { scan: ScanId::new(1), finished, outcome: ScanOutcome::Completed };
            assert_eq!(overview.last_scan, Some(last));
            let totals: Vec<_> = overview
                .totals
                .iter()
                .map(|total| (total.category.as_str(), total.count, total.sizes, total.highest_danger))
                .collect();
            assert_eq!(
                totals,
                [("alpha.caches", 2, bytes(300), DangerLevel::High), ("alpha.logs", 1, bytes(5), DangerLevel::Safe)]
            );
        }

        #[test]
        fn a_partial_size_makes_its_category_total_partial() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/demo/a", bytes(10)));
                    let partial = Sizes { completeness: Completeness::Partial, ..bytes(20) };
                    out.push(finding(&caches, "/demo/b", partial));
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();
            scan(&engine);

            let totals = engine.overview().unwrap().totals;

            assert_eq!(totals[0].sizes.logical, ByteSize::new(30));
            assert_eq!(totals[0].sizes.completeness, Completeness::Partial);
        }

        #[test]
        fn walks_the_roots_from_the_settings() {
            let default = ScriptedWalk::new();
            let default_roots = default.roots();
            let configured = ScriptedWalk::new();
            let configured_roots = configured.roots();
            let settings =
                MemorySettings::with(&[("core.scan.roots", serde_json::json!(["~/Projects", "/Volumes/Work"]))]);

            scan(&Fixture::new(default).engine());
            scan(&Fixture::new(configured).settings(settings).engine());

            assert_eq!(*default_roots.lock().unwrap(), [std::path::PathBuf::from("~")]);
            let configured: Vec<std::path::PathBuf> = vec!["~/Projects".into(), "/Volumes/Work".into()];
            assert_eq!(*configured_roots.lock().unwrap(), configured);
        }

        #[test]
        fn forwards_the_walk_progress() {
            let engine = Fixture::new(ScriptedWalk::new().progress(10, 4_096).progress(20, 8_192)).engine();

            let events = scan(&engine);

            let progress: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    Event::ScanProgress { entries, bytes, .. } => Some((*entries, bytes.bytes())),
                    _ => None,
                })
                .collect();
            assert_eq!(progress, [(10, 4_096), (20, 8_192)]);
        }

        #[test]
        fn pages_findings_by_module_and_category() {
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/a/Caches", bytes(1))
                .folder("Caches", "/home/b/Caches", bytes(1))
                .folder("Caches", "/home/c/Caches", bytes(1))
                .folder("node_modules", "/home/node_modules", bytes(1));
            let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let beta = TestModule::builder("beta").category("beta.deps", DangerLevel::Safe).folder("node_modules");
            let engine = Fixture::new(walk).module(alpha).module(beta).engine();
            scan(&engine);

            let alpha = engine.findings(&FindingsQuery::new(Some("alpha".into()), None, 1, 1)).unwrap();
            let deps = engine.findings(&FindingsQuery::new(None, Some("beta.deps".into()), 0, 10)).unwrap();
            let all = engine.findings(&FindingsQuery::new(None, None, 0, 10)).unwrap();

            assert_eq!((alpha.offset, alpha.total, alpha.findings[0].id()), (1, 3, id(2)));
            assert_eq!((deps.total, deps.findings[0].id()), (1, id(4)));
            assert_eq!(all.findings.len(), 4);
        }
    }
}
