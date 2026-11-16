//! The made-up tool `fresher-trial-tool`: its check, its listing and how one of its objects is removed.
//!
//! The listing (`fresher-trial-tool list`) prints one object per line: its id, a space and its size in bytes. A line
//! that does not read so, or whose id is not a plain name, is not claimed.

use fresher_core::{
    ByteSize, CancelToken, CategoryDescriptor, CommandRunError, CommandRunner, NewFinding, Removal, RemovalError,
    Sizes, Subject, Text, TextArg, TextKey, ToolCommand, ToolId,
};

/// The arguments of the tool's check: success means the tool is ready.
pub(crate) fn check_args() -> Vec<String> {
    vec!["version".into()]
}

/// Why the tool's listing could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ListFailed;

/// One object the tool listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Object {
    id: String,
    bytes: u64,
}

/// The tool's objects; `None` when the tool is not installed (it never is, outside the tests). A listing stopped by
/// `cancel` is a failed one: the engine reports the scan as cancelled anyway.
pub(crate) fn list(
    tool: ToolId,
    commands: &dyn CommandRunner,
    cancel: &CancelToken,
) -> Result<Option<Vec<Object>>, ListFailed> {
    match commands.run(tool.as_str(), &["list"], cancel) {
        Ok(output) if output.status == 0 => Ok(Some(output.stdout.lines().filter_map(Object::parse).collect())),
        Err(CommandRunError::NotInstalled { .. }) => Ok(None),
        Ok(_) | Err(CommandRunError::Failed { .. } | CommandRunError::Cancelled { .. }) => Err(ListFailed),
    }
}

impl Object {
    fn parse(line: &str) -> Option<Self> {
        let (id, bytes) = line.trim().split_once(' ')?;
        let plain = id.chars().all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_' | '.'));
        let starts_plain = id.chars().next().is_some_and(|char| char.is_ascii_alphanumeric());
        if !(plain && starts_plain) {
            return None;
        }
        Some(Self { id: id.into(), bytes: bytes.parse().ok()? })
    }

    /// The finding of the object: removed by `fresher-trial-tool remove <id>`, a fixed argument list.
    pub(crate) fn finding(
        &self,
        tool: ToolId,
        category: &CategoryDescriptor,
        title: &TextKey,
        reason: &Text,
    ) -> Result<NewFinding, RemovalError> {
        let command = ToolCommand::new(tool, vec!["remove".into(), self.id.clone()])?;
        let bytes = ByteSize::new(self.bytes);
        let title = Text::new(title.clone()).with("name", TextArg::Text(self.id.clone()));
        Ok(NewFinding::new(
            category,
            self.id.clone(),
            Subject::External { tool, id: self.id.clone() },
            Sizes::complete(bytes, bytes),
            title,
            reason.clone(),
            Removal::ToolCommand(command),
        ))
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::ToolId;
    use fresher_testing::FakeCommandRunner;

    use super::{ListFailed, Object, list};
    use crate::TRIAL_TOOL;

    fn listed(runner: &FakeCommandRunner) -> Result<Option<Vec<Object>>, ListFailed> {
        list(ToolId::new(TRIAL_TOOL).unwrap(), runner, &fresher_core::CancelToken::new())
    }

    mod negative {
        use fresher_core::CommandRunError;
        use fresher_testing::FakeCommandRunner;

        use super::listed;
        use crate::tool::{ListFailed, Object};

        #[test]
        fn a_tool_that_is_not_installed_lists_nothing() {
            assert_eq!(listed(&FakeCommandRunner::default()), Ok(None));
        }

        #[test]
        fn a_failed_listing_is_an_error() {
            let exiting = FakeCommandRunner::default().exiting("fresher-trial-tool list", 1);
            let failing = FakeCommandRunner::default().failing(
                "fresher-trial-tool list",
                CommandRunError::Failed { program: "fresher-trial-tool".into(), reason: "refused".into() },
            );

            assert_eq!(listed(&exiting), Err(ListFailed));
            assert_eq!(listed(&failing), Err(ListFailed));
        }

        #[test]
        fn a_line_that_is_not_a_plain_id_and_a_size_is_not_claimed() {
            for line in ["", "alone", "-rf 10", ".hidden 10", "a/b 10", "name ten", "name -1", "na me 10"] {
                assert_eq!(Object::parse(line), None, "{line:?}");
            }
        }
    }

    mod positive {
        use fresher_testing::FakeCommandRunner;

        use super::listed;
        use crate::tool::Object;

        #[test]
        fn lists_one_object_per_line() {
            let runner = FakeCommandRunner::default().answering("fresher-trial-tool list", "first 10\nsecond 20\n\n");

            let objects = listed(&runner).unwrap().unwrap();

            assert_eq!(objects, [Object { id: "first".into(), bytes: 10 }, Object { id: "second".into(), bytes: 20 }]);
            assert_eq!(runner.calls(), ["fresher-trial-tool list"]);
        }
    }
}
