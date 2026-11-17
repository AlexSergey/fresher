//! What a module recognized and chose not to offer, and the engine's sums of it.

use serde::Serialize;
use ts_rs::TS;

use crate::{ByteSize, CategoryId, Sizes, Text, TextKey};

/// An item a module recognized and did not offer, reported with `FindingSink::left_out`. It holds no path and no
/// args: left-outs of one category and one reason add up into one line, and nothing about the item is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftOutItem {
    pub category: CategoryId,
    /// Why it was not offered: one of the problems the module declares (`ModuleDescriptor::with_problems`).
    pub reason: TextKey,
    /// What the user would have freed.
    pub sizes: Sizes,
}

/// The engine's sum of the left-outs of one category and one reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LeftOutCount {
    pub category: CategoryId,
    pub reason: Text,
    pub items: u32,
    pub sizes: Sizes,
}

impl LeftOutCount {
    /// No item yet of `category` left out for `reason`.
    pub const fn new(category: CategoryId, reason: Text) -> Self {
        Self { category, reason, items: 0, sizes: Sizes::ZERO }
    }

    /// Counts one more item of `sizes`: the count and the sizes saturate instead of overflowing, and the weakest
    /// completeness is kept.
    pub fn add(&mut self, sizes: Sizes) {
        self.items = self.items.saturating_add(1);
        let saturating = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
        self.sizes = Sizes {
            logical: saturating(self.sizes.logical, sizes.logical),
            allocated: saturating(self.sizes.allocated, sizes.allocated),
            reclaimable: saturating(self.sizes.reclaimable, sizes.reclaimable),
            completeness: self.sizes.completeness.max(sizes.completeness),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{CategoryId, LeftOutCount, ModuleId, Text, TextKey};

    fn count() -> LeftOutCount {
        let category = CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap();
        LeftOutCount::new(category, Text::new(TextKey::new("demo.left-out.open").unwrap()))
    }

    mod negative {
        use super::count;
        use crate::{ByteSize, Sizes};

        #[test]
        fn a_left_out_count_never_overflows() {
            let mut sum = count();
            sum.items = u32::MAX;
            let huge = Sizes::complete(ByteSize::new(u64::MAX), ByteSize::new(u64::MAX));

            sum.add(huge);
            sum.add(huge);

            assert_eq!(sum.items, u32::MAX);
            assert_eq!(sum.sizes, huge);
        }
    }

    mod positive {
        use super::count;
        use crate::{ByteSize, Completeness, Sizes};

        #[test]
        fn left_out_counts_add_items_and_sizes() {
            let mut sum = count();
            let partial =
                Sizes { completeness: Completeness::Partial, ..Sizes::complete(ByteSize::new(2), ByteSize::new(4)) };

            sum.add(Sizes::new(ByteSize::new(1), ByteSize::new(10), ByteSize::new(5)));
            sum.add(partial);

            assert_eq!(sum.items, 2);
            assert_eq!(sum.sizes.logical, ByteSize::new(3));
            assert_eq!(sum.sizes.allocated, ByteSize::new(14));
            assert_eq!(sum.sizes.reclaimable, ByteSize::new(9));
            assert_eq!(sum.sizes.completeness, Completeness::Partial);
        }
    }
}
