//! The groups the user is in, as `id -G` lists them: what a folder's group write bit is read against.

use fresher_core::{CancelToken, CommandRunError, CommandRunner};

/// The command that lists the ids of the groups the user that runs it is in, run by its full path, without a shell.
const ID: &str = "/usr/bin/id";

/// Why the user's groups are not known.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupsUnread {
    /// The run was cancelled: nothing was learnt.
    Cancelled,
    /// The command failed, ended with a status other than 0, or answered anything but a list of group ids.
    Failed,
}

/// The user's group ids. Until they are known, no folder is writable through its group (fail closed).
pub(crate) fn user_groups(runner: &dyn CommandRunner, cancel: &CancelToken) -> Result<Vec<u32>, GroupsUnread> {
    let output = match runner.run(ID, &["-G"], cancel) {
        Ok(output) => output,
        Err(CommandRunError::Cancelled { .. }) => return Err(GroupsUnread::Cancelled),
        Err(error) => {
            tracing::warn!(%error, "the user's groups cannot be read");
            return Err(GroupsUnread::Failed);
        }
    };
    if output.status != 0 {
        tracing::warn!(status = output.status, "the user's groups cannot be read");
        return Err(GroupsUnread::Failed);
    }
    let groups: Option<Vec<u32>> = output.stdout.split_whitespace().map(|id| id.parse().ok()).collect();
    match groups {
        Some(groups) if !groups.is_empty() => Ok(groups),
        _ => {
            tracing::warn!("the user's groups are not a list of ids");
            Err(GroupsUnread::Failed)
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{CancelToken, CommandRunError};
        use fresher_testing::FakeCommandRunner;

        use crate::bundle_facts::groups::{GroupsUnread, user_groups};

        #[test]
        fn anything_but_a_list_of_ids_is_unread() {
            let failed = CommandRunError::Failed { program: "/usr/bin/id".into(), reason: "stopped".into() };
            for runner in [
                FakeCommandRunner::default().answering("/usr/bin/id -G", "20 admin 80"),
                FakeCommandRunner::default().answering("/usr/bin/id -G", "20 -1"),
                FakeCommandRunner::default().answering("/usr/bin/id -G", ""),
                FakeCommandRunner::default().answering("id -G", "20 80"),
                FakeCommandRunner::default().exiting("/usr/bin/id -G", 1),
                FakeCommandRunner::default().failing("/usr/bin/id -G", failed),
                FakeCommandRunner::default(),
            ] {
                assert_eq!(user_groups(&runner, &CancelToken::new()), Err(GroupsUnread::Failed));
            }
        }

        #[test]
        fn a_cancelled_run_is_cancelled() {
            let cancel = CancelToken::new();
            cancel.cancel();
            let runner = FakeCommandRunner::default().answering("/usr/bin/id -G", "20 80");

            assert_eq!(user_groups(&runner, &cancel), Err(GroupsUnread::Cancelled));
        }
    }

    mod positive {
        use fresher_core::CancelToken;
        use fresher_testing::FakeCommandRunner;

        use crate::bundle_facts::groups::user_groups;

        #[test]
        fn the_ids_are_read_in_order() {
            let runner = FakeCommandRunner::default().answering("/usr/bin/id -G", "20 12 61 79 80 701\n");

            assert_eq!(user_groups(&runner, &CancelToken::new()), Ok(vec![20, 12, 61, 79, 80, 701]));
            assert_eq!(runner.calls(), ["/usr/bin/id -G"]);
        }
    }
}
