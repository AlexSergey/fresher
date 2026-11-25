//! An app's uninstall plan: one finding the engine makes itself (never in the results) that moves the bundle and the
//! parts of the user's choice together, rehearsed and kept as the pending plan by the code every plan is made by, and
//! confirmed with `confirm_cleanup`. The app is judged from the engine's own listing, asked again now, both when the
//! plan is prepared and when it is confirmed.

use std::path::{Path, PathBuf};

use fresher_core::{
    AppId, AppOffer, CleanupPlanView, CommandError, Finding, InstalledApp, MAX_PARTS, NewFinding, NotOffered, Parts,
    Removal, Subject, Text, TextArg, UninstallChoice, uninstall_category, uninstall_title,
};

use super::cache::{self, Facts};
use super::list::{self, Listed};
use super::{committed, find, listing, protection, running};
use crate::cleanup::{Planned, Purpose, apps_folders, holds, rehearse_and_keep};
use crate::engine::Shared;
use crate::error::{EngineError, lock};
use crate::results::{Committed, add};

/// Prepares the uninstall of app `id` with `choice` and keeps it as the pending plan in place of any other.
///
/// Refused while an operation runs (`Busy`), for an app of another state or one the list leaves out (`UnknownApp`),
/// and `UninstallRefused` when the app is not offered now (asked again: the apps running now among it), its bundle is
/// not on disk as listed (`Unmeasured`), or the bundle and its parts are more than one item may hold
/// (`TooManyFiles`, never truncated). A part the rehearsal refuses leaves the whole item out with its reason. A choice
/// at High (`Everything`) is refused with `HighWithheld` while High is withheld, before anything is read.
pub(crate) fn prepare(shared: &Shared, id: AppId, choice: UninstallChoice) -> Result<CleanupPlanView, CommandError> {
    if !shared.high.offers(uninstall_category(choice).danger()) {
        return Err(CommandError::HighWithheld);
    }
    lock(&shared.machine)?.idle()?;
    let deaths = lock(&shared.pending)?.kill();
    let (committed, scanning) = committed(shared)?;
    let (installed, _) = find(shared, &committed, id)?;
    let listed = super::listed(shared, &committed, installed, &protection(shared));
    let facts = cache::facts(shared, committed.state.id(), installed, &listed, scanning)?;
    let (app, row) = offered(shared, &committed, id, &facts)?;
    let paths = paths(&app, &row, choice)?;
    let (finding, scan, roots) = {
        let mut results = lock(&shared.results)?;
        let scan = results.last().ok_or(CommandError::NoScan)?.scan;
        let finding_id = results.take_id().ok_or(EngineError::FindingIdsExhausted)?;
        let finding = finding(finding_id, &app, &row, choice, paths, facts_removability(&facts, &row))?;
        (finding, scan, roots(shared, results.walked_roots(), &app.path))
    };
    let purpose = Purpose::Uninstall { id, app: app.bundle_id.clone(), bundle: app.path.clone() };
    rehearse_and_keep(shared, Planned { deaths, scan, roots, purpose }, &[&finding])
}

/// The uninstall of app `id`, of bundle `bundle`, judged again before its confirmation acts: the state and the
/// bundle it was made for, the app still offered now (the apps running now asked again) and its bundle on disk as
/// listed. The facts are the ones the plan was made with, never measured here: the caller holds the state's lock.
pub(crate) fn judge_again(shared: &Shared, id: AppId, bundle: &Path) -> Result<(), CommandError> {
    let committed = lock(&shared.results)?.committed().ok_or(CommandError::PlanGone)?;
    let facts = cache::kept(shared, committed.state.id())?
        .ok_or(CommandError::UninstallRefused { reason: NotOffered::Unmeasured })?;
    let (app, _) = offered(shared, &committed, id, &facts)?;
    if app.path == bundle { Ok(()) } else { Err(CommandError::PlanGone) }
}

/// App `id` of `committed` and its row, when it is offered now: its offer judged with the apps running now, and its
/// bundle on disk as the listing named it.
fn offered(
    shared: &Shared,
    committed: &Committed,
    id: AppId,
    facts: &Facts,
) -> Result<(InstalledApp, Listed), CommandError> {
    let (installed, index) = find(shared, committed, id)?;
    let app = installed.get(index).ok_or(CommandError::UnknownApp { id })?;
    let protection = protection(shared);
    let running = running(shared);
    let listing = listing(shared, committed, installed, &protection, facts, running.as_deref());
    let row = list::row(&listing, index, app);
    if let AppOffer::NotOffered { reason, .. } = row.offer {
        return Err(CommandError::UninstallRefused { reason });
    }
    if !shared.ports.system.apps.is_as_listed(&app.path) {
        tracing::warn!("an app's bundle is not on disk as it was listed; it is not uninstalled");
        return Err(CommandError::UninstallRefused { reason: NotOffered::Unmeasured });
    }
    Ok((app.clone(), row))
}

/// The bundle, then every part of `row` that goes with `choice` and is not kept, in the table's order; at most
/// `MAX_PARTS`.
fn paths(app: &InstalledApp, row: &Listed, choice: UninstallChoice) -> Result<Vec<PathBuf>, CommandError> {
    let going = row.parts.iter().filter(|part| part.kept.is_none() && part.kind.goes_with(choice));
    let paths: Vec<PathBuf> = std::iter::once(app.path.clone()).chain(going.map(|part| part.path.clone())).collect();
    if paths.len() > MAX_PARTS {
        return Err(CommandError::UninstallRefused { reason: NotOffered::TooManyFiles });
    }
    Ok(paths)
}

/// The bundle's removability, from the facts the row was judged with.
fn facts_removability(facts: &Facts, row: &Listed) -> fresher_core::Removability {
    facts
        .get(row.index)
        .and_then(Option::as_ref)
        .and_then(|facts| facts.as_ref().ok())
        .map_or(fresher_core::Removability::NeedsAdministrator, fresher_core::BundleFacts::removability)
}

/// The uninstall's one finding, numbered `id`: the choice's category, about the bundle, moving `paths` together (the
/// bundle alone when no part goes), sized as their sum.
fn finding(
    id: fresher_core::FindingId,
    app: &InstalledApp,
    row: &Listed,
    choice: UninstallChoice,
    paths: Vec<PathBuf>,
    removability: fresher_core::Removability,
) -> Result<Finding, CommandError> {
    let going = row.parts.iter().filter(|part| paths.contains(&part.path));
    let sizes = going.fold(row.bundle, |total, part| add(total, part.sizes));
    let removal = if paths.len() == 1 {
        Removal::MoveToTrash(app.path.clone())
    } else {
        Removal::MoveTogether(Parts::new(paths).map_err(EngineError::Parts)?)
    };
    let reason = Text::new(choice.reason_key())
        .with("app", TextArg::Text(app.name.clone()))
        .with("size", TextArg::Size(sizes.logical));
    let new = NewFinding::new(
        &uninstall_category(choice),
        format!("uninstall/{}", app.bundle_id.as_str()),
        Subject::Path(app.path.clone()),
        sizes,
        uninstall_title(&app.name),
        reason,
        removal,
    )
    .for_app(app.bundle_id.clone());
    Ok(Finding::new(id, new, removability))
}

/// The plan's roots: what the last scan walked, and the apps folder that holds `bundle` when none of them does.
fn roots(shared: &Shared, mut roots: Vec<PathBuf>, bundle: &Path) -> Vec<PathBuf> {
    if !holds(&roots, bundle, shared.home().as_deref()) {
        let folders = apps_folders(&*shared.ports.system.known_folders);
        roots.extend(folders.into_iter().find(|folder| bundle.starts_with(folder)));
    }
    roots
}

#[cfg(test)]
#[path = "uninstall_tests.rs"]
mod tests;
