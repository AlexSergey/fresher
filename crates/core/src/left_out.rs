//! What a module recognized and chose not to offer, and the engine's sums of it.

use std::cmp::Ordering;
use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

use crate::{ByteSize, CategoryId, Sizes, Text, TextKey};

/// How many items of one for-now count the engine keeps to show, the largest first; the rest are only counted.
pub const LEFT_OUT_SHOWN: usize = 10;

/// An item a module recognized and did not offer, reported with `FindingSink::left_out`. Left-outs of one category
/// and one reason add up into one count; the largest of a for-now reason are kept to be shown by name and path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftOutItem {
    pub category: CategoryId,
    /// Why it was not offered: one of the problems the module declares (`ModuleDescriptor::with_problems`); its kind
    /// is the module's (`ModuleDescriptor::with_kept`).
    pub reason: TextKey,
    /// What the user would have freed.
    pub sizes: Sizes,
    /// Where it is.
    pub path: PathBuf,
    /// Its name, as a finding of it would be titled.
    pub title: Text,
}

/// Why a module keeps what it left out: for good, or only while something passing holds it back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum LeftOutKind {
    /// The item is in use, or never a candidate by its nature (the version you use, a socket, the system's file):
    /// nothing is to be done and the next scan keeps it too. The Scan screen does not show it.
    Kept,
    /// The item would be offered, but a passing condition holds it back (an app is open, it changed lately, a list
    /// could not be read): each item is shown with what to do.
    ForNow,
}

/// One item of a for-now count, as the UI lists it: its name, its path and its size.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LeftOutEntry {
    pub title: Text,
    #[ts(as = "String")]
    pub path: PathBuf,
    pub sizes: Sizes,
}

/// The engine's sum of the left-outs of one category and one reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LeftOutCount {
    pub category: CategoryId,
    pub reason: Text,
    pub kind: LeftOutKind,
    pub items: u32,
    pub sizes: Sizes,
    /// The largest items of a for-now count, at most `LEFT_OUT_SHOWN`, the largest first, then by path; none of a
    /// kept one. `items` counts the rest too.
    pub shown: Vec<LeftOutEntry>,
}

impl LeftOutCount {
    /// No item yet of `category` left out for `reason`, of `kind`.
    pub const fn new(category: CategoryId, reason: Text, kind: LeftOutKind) -> Self {
        Self { category, reason, kind, items: 0, sizes: Sizes::ZERO, shown: Vec::new() }
    }

    /// Counts one more item: the count and the sizes saturate instead of overflowing, and the weakest completeness is
    /// kept. A for-now item is kept to be shown while it is among the `LEFT_OUT_SHOWN` largest.
    pub fn add(&mut self, item: LeftOutItem) {
        self.items = self.items.saturating_add(1);
        let sizes = item.sizes;
        let saturating = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
        self.sizes = Sizes {
            logical: saturating(self.sizes.logical, sizes.logical),
            allocated: saturating(self.sizes.allocated, sizes.allocated),
            reclaimable: saturating(self.sizes.reclaimable, sizes.reclaimable),
            completeness: self.sizes.completeness.max(sizes.completeness),
        };
        if self.kind == LeftOutKind::ForNow {
            self.keep(LeftOutEntry { title: item.title, path: item.path, sizes });
        }
    }

    fn keep(&mut self, entry: LeftOutEntry) {
        let at = self.shown.partition_point(|shown| shown_order(shown, &entry) != Ordering::Greater);
        if at < LEFT_OUT_SHOWN {
            self.shown.insert(at, entry);
            self.shown.truncate(LEFT_OUT_SHOWN);
        }
    }
}

/// The larger first, then by path, so the same scan always shows the same items.
fn shown_order(a: &LeftOutEntry, b: &LeftOutEntry) -> Ordering {
    b.sizes.allocated.cmp(&a.sizes.allocated).then_with(|| a.path.cmp(&b.path))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{ByteSize, CategoryId, LeftOutCount, LeftOutItem, LeftOutKind, ModuleId, Sizes, Text, TextKey};

    fn count(kind: LeftOutKind) -> LeftOutCount {
        let category = CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap();
        LeftOutCount::new(category, Text::new(TextKey::new("demo.left-out.open").unwrap()), kind)
    }

    fn item(path: &str, sizes: Sizes) -> LeftOutItem {
        LeftOutItem {
            category: CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap(),
            reason: TextKey::new("demo.left-out.open").unwrap(),
            sizes,
            path: PathBuf::from(path),
            title: Text::new(TextKey::new("demo.finding.title").unwrap()),
        }
    }

    fn of(bytes: u64) -> Sizes {
        Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes))
    }

    fn shown_paths(count: &LeftOutCount) -> Vec<&str> {
        count.shown.iter().map(|entry| entry.path.to_str().unwrap()).collect()
    }

    mod negative {
        use super::{count, item, of, shown_paths};
        use crate::{ByteSize, LEFT_OUT_SHOWN, LeftOutKind, Sizes};

        #[test]
        fn a_left_out_count_never_overflows() {
            let mut sum = count(LeftOutKind::ForNow);
            sum.items = u32::MAX;
            let huge = Sizes::complete(ByteSize::new(u64::MAX), ByteSize::new(u64::MAX));

            sum.add(item("/a", huge));
            sum.add(item("/b", huge));

            assert_eq!(sum.items, u32::MAX);
            assert_eq!(sum.sizes, huge);
        }

        #[test]
        fn a_kept_count_shows_no_item() {
            let mut sum = count(LeftOutKind::Kept);

            sum.add(item("/a", of(1)));

            assert_eq!((sum.items, sum.shown.len()), (1, 0));
        }

        #[test]
        fn a_for_now_count_shows_only_its_largest_items() {
            let mut sum = count(LeftOutKind::ForNow);

            for bytes in 1..=u64::try_from(LEFT_OUT_SHOWN).unwrap() + 5 {
                sum.add(item(&format!("/{bytes:02}"), of(bytes)));
            }

            assert_eq!(sum.items, 15);
            assert_eq!(sum.shown.len(), LEFT_OUT_SHOWN);
            assert_eq!(shown_paths(&sum).first(), Some(&"/15"));
            assert_eq!(shown_paths(&sum).last(), Some(&"/06"));
        }
    }

    mod positive {
        use super::{count, item, of, shown_paths};
        use crate::{ByteSize, Completeness, LeftOutKind, Sizes};

        #[test]
        fn left_out_counts_add_items_and_sizes() {
            let mut sum = count(LeftOutKind::Kept);
            let partial =
                Sizes { completeness: Completeness::Partial, ..Sizes::complete(ByteSize::new(2), ByteSize::new(4)) };

            sum.add(item("/a", Sizes::new(ByteSize::new(1), ByteSize::new(10), ByteSize::new(5))));
            sum.add(item("/b", partial));

            assert_eq!(sum.items, 2);
            assert_eq!(sum.sizes.logical, ByteSize::new(3));
            assert_eq!(sum.sizes.allocated, ByteSize::new(14));
            assert_eq!(sum.sizes.reclaimable, ByteSize::new(9));
            assert_eq!(sum.sizes.completeness, Completeness::Partial);
        }

        #[test]
        fn a_for_now_count_shows_its_items_largest_first_then_by_path() {
            let mut sum = count(LeftOutKind::ForNow);

            sum.add(item("/small", of(1)));
            sum.add(item("/b", of(5)));
            sum.add(item("/a", of(5)));

            assert_eq!(shown_paths(&sum), ["/a", "/b", "/small"]);
            assert_eq!(sum.shown[0].title.key().as_str(), "demo.finding.title");
        }
    }
}
