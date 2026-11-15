//! The findings of the current or last scan, by id, and the totals per category.

use std::collections::BTreeMap;

use fresher_core::{
    ByteSize, CategoryDescriptor, CategoryId, CategoryTotal, CommandError, Finding, FindingId, FindingsPage, LastScan,
    ModuleId, Sizes,
};

/// The findings of one scan, in id order. Ids keep increasing across scans, so an id from an earlier scan is unknown.
#[derive(Debug)]
pub(crate) struct Results {
    findings: Vec<Finding>,
    next_id: Option<u32>,
    last: Option<LastScan>,
}

/// Which findings a page holds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Filter<'a> {
    pub(crate) module: Option<ModuleId>,
    pub(crate) category: Option<&'a str>,
    pub(crate) offset: u32,
    pub(crate) limit: u32,
}

impl Results {
    pub(crate) const fn new() -> Self {
        Self { findings: Vec::new(), next_id: Some(1), last: None }
    }

    /// Forgets the previous scan's findings; gives the first id of the new scan (`None` when the ids ran out).
    pub(crate) fn begin(&mut self) -> Option<u32> {
        self.findings.clear();
        self.next_id
    }

    pub(crate) fn append(&mut self, batch: &[Finding]) {
        self.findings.extend_from_slice(batch);
    }

    /// The scan ended: `next_id` is the first id the next scan may use.
    pub(crate) fn finish(&mut self, last: LastScan, next_id: Option<u32>) {
        self.last = Some(last);
        self.next_id = next_id;
    }

    pub(crate) fn last(&self) -> Option<LastScan> {
        self.last.clone()
    }

    pub(crate) fn finding(&self, id: FindingId) -> Result<Finding, CommandError> {
        let index = self.findings.binary_search_by_key(&id, Finding::id);
        index.map(|index| self.findings[index].clone()).map_err(|_| CommandError::UnknownFinding { id })
    }

    pub(crate) fn page(&self, filter: Filter<'_>) -> FindingsPage {
        let matching = self.findings.iter().filter(|finding| {
            filter.module.is_none_or(|module| finding.module() == module)
                && filter.category.is_none_or(|category| finding.finding().category().as_str() == category)
        });
        let total = u32::try_from(matching.clone().count()).unwrap_or(u32::MAX);
        let skip = usize::try_from(filter.offset).unwrap_or(usize::MAX);
        let take = usize::try_from(filter.limit).unwrap_or(usize::MAX);
        let findings = matching.skip(skip).take(take).cloned().collect();
        FindingsPage { findings, offset: filter.offset, total }
    }

    /// One total per category that has findings, in the order of `categories`.
    pub(crate) fn totals<'a>(&self, categories: impl Iterator<Item = &'a CategoryDescriptor>) -> Vec<CategoryTotal> {
        let mut totals: BTreeMap<CategoryId, CategoryTotal> = BTreeMap::new();
        for finding in &self.findings {
            let new = finding.finding();
            let total = totals.entry(new.category()).or_insert(CategoryTotal {
                category: new.category(),
                count: 0,
                sizes: Sizes::ZERO,
                highest_danger: new.danger(),
            });
            total.count = total.count.saturating_add(1);
            total.sizes = add(total.sizes, new.sizes());
            total.highest_danger = total.highest_danger.max(new.danger());
        }
        categories.filter_map(|category| totals.remove(&category.id())).collect()
    }

    #[cfg(test)]
    pub(crate) const fn set_next_id(&mut self, next_id: Option<u32>) {
        self.next_id = next_id;
    }
}

/// Sums two sizes; a sum past `u64::MAX` bytes stays at the maximum. The weakest completeness wins.
fn add(total: Sizes, more: Sizes) -> Sizes {
    let sum = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
    Sizes {
        logical: sum(total.logical, more.logical),
        allocated: sum(total.allocated, more.allocated),
        completeness: total.completeness.max(more.completeness),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{ByteSize, Completeness, Sizes};

        use crate::results::add;

        #[test]
        fn a_sum_past_the_largest_size_stays_at_the_largest() {
            let huge = Sizes {
                completeness: Completeness::Unknown,
                ..Sizes::complete(ByteSize::new(u64::MAX), ByteSize::new(1))
            };

            let sum = add(huge, Sizes::complete(ByteSize::new(5), ByteSize::new(5)));

            assert_eq!(sum.logical, ByteSize::new(u64::MAX));
            assert_eq!(sum.allocated, ByteSize::new(6));
            assert_eq!(sum.completeness, Completeness::Unknown);
        }
    }
}
