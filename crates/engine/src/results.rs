//! The findings of the current or last scan, by id, and the totals per category.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use fresher_core::{
    BundleId, ByteSize, CategoryDescriptor, CategoryId, CategoryTotal, CleanupId, CommandError, Completeness,
    DiskState, Finding, FindingId, FindingOrder, FindingsPage, HiddenBy, HiddenModule, IgnoredCount, LastScan,
    ModuleGroup, ModuleId, Place, Sizes, Subject,
};

use crate::claims::Claims;
use crate::largest::ItemsSlot;
use crate::settings::Ignored;
use crate::storage::attribution::AppPlaces;
use crate::storage::roots::RootVolumes;
use crate::storage::shares::Shares;

/// The findings of one scan, in id order. Ids keep increasing across scans, so an id from an earlier scan is unknown.
/// Also the disk state of the last committed scan, kept until another scan commits one, with the owners of its places
/// (the apps' folders and the Trash), its shares, its roots' volumes and the findings of its scan that name an app, and
/// the categories the scan's access limited, and the roots the scan walked with their volumes. The findings the user
/// ignores are kept but left out of every answer, so a restore shows them again at once.
pub(crate) struct Results {
    findings: Vec<Finding>,
    /// The rows of the Largest screen the user picked, in id order: never in a total, a page, the claims or the shares.
    own: Vec<Finding>,
    /// The paths of the picks a cleanup removed since `state` was kept, and the paths of the items that went (an
    /// uninstall's bundle and parts among them): the Largest screen and the Apps list leave them out.
    gone: Vec<PathBuf>,
    ignored: Ignored,
    roots: Vec<PathBuf>,
    /// The places the current scan walks outside its roots.
    walked: Vec<Place>,
    limited: BTreeSet<CategoryId>,
    next_id: Option<u32>,
    last: Option<LastScan>,
    state: Option<Arc<dyn DiskState>>,
    /// The apps' places and the Trash of `state`, replaced with it.
    places: Option<Arc<AppPlaces>>,
    /// The shares of `state` under `ignored`: built on the first map query, a new one with each state or ignore lists.
    shares: Arc<OnceLock<Shares>>,
    /// The Largest screen's items of `state`: built on the first query, a new slot with each state.
    largest: ItemsSlot,
    /// The volumes of `state`'s roots, read at its commit.
    state_volumes: RootVolumes,
    /// The places walked outside the roots by the scan that committed `state`.
    state_walked: Arc<[Place]>,
    /// The findings of the scan that committed `state` that name their app: the map's apps are of that scan, never of
    /// a later one that did not commit.
    state_findings: Arc<[Finding]>,
    /// The volumes of the running scan's roots, read at its start, or of the configured roots before any scan.
    volumes: RootVolumes,
    /// A scan began and has not finished.
    scanning: bool,
    /// The claims of the findings once the scan resolved them; until then every finding counts its own sizes.
    claims: Option<Claims>,
}

impl fmt::Debug for Results {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Results")
            .field("findings", &self.findings.len())
            .field("own", &self.own.len())
            .field("gone", &self.gone)
            .field("ignored", &self.ignored)
            .field("roots", &self.roots)
            .field("walked", &self.walked)
            .field("limited", &self.limited)
            .field("next_id", &self.next_id)
            .field("last", &self.last)
            .field("state", &self.state.as_ref().map(|state| state.id()))
            .field("places", &self.places.is_some())
            .field("shares", &self.shares.get().is_some())
            .field("largest", &self.largest.lock().is_ok_and(|items| items.is_some()))
            .field("state_volumes", &self.state_volumes)
            .field("state_walked", &self.state_walked)
            .field("state_findings", &self.state_findings.len())
            .field("volumes", &self.volumes)
            .field("scanning", &self.scanning)
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
    pub(crate) fn new() -> Self {
        Self {
            findings: Vec::new(),
            own: Vec::new(),
            gone: Vec::new(),
            ignored: Ignored::default(),
            roots: Vec::new(),
            walked: Vec::new(),
            limited: BTreeSet::new(),
            next_id: Some(1),
            last: None,
            state: None,
            places: None,
            shares: Arc::new(OnceLock::new()),
            largest: ItemsSlot::default(),
            state_volumes: RootVolumes::default(),
            state_walked: Arc::from([]),
            state_findings: Arc::from([]),
            volumes: RootVolumes::default(),
            scanning: false,
            claims: None,
        }
    }

    /// Forgets the previous scan's findings and the picks for those of a scan of `roots` and of the `places` outside
    /// them, whose volumes are `volumes`; gives the first id of the new scan (`None` when the ids ran out). The paths
    /// the picks removed stay until a new state is kept: they are of the state still kept.
    pub(crate) fn begin(&mut self, roots: &[PathBuf], places: &[Place], volumes: RootVolumes) -> Option<u32> {
        self.findings.clear();
        self.own.clear();
        roots.clone_into(&mut self.roots);
        places.clone_into(&mut self.walked);
        self.volumes = volumes;
        self.limited.clear();
        self.claims = None;
        self.scanning = true;
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
        self.scanning = false;
    }

    /// The findings of the current scan, in id order, the ignored ones too.
    pub(crate) fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The rows the user picked on the Largest screen, in id order.
    #[cfg(test)]
    pub(crate) fn own(&self) -> &[Finding] {
        &self.own
    }

    /// The pick of the row at `path`, if the user picked it.
    pub(crate) fn own_by_path(&self, path: &Path) -> Option<&Finding> {
        by_path(&self.own, path)
    }

    /// Keeps `finding`, a row the user picked, numbered by `take_id`.
    pub(crate) fn pick(&mut self, finding: Finding) {
        self.own.push(finding);
    }

    /// The next free finding id, taken: the next scan's ids follow it. `None` when the ids ran out.
    pub(crate) fn take_id(&mut self) -> Option<FindingId> {
        let id = self.next_id?;
        self.next_id = id.checked_add(1);
        Some(FindingId::new(id))
    }

    /// The findings the user sees, in id order: those the ignore lists do not hide.
    pub(crate) fn visible(&self) -> Vec<Finding> {
        self.shown().cloned().collect()
    }

    /// The categories of the findings the user sees whose app `owns` names.
    pub(crate) fn app_categories(&self, owns: impl Fn(&BundleId) -> bool) -> HashSet<CategoryId> {
        self.shown()
            .filter(|finding| finding.finding().app().is_some_and(&owns))
            .map(|f| f.finding().category())
            .collect()
    }

    /// From now on, `ignored` hides findings from every answer.
    pub(crate) fn set_ignored(&mut self, ignored: Ignored) {
        if self.ignored != ignored {
            self.shares = Arc::new(OnceLock::new());
        }
        self.ignored = ignored;
    }

    /// How many entries the ignore lists hold.
    pub(crate) fn ignored_count(&self) -> IgnoredCount {
        let count = |len: usize| u32::try_from(len).unwrap_or(u32::MAX);
        IgnoredCount {
            categories: count(self.ignored.categories.len()),
            paths: count(self.ignored.paths.len()),
            modules: count(self.ignored.modules.len()),
            groups: count(self.ignored.groups.len()),
        }
    }

    /// The modules the ignore lists hide, from `modules` (each with its group): by its group when that is ignored,
    /// else by its own entry.
    pub(crate) fn hidden_modules(&self, modules: impl Iterator<Item = (ModuleId, ModuleGroup)>) -> Vec<HiddenModule> {
        modules
            .filter(|(module, _)| self.ignored.hidden_modules.contains(module))
            .map(|(module, group)| HiddenModule {
                module,
                by: if self.ignored.groups.contains(&group) { HiddenBy::Group } else { HiddenBy::Module },
            })
            .collect()
    }

    fn shown(&self) -> impl Iterator<Item = &Finding> + Clone {
        self.findings.iter().filter(|finding| !self.ignored.hides(finding))
    }

    /// The places the current scan walked outside its roots.
    pub(crate) fn walked(&self) -> &[Place] {
        &self.walked
    }

    /// Everything the current scan walked: its roots as the settings had them, then its places' paths.
    pub(crate) fn walked_roots(&self) -> Vec<PathBuf> {
        self.roots.iter().cloned().chain(self.walked.iter().map(|place| place.path.clone())).collect()
    }

    /// Drops `removed` from the findings and the picks, and with them every finding or pick at or below the path of
    /// one of `cleared` (those of `removed` whose whole path went: what was inside went too, a finding left out
    /// `Covered` among it) or at or below one of `went` (the paths of a cleared item that is no finding of the
    /// results: an uninstall's bundle and parts), and their claims: the totals are of what is left. The path of every
    /// pick dropped and every path of `went` is kept as gone. Answers what was dropped: the ones of `removed` that were
    /// there (an uninstall's finding never is), then the findings and picks below, in id order. A finding that was counted less for one nested in it keeps its counted size,
    /// since what was inside it is gone too.
    pub(crate) fn remove(&mut self, removed: &[FindingId], cleared: &[FindingId], went: &[PathBuf]) -> Vec<FindingId> {
        let present = |id: &FindingId| by_id(&self.findings, *id).or_else(|| by_id(&self.own, *id)).is_some();
        let removed: Vec<FindingId> = removed.iter().copied().filter(present).collect();
        let mut dropped: HashSet<FindingId> = removed.iter().copied().collect();
        let gone: HashSet<PathBuf> = cleared
            .iter()
            .filter_map(|&id| by_id(&self.findings, id).or_else(|| by_id(&self.own, id)))
            .filter_map(|finding| path(finding).map(Path::to_path_buf))
            .chain(went.iter().cloned())
            .collect();
        let below = |finding: &Finding| path(finding).is_some_and(|path| path.ancestors().any(|at| gone.contains(at)));
        let inside: Vec<FindingId> = self
            .findings
            .iter()
            .chain(&self.own)
            .filter(|finding| !dropped.contains(&finding.id()) && below(finding))
            .map(Finding::id)
            .collect();
        dropped.extend(&inside);
        self.findings.retain(|finding| !dropped.contains(&finding.id()));
        let (left, removed_own): (Vec<Finding>, Vec<Finding>) =
            std::mem::take(&mut self.own).into_iter().partition(|finding| !dropped.contains(&finding.id()));
        self.own = left;
        self.gone.extend(removed_own.iter().filter_map(path).map(Path::to_path_buf));
        self.gone.extend_from_slice(went);
        if let Some(claims) = &mut self.claims {
            claims.forget(&dropped);
        }
        removed.into_iter().chain(inside).collect()
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

    /// Keeps `state`, the disk state a scan committed, and the owners of its places, in place of the previous ones.
    /// Its roots' `volumes` and the current findings that name their app are kept with it.
    /// The picks and the paths they removed are forgotten: they were of the previous state.
    pub(crate) fn keep_state(&mut self, state: Arc<dyn DiskState>, places: AppPlaces, volumes: RootVolumes) {
        self.state = Some(state);
        self.own.clear();
        self.gone.clear();
        self.places = Some(Arc::new(places));
        self.shares = Arc::new(OnceLock::new());
        self.largest = ItemsSlot::default();
        self.state_volumes = volumes;
        self.state_walked = Arc::from(self.walked.as_slice());
        self.state_findings =
            self.findings.iter().filter(|finding| finding.finding().app().is_some()).cloned().collect();
    }

    pub(crate) fn state(&self) -> Option<Arc<dyn DiskState>> {
        self.state.clone()
    }

    /// The kept state with what the map reads of it; `None` before any commit.
    pub(crate) fn committed(&self) -> Option<Committed> {
        Some(Committed {
            state: self.state.clone()?,
            places: self.places.clone()?,
            shares: Arc::clone(&self.shares),
            largest: Arc::clone(&self.largest),
            own: Arc::from(self.own.as_slice()),
            gone: Arc::from(self.gone.as_slice()),
            ignored: self.ignored.clone(),
            volumes: self.state_volumes.clone(),
            walked: Arc::clone(&self.state_walked),
            findings: Arc::clone(&self.state_findings),
        })
    }

    /// A scan began and has not finished.
    pub(crate) const fn is_scanning(&self) -> bool {
        self.scanning
    }

    pub(crate) fn volumes(&self) -> &RootVolumes {
        &self.volumes
    }

    pub(crate) fn keep_volumes(&mut self, volumes: RootVolumes) {
        self.volumes = volumes;
    }

    /// What the findings the user sees explain under each of `roots` (absolute): each finding about a path adds its
    /// counted size to the deepest root that holds it; a finding under no root, or about a tool's object, adds
    /// nothing.
    pub(crate) fn explained_under(&self, roots: &[PathBuf]) -> Vec<Sizes> {
        let mut explained = vec![Sizes::ZERO; roots.len()];
        for finding in self.shown() {
            let Some(path) = path(finding) else { continue };
            let holder = roots
                .iter()
                .enumerate()
                .filter(|(_, root)| path.starts_with(root))
                .max_by_key(|(_, root)| root.components().count());
            let Some(total) = holder.and_then(|(position, _)| explained.get_mut(position)) else { continue };
            let sizes =
                self.claims.as_ref().map_or_else(|| finding.finding().sizes(), |claims| claims.counted(finding));
            *total = add(*total, sizes);
        }
        explained
    }

    /// The finding `id` the user sees, or the pick `id`.
    pub(crate) fn finding(&self, id: FindingId) -> Result<Finding, CommandError> {
        let found = by_id(&self.findings, id).filter(|finding| !self.ignored.hides(finding));
        let pick = || by_id(&self.own, id).filter(|finding| !self.ignored.hides(finding));
        found.or_else(pick).cloned().ok_or(CommandError::UnknownFinding { id })
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

/// The last committed state and what the map reads of it, taken under the results' lock and read outside it.
#[derive(Clone)]
pub(crate) struct Committed {
    pub(crate) state: Arc<dyn DiskState>,
    pub(crate) places: Arc<AppPlaces>,
    /// The volumes of the state's roots, read at its commit.
    pub(crate) volumes: RootVolumes,
    /// The places walked outside the roots by the scan that committed the state.
    pub(crate) walked: Arc<[Place]>,
    /// The findings of the scan that committed the state that name their app, the ignored ones too.
    pub(crate) findings: Arc<[Finding]>,
    /// The Largest screen's items of the state, kept in the results.
    pub(crate) largest: ItemsSlot,
    /// The rows the user picked, in id order.
    pub(crate) own: Arc<[Finding]>,
    /// The paths a cleanup removed since the state was kept: the picks and every path of an item that went.
    pub(crate) gone: Arc<[PathBuf]>,
    shares: Arc<OnceLock<Shares>>,
    ignored: Ignored,
}

impl Committed {
    /// The state's shares under the ignore lists taken with it, built by the first caller and kept in the results.
    pub(crate) fn shares(&self) -> &Shares {
        self.shares.get_or_init(|| Shares::of(&*self.state, &self.ignored))
    }

    /// The paths the ignore list taken with the state holds.
    pub(crate) fn ignored_paths(&self) -> &[PathBuf] {
        &self.ignored.paths
    }
}

impl fmt::Debug for Committed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Committed").field("state", &self.state.id()).finish_non_exhaustive()
    }
}

/// The finding `id` of `findings`, in id order.
fn by_id(findings: &[Finding], id: FindingId) -> Option<&Finding> {
    findings.binary_search_by_key(&id, Finding::id).ok().map(|index| &findings[index])
}

/// The finding of `findings` about `path`.
pub(crate) fn by_path<'a>(findings: &'a [Finding], path: &Path) -> Option<&'a Finding> {
    findings.iter().find(|finding| self::path(finding) == Some(path))
}

/// The path a finding is about; `None` for a tool's object.
fn path(finding: &Finding) -> Option<&Path> {
    match finding.finding().subject() {
        Subject::Path(path) => Some(path),
        Subject::External { .. } => None,
    }
}

/// Sums two sizes; a sum past `u64::MAX` bytes stays at the maximum. The weakest completeness wins.
pub(crate) fn add(total: Sizes, more: Sizes) -> Sizes {
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
    use crate::storage::attribution::AppPlaces;
    use crate::storage::roots::RootVolumes;
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

    /// The paths the picks removed from the kept state.
    fn gone(results: &Results) -> Vec<std::path::PathBuf> {
        results.committed().unwrap().gone.to_vec()
    }

    /// `results` with a pick of `path`, numbered by the results.
    fn picked(results: &mut Results, path: &str) -> Finding {
        let id = results.take_id().unwrap();
        let pick = Finding::new(id, finding(&fresher_core::own_file(), path, sizes(100)), Removability::Removable);
        results.pick(pick.clone());
        pick
    }

    /// An empty state `id`, kept as a scan commits it.
    fn keep_state(results: &mut Results, id: u32) {
        let state = fresher_testing::FixedDiskState::new(
            fresher_core::StateId::new(id),
            fresher_core::Timestamp::from_secs(0),
            [],
        );
        results.keep_state(std::sync::Arc::new(state), AppPlaces::default(), RootVolumes::default());
    }

    fn page_ids(results: &Results, order: FindingOrder, offset: u32, limit: u32) -> Vec<u32> {
        let filter = Filter { module: None, category: None, order, offset, limit };
        results.page(filter).findings.iter().map(|finding| finding.id().get()).collect()
    }

    mod negative {
        use fresher_core::{ByteSize, Completeness, FindingOrder, Sizes};

        use fresher_core::FindingId;

        use super::{alpha, at, beta, gone, ids, keep_state, page_ids, picked, results, sizes};
        use crate::results::{Filter, Results, add};

        #[test]
        fn a_picked_finding_is_in_no_total_page_or_share() {
            let mut results = at(&["/f/a"]);
            results.set_next_id(Some(2));
            picked(&mut results, "/f/mine.iso");

            assert_eq!(results.totals([alpha(), fresher_core::own_file()].iter()).1, sizes(1));
            assert_eq!(page_ids(&results, FindingOrder::Found, 0, 10), [1]);
            assert_eq!(page_ids(&results, FindingOrder::Largest, 0, 10), [1]);
            assert_eq!(results.visible().len(), 1);
            assert_eq!(results.explained_under(&["/f".into()]), [sizes(1)]);
            assert_eq!(ids(&results), [1]);
        }

        #[test]
        fn a_new_scan_forgets_the_picks() {
            let mut results = at(&["/f/a"]);
            results.set_next_id(Some(2));
            let pick = picked(&mut results, "/f/mine.iso");

            let first = results.begin(&[], &[], crate::storage::roots::RootVolumes::default());

            assert_eq!(results.own(), []);
            assert_eq!(first, Some(pick.id().get() + 1));
            assert!(results.finding(pick.id()).is_err());
        }

        #[test]
        fn a_new_state_forgets_the_gone_paths() {
            let mut results = Results::new();
            keep_state(&mut results, 1);
            let pick = picked(&mut results, "/f/mine.iso");
            results.remove(&[pick.id()], &[pick.id()], &[]);
            assert_eq!(gone(&results), [std::path::PathBuf::from("/f/mine.iso")]);

            keep_state(&mut results, 2);

            assert_eq!(gone(&results), Vec::<std::path::PathBuf>::new());
        }

        #[test]
        fn a_removed_id_that_is_not_there_is_not_answered() {
            let mut results = at(&["/f/a", "/f/b"]);

            let removed = results.remove(&[FindingId::new(9), FindingId::new(2)], &[FindingId::new(9)], &[]);

            assert_eq!(removed, [FindingId::new(2)]);
            assert_eq!(ids(&results), [1]);
        }

        #[test]
        fn only_the_findings_below_a_wholly_removed_one_go_with_it() {
            let mut results = at(&["/f/a", "/f/ab", "/f/a2/x", "/f/b", "/f/b/in"]);

            let removed = results.remove(&[FindingId::new(1), FindingId::new(4)], &[FindingId::new(1)], &[]);

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

        use super::{alpha, at, beta, gone, ids, keep_state, page_ids, picked, results, sizes};
        use crate::results::Filter;

        #[test]
        fn a_removed_pick_is_dropped_and_its_path_is_gone() {
            let mut results = at(&["/f/a"]);
            keep_state(&mut results, 1);
            results.set_next_id(Some(2));
            let folder = picked(&mut results, "/f/mine");
            let inside = picked(&mut results, "/f/mine/in.iso");
            let other = picked(&mut results, "/f/other.iso");

            let removed = results.remove(&[folder.id()], &[folder.id()], &[]);

            assert_eq!(removed, [folder.id(), inside.id()]);
            assert_eq!(results.own(), [other]);
            assert_eq!(
                gone(&results),
                [std::path::PathBuf::from("/f/mine"), std::path::PathBuf::from("/f/mine/in.iso")]
            );
            assert_eq!(ids(&results), [1]);
        }

        #[test]
        fn a_picked_finding_is_found_by_id() {
            let mut results = at(&["/f/a"]);
            results.set_next_id(Some(2));
            let pick = picked(&mut results, "/f/mine.iso");

            assert_eq!(results.finding(pick.id()), Ok(pick.clone()));
            assert_eq!(results.own_by_path(std::path::Path::new("/f/mine.iso")), Some(&pick));
            assert_eq!(results.finding(FindingId::new(1)).map(|found| found.id()), Ok(FindingId::new(1)));
        }

        #[test]
        fn the_findings_at_or_below_a_wholly_removed_one_go_with_it() {
            let mut results = at(&["/f/a", "/f/a/in", "/f/b", "/f/a/in/deep", "/f/a"]);

            let removed = results.remove(&[FindingId::new(1)], &[FindingId::new(1)], &[]);

            assert_eq!(removed, [1, 2, 4, 5].map(FindingId::new));
            assert_eq!(ids(&results), [3]);
            assert_eq!(results.totals([alpha()].iter()).1, sizes(1));
        }

        #[test]
        fn a_finding_below_a_path_that_went_is_dropped() {
            let mut results = at(&["/home/Library/Caches/com.example.Editor", "/home/Library/Caches/com.example.Ed"]);
            keep_state(&mut results, 1);
            let went = [
                std::path::PathBuf::from("/Applications/Editor.app"),
                std::path::PathBuf::from("/home/Library/Caches/com.example.Editor"),
            ];

            let removed = results.remove(&[], &[], &went);

            assert_eq!(removed, [FindingId::new(1)]);
            assert_eq!(ids(&results), [2]);
            assert_eq!(gone(&results), went);
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
