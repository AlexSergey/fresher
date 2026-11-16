//! The Tauri 2 shell of Fresher: the composition root and thin commands over the engine.

#![forbid(unsafe_code)]

mod adapters;
mod commands;
mod compose;
mod events;
mod process;
mod shell;
#[cfg(test)]
mod testing;

use tauri::Manager;

/// Builds and runs the app until its window is closed; a second launch brings that window forward instead.
pub fn run() -> Result<(), tauri::Error> {
    let app = tauri::Builder::default()
        // First, so a second launch exits before it sets anything up.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| shell::on_second_launch(app)))
        .setup(|app| {
            let engine = compose::engine(app.path().app_data_dir()?, &app.path().home_dir()?)?;
            // The last result shows at once: the cached state is restored without reading the disk; the UI learns
            // of it through the usual events and the overview.
            if let Err(error) = engine.restore() {
                tracing::warn!(?error, "the last scan was not restored");
            }
            app.manage(engine);
            app.manage(events::EventForwarder::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::subscribe,
            commands::start_scan,
            commands::cancel_scan,
            commands::get_overview,
            commands::get_modules,
            commands::get_findings,
            commands::get_settings,
            commands::update_setting,
            commands::get_dictionary,
            commands::get_access_status,
        ])
        .build(tauri::generate_context!())?;
    app.run(|app, event| shell::on_run_event(app, &event));
    Ok(())
}
