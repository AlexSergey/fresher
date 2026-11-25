//! What one module left out during a scan, added up per category and reason, with the largest items of a for-now
//! reason.

use std::collections::{BTreeMap, BTreeSet};

use fresher_core::{CategoryId, LeftOutCount, LeftOutItem, ModuleDescriptor, Text, TextKey};

/// The sums of one module's left-outs, one per (category, reason), each of the kind the module declares for the
/// reason: bounded by the module's categories and declared problems, and by `LEFT_OUT_SHOWN` items each, never by how
/// many items it left out.
pub(crate) struct LeftOuts<'a> {
    descriptor: &'a ModuleDescriptor,
    counts: BTreeMap<(CategoryId, TextKey), LeftOutCount>,
    /// The bugs already logged, so each is logged once.
    logged: BTreeSet<Bug>,
}

/// A left-out the module should not have reported as it did; logged once per key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Bug {
    /// The reason is not one of the module's declared problems: counted all the same, the user learns why anyway.
    UndeclaredReason(TextKey),
    /// The category is not one of the module's own: dropped, the UI could not place its line.
    ForeignCategory(CategoryId),
}

impl<'a> LeftOuts<'a> {
    pub(crate) fn new(descriptor: &'a ModuleDescriptor) -> Self {
        Self { descriptor, counts: BTreeMap::new(), logged: BTreeSet::new() }
    }

    /// Adds `item` to the sum of its category and reason, unless its category is not the module's. A bug is logged
    /// the first time it is seen, and returned then.
    pub(crate) fn add(&mut self, item: LeftOutItem) -> Option<Bug> {
        let own = self.descriptor.categories().iter().any(|category| category.id() == item.category);
        let bug = if !own {
            Some(Bug::ForeignCategory(item.category))
        } else if !self.descriptor.problems().contains(&item.reason) {
            Some(Bug::UndeclaredReason(item.reason.clone()))
        } else {
            None
        };
        if own {
            let kind = self.descriptor.left_out_kind(&item.reason);
            self.counts
                .entry((item.category, item.reason.clone()))
                .or_insert_with_key(|(category, reason)| LeftOutCount::new(*category, Text::new(reason.clone()), kind))
                .add(item);
        }
        let bug = bug.filter(|bug| self.logged.insert(bug.clone()))?;
        let module = self.descriptor.id().as_str();
        match &bug {
            Bug::UndeclaredReason(reason) => {
                tracing::warn!(
                    module,
                    reason = reason.as_str(),
                    "a module left out an item for a reason it does not declare"
                );
            }
            Bug::ForeignCategory(category) => {
                tracing::warn!(
                    module,
                    category = category.as_str(),
                    "a module left out an item of a category not its own"
                );
            }
        }
        Some(bug)
    }

    /// The sums so far, sorted by category, then by reason key; the module's sums start again from nothing.
    pub(crate) fn take(&mut self) -> Vec<LeftOutCount> {
        std::mem::take(&mut self.counts).into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{
        CategoryDescriptor, CategoryId, DangerLevel, LeftOutItem, Locales, ModuleDescriptor, ModuleId, Sizes, TextKey,
    };

    fn key(key: &str) -> TextKey {
        TextKey::new(key).unwrap()
    }

    fn category(id: &'static str) -> CategoryId {
        CategoryId::new(ModuleId::new("alpha").unwrap(), id).unwrap()
    }

    /// Module `alpha` with two categories that declares `alpha.left-out.open` (for now) and `alpha.left-out.current`
    /// (kept for good).
    fn descriptor() -> ModuleDescriptor {
        let categories =
            ["alpha.caches", "alpha.logs"].map(|id| CategoryDescriptor::new(category(id), DangerLevel::Safe)).to_vec();
        ModuleDescriptor::new(
            ModuleId::new("alpha").unwrap(),
            fresher_core::ModuleGroup::Development,
            categories,
            vec![],
            Locales { en: "{}" },
        )
        .unwrap()
        .with_problems(vec![key("alpha.left-out.open"), key("alpha.left-out.current")])
        .unwrap()
        .with_kept(vec![key("alpha.left-out.current")])
        .unwrap()
    }

    fn item(category_id: &'static str, reason: &str, bytes: u64) -> LeftOutItem {
        let size = fresher_core::ByteSize::new(bytes);
        LeftOutItem {
            category: category(category_id),
            reason: key(reason),
            sizes: Sizes::complete(size, size),
            path: format!("/home/{bytes}").into(),
            title: fresher_core::Text::new(key("alpha.finding.title")),
        }
    }

    mod negative {
        use fresher_core::{CategoryId, LeftOutItem, LeftOutKind, ModuleId};

        use super::{descriptor, item, key};
        use crate::left_outs::{Bug, LeftOuts};

        #[test]
        fn an_undeclared_reason_is_reported_once_and_counted() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);

            let first = left_outs.add(item("alpha.caches", "alpha.left-out.unknown", 1));
            let second = left_outs.add(item("alpha.caches", "alpha.left-out.unknown", 2));
            let declared = left_outs.add(item("alpha.caches", "alpha.left-out.open", 4));

            assert_eq!(first, Some(Bug::UndeclaredReason(key("alpha.left-out.unknown"))));
            assert_eq!((second, declared), (None, None));
            let counts = left_outs.take();
            assert_eq!(counts.iter().map(|count| count.items).collect::<Vec<_>>(), [1, 2]);
        }

        #[test]
        fn a_foreign_category_is_reported_once_and_dropped() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);
            let foreign = CategoryId::new(ModuleId::new("beta").unwrap(), "beta.caches").unwrap();
            let item = LeftOutItem { category: foreign, ..item("alpha.caches", "alpha.left-out.open", 1) };

            let first = left_outs.add(item.clone());
            let second = left_outs.add(item);

            assert_eq!((first, second), (Some(Bug::ForeignCategory(foreign)), None));
            assert_eq!(left_outs.take(), []);
        }

        #[test]
        fn a_reason_kept_for_good_shows_no_item() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);

            left_outs.add(item("alpha.caches", "alpha.left-out.current", 1));
            let counts = left_outs.take();

            assert_eq!(
                counts.iter().map(|count| (count.kind, count.items, count.shown.len())).collect::<Vec<_>>(),
                [(LeftOutKind::Kept, 1, 0)]
            );
        }

        #[test]
        fn taken_sums_start_again_from_nothing() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);
            left_outs.add(item("alpha.caches", "alpha.left-out.open", 1));

            left_outs.take();

            assert_eq!(left_outs.take(), []);
        }
    }

    mod positive {
        use fresher_core::{ByteSize, LeftOutKind};

        use super::{descriptor, item};
        use crate::left_outs::LeftOuts;

        #[test]
        fn one_sum_per_category_and_reason_sorted() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);

            left_outs.add(item("alpha.logs", "alpha.left-out.open", 1));
            left_outs.add(item("alpha.caches", "alpha.left-out.open", 2));
            left_outs.add(item("alpha.caches", "alpha.left-out.open", 3));
            let counts = left_outs.take();

            let lines: Vec<_> = counts
                .iter()
                .map(|count| (count.category.as_str(), count.reason.key().as_str(), count.items, count.sizes.logical))
                .collect();
            assert_eq!(
                lines,
                [
                    ("alpha.caches", "alpha.left-out.open", 2, ByteSize::new(5)),
                    ("alpha.logs", "alpha.left-out.open", 1, ByteSize::new(1)),
                ]
            );
        }

        #[test]
        fn a_reason_held_back_for_now_shows_its_items_largest_first() {
            let descriptor = descriptor();
            let mut left_outs = LeftOuts::new(&descriptor);

            left_outs.add(item("alpha.caches", "alpha.left-out.open", 2));
            left_outs.add(item("alpha.caches", "alpha.left-out.open", 3));
            let counts = left_outs.take();

            assert_eq!(counts[0].kind, LeftOutKind::ForNow);
            let shown: Vec<_> = counts[0].shown.iter().map(|entry| entry.path.to_str().unwrap()).collect();
            assert_eq!(shown, ["/home/3", "/home/2"]);
            assert_eq!(counts[0].shown[0].title.key().as_str(), "alpha.finding.title");
        }
    }
}
