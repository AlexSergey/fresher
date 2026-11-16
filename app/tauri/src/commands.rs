//! The commands the UI calls: each one passes its input to the engine and returns the engine's answer. They run off
//! the main thread (`async`), so a command that writes the settings file never blocks the window. A refused command
//! returns a `Refusal`: the engine's error with the text the UI shows for it.
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use fresher_core::{
    AccessStatus, Dictionary, Event, FindingsPage, FindingsQuery, Locale, ModuleInfo, Overview, Refusal, ScanId,
    SettingUpdate, SettingsView,
};
use fresher_engine::Engine;
use tauri::State;
use tauri::ipc::Channel;

use crate::events::EventForwarder;

/// The version of the app.
#[tauri::command]
pub(crate) fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
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
    engine.start_scan().map_err(Refusal::from)
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

/// The merged texts of the core and every module; only `en` ships.
#[tauri::command(async)]
pub(crate) fn get_dictionary(engine: State<'_, Engine>, locale: &str) -> Result<Dictionary, Refusal> {
    Locale::parse(locale).map(|_| engine.dictionary().clone()).map_err(Refusal::from)
}

#[tauri::command(async)]
pub(crate) fn get_access_status(engine: State<'_, Engine>) -> AccessStatus {
    engine.access_status()
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{
            CommandError, FindingOrder, FindingsQuery, Operation, Refusal, SettingKey, SettingUpdate, SettingValue,
        };

        use crate::commands::{cancel_scan, get_dictionary, get_findings, start_scan, subscribe, update_setting};
        use crate::testing::{Shell, is_scan_finished, recording_channel, until_finished};

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
        fn get_dictionary_for_an_unknown_locale_is_refused() {
            let shell = Shell::new();

            assert_eq!(
                get_dictionary(shell.engine(), "fr"),
                Err(Refusal::from(CommandError::UnknownLocale { locale: "fr".into() }))
            );
        }
    }

    mod positive {
        use fresher_core::{
            Access, EngineStatus, FindingOrder, FindingsQuery, SettingUpdate, SettingValue, SettingsFile,
        };

        use crate::commands::{
            app_version, get_access_status, get_dictionary, get_findings, get_modules, get_overview, get_settings,
            start_scan, subscribe, update_setting,
        };
        use crate::testing::{Shell, recording_channel, until_finished};

        #[test]
        fn app_version_is_the_crate_version() {
            assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
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

            assert_eq!(modules, ["demo"]);
            assert_eq!(get_access_status(shell.engine()).full_disk_access, Access::Unknown);
        }
    }
}
