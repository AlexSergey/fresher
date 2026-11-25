//! Whether a program is signed by a given developer: the system's own `codesign` checks the signature against a
//! requirement, so a look-alike bundle (any bundle id is free to write) never passes for the real one.

use std::path::Path;

use fresher_core::{CancelToken, CommandRunner, TeamId};

/// The system's code signing tool, run by its absolute path.
const CODESIGN: &str = "/usr/bin/codesign";

/// The requirement of a program signed by `team` under a certificate Apple issued.
fn requirement(team: &TeamId) -> String {
    format!("-R=anchor apple generic and certificate leaf[subject.OU] = \"{}\"", team.as_str())
}

/// Whether `codesign --verify --strict` finds a valid signature of `team` on the program at `file`: only its exit
/// status 0 is a yes; any other status, a failure to run, a cancel or a path that is not UTF-8 is a no.
pub(crate) fn signed_by(runner: &dyn CommandRunner, file: &Path, team: &TeamId, cancel: &CancelToken) -> bool {
    let Some(file) = file.to_str() else { return false };
    let requirement = requirement(team);
    let args = ["--verify", "--strict", requirement.as_str(), file];
    runner.run(CODESIGN, &args, cancel).is_ok_and(|output| output.status == 0)
}

#[cfg(test)]
mod tests {
    use fresher_core::TeamId;

    fn team() -> TeamId {
        TeamId::new("HUAQ24HBR6").unwrap()
    }

    /// The call `signed_by` makes for `file`, as the fake runner keys it.
    fn call(file: &str) -> String {
        format!(
            "/usr/bin/codesign --verify --strict -R=anchor apple generic and certificate leaf[subject.OU] = \
             \"HUAQ24HBR6\" {file}"
        )
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;
        use std::path::Path;

        use fresher_core::{CancelToken, CommandRunError};
        use fresher_testing::FakeCommandRunner;

        use super::{call, team};
        use crate::ProcessCommandRunner;
        use crate::code_check::signed_by;

        #[test]
        fn a_program_the_check_refuses_is_not_signed() {
            let runner = FakeCommandRunner::default().exiting(&call("/Apps/Docker.app/docker"), 3);

            assert!(!signed_by(&runner, Path::new("/Apps/Docker.app/docker"), &team(), &CancelToken::new()));
        }

        #[test]
        fn a_check_that_cannot_run_or_is_cancelled_is_not_signed() {
            let failed = CommandRunError::NotInstalled { program: "/usr/bin/codesign".into() };
            let runner = FakeCommandRunner::default().failing(&call("/Apps/x"), failed);
            let cancel = CancelToken::new();
            cancel.cancel();

            assert!(!signed_by(&runner, Path::new("/Apps/x"), &team(), &CancelToken::new()));
            assert!(!signed_by(&FakeCommandRunner::default(), Path::new("/Apps/x"), &team(), &cancel));
        }

        #[test]
        fn a_path_that_is_not_utf8_is_not_checked() {
            let runner = FakeCommandRunner::default();

            assert!(!signed_by(&runner, Path::new(OsStr::from_bytes(b"/Apps/\xFF")), &team(), &CancelToken::new()));
            assert_eq!(runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn an_unsigned_program_on_disk_is_not_signed_by_any_team() {
            let temp = tempfile::tempdir().unwrap();
            let program = temp.path().join("docker");
            std::fs::write(&program, b"#!/bin/sh\necho planted\n").unwrap();
            let runner = ProcessCommandRunner::new(crate::COMMAND_TIMEOUT);

            assert!(!signed_by(&runner, &program, &team(), &CancelToken::new()));
            assert!(!signed_by(&runner, Path::new("/bin/ls"), &team(), &CancelToken::new()), "Apple's, not the team's");
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::CancelToken;
        use fresher_testing::FakeCommandRunner;

        use super::{call, team};
        use crate::code_check::signed_by;

        #[test]
        fn a_program_the_check_accepts_is_signed() {
            let runner = FakeCommandRunner::default().answering(&call("/Apps/My App.app/docker"), "");

            assert!(signed_by(&runner, Path::new("/Apps/My App.app/docker"), &team(), &CancelToken::new()));
            assert_eq!(runner.calls(), [call("/Apps/My App.app/docker")]);
        }
    }
}
