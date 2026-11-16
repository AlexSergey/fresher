//! The Tauri 2 shell of Fresher: the composition root and thin commands over the engine.

#![forbid(unsafe_code)]

mod adapters;
mod commands;
mod compose;
mod events;
mod settings_commands;
mod shell;
#[cfg(test)]
mod testing;

use tauri::Manager;

/// Builds and runs the app until its window is closed; a second launch brings that window forward instead.
pub fn run() -> Result<(), tauri::Error> {
    let app = tauri::Builder::default()
        // First, so a second launch exits before it sets anything up.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| shell::on_second_launch(app)))
        // The folder picker, called from Rust only: the web view is granted none of its commands.
        .plugin(tauri_plugin_dialog::init())
        .menu(shell::menu)
        .on_menu_event(|app, event| shell::on_menu_event(app, &event))
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            let engine = compose::engine(&data, &app.path().home_dir()?)?;
            // The last result shows at once: the cached state is restored without reading the disk; the UI learns
            // of it through the usual events and the overview.
            if let Err(error) = engine.restore() {
                tracing::warn!(?error, "the last scan was not restored");
            }
            app.manage(engine);
            app.manage(events::EventForwarder::default());
            app.manage(fresher_store::DataFolder::open(data)?);
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
            commands::reset_settings,
            commands::ignore,
            commands::restore_ignored,
            settings_commands::pick_folder,
            settings_commands::get_own_data,
            commands::get_dictionary,
            commands::get_access_status,
            commands::prepare_cleanup,
            commands::discard_cleanup,
            commands::confirm_cleanup,
            commands::cancel_cleanup,
            commands::get_cleanups,
            commands::get_cleanup,
            commands::put_back,
            commands::reveal_in_trash,
            commands::quit_after_cleanup,
        ])
        .build(tauri::generate_context!())?;
    app.run(|app, event| shell::on_run_event(app, &event));
    Ok(())
}
