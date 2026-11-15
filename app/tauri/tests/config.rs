//! The shell's configuration, read as files: the commands the app exposes and what the window is allowed.

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

    /// The commands in `tauri::generate_handler!` of `src/lib.rs`.
    pub(crate) fn handler_commands() -> Vec<String> {
        names(between(&read("src/lib.rs"), "generate_handler![", "]"), "commands::")
    }

    /// The commands of the app manifest in `build.rs`.
    pub(crate) fn manifest_commands() -> Vec<String> {
        names(between(&read("build.rs"), "const COMMANDS: &[&str] = &[", "];"), "")
    }

    /// `tauri.conf.json`.
    pub(crate) fn config() -> Value {
        serde_json::from_str(&read("tauri.conf.json")).unwrap()
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
}

mod negative {
    use crate::support::{capability_permissions, config, directives};

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
    fn the_capability_file_grants_no_core_or_plugin_permission() {
        // The UI calls only the app's commands. A `Channel` needs no permission: its JS side is a callback, and the
        // core's `plugin:__TAURI_CHANNEL__|fetch` is exempt from the ACL. Closing the window is native, not IPC.
        let plugins: Vec<String> =
            capability_permissions().into_iter().filter(|permission| permission.contains(':')).collect();

        assert_eq!(plugins, Vec::<String>::new());
    }
}

mod positive {
    use crate::support::{capability_permissions, config, handler_commands, manifest_commands};

    #[test]
    fn the_main_window_is_incognito() {
        let config = config();
        let windows = config["app"]["windows"].as_array().unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0]["label"], "main");
        assert_eq!(windows[0]["incognito"], true);
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
}
