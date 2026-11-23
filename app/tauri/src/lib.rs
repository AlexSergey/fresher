//! The Tauri 2 shell of Fresher: the composition root and thin commands over the engine.

#![forbid(unsafe_code)]

#[cfg(any(debug_assertions, test))]
mod adapters;
mod apps_commands;
mod commands;
mod compose;
#[cfg(debug_assertions)]
mod demo;
#[cfg(debug_assertions)]
mod dev_runs;
#[cfg(debug_assertions)]
mod dry_run;
mod events;
mod identity;
mod largest_commands;
#[cfg(debug_assertions)]
mod sandbox;
mod settings_commands;
mod shell;
mod start_failure;
mod storage_commands;
#[cfg(test)]
mod testing;

use tauri::Manager;

/// Sandbox mode, for the autotests of the sandboxes: debug builds only.
#[cfg(debug_assertions)]
pub use sandbox::{
    SandboxEngineError, SandboxMode, SandboxPorts, SandboxRefusal, SandboxTrash, access_of, sandbox_engine,
};

/// Builds and runs the app until its window is closed; a second launch brings that window forward instead.
pub fn run() -> Result<(), tauri::Error> {
    let handler = handler();
    let app = tauri::Builder::default()
        // First, so a second launch exits before it sets anything up.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| shell::on_second_launch(app)))
        // The folder picker, called from Rust only: the web view is granted none of its commands.
        .plugin(tauri_plugin_dialog::init())
        .menu(shell::menu)
        .on_menu_event(|app, event| shell::on_menu_event(app, &event))
        .setup(|app| {
            // A failed start ends the app after a native message; setup itself succeeds so the event loop runs and
            // the message can show.
            if let Err(error) = start(app) {
                start_failure::report(app.handle(), &*error);
            }
            Ok(())
        })
        .invoke_handler(move |invoke| {
            // A development build records the actions the UI asks for, before they run.
            #[cfg(debug_assertions)]
            dev_runs::commands::observe(&invoke.message);
            handler(invoke)
        })
        .build(tauri::generate_context!())?;
    app.run(|app, event| shell::on_run_event(app, &event));
    Ok(())
}

/// Every command the UI may call. Each is an action or a read of `dev_runs::commands`, which a test checks.
fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
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
        commands::get_agreement,
        commands::accept_agreement,
        commands::ignore,
        commands::restore_ignored,
        settings_commands::pick_folder,
        settings_commands::get_own_data,
        storage_commands::get_storage_map,
        storage_commands::get_storage_meaning,
        largest_commands::get_largest,
        largest_commands::get_largest_place,
        largest_commands::pick_own_item,
        apps_commands::get_apps,
        apps_commands::get_app,
        apps_commands::reveal_app,
        apps_commands::prepare_uninstall,
        commands::get_dictionary,
        commands::get_access_status,
        commands::refresh_access,
        commands::open_access_settings,
        commands::reveal_finding,
        commands::relaunch,
        commands::prepare_cleanup,
        commands::discard_cleanup,
        commands::confirm_cleanup,
        commands::cancel_cleanup,
        commands::get_cleanups,
        commands::get_cleanup,
        commands::put_back,
        commands::reveal_in_trash,
        commands::get_trash,
        commands::open_trash,
        commands::quit_after_cleanup,
        #[cfg(debug_assertions)]
        demo::demo_play,
        #[cfg(debug_assertions)]
        demo::demo_finished,
    ]
}

/// Everything the app does at start; any error ends the app after the native message.
fn start(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    identity::check(&app.config().identifier, cfg!(debug_assertions))?;
    let data = app.path().app_data_dir()?;
    // The app's one home lookup: everything else gets the home injected.
    #[expect(clippy::disallowed_methods, reason = "the home folder is looked up once, in the app's setup")]
    let home = app.path().home_dir()?;
    // In a development build, `FRESHER_SANDBOX` moves the whole app into a sandbox folder; a refusal stops
    // the start rather than run anywhere else.
    let start = compose::start(data, home, |name| std::env::var_os(name))?;
    let compose::Start {
        engine,
        data_folder,
        home,
        build,
        #[cfg(debug_assertions)]
        run,
        #[cfg(debug_assertions)]
        demo,
    } = start;
    // Demo mode sizes and places the window before the UI asks for its scenario; every development build manages
    // the mode, `None` without `FRESHER_DEMO`.
    #[cfg(debug_assertions)]
    if let Some(mode) = &demo
        && let Some(window) = app.get_webview_window(shell::MAIN_WINDOW)
    {
        mode.place(&window);
    }
    #[cfg(debug_assertions)]
    app.manage(demo);
    // The record subscribes before the restore, so the restored scan is recorded.
    #[cfg(debug_assertions)]
    let recorded = run.map(|run| {
        let events = engine.subscribe();
        app.manage(dev_runs::DevRuns::new(run));
        events
    });
    // The last result shows at once: the cached state is restored without reading the disk; the UI learns
    // of it through the usual events and the overview.
    if let Err(error) = engine.restore() {
        tracing::warn!(?error, "the last scan was not restored");
    }
    app.manage(engine);
    #[cfg(debug_assertions)]
    if let Some(events) = recorded {
        dev_runs::follow(app.handle().clone(), events);
    }
    app.manage(build);
    app.manage(settings_commands::HomeFolder(home));
    app.manage(events::EventForwarder::default());
    app.manage(fresher_store::DataFolder::open(data_folder)?);
    Ok(())
}
