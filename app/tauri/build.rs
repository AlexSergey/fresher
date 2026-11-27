//! The app manifest allowlists exactly the app's own commands; `capabilities/default.json` grants each one.
//! It also makes the build's flags (`flags.toml`, `FRESHER_FLAGS`) into `FLAGS` (`src/flags.rs`).

#[path = "build/flags.rs"]
mod flags;

use std::path::PathBuf;

/// Every command in `tauri::generate_handler!` (`src/lib.rs`), in the same order.
/// The last two, `demo_play` and `demo_finished`, are handled in debug builds only (`src/demo.rs`): a release build
/// has no such command, so their grants reach nothing.
const COMMANDS: &[&str] = &[
    "app_info",
    "subscribe",
    "start_scan",
    "cancel_scan",
    "get_overview",
    "get_modules",
    "get_findings",
    "get_settings",
    "update_setting",
    "reset_settings",
    "get_agreement",
    "accept_agreement",
    "ignore",
    "restore_ignored",
    "pick_folder",
    "get_own_data",
    "get_storage_map",
    "get_storage_meaning",
    "get_largest",
    "get_largest_place",
    "pick_own_item",
    "get_largest_folders",
    "reveal_node",
    "get_apps",
    "get_app",
    "reveal_app",
    "prepare_uninstall",
    "get_dictionary",
    "get_access_status",
    "refresh_access",
    "open_access_settings",
    "reveal_finding",
    "relaunch",
    "prepare_cleanup",
    "discard_cleanup",
    "confirm_cleanup",
    "cancel_cleanup",
    "get_cleanups",
    "get_cleanup",
    "put_back",
    "reveal_in_trash",
    "get_trash",
    "open_trash",
    "quit_after_cleanup",
    "demo_play",
    "demo_finished",
];

/// A refusal printed as its message: `main`'s error is shown with `Debug`, which would quote a plain string.
struct Refusal(String);

impl std::fmt::Debug for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refusal {}

/// Writes `$OUT_DIR/flags.rs` from `flags.toml` and `FRESHER_FLAGS`.
fn write_flags() -> Result<(), String> {
    let path = "../../flags.toml";
    let file = std::fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let profile = if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        flags::Profile::Development
    } else {
        flags::Profile::Release
    };
    let overrides = match std::env::var("FRESHER_FLAGS") {
        Ok(text) => Some(text),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => return Err("FRESHER_FLAGS is not valid UTF-8".to_owned()),
    };
    let resolved = flags::resolve(&file, profile, overrides.as_deref()).map_err(|error| error.to_string())?;
    let out_dir = std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?;
    let target = PathBuf::from(out_dir).join("flags.rs");
    std::fs::write(&target, flags::render(&resolved))
        .map_err(|error| format!("cannot write {}: {error}", target.display()))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("cargo:rerun-if-changed=../../flags.toml");
    println!("cargo:rerun-if-changed=build/flags.rs");
    println!("cargo:rerun-if-env-changed=FRESHER_FLAGS");
    write_flags().map_err(Refusal)?;

    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
