//! The Folders list of the Largest screen: what it asks and is answered. The engine fills these in.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ByteSize, NodeInfo, Sizes, Timestamp};

const DAY: u64 = 86_400;
/// The age in days from which a folder is told in years.
const YEARS_FROM: u64 = 730;

/// One page of the Folders list. Only where to start: the rule takes its size and age from the settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FoldersQuery {
    pub offset: u32,
    /// At most `MAX_PAGE`; the engine caps a larger one.
    pub limit: u32,
}

/// How long nothing in a folder changed, as the row says it: months under two years, then years.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "unit", content = "count", rename_all = "kebab-case")]
pub enum LeftFor {
    Months(u32),
    Years(u32),
}

impl LeftFor {
    /// The time from `newest` to `taken` (the scan's time, never the clock): whole days, none for a `newest` after
    /// `taken`; months of 30 days while under 730 days, then years of 365 days.
    pub fn between(newest: Timestamp, taken: Timestamp) -> Self {
        let days = taken.secs().saturating_sub(newest.secs()) / DAY;
        let count = |unit: u64| u32::try_from(days / unit).unwrap_or(u32::MAX);
        if days < YEARS_FROM { Self::Months(count(30)) } else { Self::Years(count(365)) }
    }
}

/// One folder left alone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FolderRow {
    pub node: NodeInfo,
    /// The folder it is in, as a path from the home, for display (`~`, `~/Code`).
    pub place: String,
    /// The newest change of anything inside it.
    pub newest: Timestamp,
    pub left_for: LeftFor,
    /// It holds a `.git` folder.
    pub project: bool,
}

/// The answer to a `FoldersQuery`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FoldersPage {
    pub rows: Vec<FolderRow>,
    pub offset: u32,
    /// How many folders are listed, in all.
    pub total: u32,
    /// The sizes of every folder listed.
    pub sizes: Sizes,
    /// How many of them are projects.
    pub projects: u32,
    /// The home's sizes, for the bar.
    pub home: Sizes,
    /// The smallest size listed (the setting's value).
    pub min_size: ByteSize,
    /// The days nothing in a listed folder changed, at least (the stale projects' setting).
    pub days: u32,
}

#[cfg(test)]
mod tests {
    use super::{DAY, FoldersQuery, LeftFor};
    use crate::Timestamp;

    fn after_days(days: u64) -> LeftFor {
        LeftFor::between(Timestamp::from_secs(1_000_000), Timestamp::from_secs(1_000_000 + days * DAY))
    }

    mod negative {
        use super::{FoldersQuery, LeftFor, Timestamp};

        #[test]
        fn an_unknown_field_of_a_folders_query_is_refused() {
            let query = serde_json::from_str::<FoldersQuery>(r#"{"offset":0,"limit":10,"place":1}"#);

            assert!(query.is_err());
        }

        #[test]
        fn a_newest_after_the_scan_is_zero_months() {
            let newest = Timestamp::from_secs(5_000_000);

            assert_eq!(LeftFor::between(newest, Timestamp::from_secs(1_000)), LeftFor::Months(0));
        }
    }

    mod positive {
        use super::{LeftFor, after_days};

        #[test]
        fn under_two_years_counts_months() {
            assert_eq!(after_days(29), LeftFor::Months(0));
            assert_eq!(after_days(400), LeftFor::Months(13));
            assert_eq!(after_days(729), LeftFor::Months(24));
        }

        #[test]
        fn from_two_years_counts_years() {
            assert_eq!(after_days(730), LeftFor::Years(2));
            assert_eq!(after_days(1_100), LeftFor::Years(3));
        }

        #[test]
        fn left_for_serializes_as_the_ui_reads_it() {
            let json = serde_json::to_string(&LeftFor::Months(14)).unwrap();

            assert_eq!(json, r#"{"unit":"months","count":14}"#);
        }
    }
}
