//! A tool's removal: its program with its fixed arguments through the `CommandRunner` port, never a shell.

use fresher_core::{Act, CancelToken, CommandRunner, Failure, ItemOutcome, ToolCommand};

use crate::remove::Removed;

/// At most this many characters of a tool's error are shown.
const ERROR_LENGTH: usize = 200;

/// Runs the tool: exit 0 is `ToolDone`; any other status fails with the first line of its standard error (shown,
/// never journalled: the journal keeps the status); a tool that cannot be started fails as missing.
///
/// The tool runs to its end (or its timeout) even when the user stops the cleanup: Stop ends the cleanup after the
/// item in flight, and a removal killed halfway would leave its state unknown.
pub(crate) fn run_tool(command: &ToolCommand, _act: Act<'_>, runner: &dyn CommandRunner) -> Removed {
    let args: Vec<&str> = command.args().iter().map(String::as_str).collect();
    match runner.run(command.tool().as_str(), &args, &CancelToken::new()) {
        Ok(output) if output.status == 0 => {
            Removed { outcome: ItemOutcome::ToolDone, trashed: Vec::new(), status: Some(0) }
        }
        Ok(output) => {
            let first = output.stderr.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or_default();
            let error = first.chars().take(ERROR_LENGTH).collect();
            let reason = Failure::ToolFailed { error }.text();
            Removed { outcome: ItemOutcome::Failed { reason }, trashed: Vec::new(), status: Some(output.status) }
        }
        Err(_) => Removed::ended(ItemOutcome::Failed { reason: Failure::ToolMissing.text() }),
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{CommandOutput, CommandRunError, CommandRunner, ToolCommand, ToolId};

    fn command() -> ToolCommand {
        ToolCommand::new(ToolId::new("docker").unwrap(), vec!["image".into(), "rm".into(), "abc".into()]).unwrap()
    }

    /// A tool that fails with `stderr`.
    #[derive(Debug)]
    struct Failing(String);

    impl CommandRunner for Failing {
        fn run(
            &self,
            _program: &str,
            _args: &[&str],
            _cancel: &fresher_core::CancelToken,
        ) -> Result<CommandOutput, CommandRunError> {
            Ok(CommandOutput { status: 1, stdout: String::new(), stderr: self.0.clone() })
        }
    }

    mod negative {
        use fresher_core::{CommandRunError, Failure, ItemOutcome, Method};
        use fresher_testing::FakeCommandRunner;

        use super::{Failing, command};
        use crate::remove::run_tool;
        use crate::test_support::{Setup, confirmed};

        #[test]
        fn a_tool_that_fails_reports_its_first_error_line() {
            let setup = Setup::new("");
            let plan = confirmed(&setup, &[], Method::Trash);
            let long = "x".repeat(300);
            let runner = Failing(format!("\n  Error: image is in use\n{long}"));

            let removed = run_tool(&command(), plan.act(), &runner);

            let reason = Failure::ToolFailed { error: "Error: image is in use".into() }.text();
            assert_eq!(removed.outcome, ItemOutcome::Failed { reason });
            assert_eq!(removed.status, Some(1));
        }

        #[test]
        fn a_tool_error_is_cut_to_two_hundred_characters() {
            let setup = Setup::new("");
            let plan = confirmed(&setup, &[], Method::Trash);

            let removed = run_tool(&command(), plan.act(), &Failing("é".repeat(300)));

            let reason = Failure::ToolFailed { error: "é".repeat(200) }.text();
            assert_eq!(removed.outcome, ItemOutcome::Failed { reason });
        }

        #[test]
        fn a_tool_that_is_not_installed_fails() {
            let setup = Setup::new("");
            let plan = confirmed(&setup, &[], Method::Trash);
            let runner = FakeCommandRunner::default()
                .failing("docker image rm abc", CommandRunError::NotInstalled { program: "docker".into() });

            let removed = run_tool(&command(), plan.act(), &runner);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::ToolMissing.text() });
            assert_eq!(removed.status, None);
            assert_eq!(runner.calls(), ["docker image rm abc"]);
        }
    }

    mod positive {
        use fresher_core::{ItemOutcome, Method};
        use fresher_testing::FakeCommandRunner;

        use super::command;
        use crate::remove::run_tool;
        use crate::test_support::{Setup, confirmed};

        #[test]
        fn a_tool_that_succeeds_is_done() {
            let setup = Setup::new("");
            let plan = confirmed(&setup, &[], Method::Trash);
            let runner = FakeCommandRunner::default().answering("docker image rm abc", "Deleted: abc");

            let removed = run_tool(&command(), plan.act(), &runner);

            assert_eq!(removed.outcome, ItemOutcome::ToolDone);
            assert_eq!(removed.status, Some(0));
            assert_eq!(runner.calls(), ["docker image rm abc"]);
        }
    }
}
