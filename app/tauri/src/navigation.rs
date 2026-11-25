//! The window never leaves the app's own page: a navigation to any other address (a link, a form, a script) is
//! cancelled. Only the embedded page's origin is allowed, and in Tauri's dev mode the dev server's.

use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, Url};

use crate::identity::Page;

/// The scheme and host of the embedded page on macOS (`tauri://localhost`).
const APP_SCHEME: &str = "tauri";
const APP_HOST: &str = "localhost";

/// Whether the window may go to `url`: the embedded page's origin, or the dev server's when there is one.
pub(crate) fn allows(url: &Url, dev_server: Option<&Url>) -> bool {
    let own = url.scheme() == APP_SCHEME && url.host_str() == Some(APP_HOST) && url.port().is_none();
    own || dev_server.is_some_and(|dev| dev.origin() == url.origin())
}

/// The guard, as a plugin so it holds for every webview of the app.
pub(crate) fn guard<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("navigation-guard")
        .on_navigation(|webview, url| {
            let config = webview.app_handle().config();
            let dev_server = match Page::of_this_build() {
                Page::DevServer => config.build.dev_url.as_ref(),
                Page::Embedded => None,
            };
            let allowed = allows(url, dev_server);
            if !allowed {
                tracing::warn!("a navigation away from the app's page was cancelled");
            }
            allowed
        })
        .build()
}

#[cfg(test)]
mod tests {
    mod negative {
        use tauri::Url;

        use crate::navigation::allows;

        fn url(text: &str) -> Url {
            Url::parse(text).unwrap()
        }

        #[test]
        fn a_remote_page_is_refused() {
            for text in ["https://example.com/", "http://localhost/", "file:///etc/passwd", "data:text/html,<b>x</b>"] {
                assert!(!allows(&url(text), None), "{text}");
            }
        }

        #[test]
        fn a_lookalike_of_the_app_origin_is_refused() {
            for text in
                ["tauri://localhost.example.com/", "tauri://evil/", "tauri://localhost:1/", "http://tauri.localhost/"]
            {
                assert!(!allows(&url(text), None), "{text}");
            }
        }

        #[test]
        fn another_port_than_the_dev_servers_is_refused() {
            let dev = url("http://localhost:1420");

            assert!(!allows(&url("http://localhost:1421/"), Some(&dev)));
            assert!(!allows(&url("https://localhost:1420/"), Some(&dev)));
        }

        #[test]
        fn the_dev_server_is_refused_without_one() {
            assert!(!allows(&url("http://localhost:1420/"), None));
        }
    }

    mod positive {
        use tauri::Url;

        use crate::navigation::allows;

        fn url(text: &str) -> Url {
            Url::parse(text).unwrap()
        }

        #[test]
        fn the_embedded_page_is_allowed() {
            assert!(allows(&url("tauri://localhost"), None));
            assert!(allows(&url("tauri://localhost/index.html#/history"), None));
        }

        #[test]
        fn the_dev_server_is_allowed_in_dev_mode() {
            let dev = url("http://localhost:1420");

            assert!(allows(&url("http://localhost:1420/"), Some(&dev)));
            assert!(allows(&url("http://localhost:1420/src/main.tsx?t=1"), Some(&dev)));
        }
    }
}
