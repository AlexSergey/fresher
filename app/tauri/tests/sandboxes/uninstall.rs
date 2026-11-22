//! The Apps screen and the uninstall of a case file's run: after the scan, the apps listed; each `[case.uninstall]`
//! checked — an app not listed, an app shown and not offered for its reason, or an app uninstalled with its choice
//! (one item, nothing left out, confirmed, its paths in the sandbox's Trash, every other path of the file in place, the
//! app no longer listed) and put back (every path back, byte for byte). Only the sandbox's own apps folders and Trash
//! are ever touched. The report holds the apps and the plans; a failure names its case.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use fresher_core::{
    AppDetail, AppOffer, AppSummary, AppsQuery, CleanupPlanView, Confirmation, Event, FootprintKind, NotOffered,
    PutBackOutcome, UninstallChoice,
};
use fresher_engine::Engine;
use fresher_sandbox::{Case, CaseFile, Uninstall, UninstallExpectation};
use serde_json::{Value, json};

use crate::harness::{Failure, failure, relative, until};

/// One app of the list, with the bundle its page names.
struct Listed {
    summary: AppSummary,
    bundle: PathBuf,
}

/// The sandbox's Trash, relative to the sandbox.
const TRASH: &str = "home/.Trash";

/// What a file of a bundle or of a folder of it holds: a file's bytes, a folder's names.
type Snapshot = BTreeMap<PathBuf, Option<Vec<u8>>>;

/// Checks every `[case.uninstall]` of `file` on the engine after its scan, in the sandbox at `sandbox`; the report's
/// `apps` and `uninstalls`, or `None` when no case has one.
pub(crate) fn check_uninstalls(
    name: &str,
    file: &CaseFile,
    engine: &Engine,
    sandbox: &Path,
) -> Result<Option<(Value, Value)>, Failure> {
    let expected: Vec<(&Case, &UninstallExpectation)> =
        file.cases().iter().filter_map(|case| Some((case, case.uninstall()?))).collect();
    if expected.is_empty() {
        return Ok(None);
    }
    let listed = list(name, engine)?;
    let apps = apps_report(&listed, sandbox);
    for &(case, uninstall) in &expected {
        let bundle = sandbox.join(uninstall.app());
        let found = listed.iter().find(|app| app.bundle == bundle);
        match uninstall.expected() {
            Uninstall::NotListed => {
                if let Some(app) = found {
                    return Err(failure(case.name(), format!("listed: {}", app.summary.name)));
                }
            }
            Uninstall::NotOffered(reason) => check_not_offered(case, found, *reason)?,
            Uninstall::Chosen { .. } => {}
        }
    }
    let mut uninstalls = Vec::new();
    for &(case, uninstall) in &expected {
        if matches!(uninstall.expected(), Uninstall::Chosen { .. }) {
            uninstalls.push(uninstall_and_put_back(case, file, engine, sandbox, uninstall)?);
        }
    }
    Ok(Some((apps, Value::Array(uninstalls))))
}

/// Every app of the list with its bundle.
fn list(name: &str, engine: &Engine) -> Result<Vec<Listed>, Failure> {
    let view = engine
        .apps(AppsQuery::default())
        .map_err(|error| failure(name, format!("the apps cannot be listed: {error:?}")))?;
    view.apps
        .into_iter()
        .map(|summary| {
            let detail: AppDetail =
                engine.app(summary.app).map_err(|error| failure(name, format!("{}: {error:?}", summary.name)))?;
            let bundle = detail
                .items
                .iter()
                .find(|item| item.kind == FootprintKind::Bundle)
                .map(|item| item.path.clone())
                .ok_or_else(|| failure(name, format!("the page of {} names no bundle", summary.name)))?;
            Ok(Listed { summary, bundle })
        })
        .collect()
}

/// The case's app is listed and not offered, for `reason`.
fn check_not_offered(case: &Case, found: Option<&Listed>, reason: NotOffered) -> Result<(), Failure> {
    let app = found.ok_or_else(|| failure(case.name(), "not listed"))?;
    match &app.summary.offer {
        AppOffer::NotOffered { reason: actual, .. } if *actual == reason => Ok(()),
        AppOffer::NotOffered { reason: actual, .. } => {
            Err(failure(case.name(), format!("not offered for `{}`, not for `{}`", actual.code(), reason.code())))
        }
        AppOffer::Offered => Err(failure(case.name(), format!("offered, not refused for `{}`", reason.code()))),
    }
}

/// Uninstalls the case's app with its choice and puts it back, checking every path on the way; the plan's report.
fn uninstall_and_put_back(
    case: &Case,
    file: &CaseFile,
    engine: &Engine,
    sandbox: &Path,
    uninstall: &UninstallExpectation,
) -> Result<Value, Failure> {
    let name = case.name();
    let Uninstall::Chosen { choice, trashed, kept } = uninstall.expected() else { return Ok(Value::Null) };
    let choice: UninstallChoice = *choice;
    let bundle = sandbox.join(uninstall.app());
    let listed = list(name, engine)?;
    let found = listed.iter().find(|listed| listed.bundle == bundle).ok_or_else(|| failure(name, "not listed"))?;
    if let AppOffer::NotOffered { reason, .. } = &found.summary.offer {
        return Err(failure(name, format!("not offered: `{}`", reason.code())));
    }
    let before = snapshot(name, sandbox, trashed)?;
    let view = engine
        .prepare_uninstall(found.summary.app, choice)
        .map_err(|error| failure(name, format!("the uninstall cannot be prepared: {error:?}")))?;
    let paths = plan_paths(&view);
    let count: usize = view.groups.iter().map(|group| group.items.len()).sum();
    if count != 1 || !view.left_out.is_empty() {
        let problem = format!("the plan has {count} items and leaves {} out, not one and none", view.left_out.len());
        return Err(failure(name, problem));
    }
    let expected: Vec<PathBuf> = trashed.iter().map(|path| sandbox.join(path)).collect();
    if sorted(paths.clone()) != sorted(expected) {
        let shown = |paths: &[PathBuf]| paths.iter().map(|path| relative(path, sandbox)).collect::<Vec<_>>();
        let problem = format!("the plan moves {:?}, the case expects {:?}", shown(&paths), trashed);
        return Err(failure(name, problem));
    }
    let report = plan_report(&view, &paths, sandbox);
    let in_trash = trash_entries(name, sandbox)?;
    let cleanup = confirm(name, engine, &view)?;

    for path in trashed {
        if fs::symlink_metadata(sandbox.join(path)).is_ok() {
            return Err(failure(name, format!("still in place after the uninstall: {}", path.display())));
        }
    }
    check_in_trash(name, sandbox, trashed, &in_trash, &before)?;
    for path in kept.iter().chain(&others(file, trashed)) {
        if fs::symlink_metadata(sandbox.join(path)).is_err() {
            return Err(failure(name, format!("gone after the uninstall: {}", path.display())));
        }
    }
    if list(name, engine)?.iter().any(|listed| listed.bundle == bundle) {
        return Err(failure(name, "still listed after the uninstall"));
    }

    let put_back =
        engine.put_back(cleanup, None).map_err(|error| failure(name, format!("cannot be put back: {error:?}")))?;
    if let Some(item) = put_back.items.iter().find(|item| item.outcome != PutBackOutcome::PutBack) {
        return Err(failure(name, format!("item {} is not put back: {:?}", item.index, item.outcome)));
    }
    if snapshot(name, sandbox, trashed)? != before {
        return Err(failure(name, "what is put back differs from what was uninstalled"));
    }
    Ok(report)
}

/// Every path the plan's one item moves: its subject, then the parts that leave with it.
fn plan_paths(view: &CleanupPlanView) -> Vec<PathBuf> {
    view.groups
        .iter()
        .flat_map(|group| &group.items)
        .flat_map(|item| item.path.iter().chain(&item.more).cloned())
        .collect()
}

fn sorted(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort();
    paths
}

/// Confirms `view` to the Trash with its own summary and waits for the end; the cleanup's id.
fn confirm(name: &str, engine: &Engine, view: &CleanupPlanView) -> Result<fresher_core::CleanupId, Failure> {
    let summary = view.summary;
    let echo: Confirmation = serde_json::from_value(json!({ "plan": view.plan, "count": summary.count,
        "allocated": summary.sizes.allocated, "highest": summary.highest, "method": "trash" }))
    .map_err(|error| failure(name, error))?;
    let events = engine.subscribe();
    let id = engine
        .confirm_cleanup(&echo)
        .map_err(|error| failure(name, format!("the uninstall is not confirmed: {error:?}")))?;
    until(name, &events, |event| matches!(event, Event::CleanupFinished { .. }))?;
    Ok(id)
}

/// The names of the entries of the sandbox's Trash, `home/.Trash`.
fn trash_entries(name: &str, sandbox: &Path) -> Result<Vec<String>, Failure> {
    let entries =
        fs::read_dir(sandbox.join(TRASH)).map_err(|error| failure(name, format!("the sandbox's Trash: {error}")))?;
    Ok(entries.filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
}

/// The uninstall added exactly one entry to the sandbox's Trash per `trashed` path, each named `<n>-<its name>` and
/// holding, byte for byte, what the path held `before`; `earlier` are the Trash's entries before the uninstall.
fn check_in_trash(
    name: &str,
    sandbox: &Path,
    trashed: &[PathBuf],
    earlier: &[String],
    before: &Snapshot,
) -> Result<(), Failure> {
    let mut arrived: Vec<String> =
        trash_entries(name, sandbox)?.into_iter().filter(|entry| !earlier.contains(entry)).collect();
    if arrived.len() != trashed.len() {
        let problem = format!("{} entries arrived in the sandbox's Trash for {} paths", arrived.len(), trashed.len());
        return Err(failure(name, problem));
    }
    for path in trashed {
        let file_name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let held: Snapshot = before
            .iter()
            .filter(|(entry, _)| entry.starts_with(path))
            .map(|(entry, bytes)| (entry.clone(), bytes.clone()))
            .collect();
        let mut found = None;
        for (position, entry) in arrived.iter().enumerate() {
            let named = entry
                .split_once('-')
                .is_some_and(|(order, rest)| order.bytes().all(|b| b.is_ascii_digit()) && rest == file_name);
            if named && moved_as(name, sandbox, entry, path)? == held {
                found = Some(position);
                break;
            }
        }
        match found {
            Some(position) => {
                arrived.remove(position);
            }
            None => return Err(failure(name, format!("not in the sandbox's Trash as it was: {}", path.display()))),
        }
    }
    Ok(())
}

/// What the Trash's `entry` holds, its paths as they were at `path`.
fn moved_as(name: &str, sandbox: &Path, entry: &str, path: &Path) -> Result<Snapshot, Failure> {
    let in_trash = Path::new(TRASH).join(entry);
    let held = snapshot(name, sandbox, std::slice::from_ref(&in_trash))?;
    Ok(held.into_iter().filter_map(|(at, bytes)| Some((path.join(at.strip_prefix(&in_trash).ok()?), bytes))).collect())
}

/// Every path the file's cases write that is neither a `trashed` path nor below one.
fn others(file: &CaseFile, trashed: &[PathBuf]) -> Vec<PathBuf> {
    file.cases()
        .iter()
        .flat_map(|case| case.tree().paths())
        .filter(|path| !trashed.iter().any(|gone| path.starts_with(gone)))
        .map(Path::to_path_buf)
        .collect()
}

/// What every entry at or below the `paths` holds, by its path relative to the sandbox; a link is never followed.
fn snapshot(name: &str, sandbox: &Path, paths: &[PathBuf]) -> Result<Snapshot, Failure> {
    let mut held = Snapshot::new();
    let mut pending: Vec<PathBuf> = paths.to_vec();
    while let Some(path) = pending.pop() {
        let full = sandbox.join(&path);
        let metadata =
            fs::symlink_metadata(&full).map_err(|error| failure(name, format!("{}: {error}", path.display())))?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&full).map_err(|error| failure(name, error))? {
                let entry = entry.map_err(|error| failure(name, error))?;
                pending.push(path.join(entry.file_name()));
            }
            held.insert(path, None);
        } else {
            let bytes = if metadata.is_file() {
                fs::read(&full).map_err(|error| failure(name, error))?
            } else {
                fs::read_link(&full).map_err(|error| failure(name, error))?.into_os_string().into_encoded_bytes()
            };
            held.insert(path, Some(bytes));
        }
    }
    Ok(held)
}

/// The apps of the list, by bundle: name, sizes, sizes by kind and offer.
fn apps_report(listed: &[Listed], sandbox: &Path) -> Value {
    let mut apps: Vec<(String, Value)> = listed
        .iter()
        .map(|app| {
            let summary = &app.summary;
            let parts: Vec<Value> = summary
                .parts
                .iter()
                .map(|part| {
                    let sizes = part.sizes;
                    json!({ "kind": part.kind, "logical": sizes.logical, "allocated": sizes.allocated })
                })
                .collect();
            let offer = match &summary.offer {
                AppOffer::Offered => "offered",
                AppOffer::NotOffered { reason, .. } => reason.code(),
            };
            let bundle = relative(&app.bundle, sandbox);
            let report = json!({
                "bundle": bundle,
                "name": summary.name,
                "logical": summary.sizes.logical,
                "allocated": summary.sizes.allocated,
                "parts": parts,
                "offer": offer,
            });
            (bundle, report)
        })
        .collect();
    apps.sort_by(|left, right| left.0.cmp(&right.0));
    Value::Array(apps.into_iter().map(|(_, report)| report).collect())
}

/// The uninstall's plan: its category, danger, items and every path it moves.
fn plan_report(view: &CleanupPlanView, paths: &[PathBuf], sandbox: &Path) -> Value {
    let group = view.groups.first();
    json!({
        "category": group.map(|group| group.category),
        "danger": view.summary.highest,
        "items": view.summary.count,
        "paths": paths.iter().map(|path| relative(path, sandbox)).collect::<Vec<_>>(),
    })
}
