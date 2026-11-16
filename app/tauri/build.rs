//! The app manifest allowlists exactly the app's own commands; `capabilities/default.json` grants each one.

/// Every command in `tauri::generate_handler!` (`src/lib.rs`), in the same order.
const COMMANDS: &[&str] = &[
    "app_version",
    "subscribe",
    "start_scan",
    "cancel_scan",
    "get_overview",
    "get_modules",
    "get_findings",
    "get_settings",
    "update_setting",
    "reset_settings",
    "ignore",
    "restore_ignored",
    "pick_folder",
    "get_own_data",
    "get_dictionary",
    "get_access_status",
    "prepare_cleanup",
    "discard_cleanup",
    "confirm_cleanup",
    "cancel_cleanup",
    "get_cleanups",
    "get_cleanup",
    "put_back",
    "reveal_in_trash",
    "quit_after_cleanup",
];

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
