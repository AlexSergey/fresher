//! What the app says about itself: its version and which kind of build it is.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

use crate::{Text, TextArg, TextKey};

/// The app's answer to "which build is this".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AppInfo {
    pub version: String,
    pub build: Build,
}

/// The kind of build. Only a development build can run in a sandbox; a release build never reports one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum Build {
    /// The app as users run it.
    Release,
    /// A development build on the real home folder; it removes nothing outside a sandbox.
    Development,
    /// A development build living inside the sandbox folder `path`; `simulated` lists what its world answers instead
    /// of the Mac, in this order; `trash` says where its cleanups move items, as the sentence the banner and a
    /// cleanup's result show.
    Sandbox { path: PathBuf, simulated: Vec<Simulated>, trash: Text },
}

/// Where a sandbox's cleanups move items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTrashAt<'a> {
    /// The sandbox's own Trash folder.
    Sandbox(&'a Path),
    /// The Trash of this Mac (`--real-trash`).
    System,
}

impl SandboxTrashAt<'_> {
    /// `core.build.trash.sandbox` with `{path}`, or `core.build.trash.system`.
    pub fn text(&self) -> Text {
        match self {
            Self::Sandbox(path) => Text::new(TextKey::from_valid("core.build.trash.sandbox".into()))
                .with("path", TextArg::Path(path.to_path_buf())),
            Self::System => Text::new(TextKey::from_valid("core.build.trash.system".into())),
        }
    }
}

/// A part of the Mac a sandbox's world answers instead of the real one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Simulated {
    /// The apps it sees open are the sandbox's declared ones, not the Mac's.
    Running,
    /// What Spotlight answers: an app found elsewhere, or Spotlight off.
    Spotlight,
    /// The access of the guarded folders.
    Access,
    /// The installer receipts and the macOS version.
    Installs,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::app_info::{Build, SandboxTrashAt};

        #[test]
        fn a_build_outside_a_sandbox_serializes_no_path() {
            for build in [Build::Release, Build::Development] {
                let json = serde_json::to_string(&build).unwrap();
                assert!(!json.contains("path"), "{json}");
                assert!(!json.contains("trash"), "{json}");
            }
        }

        #[test]
        fn the_systems_trash_has_no_path_arg() {
            let text = SandboxTrashAt::System.text();

            assert_eq!(text.key().as_str(), "core.build.trash.system");
            assert!(text.args().is_empty(), "{text:?}");
        }
    }

    mod positive {
        use std::path::Path;

        use crate::app_info::{AppInfo, Build, SandboxTrashAt, Simulated};

        #[test]
        fn a_sandbox_build_serializes_with_its_folder() {
            let build = Build::Sandbox {
                path: "/tmp/box".into(),
                simulated: vec![Simulated::Running, Simulated::Access],
                trash: SandboxTrashAt::System.text(),
            };
            let info = AppInfo { version: "0.1.0".into(), build };
            let json = serde_json::to_string(&info).unwrap();
            assert_eq!(
                json,
                r#"{"version":"0.1.0","build":{"kind":"sandbox","path":"/tmp/box","simulated":["running","access"],"trash":{"key":"core.build.trash.system","args":{}}}}"#
            );
        }

        #[test]
        fn a_sandbox_build_serializes_its_trash_sentence() {
            let trash = SandboxTrashAt::Sandbox(Path::new("/tmp/box/home/.Trash")).text();
            let build = Build::Sandbox { path: "/tmp/box".into(), simulated: vec![], trash };

            let json = serde_json::to_value(&build).unwrap();

            assert_eq!(
                json["trash"],
                serde_json::json!({
                    "key": "core.build.trash.sandbox",
                    "args": { "path": { "kind": "path", "value": "/tmp/box/home/.Trash" } }
                })
            );
        }

        #[test]
        fn a_release_build_serializes_as_its_kind_alone() {
            let json = serde_json::to_string(&Build::Release).unwrap();
            assert_eq!(json, r#"{"kind":"release"}"#);
        }
    }
}
