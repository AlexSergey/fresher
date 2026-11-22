//! The system's interface on macOS: System Settings at a privacy pane, and Finder, both through `/usr/bin/open`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{CancelToken, Capability, CommandRunner, SystemUi, SystemUiError};
use fresher_macos::{OPEN, open_args, open_folder_args, pane_of, reveal_args};

use crate::access::guard_of;

/// Opens System Settings and Finder with `/usr/bin/open`, on the user's click only.
pub struct MacSystemUi {
    commands: Arc<dyn CommandRunner>,
}

impl std::fmt::Debug for MacSystemUi {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MacSystemUi").finish_non_exhaustive()
    }
}

impl MacSystemUi {
    /// The interface that runs `/usr/bin/open` through `commands`.
    pub fn new(commands: Arc<dyn CommandRunner>) -> Self {
        Self { commands }
    }

    /// Runs `/usr/bin/open` with `args`. A fresh token: the call is the user's click, and `open` returns at once.
    fn open(&self, args: &[&str]) -> Result<(), SystemUiError> {
        match self.commands.run(OPEN, args, &CancelToken::new()) {
            Ok(output) if output.status == 0 => Ok(()),
            Ok(output) => Err(SystemUiError::Failed {
                reason: format!("{OPEN} exited with status {}: {}", output.status, output.stderr.trim()),
            }),
            Err(error) => Err(SystemUiError::Failed { reason: error.to_string() }),
        }
    }
}

impl SystemUi for MacSystemUi {
    fn open_access_settings(&self, capability: &Capability) -> Result<(), SystemUiError> {
        let guard = guard_of(*capability).ok_or(SystemUiError::Unsupported)?;
        self.open(&open_args(pane_of(guard)))
    }

    fn reveal(&self, path: &Path) -> Result<(), SystemUiError> {
        let [flag, path] = reveal_args(path);
        let not_text = || SystemUiError::Failed { reason: "the path is not valid UTF-8".into() };
        let args = [flag.to_str().ok_or_else(not_text)?, path.to_str().ok_or_else(not_text)?];
        self.open(&args)
    }

    /// Refuses a path that is not absolute before anything runs: `open` would resolve it against its own folder.
    fn open_folder(&self, path: &Path) -> Result<(), SystemUiError> {
        if !path.is_absolute() {
            return Err(SystemUiError::Failed { reason: "the path is not absolute".into() });
        }
        let [path] = open_folder_args(path);
        let path =
            path.to_str().ok_or_else(|| SystemUiError::Failed { reason: "the path is not valid UTF-8".into() })?;
        self.open(&[path])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_testing::FakeCommandRunner;

    use crate::MacSystemUi;

    fn ui(runner: &Arc<FakeCommandRunner>) -> MacSystemUi {
        MacSystemUi::new(Arc::clone(runner) as _)
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{Capability, CommandRunError, SystemUi, SystemUiError, ToolId};
        use fresher_macos::{Pane, pane_address};
        use fresher_testing::FakeCommandRunner;

        use super::ui;

        #[test]
        fn a_tool_has_no_settings_pane() {
            let runner = Arc::new(FakeCommandRunner::default());
            let tool = Capability::Tool(ToolId::new("docker").unwrap());

            assert_eq!(ui(&runner).open_access_settings(&tool), Err(SystemUiError::Unsupported));
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_failed_open_is_reported_as_a_system_ui_error() {
            let address = pane_address(Pane::FullDiskAccess);
            let exiting = Arc::new(FakeCommandRunner::default().exiting(&format!("/usr/bin/open {address}"), 1));
            let error = CommandRunError::Failed { program: "/usr/bin/open".into(), reason: "no".into() };
            let failing = Arc::new(FakeCommandRunner::default().failing("/usr/bin/open -R /tmp/x", error));

            let exited = ui(&exiting).open_access_settings(&Capability::AllFiles);
            let failed = ui(&failing).reveal(Path::new("/tmp/x"));

            assert!(matches!(exited, Err(SystemUiError::Failed { .. })), "{exited:?}");
            assert!(matches!(failed, Err(SystemUiError::Failed { .. })), "{failed:?}");
        }

        #[test]
        fn a_path_that_is_not_utf8_is_not_revealed() {
            let runner = Arc::new(FakeCommandRunner::default());
            let path = Path::new(OsStr::from_bytes(b"/tmp/not-\xFF-text"));

            let result = ui(&runner).reveal(path);

            assert!(matches!(result, Err(SystemUiError::Failed { .. })), "{result:?}");
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_relative_folder_is_not_opened_and_nothing_runs() {
            let runner = Arc::new(FakeCommandRunner::default());

            let result = ui(&runner).open_folder(Path::new(".Trash"));

            assert_eq!(result, Err(SystemUiError::Failed { reason: "the path is not absolute".into() }));
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_folder_path_that_is_not_utf8_is_not_opened() {
            let runner = Arc::new(FakeCommandRunner::default());
            let path = Path::new(OsStr::from_bytes(b"/tmp/not-\xFF-text"));

            let result = ui(&runner).open_folder(path);

            assert!(matches!(result, Err(SystemUiError::Failed { .. })), "{result:?}");
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_failing_open_of_a_folder_is_a_system_ui_error() {
            let exiting = Arc::new(FakeCommandRunner::default().exiting("/usr/bin/open /tmp/x", 1));
            let error = CommandRunError::Failed { program: "/usr/bin/open".into(), reason: "no".into() };
            let failing = Arc::new(FakeCommandRunner::default().failing("/usr/bin/open /tmp/x", error));

            let exited = ui(&exiting).open_folder(Path::new("/tmp/x"));
            let failed = ui(&failing).open_folder(Path::new("/tmp/x"));

            assert!(matches!(exited, Err(SystemUiError::Failed { .. })), "{exited:?}");
            assert!(matches!(failed, Err(SystemUiError::Failed { .. })), "{failed:?}");
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{Capability, GuardedFolder, SystemUi, VolumeKind};
        use fresher_macos::{Pane, pane_address};
        use fresher_testing::FakeCommandRunner;

        use super::ui;

        #[test]
        fn open_access_settings_runs_open_with_the_pane_address() {
            let open = |pane| format!("/usr/bin/open {}", pane_address(pane));
            let runner = Arc::new(
                FakeCommandRunner::default()
                    .answering(&open(Pane::FullDiskAccess), "")
                    .answering(&open(Pane::FilesAndFolders), "")
                    .answering(&open(Pane::PrivacyAndSecurity), ""),
            );
            let ui = ui(&runner);

            for capability in [
                Capability::AllFiles,
                Capability::Folder(GuardedFolder::Documents),
                Capability::Volume(VolumeKind::Network),
                Capability::OtherAppsData,
            ] {
                assert_eq!(ui.open_access_settings(&capability), Ok(()), "{capability:?}");
            }

            assert_eq!(
                runner.calls(),
                [
                    "/usr/bin/open x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
                    "/usr/bin/open x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders",
                    "/usr/bin/open x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders",
                    "/usr/bin/open x-apple.systempreferences:com.apple.preference.security?Privacy",
                ]
            );
        }

        #[test]
        fn reveal_runs_open_with_dash_r_and_the_path() {
            let runner = Arc::new(FakeCommandRunner::default().answering("/usr/bin/open -R /Users/a b/it's", ""));

            assert_eq!(ui(&runner).reveal(Path::new("/Users/a b/it's")), Ok(()));
            assert_eq!(runner.calls(), ["/usr/bin/open -R /Users/a b/it's"]);
        }

        #[test]
        fn open_folder_runs_open_with_the_path_alone() {
            let runner = Arc::new(FakeCommandRunner::default().answering("/usr/bin/open /tmp/x", ""));

            assert_eq!(ui(&runner).open_folder(Path::new("/tmp/x")), Ok(()));
            assert_eq!(runner.calls(), ["/usr/bin/open /tmp/x"]);
        }
    }
}
