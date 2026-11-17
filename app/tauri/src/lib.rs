//! The Tauri 2 shell of Fresher: the composition root and thin commands over the engine.

#![forbid(unsafe_code)]

mod adapters;
mod commands;
mod compose;
mod events;
mod identity;
#[cfg(debug_assertions)]
mod sandbox;
mod settings_commands;
mod shell;
#[cfg(test)]
mod testing;

use tauri::Manager;

/// Sandbox mode, for the autotests of the sandboxes: debug builds only.
#[cfg(debug_assertions)]
pub use sandbox::{SandboxEngineError, SandboxMode, SandboxPorts, SandboxRefusal, SandboxTrash, sandbox_engine};

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
            identity::check(&app.config().identifier, cfg!(debug_assertions))?;
            let data = app.path().app_data_dir()?;
            // The app's one home lookup: everything else gets the home injected.
            #[expect(clippy::disallowed_methods, reason = "the home folder is looked up once, in the app's setup")]
            let home = app.path().home_dir()?;
            // In a development build, `FRESHER_SANDBOX` moves the whole app into a sandbox folder; a refusal stops
            // the start rather than run anywhere else.
            let start = compose::start(data, home, |name| std::env::var_os(name))
                .inspect_err(|error| tracing::error!(%error, "the app cannot start"))?;
            let compose::Start { engine, data_folder, home, build } = start;
            // The last result shows at once: the cached state is restored without reading the disk; the UI learns
            // of it through the usual events and the overview.
            if let Err(error) = engine.restore() {
                tracing::warn!(?error, "the last scan was not restored");
            }
            app.manage(engine);
            app.manage(build);
            app.manage(settings_commands::HomeFolder(home));
            app.manage(events::EventForwarder::default());
            app.manage(fresher_store::DataFolder::open(data_folder)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
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
