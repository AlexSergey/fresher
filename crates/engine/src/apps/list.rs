//! Which installed apps the Apps list holds, which of them may be uninstalled, and what each row says.

use std::cmp::Reverse;
use std::path::{Path, PathBuf};

use fresher_core::{
    AppId, AppOffer, AppSummary, AppsQuery, BundleFacts, Completeness, DiskState, FootprintKind, HighLevel,
    InstalledApp, InstalledBy, KnownFolder, KnownFolders, LastUse, NotOffered, PartSizes, Protection, RunningApp,
    Sizes, StateId, SystemError, Timestamp,
};

use super::footprint::{FootprintCx, Part, footprint};
use crate::results::add;

/// An app is "not used" in its sentence once its last use is older than this.
const UNUSED_AFTER: u64 = 30 * DAY;
const DAY: u64 = 24 * 60 * 60;

/// The prefix of Apple's own bundle ids: never listed.
const APPLE: &str = "com.apple.";

/// Fresher's own bundle id, the `identifier` of the app's configuration (its development build's and its helpers'
/// start with it and a dot): never listed, whatever the cleaner's protection says.
pub const OWN: &str = "io.github.alexsergey.fresher";

/// The kinds whose newest change tells a use of the app: it writes them while it runs.
const USED_BY: [FootprintKind; 4] =
    [FootprintKind::Caches, FootprintKind::SavedState, FootprintKind::Settings, FootprintKind::Support];

/// What a listing reads.
#[derive(Clone, Copy)]
pub(crate) struct Listing<'a> {
    pub(crate) state: &'a dyn DiskState,
    pub(crate) folders: &'a dyn KnownFolders,
    pub(crate) installed: &'a [InstalledApp],
    /// The cleaner's protection: an app whose bundle it refuses is not listed.
    pub(crate) protection: &'a Protection,
    /// The paths a cleanup removed since the state was kept: an app whose bundle went is not listed, a part that went
    /// is no part.
    pub(crate) gone: &'a [PathBuf],
    /// The bundles' facts, by the app's position in `installed`.
    pub(crate) facts: &'a [Option<Result<BundleFacts, SystemError>>],
    /// The apps running now; `None` when they could not be listed: then every app counts as open.
    pub(crate) running: Option<&'a [RunningApp]>,
    pub(crate) cx: FootprintCx<'a>,
}

/// One app of the list, before it is told.
#[derive(Clone, Debug)]
pub(crate) struct Listed {
    /// Its position in the installed list: its `AppId`'s index.
    pub(crate) index: usize,
    pub(crate) offer: AppOffer,
    /// The bundle's sizes; unknown when it was not measured.
    pub(crate) bundle: Sizes,
    pub(crate) parts: Vec<Part>,
    pub(crate) last_used: Option<Timestamp>,
    pub(crate) installed_by: Option<InstalledBy>,
}

/// Whether `app` is listed at all: not Apple's nor Fresher's own, not under the system's apps folder, directly in an
/// apps folder or one folder below, not refused by the cleaner's protection, and not at or below a path a cleanup
/// removed since the state was kept (`gone`: an app uninstalled leaves the list at once).
pub(crate) fn is_listed(
    app: &InstalledApp,
    folders: &dyn KnownFolders,
    protection: &Protection,
    gone: &[PathBuf],
) -> bool {
    let apple = app.bundle_id.folded().starts_with(APPLE);
    let own = std::iter::once(&app.bundle_id).chain(&app.helpers).any(|id| {
        let id = id.folded();
        id == OWN || id.strip_prefix(OWN).is_some_and(|rest| rest.starts_with('.'))
    });
    let system = folders.locate(KnownFolder::SystemApplications).is_some_and(|folder| app.path.starts_with(folder));
    let in_apps_folder = [KnownFolder::Applications, KnownFolder::UserApplications]
        .into_iter()
        .filter_map(|folder| folders.locate(folder))
        .any(|folder| is_bundle_in(&app.path, &folder));
    let went = gone.iter().any(|path| app.path.starts_with(path));
    !apple && !own && !system && in_apps_folder && !went && protection.check(&app.path).is_ok()
}

/// `<folder>/<name>.app` or `<folder>/<sub>/<name>.app`, `<sub>` not itself a bundle.
fn is_bundle_in(path: &Path, folder: &Path) -> bool {
    let is_bundle = |path: &Path| path.extension().is_some_and(|extension| extension == "app");
    let Some(parent) = path.parent() else { return false };
    is_bundle(path) && (parent == folder || (parent.parent() == Some(folder) && !is_bundle(parent)))
}

/// The listed apps of `listing`, in the installed list's order, each with its offer: the design's rows 2 to 8.
pub(crate) fn rows(listing: &Listing<'_>) -> Vec<Listed> {
    listing
        .installed
        .iter()
        .enumerate()
        .filter(|(_, app)| is_listed(app, listing.folders, listing.protection, listing.gone))
        .map(|(index, app)| row(listing, index, app))
        .collect()
}

/// The row of `app`, at `index` in the installed list: its footprint, its offer (rows 4 to 8) and its last use.
pub(crate) fn row(listing: &Listing<'_>, index: usize, app: &InstalledApp) -> Listed {
    let mut parts = footprint(listing.state, listing.folders, listing.installed, index, &listing.cx);
    parts.retain(|part| !listing.gone.iter().any(|path| part.path.starts_with(path)));
    let facts = listing.facts.get(index).and_then(Option::as_ref).and_then(|facts| facts.as_ref().ok());
    let open = listing
        .running
        .is_none_or(|running| running.iter().any(|running| running.bundle_ids.iter().any(|id| app.owns(id))));
    let offer = match facts {
        None => AppOffer::not_offered(NotOffered::Unmeasured),
        Some(BundleFacts { installed_by: Some(installed), .. }) => AppOffer::installed_by(installed.manager()),
        Some(facts) => match NotOffered::of(facts.removability()) {
            Some(reason) => AppOffer::not_offered(reason),
            None if open => AppOffer::not_offered(NotOffered::Open),
            None => AppOffer::Offered,
        },
    };
    let used = parts.iter().filter(|part| USED_BY.contains(&part.kind)).filter_map(|part| part.modified);
    let last_used = facts.and_then(|facts| facts.last_used).into_iter().chain(used).max();
    Listed {
        index,
        offer,
        bundle: facts.map_or(Sizes { completeness: Completeness::Unknown, ..Sizes::ZERO }, |facts| facts.sizes),
        parts,
        last_used,
        installed_by: facts.and_then(|facts| facts.installed_by.clone()),
    }
}

/// The row of `listed`, an app of `state`, as the list shows it at `now`. Its sizes and parts are those `high` offers:
/// while High is withheld, no part of a High kind and none of its bytes.
pub(crate) fn summary(
    listed: &Listed,
    app: &InstalledApp,
    state: StateId,
    now: Timestamp,
    high: HighLevel,
) -> AppSummary {
    let shown = listed.parts.iter().filter(|part| high.offers(part.kind.danger()));
    let sizes = shown.clone().fold(listed.bundle, |total, part| add(total, part.sizes));
    let last_use = match listed.last_used {
        Some(since) if now.secs().saturating_sub(since.secs()) > UNUSED_AFTER => LastUse::Unused { since },
        Some(_) => LastUse::Used,
        None => LastUse::Unknown,
    };
    let mut parts = vec![PartSizes { kind: FootprintKind::Bundle, sizes: listed.bundle }];
    for part in shown {
        match parts.iter_mut().find(|sizes| sizes.kind == part.kind) {
            Some(sizes) => sizes.sizes = add(sizes.sizes, part.sizes),
            None => parts.push(PartSizes { kind: part.kind, sizes: part.sizes }),
        }
    }
    AppSummary {
        app: AppId::new(state, u32::try_from(listed.index).unwrap_or(u32::MAX)),
        name: app.name.clone(),
        version: app.version.clone(),
        sizes,
        parts,
        last_used: listed.last_used,
        summary: last_use.summary(&app.name, sizes.allocated),
        offer: listed.offer.clone(),
    }
}

/// The rows `query` keeps, largest first, then by name: with a filter, only the apps whose last use is known and at
/// least its days old.
pub(crate) fn filtered(mut rows: Vec<AppSummary>, query: AppsQuery, now: Timestamp) -> Vec<AppSummary> {
    if let Some(unused) = query.unused() {
        let age = unused.days().saturating_mul(DAY);
        rows.retain(|row| row.last_used.is_some_and(|last| now.secs().saturating_sub(last.secs()) >= age));
    }
    rows.sort_by(|a, b| {
        (Reverse(a.sizes.allocated), &a.name, a.app).cmp(&(Reverse(b.sizes.allocated), &b.name, b.app))
    });
    rows
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
