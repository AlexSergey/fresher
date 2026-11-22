//! The programs sandbox mode runs: `/usr/bin/open` (the system's interface: Show in Finder, System Settings) and the
//! one read of Finder's Trash setting (`defaults read`, the Mac's own setting, so the Trash view's "gone by" date
//! shows) for real, and every other program only as the sandbox's world answers it, so no program of the Mac is run
//! whatever the case files say.

use std::sync::Arc;

use fresher_core::{CancelToken, CommandOutput, CommandRunError, CommandRunner};
use fresher_macos::OPEN;
use fresher_sandbox::WorldCommands;
use fresher_system::is_trash_setting_read;

/// Sends a call to exactly [`OPEN`], and the exact read of Finder's Trash setting, to `open`; every other call,
/// whatever its program, goes to the world's answers.
pub(crate) struct SandboxCommands {
    pub(crate) world: Arc<WorldCommands>,
    pub(crate) open: Arc<dyn CommandRunner>,
}

impl CommandRunner for SandboxCommands {
    fn run(&self, program: &str, args: &[&str], cancel: &CancelToken) -> Result<CommandOutput, CommandRunError> {
        if program == OPEN || is_trash_setting_read(program, args) {
            self.open.run(program, args, cancel)
        } else {
            self.world.run(program, args, cancel)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use fresher_sandbox::World;
    use fresher_testing::FakeCommandRunner;

    use super::SandboxCommands;

    /// Sandbox commands of a world stating nothing, at `/box`, with `open` as the real runner.
    fn commands(open: &Arc<FakeCommandRunner>) -> (SandboxCommands, Arc<fresher_sandbox::WorldCommands>) {
        let world = Arc::new(World::default().commands(Path::new("/box")));
        (SandboxCommands { world: Arc::clone(&world), open: Arc::clone(open) as _ }, world)
    }

    mod negative {
        use std::sync::Arc;

        use fresher_core::{CancelToken, CommandRunError, CommandRunner};
        use fresher_testing::FakeCommandRunner;

        use super::commands;

        #[test]
        fn sandbox_commands_never_reach_a_real_program_but_open() {
            let open = Arc::new(FakeCommandRunner::default());
            let (commands, world) = commands(&open);

            let echo = commands.run("/bin/echo", &["hi"], &CancelToken::new());
            let near = commands.run("/usr/bin/open2", &[], &CancelToken::new());

            assert_eq!(echo, Err(CommandRunError::NotInstalled { program: "/bin/echo".into() }));
            assert!(matches!(near, Err(CommandRunError::NotInstalled { .. })), "{near:?}");
            assert_eq!(world.unanswered(), ["/bin/echo hi", "/usr/bin/open2"]);
            assert_eq!(open.calls(), Vec::<String>::new());
        }

        #[test]
        fn defaults_other_than_the_trash_setting_read_is_not_let_through() {
            let open = Arc::new(FakeCommandRunner::default());
            let (commands, world) = commands(&open);

            let write = commands.run(
                "/usr/bin/defaults",
                &["write", "com.apple.finder", "FXRemoveOldTrashItems", "1"],
                &CancelToken::new(),
            );
            let other =
                commands.run("/usr/bin/defaults", &["read", "com.apple.finder", "ShowPathbar"], &CancelToken::new());

            assert!(matches!(write, Err(CommandRunError::NotInstalled { .. })), "{write:?}");
            assert!(matches!(other, Err(CommandRunError::NotInstalled { .. })), "{other:?}");
            assert_eq!(open.calls(), Vec::<String>::new());
            assert_eq!(world.unanswered().len(), 2);
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{CancelToken, CommandRunner};
        use fresher_macos::OPEN;
        use fresher_testing::FakeCommandRunner;

        use super::commands;

        #[test]
        fn open_goes_to_the_real_runner() {
            let open = Arc::new(FakeCommandRunner::default().answering(&format!("{OPEN} -R /box/home/a"), ""));
            let (commands, world) = commands(&open);

            let answer = commands.run(OPEN, &["-R", "/box/home/a"], &CancelToken::new());

            assert_eq!(answer.map(|output| output.status), Ok(0));
            assert_eq!(open.calls(), [format!("{OPEN} -R /box/home/a")]);
            assert_eq!(world.unanswered(), Vec::<String>::new());
        }

        #[test]
        fn the_exact_trash_setting_read_goes_to_the_real_runner() {
            let call = "/usr/bin/defaults read com.apple.finder FXRemoveOldTrashItems";
            let open = Arc::new(FakeCommandRunner::default().answering(call, "1\n"));
            let (commands, world) = commands(&open);

            let args = ["read", "com.apple.finder", "FXRemoveOldTrashItems"];
            let answer = commands.run("/usr/bin/defaults", &args, &CancelToken::new());

            assert_eq!(answer.map(|output| output.status), Ok(0));
            assert_eq!(open.calls(), [call]);
            assert_eq!(world.unanswered(), Vec::<String>::new());
        }
    }
}
