//! The shell's configuration, read as files: the commands the app exposes, the window and what it is allowed.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support {
    use std::path::PathBuf;

    use serde_json::Value;

    fn read(path: &str) -> String {
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
    }

    /// The text between `start` and the first `end` after it.
    fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
        let (_, after) = text.split_once(start).unwrap();
        after.split_once(end).unwrap().0
    }

    fn names(list: &str, prefix: &str) -> Vec<String> {
        list.split(',')
            .map(|name| name.trim().trim_matches('"').trim_start_matches(prefix).to_owned())
            .filter(|name| !name.is_empty())
            .collect()
    }

    /// The commands in `tauri::generate_handler!` of `src/lib.rs`, without their module path.
    pub(crate) fn handler_commands() -> Vec<String> {
        let listed = names(between(&read("src/lib.rs"), "generate_handler![", "]"), "");
        listed.iter().map(|name| name.rsplit("::").next().unwrap_or(name).to_owned()).collect()
    }

    /// The commands of the app manifest in `build.rs`.
    pub(crate) fn manifest_commands() -> Vec<String> {
        names(between(&read("build.rs"), "const COMMANDS: &[&str] = &[", "];"), "")
    }

    /// `tauri.conf.json`.
    pub(crate) fn config() -> Value {
        serde_json::from_str(&read("tauri.conf.json")).unwrap()
    }

    /// `tauri.dev.conf.json`, the development builds' overrides.
    pub(crate) fn dev_config() -> Value {
        serde_json::from_str(&read("tauri.dev.conf.json")).unwrap()
    }

    /// The directives of a CSP, in order, each with its sources.
    pub(crate) fn directives(csp: &str) -> Vec<(&str, Vec<&str>)> {
        csp.split(';')
            .map(|directive| {
                let mut parts = directive.split_whitespace();
                (parts.next().unwrap(), parts.collect())
            })
            .collect()
    }

    /// The permissions of the main window's capability.
    pub(crate) fn capability_permissions() -> Vec<String> {
        let capability: Value = serde_json::from_str(&read("capabilities/default.json")).unwrap();
        let permissions = capability["permissions"].as_array().unwrap();
        permissions.iter().map(|permission| permission.as_str().unwrap().to_owned()).collect()
    }

    /// The core permissions of the overlay title bar's drag region: dragging the window, and zooming it by a double
    /// click (Tauri's drag script calls `start_dragging` and `internal_toggle_maximize`).
    pub(crate) const DRAG_REGION: [&str; 2] =
        ["core:window:allow-start-dragging", "core:window:allow-internal-toggle-maximize"];

    /// The core and plugin permissions of a list beyond the drag region's two.
    pub(crate) fn beyond_drag_region(permissions: &[String]) -> Vec<String> {
        permissions
            .iter()
            .filter(|permission| permission.contains(':') && !DRAG_REGION.contains(&permission.as_str()))
            .cloned()
            .collect()
    }
}

mod negative {
    use crate::support::{
        DRAG_REGION, beyond_drag_region, capability_permissions, config, dev_config, directives, handler_commands,
    };

    #[test]
    fn the_capability_allows_no_new_permission() {
        // Exactly the app's own commands and the drag region's two: a command, plugin or core permission added
        // anywhere else fails here.
        let mut expected: Vec<String> =
            handler_commands().iter().map(|command| format!("allow-{}", command.replace('_', "-"))).collect();
        expected.extend(DRAG_REGION.iter().map(|permission| (*permission).to_owned()));

        assert_eq!(capability_permissions(), expected);
        assert!(handler_commands().iter().any(|command| command == "quit_after_cleanup"));
    }

    #[test]
    fn the_config_keeps_no_remote_origin_in_the_csp() {
        // The app's own files, inline images, and Tauri's IPC: nothing else may load.
        let allowed = ["'self'", "data:", "ipc:", "http://ipc.localhost"];
        let config = config();
        let csp = config["app"]["security"]["csp"].as_str().unwrap();

        for directive in csp.split(';') {
            let mut parts = directive.split_whitespace();
            let name = parts.next().unwrap();
            for source in parts {
                assert!(allowed.contains(&source), "{name} allows {source}");
            }
        }
        assert!(csp.starts_with("default-src 'self';"), "{csp}");
    }

    #[test]
    fn the_dev_csp_is_the_production_csp_plus_what_the_dev_server_needs() {
        // Vite's dev page runs an inline React refresh preamble, injects styles inline, and reloads over a websocket
        // to the dev server; nothing else is added.
        let config = config();
        let security = &config["app"]["security"];
        let csp = directives(security["csp"].as_str().unwrap());
        let dev_csp = directives(security["devCsp"].as_str().unwrap());
        let dev_url = config["build"]["devUrl"].as_str().unwrap();
        let websocket = dev_url.replacen("http://", "ws://", 1);

        assert_eq!(
            dev_csp.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            csp.iter().map(|(name, _)| name).collect::<Vec<_>>()
        );
        for ((name, sources), (_, dev_sources)) in csp.iter().zip(&dev_csp) {
            let added: Vec<&str> = dev_sources.iter().copied().filter(|source| !sources.contains(source)).collect();
            let allowed: &[&str] = match *name {
                "script-src" | "style-src" => &["'unsafe-inline'"],
                "connect-src" => &[websocket.as_str(), dev_url],
                _ => &[],
            };
            assert_eq!(added, allowed, "{name}");
            assert!(sources.iter().all(|source| dev_sources.contains(source)), "{name}");
        }
    }

    #[test]
    fn the_dev_windows_are_the_release_windows_but_for_the_title() {
        // Tauri's merge replaces an array, so the dev config repeats the window; it must not drift.
        let release = config();
        let dev = dev_config();
        let mut release_windows = release["app"]["windows"].as_array().unwrap().clone();
        let mut dev_windows = dev["app"]["windows"].as_array().unwrap().clone();

        assert_eq!(dev_windows.len(), release_windows.len());
        for window in release_windows.iter_mut().chain(dev_windows.iter_mut()) {
            window.as_object_mut().unwrap().remove("title");
        }
        assert_eq!(dev_windows, release_windows);
        assert_ne!(dev["app"]["windows"][0]["title"], release["app"]["windows"][0]["title"]);
    }

    #[test]
    fn the_capability_allows_no_core_permission_beyond_dragging() {
        // The UI calls only the app's commands and the drag region's two. A `Channel` needs no permission: its JS
        // side is a callback, and the core's `plugin:__TAURI_CHANNEL__|fetch` is exempt from the ACL. Closing the
        // window is native, not IPC; the zoom hotkeys stay off (they would need a webview permission).
        let mut with_a_third = capability_permissions();
        with_a_third.push("core:window:allow-close".to_owned());

        assert_eq!(beyond_drag_region(&capability_permissions()), Vec::<String>::new());
        assert_eq!(beyond_drag_region(&with_a_third), ["core:window:allow-close"]);
    }
}

mod positive {
    use crate::support::{
        DRAG_REGION, capability_permissions, config, dev_config, handler_commands, manifest_commands,
    };

    #[test]
    fn the_main_window_is_incognito() {
        let config = config();
        let windows = config["app"]["windows"].as_array().unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0]["label"], "main");
        assert_eq!(windows[0]["incognito"], true);
    }

    #[test]
    fn the_dev_identifier_is_the_release_one_plus_dev() {
        let release = config()["identifier"].as_str().unwrap().to_owned();

        assert_eq!(dev_config()["identifier"], format!("{release}.dev"));
    }

    #[test]
    fn every_command_in_the_handler_is_in_the_app_manifest_and_the_capability() {
        let handler = handler_commands();
        let allowed: Vec<String> =
            handler.iter().map(|command| format!("allow-{}", command.replace('_', "-"))).collect();
        let granted: Vec<String> =
            capability_permissions().into_iter().filter(|permission| !permission.contains(':')).collect();

        assert!(handler.len() > 1, "{handler:?}");
        assert_eq!(manifest_commands(), handler);
        assert_eq!(granted, allowed);
    }

    #[test]
    fn the_capability_allows_dragging_and_zoom_by_double_click() {
        let core: Vec<String> =
            capability_permissions().into_iter().filter(|permission| permission.contains(':')).collect();

        assert_eq!(core, DRAG_REGION);
    }

    #[test]
    fn the_main_window_is_transparent_over_the_sidebar_material() {
        let config = config();
        let window = &config["app"]["windows"][0];

        assert_eq!(config["app"]["macOSPrivateApi"], true);
        assert_eq!(window["titleBarStyle"], "Overlay");
        assert_eq!(window["hiddenTitle"], true);
        assert_eq!(window["transparent"], true);
        assert_eq!(window["windowEffects"]["effects"], serde_json::json!(["sidebar"]));
        assert_eq!(window["windowEffects"]["state"], "followsWindowActiveState");
    }
}
