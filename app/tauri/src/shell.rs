//! How the app lives: one instance, and it ends when its window closes. No tray, no thread survives the window.
//! While a cleanup runs, closing the window or quitting from the app menu (⌘Q) asks first. When the window gains
//! focus the system's permissions are checked again, off the event loop's thread.

use crate::start_failure::holds_exit;
use fresher_core::{CommandError, EngineStatus, Operation};
use fresher_engine::Engine;
use tauri::async_runtime::{self, JoinHandle};
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::{AppHandle, Manager, RunEvent, Runtime, WindowEvent};

/// The label of the app's only window (`tauri.conf.json`).
pub(crate) const MAIN_WINDOW: &str = "main";

/// The id of the app menu's Quit, the app's own item in place of the system's.
const QUIT_ITEM: &str = "fresher-quit";

/// The default menu with its Quit replaced by the app's own item (same title, same ⌘Q), so quitting from the menu
/// during a cleanup asks first, as closing the window does. The system's Quit cannot be held: macOS ends the app from
/// it. The Dock's Quit stays the system's (`docs/known-issues/app-quit-during-cleanup.md`).
pub(crate) fn menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::default(app)?;
    // On macOS the first submenu is the app's, and its last item the system's Quit.
    if cfg!(target_os = "macos")
        && let Some(app_menu) = menu.items()?.first().and_then(|item| item.as_submenu().cloned())
    {
        let last = app_menu.items()?.len().saturating_sub(1);
        app_menu.remove_at(last)?;
        let title = format!("Quit {}", app.package_info().name);
        app_menu.append(&MenuItem::with_id(app, QUIT_ITEM, title, true, Some("CmdOrCtrl+Q"))?)?;
    }
    Ok(menu)
}

/// The app's Quit: held while a cleanup runs (the UI asks, as for the window's close); otherwise the app exits.
pub(crate) fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    if event.id() == QUIT_ITEM && !holds_close(app) {
        app.exit(0);
    }
}

/// A second launch of the app: the open window comes forward, and the second process exits (the plugin ends it).
pub(crate) fn on_second_launch<R: Runtime>(app: &AppHandle<R>) {
    if cfg!(debug_assertions) {
        // `npm run dev` and `npm run dev:sandbox` share the development identifier: the running window keeps its mode.
        tracing::warn!("a second launch went to the running development window, which keeps its own sandbox mode");
    }
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        // A window that cannot be brought forward stays where it is; there is nothing else to do.
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// The app's run loop: closing the window is held while a cleanup runs; otherwise it asks the app to exit, and the
/// engine is shut down first. The Dock's Quit cannot be held: the engine is shut down as the app ends.
pub(crate) fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: &RunEvent) {
    match event {
        RunEvent::WindowEvent { event: WindowEvent::CloseRequested { api, .. }, .. } => {
            if holds_close(app) {
                api.prevent_close();
            }
        }
        RunEvent::WindowEvent { label, event: WindowEvent::Focused(true), .. } => {
            // The check runs on its own; nothing waits for it.
            let _ = on_focus(app, label);
        }
        // A failed start leaves no window, but the app lives until its message is dismissed.
        RunEvent::ExitRequested { api, code, .. } if holds_exit(app, *code) => api.prevent_exit(),
        RunEvent::ExitRequested { .. } | RunEvent::Exit => on_exit(app),
        _ => {}
    }
}

/// The window `label` gained focus: for the main window, the system's permissions are probed again on a blocking
/// thread (the engine publishes `access-changed` when something changed), never on the event loop's. Nothing is
/// checked for another window or without an engine (a failed start). Answers the check's task, if one started.
pub(crate) fn on_focus<R: Runtime>(app: &AppHandle<R>, label: &str) -> Option<JoinHandle<bool>> {
    if label != MAIN_WINDOW || app.try_state::<Engine>().is_none() {
        return None;
    }
    let app = app.clone();
    Some(async_runtime::spawn_blocking(move || app.try_state::<Engine>().is_some_and(|engine| engine.refresh_access())))
}

/// Relaunches the app, as a Full Disk Access grant needs: refused while a cleanup runs (it is never cut short this
/// way); otherwise the engine is shut down as on exit, then the app restarts. A running cleanup is told from the
/// engine's status, not by `holds_close`, which would also ask the user to quit.
pub(crate) fn relaunch<R: Runtime>(app: &AppHandle<R>) -> Result<(), CommandError> {
    if let Some(engine) = app.try_state::<Engine>()
        && matches!(engine.status()?, EngineStatus::Cleaning { .. })
    {
        return Err(CommandError::Busy { running: Operation::Clean });
    }
    on_exit(app);
    app.restart()
}

/// Whether the window's close is held: while a cleanup runs, the engine publishes `quit-requested` and the UI asks
/// the user; the answer is `quit_after_cleanup`, or nothing.
pub(crate) fn holds_close<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Engine>().is_some_and(|engine| engine.request_quit())
}

/// The user chose to quit during a cleanup: it stops after the item in flight and its journal is closed; then the
/// window is destroyed, which does not ask again, and the app ends as when its last window closes.
pub(crate) fn quit_after_cleanup<R: Runtime>(app: &AppHandle<R>) {
    if let Some(engine) = app.try_state::<Engine>() {
        engine.quit_after_cleanup();
    }
    let destroyed = app.get_webview_window(MAIN_WINDOW).map(|window| window.destroy());
    if !matches!(destroyed, Some(Ok(()))) {
        app.exit(0);
    }
}

/// Cancels the running operation (a scan, or a cleanup after the item in flight) and waits for its thread, so the
/// process ends with nothing half done.
pub(crate) fn on_exit<R: Runtime>(app: &AppHandle<R>) {
    if let Some(engine) = app.try_state::<Engine>() {
        engine.shutdown();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::sync::Arc;

        use fresher_core::{Access, Capability, CommandError, EngineStatus, Event, Operation, ScanKind, ScanOutcome};
        use fresher_testing::FakeAccess;
        use tauri::async_runtime::block_on;

        use tauri::Manager;

        use tauri::menu::{MenuEvent, MenuId};

        use crate::shell::{MAIN_WINDOW, holds_close, on_exit, on_focus, on_menu_event, on_second_launch, relaunch};
        use crate::testing::{Shell, caches, echo, is_cleanup_finished, scan, trial_home, until_finished};

        #[test]
        fn exit_requested_shuts_the_engine_down() {
            let shell = Shell::held();
            let events = shell.engine().subscribe();
            let scan = shell.engine().start_scan(fresher_engine::ScanMode::Auto).unwrap();

            on_exit(shell.app.handle());

            assert_eq!(shell.engine().status(), Ok(EngineStatus::Idle));
            assert_eq!(
                events.try_iter().last(),
                Some(Event::ScanFinished { scan, outcome: ScanOutcome::Partial, kind: ScanKind::Full })
            );
        }

        #[test]
        fn exit_and_a_second_launch_without_an_engine_or_a_window_do_nothing() {
            let app = tauri::test::mock_app();

            on_exit(app.handle());
            on_second_launch(app.handle());

            assert!(on_focus(app.handle(), MAIN_WINDOW).is_none());
            assert!(!holds_close(app.handle()));
            assert!(app.webview_windows().is_empty());
        }

        #[test]
        fn another_menu_item_asks_nothing() {
            let shell = Shell::new();
            let events = shell.engine().subscribe();

            on_menu_event(shell.app.handle(), &MenuEvent { id: MenuId::new("about") });

            assert_eq!(events.try_iter().count(), 0);
        }

        #[test]
        fn a_focus_without_a_change_sends_no_event() {
            let shell = Shell::with_access(Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)])));
            let events = shell.engine().subscribe();

            let changed = block_on(on_focus(shell.app.handle(), MAIN_WINDOW).unwrap()).unwrap();

            assert!(!changed);
            assert_eq!(events.try_iter().count(), 0);
        }

        #[test]
        fn a_focus_of_another_window_checks_nothing() {
            let access = Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]));
            let shell = Shell::with_access(Arc::<FakeAccess>::clone(&access));
            let events = shell.engine().subscribe();
            access.set(Capability::AllFiles, Access::Granted);

            assert!(on_focus(shell.app.handle(), "settings").is_none());
            assert_eq!(events.try_iter().count(), 0);
            assert_eq!(shell.engine().access_status().access(&Capability::AllFiles), Access::Denied);
        }

        #[test]
        fn relaunch_during_a_cleanup_is_refused() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_held(tree.root());
            scan(&shell);
            let view = shell.engine().prepare_cleanup(&caches()).unwrap();
            let events = shell.engine().subscribe();
            shell.engine().confirm_cleanup(&echo(&view)).unwrap();

            let refused = relaunch(shell.app.handle());
            shell.engine().cancel_cleanup().unwrap();

            assert_eq!(refused, Err(CommandError::Busy { running: Operation::Clean }));
            let events = until_finished(&events, is_cleanup_finished);
            assert!(!events.contains(&Event::QuitRequested), "a relaunch does not ask to quit");
        }

        #[test]
        fn close_while_idle_is_not_held() {
            let shell = Shell::new();
            let events = shell.engine().subscribe();

            assert!(!holds_close(shell.app.handle()));
            assert_eq!(events.try_iter().count(), 0);
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{Access, Capability, Ended, EngineStatus, Event};
        use fresher_testing::FakeAccess;
        use tauri::async_runtime::block_on;
        use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

        use tauri::menu::{MenuEvent, MenuId};

        use crate::shell::{
            MAIN_WINDOW, QUIT_ITEM, holds_close, on_focus, on_menu_event, on_second_launch, quit_after_cleanup,
        };
        use crate::testing::{Shell, caches, echo, is_cleanup_finished, scan, trial_home, until_finished};

        #[test]
        fn close_while_cleaning_is_held_and_quit_after_cleanup_quits() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_held(tree.root());
            WebviewWindowBuilder::new(&shell.app, MAIN_WINDOW, WebviewUrl::default()).build().unwrap();
            scan(&shell);
            let view = shell.engine().prepare_cleanup(&caches()).unwrap();
            let events = shell.engine().subscribe();
            shell.engine().confirm_cleanup(&echo(&view)).unwrap();

            assert!(holds_close(shell.app.handle()));
            assert!(holds_close(shell.app.handle()), "a second close asks again");
            quit_after_cleanup(shell.app.handle());

            let events = until_finished(&events, is_cleanup_finished);
            let asked = events.iter().filter(|event| **event == Event::QuitRequested).count();
            assert_eq!(asked, 2);
            assert!(
                matches!(events.last(), Some(Event::CleanupFinished { report, .. }) if report.ended == Ended::Cancelled)
            );
            assert_eq!(shell.engine().status(), Ok(EngineStatus::Idle));
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists(), "the item was not reached");
            // The window is destroyed; the mock runtime does not report it, so this shows the close is let through.
            assert!(!holds_close(shell.app.handle()));
        }

        #[test]
        fn quit_from_the_menu_while_cleaning_is_held_and_asks() {
            let tree = trial_home(true);
            let shell = Shell::cleaning_held(tree.root());
            WebviewWindowBuilder::new(&shell.app, MAIN_WINDOW, WebviewUrl::default()).build().unwrap();
            scan(&shell);
            let view = shell.engine().prepare_cleanup(&caches()).unwrap();
            let events = shell.engine().subscribe();
            shell.engine().confirm_cleanup(&echo(&view)).unwrap();

            on_menu_event(shell.app.handle(), &MenuEvent { id: MenuId::new(QUIT_ITEM) });
            quit_after_cleanup(shell.app.handle());

            let events = until_finished(&events, is_cleanup_finished);
            assert_eq!(events.iter().filter(|event| **event == Event::QuitRequested).count(), 1);
            assert!(tree.root().join("sandbox/Sample app cache/blob.bin").exists(), "the item was not reached");
        }

        #[test]
        fn a_focus_after_a_grant_sends_access_changed() {
            let access = Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]));
            let shell = Shell::with_access(Arc::<FakeAccess>::clone(&access));
            let events = shell.engine().subscribe();

            access.set(Capability::AllFiles, Access::Granted);
            let changed = block_on(on_focus(shell.app.handle(), MAIN_WINDOW).unwrap()).unwrap();

            assert!(changed);
            let status = shell.engine().access_status();
            assert_eq!(status.access(&Capability::AllFiles), Access::Granted);
            assert_eq!(events.try_iter().collect::<Vec<_>>(), [Event::AccessChanged { status }]);
        }

        #[test]
        fn a_second_launch_reaches_the_main_window() {
            // The mock runtime does not track focus or visibility; this proves the window is found and every call
            // on it is accepted. The real focus is checked by hand (the plan's conclusions).
            let shell = Shell::new();
            WebviewWindowBuilder::new(&shell.app, MAIN_WINDOW, WebviewUrl::default()).build().unwrap();

            on_second_launch(shell.app.handle());

            assert!(shell.app.get_webview_window(MAIN_WINDOW).is_some());
        }
    }
}
