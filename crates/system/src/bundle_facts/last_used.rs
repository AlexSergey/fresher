//! Spotlight's last-used date of a bundle, read with `mdls`.

use std::path::Path;

use fresher_core::{CancelToken, CommandRunner, Timestamp};

/// Spotlight's metadata command, run by its full path, without a shell.
const MDLS: &str = "/usr/bin/mdls";

/// The attribute Spotlight keeps the last use of an app in.
const LAST_USED: &str = "kMDItemLastUsedDate";

/// The date Spotlight last saw the app at `bundle` used: `mdls -raw -name kMDItemLastUsedDate <bundle>`, through
/// `runner` (its time limit applies). `None` when Spotlight does not know (`(null)`), answers anything but one date in
/// UTC, fails or is stopped, or when the path is not an absolute UTF-8 path (an argument it cannot be given safely).
pub(crate) fn last_used(runner: &dyn CommandRunner, bundle: &Path, cancel: &CancelToken) -> Option<Timestamp> {
    let Some(path) = bundle.to_str().filter(|_| bundle.is_absolute()) else {
        tracing::debug!("a bundle's path cannot be passed to Spotlight");
        return None;
    };
    match runner.run(MDLS, &["-raw", "-name", LAST_USED, path], cancel) {
        Ok(output) if output.status == 0 => {
            let date = parse_utc(output.stdout.trim());
            if date.is_none() {
                tracing::debug!("Spotlight knows no last-used date of a bundle");
            }
            date
        }
        Ok(output) => {
            tracing::debug!(status = output.status, "Spotlight's last-used date could not be read");
            None
        }
        Err(error) => {
            tracing::debug!(%error, "Spotlight's last-used date could not be read");
            None
        }
    }
}

/// `YYYY-MM-DD HH:MM:SS +0000`, the form `mdls -raw` writes a date in, as seconds since 1970; anything else is `None`.
fn parse_utc(text: &str) -> Option<Timestamp> {
    let (date, rest) = text.split_once(' ')?;
    let (time, zone) = rest.split_once(' ')?;
    if zone != "+0000" {
        return None;
    }
    let [year, month, day] = fields(date, '-', [4, 2, 2])?;
    let [hour, minute, second] = fields(time, ':', [2, 2, 2])?;
    if year < 1970 || !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let days = (1970..year).map(days_in_year).sum::<u64>()
        + (1..month).map(|earlier| days_in_month(year, earlier)).sum::<u64>()
        + day
        - 1;
    Some(Timestamp::from_secs(((days * 24 + hour) * 60 + minute) * 60 + second))
}

/// Three numbers of exactly `widths` ASCII digits, separated by `separator`.
fn fields(text: &str, separator: char, widths: [usize; 3]) -> Option<[u64; 3]> {
    let mut parts = text.split(separator);
    let mut numbers = [0; 3];
    for (number, width) in numbers.iter_mut().zip(widths) {
        let part = parts.next()?;
        if part.len() != width || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        *number = part.parse().ok()?;
    }
    parts.next().is_none().then_some(numbers)
}

const fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

const fn days_in_year(year: u64) -> u64 {
    if is_leap(year) { 366 } else { 365 }
}

const fn days_in_month(year: u64, month: u64) -> u64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    const COMMAND: &str = "/usr/bin/mdls -raw -name kMDItemLastUsedDate /fixture/Applications/Tool.app";

    mod negative {
        use std::path::Path;

        use fresher_core::{CancelToken, CommandRunError};
        use fresher_testing::FakeCommandRunner;

        use super::COMMAND;
        use crate::bundle_facts::last_used::{last_used, parse_utc};

        fn answered(stdout: &str) -> Option<fresher_core::Timestamp> {
            let runner = FakeCommandRunner::default().answering(COMMAND, stdout);
            last_used(&runner, Path::new("/fixture/Applications/Tool.app"), &CancelToken::new())
        }

        #[test]
        fn null_is_no_date() {
            assert_eq!(answered("(null)"), None);
            assert_eq!(answered(""), None);
        }

        #[test]
        fn garbage_is_no_date() {
            for text in [
                "2026-11-20 14:03:05",
                "2026-11-20 14:03:05 +0100",
                "2026-11-20T14:03:05 +0000",
                "2026-13-20 14:03:05 +0000",
                "2026-02-29 14:03:05 +0000",
                "2026-11-00 14:03:05 +0000",
                "2026-11-20 24:03:05 +0000",
                "2026-11-20 14:60:05 +0000",
                "2026-11-20 14:03:60 +0000",
                "1969-12-31 23:59:59 +0000",
                "26-11-20 14:03:05 +0000",
                "2026-11-20 14:03:05 +0000 extra",
                "2026-11-20-01 14:03:05 +0000",
                "+026-11-20 14:03:05 +0000",
                "(\n    \"2026-11-20 14:03:05 +0000\"\n)",
            ] {
                assert_eq!(answered(text), None, "{text:?}");
                assert_eq!(parse_utc(text), None, "{text:?}");
            }
        }

        #[test]
        fn a_failed_run_is_no_date() {
            let path = Path::new("/fixture/Applications/Tool.app");
            let failing = FakeCommandRunner::default().failing(
                COMMAND,
                CommandRunError::Failed {
                    program: "/usr/bin/mdls".into(),
                    reason: "it ran longer than 60000 ms and was stopped".into(),
                },
            );
            let exiting = FakeCommandRunner::default().exiting(COMMAND, 1);
            let missing = FakeCommandRunner::default();
            for runner in [failing, exiting, missing] {
                assert_eq!(last_used(&runner, path, &CancelToken::new()), None);
            }
            let cancel = CancelToken::new();
            cancel.cancel();
            let answering = FakeCommandRunner::default().answering(COMMAND, "2026-11-20 14:03:05 +0000");
            assert_eq!(last_used(&answering, path, &cancel), None);
        }

        #[test]
        fn a_relative_path_is_never_passed() {
            let runner = FakeCommandRunner::default()
                .answering("/usr/bin/mdls -raw -name kMDItemLastUsedDate -Tool.app", "2026-11-20 14:03:05 +0000");

            assert_eq!(last_used(&runner, Path::new("-Tool.app"), &CancelToken::new()), None);
            assert_eq!(runner.calls(), Vec::<String>::new());
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Mutex;

        use fresher_core::{CancelToken, CommandOutput, CommandRunError, CommandRunner, Timestamp};
        use fresher_testing::FakeCommandRunner;

        use super::COMMAND;
        use crate::bundle_facts::last_used::{last_used, parse_utc};

        #[test]
        fn a_spotlight_date_is_read() {
            let runner = FakeCommandRunner::default().answering(COMMAND, "2026-11-20 14:03:05 +0000");

            let date = last_used(&runner, Path::new("/fixture/Applications/Tool.app"), &CancelToken::new());

            assert_eq!(date, Some(Timestamp::from_secs(1_795_183_385)));
        }

        #[test]
        fn dates_are_counted_from_1970_in_utc() {
            assert_eq!(parse_utc("1970-01-01 00:00:00 +0000"), Some(Timestamp::from_secs(0)));
            assert_eq!(parse_utc("2000-02-29 12:00:00 +0000"), Some(Timestamp::from_secs(951_825_600)));
            assert_eq!(parse_utc("2024-12-31 23:59:59 +0000"), Some(Timestamp::from_secs(1_735_689_599)));
        }

        /// Records each program and its arguments, one by one, and answers `(null)`.
        #[derive(Default)]
        struct Recording(Mutex<Vec<Vec<String>>>);

        impl CommandRunner for Recording {
            fn run(&self, program: &str, args: &[&str], _: &CancelToken) -> Result<CommandOutput, CommandRunError> {
                let call = std::iter::once(program).chain(args.iter().copied()).map(String::from).collect();
                self.0.lock().unwrap().push(call);
                Ok(CommandOutput { status: 0, stdout: "(null)".into(), stderr: String::new() })
            }
        }

        #[test]
        fn the_arguments_are_fixed() {
            let runner = Recording::default();

            let date = last_used(&runner, Path::new("/fixture/Applications/My Tool.app"), &CancelToken::new());

            assert_eq!(date, None);
            assert_eq!(
                *runner.0.lock().unwrap(),
                [["/usr/bin/mdls", "-raw", "-name", "kMDItemLastUsedDate", "/fixture/Applications/My Tool.app"]]
            );
        }
    }
}
