//! The rows of the decision and the answers, one test each.

use fresher_core::{ByteSize, CommandOutput, InstalledApp, Sizes};

use crate::decide::{Disk, NoSystem};

fn app() -> InstalledApp {
    fresher_testing::installed_app("dev.kdrag0n.MacVirt", "OrbStack", "2.2.5").unwrap()
}

fn seen() -> Disk {
    Disk::Seen(Sizes::complete(ByteSize::new(8_000_000_000), ByteSize::new(2_000_000_000)))
}

/// A `running` for rows that must not reach it: were it called, the app would run and the verdict would ask.
#[expect(clippy::unnecessary_wraps, reason = "the signature of `decide`'s `running`")]
fn never_asked() -> Result<bool, NoSystem> {
    Ok(true)
}

fn output(status: i32, stdout: &str) -> CommandOutput {
    CommandOutput { status, stdout: stdout.into(), stderr: String::new() }
}

mod negative {
    use fresher_core::CommandRunError;

    use super::{app, never_asked, output, seen};
    use crate::decide::{Cancelled, Disk, NoSystem, State, Verdict, answered, decide};

    #[test]
    fn nothing_is_said_when_the_installed_apps_cannot_be_listed() {
        for disk in [seen(), Disk::Absent, Disk::Unseen] {
            assert_eq!(decide(Err(NoSystem), never_asked, true, disk, false), Verdict::Silent);
        }
    }

    #[test]
    fn nothing_is_said_when_the_app_and_its_disk_are_absent() {
        assert_eq!(decide(Ok(None), never_asked, true, Disk::Absent, false), Verdict::Silent);
        assert_eq!(decide(Ok(None), never_asked, true, Disk::Unseen, false), Verdict::Silent);
    }

    #[test]
    fn a_restored_scan_never_asks() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), never_asked, true, seen(), true), Verdict::Describe(State::Restored));
        assert_eq!(decide(Ok(Some(&app)), never_asked, true, Disk::Absent, true), Verdict::Describe(State::Restored));
    }

    #[test]
    fn it_never_asks_when_the_running_apps_cannot_be_listed() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), || Err(NoSystem), true, seen(), false), Verdict::Describe(State::Unknown));
    }

    #[test]
    fn a_closed_app_is_never_asked() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), || Ok(false), true, seen(), false), Verdict::Describe(State::Stopped));
    }

    #[test]
    fn an_open_app_without_its_socket_is_never_asked() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), || Ok(true), false, seen(), false), Verdict::Describe(State::Stopped));
    }

    #[test]
    fn a_stale_socket_alone_is_never_asked() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), || Ok(false), true, Disk::Absent, false), Verdict::Describe(State::Stopped));
        assert_eq!(decide(Ok(None), never_asked, true, Disk::Absent, false), Verdict::Silent);
    }

    #[test]
    fn a_failed_run_is_no_answer() {
        let not_installed = CommandRunError::NotInstalled { program: "docker".into() };
        let failed = CommandRunError::Failed { program: "docker".into(), reason: "timed out".into() };

        assert_eq!(answered(Err(not_installed)), Ok(State::NoAnswer));
        assert_eq!(answered(Err(failed)), Ok(State::NoAnswer));
    }

    #[test]
    fn a_non_zero_status_is_no_answer() {
        assert_eq!(answered(Ok(output(1, ""))), Ok(State::NoAnswer));
        assert_eq!(answered(Ok(output(-1, "{}"))), Ok(State::NoAnswer));
    }

    #[test]
    fn a_cancelled_run_is_cancelled() {
        assert_eq!(answered(Err(CommandRunError::Cancelled { program: "docker".into() })), Err(Cancelled));
    }

    #[test]
    fn an_answer_that_does_not_read_is_unreadable() {
        assert_eq!(answered(Ok(output(0, "TYPE TOTAL ACTIVE SIZE RECLAIMABLE"))), Ok(State::Unreadable));
    }
}

mod positive {
    use std::cell::Cell;

    use proptest::prelude::*;

    use super::{app, never_asked, output, seen};
    use crate::answer::parse;
    use crate::decide::{Disk, NoSystem, State, Verdict, answered, decide};

    const ANSWER: &str = r#"{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}"#;

    #[test]
    fn a_left_disk_file_is_described_as_left() {
        assert_eq!(decide(Ok(None), never_asked, false, seen(), false), Verdict::Describe(State::Left));
        assert_eq!(decide(Ok(None), never_asked, true, seen(), true), Verdict::Describe(State::Left));
    }

    #[test]
    fn an_open_app_with_its_socket_is_asked() {
        let app = app();

        assert_eq!(decide(Ok(Some(&app)), || Ok(true), true, seen(), false), Verdict::Ask);
        assert_eq!(decide(Ok(Some(&app)), || Ok(true), true, Disk::Unseen, false), Verdict::Ask);
    }

    #[test]
    fn a_readable_answer_is_answered() {
        assert_eq!(answered(Ok(output(0, ANSWER))), Ok(State::Answered(parse(ANSWER).unwrap())));
    }

    #[test]
    fn every_state_has_its_summary_key() {
        let usage = parse(ANSWER).unwrap();
        let keys: Vec<&str> = [
            State::Left,
            State::Restored,
            State::Unknown,
            State::Stopped,
            State::NoAnswer,
            State::Unreadable,
            State::Answered(usage),
        ]
        .iter()
        .map(State::key)
        .collect();

        assert_eq!(
            keys,
            [
                "docker.summary.left",
                "docker.summary.restored",
                "docker.summary.unknown",
                "docker.summary.stopped",
                "docker.summary.no-answer",
                "docker.summary.unreadable",
                "docker.summary.answered",
            ]
        );
    }

    fn disk(index: u8) -> Disk {
        match index {
            0 => seen(),
            1 => Disk::Absent,
            _ => Disk::Unseen,
        }
    }

    proptest! {
        #[test]
        fn a_row_stopped_before_row_5_never_asks_the_running_apps(
            listed in any::<bool>(),
            installed in any::<bool>(),
            socket in any::<bool>(),
            on_disk in 0_u8..3,
            restored in any::<bool>(),
            running in any::<bool>(),
        ) {
            let app = app();
            let apps = if listed { Ok(installed.then_some(&app)) } else { Err(NoSystem) };
            let asked = Cell::new(0);

            let verdict = decide(apps, || { asked.set(asked.get() + 1); Ok(running) }, socket, disk(on_disk), restored);

            let before_row_5 = !listed || !installed || restored;
            prop_assert_eq!(asked.get(), usize::from(!before_row_5));
            if before_row_5 {
                prop_assert_ne!(verdict, Verdict::Ask);
            }
        }
    }
}
