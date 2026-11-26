//! One scan: the shared walk for every enabled module, each match to the module that asked for it, then every
//! module's `finish`. Runs on the scan's own thread and reports only through the bus.

use std::collections::{BTreeSet, HashMap};
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{
    AccessStatus, Availability, ByteSize, CancelToken, Capability, CategoryId, ClaimedPath, Coverage, EntryFacts,
    Event, Finding, FindingId, FindingSink, Insight, InterestSet, LastScan, LeftOutItem, Match, Module, ModuleContext,
    ModuleError, ModuleId, ModuleOutcome, ModuleSettings, NewFinding, Place, PlaceSeen, ProjectFinder, Removability,
    ScanId, ScanKind, ScanOutcome, SystemFacts, Timestamp, WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
};

use crate::batch::Batcher;
use crate::claims;
use crate::coverage;
use crate::engine::Shared;
use crate::error::lock_anyway;
use crate::estimate;
use crate::left_outs::LeftOuts;
use crate::settings::Ignored;
use crate::storage::attribution::AppPlaces;
use crate::storage::roots::expand_home;
use crate::throttle::Throttle;
use crate::withheld::Withheld;

/// What a scan runs, fixed when it starts: later changes to the settings wait for the next scan.
pub(crate) struct ScanPlan {
    pub(crate) roots: Vec<PathBuf>,
    /// The places walked outside the roots (`crate::places`).
    pub(crate) places: Vec<Place>,
    /// The guarded places the user opted into, by capability.
    pub(crate) opt_ins: Vec<Capability>,
    pub(crate) modules: Vec<Planned>,
    /// How the walk reads the disk.
    pub(crate) mode: WalkMode,
    /// The ignore lists: a left-out of an ignored category is not counted.
    pub(crate) ignored: Ignored,
}

/// An enabled module and its settings.
pub(crate) struct Planned {
    pub(crate) module: Arc<dyn Module>,
    pub(crate) settings: ModuleSettings,
}

/// Runs `plan` to its end. The tool checks run first, then the guarded places are decided (a place not allowed and
/// not opted into is never touched; an opted-in one is tried) and the engine learns what the tries answered; only
/// then is each module's availability decided, so a module needing a place runs in the scan that first tries it. A
/// module whose every category is unavailable is not run; the totals of a limited category are partial. A scan that
/// did not get through its work (cancelled, the walk or a module stopped early, its state not kept, the ids ran out)
/// ends `Stopped`; one that got through but did not see every folder ends `Partial`.
///
/// The projects every module's `finish` sees carry what the walk saw of them: the walk is asked once after it ends
/// and before the commit; a walk that did not end is asked nothing.
///
/// The events, in order: `ScanStarted`; `AccessChanged` when a tool's state changed; `AccessChanged` when a try
/// changed a state; the `ModuleFinished` of each module not run for what it lacks; then the walk's progress and
/// findings, the other modules' `ModuleFinished`, `ScanCoverage` and `ScanFinished` (the walk's state committed
/// before them).
///
/// A module that panics is caught and counts as failed; any other panic still ends the scan `Stopped` (see
/// [`Output`]'s `Drop`), so the engine never stays `Scanning`. A cancelled or stopped walk is not committed: the
/// engine keeps the state it had.
pub(crate) fn run(shared: &Shared, scan: ScanId, cancel: &CancelToken, plan: &ScanPlan) {
    let ids: Vec<ModuleId> = plan.modules.iter().map(|planned| planned.module.descriptor().id()).collect();
    let interests = all_interests(shared);
    // The previous scan's findings are forgotten only now that this scan runs.
    let (first_id, expected) = begin(shared, plan);
    shared.bus.publish(&Event::ScanStarted { scan, modules: ids.clone() });

    let (clock, commands) = (&*shared.ports.clock, &*shared.ports.commands);
    // One scan's facts of the system, shared by every module and gathered only when one asks.
    let system = SystemFacts::new(&shared.ports.system);
    let contexts: Vec<ModuleContext<'_>> = plan
        .modules
        .iter()
        .map(|planned| ModuleContext::new(clock, commands, &system, &planned.settings, cancel))
        .collect();
    let mut outcomes: Vec<Option<ModuleOutcome>> = vec![None; plan.modules.len()];
    let mut output = Output::new(shared, scan, first_id, plan);

    let (entry, unavailable) = decide_access(shared, plan, cancel);
    for (index, missing) in unavailable.into_iter().enumerate() {
        if let Some(missing) = missing {
            outcomes[index] = Some(output.module_finished(index, ids[index], ModuleOutcome::Unavailable { missing }));
        }
    }

    let mut throttle = Throttle::default();
    let publish_progress = |progress: WalkProgress| {
        shared.bus.publish(&Event::ScanProgress { scan, entries: progress.entries, bytes: progress.bytes, expected });
    };
    let mut projects = ProjectFinder::default();
    let walked = {
        let mut on_match = |found: Match| {
            let Some((owner, local)) = interests.owner(found.interest) else { return };
            if owner == ModuleId::system() {
                projects.add(local, &found);
                return;
            }
            // A module that is off is not in the plan, one not run for what it lacks has its outcome: no match.
            let Some(index) = ids.iter().position(|&id| id == owner) else { return };
            if outcomes[index].is_some() {
                return;
            }
            output.facts.insert(found.path.clone(), found.facts);
            let found = Match { interest: local, ..found };
            let module = &plan.modules[index].module;
            let called = isolated(|| module.on_match(&found, &contexts[index], &mut output.sink(index)));
            if let Err(outcome) = called {
                outcomes[index] = Some(output.module_finished(index, owner, outcome));
            }
        };
        let mut on_progress = |progress: WalkProgress| {
            if let Some(due) = throttle.offer(progress, clock.now()) {
                publish_progress(due);
            }
        };
        let (roots, avoid, places) = (&plan.roots, &entry.avoid, &plan.places);
        let request = WalkRequest { roots, interests: &interests, avoid, places, mode: plan.mode };
        shared.ports.walk.walk(&request, cancel, &mut on_match, &mut on_progress)
    };
    throttle.take().into_iter().for_each(&publish_progress);
    let ended = walked.is_ok().then_some(&*shared.ports.walk);
    system.set_projects(projects.finish(ended));
    let summary = summary_of(walked);
    output.coverage = coverage::after_walk(entry.not_entered, summary.as_ref());
    if let Some(summary) = &summary {
        output.kind = summary.kind;
        if let Some(reason) = &summary.fallback {
            tracing::info!(?reason, "the rescan read every folder");
        }
        *lock_anyway(&shared.fallback) = Some((scan, summary.fallback.clone()));
    }

    let walk_complete = summary.as_ref().is_some_and(WalkSummary::is_complete);
    let places = places_of(plan, &interests, summary.as_ref());
    finish_modules(plan, &contexts, walk_complete, &places, &mut outcomes, &mut output, cancel);

    let committed = !cancel.is_cancelled() && {
        let claims = resolve_claims(shared);
        summary.is_some() && commit(shared, &claims, &system)
    };
    let through = committed && every_module_completed(&outcomes) && !output.exhausted && !cancel.is_cancelled();
    let outcome = if !through {
        ScanOutcome::Stopped
    } else if walk_complete && output.coverage.is_complete() {
        ScanOutcome::Completed
    } else {
        ScanOutcome::Partial
    };
    output.end(outcome);
}

/// Forgets the previous scan's findings for this scan's, with the volumes of its roots and places read first: a map
/// query never sees this scan's findings with the last scan's volumes. Gives the scan's first id (`None` when the ids
/// ran out) and its estimate, taken from the last committed state before it is replaced (`crate::estimate`).
fn begin(shared: &Shared, plan: &ScanPlan) -> (Option<u32>, Option<ByteSize>) {
    let roots = expand_home(&plan.roots, shared.home().as_deref());
    let mut walked = roots.clone();
    walked.extend(plan.places.iter().map(|place| place.path.clone()));
    let volumes = shared.read_volumes(&walked);
    let mut results = lock_anyway(&shared.results);
    let expected = estimate::expected(results.state().as_deref(), &roots, &volumes);
    (results.begin(&plan.roots, &plan.places, volumes), expected)
}

/// Every registered module's interests, enabled or not, so turning one off never changes what the walk reads (design
/// call 22; the matches of a module that is off are dropped), then the project finder's, owned by
/// `ModuleId::system()`.
fn all_interests(shared: &Shared) -> InterestSet {
    let mut interests = InterestSet::new();
    for module in shared.registry.registered() {
        interests.add(module.descriptor().id(), module.interests());
    }
    interests.add(ModuleId::system(), &ProjectFinder::interests());
    interests
}

/// What the walk saw of the place of each planned module's interests, by the index in the module's own `interests()`:
/// `Unknown` for an interest the walk said nothing of (one without one place, or no summary).
fn places_of(plan: &ScanPlan, interests: &InterestSet, summary: Option<&WalkSummary>) -> Vec<Vec<PlaceSeen>> {
    let mut places: Vec<Vec<PlaceSeen>> =
        plan.modules.iter().map(|planned| vec![PlaceSeen::Unknown; planned.module.interests().len()]).collect();
    for (&interest, &seen) in summary.iter().flat_map(|summary| &summary.places) {
        let Some((owner, local)) = interests.owner(interest) else { continue };
        let Some(index) = plan.modules.iter().position(|planned| planned.module.descriptor().id() == owner) else {
            continue;
        };
        if let Some(slot) = places[index].get_mut(local) {
            *slot = seen;
        }
    }
    places
}

/// Calls `finish` of every module that has no outcome yet, in order, each told whether the walk read every folder and
/// what it saw of the places of the module's own interests; after a cancel, each is `Cancelled` instead.
fn finish_modules(
    plan: &ScanPlan,
    contexts: &[ModuleContext<'_>],
    walk_complete: bool,
    places: &[Vec<PlaceSeen>],
    outcomes: &mut [Option<ModuleOutcome>],
    output: &mut Output<'_>,
    cancel: &CancelToken,
) {
    for (index, ((planned, cx), outcome)) in plan.modules.iter().zip(contexts).zip(outcomes.iter_mut()).enumerate() {
        if outcome.is_some() {
            continue;
        }
        let finished = if cancel.is_cancelled() {
            ModuleOutcome::Cancelled
        } else {
            let cx = cx.with_places(places.get(index).map_or(&[], Vec::as_slice));
            let cx = if walk_complete { cx.complete_walk() } else { cx };
            let finished = isolated(|| planned.module.finish(&cx, &mut output.sink(index)));
            finished.map_or_else(|outcome| outcome, |()| ModuleOutcome::Completed)
        };
        *outcome = Some(output.module_finished(index, planned.module.descriptor().id(), finished));
    }
}

/// The tool checks, the guarded places (the engine learns what their tries answered), then what each module lacks:
/// the places to avoid and the modules not run. An `AccessChanged` for the tools, then one for the places, each only
/// when a state changed.
fn decide_access(
    shared: &Shared,
    plan: &ScanPlan,
    cancel: &CancelToken,
) -> (coverage::Entry, Vec<Option<Vec<Capability>>>) {
    check_tools(shared, plan, cancel);
    let access = coverage::as_opted(&shared.access.status(), &plan.opt_ins);
    let entry = coverage::before_walk(&*shared.ports.access, &access, &plan.roots, &plan.opt_ins);
    if shared.access.learn(&entry.tried) {
        shared.bus.publish(&Event::AccessChanged { status: shared.access.status() });
    }
    (entry, decide_modules(shared, plan))
}

/// Checks the tools of the scan's modules; `AccessChanged` when a tool's state changed.
fn check_tools(shared: &Shared, plan: &ScanPlan, cancel: &CancelToken) {
    let checks = plan.modules.iter().flat_map(|planned| planned.module.descriptor().tool_checks());
    if shared.access.check_tools(checks, cancel) {
        shared.bus.publish(&Event::AccessChanged { status: shared.access.status() });
    }
}

/// Decides with the access as last learnt, as the switches allow it, what each module lacks to run at all (`None`
/// when it can run) and which categories are limited.
fn decide_modules(shared: &Shared, plan: &ScanPlan) -> Vec<Option<Vec<Capability>>> {
    let access = coverage::as_opted(&shared.access.status(), &plan.opt_ins);
    lock_anyway(&shared.results).limit(limited(plan, &access));
    plan.modules.iter().map(|planned| unavailable(&*planned.module, &access)).collect()
}

/// Every module completed; a module not run for what it lacks says so itself, so it does not stop the scan.
fn every_module_completed(outcomes: &[Option<ModuleOutcome>]) -> bool {
    outcomes.iter().all(|outcome| matches!(outcome, Some(ModuleOutcome::Completed | ModuleOutcome::Unavailable { .. })))
}

/// The walk's summary; `None` when it was cancelled or stopped early (logged).
fn summary_of(walked: Result<WalkSummary, WalkError>) -> Option<WalkSummary> {
    match walked {
        Ok(summary) => Some(summary),
        Err(WalkError::Cancelled) => None,
        Err(error) => {
            tracing::warn!(%error, "the walk stopped early");
            None
        }
    }
}

/// What `module` lacks when every one of its categories is unavailable: their missing capabilities, once each. `None`
/// when it can run (a module without categories always can).
fn unavailable(module: &dyn Module, access: &AccessStatus) -> Option<Vec<Capability>> {
    let categories = module.descriptor().categories();
    let mut missing: Vec<Capability> = Vec::new();
    for category in categories {
        let Availability::Unavailable { missing: lacks } = Availability::of(category.needs(), access) else {
            return None;
        };
        for capability in lacks {
            if !missing.contains(&capability) {
                missing.push(capability);
            }
        }
    }
    (!categories.is_empty()).then_some(missing)
}

/// The categories of the scan's modules that are limited by what the access lacks.
fn limited(plan: &ScanPlan, access: &AccessStatus) -> BTreeSet<CategoryId> {
    let categories = plan.modules.iter().flat_map(|planned| planned.module.descriptor().categories());
    let limited = categories
        .filter(|category| matches!(Availability::of(category.needs(), access), Availability::Limited { .. }));
    limited.map(fresher_core::CategoryDescriptor::id).collect()
}

/// Resolves the claims of the scan's findings and keeps them for the category totals; gives the claimed paths.
fn resolve_claims(shared: &Shared) -> Vec<ClaimedPath> {
    let order: Vec<ModuleId> = shared.registry.registered().iter().map(|module| module.descriptor().id()).collect();
    let mut results = lock_anyway(&shared.results);
    let claims = claims::resolve(results.findings(), &order);
    let paths = claims.paths();
    results.keep_claims(claims);
    paths
}

/// Commits the walk's state with the scan's claims and keeps it with the owners of its places, found with the scan's
/// `system`; `false` when the commit failed (logged).
fn commit(shared: &Shared, claims: &[ClaimedPath], system: &SystemFacts) -> bool {
    match shared.ports.walk.commit(claims) {
        Ok(state) => {
            let places = AppPlaces::of(&*state, system);
            let roots: Vec<PathBuf> = state.roots().into_iter().filter_map(|root| state.path(root)).collect();
            let volumes = shared.read_volumes(&roots);
            lock_anyway(&shared.results).keep_state(state, places, volumes);
            true
        }
        Err(error) => {
            tracing::warn!(%error, "the scan's state was not kept");
            false
        }
    }
}

/// Calls a module; its error or its panic becomes the outcome it ends with. The panic does not reach the scan: what
/// the module left behind is only the findings and left-outs it already reported, which are kept like those of a
/// failing module.
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
/// if it is dropped before (a panic outside the modules), its `Drop` ends the scan `Stopped`.
struct Output<'a> {
    shared: &'a Shared,
    scan: ScanId,
    next_id: Option<u32>,
    batcher: Batcher<Finding>,
    exhausted: bool,
    ended: bool,
    /// How the walk read the disk; `Full` until it says otherwise.
    kind: ScanKind,
    /// What the scan saw; complete until the walk is decided.
    coverage: Coverage,
    /// What the walk read about each matched entry of this scan, by path: a finding's removability comes from the
    /// entry its removal names, also for a finding made in `finish`.
    facts: HashMap<PathBuf, EntryFacts>,
    /// What each planned module left out so far, by its index in the plan.
    left_outs: Vec<LeftOuts<'a>>,
    /// What each planned module described so far, by its index in the plan, in the order it gave it.
    insights: Vec<(ModuleId, Vec<Insight>)>,
    /// The scan's ignore lists.
    ignored: &'a Ignored,
    /// What the High switch leaves out: never published.
    withheld: Withheld,
}

impl<'a> Output<'a> {
    /// The output of scan `scan` of `plan`, numbering its findings from `first_id`.
    fn new(shared: &'a Shared, scan: ScanId, first_id: Option<u32>, plan: &'a ScanPlan) -> Self {
        Self {
            shared,
            scan,
            next_id: first_id,
            batcher: Batcher::default(),
            exhausted: false,
            ended: false,
            kind: ScanKind::Full,
            coverage: Coverage::default(),
            facts: HashMap::new(),
            left_outs: plan.modules.iter().map(|planned| LeftOuts::new(planned.module.descriptor())).collect(),
            insights: plan.modules.iter().map(|planned| (planned.module.descriptor().id(), Vec::new())).collect(),
            ignored: &plan.ignored,
            withheld: lock_anyway(&shared.results).withheld().clone(),
        }
    }

    /// Records the last scan, publishes `ScanCoverage` then `ScanFinished` and lets the engine go `Idle`, all under the
    /// state's lock.
    fn end(&mut self, outcome: ScanOutcome) {
        self.ended = true;
        let finished = Timestamp::from_system_time(self.shared.ports.clock.now());
        let mut machine = lock_anyway(&self.shared.machine);
        let coverage = self.coverage.clone();
        let last = LastScan {
            scan: self.scan,
            finished,
            outcome: outcome.clone(),
            kind: self.kind,
            coverage,
            changed_since: None,
            settings_changed: false,
        };
        lock_anyway(&self.shared.results).finish(last, self.next_id);
        self.shared.bus.publish(&Event::ScanCoverage { scan: self.scan, coverage: self.coverage.clone() });
        self.shared.bus.publish(&Event::ScanFinished { scan: self.scan, outcome, kind: self.kind });
        machine.finish(self.scan);
    }

    /// Where the planned module at `index` reports.
    fn sink(&mut self, index: usize) -> ModuleSink<'_, 'a> {
        ModuleSink { output: self, index }
    }

    /// Sends what the module left in the open batch, then its `ModuleFinished` with what it left out and what it
    /// described, whatever its outcome: what it gave before it failed or was cancelled is still true. Neither holds
    /// what the High switch withholds: a left-out of a withheld category, an insight's part at a withheld level.
    fn module_finished(&mut self, index: usize, module: ModuleId, outcome: ModuleOutcome) -> ModuleOutcome {
        if let Some(batch) = self.batcher.take() {
            self.send(batch);
        }
        if let ModuleOutcome::Failed { reason } = &outcome {
            tracing::warn!(module = module.as_str(), reason = reason.key().as_str(), "a module failed");
        }
        let mut left_out = self.left_outs.get_mut(index).map(LeftOuts::take).unwrap_or_default();
        left_out.retain(|count| !self.withheld.hides_category(count.category));
        let mut insights = self.insights.get_mut(index).map(|(_, given)| std::mem::take(given)).unwrap_or_default();
        for insight in &mut insights {
            insight.retain_parts(|part| !self.withheld.hides_level(part.danger));
        }
        let finished = Event::ModuleFinished { scan: self.scan, module, outcome: outcome.clone(), left_out, insights };
        self.shared.bus.publish(&finished);
        outcome
    }

    /// Numbers `finding`, adds it to the open batch and sends the batch when it is due.
    fn push(&mut self, finding: NewFinding) {
        let Some(id) = self.next_id else {
            self.exhausted = true;
            return;
        };
        self.next_id = id.checked_add(1);
        let now = self.shared.ports.clock.now();
        let removability = Removability::of_removal(finding.removal(), |path| self.facts.get(path).copied());
        if let Some(batch) = self.batcher.push(Finding::new(FindingId::new(id), finding, removability), now) {
            self.send(batch);
        }
    }

    /// Keeps `batch` in the results and publishes the findings of it the High switch does not withhold, if any.
    fn send(&self, batch: Vec<Finding>) {
        lock_anyway(&self.shared.results).append(&batch);
        let offered: Vec<Finding> = batch.into_iter().filter(|finding| !self.withheld.hides(finding)).collect();
        if !offered.is_empty() {
            self.shared.bus.publish(&Event::FindingsAdded { scan: self.scan, findings: offered });
        }
    }
}

impl Drop for Output<'_> {
    fn drop(&mut self) {
        if !self.ended {
            tracing::error!(scan = self.scan.get(), "the scan stopped on a panic");
            self.end(ScanOutcome::Stopped);
        }
    }
}

/// The sink of one module's call: its findings go to the scan's output, its left-outs to its own sums, its insights
/// to its own list.
struct ModuleSink<'o, 'a> {
    output: &'o mut Output<'a>,
    index: usize,
}

impl FindingSink for ModuleSink<'_, '_> {
    fn push(&mut self, finding: NewFinding) {
        self.output.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        if self.output.ignored.hides_category(left_out.category) {
            return;
        }
        if let Some(left_outs) = self.output.left_outs.get_mut(self.index) {
            left_outs.add(left_out);
        }
    }

    fn describe(&mut self, insight: Insight) {
        let Some((module, insights)) = self.output.insights.get_mut(self.index) else {
            return;
        };
        if insights.iter().any(|given| given.identity() == insight.identity()) {
            tracing::warn!(
                module = module.as_str(),
                identity = insight.identity(),
                "a module described the same identity twice"
            );
            return;
        }
        insights.push(insight);
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

    /// The removability of every published finding, in order.
    fn removabilities(events: &[Event]) -> Vec<fresher_core::Removability> {
        crate::testing::batches(events).iter().flatten().map(fresher_core::Finding::removability).collect()
    }

    mod negative {
        use fresher_core::{
            CommandError, DangerLevel, EngineStatus, Event, FindingOrder, FindingsQuery, ModuleError, ModuleId,
            ModuleOutcome, ScanOutcome, WalkError,
        };
        use serde_json::json;

        use fresher_core::{EntryFacts, Removability};

        use super::{id, ids, module_outcomes, removabilities, scan_outcome};
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
        fn a_scan_that_did_not_fall_back_has_no_fallback() {
            let engine = Fixture::new(ScriptedWalk::new()).engine();

            scan(&engine);

            assert_eq!(engine.fallback(fresher_core::ScanId::new(1)), None);
        }

        #[test]
        fn the_fallback_of_an_earlier_scan_is_not_answered() {
            let walk = ScriptedWalk::new().falling_back(fresher_core::FullScanReason::NoCache);
            let engine = Fixture::new(walk).engine();

            scan(&engine);

            assert_eq!(engine.fallback(fresher_core::ScanId::new(2)), None);
        }

        #[test]
        fn a_matched_item_of_another_user_needs_an_administrator() {
            let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
            let walk = ScriptedWalk::new().folder_with("Caches", "/home/Caches", bytes(1), theirs);
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            assert_eq!(removabilities(&events), [Removability::NeedsAdministrator]);
        }

        #[test]
        fn a_path_finding_without_a_match_is_not_removable() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module =
                TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).on_finish(move |_, out| {
                    out.push(finding(&caches, "/home/elsewhere", bytes(1)));
                    Ok(())
                });
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            assert_eq!(removabilities(&events), [Removability::NeedsAdministrator]);
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
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
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
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
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
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
        }

        #[test]
        fn a_panic_outside_the_modules_still_ends_the_scan_stopped() {
            let walk = ScriptedWalk::new().panicking();
            let engine = Fixture::new(walk).module(TestModule::builder("alpha")).engine();

            let events = scan(&engine);

            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Stopped));
            assert!(engine.start_scan(crate::ScanMode::Auto).is_ok());
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
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
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
        fn a_cancelled_scan_still_publishes_scan_finished_stopped() {
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let (walk, _gate) = walk.hold();
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let engine = Fixture::new(walk).module(module).engine();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.cancel_scan().unwrap();

            let events = wait_for_finish(&events);
            assert_eq!(module_outcomes(&events), vec![(alpha(), ModuleOutcome::Cancelled)]);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
            assert_eq!(engine.finding(id(1)).unwrap().module(), alpha());
        }

        #[test]
        fn a_walk_that_could_not_read_everything_ends_partial_and_one_that_failed_ends_stopped() {
            let unreadable = Fixture::new(ScriptedWalk::new().unreadable(3)).engine();
            let root = WalkError::RootUnreadable { index: 0, kind: std::io::ErrorKind::PermissionDenied };
            let stopped = Fixture::new(ScriptedWalk::new().failing(root)).module(TestModule::builder("alpha")).engine();

            let unreadable = scan(&unreadable);
            let stopped = scan(&stopped);

            assert_eq!(scan_outcome(&unreadable), Some(ScanOutcome::Partial));
            assert_eq!(scan_outcome(&stopped), Some(ScanOutcome::Stopped));
            assert_eq!(module_outcomes(&stopped), vec![(alpha(), ModuleOutcome::Completed)]);
        }

        #[test]
        fn a_scan_that_stopped_and_missed_folders_ends_stopped_with_its_coverage() {
            let failing = TestModule::builder("alpha")
                .on_finish(|_, _| Err(ModuleError::Failed { reason: text("alpha.error.broken") }));
            let engine = Fixture::new(ScriptedWalk::new().unreadable(3)).module(failing).engine();

            let events = scan(&engine);

            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
            let coverage = events.iter().find_map(|event| match event {
                Event::ScanCoverage { coverage, .. } => Some(coverage),
                _ => None,
            });
            assert!(coverage.is_some_and(|coverage| !coverage.is_complete()));
        }

        #[test]
        fn a_cancelled_scan_is_not_committed() {
            let (walk, _gate) = ScriptedWalk::held();
            let commits = walk.commits();
            let engine = Fixture::new(walk).engine();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.cancel_scan().unwrap();

            assert_eq!(scan_outcome(&wait_for_finish(&events)), Some(ScanOutcome::Stopped));
            assert!(commits.lock().unwrap().is_empty());
            assert!(engine.disk_state().unwrap().is_none());
        }

        #[test]
        fn a_cancelled_scan_keeps_the_previous_places() {
            use std::path::{Path, PathBuf};
            use std::sync::Arc;

            use fresher_core::{DiskState, NodeKind, StateId, Timestamp};
            use fresher_testing::FixedDiskState;

            let nodes = ["/home", "/home/.Trash"].map(|path| (PathBuf::from(path), NodeKind::Folder, bytes(1)));
            let state = FixedDiskState::new(StateId::new(7), Timestamp::from_secs(0), nodes);
            let trash = state.find(Path::new("/home/.Trash"));
            // A second commit would replace the places with those of a state without a Trash.
            let next = FixedDiskState::new(
                StateId::new(8),
                Timestamp::from_secs(0),
                [(PathBuf::from("/home"), NodeKind::Folder, bytes(1))],
            );
            let (walk, gate) = ScriptedWalk::new().committing(vec![Arc::new(state), Arc::new(next)]).hold();
            gate.release();
            let engine = Fixture::new(walk).engine();
            scan(&engine);
            let places = || crate::error::lock_anyway(&engine.shared.results).committed().unwrap().places.trash();
            assert_eq!(places(), trash);
            gate.close();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.cancel_scan().unwrap();

            assert_eq!(scan_outcome(&wait_for_finish(&events)), Some(ScanOutcome::Stopped));
            assert_eq!(places(), trash);
            assert_eq!(engine.disk_state().unwrap().unwrap().id(), StateId::new(7));
        }

        #[test]
        fn a_walk_that_did_not_end_asks_nothing() {
            let walk = ScriptedWalk::new().marker("Cargo.toml", "/home/tool");
            let (walk, _gate) = walk.hold();
            let queries = walk.subtree_queries();
            let engine = Fixture::new(walk).module(TestModule::builder("alpha")).engine();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            engine.cancel_scan().unwrap();

            assert_eq!(scan_outcome(&wait_for_finish(&events)), Some(ScanOutcome::Stopped));
            assert!(queries.lock().unwrap().is_empty());
        }

        #[test]
        fn a_walk_that_stopped_is_not_committed() {
            let root = WalkError::RootUnreadable { index: 0, kind: std::io::ErrorKind::NotFound };
            let walk = ScriptedWalk::new().failing(root);
            let commits = walk.commits();
            let engine = Fixture::new(walk).engine();

            scan(&engine);

            assert!(commits.lock().unwrap().is_empty());
        }

        #[test]
        fn a_failed_commit_ends_the_scan_stopped() {
            let walk = ScriptedWalk::new().failing_commit(WalkError::Commit { kind: std::io::ErrorKind::Other });
            let commits = walk.commits();
            let engine = Fixture::new(walk).engine();

            let events = scan(&engine);

            assert_eq!(commits.lock().unwrap().len(), 1);
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
            assert!(engine.disk_state().unwrap().is_none());
        }

        #[test]
        fn findings_past_the_last_id_are_dropped_and_the_scan_is_stopped() {
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
            assert_eq!(scan_outcome(&events), Some(ScanOutcome::Stopped));
        }

        #[test]
        fn refuses_a_query_for_an_unknown_module() {
            let engine = Fixture::new(ScriptedWalk::new()).module(TestModule::builder("alpha")).engine();
            let query = FindingsQuery::new(Some("gamma".into()), None, FindingOrder::Found, 0, 10);

            assert_eq!(engine.findings(&query), Err(CommandError::UnknownModule { id: "gamma".into() }));
        }

        #[test]
        fn a_category_nobody_declared_matches_nothing() {
            let engine = Fixture::new(ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1)))
                .module(TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches"))
                .engine();
            scan(&engine);

            let page = engine
                .findings(&FindingsQuery::new(None, Some("alpha.unknown".into()), FindingOrder::Found, 0, 10))
                .unwrap();

            assert_eq!((page.findings.len(), page.total), (0, 0));
        }

        #[test]
        fn a_disabled_modules_match_reaches_no_module() {
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let off = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let off_seen = off.seen();
            let on = TestModule::builder("beta").category("beta.caches", DangerLevel::Safe).folder("Caches");
            let on_seen = on.seen();
            let settings = MemorySettings::with(&[("alpha.enabled", json!(false))]);
            let engine = Fixture::new(walk).module(off).module(on).settings(settings).engine();

            scan(&engine);

            assert_eq!(off_seen.lock().unwrap().len(), 0);
            assert_eq!(on_seen.lock().unwrap().len(), 1, "the walk still matched both modules' interests");
            assert_eq!(on_seen.lock().unwrap()[0].interest, 0, "the module sees its own interest index");
        }

        #[test]
        fn a_tool_finding_is_counted_whole_and_not_claimed() {
            let tools = category("alpha.images", DangerLevel::Low);
            let module =
                TestModule::builder("alpha").category("alpha.images", DangerLevel::Low).on_finish(move |_, out| {
                    let mut image = finding(&tools, "/unused", bytes(9));
                    image = fresher_core::NewFinding::new(
                        &tools,
                        "image",
                        fresher_core::Subject::External {
                            tool: fresher_core::ToolId::new("docker").unwrap(),
                            id: "sha256:1".into(),
                        },
                        image.sizes(),
                        image.title().clone(),
                        image.reason().clone(),
                        image.removal().clone(),
                    );
                    out.push(image);
                    Ok(())
                });
            let walk = ScriptedWalk::new();
            let commits = walk.commits();
            let engine = Fixture::new(walk).module(module).engine();

            scan(&engine);

            assert_eq!(commits.lock().unwrap().as_slice(), [vec![]]);
            assert_eq!(engine.overview().unwrap().totals[0].sizes, bytes(9));
        }
    }

    mod positive {
        use std::time::Duration;

        use fresher_core::{
            ByteSize, Completeness, Coverage, DangerLevel, EngineStatus, Event, FindingOrder, FindingsQuery, LastScan,
            ScanId, ScanKind, ScanOutcome, Sizes, Timestamp,
        };

        use fresher_core::{EntryFacts, Removability, Removal, ToolCommand, ToolId};

        use super::{id, ids, removabilities, scan_outcome};
        use crate::testing::{
            Fixture, MemorySettings, START, ScriptedWalk, TestModule, batches, bytes, category, finding, scan, together,
        };

        #[test]
        fn a_module_sees_in_finish_what_the_walk_saw_of_its_own_places_only() {
            use std::sync::{Arc, Mutex};

            use fresher_core::PlaceSeen;

            let recorder = |seen: &Arc<Mutex<Vec<PlaceSeen>>>| {
                let record = Arc::clone(seen);
                move |cx: &fresher_core::ModuleContext<'_>, _: &mut dyn fresher_core::FindingSink| {
                    record.lock().unwrap().extend((0..3).map(|interest| cx.place_seen(interest)));
                    Ok(())
                }
            };
            let (alpha_seen, beta_seen) = (Arc::default(), Arc::default());
            let alpha = TestModule::builder("alpha")
                .category("alpha.caches", DangerLevel::Safe)
                .folder("Caches")
                .folder("Logs")
                .on_finish(recorder(&alpha_seen));
            let beta = TestModule::builder("beta")
                .category("beta.caches", DangerLevel::Safe)
                .folder("Tmp")
                .on_finish(recorder(&beta_seen));
            let walk = ScriptedWalk::new()
                .seeing(0, PlaceSeen::Read)
                .seeing(1, PlaceSeen::Unread)
                .seeing(2, PlaceSeen::NotScanned);
            let engine = Fixture::new(walk).module(alpha).module(beta).engine();

            scan(&engine);

            let unknown = PlaceSeen::Unknown;
            assert_eq!(*alpha_seen.lock().unwrap(), [PlaceSeen::Read, PlaceSeen::Unread, unknown]);
            assert_eq!(*beta_seen.lock().unwrap(), [PlaceSeen::NotScanned, unknown, unknown]);
        }

        #[test]
        fn the_projects_carry_what_the_walk_saw_of_them_before_any_finish() {
            use std::path::PathBuf;
            use std::sync::{Arc, Mutex};

            use fresher_core::{Artifact, LastChange, Project, ProjectKind, Subtree};

            let tool = Subtree { sizes: bytes(1), newest: Some(Timestamp::from_secs(40)), complete: true };
            let target = Subtree { sizes: bytes(9), newest: Some(Timestamp::from_secs(30)), complete: true };
            let walk = ScriptedWalk::new()
                .marker("Cargo.toml", "/home/tool")
                .folder("target", "/home/tool/target", bytes(9))
                .answering_subtrees(vec![("/home/tool".into(), tool), ("/home/tool/target".into(), target)]);
            let (queries, commits) = (walk.subtree_queries(), walk.commits());
            let seen = Arc::new(Mutex::new(None));
            let record = Arc::clone(&seen);
            let (asked, committed) = (Arc::clone(&queries), Arc::clone(&commits));
            let module = TestModule::builder("alpha").on_finish(move |cx, _| {
                let before_commit = (asked.lock().unwrap().len(), committed.lock().unwrap().len());
                *record.lock().unwrap() = Some((cx.system().projects().map(<[Project]>::to_vec), before_commit));
                Ok(())
            });
            let engine = Fixture::new(walk).module(module).engine();

            scan(&engine);

            let (projects, before_commit) = seen.lock().unwrap().take().unwrap();
            let target = Artifact {
                path: PathBuf::from("/home/tool/target"),
                sizes: bytes(9),
                last_change: LastChange::Known(Timestamp::from_secs(30)),
            };
            let expected = Project {
                root: PathBuf::from("/home/tool"),
                kinds: vec![ProjectKind::Rust],
                artifacts: vec![target],
                caches: Vec::new(),
                last_activity: LastChange::Known(Timestamp::from_secs(40)),
            };
            assert_eq!(projects, Some(vec![expected]));
            assert_eq!(before_commit, (1, 0), "asked once, after the walk and before the commit");
            assert_eq!(queries.lock().unwrap().len(), 1);
            assert_eq!(commits.lock().unwrap().len(), 1);
        }

        #[test]
        fn a_users_own_matched_item_is_removable() {
            let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
            let module = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            assert_eq!(removabilities(&events), [Removability::Removable]);
            assert_eq!(engine.finding(id(1)).map(|found| found.removability()), Ok(Removability::Removable));
        }

        #[test]
        fn a_finish_finding_takes_the_facts_of_its_match() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let module = TestModule::builder("alpha")
                .category("alpha.caches", DangerLevel::Safe)
                .folder("Versions")
                .on_finish(move |_, out| {
                    out.push(finding(&caches, "/home/Versions", bytes(1)));
                    Ok(())
                });
            let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
            let walk = ScriptedWalk::new().folder_with("Versions", "/home/Versions", bytes(1), locked);
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            // The first finding is made on the match, the second in `finish`.
            assert_eq!(removabilities(&events), [Removability::Locked, Removability::Locked]);
        }

        #[test]
        fn a_together_findings_removability_reads_every_part() {
            let models = category("alpha.models", DangerLevel::Medium);
            let module = TestModule::builder("alpha")
                .category("alpha.models", DangerLevel::Medium)
                .folder("model")
                .folder("layer")
                .on_finish(move |_, out| {
                    out.push(together(&models, &["/home/model", "/home/blobs/layer"], bytes(2)));
                    out.push(together(&models, &["/home/model", "/home/blobs/unmatched"], bytes(2)));
                    Ok(())
                });
            let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
            let walk = ScriptedWalk::new().folder("model", "/home/model", bytes(1)).folder_with(
                "layer",
                "/home/blobs/layer",
                bytes(1),
                locked,
            );
            let engine = Fixture::new(walk).module(module).engine();

            let events = scan(&engine);

            // The two matches' own findings, then the together findings: a locked part, a part no match reported.
            assert_eq!(
                removabilities(&events),
                [Removability::Removable, Removability::Locked, Removability::Locked, Removability::NeedsAdministrator]
            );
        }

        #[test]
        fn a_tool_removal_is_removable() {
            let images = category("alpha.images", DangerLevel::Low);
            let module =
                TestModule::builder("alpha").category("alpha.images", DangerLevel::Low).on_finish(move |_, out| {
                    let mut item = finding(&images, "/home/unused", bytes(1));
                    let tool = ToolCommand::new(ToolId::new("docker").unwrap(), vec!["rmi".into()]).unwrap();
                    item = fresher_core::NewFinding::new(
                        &images,
                        "image",
                        item.subject().clone(),
                        item.sizes(),
                        item.title().clone(),
                        item.reason().clone(),
                        Removal::ToolCommand(tool),
                    );
                    out.push(item);
                    Ok(())
                });
            let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

            let events = scan(&engine);

            assert_eq!(removabilities(&events), [Removability::Removable]);
        }

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
        fn the_overview_counts_nested_claims_once_and_commits_them() {
            let walk = ScriptedWalk::new().folder("project", "/home/project", bytes(100)).folder(
                "cache",
                "/home/project/cache",
                bytes(30),
            );
            let commits = walk.commits();
            let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe);
            let engine = Fixture::new(walk).module(alpha.folder("project").folder("cache")).engine();

            scan(&engine);

            let overview = engine.overview().unwrap();
            assert_eq!((overview.totals[0].count, overview.totals[0].sizes), (2, bytes(100)));
            assert_eq!(overview.found, bytes(100), "the overview's sum counts a nested claim once too");
            let claimed: Vec<_> = commits.lock().unwrap()[0].iter().map(|claim| claim.path.clone()).collect();
            assert_eq!(claimed, ["/home/project", "/home/project/cache"].map(std::path::PathBuf::from));
            let page = engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 10)).unwrap();
            assert_eq!(page.findings[0].finding().sizes(), bytes(100), "a finding keeps its own size for display");
        }

        #[test]
        fn a_completed_scan_is_committed_and_its_state_kept() {
            let walk = ScriptedWalk::new();
            let commits = walk.commits();
            let engine = Fixture::new(walk).engine();
            assert!(engine.disk_state().unwrap().is_none());

            scan(&engine);
            let first = engine.disk_state().unwrap().unwrap().id();
            scan(&engine);
            let second = engine.disk_state().unwrap().unwrap().id();

            assert_eq!(*commits.lock().unwrap(), [vec![], vec![]]);
            assert_eq!((first.get(), second.get()), (1, 2));
        }

        #[test]
        fn the_fallback_of_a_scan_says_which_check_failed() {
            use std::path::PathBuf;

            use fresher_core::{CheckDifference, CheckFailure, FullScanReason};

            let failure = CheckFailure {
                folder: PathBuf::from("Library/Caches"),
                difference: CheckDifference::EntryAdded(PathBuf::from("new.bin")),
                folders: 2,
            };
            let reason = FullScanReason::CheckFailed(failure);
            let engine = Fixture::new(ScriptedWalk::new().falling_back(reason.clone())).engine();

            scan(&engine);

            assert_eq!(engine.fallback(ScanId::new(1)), Some(reason));
        }

        #[test]
        fn scan_finished_carries_the_scan_kind() {
            let kind = ScanKind::Rescan { changed_folders: 7 };
            let engine = Fixture::new(ScriptedWalk::new().kind(kind)).engine();

            let events = scan(&engine);

            assert_eq!(
                events.last(),
                Some(&Event::ScanFinished { scan: ScanId::new(1), outcome: ScanOutcome::Completed, kind })
            );
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.kind), Some(kind));
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
            let (outcome, kind, coverage) = (ScanOutcome::Completed, ScanKind::Full, Coverage::default());
            let last = LastScan {
                scan: ScanId::new(1),
                finished,
                outcome,
                kind,
                coverage,
                changed_since: None,
                settings_changed: false,
            };
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

            let alpha =
                engine.findings(&FindingsQuery::new(Some("alpha".into()), None, FindingOrder::Found, 1, 1)).unwrap();
            let deps = engine
                .findings(&FindingsQuery::new(None, Some("beta.deps".into()), FindingOrder::Found, 0, 10))
                .unwrap();
            let all = engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 10)).unwrap();

            assert_eq!((alpha.offset, alpha.total, alpha.findings[0].id()), (1, 3, id(2)));
            assert_eq!((deps.total, deps.findings[0].id()), (1, id(4)));
            assert_eq!(all.findings.len(), 4);
        }
    }
}
