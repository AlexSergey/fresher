//! The pending plan: at most one, made by `prepare` and taken by a confirmation. It dies — and a later confirmation is
//! refused with `PlanGone` — when a scan starts, a newer plan is prepared, it is discarded, a setting that
//! changes what is scanned changes, or [`PLAN_LIFETIME`](fresher_core::PLAN_LIFETIME) has passed (the confirmation
//! checks that).

use std::path::PathBuf;

use fresher_core::{
    AppId, BundleId, CategoryDescriptor, CleanupPlan, CleanupPlanView, CleanupSelection, CommandError, Depth, Finding,
    PlanError, PlanId, ScanId, Timestamp,
};

use crate::cleanup::context::{Context, Scope};
use crate::cleanup::{blocked, plan, view};
use crate::engine::Shared;
use crate::error::{EngineError, lock};
use crate::own::offered_picks;

/// What the pending plan was made for: its confirmation is judged and cleaned for it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Purpose {
    /// The findings and picks the user selected.
    #[default]
    Selection,
    /// The uninstall of app `id`, of bundle id `app`, whose bundle is `bundle` as the system listed it.
    Uninstall { id: AppId, app: BundleId, bundle: PathBuf },
}

impl Purpose {
    /// The scope of the cleanup context the plan is rehearsed and cleaned with.
    pub(crate) const fn scope(&self) -> Scope {
        match self {
            Self::Selection => Scope::Cleanup,
            Self::Uninstall { .. } => Scope::Uninstall,
        }
    }
}

/// The plan waiting for its confirmation and what it was made for, the id of the last plan made, and how many times a
/// plan died: a plan prepared while one died is not kept.
#[derive(Debug, Default)]
pub(crate) struct Pending {
    plan: Option<CleanupPlan>,
    purpose: Purpose,
    last_id: u32,
    deaths: u64,
}

impl Pending {
    /// The pending plan dies, if there is one; answers the count a plan prepared from now on must find to be kept.
    pub(crate) fn kill(&mut self) -> u64 {
        self.plan = None;
        self.purpose = Purpose::Selection;
        self.deaths = self.deaths.wrapping_add(1);
        self.deaths
    }

    /// The pending plan and what it was made for, if there is one.
    #[cfg(test)]
    pub(crate) fn pending(&self) -> Option<(&CleanupPlan, &Purpose)> {
        self.plan.as_ref().map(|plan| (plan, &self.purpose))
    }

    /// The pending plan dies when it is `id`; any other id changes nothing.
    pub(crate) fn discard(&mut self, id: PlanId) {
        if self.plan.as_ref().is_some_and(|plan| plan.id() == id) {
            self.kill();
        }
    }

    /// Takes the pending plan and its purpose for its confirmation: it is pending no more, whatever the confirmation
    /// says.
    pub(crate) fn take(&mut self) -> Option<(CleanupPlan, Purpose)> {
        let plan = self.plan.take();
        let purpose = std::mem::take(&mut self.purpose);
        self.kill();
        plan.map(|plan| (plan, purpose))
    }

    /// Keeps the plan `make` builds with the next id, for `purpose`, when no plan died since `deaths` was answered;
    /// `PlanGone` otherwise.
    fn keep(
        &mut self,
        deaths: u64,
        purpose: Purpose,
        make: impl FnOnce(PlanId) -> Result<CleanupPlan, CommandError>,
    ) -> Result<&CleanupPlan, CommandError> {
        if deaths != self.deaths {
            return Err(CommandError::PlanGone);
        }
        let id = self.last_id.checked_add(1).ok_or(EngineError::PlanIdsExhausted)?;
        let plan = make(PlanId::new(id))?;
        self.last_id = id;
        self.purpose = purpose;
        Ok(self.plan.insert(plan))
    }
}

/// Prepares the plan of `selection` over the last scan and the rows the user picked that are still offered now:
/// refused while an operation runs (`Busy`), before any scan (`NoScan`), for a selection the builder refuses, or when
/// cleaning is blocked. The findings the builder keeps are rehearsed (`Depth::Quick`) with the context as it stands
/// now; the plan, frozen, replaces the pending one.
pub(crate) fn prepare(shared: &Shared, selection: &CleanupSelection) -> Result<CleanupPlanView, CommandError> {
    lock(&shared.machine)?.idle()?;
    let deaths = lock(&shared.pending)?.kill();
    let picks = offered_picks(shared)?;
    let (scan, roots, selected) = {
        let results = lock(&shared.results)?;
        let last = results.last().ok_or(CommandError::NoScan)?;
        let categories: Vec<_> = shared.registry.categories().map(CategoryDescriptor::id).collect();
        // The picks' ids follow every id of the scan: the list stays in id order, as the builder needs.
        let findings: Vec<_> = results.visible().into_iter().chain(picks).collect();
        let selected: Vec<_> = plan::expand(selection, &findings, &categories)?.into_iter().cloned().collect();
        (last.scan, results.walked_roots(), selected)
    };
    let selected: Vec<_> = selected.iter().collect();
    rehearse_and_keep(shared, Planned { deaths, scan, roots, purpose: Purpose::Selection }, &selected)
}

/// What a plan is made of besides its findings.
#[derive(Debug)]
pub(crate) struct Planned {
    /// The deaths count answered when the pending plan was killed.
    pub(crate) deaths: u64,
    pub(crate) scan: ScanId,
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) purpose: Purpose,
}

/// The tail every plan is made by: refused with `HighWithheld` when an item of `selected` is High while High is
/// withheld, before anything else is checked; refused when cleaning is blocked; `selected` (in the confirmation's
/// order) sorted out, the findings kept rehearsed (`Depth::Quick`) with the context of the purpose's scope as it stands
/// now, frozen, and kept as the pending plan, unless an operation started or a plan died meanwhile.
pub(crate) fn rehearse_and_keep(
    shared: &Shared,
    planned: Planned,
    selected: &[&Finding],
) -> Result<CleanupPlanView, CommandError> {
    if selected.iter().any(|finding| !shared.high.offers(finding.finding().danger())) {
        return Err(CommandError::HighWithheld);
    }
    let Planned { deaths, scan, roots, purpose } = planned;
    let ports = blocked::check(&*lock(&shared.settings)?, shared.cleanup.as_ref())?;
    let sorted = plan::sort_out(selected, &[]);
    let cx = Context::now(shared, purpose.scope())?;
    let answers = ports.cleaner.rehearse(&roots, &plan::to_rehearse(&sorted), &cx.borrow(), Depth::Quick);
    let (items, left_out) = plan::freeze(sorted, answers)?;
    let made = Timestamp::from_system_time(shared.ports.clock.now());

    let machine = lock(&shared.machine)?;
    machine.idle()?;
    let mut pending = lock(&shared.pending)?;
    let view_purpose = purpose.clone();
    let plan = pending.keep(deaths, purpose, |id| {
        CleanupPlan::new(id, made, scan, roots, items, left_out).map_err(|error| match error {
            PlanError::TooManyItems { limit } => CommandError::TooManyItems { limit },
            error @ (PlanError::Duplicate { .. } | PlanError::Overflow) => EngineError::Plan(error).into(),
        })
    })?;
    Ok(view::of(plan, &view_purpose)?)
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
            AGREEMENT_VERSION, BlockedReason, CleanupPlan, CommandError, ConfirmError, FindingId, Operation,
            PLAN_LIFETIME, SettingUpdate, SettingValue, SkipReason, StoreError, Timestamp,
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
        fn prepare_without_the_agreement_is_blocked_and_holds_no_plan() {
            let engine = scanned(fixture().unaccepted());

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Agreement }));
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn a_broken_settings_file_is_reported_before_the_agreement() {
            let engine = scanned(fixture().unaccepted().settings(MemorySettings::unloadable(StoreError::Broken)));

            let error = engine.prepare_cleanup(&select(&[1], &[]));

            assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings }));
        }

        #[test]
        fn accepting_on_a_broken_settings_file_saves_nothing_and_cleaning_stays_blocked() {
            let fixture = fixture().unaccepted().settings(MemorySettings::unloadable(StoreError::Broken));
            let settings = Arc::clone(&fixture.settings);
            let engine = scanned(fixture);

            let accepted = engine.accept_agreement(AGREEMENT_VERSION);
            let error = engine.prepare_cleanup(&select(&[1], &[]));

            let blocked = CommandError::CleaningBlocked { reason: BlockedReason::Settings };
            assert_eq!(accepted, Err(blocked.clone()));
            assert_eq!(settings.saves(), 0);
            assert_eq!(error, Err(blocked));
            assert_eq!(pending(&engine), None);
        }

        #[test]
        fn an_acceptance_of_another_version_is_blocked() {
            let at = serde_json::json!(START);
            for version in [0, AGREEMENT_VERSION + 1] {
                let stored = [
                    ("core.agreement.version", serde_json::json!(version)),
                    ("core.agreement.accepted-at", at.clone()),
                ];
                let engine = scanned(fixture().unaccepted().settings(MemorySettings::with(&stored)));

                let error = engine.prepare_cleanup(&select(&[1], &[]));

                assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Agreement }), "{version}");
                assert_eq!(pending(&engine), None);
            }
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
            fixture.settings.seed_acceptance(START);
            let ports = Ports {
                walk: Arc::new(ScriptedWalk::new().folder("Caches", "/home/a/Caches", crate::testing::bytes(1))),
                settings: fixture.settings,
                clock: fixture.clock,
                commands: fixture.commands,
                access: fixture.access,
                system_ui: fixture.system_ui,
                system: fixture.system,
            };
            let mut registry = Registry::new().unwrap();
            let alpha = crate::testing::TestModule::builder("alpha")
                .category("alpha.caches", fresher_core::DangerLevel::Safe)
                .folder("Caches");
            registry.register(alpha.boxed()).unwrap();
            let engine = Engine::new(registry, ports, fresher_core::HighLevel::Offered).unwrap();
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

            let kept = pending
                .keep(deaths, crate::cleanup::pending::Purpose::Selection, |_| unreachable!("a plan is not made"));

            assert_eq!(kept.err(), Some(CommandError::PlanGone));
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{ActionKind, DangerLevel, EntryKind, FindingId, ItemIdentity, SettingUpdate, SettingValue};
        use fresher_testing::FakeCleaner;

        use super::pending;
        use crate::cleanup::testing::{echo, fixture, items, left_out, scanned, select, wait_for_cleanup};

        #[test]
        fn after_acceptance_the_cleanup_runs() {
            let engine = scanned(fixture().unaccepted());
            let events = engine.subscribe();
            assert!(engine.prepare_cleanup(&select(&[1], &[])).is_err());

            engine.accept_agreement(fresher_core::AGREEMENT_VERSION).unwrap();
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            let id = engine.confirm_cleanup(&echo(&view, "trash")).unwrap();

            let seen = wait_for_cleanup(&events);
            assert!(
                matches!(seen.last(), Some(fresher_core::Event::CleanupFinished { cleanup, .. }) if *cleanup == id)
            );
            assert_eq!(pending(&engine), None);
        }

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
