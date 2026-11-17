//! What the app says about itself: its version and which kind of build it is.

use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

/// The app's answer to "which build is this".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AppInfo {
    pub version: String,
    pub build: Build,
}

/// The kind of build. Only a development build can run in a sandbox; a release build never reports one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Build {
    /// The app as users run it.
    Release,
    /// A development build on the real home folder; it removes nothing outside a sandbox.
    Development,
    /// A development build living inside the sandbox folder `path`.
    Sandbox { path: PathBuf },
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::app_info::Build;

        #[test]
        fn a_build_outside_a_sandbox_serializes_no_path() {
            for build in [Build::Release, Build::Development] {
                let json = serde_json::to_string(&build).unwrap();
                assert!(!json.contains("path"), "{json}");
            }
        }
    }

    mod positive {
        use crate::app_info::{AppInfo, Build};

        #[test]
        fn a_sandbox_build_serializes_with_its_folder() {
            let info = AppInfo { version: "0.1.0".into(), build: Build::Sandbox { path: "/tmp/box".into() } };
            let json = serde_json::to_string(&info).unwrap();
            assert_eq!(json, r#"{"version":"0.1.0","build":{"kind":"sandbox","path":"/tmp/box"}}"#);
        }

        #[test]
        fn a_release_build_serializes_as_its_kind_alone() {
            let json = serde_json::to_string(&Build::Release).unwrap();
            assert_eq!(json, r#"{"kind":"release"}"#);
        }
    }
}
