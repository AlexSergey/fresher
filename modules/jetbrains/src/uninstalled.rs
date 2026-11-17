//! The IDEs that are no longer installed (row 4 of the design's section 4): their version folders are gathered while
//! the walk runs and decided in `finish`, once every folder of the scan is seen, by how long ago the newest of them
//! changed. Nothing here reads the disk.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{CancelToken, Sizes, Timestamp};

use crate::name::Version;
use crate::places::Place;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// A version folder of a product that has no installed app, waiting for the scan's other folders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Waiting {
    /// The product's folder name (`DataGrip`).
    pub(crate) product: String,
    pub(crate) version: Version,
    pub(crate) place: Place,
    pub(crate) path: PathBuf,
    pub(crate) sizes: Sizes,
    pub(crate) modified: Option<Timestamp>,
}

/// How long ago the newest folder of a product changed, against the setting's days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Age {
    /// Every folder changed at least the setting's days ago: the product's data is offered.
    Old,
    /// A folder changed within the days, in the future, or at a time the walk could not read: everything is kept.
    Recent,
}

/// `Old` when every one of `modified` is known and at least `days` before `now`; `Recent` otherwise, and for no folder.
pub(crate) fn age(modified: impl IntoIterator<Item = Option<Timestamp>>, now: Timestamp, days: u32) -> Age {
    let threshold = u64::from(days).saturating_mul(SECONDS_PER_DAY);
    let mut seen = false;
    for time in modified {
        seen = true;
        let old = time.is_some_and(|time| time <= now && now.secs() - time.secs() >= threshold);
        if !old {
            return Age::Recent;
        }
    }
    if seen { Age::Old } else { Age::Recent }
}

/// The folders gathered for one scan, kept with the scan's cancel token: a scan cancelled before its `finish` leaves
/// its folders behind, and the next scan, with another token, drops them instead of deciding them.
#[derive(Debug, Default)]
pub(crate) struct Gathered {
    scan: Mutex<Option<(CancelToken, Vec<Waiting>)>>,
}

impl Gathered {
    /// Keeps `folder` for the scan of `token`, dropping what an earlier scan left.
    pub(crate) fn add(&self, token: &CancelToken, folder: Waiting) {
        let mut scan = self.lock();
        match scan.as_mut() {
            Some((kept, folders)) if kept.is_same(token) => folders.push(folder),
            _ => *scan = Some((token.clone(), vec![folder])),
        }
    }

    /// The folders gathered for the scan of `token`, by product, each product's sorted by path; nothing is kept after.
    pub(crate) fn take(&self, token: &CancelToken) -> BTreeMap<String, Vec<Waiting>> {
        let mut by_product: BTreeMap<String, Vec<Waiting>> = BTreeMap::new();
        if let Some((kept, folders)) = self.lock().take()
            && kept.is_same(token)
        {
            for folder in folders {
                by_product.entry(folder.product.clone()).or_default().push(folder);
            }
        }
        for folders in by_product.values_mut() {
            folders.sort_by(|first, second| first.path.cmp(&second.path));
        }
        by_product
    }

    /// The lock, even after a panic while it was held: what it guards is a plain list.
    fn lock(&self) -> MutexGuard<'_, Option<(CancelToken, Vec<Waiting>)>> {
        self.scan.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Sizes, Timestamp};

    use crate::name::Version;
    use crate::places::Place;
    use crate::uninstalled::Waiting;

    const DAY: u64 = 24 * 60 * 60;
    const NOW: u64 = 1_767_225_600;

    /// The time of a folder changed `days` ago.
    fn days_ago(days: u64) -> Option<Timestamp> {
        NOW.checked_sub(days * DAY).map(Timestamp::from_secs)
    }

    fn now() -> Timestamp {
        Timestamp::from_secs(NOW)
    }

    fn waiting(product: &str, path: &str) -> Waiting {
        Waiting {
            product: product.into(),
            version: Version::new(2024, 3).unwrap(),
            place: Place::Caches,
            path: path.into(),
            sizes: Sizes::ZERO,
            modified: days_ago(40),
        }
    }

    mod negative {
        use fresher_core::{CancelToken, Timestamp};

        use super::{DAY, NOW, days_ago, now, waiting};
        use crate::uninstalled::{Age, Gathered, age};

        #[test]
        fn a_folder_changed_within_the_days_keeps_every_folder_of_the_product() {
            assert_eq!(age([days_ago(40), days_ago(29)], now(), 30), Age::Recent);
            assert_eq!(age([Some(Timestamp::from_secs(NOW - 30 * DAY + 1))], now(), 30), Age::Recent);
        }

        #[test]
        fn a_time_that_is_unknown_or_in_the_future_is_recent() {
            assert_eq!(age([days_ago(40), None], now(), 30), Age::Recent);
            assert_eq!(age([days_ago(40), Some(Timestamp::from_secs(NOW + DAY))], now(), 30), Age::Recent);
        }

        #[test]
        fn no_folder_is_recent() {
            assert_eq!(age([], now(), 30), Age::Recent);
        }

        #[test]
        fn a_longer_setting_keeps_what_a_shorter_one_offers() {
            assert_eq!(age([days_ago(40)], now(), 30), Age::Old);
            assert_eq!(age([days_ago(40)], now(), 41), Age::Recent);
        }

        #[test]
        fn a_scan_that_never_finished_leaves_nothing_to_the_next_one() {
            let gathered = Gathered::default();
            let (cancelled, next) = (CancelToken::new(), CancelToken::new());
            gathered.add(&cancelled, waiting("DataGrip", "/a"));

            assert!(gathered.take(&next).is_empty());

            gathered.add(&cancelled, waiting("DataGrip", "/a"));
            gathered.add(&next, waiting("WebStorm", "/b"));
            let taken = gathered.take(&next);
            assert_eq!(taken.keys().collect::<Vec<_>>(), ["WebStorm"]);
        }

        #[test]
        fn nothing_is_kept_once_taken() {
            let gathered = Gathered::default();
            let token = CancelToken::new();
            gathered.add(&token, waiting("DataGrip", "/a"));

            assert_eq!(gathered.take(&token).len(), 1);
            assert!(gathered.take(&token).is_empty());
        }
    }

    mod positive {
        use fresher_core::CancelToken;

        use super::{DAY, NOW, days_ago, now, waiting};
        use crate::uninstalled::{Age, Gathered, age};

        #[test]
        fn every_folder_unchanged_for_the_days_is_old() {
            assert_eq!(age([days_ago(30), days_ago(400)], now(), 30), Age::Old);
            assert_eq!(age([days_ago(1)], now(), 1), Age::Old);
            assert_eq!(age([Some(fresher_core::Timestamp::from_secs(NOW - 2 * DAY))], now(), 2), Age::Old);
        }

        #[test]
        fn the_folders_of_one_scan_come_back_by_product_sorted_by_path() {
            let gathered = Gathered::default();
            let scan = CancelToken::new();
            for (product, path) in [("WebStorm", "/w"), ("DataGrip", "/d/2"), ("DataGrip", "/d/1")] {
                gathered.add(&scan, waiting(product, path));
            }

            let by_product = gathered.take(&scan);

            let paths: Vec<(&str, Vec<&str>)> = by_product
                .iter()
                .map(|(product, folders)| {
                    (product.as_str(), folders.iter().map(|folder| folder.path.to_str().unwrap()).collect())
                })
                .collect();
            assert_eq!(paths, [("DataGrip", vec!["/d/1", "/d/2"]), ("WebStorm", vec!["/w"])]);
        }

        mod properties {
            use fresher_core::Timestamp;
            use proptest::prelude::*;

            use super::super::{DAY, NOW, now};
            use crate::uninstalled::{Age, age};

            proptest! {
                #[test]
                fn old_exactly_when_every_folder_is_unchanged_for_the_days_whatever_the_order(
                    mut ages in prop::collection::vec(prop::option::of(0u64..2000 * DAY), 1..6),
                    days in 1u32..1000,
                ) {
                    let times = |ages: &[Option<u64>]| -> Vec<Option<Timestamp>> {
                        ages.iter().map(|age| age.map(|age| Timestamp::from_secs(NOW - age))).collect()
                    };
                    let expected = if ages.iter().all(|age| age.is_some_and(|age| age >= u64::from(days) * DAY)) {
                        Age::Old
                    } else {
                        Age::Recent
                    };

                    prop_assert_eq!(age(times(&ages), now(), days), expected);
                    ages.reverse();
                    prop_assert_eq!(age(times(&ages), now(), days), expected);
                }
            }
        }
    }
}
