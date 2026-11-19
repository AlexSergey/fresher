//! UTC date and time of a `SystemTime`, for the file names and the record.

use std::time::{Duration, SystemTime};

/// A UTC instant in whole seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Utc {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

/// The UTC instant of `time`; a time before the epoch is the epoch.
pub(crate) fn utc(time: SystemTime) -> Utc {
    let secs = time.duration_since(SystemTime::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    let of_day = secs % 86_400;
    Utc {
        year,
        month,
        day,
        hour: u32::try_from(of_day / 3600).unwrap_or(0),
        minute: u32::try_from(of_day % 3600 / 60).unwrap_or(0),
        second: u32::try_from(of_day % 60).unwrap_or(0),
    }
}

/// The instant `secs` whole seconds after the epoch.
pub(crate) fn at_secs(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

/// The proleptic Gregorian date of a day count since 1970-01-01 (eras of 400 years, March-based years).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = u32::try_from(day_of_year - (153 * month_index + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if month_index < 10 { month_index + 3 } else { month_index - 9 }).unwrap_or(1);
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

impl Utc {
    /// `YYYY-MM-DD_HH-MM-SS`, safe in a file name.
    pub(crate) fn stamp(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    /// `YYYY-MM-DDTHH:MM:SSZ`.
    pub(crate) fn iso(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    /// `YYYY-MM-DD`.
    pub(crate) fn date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::time::{Duration, SystemTime};

        use crate::dev_runs::clock::utc;

        #[test]
        fn a_time_before_the_epoch_is_the_epoch() {
            let before = SystemTime::UNIX_EPOCH - Duration::from_secs(5);

            assert_eq!(utc(before).iso(), "1970-01-01T00:00:00Z");
        }
    }

    mod positive {
        use crate::dev_runs::clock::{at_secs, utc};

        #[test]
        fn known_instants_are_written_in_utc() {
            for (secs, iso) in [
                (0, "1970-01-01T00:00:00Z"),
                (951_782_400, "2000-02-29T00:00:00Z"),
                (1_763_633_071, "2025-11-20T10:04:31Z"),
                (4_107_542_399, "2100-02-28T23:59:59Z"),
            ] {
                assert_eq!(utc(at_secs(secs)).iso(), iso, "{secs}");
            }
        }

        #[test]
        fn the_stamp_has_no_colon() {
            let instant = utc(at_secs(1_763_633_071));

            assert_eq!(instant.stamp(), "2025-11-20_10-04-31");
            assert_eq!(instant.date(), "2025-11-20");
        }
    }
}
