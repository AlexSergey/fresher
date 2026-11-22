//! Finder's settings on macOS, read with `/usr/bin/defaults`. Only reads: nothing here changes a setting.

use std::sync::Arc;

use fresher_core::{AutoEmpty, CancelToken, CommandOutput, CommandRunner, FinderSettings};

/// The program that reads a preference.
const DEFAULTS: &str = "/usr/bin/defaults";
/// The verb of `defaults` that prints a preference.
const READ: &str = "read";
/// Finder's preference domain.
const FINDER_DOMAIN: &str = "com.apple.finder";
/// The key of "Remove items from the Trash after 30 days".
const REMOVE_OLD_TRASH_ITEMS: &str = "FXRemoveOldTrashItems";

/// Whether `program` with `args` is exactly the read of Finder's Trash setting, the one `defaults` call made here.
pub fn is_trash_setting_read(program: &str, args: &[&str]) -> bool {
    program == DEFAULTS && args == [READ, FINDER_DOMAIN, REMOVE_OLD_TRASH_ITEMS]
}

/// Reads Finder's settings through `defaults read`.
pub struct MacFinderSettings {
    commands: Arc<dyn CommandRunner>,
}

impl std::fmt::Debug for MacFinderSettings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MacFinderSettings").finish_non_exhaustive()
    }
}

impl MacFinderSettings {
    /// The settings read by running `/usr/bin/defaults` through `commands`.
    pub fn new(commands: Arc<dyn CommandRunner>) -> Self {
        Self { commands }
    }
}

impl FinderSettings for MacFinderSettings {
    /// A key that was never set is Finder's default: off.
    fn trash_auto_empty(&self) -> AutoEmpty {
        let args = [READ, FINDER_DOMAIN, REMOVE_OLD_TRASH_ITEMS];
        match self.commands.run(DEFAULTS, &args, &CancelToken::new()) {
            Ok(output) => auto_empty_of(&output),
            Err(error) => {
                tracing::warn!(%error, "Finder's Trash setting cannot be read");
                AutoEmpty::Unknown
            }
        }
    }
}

fn auto_empty_of(output: &CommandOutput) -> AutoEmpty {
    match (output.status, output.stdout.trim()) {
        (0, "1") => AutoEmpty::After30Days,
        (0, "0") => AutoEmpty::Never,
        (status, _) if status != 0 && output.stderr.contains("does not exist") => AutoEmpty::Never,
        (status, _) => {
            tracing::warn!(status, "Finder's Trash setting has an unexpected answer");
            AutoEmpty::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{AutoEmpty, FinderSettings};
    use fresher_testing::FakeCommandRunner;

    use crate::MacFinderSettings;

    const COMMAND: &str = "/usr/bin/defaults read com.apple.finder FXRemoveOldTrashItems";

    fn read(runner: FakeCommandRunner) -> AutoEmpty {
        MacFinderSettings::new(Arc::new(runner)).trash_auto_empty()
    }

    mod negative {
        use fresher_core::{AutoEmpty, CommandRunError};
        use fresher_testing::FakeCommandRunner;

        use super::{COMMAND, read};

        #[test]
        fn a_failing_runner_is_unknown() {
            let error = CommandRunError::Failed { program: "/usr/bin/defaults".into(), reason: "no".into() };

            assert_eq!(read(FakeCommandRunner::default().failing(COMMAND, error)), AutoEmpty::Unknown);
            assert_eq!(read(FakeCommandRunner::default()), AutoEmpty::Unknown);
        }

        #[test]
        fn an_unexpected_answer_is_unknown() {
            assert_eq!(read(FakeCommandRunner::default().answering(COMMAND, "yes\n")), AutoEmpty::Unknown);
        }

        #[test]
        fn another_failing_status_is_unknown() {
            let runner = FakeCommandRunner::default().exiting_with_stderr(COMMAND, 1, "Could not read the domain");

            assert_eq!(read(runner), AutoEmpty::Unknown);
            assert_eq!(read(FakeCommandRunner::default().exiting(COMMAND, 1)), AutoEmpty::Unknown);
        }

        #[test]
        fn a_missing_key_is_finders_default_never() {
            let stderr = "The domain/default pair of (com.apple.finder, FXRemoveOldTrashItems) does not exist\n";
            let runner = FakeCommandRunner::default().exiting_with_stderr(COMMAND, 1, stderr);

            assert_eq!(read(runner), AutoEmpty::Never);
        }

        #[test]
        fn the_reader_never_writes_a_setting() {
            let source = include_str!("finder_settings.rs");
            let reader = source.split("#[cfg(test)]").next().unwrap_or(source);

            assert!(!reader.contains("write"));
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{AutoEmpty, FinderSettings};
        use fresher_testing::FakeCommandRunner;

        use super::{COMMAND, read};
        use crate::MacFinderSettings;

        #[test]
        fn one_is_after_30_days() {
            assert_eq!(read(FakeCommandRunner::default().answering(COMMAND, "1\n")), AutoEmpty::After30Days);
        }

        #[test]
        fn zero_is_never() {
            assert_eq!(read(FakeCommandRunner::default().answering(COMMAND, "0\n")), AutoEmpty::Never);
        }

        #[test]
        fn it_runs_defaults_read_with_the_exact_arguments() {
            let runner = Arc::new(FakeCommandRunner::default().answering(COMMAND, "1"));

            MacFinderSettings::new(Arc::clone(&runner) as _).trash_auto_empty();

            assert_eq!(runner.calls(), [COMMAND]);
        }
    }
}
