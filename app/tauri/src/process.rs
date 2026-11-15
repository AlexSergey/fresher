//! The real `CommandRunner`: runs a program with a fixed argument list, never through a shell, and kills it when it
//! runs past its time. The port takes no cancel token, so a run that is not timed out cannot be cancelled early.

use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use fresher_core::{CommandOutput, CommandRunError, CommandRunner};

/// How long a program may run before it is killed.
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

/// How often a running program is checked.
const POLL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub(crate) struct ProcessCommandRunner {
    timeout: Duration,
}

impl ProcessCommandRunner {
    pub(crate) fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CommandRunError> {
        let failed = |reason: String| CommandRunError::Failed { program: program.into(), reason };
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => CommandRunError::NotInstalled { program: program.into() },
                _ => failed(error.to_string()),
            })?;
        // The pipes are drained while the program runs, so a program that writes a lot never blocks on a full pipe.
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    kill(&mut child);
                    // The readers are left to end on their own: a child of the program may still hold the pipes.
                    return Err(failed(format!("it ran longer than {} ms and was stopped", self.timeout.as_millis())));
                }
                Ok(None) => thread::sleep(POLL),
                Err(error) => {
                    kill(&mut child);
                    return Err(failed(error.to_string()));
                }
            }
        };
        Ok(CommandOutput {
            // A program ended by a signal has no exit code.
            status: status.code().unwrap_or(-1),
            stdout: collect(stdout),
            stderr: collect(stderr),
        })
    }
}

/// Ends the program and reaps it; a program that already ended needs nothing more.
fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Reads a pipe to its end on a thread of its own.
fn drain(pipe: Option<impl Read + Send + 'static>) -> Option<JoinHandle<Vec<u8>>> {
    let mut pipe = pipe?;
    thread::Builder::new()
        .name("fresher-command-output".into())
        .spawn(move || {
            let mut bytes = Vec::new();
            // What was read before a read error is kept; the exit status tells whether the program succeeded.
            let _ = pipe.read_to_end(&mut bytes);
            bytes
        })
        .ok()
}

fn collect(reader: Option<JoinHandle<Vec<u8>>>) -> String {
    let bytes = reader.and_then(|reader| reader.join().ok()).unwrap_or_default();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::time::{Duration, Instant};

        use fresher_core::{CommandRunError, CommandRunner};

        use crate::process::{COMMAND_TIMEOUT, ProcessCommandRunner};

        #[test]
        fn a_missing_program_is_not_installed() {
            let program = "/nonexistent/fresher-no-such-program";

            let error = ProcessCommandRunner::new(COMMAND_TIMEOUT).run(program, &[]);

            assert_eq!(error, Err(CommandRunError::NotInstalled { program: program.into() }));
        }

        #[test]
        fn a_program_that_cannot_be_started_fails() {
            // A folder is not a program: the system refuses to start it.
            let error = ProcessCommandRunner::new(COMMAND_TIMEOUT).run("/", &[]);

            assert!(matches!(error, Err(CommandRunError::Failed { .. })), "{error:?}");
        }

        #[test]
        fn a_program_that_runs_past_its_time_is_stopped() {
            let started = Instant::now();

            let error = ProcessCommandRunner::new(Duration::from_millis(50)).run("/bin/sleep", &["5"]);

            assert_eq!(
                error,
                Err(CommandRunError::Failed {
                    program: "/bin/sleep".into(),
                    reason: "it ran longer than 50 ms and was stopped".into(),
                })
            );
            assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
        }
    }

    mod positive {
        use fresher_core::CommandRunner;

        use crate::process::{COMMAND_TIMEOUT, ProcessCommandRunner};

        #[test]
        fn runs_a_program_with_its_arguments() {
            let output = ProcessCommandRunner::new(COMMAND_TIMEOUT).run("/bin/echo", &["fresher", "test"]).unwrap();

            assert_eq!((output.status, output.stdout.as_str(), output.stderr.as_str()), (0, "fresher test\n", ""));
        }

        #[test]
        fn reads_an_output_larger_than_a_pipe_holds() {
            let output = ProcessCommandRunner::new(COMMAND_TIMEOUT)
                .run("/bin/dd", &["if=/dev/zero", "bs=1024", "count=512"])
                .unwrap();

            assert_eq!((output.status, output.stdout.len()), (0, 512 * 1024));
            assert!(output.stderr.contains("512+0 records in"), "{}", output.stderr);
        }
    }
}
