//! The commands the UI calls: each one passes its input to the engine and returns the engine's answer. They run off
//! the main thread (`async`), so a command that writes the settings file never blocks the window. A refused command
//! returns a `Refusal`: the engine's error with the text the UI shows for it.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use fresher_core::{
    AccessStatus, AgreementView, AppInfo, Build, BuildFlags, Capability, CleanupDetail, CleanupId, CleanupItemRef,
    CleanupPlanView, CleanupSelection, CleanupsPage, CleanupsQuery, Confirmation, Dictionary, Event, FindingId,
    FindingsPage, FindingsQuery, IgnoreTarget, IgnoredEntry, Locale, ModuleInfo, Overview, PlanId, PutBackReport,
    PutBackRequest, Refusal, ScanId, SettingUpdate, SettingsView, TrashView,
};
use fresher_engine::{Engine, ScanMode};
use tauri::ipc::Channel;
use tauri::{AppHandle, Runtime, State};

use crate::events::EventForwarder;
use crate::shell;

/// The version of the app, the kind of build it is, the build's flags (their `high` is the engine's) and its home
/// folder.
#[tauri::command]
pub(crate) fn app_info(build: State<'_, Build>, flags: State<'_, BuildFlags>, engine: State<'_, Engine>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        build: build.inner().clone(),
        flags: BuildFlags { high: engine.high_level(), ..*flags.inner() },
        home: engine.home_folder(),
        now: engine.now(),
    }
}

/// Forwards every event of the engine to `channel` from now on, in place of the channel of an earlier subscribe.
#[tauri::command(async)]
pub(crate) fn subscribe(
    engine: State<'_, Engine>,
    forwarder: State<'_, EventForwarder>,
    channel: Channel<Event>,
) -> Result<(), Refusal> {
    forwarder.forward(&engine, channel).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn start_scan(engine: State<'_, Engine>) -> Result<ScanId, Refusal> {
    engine.start_scan(ScanMode::Auto).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn cancel_scan(engine: State<'_, Engine>) -> Result<(), Refusal> {
    engine.cancel_scan().map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn get_overview(engine: State<'_, Engine>) -> Result<Overview, Refusal> {
    engine.overview().map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn get_modules(engine: State<'_, Engine>) -> Vec<ModuleInfo> {
    engine.modules()
}

#[tauri::command(async)]
pub(crate) fn get_findings(engine: State<'_, Engine>, query: FindingsQuery) -> Result<FindingsPage, Refusal> {
    engine.findings(&query).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn get_settings(engine: State<'_, Engine>) -> Result<SettingsView, Refusal> {
    engine.settings().map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn update_setting(engine: State<'_, Engine>, update: SettingUpdate) -> Result<(), Refusal> {
    engine.update_setting(update).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn reset_settings(engine: State<'_, Engine>) -> Result<(), Refusal> {
    engine.reset_settings().map_err(Refusal::from)
}

/// The agreement the user accepts before the first cleanup, and the last acceptance of it.
#[tauri::command(async)]
pub(crate) fn get_agreement(engine: State<'_, Engine>) -> Result<AgreementView, Refusal> {
    engine.agreement().map_err(Refusal::from)
}

/// The user accepted version `version` of the agreement; refused when the text changed while they read it.
#[tauri::command(async)]
pub(crate) fn accept_agreement(engine: State<'_, Engine>, version: u32) -> Result<AgreementView, Refusal> {
    engine.accept_agreement(version).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn ignore(engine: State<'_, Engine>, target: IgnoreTarget) -> Result<(), Refusal> {
    engine.ignore(&target).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn restore_ignored(engine: State<'_, Engine>, entry: IgnoredEntry) -> Result<(), Refusal> {
    engine.restore_ignored(&entry).map_err(Refusal::from)
}

/// The merged texts of the core and every module; only `en` ships.
#[tauri::command(async)]
pub(crate) fn get_dictionary(engine: State<'_, Engine>, locale: &str) -> Result<Dictionary, Refusal> {
    Locale::parse(locale).map(|_| engine.dictionary().clone()).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn get_access_status(engine: State<'_, Engine>) -> AccessStatus {
    engine.access_status()
}

/// "Check again": probes the system's permissions and runs every module's tool check (the focus re-check runs no
/// tool), and answers the status (`access-changed` is published when something changed).
#[tauri::command(async)]
pub(crate) fn refresh_access(engine: State<'_, Engine>) -> AccessStatus {
    engine.check_access_again();
    engine.access_status()
}

/// Opens the system's settings at the pane that grants `capability`; never an address or a path from the UI.
#[tauri::command(async)]
pub(crate) fn open_access_settings(engine: State<'_, Engine>, capability: Capability) -> Result<(), Refusal> {
    engine.open_access_settings(&capability).map_err(Refusal::from)
}

/// "Show in Finder": shows a finding of the current scan in Finder, selected; takes the finding's id, never a path.
#[tauri::command(async)]
pub(crate) fn reveal_finding(engine: State<'_, Engine>, finding: FindingId) -> Result<(), Refusal> {
    engine.reveal_finding(finding).map_err(Refusal::from)
}

/// "Reopen Fresher": a Full Disk Access grant applies after a relaunch. Refused while a cleanup runs.
#[tauri::command(async)]
pub(crate) fn relaunch<R: Runtime>(app: AppHandle<R>) -> Result<(), Refusal> {
    shell::relaunch(&app).map_err(Refusal::from)
}

/// The plan of what `selection` would remove, rehearsed and nothing done: what the confirmation shows.
#[tauri::command(async)]
pub(crate) fn prepare_cleanup(
    engine: State<'_, Engine>,
    selection: CleanupSelection,
) -> Result<CleanupPlanView, Refusal> {
    engine.prepare_cleanup(&selection).map_err(Refusal::from)
}

/// The confirmation's Cancel: the plan is dropped.
#[tauri::command(async)]
pub(crate) fn discard_cleanup(engine: State<'_, Engine>, plan: PlanId) -> Result<(), Refusal> {
    engine.discard_cleanup(plan).map_err(Refusal::from)
}

/// The user confirmed the plan with the summary they saw; the cleanup runs on its own thread and reports by events.
#[tauri::command(async)]
pub(crate) fn confirm_cleanup(engine: State<'_, Engine>, confirmation: Confirmation) -> Result<CleanupId, Refusal> {
    engine.confirm_cleanup(&confirmation).map_err(Refusal::from)
}

/// Stops the running cleanup after the item in flight.
#[tauri::command(async)]
pub(crate) fn cancel_cleanup(engine: State<'_, Engine>) -> Result<(), Refusal> {
    engine.cancel_cleanup().map_err(Refusal::from)
}

/// A page of the history of cleanups, newest first, at most 100.
#[tauri::command(async)]
pub(crate) fn get_cleanups(engine: State<'_, Engine>, query: CleanupsQuery) -> CleanupsPage {
    engine.cleanups(query.offset(), query.limit())
}

#[tauri::command(async)]
pub(crate) fn get_cleanup(engine: State<'_, Engine>, cleanup: CleanupId) -> Result<CleanupDetail, Refusal> {
    engine.cleanup(cleanup).map_err(Refusal::from)
}

/// Puts items of a cleanup back from the Trash, never replacing anything.
#[tauri::command(async)]
pub(crate) fn put_back(engine: State<'_, Engine>, request: PutBackRequest) -> Result<PutBackReport, Refusal> {
    engine.put_back(request.cleanup(), request.items()).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn reveal_in_trash(engine: State<'_, Engine>, item: CleanupItemRef) -> Result<(), Refusal> {
    engine.reveal_in_trash(item.cleanup(), item.item()).map_err(Refusal::from)
}

/// What waits in each Trash and how much of it this app's cleanups put there: the scan screen's Trash line.
#[tauri::command(async)]
pub(crate) fn get_trash(engine: State<'_, Engine>) -> TrashView {
    engine.trash()
}

/// "Open Trash": opens a Trash of the last answer in Finder; takes its index, never a path. Never empties it.
#[tauri::command(async)]
pub(crate) fn open_trash(engine: State<'_, Engine>, index: u32) -> Result<(), Refusal> {
    engine.open_trash(index).map_err(Refusal::from)
}

/// The user chose to quit during a cleanup: it stops after the item in flight, then the app quits.
#[tauri::command(async)]
pub(crate) fn quit_after_cleanup<R: Runtime>(app: AppHandle<R>) {
    shell::quit_after_cleanup(&app);
}

#[cfg(test)]
mod tests {
    use fresher_core::CleanupsQuery;
    use serde_json::json;

    fn first_page() -> CleanupsQuery {
        serde_json::from_value(json!({ "offset": 0, "limit": 10 })).unwrap()
    }

    /// Both trial categories, picked whole: the folder to move to the Trash and the one to empty.
    #[cfg(debug_assertions)]
    fn both_trial_categories() -> fresher_core::CleanupSelection {
        let selection = json!({ "findings": [], "categories": [
            { "category": "trial.caches", "except": [] }, { "category": "trial.kept", "except": [] } ] });
        serde_json::from_value(selection).unwrap()
    }

    mod negative {
        use std::sync::Arc;

        use fresher_core::{
            Access, Capability, CommandError, DangerLevel, FindingOrder, FindingsQuery, GuardedFolder, Operation,
            Refusal, SettingKey, SettingUpdate, SettingValue,
        };
        use fresher_testing::FakeAccess;

        #[cfg(debug_assertions)]
        use fresher_core::{ByteSize, PutBackOutcome, RecordedOutcome, SkipCode};
        use fresher_core::{CleanupId, LeftOut, SkipReason};
        use serde_json::json;

        use super::first_page;
        use crate::commands::app_info;
        use crate::commands::{
            accept_agreement, cancel_cleanup, cancel_scan, confirm_cleanup, discard_cleanup, get_agreement,
            get_cleanup, get_cleanups, get_dictionary, get_findings, open_access_settings, open_trash, prepare_cleanup,
            put_back, reveal_finding, reveal_in_trash, start_scan, subscribe, update_setting,
        };
        use crate::testing::{
            Shell, caches, echo, is_scan_finished, recording_channel, scan, trial_home, until_finished,
        };
        #[cfg(debug_assertions)]
        use crate::testing::{is_cleanup_finished, listing};
        use fresher_core::{Build, BuildFlags, HighLevel, LargestScreen};
        use tauri::Manager;

        #[test]
        fn app_info_reports_the_engines_high_over_the_managed_one() {
            let shell = Shell::new();
            shell.app.manage(Build::Development);
            shell.app.manage(BuildFlags { high: HighLevel::Withheld, largest: LargestScreen::Hidden });

            let info = app_info(shell.app.state::<Build>(), shell.app.state::<BuildFlags>(), shell.engine());

            assert_eq!(info.flags, BuildFlags { high: HighLevel::Offered, largest: LargestScreen::Hidden });
        }

        #[test]
        fn start_scan_twice_is_busy() {
            let shell = Shell::held();
            let events = shell.engine().subscribe();
            start_scan(shell.engine()).unwrap();

            assert_eq!(start_scan(shell.engine()), Err(Refusal::from(CommandError::Busy { running: Operation::Scan })));

            cancel_scan(shell.engine()).unwrap();
            until_finished(&events, is_scan_finished);
        }

        #[test]
        fn cancel_scan_when_idle_is_not_running() {
            let shell = Shell::new();

            assert_eq!(cancel_scan(shell.engine()), Err(Refusal::from(CommandError::NotRunning)));
        }

        #[test]
        fn get_findings_with_an_unknown_module_is_refused() {
            let shell = Shell::new();
            let query = FindingsQuery::new(Some("gone".into()), None, FindingOrder::Found, 0, 10);

            assert_eq!(
                get_findings(shell.engine(), query),
                Err(Refusal::from(CommandError::UnknownModule { id: "gone".into() }))
            );
        }

        #[test]
        fn update_setting_with_a_wrong_type_is_refused() {
            let shell = Shell::new();
            let update = SettingUpdate { key: "demo.count".into(), value: SettingValue::Switch(true) };

            let error = update_setting(shell.engine(), update);

            assert_eq!(
                error,
                Err(Refusal::from(CommandError::InvalidSettingValue { key: SettingKey::new("demo.count").unwrap() }))
            );
        }

        #[test]
        fn ignoring_what_is_not_there_is_refused() {
            let shell = Shell::new();
            let target = fresher_core::IgnoreTarget::Category { category: "gone.caches".into() };

            let error = crate::commands::ignore(shell.engine(), target);

            assert_eq!(error, Err(Refusal::from(CommandError::UnknownCategory { id: "gone.caches".into() })));
        }

        #[test]
        fn a_second_subscribe_takes_the_events_from_the_first_channel() {
            let shell = Shell::new();
            let (first, first_received) = recording_channel();
            let (second, second_received) = recording_channel();
            subscribe(shell.engine(), shell.forwarder(), first).unwrap();
            subscribe(shell.engine(), shell.forwarder(), second).unwrap();

            start_scan(shell.engine()).unwrap();

            until_finished(&second_received, |event| event["type"] == "scan-finished");
            assert_eq!(first_received.try_iter().count(), 0);
        }

        #[test]
        fn confirm_with_a_mismatched_echo_is_refused() {
            let tree = trial_home(true);
            let shell = Shell::cleaning(tree.root());
            scan(&shell);
            let view = prepare_cleanup(shell.engine(), caches()).unwrap();
            let count = view.summary.count + 1;
            let echo = json!({ "plan": view.plan, "count": count, "allocated": view.summary.sizes.allocated,
                               "highest": view.summary.highest, "method": "trash",
                               "highRiskUnderstood": view.summary.highest == DangerLevel::High });

            let error = confirm_cleanup(shell.engine(), serde_json::from_value(echo).unwrap());

            assert_eq!(error, Err(Refusal::from(CommandError::PlanMismatch)));
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
            assert!(!shell.trash().exists());
            assert_eq!(get_cleanups(shell.engine(), first_page()).total, 0);
        }

        #[test]
        fn a_discarded_plan_cannot_be_confirmed() {
            let tree = trial_home(true);
            let shell = Shell::cleaning(tree.root());
            scan(&shell);
            let view = prepare_cleanup(shell.engine(), caches()).unwrap();
            discard_cleanup(shell.engine(), view.plan).unwrap();

            let error = confirm_cleanup(shell.engine(), echo(&view));

            assert_eq!(error, Err(Refusal::from(CommandError::PlanGone)));
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
        }

        #[test]
        fn the_sandbox_reach_cleans_nothing_outside_a_sandbox() {
            let tree = trial_home(false);
            let shell = Shell::cleaning(tree.root());
            scan(&shell);
            let selection = json!({ "findings": [], "categories": [
                { "category": "trial.caches", "except": [] }, { "category": "trial.kept", "except": [] } ] });

            let view = prepare_cleanup(shell.engine(), serde_json::from_value(selection).unwrap()).unwrap();

            let reasons: Vec<&SkipReason> = view.left_out.iter().map(LeftOut::reason).collect();
            assert_eq!(view.summary.count, 0);
            assert_eq!(reasons, [&SkipReason::NotInSandbox, &SkipReason::NotInSandbox]);
        }

        /// The development build's reach on the trial folders beside a valid marker: every item is checked and
        /// skipped as a dry run, the tree and the Trash are untouched, and nothing can be put back.
        #[cfg(debug_assertions)]
        #[test]
        fn a_dry_run_through_the_commands_moves_nothing() {
            let tree = trial_home(true);
            let shell = Shell::dry_run(tree.root());
            scan(&shell);
            let before = listing(tree.root());
            let events = shell.engine().subscribe();
            let view = prepare_cleanup(shell.engine(), super::both_trial_categories()).unwrap();

            let cleanup = confirm_cleanup(shell.engine(), echo(&view)).unwrap();

            until_finished(&events, is_cleanup_finished);
            assert_eq!(view.summary.count, 2);
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
            assert_eq!(listing(tree.root()), before);
            assert!(!shell.trash().exists());
            let detail = get_cleanup(shell.engine(), cleanup).unwrap();
            let outcomes: Vec<RecordedOutcome> = detail.items.iter().map(|item| item.outcome).collect();
            assert_eq!(outcomes, [RecordedOutcome::Skipped { reason: SkipCode::DryRun }; 2]);
            assert_eq!(
                (detail.summary.moved.allocated, detail.summary.deleted.allocated),
                (ByteSize::new(0), ByteSize::new(0))
            );
            assert_eq!(
                (detail.summary.moved.logical, detail.summary.deleted.logical),
                (ByteSize::new(0), ByteSize::new(0))
            );

            let request = json!({ "cleanup": cleanup, "items": [0, 1] });
            let back = put_back(shell.engine(), serde_json::from_value(request).unwrap()).unwrap();

            let answers: Vec<PutBackOutcome> = back.items.iter().map(|item| item.outcome).collect();
            assert_eq!(answers, [PutBackOutcome::NotTrashed; 2]);
            assert_eq!(listing(tree.root()), before);
            assert!(!shell.trash().exists());
        }

        /// The second wall alone: the sandbox reach admits the trial folders beside a valid marker, and the refusing
        /// Trash moves none of them.
        #[cfg(debug_assertions)]
        #[test]
        fn the_refusing_trash_alone_moves_nothing_in_a_marked_sandbox() {
            let tree = trial_home(true);
            let shell = Shell::refusing_trash(tree.root());
            scan(&shell);
            let before = listing(tree.root());
            let events = shell.engine().subscribe();
            let view = prepare_cleanup(shell.engine(), super::both_trial_categories()).unwrap();

            let cleanup = confirm_cleanup(shell.engine(), echo(&view)).unwrap();

            until_finished(&events, is_cleanup_finished);
            assert_eq!(view.summary.count, 2);
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
            assert_eq!(listing(tree.root()), before);
            assert!(!shell.trash().exists());
            let items = get_cleanup(shell.engine(), cleanup).unwrap().items;
            for item in &items {
                assert!(!matches!(item.outcome, RecordedOutcome::Moved | RecordedOutcome::Deleted), "{item:?}");
                if let RecordedOutcome::Emptied { moved, .. } = item.outcome {
                    assert_eq!(moved, 0, "{item:?}");
                }
            }
            let cache =
                items.iter().find(|item| item.path.as_deref() == Some(&tree.root().join("sandbox/Sample app cache")));
            assert!(matches!(cache.map(|item| item.outcome), Some(RecordedOutcome::Failed { .. })), "{items:?}");
        }

        #[test]
        fn put_back_while_scanning_is_busy() {
            let shell = Shell::held();
            let events = shell.engine().subscribe();
            start_scan(shell.engine()).unwrap();

            let error =
                put_back(shell.engine(), serde_json::from_value(json!({ "cleanup": 1, "items": null })).unwrap());

            assert_eq!(error, Err(Refusal::from(CommandError::Busy { running: Operation::Scan })));
            cancel_scan(shell.engine()).unwrap();
            until_finished(&events, is_scan_finished);
        }

        #[test]
        fn accept_agreement_of_another_version_is_refused() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_unaccepted(tree.root());

            let error = accept_agreement(shell.engine(), fresher_core::AGREEMENT_VERSION + 1).unwrap_err();

            assert_eq!(error, Refusal::from(CommandError::AgreementChanged));
            assert_eq!(error.text.key().as_str(), "core.error.agreement-changed");
            assert_eq!(get_agreement(shell.engine()).unwrap().accepted, None);
        }

        #[test]
        fn prepare_cleanup_before_the_agreement_is_refused_with_the_blocked_text() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_unaccepted(tree.root());
            scan(&shell);

            let error = prepare_cleanup(shell.engine(), caches()).unwrap_err();

            let reason = fresher_core::BlockedReason::Agreement;
            assert_eq!(error, Refusal::from(CommandError::CleaningBlocked { reason }));
            assert_eq!(reason.text().key().as_str(), "core.cleanup.blocked.agreement");
            assert!(get_dictionary(shell.engine(), "en").unwrap().get("core.cleanup.blocked.agreement").is_some());
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists());
            assert!(!shell.trash().exists());
        }

        #[test]
        fn the_cleanup_commands_refuse_what_is_not_there() {
            let shell = Shell::new();
            let id: CleanupId = serde_json::from_value(json!(7)).unwrap();
            let unknown = Refusal::from(CommandError::UnknownCleanup { id });

            assert_eq!(cancel_cleanup(shell.engine()), Err(Refusal::from(CommandError::NotRunning)));
            assert_eq!(prepare_cleanup(shell.engine(), caches()), Err(Refusal::from(CommandError::NoScan)));
            assert_eq!(get_cleanup(shell.engine(), id), Err(unknown.clone()));
            let item = json!({ "cleanup": 7, "item": 0 });
            assert_eq!(reveal_in_trash(shell.engine(), serde_json::from_value(item).unwrap()), Err(unknown));
        }

        #[test]
        fn open_trash_of_an_index_never_answered_is_refused() {
            let shell = Shell::new();

            let refused = open_trash(shell.engine(), 0);

            assert_eq!(refused, Err(Refusal::from(CommandError::UnknownTrash { index: 0 })));
        }

        #[test]
        fn reveal_finding_of_an_unknown_id_is_refused() {
            let shell = Shell::new();
            let finding = serde_json::from_value(json!(9)).unwrap();

            let refused = reveal_finding(shell.engine(), finding);

            assert_eq!(refused, Err(Refusal::from(CommandError::UnknownFinding { id: finding })));
        }

        #[test]
        fn open_access_settings_for_a_tool_is_refused() {
            // The UI cannot even name one: a tool sent from outside the app does not deserialize.
            let tool = serde_json::from_value::<Capability>(json!({ "kind": "tool", "value": "docker" }));

            assert!(tool.is_err());
        }

        #[test]
        fn open_access_settings_for_a_capability_this_mac_lacks_is_refused() {
            let shell = Shell::with_access(Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)])));
            let desktop = Capability::Folder(GuardedFolder::Desktop);

            assert_eq!(
                open_access_settings(shell.engine(), desktop),
                Err(Refusal::from(CommandError::NoSettingsPane { capability: desktop }))
            );
        }

        #[test]
        fn get_dictionary_for_an_unknown_locale_is_refused() {
            let shell = Shell::new();

            assert_eq!(
                get_dictionary(shell.engine(), "fr"),
                Err(Refusal::from(CommandError::UnknownLocale { locale: "fr".into() }))
            );
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{
            Access, Capability, EngineStatus, Event, FindingOrder, FindingsQuery, SettingUpdate, SettingValue,
            SettingsFile,
        };
        use fresher_testing::FakeAccess;

        use fresher_core::{Build, BuildFlags, PutBackOutcome, SandboxTrashAt, Simulated, TrashState};
        use serde_json::json;
        use tauri::Manager;

        use super::first_page;
        use crate::commands::{
            accept_agreement, app_info, confirm_cleanup, get_access_status, get_agreement, get_cleanup, get_cleanups,
            get_dictionary, get_findings, get_modules, get_overview, get_settings, get_trash, open_access_settings,
            prepare_cleanup, put_back, refresh_access, start_scan, subscribe, update_setting,
        };
        use crate::testing::{Shell, caches, echo, recording_channel, scan, trial_home, until_finished};

        #[test]
        fn get_agreement_answers_the_five_lines_and_no_acceptance() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_unaccepted(tree.root());

            let view = get_agreement(shell.engine()).unwrap();

            let dictionary = get_dictionary(shell.engine(), "en").unwrap();
            assert_eq!(view.version, fresher_core::AGREEMENT_VERSION);
            assert_eq!(view.lines.len(), 5);
            assert!(
                view.lines
                    .iter()
                    .chain([&view.title, &view.intro])
                    .all(|text| dictionary.get(text.key().as_str()).is_some())
            );
            assert!(!view.is_accepted());
            assert_eq!((view.accepted, view.changes), (None, None));
        }

        #[test]
        fn accept_agreement_then_prepare_and_confirm_cleans_a_sandbox_through_the_commands() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_unaccepted(tree.root());
            scan(&shell);

            let accepted = accept_agreement(shell.engine(), fresher_core::AGREEMENT_VERSION).unwrap();
            let view = prepare_cleanup(shell.engine(), caches()).unwrap();
            let events = shell.engine().subscribe();
            let cleanup = confirm_cleanup(shell.engine(), echo(&view)).unwrap();

            until_finished(&events, crate::testing::is_cleanup_finished);
            assert!(accepted.is_accepted());
            assert_eq!(get_agreement(shell.engine()), Ok(accepted));
            assert!(!tree.root().join("sandbox/Sample app cache").exists());
            assert_eq!(std::fs::read_dir(shell.trash()).unwrap().count(), 1);
            assert_eq!(get_cleanups(shell.engine(), first_page()).cleanups[0].cleanup, cleanup);
        }

        #[test]
        fn get_trash_on_a_fresh_engine_answers_no_line() {
            let shell = Shell::new();

            assert_eq!(get_trash(shell.engine()).trashes.len(), 0);
        }

        #[test]
        fn prepare_then_confirm_cleans_a_sandbox_through_the_commands() {
            let tree = trial_home(true);
            let shell = Shell::cleaning(tree.root());
            let (channel, received) = recording_channel();
            subscribe(shell.engine(), shell.forwarder(), channel).unwrap();
            scan(&shell);
            let view = prepare_cleanup(shell.engine(), caches()).unwrap();

            let cleanup = confirm_cleanup(shell.engine(), echo(&view)).unwrap();

            let events = until_finished(&received, |event| event["type"] == "cleanup-finished");
            let types: Vec<&str> = events.iter().filter_map(|event| event["type"].as_str()).collect();
            assert!(types.ends_with(&["cleanup-started", "cleanup-items", "findings-removed", "cleanup-finished"]));
            assert_eq!(events.last().map(|event| event["report"]["counts"]["moved"].as_u64()), Some(Some(1)));
            assert_eq!((view.summary.count, view.left_out.len()), (1, 0));
            assert!(!tree.root().join("sandbox/Sample app cache").exists());
            assert!(tree.root().join("sandbox/keep-me.txt").exists());
            assert!(tree.root().join("sandbox/Old sample downloads/entry.bin").exists());
            assert_eq!(std::fs::read_dir(shell.trash()).unwrap().count(), 1);
            let page = get_cleanups(shell.engine(), first_page());
            assert_eq!((page.total, page.cleanups[0].cleanup, page.cleanups[0].in_trash), (1, cleanup, 1));
            let detail = get_cleanup(shell.engine(), cleanup).unwrap();
            assert_eq!(detail.items[0].trash, Some(TrashState::InTrash));

            let back =
                put_back(shell.engine(), serde_json::from_value(json!({ "cleanup": cleanup, "items": null })).unwrap());

            assert_eq!(back.unwrap().items[0].outcome, PutBackOutcome::PutBack);
            assert!(tree.root().join("sandbox/Sample app cache/nested/blob.bin").exists());
            assert_eq!(get_cleanups(shell.engine(), first_page()).cleanups[0].in_trash, 0);
        }

        /// What the sandbox reach leaves out with no valid marker
        /// (`the_sandbox_reach_cleans_nothing_outside_a_sandbox`), the dry run admits: it checks the real disk.
        #[cfg(debug_assertions)]
        #[test]
        fn a_dry_run_plan_holds_what_the_sandbox_reach_leaves_out() {
            let tree = trial_home(false);
            let shell = Shell::dry_run(tree.root());
            scan(&shell);

            let view = prepare_cleanup(shell.engine(), super::both_trial_categories()).unwrap();

            assert_eq!((view.summary.count, view.left_out.len()), (2, 0));
        }

        #[test]
        fn app_info_is_the_crate_version_and_the_managed_build() {
            let shell = Shell::new();
            let build = Build::Sandbox {
                path: "/tmp/box".into(),
                simulated: vec![Simulated::Running],
                trash: SandboxTrashAt::System.text(),
            };
            shell.app.manage(build.clone());
            shell.app.manage(BuildFlags {
                high: fresher_core::HighLevel::Offered,
                largest: fresher_core::LargestScreen::Shown,
            });

            let info = app_info(shell.app.state::<Build>(), shell.app.state::<BuildFlags>(), shell.engine());

            assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
            assert_eq!(info.build, build);
            assert_eq!(
                info.flags,
                BuildFlags { high: fresher_core::HighLevel::Offered, largest: fresher_core::LargestScreen::Shown }
            );
            assert_eq!(
                info.now,
                fresher_core::Timestamp::from_secs(1_700_000_000),
                "the engine's clock, not the Mac's"
            );
        }

        #[test]
        fn app_info_reports_the_home_the_engine_was_given() {
            let tree = trial_home(false);
            let shell = Shell::cleaning(tree.root());
            shell.app.manage(Build::Development);
            shell.app.manage(BuildFlags {
                high: fresher_core::HighLevel::Offered,
                largest: fresher_core::LargestScreen::Shown,
            });

            let info = app_info(shell.app.state::<Build>(), shell.app.state::<BuildFlags>(), shell.engine());

            assert_eq!(info.home.as_deref(), Some(tree.root()));
        }

        #[test]
        fn subscribe_forwards_a_scan_to_the_channel() {
            let shell = Shell::new();
            let (channel, received) = recording_channel();
            subscribe(shell.engine(), shell.forwarder(), channel).unwrap();

            let scan = start_scan(shell.engine()).unwrap();

            let events = until_finished(&received, |event| event["type"] == "scan-finished");
            let types: Vec<&str> = events.iter().map(|event| event["type"].as_str().unwrap()).collect();
            assert_eq!(types.first(), Some(&"scan-started"));
            assert!(types.contains(&"findings-added"), "{types:?}");
            assert_eq!(events.last().map(|event| event["scan"].as_u64()), Some(Some(scan.get().into())));
            assert_eq!(get_overview(shell.engine()).unwrap().status, EngineStatus::Idle);
            let page = get_findings(
                shell.engine(),
                FindingsQuery::new(Some("demo".into()), None, FindingOrder::Found, 0, 500),
            )
            .unwrap();
            assert!(page.total > 0);
        }

        #[test]
        fn repeated_subscribes_keep_one_thread_and_feed_the_last_channel() {
            let shell = Shell::new();
            let (first, _) = recording_channel();
            subscribe(shell.engine(), shell.forwarder(), first).unwrap();
            let thread = shell.forwarder().thread_id();
            for _ in 0..50 {
                let (reloaded, _) = recording_channel();
                subscribe(shell.engine(), shell.forwarder(), reloaded).unwrap();
            }
            let (last, received) = recording_channel();
            subscribe(shell.engine(), shell.forwarder(), last).unwrap();

            let scan = start_scan(shell.engine()).unwrap();

            let events = until_finished(&received, |event| event["type"] == "scan-finished");
            assert!(thread.is_some());
            assert_eq!(shell.forwarder().thread_id(), thread);
            assert_eq!(events.first().map(|event| event["scan"].as_u64()), Some(Some(scan.get().into())));
        }

        #[test]
        fn a_setting_is_saved_and_shown() {
            let shell = Shell::new();

            update_setting(shell.engine(), SettingUpdate { key: "demo.count".into(), value: SettingValue::Number(3) })
                .unwrap();

            let view = get_settings(shell.engine()).unwrap();
            assert_eq!(view.file, SettingsFile::Loaded);
            assert!(view.values.values().any(|value| *value == SettingValue::Number(3)));
        }

        #[test]
        fn settings_are_reset_and_a_category_ignored_and_restored() {
            let shell = Shell::new();
            update_setting(shell.engine(), SettingUpdate { key: "demo.count".into(), value: SettingValue::Number(3) })
                .unwrap();
            let category = || "demo.caches".to_owned();

            crate::commands::reset_settings(shell.engine()).unwrap();
            crate::commands::ignore(shell.engine(), fresher_core::IgnoreTarget::Category { category: category() })
                .unwrap();
            let ignored = get_overview(shell.engine()).unwrap().ignored;
            crate::commands::restore_ignored(
                shell.engine(),
                fresher_core::IgnoredEntry::Category { category: category() },
            )
            .unwrap();

            assert!(
                !get_settings(shell.engine()).unwrap().values.values().any(|value| *value == SettingValue::Number(3))
            );
            assert_eq!(ignored, fresher_core::IgnoredCount { categories: 1, ..fresher_core::IgnoredCount::default() });
            assert_eq!(get_overview(shell.engine()).unwrap().ignored, fresher_core::IgnoredCount::default());
        }

        #[test]
        fn the_dictionary_has_core_and_demo_keys() {
            let shell = Shell::new();

            let dictionary = get_dictionary(shell.engine(), "en").unwrap();

            assert!(dictionary.get("core.error.busy").is_some());
            assert!(dictionary.get("demo.caches.name").is_some());
        }

        #[test]
        fn answers_the_modules_and_the_access_status() {
            let shell = Shell::new();

            let modules: Vec<_> = get_modules(shell.engine()).into_iter().map(|module| module.id.as_str()).collect();

            assert_eq!(
                modules,
                [
                    "app-caches",
                    "temp",
                    "logs",
                    "dev-caches",
                    "app-leftovers",
                    "downloads",
                    "jetbrains",
                    "stale-projects",
                    "local-ai-models",
                    "unused-toolchains",
                    "xcode",
                    "docker",
                    "demo",
                    "trial"
                ]
            );
            assert_eq!(get_access_status(shell.engine()).access(&Capability::AllFiles), Access::Unknown);
        }

        #[test]
        fn refresh_access_answers_the_new_status() {
            let access = Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]));
            let shell = Shell::with_access(Arc::<FakeAccess>::clone(&access));
            let events = shell.engine().subscribe();

            access.set(Capability::AllFiles, Access::Granted);
            let status = refresh_access(shell.engine());

            assert_eq!(status.access(&Capability::AllFiles), Access::Granted);
            assert_eq!(events.try_iter().collect::<Vec<_>>(), [Event::AccessChanged { status }]);
        }

        #[test]
        fn open_access_settings_for_a_capability_this_mac_has_is_accepted() {
            let shell = Shell::with_access(Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)])));

            assert_eq!(open_access_settings(shell.engine(), Capability::AllFiles), Ok(()));
        }
    }
}
