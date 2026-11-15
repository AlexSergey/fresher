//! The Tauri 2 shell of Mac Fresher: thin commands over the core crates.

/// Returns the version of the app; the example of a typed IPC command.
#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Builds and runs the app until its last window is closed.
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default().invoke_handler(tauri::generate_handler![app_version]).run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    mod positive {
        #[test]
        fn app_version_is_the_crate_version() {
            assert_eq!(super::super::app_version(), env!("CARGO_PKG_VERSION"));
        }
    }
}
