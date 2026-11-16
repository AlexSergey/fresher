//! The findings of the current or last scan, by id, and the totals per category.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CategoryDescriptor, CategoryId, CategoryTotal, CleanupId, CommandError, Completeness, DiskState, Finding,
    FindingId, FindingOrder, FindingsPage, IgnoredCount, LastScan, ModuleId, Sizes, Subject,
};

use crate::claims::Claims;
use crate::settings::Ignored;

/// The findings of one scan, in id order. Ids keep increasing across scans, so an id from an earlier scan is unknown.
/// Also the disk state of the last committed scan, kept until another scan commits one, and the categories the scan's
/// access limited, and the roots the scan walked. The findings the user ignores are kept but left out of every
/// answer, so a restore shows them again at once.
pub(crate) struct Results {
    findings: Vec<Finding>,
    ignored: Ignored,
    roots: Vec<PathBuf>,
    limited: BTreeSet<CategoryId>,
    next_id: Option<u32>,
    last: Option<LastScan>,
    state: Option<Arc<dyn DiskState>>,
    /// The claims of the findings once the scan resolved them; until then every finding counts its own sizes.
    claims: Option<Claims>,
}

impl fmt::Debug for Results {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Results")
            .field("findings", &self.findings.len())
            .field("ignored", &self.ignored)
            .field("roots", &self.roots)
            .field("limited", &self.limited)
            .field("next_id", &self.next_id)
            .field("last", &self.last)
            .field("state", &self.state.as_ref().map(|state| state.id()))
            .field("claims", &self.claims.is_some())
            .finish()
    }
}

/// Which findings a page holds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Filter<'a> {
    pub(crate) module: Option<ModuleId>,
    pub(crate) category: Option<&'a str>,
    pub(crate) order: FindingOrder,
    pub(crate) offset: u32,
    pub(crate) limit: u32,
}

impl Results {
    pub(crate) const fn new() -> Self {
        Self {
            findings: Vec::new(),
            ignored: Ignored { categories: Vec::new(), paths: Vec::new() },
            roots: Vec::new(),
            limited: BTreeSet::new(),
            next_id: Some(1),
            last: None,
            state: None,
            claims: None,
        }
    }

    /// Forgets the previous scan's findings for those of a scan of `roots`; gives the first id of the new scan
    /// (`None` when the ids ran out).
    pub(crate) fn begin(&mut self, roots: &[PathBuf]) -> Option<u32> {
        self.findings.clear();
        roots.clone_into(&mut self.roots);
        self.limited.clear();
        self.claims = None;
        self.next_id
    }

    /// The scan's access hides part of `categories`: their totals are partial.
    pub(crate) fn limit(&mut self, categories: BTreeSet<CategoryId>) {
        self.limited = categories;
    }

    pub(crate) fn append(&mut self, batch: &[Finding]) {
        self.findings.extend_from_slice(batch);
    }

    /// The scan ended: `next_id` is the first id the next scan may use.
    pub(crate) fn finish(&mut self, last: LastScan, next_id: Option<u32>) {
        self.last = Some(last);
        self.next_id = next_id;
    }

    /// The findings of the current scan, in id order, the ignored ones too.
    pub(crate) fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The findings the user sees, in id order: those the ignore lists do not hide.
    pub(crate) fn visible(&self) -> Vec<Finding> {
        self.shown().cloned().collect()
    }

    /// From now on, `ignored` hides findings from every answer.
    pub(crate) fn set_ignored(&mut self, ignored: Ignored) {
        self.ignored = ignored;
    }

    /// How many entries the ignore lists hold.
    pub(crate) fn ignored_count(&self) -> IgnoredCount {
        let count = |len: usize| u32::try_from(len).unwrap_or(u32::MAX);
        IgnoredCount { categories: count(self.ignored.categories.len()), paths: count(self.ignored.paths.len()) }
    }

    fn shown(&self) -> impl Iterator<Item = &Finding> + Clone {
        self.findings.iter().filter(|finding| !self.ignored.hides(finding))
    }

    /// The roots the current scan walked, as the settings had them.
    pub(crate) fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Drops `removed` from the findings, and with them every finding at or below the path of one of `cleared` (those
    /// of `removed` whose whole path went: what was inside went too, a finding left out `Covered` among it), and their
    /// claims: the totals are of what is left. Answers what was dropped: `removed`, then the findings below, in id
    /// order. A finding that was counted less for one nested in it keeps its counted size, since what was inside it
    /// is gone too.
    pub(crate) fn remove(&mut self, removed: &[FindingId], cleared: &[FindingId]) -> Vec<FindingId> {
        let mut dropped: HashSet<FindingId> = removed.iter().copied().collect();
        let gone: HashSet<PathBuf> = cleared
            .iter()
            .filter_map(|&id| self.findings.binary_search_by_key(&id, Finding::id).ok())
            .filter_map(|index| path(&self.findings[index]).map(Path::to_path_buf))
            .collect();
        let below = |finding: &Finding| path(finding).is_some_and(|path| path.ancestors().any(|at| gone.contains(at)));
        let inside: Vec<FindingId> = self
            .findings
            .iter()
            .filter(|finding| !dropped.contains(&finding.id()) && below(finding))
            .map(Finding::id)
            .collect();
        dropped.extend(&inside);
        self.findings.retain(|finding| !dropped.contains(&finding.id()));
        if let Some(claims) = &mut self.claims {
            claims.forget(&dropped);
        }
        removed.iter().copied().chain(inside).collect()
    }

    /// The disk changed since the last scan: `cleanup` acted on it.
    pub(crate) fn mark_changed(&mut self, cleanup: CleanupId) {
        if let Some(last) = &mut self.last {
            last.changed_since = Some(cleanup);
        }
    }

    /// A setting that applies at the next scan changed since the last scan.
    pub(crate) fn mark_settings_changed(&mut self) {
        if let Some(last) = &mut self.last {
            last.settings_changed = true;
        }
    }

    /// From now on, category totals add each finding's counted size.
    pub(crate) fn keep_claims(&mut self, claims: Claims) {
        self.claims = Some(claims);
    }

    pub(crate) fn last(&self) -> Option<LastScan> {
        self.last.clone()
    }

    /// Keeps `state`, the disk state a scan committed, in place of the previous one.
    pub(crate) fn keep_state(&mut self, state: Arc<dyn DiskState>) {
        self.state = Some(state);
    }

    pub(crate) fn state(&self) -> Option<Arc<dyn DiskState>> {
        self.state.clone()
    }

    pub(crate) fn finding(&self, id: FindingId) -> Result<Finding, CommandError> {
        let index = self.findings.binary_search_by_key(&id, Finding::id);
        let found = index.ok().map(|index| &self.findings[index]).filter(|finding| !self.ignored.hides(finding));
        found.cloned().ok_or(CommandError::UnknownFinding { id })
    }

    pub(crate) fn page(&self, filter: Filter<'_>) -> FindingsPage {
        let matching = self.shown().filter(|finding| {
            filter.module.is_none_or(|module| finding.module() == module)
                && filter.category.is_none_or(|category| finding.finding().category().as_str() == category)
        });
        let skip = usize::try_from(filter.offset).unwrap_or(usize::MAX);
        let take = usize::try_from(filter.limit).unwrap_or(usize::MAX);
        let (total, findings) = match filter.order {
            FindingOrder::Found => (matching.clone().count(), matching.skip(skip).take(take).cloned().collect()),
            FindingOrder::Largest => {
                let mut sorted: Vec<&Finding> = matching.collect();
                sorted.sort_unstable_by_key(|finding| (Reverse(finding.finding().sizes().allocated), finding.id()));
                (sorted.len(), sorted.into_iter().skip(skip).take(take).cloned().collect())
            }
        };
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        FindingsPage { findings, offset: filter.offset, total }
    }

    /// One total per category that has findings, in the order of `categories`, and every finding summed. Once the
    /// claims are resolved, a finding adds its counted size, so a path claimed twice or nested in another finding is
    /// counted once. A category the scan's access limited is partial, and so is the sum.
    pub(crate) fn totals<'a>(
        &self,
        categories: impl Iterator<Item = &'a CategoryDescriptor>,
    ) -> (Vec<CategoryTotal>, Sizes) {
        let mut totals: BTreeMap<CategoryId, CategoryTotal> = BTreeMap::new();
        let mut found = Sizes::ZERO;
        for finding in self.shown() {
            let new = finding.finding();
            let total = totals.entry(new.category()).or_insert(CategoryTotal {
                category: new.category(),
                count: 0,
                sizes: Sizes::ZERO,
                highest_danger: new.danger(),
            });
            total.count = total.count.saturating_add(1);
            let sizes = self.claims.as_ref().map_or_else(|| new.sizes(), |claims| claims.counted(finding));
            total.sizes = add(total.sizes, sizes);
            total.highest_danger = total.highest_danger.max(new.danger());
            found = add(found, sizes);
        }
        for (category, total) in &mut totals {
            if self.limited.contains(category) {
                total.sizes.completeness = total.sizes.completeness.max(Completeness::Partial);
                found.completeness = found.completeness.max(Completeness::Partial);
            }
        }
        (categories.filter_map(|category| totals.remove(&category.id())).collect(), found)
    }

    #[cfg(test)]
    pub(crate) const fn set_next_id(&mut self, next_id: Option<u32>) {
        self.next_id = next_id;
    }
}

/// The path a finding is about; `None` for a tool's object.
fn path(finding: &Finding) -> Option<&Path> {
    match finding.finding().subject() {
        Subject::Path(path) => Some(path),
        Subject::External { .. } => None,
    }
}

/// Sums two sizes; a sum past `u64::MAX` bytes stays at the maximum. The weakest completeness wins.
fn add(total: Sizes, more: Sizes) -> Sizes {
    let sum = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
    Sizes {
        logical: sum(total.logical, more.logical),
        allocated: sum(total.allocated, more.allocated),
        reclaimable: sum(total.reclaimable, more.reclaimable),
        completeness: total.completeness.max(more.completeness),
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{
        ByteSize, CategoryDescriptor, DangerLevel, Finding, FindingId, FindingOrder, Removability, Sizes,
    };

    use crate::results::{Filter, Results};
    use crate::testing::{category, finding};

    fn alpha() -> CategoryDescriptor {
        category("alpha.caches", DangerLevel::Safe)
    }

    fn beta() -> CategoryDescriptor {
        category("beta.deps", DangerLevel::Low)
    }

    fn sizes(allocated: u64) -> Sizes {
        Sizes::complete(ByteSize::new(allocated), ByteSize::new(allocated))
    }

    /// Results holding one finding per entry, ids from 1 in the order given.
    fn results(findings: &[(&CategoryDescriptor, Sizes)]) -> Results {
        let mut results = Results::new();
        let batch: Vec<Finding> = (1..)
            .zip(findings)
            .map(|(id, (category, sizes))| {
                Finding::new(FindingId::new(id), finding(category, format!("/f/{id}"), *sizes), Removability::Removable)
            })
            .collect();
        results.append(&batch);
        results
    }

    /// Results holding one `alpha` finding of 1 byte per path, ids from 1 in the order given.
    fn at(paths: &[&str]) -> Results {
        let mut results = Results::new();
        let batch: Vec<Finding> = (1..)
            .zip(paths)
            .map(|(id, path)| {
                Finding::new(FindingId::new(id), finding(&alpha(), path, sizes(1)), Removability::Removable)
            })
            .collect();
        results.append(&batch);
        results
    }

    fn ids(results: &Results) -> Vec<u32> {
        results.findings().iter().map(|finding| finding.id().get()).collect()
    }

    fn page_ids(results: &Results, order: FindingOrder, offset: u32, limit: u32) -> Vec<u32> {
        let filter = Filter { module: None, category: None, order, offset, limit };
        results.page(filter).findings.iter().map(|finding| finding.id().get()).collect()
    }

    mod negative {
        use fresher_core::{ByteSize, Completeness, FindingOrder, Sizes};

        use fresher_core::FindingId;

        use super::{alpha, at, beta, ids, page_ids, results, sizes};
        use crate::results::{Filter, add};

        #[test]
        fn only_the_findings_below_a_wholly_removed_one_go_with_it() {
            let mut results = at(&["/f/a", "/f/ab", "/f/a2/x", "/f/b", "/f/b/in"]);

            let removed = results.remove(&[FindingId::new(1), FindingId::new(4)], &[FindingId::new(1)]);

            assert_eq!(removed, [1, 4].map(FindingId::new));
            assert_eq!(ids(&results), [2, 3, 5]);
        }

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

        #[test]
        fn found_is_zero_and_complete_without_findings() {
            let (totals, found) = results(&[]).totals([alpha()].iter());

            assert_eq!(totals, []);
            assert_eq!(found, Sizes::ZERO);
        }

        #[test]
        fn found_takes_the_weakest_completeness() {
            let partial = Sizes { completeness: Completeness::Partial, ..sizes(5) };
            let unknown = Sizes { completeness: Completeness::Unknown, ..sizes(0) };

            let (_, found) = results(&[(&alpha(), sizes(1)), (&beta(), unknown), (&alpha(), partial)])
                .totals([alpha(), beta()].iter());

            assert_eq!(found.allocated, ByteSize::new(6));
            assert_eq!(found.completeness, Completeness::Unknown);
        }

        #[test]
        fn found_saturates_at_the_largest_size() {
            let (_, found) =
                results(&[(&alpha(), sizes(u64::MAX)), (&beta(), sizes(7))]).totals([alpha(), beta()].iter());

            assert_eq!(found, sizes(u64::MAX));
        }

        #[test]
        fn largest_order_breaks_ties_by_id() {
            let results =
                results(&[(&alpha(), sizes(5)), (&beta(), sizes(9)), (&alpha(), sizes(5)), (&beta(), sizes(5))]);

            assert_eq!(page_ids(&results, FindingOrder::Largest, 0, 10), [2, 1, 3, 4]);
        }

        #[test]
        fn an_offset_past_the_end_gives_an_empty_page_in_both_orders() {
            let results = results(&[(&alpha(), sizes(1)), (&alpha(), sizes(2))]);

            for order in [FindingOrder::Found, FindingOrder::Largest] {
                let page = results.page(Filter { module: None, category: None, order, offset: 5, limit: 10 });

                assert_eq!(page.findings, []);
                assert_eq!(page.offset, 5);
                assert_eq!(page.total, 2);
            }
        }
    }

    mod positive {
        use fresher_core::FindingOrder;

        use fresher_core::FindingId;

        use super::{alpha, at, beta, ids, page_ids, results, sizes};
        use crate::results::Filter;

        #[test]
        fn the_findings_at_or_below_a_wholly_removed_one_go_with_it() {
            let mut results = at(&["/f/a", "/f/a/in", "/f/b", "/f/a/in/deep", "/f/a"]);

            let removed = results.remove(&[FindingId::new(1)], &[FindingId::new(1)]);

            assert_eq!(removed, [1, 2, 4, 5].map(FindingId::new));
            assert_eq!(ids(&results), [3]);
            assert_eq!(results.totals([alpha()].iter()).1, sizes(1));
        }

        #[test]
        fn found_sums_every_category() {
            let (totals, found) = results(&[(&alpha(), sizes(1)), (&beta(), sizes(20)), (&alpha(), sizes(300))])
                .totals([alpha(), beta()].iter());

            assert_eq!(totals.len(), 2);
            assert_eq!(found, sizes(321));
        }

        #[test]
        fn largest_order_pages_through_every_finding_once() {
            let all: Vec<_> =
                (0..25_u64).map(|n| (if n % 2 == 0 { alpha() } else { beta() }, sizes(n * 7 % 11))).collect();
            let entries: Vec<_> = all.iter().map(|(category, sizes)| (category, *sizes)).collect();
            let results = results(&entries);

            let mut paged = Vec::new();
            for offset in (0..25).step_by(4) {
                paged.extend(page_ids(&results, FindingOrder::Largest, offset, 4));
            }

            let mut expected: Vec<u32> = (1..=25).collect();
            expected.sort_by_key(|id| (std::cmp::Reverse(all[*id as usize - 1].1.allocated), *id));
            assert_eq!(paged, expected);
        }

        #[test]
        fn largest_order_keeps_the_filter() {
            let results = results(&[(&alpha(), sizes(1)), (&beta(), sizes(50)), (&alpha(), sizes(9))]);

            let page = results.page(Filter {
                module: None,
                category: Some("alpha.caches"),
                order: FindingOrder::Largest,
                offset: 0,
                limit: 10,
            });

            assert_eq!(page.findings.iter().map(|finding| finding.id().get()).collect::<Vec<_>>(), [3, 1]);
            assert_eq!(page.total, 2);
        }

        #[test]
        fn found_order_is_the_order_of_the_scan() {
            let results = results(&[(&alpha(), sizes(1)), (&beta(), sizes(50)), (&alpha(), sizes(9))]);

            assert_eq!(page_ids(&results, FindingOrder::Found, 0, 10), [1, 2, 3]);
            assert_eq!(page_ids(&results, FindingOrder::Found, 1, 1), [2]);
        }
    }
}
