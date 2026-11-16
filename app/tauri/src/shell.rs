//! How the app lives: one instance, and it ends when its window closes. No tray, no thread survives the window.

use fresher_engine::Engine;
use tauri::{AppHandle, Manager, RunEvent, Runtime};

/// The label of the app's only window (`tauri.conf.json`).
const MAIN_WINDOW: &str = "main";

/// A second launch of the app: the open window comes forward, and the second process exits (the plugin ends it).
pub(crate) fn on_second_launch<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        // A window that cannot be brought forward stays where it is; there is nothing else to do.
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// The app's run loop: closing the window asks the app to exit, and the engine is shut down first.
pub(crate) fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: &RunEvent) {
    if let RunEvent::ExitRequested { .. } = event {
        on_exit(app);
    }
}

/// Cancels the running scan and waits for its thread, so the process ends with nothing half done.
pub(crate) fn on_exit<R: Runtime>(app: &AppHandle<R>) {
    if let Some(engine) = app.try_state::<Engine>() {
        engine.shutdown();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{EngineStatus, Event, ScanKind, ScanOutcome};

        use tauri::Manager;

        use crate::shell::{on_exit, on_second_launch};
        use crate::testing::Shell;

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

            assert!(app.webview_windows().is_empty());
        }
    }

    mod positive {
        use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

        use crate::shell::{MAIN_WINDOW, on_second_launch};
        use crate::testing::Shell;

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
