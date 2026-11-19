//! Which commands the record keeps, and the hook every call from the UI passes.

use tauri::ipc::InvokeMessage;
use tauri::{Manager, Runtime};

use super::DevRuns;

/// The commands that change something or reach outside the app: each call is recorded with its arguments.
pub(crate) const ACTIONS: [&str; 18] = [
    "start_scan",
    "cancel_scan",
    "update_setting",
    "reset_settings",
    "ignore",
    "restore_ignored",
    "refresh_access",
    "open_access_settings",
    "reveal_finding",
    "relaunch",
    "prepare_cleanup",
    "discard_cleanup",
    "confirm_cleanup",
    "cancel_cleanup",
    "put_back",
    "reveal_in_trash",
    "quit_after_cleanup",
    "pick_folder",
];

/// The commands that only read: never recorded. A test checks every command is in one of the two lists.
#[cfg_attr(not(test), expect(dead_code, reason = "only the classification test reads it"))]
pub(crate) const READS: [&str; 14] = [
    "app_info",
    "subscribe",
    "get_overview",
    "get_modules",
    "get_findings",
    "get_settings",
    "get_own_data",
    "get_storage_map",
    "get_storage_meaning",
    "get_storage_place",
    "get_dictionary",
    "get_access_status",
    "get_cleanups",
    "get_cleanup",
];

pub(crate) fn is_action(command: &str) -> bool {
    ACTIONS.contains(&command)
}

/// Records `message` when the app records a run and the command is an action; before the command runs.
pub(crate) fn observe<R: Runtime>(message: &InvokeMessage<R>) {
    if let Some(runs) = message.webview_ref().try_state::<DevRuns>() {
        runs.call(message.command(), message.payload());
    }
}
