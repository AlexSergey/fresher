//! The clean operation: the confirmed plan executed on its own thread, the journal written ahead of every item, the
//! outcomes batched into events, the space measured on the volumes, the removed findings out of the results.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use fresher_core::{
    BeginRecord, BlockedReason, ByteSize, CancelToken, CleanupId, CleanupReport, CommandError, ConfirmError,
    Confirmation, EndRecord, Ended, Event, FindingId, ItemDone, ItemOutcome, ItemReport, JournalError, Method,
    Timestamp, VolumeSpace, Volumes,
};

use crate::apps::{management, uninstall};
use crate::batch::Batcher;
use crate::cleanup::blocked;
use crate::cleanup::context::Context;
use crate::cleanup::pending::Purpose;
use crate::engine::{CleanupPorts, Shared};
use crate::error::{EngineError, lock, lock_anyway};
use crate::own::check_picks;
use crate::state::{OperationThread, Running};

/// The journal is made durable after this many outcomes…
const SYNC_EVERY: u32 = 50;
/// …and when this much time has passed on the clock since it last was.
const SYNC_INTERVAL: Duration = Duration::from_secs(1);

/// Confirms the pending plan with `echo` and starts its cleanup; answers the cleanup's id from the journal.
///
/// Refused while an operation runs (`Busy`), without a pending plan or for one that expired or that the echo does not
/// name (`PlanGone`), for an echo that differs from the plan (`PlanMismatch`), for a plan whose highest level is High
/// confirmed without the user's tick that its risk is understood (`HighRiskNotUnderstood`, checked before anything
/// an uninstall adds, so an uninstall of everything is refused for it first), for a plan whose every item was left
/// out (`NothingToClean`), for a plan holding a row the user picked that is no longer offered (`PlanGone`), and when
/// cleaning is blocked or the journal cannot begin (`CleaningBlocked`): nothing is acted on or journalled then. An
/// uninstall's plan is refused permanent (`PlanMismatch`: an uninstall always goes to the Trash), judged again
/// (`UninstallRefused` when its app is no longer offered, opened since, say) and cleaned with the uninstall's context;
/// every other plan with the ordinary one, which refuses the apps folders. The plan is consumed whatever the answer.
pub(crate) fn confirm(shared: &Arc<Shared>, echo: &Confirmation) -> Result<CleanupId, CommandError> {
    let mut machine = lock(&shared.machine)?;
    machine.idle()?;
    let (plan, purpose) = lock(&shared.pending)?.take().ok_or(CommandError::PlanGone)?;
    let ports = blocked::check(&*lock(&shared.settings)?, shared.cleanup.as_ref())?.clone();
    let now = Timestamp::from_system_time(shared.ports.clock.now());
    let confirmed = plan.confirm(echo, now).map_err(|error| match error {
        ConfirmError::Mismatch { field } => {
            tracing::warn!(%field, "the confirmation does not match the plan");
            CommandError::PlanMismatch
        }
        ConfirmError::WrongPlan | ConfirmError::Expired => CommandError::PlanGone,
        ConfirmError::HighRiskNotUnderstood => {
            tracing::warn!("a plan holding a High item is confirmed without its risk understood");
            CommandError::HighRiskNotUnderstood
        }
    })?;
    if matches!(purpose, Purpose::Uninstall { .. }) && confirmed.method() == Method::Permanent {
        tracing::warn!("an uninstall is confirmed to be deleted permanently; it only goes to the Trash");
        return Err(CommandError::PlanMismatch);
    }
    if confirmed.items().is_empty() {
        return Err(CommandError::NothingToClean);
    }
    check_picks(shared, confirmed.items())?;
    if let Purpose::Uninstall { id, bundle, .. } = &purpose {
        uninstall::judge_again(shared, *id, bundle)?;
    }
    let cx = Context::now(shared, purpose.scope())?;
    let paths: Vec<Vec<PathBuf>> = confirmed.items().iter().map(|item| item.action().parts().to_vec()).collect();
    let uninstalled = match &purpose {
        Purpose::Uninstall { bundle, .. } => management::guards(shared, bundle).then(Vec::new),
        Purpose::Selection => None,
    };
    let before = Space::measure(&*ports.volumes, confirmed.plan().roots());
    let cleanup = ports.journal.begin(&BeginRecord::new(now, &confirmed, before.volumes.clone())).map_err(|error| {
        tracing::warn!(%error, "the cleanup could not be recorded; nothing was acted on");
        let reason = if error == JournalError::NoSpace { BlockedReason::DiskFull } else { BlockedReason::Journal };
        CommandError::CleaningBlocked { reason }
    })?;

    let cancel = CancelToken::new();
    let (token, journal) = (cancel.clone(), Arc::clone(&ports.journal));
    let (method, total) = (confirmed.method(), u32::try_from(confirmed.items().len()).unwrap_or(u32::MAX));
    let shared_by_thread = Arc::clone(shared);
    let (done, ended) = mpsc::channel::<()>();
    let spawned = thread::Builder::new().name(format!("fresher-cleanup-{cleanup}")).spawn(move || {
        // Dropped when the thread ends, on a return or a panic: `shutdown` waits for that, not for ever.
        let _done = done;
        let mut run = Run::new(&shared_by_thread, &ports, cleanup, method, before, paths, uninstalled);
        shared_by_thread.bus.publish(&Event::CleanupStarted { cleanup, method, total });
        let ended = ports.cleaner.execute(confirmed, &cx.borrow(), &token, &mut |item| run.item(&item, &token));
        run.end(ended);
    });
    let handle = spawned.map_err(|error| {
        let at = Timestamp::from_system_time(shared.ports.clock.now());
        let end = EndRecord { cleanup, at, ended: Ended::Stopped, space: Vec::new() };
        if let Err(error) = journal.end(&end) {
            tracing::warn!(%error, "the end of a cleanup that never started was not recorded");
        }
        EngineError::Spawn { kind: error.kind() }
    })?;
    machine.start_cleaning(cleanup, Running { cancel, thread: Some(OperationThread { handle, done: ended }) });
    Ok(cleanup)
}

/// A cleanup on its thread: what the events, the journal and the report need as the items end. If it is dropped
/// before it ended (a panic), its `Drop` ends the cleanup `Stopped`, so the engine never stays `Cleaning`.
struct Run<'a> {
    shared: &'a Shared,
    ports: &'a CleanupPorts,
    cleanup: CleanupId,
    report: CleanupReport,
    batcher: Batcher<ItemReport>,
    done: u32,
    removed: Vec<FindingId>,
    /// The removed findings whose whole path went: the findings below them leave the results too.
    cleared: Vec<FindingId>,
    /// The paths of every item, by its place in the plan, copied before the plan went to the cleaner.
    paths: Vec<Vec<PathBuf>>,
    /// The paths that went: every path of an item whose whole path went, and the parts of an item of several paths
    /// that went before it stopped. What was at or below them leaves the results too, picks or not.
    went: Vec<PathBuf>,
    /// How the items of an uninstall ended and how many of their parts went, in order, to teach App Management at the
    /// end; `None` for any other plan and for an uninstall App Management is not involved in.
    uninstalled: Option<Vec<(ItemOutcome, Option<u32>)>>,
    unsynced: u32,
    synced_at: SystemTime,
    /// A record could not be written: the cleanup stops after the item in flight.
    stopped: bool,
    before: Space,
    ended: bool,
}

impl<'a> Run<'a> {
    fn new(
        shared: &'a Shared,
        ports: &'a CleanupPorts,
        cleanup: CleanupId,
        method: Method,
        before: Space,
        paths: Vec<Vec<PathBuf>>,
        uninstalled: Option<Vec<(ItemOutcome, Option<u32>)>>,
    ) -> Self {
        Self {
            shared,
            ports,
            cleanup,
            report: CleanupReport::new(method),
            batcher: Batcher::default(),
            done: 0,
            removed: Vec::new(),
            cleared: Vec::new(),
            paths,
            went: Vec::new(),
            uninstalled,
            unsynced: 0,
            synced_at: shared.ports.clock.now(),
            stopped: false,
            before,
            ended: false,
        }
    }

    /// One item ended: its record appended (durable every 50 items and every second), counted, and batched into
    /// `CleanupItems`. A record that cannot be written cancels the rest: no removal goes unrecorded.
    fn item(&mut self, item: &ItemDone, cancel: &CancelToken) {
        let now = self.shared.ports.clock.now();
        if !self.stopped {
            let mut written = self.ports.journal.item(&item.record(self.cleanup));
            self.unsynced = self.unsynced.saturating_add(1);
            let waited = now.duration_since(self.synced_at).map_or(true, |waited| waited >= SYNC_INTERVAL);
            if written.is_ok() && (self.unsynced >= SYNC_EVERY || waited) {
                written = self.ports.journal.sync();
                (self.unsynced, self.synced_at) = (0, now);
            }
            if let Err(error) = written {
                tracing::error!(%error, cleanup = self.cleanup.get(), "the journal failed; the cleanup stops");
                self.stopped = true;
                cancel.cancel();
            }
        }
        self.report.count(&item.outcome, item.sizes);
        if removes(&item.outcome) {
            self.removed.push(item.finding);
        }
        if clears(&item.outcome) {
            self.cleared.push(item.finding);
        }
        let paths = usize::try_from(item.index).ok().and_then(|index| self.paths.get(index));
        self.went.extend_from_slice(went(&item.outcome, paths.map_or(&[], Vec::as_slice), item.went));
        if let Some(uninstalled) = &mut self.uninstalled {
            uninstalled.push((item.outcome.clone(), item.went));
        }
        self.done = self.done.saturating_add(1);
        if let Some(batch) = self.batcher.push(item.report(), now) {
            self.publish(batch);
        }
    }

    /// The cleanup ended: the last outcomes, the space measured again, the journal's end, the removed findings (and
    /// those at or below a path that went, an uninstalled app's bundle and parts among them) out of the results and
    /// the last scan marked changed, what an uninstall taught App Management (`AccessChanged` once when it changed
    /// the state), `FindingsRemoved`, `CleanupFinished`, then `Idle` — the last three under the state's lock.
    fn end(&mut self, ended: Ended) {
        self.ended = true;
        if let Some(batch) = self.batcher.take() {
            self.publish(batch);
        }
        let after = self.before.again(&*self.ports.volumes);
        let ended = if self.stopped { Ended::Stopped } else { ended };
        let at = Timestamp::from_system_time(self.shared.ports.clock.now());
        let end = EndRecord { cleanup: self.cleanup, at, ended, space: after.volumes.clone() };
        if let Err(error) = self.ports.journal.end(&end) {
            tracing::error!(%error, cleanup = self.cleanup.get(), "the end of the cleanup was not recorded");
        }
        self.report.ended = ended;
        self.report.measured = self.before.gained(&after);
        self.learn();

        let mut machine = lock_anyway(&self.shared.machine);
        let (scan, findings) = {
            let mut results = lock_anyway(&self.shared.results);
            let findings = results.remove(&self.removed, &self.cleared, &self.went);
            results.mark_changed(self.cleanup);
            (results.last().map(|last| last.scan), findings)
        };
        if let Some(scan) = scan
            && !findings.is_empty()
        {
            self.shared.bus.publish(&Event::FindingsRemoved { scan, findings });
        }
        self.shared.bus.publish(&Event::CleanupFinished { cleanup: self.cleanup, report: self.report });
        machine.finish_cleaning(self.cleanup);
    }

    /// Teaches App Management how the uninstall's items ended; `AccessChanged` once when its state changed.
    fn learn(&mut self) {
        let management = self.shared.access.management();
        let mut changed = false;
        for (outcome, went) in self.uninstalled.take().into_iter().flatten() {
            changed |= management.learn(&outcome, went);
        }
        if changed {
            self.shared.bus.publish(&Event::AccessChanged { status: self.shared.access.status() });
        }
    }

    fn publish(&self, items: Vec<ItemReport>) {
        self.shared.bus.publish(&Event::CleanupItems { cleanup: self.cleanup, done: self.done, items });
    }
}

impl Drop for Run<'_> {
    fn drop(&mut self) {
        if !self.ended {
            tracing::error!(cleanup = self.cleanup.get(), "the cleanup stopped on a panic");
            self.end(Ended::Stopped);
        }
    }
}

/// Whether the item left: its finding leaves the results.
const fn removes(outcome: &ItemOutcome) -> bool {
    matches!(outcome, ItemOutcome::Moved | ItemOutcome::Emptied { .. } | ItemOutcome::Deleted | ItemOutcome::ToolDone)
}

/// Whether the item's whole path went, and with it every finding below it: moved, deleted, or emptied with no entry
/// left.
const fn clears(outcome: &ItemOutcome) -> bool {
    matches!(outcome, ItemOutcome::Moved | ItemOutcome::Deleted | ItemOutcome::Emptied { skipped: 0, .. })
}

/// The paths of an item that went, of its `paths`: every one when it was moved or deleted, the first `went` of an item
/// of several paths that stopped part way, none otherwise. An emptied folder stays: it did not go.
fn went<'a>(outcome: &ItemOutcome, paths: &'a [PathBuf], went: Option<u32>) -> &'a [PathBuf] {
    if matches!(outcome, ItemOutcome::Moved | ItemOutcome::Deleted) {
        return paths;
    }
    let went = went.map_or(0, |went| usize::try_from(went).unwrap_or(usize::MAX));
    paths.get(..went.min(paths.len())).unwrap_or_default()
}

/// The available space of each volume a cleanup touches, once per volume; `complete` when every one was read.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Space {
    volumes: Vec<VolumeSpace>,
    complete: bool,
}

impl Space {
    /// The available space of the volume of each of `roots`, a volume once.
    fn measure(volumes: &dyn Volumes, roots: &[PathBuf]) -> Self {
        let mut space = Self { volumes: Vec::new(), complete: true };
        let mut seen = Vec::new();
        for root in roots {
            let read = volumes.identify(root).and_then(|facts| {
                if seen.contains(&facts.id) {
                    return Ok(None);
                }
                seen.push(facts.id);
                volumes.available(root).map(Some)
            });
            match read {
                Ok(Some(available)) => space.volumes.push(VolumeSpace { root: root.clone(), available }),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(%error, "the available space of a volume could not be read");
                    space.complete = false;
                }
            }
        }
        space
    }

    /// The same volumes read again.
    fn again(&self, volumes: &dyn Volumes) -> Self {
        let mut space = Self { volumes: Vec::new(), complete: self.complete };
        for volume in &self.volumes {
            match volumes.available(&volume.root) {
                Ok(available) => space.volumes.push(VolumeSpace { root: volume.root.clone(), available }),
                Err(error) => {
                    tracing::info!(%error, "the available space of a volume could not be read");
                    space.complete = false;
                }
            }
        }
        space
    }

    /// The space gained on every volume from `self` to `after`, each at least 0; `None` when a volume was not read.
    fn gained(&self, after: &Self) -> Option<ByteSize> {
        if !self.complete || !after.complete || self.volumes.len() != after.volumes.len() {
            return None;
        }
        let gains = self
            .volumes
            .iter()
            .zip(&after.volumes)
            .map(|(before, after)| after.available.bytes().saturating_sub(before.available.bytes()));
        Some(ByteSize::new(gains.fold(0, u64::saturating_add)))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{CleanupPlanView, Confirmation};
    use fresher_testing::FakeCleaner;

    use crate::Engine;
    use crate::cleanup::testing::{echo, scanned, select};
    use crate::testing::{Fixture, Gate, HeldCleaner};

    /// The plan of findings 1, 2 and 3 (acted on as 3, 1, 2), and its confirmation by `method`.
    fn prepared(engine: &Engine, method: &str) -> (CleanupPlanView, Confirmation) {
        let view = engine.prepare_cleanup(&select(&[1, 2, 3], &[])).unwrap();
        let echo = echo(&view, method);
        (view, echo)
    }

    /// An engine after a scan of `fixture` that cleans through a held cleaner, and the gate that lets it go on.
    fn held(fixture: Fixture) -> (Engine, Arc<HeldCleaner>, Gate) {
        let (cleaner, gate) = HeldCleaner::new(FakeCleaner::default());
        let cleaner = Arc::new(cleaner);
        (scanned(fixture.cleaner(Arc::clone(&cleaner))), cleaner, gate)
    }

    mod negative {
        use std::sync::Arc;
        use std::time::Duration;

        use fresher_core::{
            BlockedReason, CleanupId, CommandError, DangerLevel, Ended, EngineStatus, Event, FindingId, ItemOutcome,
            Operation, PLAN_LIFETIME, RecordedOutcome, SkipReason, VolumeFacts, VolumeId,
        };
        use fresher_testing::{FakeCleaner, FakeVolumes, MemoryCleanupJournal};
        use serde_json::json;

        use super::{held, prepared};
        use crate::cleanup::testing::{
            echo, echo_with, fixture, report, scanned, select, wait_for_cleanup, withdraw_agreement,
        };
        use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, panic_now, wait_until_idle};

        #[test]
        fn confirm_without_a_pending_plan_is_gone() {
            let engine = scanned(fixture());
            let (view, echo) = prepared(&engine, "trash");
            engine.discard_cleanup(view.plan).unwrap();

            assert_eq!(engine.confirm_cleanup(&echo), Err(CommandError::PlanGone));
        }

        #[test]
        fn a_mismatched_echo_is_refused_and_the_plan_is_gone() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));
            let changes: [(&str, serde_json::Value); 3] =
                [("count", json!(300)), ("allocated", json!(1)), ("highest", json!("high"))];

            for (field, value) in changes {
                let (view, echo) = prepared(&engine, "trash");
                let mismatched = echo_with(&view, "trash", |json| json[field] = value);

                assert_eq!(engine.confirm_cleanup(&mismatched), Err(CommandError::PlanMismatch), "{field}");
                assert_eq!(engine.confirm_cleanup(&echo), Err(CommandError::PlanGone), "{field}");
            }
            assert_eq!(cleaner.executed(), []);
        }

        #[test]
        fn a_confirmation_of_another_plan_is_gone() {
            let engine = scanned(fixture());
            let (view, _) = prepared(&engine, "trash");

            let other = echo_with(&view, "trash", |json| json["plan"] = json!(view.plan.get() + 1));

            assert_eq!(engine.confirm_cleanup(&other), Err(CommandError::PlanGone));
        }

        #[test]
        fn an_expired_plan_is_gone() {
            let fixture = fixture();
            let clock = Arc::clone(&fixture.clock);
            let engine = scanned(fixture);
            let (_, echo) = prepared(&engine, "trash");

            clock.advance(PLAN_LIFETIME + Duration::from_secs(1));

            assert_eq!(engine.confirm_cleanup(&echo), Err(CommandError::PlanGone));
        }

        #[test]
        fn a_confirmed_plan_cannot_be_confirmed_again() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            engine.confirm_cleanup(&echo).unwrap();
            wait_for_cleanup(&events);

            assert_eq!(engine.confirm_cleanup(&echo), Err(CommandError::PlanGone));
            assert_eq!(cleaner.executed().len(), 1);
        }

        #[test]
        fn confirm_without_the_agreement_is_blocked_and_acts_on_nothing() {
            let cleaner = Arc::new(FakeCleaner::default());
            let fixture = fixture().cleaner(Arc::clone(&cleaner));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let (_, echo) = prepared(&engine, "trash");
            withdraw_agreement(&engine);

            let error = engine.confirm_cleanup(&echo);

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Agreement }));
            assert_eq!(cleaner.executed(), []);
            assert_eq!(journal.recorded().len(), 0);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.confirm_cleanup(&echo), Err(CommandError::PlanGone));
        }

        #[test]
        fn confirm_of_a_high_plan_without_the_risk_is_refused_and_acts_on_nothing() {
            let cleaner = Arc::new(FakeCleaner::default());
            let walk = ScriptedWalk::new().folder("Data", "/home/Data", bytes(10));
            let gamma = TestModule::builder("gamma").category("gamma.data", DangerLevel::High).folder("Data");
            let fixture = Fixture::new(walk).module(gamma).cleaner(Arc::clone(&cleaner));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            let unticked = echo_with(&view, "trash", |json| json["highRiskUnderstood"] = json!(false));

            let error = engine.confirm_cleanup(&unticked);

            assert_eq!(view.summary.highest, DangerLevel::High);
            assert_eq!(error, Err(CommandError::HighRiskNotUnderstood));
            assert_eq!(cleaner.executed(), []);
            assert_eq!(journal.recorded().len(), 0);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
        }

        #[test]
        fn a_journal_that_cannot_begin_refuses_and_acts_on_nothing() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine =
                scanned(fixture().cleaner(Arc::clone(&cleaner)).journal(MemoryCleanupJournal::failing_after(0)));
            let (_, echo) = prepared(&engine, "trash");

            let error = engine.confirm_cleanup(&echo);

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }));
            assert_eq!(cleaner.executed(), []);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
        }

        #[test]
        fn a_failed_append_stops_after_the_current_item() {
            let fixture = fixture().journal(MemoryCleanupJournal::failing_after(2));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            engine.confirm_cleanup(&echo).unwrap();
            let seen = wait_for_cleanup(&events);

            let report = report(&seen);
            assert_eq!(report.ended, Ended::Stopped);
            assert_eq!((report.counts.moved, report.counts.not_reached), (2, 1));
            let recorded = &journal.recorded()[0];
            assert_eq!(recorded.items.len(), 1);
            assert_eq!(recorded.items[0].outcome, RecordedOutcome::Moved);
            assert_eq!(recorded.end, None);
        }

        #[test]
        fn scanning_while_cleaning_is_busy() {
            let (engine, _, gate) = held(fixture());
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");
            let cleanup = engine.confirm_cleanup(&echo).unwrap();
            let busy = Err(CommandError::Busy { running: Operation::Clean });

            assert_eq!(engine.status(), Ok(EngineStatus::Cleaning { cleanup }));
            assert_eq!(engine.start_scan(crate::ScanMode::Auto).map(|_| ()), busy);
            assert_eq!(engine.prepare_cleanup(&select(&[1], &[])).map(|_| ()), busy);
            assert_eq!(engine.confirm_cleanup(&echo).map(|_| ()), busy);
            assert_eq!(engine.cancel_scan(), Err(CommandError::NotRunning));

            gate.release();
            wait_for_cleanup(&events);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.cancel_cleanup(), Err(CommandError::NotRunning));
        }

        #[test]
        fn a_cancel_ends_cancelled_with_the_rest_not_reached() {
            let (engine, cleaner, _gate) = held(fixture());
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");
            engine.confirm_cleanup(&echo).unwrap();
            cleaner.wait_until_holding();

            engine.cancel_cleanup().unwrap();
            let seen = wait_for_cleanup(&events);

            let report = report(&seen);
            assert_eq!(report.ended, Ended::Cancelled);
            assert_eq!((report.counts.moved, report.counts.not_reached), (1, 2));
            let outcomes: Vec<_> = seen
                .iter()
                .filter_map(|event| match event {
                    Event::CleanupItems { items, .. } => Some(items.iter().map(|item| item.outcome.clone())),
                    _ => None,
                })
                .flatten()
                .collect();
            assert_eq!(outcomes, [ItemOutcome::Moved, ItemOutcome::NotReached, ItemOutcome::NotReached]);
            assert_eq!(cleaner.inner().executed().len(), 1);
        }

        #[test]
        fn a_volume_that_cannot_be_read_gives_no_measured_space() {
            let facts = VolumeFacts {
                id: VolumeId::new(1),
                device: 1,
                case_sensitive: false,
                used: fresher_core::ByteSize::ZERO,
            };
            let unread_after =
                FakeVolumes::default().with("~", facts).answering_available(Ok(fresher_core::ByteSize::new(100)));
            for volumes in [FakeVolumes::default(), unread_after] {
                let engine = scanned(fixture().volumes(volumes));
                let events = engine.subscribe();
                let (_, echo) = prepared(&engine, "trash");

                engine.confirm_cleanup(&echo).unwrap();

                assert_eq!(report(&wait_for_cleanup(&events)).measured, None);
            }
        }

        #[test]
        fn a_plan_of_no_item_is_refused_and_journals_nothing() {
            let cleaner = Arc::new(FakeCleaner::default().skipping(FindingId::new(1), SkipReason::Vanished));
            let fixture = fixture().cleaner(Arc::clone(&cleaner));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let events = engine.subscribe();
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            assert_eq!((view.summary.count, view.left_out.len()), (0, 1));

            let error = engine.confirm_cleanup(&echo(&view, "trash"));

            assert_eq!(error, Err(CommandError::NothingToClean));
            assert_eq!(cleaner.executed(), []);
            assert_eq!(journal.recorded().len(), 0);
            assert_eq!(engine.cleanups(0, 10).total, 0);
            assert_eq!(events.try_iter().count(), 0);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            assert_eq!(engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
        }

        #[test]
        fn a_finding_inside_an_item_that_did_not_wholly_go_stays() {
            let outcomes =
                [ItemOutcome::Skipped { reason: SkipReason::Grown }, ItemOutcome::Emptied { moved: 1, skipped: 1 }];
            for outcome in outcomes {
                let cleaner = FakeCleaner::default().ending(FindingId::new(3), outcome.clone());
                let engine = scanned(fixture().cleaner(Arc::new(cleaner)));
                let events = engine.subscribe();
                let view = engine.prepare_cleanup(&select(&[3], &[])).unwrap();

                engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
                wait_for_cleanup(&events);

                assert!(engine.finding(FindingId::new(4)).is_ok(), "{outcome:?}");
            }
        }

        #[test]
        fn an_emptied_folder_did_not_go() {
            let paths = [std::path::PathBuf::from("/home/a"), std::path::PathBuf::from("/home/b")];
            let emptied =
                [ItemOutcome::Emptied { moved: 2, skipped: 0 }, ItemOutcome::Emptied { moved: 1, skipped: 1 }];
            for outcome in emptied {
                assert_eq!(super::super::went(&outcome, &paths[..1], None), [] as [std::path::PathBuf; 0]);
            }
            let skipped = ItemOutcome::Skipped { reason: SkipReason::Grown };
            assert_eq!(super::super::went(&skipped, &paths, None), [] as [std::path::PathBuf; 0]);
        }

        #[test]
        fn a_skipped_item_stays_in_the_results() {
            let cleaner =
                FakeCleaner::default().ending(FindingId::new(1), ItemOutcome::Skipped { reason: SkipReason::Grown });
            let engine = scanned(fixture().cleaner(Arc::new(cleaner)));
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            engine.confirm_cleanup(&echo).unwrap();
            let seen = wait_for_cleanup(&events);

            assert!(engine.finding(FindingId::new(1)).is_ok());
            let removed = seen.iter().find_map(|event| match event {
                Event::FindingsRemoved { findings, .. } => Some(findings.clone()),
                _ => None,
            });
            assert_eq!(removed, Some([3, 2, 4].map(FindingId::new).to_vec()));
        }

        #[test]
        fn a_cleaner_that_panics_ends_the_cleanup_stopped() {
            #[derive(Debug)]
            struct Panicking(FakeCleaner);

            impl fresher_core::Cleaner for Panicking {
                fn rehearse(
                    &self,
                    roots: &[std::path::PathBuf],
                    items: &[fresher_core::PlanItem],
                    cx: &fresher_core::CleanupContext<'_>,
                    depth: fresher_core::Depth,
                ) -> Vec<fresher_core::Rehearsal> {
                    self.0.rehearse(roots, items, cx, depth)
                }

                fn execute(
                    &self,
                    _plan: fresher_core::ConfirmedPlan,
                    _cx: &fresher_core::CleanupContext<'_>,
                    _cancel: &fresher_core::CancelToken,
                    _out: &mut dyn FnMut(fresher_core::ItemDone),
                ) -> Ended {
                    panic_now("the cleaner panics")
                }

                fn protection(&self) -> fresher_core::Protection {
                    self.0.protection()
                }

                fn locate(&self, items: &[fresher_core::TrashedItem]) -> Vec<fresher_core::TrashState> {
                    self.0.locate(items)
                }

                fn put_back(
                    &self,
                    item: &fresher_core::TrashedItem,
                    cx: &fresher_core::CleanupContext<'_>,
                ) -> fresher_core::PutBackOutcome {
                    self.0.put_back(item, cx)
                }
            }

            let fixture = fixture().cleaner(Arc::new(Panicking(FakeCleaner::default())));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            let cleanup = engine.confirm_cleanup(&echo).unwrap();
            let seen = wait_for_cleanup(&events);
            wait_until_idle(&engine);

            assert_eq!(report(&seen).ended, Ended::Stopped);
            assert_eq!(journal.recorded()[0].end.as_ref().map(|end| end.ended), Some(Ended::Stopped));
            assert_eq!(engine.overview().unwrap().last_scan.and_then(|last| last.changed_since), Some(cleanup));
            assert_eq!(cleanup, CleanupId::new(1));
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{
            ByteSize, CleanupId, CommandError, Ended, Event, FindingId, ItemOutcome, Method, RecordedOutcome, ScanId,
            SkipReason, VolumeFacts, VolumeId,
        };
        use fresher_testing::{FakeCleaner, FakeVolumes};

        use super::{held, prepared};
        use crate::cleanup::testing::{echo, fixture, report, scanned, select, wait_for_cleanup, withdraw_agreement};
        use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes};

        #[test]
        fn put_back_needs_no_acceptance() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");
            let cleanup = engine.confirm_cleanup(&echo).unwrap();
            wait_for_cleanup(&events);
            withdraw_agreement(&engine);
            assert!(!engine.agreement().unwrap().is_accepted());

            let report = engine.put_back(cleanup, None).unwrap();

            assert_eq!(report.items.len(), 3);
            assert!(report.items.iter().all(|item| item.outcome == fresher_core::PutBackOutcome::PutBack));
            assert_eq!(cleaner.put_backs_tried().len(), 3);
        }

        #[test]
        fn the_begin_record_is_written_before_the_first_item() {
            let fixture = fixture();
            let journal = Arc::clone(&fixture.journal);
            let (engine, _, gate) = held(fixture);
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            let cleanup = engine.confirm_cleanup(&echo).unwrap();

            let recorded = journal.recorded();
            assert_eq!(recorded[0].cleanup, cleanup);
            assert_eq!(recorded[0].begin.method, Method::Trash);
            assert_eq!(recorded[0].begin.items.len(), 3);
            assert_eq!(recorded[0].begin.roots, [std::path::PathBuf::from("~")]);
            assert_eq!(recorded[0].items, []);
            gate.release();
            wait_for_cleanup(&events);
            assert_eq!(journal.recorded()[0].items.len(), 3);
        }

        #[test]
        fn events_come_started_items_finished_in_order() {
            let engine = scanned(fixture());
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            let cleanup = engine.confirm_cleanup(&echo).unwrap();
            let seen = wait_for_cleanup(&events);

            let kinds: Vec<_> = seen
                .iter()
                .map(|event| match event {
                    Event::CleanupStarted { method, total, .. } => format!("started {method:?} {total}"),
                    Event::CleanupItems { done, items, .. } => format!("items {done} {}", items.len()),
                    Event::FindingsRemoved { scan, findings } => format!("removed {} {}", scan.get(), findings.len()),
                    Event::CleanupFinished { .. } => "finished".to_owned(),
                    other => format!("{other:?}"),
                })
                .collect();
            assert_eq!(kinds, ["started Trash 3", "items 3 3", "removed 1 4", "finished"]);
            assert!(seen.iter().all(|event| match event {
                Event::CleanupStarted { cleanup: id, .. }
                | Event::CleanupItems { cleanup: id, .. }
                | Event::CleanupFinished { cleanup: id, .. } => *id == cleanup,
                _ => true,
            }));
            let reports: Vec<_> = seen
                .iter()
                .filter_map(|event| match event {
                    Event::CleanupItems { items, .. } => {
                        Some(items.iter().map(|item| (item.index, item.finding.get())))
                    }
                    _ => None,
                })
                .flatten()
                .collect();
            assert_eq!(reports, [(0, 3), (1, 1), (2, 2)]);
        }

        #[test]
        fn outcomes_are_batched_and_the_journal_synced_every_50_items() {
            let walk = (0..300)
                .fold(ScriptedWalk::new(), |walk, n| walk.folder("Caches", &format!("/home/{n}/Caches"), bytes(1)));
            let alpha =
                TestModule::builder("alpha").category("alpha.caches", fresher_core::DangerLevel::Safe).folder("Caches");
            let fixture = Fixture::new(walk).module(alpha);
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let events = engine.subscribe();
            let view = engine.prepare_cleanup(&select(&[], &[("alpha.caches", &[])])).unwrap();
            let syncs = journal.syncs();

            engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
            let seen = wait_for_cleanup(&events);

            let batches: Vec<_> = seen
                .iter()
                .filter_map(|event| match event {
                    Event::CleanupItems { done, items, .. } => Some((*done, items.len())),
                    _ => None,
                })
                .collect();
            assert_eq!(batches, [(256, 256), (300, 44)]);
            // The begin, one every 50 items, the end.
            assert_eq!(journal.syncs() - syncs, 1 + 6 + 1);
        }

        #[test]
        fn the_report_counts_and_measures() {
            let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::ZERO };
            let volumes = FakeVolumes::default()
                .with("~", facts)
                .answering_available(Ok(ByteSize::new(1_000)))
                .answering_available(Ok(ByteSize::new(5_000)));
            let cleaner =
                FakeCleaner::default().ending(FindingId::new(2), ItemOutcome::Skipped { reason: SkipReason::Grown });
            let fixture = fixture().volumes(volumes).cleaner(Arc::new(cleaner));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");

            engine.confirm_cleanup(&echo).unwrap();
            let report = report(&wait_for_cleanup(&events));

            assert_eq!(report.method, Method::Trash);
            assert_eq!(report.ended, Ended::Completed);
            assert_eq!((report.counts.moved, report.counts.skipped), (2, 1));
            assert_eq!(report.moved, bytes(40));
            assert_eq!(report.measured, Some(ByteSize::new(4_000)));
            let recorded = &journal.recorded()[0];
            assert_eq!(recorded.begin.space[0].available, ByteSize::new(1_000));
            assert_eq!(recorded.end.as_ref().map(|end| end.space[0].available), Some(ByteSize::new(5_000)));
            assert_eq!(recorded.outcome(2), RecordedOutcome::Skipped { reason: fresher_core::SkipCode::Grown });
        }

        #[test]
        fn a_permanent_cleanup_deletes() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "permanent");

            engine.confirm_cleanup(&echo).unwrap();
            let report = report(&wait_for_cleanup(&events));

            assert_eq!(cleaner.executed()[0].method, Method::Permanent);
            assert_eq!((report.counts.deleted, report.deleted), (3, bytes(60)));
        }

        #[test]
        fn removed_findings_leave_the_overview() {
            let engine = scanned(fixture());
            let events = engine.subscribe();
            let before = engine.overview().unwrap();
            let (_, echo) = prepared(&engine, "trash");

            engine.confirm_cleanup(&echo).unwrap();
            let seen = wait_for_cleanup(&events);

            let removed = seen.iter().find_map(|event| match event {
                Event::FindingsRemoved { scan, findings } => Some((*scan, findings.clone())),
                _ => None,
            });
            assert_eq!(removed, Some((ScanId::new(1), [3, 1, 2, 4].map(FindingId::new).to_vec())));
            for id in [1, 2, 3, 4] {
                let id = FindingId::new(id);
                assert_eq!(engine.finding(id), Err(CommandError::UnknownFinding { id }));
            }
            let after = engine.overview().unwrap();
            assert_eq!(before.totals.len(), 2);
            assert_eq!(after.totals, []);
            assert_eq!(after.found, fresher_core::Sizes::ZERO);
        }

        #[test]
        fn a_finding_inside_a_removed_item_leaves_with_it() {
            let outcomes = [ItemOutcome::Moved, ItemOutcome::Deleted, ItemOutcome::Emptied { moved: 2, skipped: 0 }];
            for outcome in outcomes {
                let cleaner = FakeCleaner::default().ending(FindingId::new(3), outcome.clone());
                let engine = scanned(fixture().cleaner(Arc::new(cleaner)));
                let events = engine.subscribe();
                let view = engine.prepare_cleanup(&select(&[3], &[])).unwrap();

                engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
                let seen = wait_for_cleanup(&events);

                let removed = seen.iter().find_map(|event| match event {
                    Event::FindingsRemoved { findings, .. } => Some(findings.clone()),
                    _ => None,
                });
                assert_eq!(removed, Some([3, 4].map(FindingId::new).to_vec()), "{outcome:?}");
                let id = FindingId::new(4);
                assert_eq!(engine.finding(id), Err(CommandError::UnknownFinding { id }), "{outcome:?}");
                let after = engine.overview().unwrap();
                assert_eq!((after.totals[0].count, after.totals[0].sizes), (1, bytes(10)), "{outcome:?}");
                assert_eq!(after.found, bytes(30), "{outcome:?}");
            }
        }

        #[test]
        fn a_moved_or_deleted_item_went_whole_and_a_partly_done_one_its_first_parts() {
            let paths = [std::path::PathBuf::from("/home/a"), std::path::PathBuf::from("/home/b")];
            let partly = ItemOutcome::Failed { reason: fresher_core::Failure::TrashRefused.text() };

            assert_eq!(super::super::went(&ItemOutcome::Moved, &paths, None), paths);
            assert_eq!(super::super::went(&ItemOutcome::Deleted, &paths[..1], None), &paths[..1]);
            assert_eq!(super::super::went(&partly, &paths, Some(1)), &paths[..1]);
            assert_eq!(super::super::went(&partly, &paths, Some(9)), paths);
        }

        #[test]
        fn the_last_scan_is_marked_changed() {
            let engine = scanned(fixture());
            let events = engine.subscribe();
            let (_, echo) = prepared(&engine, "trash");
            assert_eq!(engine.overview().unwrap().last_scan.and_then(|last| last.changed_since), None);

            let cleanup = engine.confirm_cleanup(&echo).unwrap();
            wait_for_cleanup(&events);

            assert_eq!(cleanup, CleanupId::new(1));
            assert_eq!(engine.overview().unwrap().last_scan.and_then(|last| last.changed_since), Some(cleanup));
        }

        #[test]
        fn quit_while_cleaning_asks_and_quit_after_cleanup_waits_for_the_item_in_flight() {
            let fixture = fixture();
            let journal = Arc::clone(&fixture.journal);
            let (engine, cleaner, _gate) = held(fixture);
            let events = engine.subscribe();
            assert!(!engine.request_quit());
            let (_, echo) = prepared(&engine, "trash");
            engine.confirm_cleanup(&echo).unwrap();
            cleaner.wait_until_holding();

            assert!(engine.request_quit());
            engine.quit_after_cleanup();

            assert_eq!(engine.status(), Ok(fresher_core::EngineStatus::Idle));
            let seen: Vec<_> = events.try_iter().collect();
            assert!(seen.contains(&Event::QuitRequested));
            assert_eq!(report(&seen).ended, Ended::Cancelled);
            let recorded = &journal.recorded()[0];
            assert_eq!(recorded.end.as_ref().map(|end| end.ended), Some(Ended::Cancelled));
            let outcomes: Vec<_> = (0..3).map(|index| recorded.outcome(index)).collect();
            assert_eq!(outcomes, [RecordedOutcome::Moved, RecordedOutcome::NotReached, RecordedOutcome::NotReached]);
        }
    }
}
