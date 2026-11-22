//! The installed apps over the last committed scan: the Apps list, an app's page and Show in Finder. Read-only:
//! nothing here prepares or removes anything. The UI passes only an `AppId`; the path it names is one the engine
//! listed itself.

mod cache;
#[cfg(test)]
pub(crate) mod fixture;
mod footprint;
mod list;
pub(crate) mod management;
mod page;
mod places;
pub(crate) mod uninstall;

use std::collections::HashSet;

use fresher_core::{
    AppDetail, AppId, AppsQuery, AppsView, Capability, CategoryId, CommandError, EngineStatus, InstalledApp,
    Protection, RunningApp, SystemError, Timestamp,
};

pub(crate) use cache::AppsCache;
use footprint::FootprintCx;
pub use list::OWN as OWN_BUNDLE_ID;
use list::{Listed, Listing};
pub(crate) use management::Management;

use crate::engine::Shared;
use crate::error::{EngineError, lock};
use crate::results::Committed;

/// The Apps list of the last committed state under `query`; `NoScan` before any commit.
pub(crate) fn apps(shared: &Shared, query: AppsQuery) -> Result<AppsView, CommandError> {
    let (committed, scanning) = committed(shared)?;
    let Some(installed) = committed.places.installed() else { return Ok(AppsView::unlisted()) };
    let protection = protection(shared);
    let listed = listed(shared, &committed, installed, &protection);
    let facts = cache::facts(shared, committed.state.id(), installed, &listed, scanning)?;
    let running = running(shared);
    let listing = listing(shared, &committed, installed, &protection, &facts, running.as_deref());
    let now = now(shared);
    let summaries = list::rows(&listing)
        .iter()
        .filter_map(|row| Some(list::summary(row, installed.get(row.index)?, committed.state.id(), now)))
        .collect();
    Ok(AppsView::listed(list::filtered(summaries, query, now)))
}

/// The page of app `id`; `UnknownApp` for an id of another state, past the list, or of an app the list leaves out.
pub(crate) fn app(shared: &Shared, id: AppId) -> Result<AppDetail, CommandError> {
    let (committed, scanning) = committed(shared)?;
    let (installed, index) = find(shared, &committed, id)?;
    let protection = protection(shared);
    let listed = listed(shared, &committed, installed, &protection);
    let facts = cache::facts(shared, committed.state.id(), installed, &listed, scanning)?;
    let running = running(shared);
    let listing = listing(shared, &committed, installed, &protection, &facts, running.as_deref());
    let app = installed.get(index).ok_or(CommandError::UnknownApp { id })?;
    let row: Listed = list::row(&listing, index, app);
    let summary = list::summary(&row, app, committed.state.id(), now(shared));
    let management = shared.access.management().access();
    Ok(page::detail(&row, app, summary, categories(shared, app)?, management))
}

/// Shows app `id`'s bundle in Finder, selected; refused as [`app`] refuses.
pub(crate) fn reveal(shared: &Shared, id: AppId) -> Result<(), CommandError> {
    let (committed, _) = committed(shared)?;
    let (installed, index) = find(shared, &committed, id)?;
    let app = installed.get(index).ok_or(CommandError::UnknownApp { id })?;
    shared.ports.system_ui.reveal(&app.path).map_err(|error| EngineError::SystemUi(error).into())
}

/// The last committed state, taken out of the results' lock, and whether a scan runs now: started (the machine says
/// so at once) or not finished (the results say so until its end).
fn committed(shared: &Shared) -> Result<(Committed, bool), CommandError> {
    let started = matches!(lock(&shared.machine)?.status(), EngineStatus::Scanning { .. });
    let results = lock(&shared.results)?;
    Ok((results.committed().ok_or(CommandError::NoScan)?, started || results.is_scanning()))
}

/// The installed list of `committed` and the position of `id` in it, when `id` is of that state and the list holds it.
fn find<'a>(shared: &Shared, committed: &'a Committed, id: AppId) -> Result<(&'a [InstalledApp], usize), CommandError> {
    let unknown = || CommandError::UnknownApp { id };
    if id.state() != committed.state.id() {
        return Err(unknown());
    }
    let installed = committed.places.installed().ok_or_else(unknown)?;
    let index = usize::try_from(id.index()).map_err(|_| unknown())?;
    let app = installed.get(index).ok_or_else(unknown)?;
    if list::is_listed(app, &*shared.ports.system.known_folders, &protection(shared), &committed.gone) {
        Ok((installed, index))
    } else {
        Err(unknown())
    }
}

/// The positions of the apps the list of `committed` holds.
fn listed(shared: &Shared, committed: &Committed, installed: &[InstalledApp], protection: &Protection) -> Vec<usize> {
    let folders = &*shared.ports.system.known_folders;
    installed
        .iter()
        .enumerate()
        .filter(|(_, app)| list::is_listed(app, folders, protection, &committed.gone))
        .map(|(index, _)| index)
        .collect()
}

fn listing<'a>(
    shared: &'a Shared,
    committed: &'a Committed,
    installed: &'a [InstalledApp],
    protection: &'a Protection,
    facts: &'a cache::Facts,
    running: Option<&'a [RunningApp]>,
) -> Listing<'a> {
    Listing {
        state: &*committed.state,
        folders: &*shared.ports.system.known_folders,
        installed,
        protection,
        gone: &committed.gone,
        facts,
        running,
        cx: FootprintCx {
            other_apps_data: shared.access.status().access(&Capability::OtherAppsData),
            ignored: committed.ignored_paths(),
            complete: committed.places.installed_complete(),
        },
    }
}

/// The cleaner's protection; nothing protected when the engine cannot clean.
fn protection(shared: &Shared) -> Protection {
    shared.cleanup.as_ref().map(|cleanup| cleanup.cleaner.protection()).unwrap_or_default()
}

/// The apps running now, asked once per listing; `None` (logged) when they could not be listed.
fn running(shared: &Shared) -> Option<Vec<RunningApp>> {
    shared
        .ports
        .system
        .running
        .list()
        .inspect_err(|error: &SystemError| tracing::warn!(%error, "the running apps could not be listed"))
        .ok()
}

fn now(shared: &Shared) -> Timestamp {
    Timestamp::from_system_time(shared.ports.clock.now())
}

/// The categories of the findings on Scan that belong to `app` or one of its helpers, in the registry's order.
fn categories(shared: &Shared, app: &InstalledApp) -> Result<Vec<CategoryId>, CommandError> {
    let found: HashSet<CategoryId> = lock(&shared.results)?.app_categories(|id| app.owns(id));
    Ok(shared.registry.categories().map(fresher_core::CategoryDescriptor::id).filter(|id| found.contains(id)).collect())
}
