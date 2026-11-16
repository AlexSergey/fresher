//! The pending plan: at most one, made by `prepare` and taken by a confirmation. It dies — and a later confirmation is
//! refused with `PlanGone` — when a scan or a restore starts, a newer plan is prepared, it is discarded, a setting that
//! changes what is scanned changes, or [`PLAN_LIFETIME`](fresher_core::PLAN_LIFETIME) has passed (the confirmation
//! checks that).

use fresher_core::{
    CategoryDescriptor, CleanupPlan, CleanupPlanView, CleanupSelection, CommandError, Depth, PlanError, PlanId,
    Timestamp,
};

use crate::cleanup::context::Context;
use crate::cleanup::{blocked, plan, view};
use crate::engine::Shared;
use crate::error::{EngineError, lock};

/// The plan waiting for its confirmation, the id of the last plan made, and how many times a plan died: a plan
/// prepared while one died is not kept.
#[derive(Debug, Default)]
pub(crate) struct Pending {
    plan: Option<CleanupPlan>,
    last_id: u32,
    deaths: u64,
}

impl Pending {
    /// The pending plan dies, if there is one; answers the count a plan prepared from now on must find to be kept.
    pub(crate) fn kill(&mut self) -> u64 {
        self.plan = None;
        self.deaths = self.deaths.wrapping_add(1);
        self.deaths
    }

    /// The pending plan dies when it is `id`; any other id changes nothing.
    pub(crate) fn discard(&mut self, id: PlanId) {
        if self.plan.as_ref().is_some_and(|plan| plan.id() == id) {
            self.kill();
        }
    }

    /// Takes the pending plan for its confirmation: it is pending no more, whatever the confirmation says.
    pub(crate) fn take(&mut self) -> Option<CleanupPlan> {
        let plan = self.plan.take();
        self.kill();
        plan
    }

    /// Keeps the plan `make` builds with the next id, when no plan died since `deaths` was answered; `PlanGone`
    /// otherwise.
    fn keep(
        &mut self,
        deaths: u64,
        make: impl FnOnce(PlanId) -> Result<CleanupPlan, CommandError>,
    ) -> Result<&CleanupPlan, CommandError> {
        if deaths != self.deaths {
            return Err(CommandError::PlanGone);
        }
        let id = self.last_id.checked_add(1).ok_or(EngineError::PlanIdsExhausted)?;
        let plan = make(PlanId::new(id))?;
        self.last_id = id;
        Ok(self.plan.insert(plan))
    }
}

/// Prepares the plan of `selection` over the last scan: refused while an operation runs (`Busy`), before any scan
/// (`NoScan`), for a selection the builder refuses, or when cleaning is blocked. The findings the builder keeps are
/// rehearsed (`Depth::Quick`) with the context as it stands now; the plan, frozen, replaces the pending one.
pub(crate) fn prepare(shared: &Shared, selection: &CleanupSelection) -> Result<CleanupPlanView, CommandError> {
    lock(&shared.machine)?.idle()?;
    let deaths = lock(&shared.pending)?.kill();
    let (scan, roots, selected) = {
        let results = lock(&shared.results)?;
        let last = results.last().ok_or(CommandError::NoScan)?;
        let categories: Vec<_> = shared.registry.categories().map(CategoryDescriptor::id).collect();
        let selected: Vec<_> = plan::expand(selection, &results.visible(), &categories)?.into_iter().cloned().collect();
        (last.scan, results.roots().to_vec(), selected)
    };
    let ports = blocked::check(lock(&shared.settings)?.file(), shared.cleanup.as_ref())?;
    let selected: Vec<_> = selected.iter().collect();
    let sorted = plan::sort_out(&selected, &[]);
    let cx = Context::now(shared)?;
    let answers = ports.cleaner.rehearse(&roots, &plan::to_rehearse(&sorted), &cx.borrow(), Depth::Quick);
    let (items, left_out) = plan::freeze(sorted, answers)?;
    let made = Timestamp::from_system_time(shared.ports.clock.now());

    let machine = lock(&shared.machine)?;
    machine.idle()?;
    let mut pending = lock(&shared.pending)?;
    let plan = pending.keep(deaths, |id| {
        CleanupPlan::new(id, made, scan, roots, items, left_out).map_err(|error| match error {
            PlanError::TooManyItems { limit } => CommandError::TooManyItems { limit },
            error @ (PlanError::Duplicate { .. } | PlanError::Overflow) => EngineError::Plan(error).into(),
        })
    })?;
    Ok(view::of(plan)?)
}

#[cfg(test)]
mod tests {
    use fresher_core::PlanId;

    use crate::Engine;

    /// The id of the pending plan, if there is one.
    fn pending(engine: &Engine) -> Option<PlanId> {
        engine.shared.pending.lock().unwrap().plan.as_ref().map(fresher_core::CleanupPlan::id)
    }

    mod negative {
        use std::sync::Arc;
        use std::time::Duration;

        use fresher_core::{
            BlockedReason, CleanupPlan, CommandError, ConfirmError, FindingId, Operation, PLAN_LIFETIME, SettingUpdate,
            SettingValue, SkipReason, StoreError, Timestamp,
        };
        use fresher_testing::{FakeCleaner, MemoryCleanupJournal};

        use super::pending;
        use crate::cleanup::pending::Pending;
        use crate::cleanup::testing::{echo, fixture, left_out, scanned, select};
        use crate::testing::{MemorySettings, START, ScriptedWalk, engine, scan, wait_for_finish};
        use crate::{Engine, Ports, Registry};

        #[test]
        fn prepare_while_scanning_is_busy() {
            let (walk, gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::Busy { running: Operation::Scan }));
            gate.release();
            wait_for_finish(&events);
        }

        #[test]
        fn prepare_before_any_scan_is_no_scan() {
            let engine = fixture().engine();

            assert_eq!(engine.prepare_cleanup(&select(&[1], &[])), Err(CommandError::NoScan));
        }

        #[test]
        fn an_unknown_finding_or_category_is_refused() {
            let engine = scanned(fixture());

            let finding = engine.prepare_cleanup(&select(&[99], &[]));
            let category = engine.prepare_cleanup(&select(&[], &[("gamma.caches", &[])]));

            assert_eq!(finding, Err(CommandError::UnknownFinding { id: FindingId::new(99) }));
            assert_eq!(category, Err(CommandError::UnknownCategory { id: "gamma.caches".into() }));
            assert_eq!(engine.prepare_cleanup(&select(&[], &[])), Err(CommandError::NothingToClean));
        }

        #[test]
        fn an_unloaded_settings_file_blocks_cleaning() {
            let engine = scanned(fixture().settings(MemorySettings::unloadable(StoreError::Broken)));

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings }));
        }

        #[test]
        fn a_read_only_journal_blocks_cleaning() {
            let engine = scanned(fixture().journal(MemoryCleanupJournal::read_only()));

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }));
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn an_engine_without_cleanup_ports_blocks_cleaning() {
            let fixture = fixture();
            let ports = Ports {
                walk: Arc::new(ScriptedWalk::new().folder("Caches", "/home/a/Caches", crate::testing::bytes(1))),
                settings: fixture.settings,
                clock: fixture.clock,
                commands: fixture.commands,
                access: fixture.access,
                system_ui: fixture.system_ui,
            };
            let mut registry = Registry::new().unwrap();
            let alpha = crate::testing::TestModule::builder("alpha")
                .category("alpha.caches", fresher_core::DangerLevel::Safe)
                .folder("Caches");
            registry.register(alpha.boxed()).unwrap();
            let engine = Engine::new(registry, ports).unwrap();
            scan(&engine);

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }));
        }

        #[test]
        fn a_rehearsal_skip_is_left_out_with_its_reason() {
            let cleaner = Arc::new(FakeCleaner::default().skipping(FindingId::new(1), SkipReason::Vanished));
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));

            let view = engine.prepare_cleanup(&select(&[1, 2], &[])).unwrap();

            assert_eq!(left_out(&view), [FindingId::new(1)]);
            assert_eq!(view.left_out[0].reason(), &SkipReason::Vanished);
            assert_eq!(view.summary.count, 1);
        }

        #[test]
        fn a_new_scan_kills_the_pending_plan() {
            let engine = scanned(fixture());
            engine.prepare_cleanup(&select(&[1], &[])).unwrap();

            scan(&engine);

            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn a_newer_plan_replaces_the_pending_one() {
            let engine = scanned(fixture());
            let first = engine.prepare_cleanup(&select(&[1], &[])).unwrap();

            let second = engine.prepare_cleanup(&select(&[2], &[])).unwrap();
            let refused = engine.prepare_cleanup(&select(&[99], &[]));

            assert_ne!(first.plan, second.plan);
            assert!(refused.is_err());
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn a_changed_root_kills_the_pending_plan() {
            let engine = scanned(fixture());
            let change = |key: &str, value| engine.update_setting(SettingUpdate { key: key.into(), value }).unwrap();

            engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            change("core.scan.roots", SettingValue::Folders(vec!["~/Projects".into()]));
            assert_eq!(pending(&engine), None);

            engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            change("beta.enabled", SettingValue::Switch(false));
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn a_plan_past_its_lifetime_is_gone() {
            let fixture = fixture();
            let clock = Arc::clone(&fixture.clock);
            let engine = scanned(fixture);
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            let plan: CleanupPlan = engine.shared.pending.lock().unwrap().plan.take().unwrap();

            clock.advance(PLAN_LIFETIME + Duration::from_secs(1));
            let now = Timestamp::from_system_time(fresher_core::Clock::now(&*clock));

            assert_eq!(plan.made(), Timestamp::from_secs(START));
            assert_eq!(plan.confirm(&echo(&view, "trash"), now).err(), Some(ConfirmError::Expired));
        }

        #[test]
        fn a_plan_made_while_one_died_is_not_kept() {
            let mut pending = Pending::default();
            let deaths = pending.kill();
            pending.kill();

            let kept = pending.keep(deaths, |_| unreachable!("a plan is not made"));

            assert_eq!(kept.err(), Some(CommandError::PlanGone));
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{ActionKind, DangerLevel, EntryKind, FindingId, ItemIdentity, SettingUpdate, SettingValue};
        use fresher_testing::FakeCleaner;

        use super::pending;
        use crate::cleanup::testing::{fixture, items, left_out, scanned, select};

        #[test]
        fn the_view_groups_by_category_in_the_overviews_order() {
            let engine = scanned(fixture());

            let view = engine.prepare_cleanup(&select(&[2], &[("alpha.caches", &[])])).unwrap();

            let groups: Vec<_> = view.groups.iter().map(|group| (group.category.as_str(), group.action)).collect();
            assert_eq!(groups, [("alpha.caches", ActionKind::Remove), ("beta.logs", ActionKind::Remove)]);
            assert_eq!(items(&view), [3, 1, 2]);
            assert_eq!(left_out(&view), [FindingId::new(4)]);
            assert_eq!(view.groups[0].summary.count, 2);
            assert_eq!(view.groups[0].summary.sizes.allocated.bytes(), 40);
            assert_eq!(view.summary.count, 3);
            assert_eq!(view.summary.sizes.allocated.bytes(), 60);
            assert_eq!(view.summary.highest, DangerLevel::Low);
            assert!(view.permanent_available);
            assert_eq!(pending(&engine), Some(view.plan));
        }

        #[test]
        fn the_items_carry_the_rehearsals_identities() {
            let cleaner = Arc::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::clone(&cleaner)));

            engine.prepare_cleanup(&select(&[1, 2], &[])).unwrap();

            assert_eq!(cleaner.rehearsed(), [vec![FindingId::new(1), FindingId::new(2)]]);
            let pending = engine.shared.pending.lock().unwrap();
            let plan = pending.plan.as_ref().unwrap();
            let identities: Vec<_> = plan.items().iter().map(fresher_core::PlanItem::identity).collect();
            assert_eq!(
                identities,
                [Some(ItemIdentity::new(1, 1, EntryKind::Folder)), Some(ItemIdentity::new(1, 2, EntryKind::Folder))]
            );
            assert_eq!(plan.roots(), [std::path::PathBuf::from("~")]);
        }

        #[test]
        fn discard_drops_the_pending_plan() {
            let engine = scanned(fixture());
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();

            engine.discard_cleanup(fresher_core::PlanId::new(view.plan.get() + 1)).unwrap();
            assert_eq!(pending(&engine), Some(view.plan));

            engine.discard_cleanup(view.plan).unwrap();
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn a_setting_that_changes_nothing_scanned_keeps_the_pending_plan() {
            let engine = scanned(fixture());
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();

            let update = SettingUpdate { key: "core.access.desktop".into(), value: SettingValue::Switch(true) };
            engine.update_setting(update).unwrap();

            assert_eq!(pending(&engine), Some(view.plan));
        }
    }
}
