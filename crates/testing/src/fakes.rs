use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime};

use fresher_core::{Clock, CommandOutput, CommandRunError, CommandRunner, Trash, TrashError};

/// Locks `mutex`; a test that panicked while holding it leaves the data as it was.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A clock that stands still until a test moves it.
#[derive(Debug)]
pub struct FakeClock {
    now: Mutex<SystemTime>,
}

impl FakeClock {
    /// A clock at `seconds` after the Unix epoch.
    pub fn at(seconds: u64) -> Self {
        Self { now: Mutex::new(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)) }
    }

    pub fn advance(&self, by: Duration) {
        *lock(&self.now) += by;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        *lock(&self.now)
    }
}

/// A Trash inside the fixture tree: items are moved (renamed) into its folder, never deleted.
#[derive(Debug)]
pub struct FakeTrash {
    dir: PathBuf,
    moved: Mutex<Vec<PathBuf>>,
    refused: HashSet<PathBuf>,
}

impl FakeTrash {
    /// A Trash whose folder is `dir`; it is created on the first move.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), moved: Mutex::new(Vec::new()), refused: HashSet::new() }
    }

    /// Makes the Trash refuse `path`, as macOS does for a file in use or without permission.
    #[must_use]
    pub fn refusing(mut self, path: impl Into<PathBuf>) -> Self {
        self.refused.insert(path.into());
        self
    }

    /// The original paths moved to the Trash, in order.
    pub fn moved(&self) -> Vec<PathBuf> {
        lock(&self.moved).clone()
    }
}

impl Trash for FakeTrash {
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError> {
        if self.refused.contains(path) {
            return Err(TrashError::Refused { path: path.to_path_buf(), reason: "refused by the test".into() });
        }
        let Some(name) = path.file_name() else {
            return Err(TrashError::NotFound { path: path.to_path_buf() });
        };
        if path.symlink_metadata().is_err() {
            return Err(TrashError::NotFound { path: path.to_path_buf() });
        }
        let refused =
            |error: std::io::Error| TrashError::Refused { path: path.to_path_buf(), reason: error.to_string() };
        fs::create_dir_all(&self.dir).map_err(refused)?;
        let mut moved = lock(&self.moved);
        fs::rename(path, self.dir.join(format!("{}-{}", moved.len(), name.to_string_lossy()))).map_err(refused)?;
        moved.push(path.to_path_buf());
        Ok(())
    }
}

/// Answers external commands from a table, and records every call.
#[derive(Debug, Default)]
pub struct FakeCommandRunner {
    answers: HashMap<String, Result<CommandOutput, CommandRunError>>,
    calls: Mutex<Vec<String>>,
}

impl FakeCommandRunner {
    /// Answers `command` (the program and its arguments joined by spaces) with a successful output.
    #[must_use]
    pub fn answering(mut self, command: &str, stdout: &str) -> Self {
        let output = CommandOutput { status: 0, stdout: stdout.into(), stderr: String::new() };
        self.answers.insert(command.into(), Ok(output));
        self
    }

    /// Answers `command` with an error.
    #[must_use]
    pub fn failing(mut self, command: &str, error: CommandRunError) -> Self {
        self.answers.insert(command.into(), Err(error));
        self
    }

    /// The commands run so far, in order.
    pub fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CommandRunError> {
        let command = std::iter::once(program).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
        lock(&self.calls).push(command.clone());
        self.answers
            .get(&command)
            .cloned()
            .unwrap_or_else(|| Err(CommandRunError::NotInstalled { program: program.into() }))
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{CommandRunError, CommandRunner, Trash, TrashError};

        use crate::{FakeCommandRunner, FakeTrash, FixtureTree};

        #[test]
        fn the_trash_reports_a_missing_path() {
            let tree = FixtureTree::from_toml_str("").unwrap();
            let trash = FakeTrash::new(tree.root().join(".Trash"));

            let error = trash.move_to_trash(&tree.root().join("missing")).unwrap_err();

            assert!(matches!(error, TrashError::NotFound { .. }));
            assert_eq!(trash.moved(), Vec::<std::path::PathBuf>::new());
        }

        #[test]
        fn the_trash_reports_a_path_without_a_name() {
            let trash = FakeTrash::new("/nonexistent/.Trash");

            assert!(matches!(trash.move_to_trash(std::path::Path::new("/")), Err(TrashError::NotFound { .. })));
        }

        #[test]
        fn the_trash_refuses_a_path_it_was_told_to() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"locked.db\"\n").unwrap();
            let locked = tree.root().join("locked.db");
            let trash = FakeTrash::new(tree.root().join(".Trash")).refusing(&locked);

            assert!(matches!(trash.move_to_trash(&locked), Err(TrashError::Refused { .. })));
            assert!(locked.exists());
        }

        #[test]
        fn an_unknown_command_is_not_installed() {
            let runner = FakeCommandRunner::default();

            let error = runner.run("docker", &["images"]).unwrap_err();

            assert_eq!(error, CommandRunError::NotInstalled { program: "docker".into() });
        }

        #[test]
        fn a_failing_command_returns_its_error() {
            let error = CommandRunError::Failed { program: "docker".into(), reason: "daemon down".into() };
            let runner = FakeCommandRunner::default().failing("docker ps", error.clone());

            assert_eq!(runner.run("docker", &["ps"]), Err(error));
        }
    }

    mod positive {
        use std::time::{Duration, SystemTime};

        use fresher_core::{Clock, CommandRunner, Trash};

        use crate::{FakeClock, FakeCommandRunner, FakeTrash, FixtureTree};

        #[test]
        fn the_clock_stands_still_until_advanced() {
            let clock = FakeClock::at(100);
            assert_eq!(clock.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(100));

            clock.advance(Duration::from_secs(5));

            assert_eq!(clock.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(105));
        }

        #[test]
        fn the_trash_moves_items_into_its_folder() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Library/Caches/a.bin\"\nsize = 10\n").unwrap();
            let item = tree.root().join("Library/Caches/a.bin");
            let trash = FakeTrash::new(tree.root().join(".Trash"));

            trash.move_to_trash(&item).unwrap();

            assert!(!item.exists());
            assert!(tree.root().join(".Trash/0-a.bin").exists());
            assert_eq!(trash.moved(), [item]);
        }

        #[test]
        fn a_known_command_answers_and_is_recorded() {
            let runner = FakeCommandRunner::default().answering("docker images --format json", "[]");

            let output = runner.run("docker", &["images", "--format", "json"]).unwrap();

            assert_eq!(output.stdout, "[]");
            assert_eq!(runner.calls(), ["docker images --format json"]);
        }
    }
}
