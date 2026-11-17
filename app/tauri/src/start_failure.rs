//! The text of a failed start: what the native window says when the app cannot come up.

use std::fmt::Display;

use fresher_core::core_dictionary;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::shell::MAIN_WINDOW;

/// The exit code of an app that could not start.
const EXIT_CODE: i32 = 1;

const START_FAILED_KEY: &str = "core.start.failed";

/// Used only if the core's dictionary cannot be read, a bug the core's tests catch.
const FALLBACK: &str = "Fresher could not start. It changed nothing on your Mac. Quit it and open it again; if this \
                        happens again, reinstall Fresher.";

/// The sentence of `core.start.failed`; a development build adds the error under it, a release build never does.
pub(crate) fn start_failure_message(error: &dyn Display, debug: bool) -> String {
    let sentence = core_dictionary()
        .ok()
        .and_then(|texts| texts.get(START_FAILED_KEY).map(str::to_owned))
        .unwrap_or_else(|| FALLBACK.to_owned());
    if debug { format!("{sentence}\n\n{error}") } else { sentence }
}

/// Managed while a start failure is being shown: no command has its state, so the window is gone and the app
/// stays alive only for the dialog.
struct StartFailed;

/// Whether the app must stay up although its last window is gone: the exit has no code (the window closing), while
/// the app's own exit after the dialog carries one.
pub(crate) fn holds_exit<R: Runtime>(app: &AppHandle<R>, code: Option<i32>) -> bool {
    code.is_none() && app.try_state::<StartFailed>().is_some()
}

/// Logs the error, destroys the half-started window (no web view, so no command runs without its state), shows the
/// native message and ends the app once it is dismissed. The dialog is not blocking: it must not wait on the main
/// thread, where setup runs.
pub(crate) fn report<R: Runtime>(app: &AppHandle<R>, error: &dyn Display) {
    tracing::error!(%error, "the app cannot start");
    app.manage(StartFailed);
    if let Some(window) = app.get_webview_window(MAIN_WINDOW)
        && let Err(error) = window.destroy()
    {
        tracing::warn!(%error, "the main window could not be destroyed");
    }
    let handle = app.clone();
    app.dialog()
        .message(start_failure_message(error, cfg!(debug_assertions)))
        .kind(MessageDialogKind::Error)
        .show(move |_| handle.exit(EXIT_CODE));
}

#[cfg(test)]
mod tests {
    use super::{FALLBACK, start_failure_message};

    mod negative {
        use super::{FALLBACK, start_failure_message};

        #[test]
        fn a_release_message_holds_no_error_text() {
            let message = start_failure_message(&"the data folder is read-only", false);

            assert!(!message.contains("read-only"));
            assert!(!message.contains('\n'));
        }

        #[test]
        fn a_message_never_names_a_placeholder() {
            for debug in [false, true] {
                assert!(!start_failure_message(&"boom", debug).contains('{'));
            }
            assert!(!FALLBACK.contains('{'));
        }
    }

    mod positive {
        use tauri::Manager;

        use super::start_failure_message;
        use crate::start_failure::{StartFailed, holds_exit};

        #[test]
        fn the_app_stays_up_for_the_dialog_when_its_window_is_gone() {
            let app = tauri::test::mock_app();
            app.manage(StartFailed);

            assert!(holds_exit(app.handle(), None));
            assert!(!holds_exit(app.handle(), Some(1)));
        }

        #[test]
        fn a_debug_message_shows_the_error_under_the_sentence() {
            let message = start_failure_message(&"the data folder is read-only", true);

            let (sentence, error) = message.split_once("\n\n").unwrap();
            assert_eq!(sentence, start_failure_message(&"", false));
            assert_eq!(error, "the data folder is read-only");
        }

        #[test]
        fn the_release_message_is_the_core_sentence() {
            let texts = fresher_core::core_dictionary().unwrap();

            assert_eq!(start_failure_message(&"x", false), texts.get("core.start.failed").unwrap());
        }
    }
}
