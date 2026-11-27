//! The Largest screen of a case file's sandbox, read after the scan and before the cleanup: every `[case.largest]`
//! holds in the Files list, the Folders list (the folders left alone) or the Browse view, and the positives of the
//! core's own category are picked on the Files list, as a tick on the screen does, so the cleanup takes them through
//! the confirmed plan.

use std::path::{Path, PathBuf};

use fresher_core::{
    Capability, DiskState, Finding, FolderRow, FoldersQuery, GuardedFolder, LargestQuery, LargestRow, NodeId, NodeKind,
    NotEntered, OWN_FILE_ID, Offer, OwnRow, Unchanged,
};
use fresher_engine::Engine;
use fresher_sandbox::{Case, CaseFile, LargestExpectation, Shown};
use serde_json::{Value, json};

use crate::harness::{Failure, failure, relative};

/// The rows one call asks for: the most a page holds.
const PAGE: u32 = 100;

/// Checks every `[case.largest]` of `file` and picks every positive of `core.own-file`: the report (the Files rows,
/// the Browse rows checked and, for a file that lists or unlists a folder, the Folders list) and the picks' findings;
/// `(None, [])` for a file with no `[case.largest]`.
pub(crate) fn check_largest(
    name: &str,
    file: &CaseFile,
    engine: &Engine,
    sandbox: &Path,
) -> Result<(Option<Value>, Vec<Finding>), Failure> {
    if file.cases().iter().all(|case| case.largest().is_none()) {
        return Ok((None, Vec::new()));
    }
    let state = engine
        .disk_state()
        .map_err(|error| failure(name, format!("{error:?}")))?
        .ok_or_else(|| failure(name, "the scan committed no disk state"))?;
    let listed = Listed::read(name, engine, &*state)?;
    let folders = Folders::read(name, engine, &*state)?;
    let not_entered = engine
        .overview()
        .map_err(|error| failure(name, format!("{error:?}")))?
        .last_scan
        .map(|last| last.coverage.not_entered)
        .unwrap_or_default();
    let home = sandbox.join("home");
    let mut places = Vec::new();
    for case in file.cases() {
        let Some(expect) = case.largest() else { continue };
        let at = sandbox.join(expect.at());
        let folder =
            state.find(&at).and_then(|node| state.node(node)).is_some_and(|node| node.kind == NodeKind::Folder);
        if check_case(case, expect, &at, folder, (&listed, &folders), &not_entered, sandbox)? == Next::FolderRow {
            let row = folder_row(case, engine, &*state, &at, &home)?;
            check_folder_row(case.name(), expect, &row)?;
            places.push(json!({
                "case": case.name(),
                "at": expect.at(),
                "kind": row.kind,
                "offer": row.offer,
                "opens": row.opens,
            }));
        }
    }
    let picks = pick(file, engine, &*state, sandbox)?;
    let files: Vec<Value> = listed
        .rows
        .iter()
        .map(|(path, row)| {
            json!({
                "path": relative(path, sandbox),
                "kind": row.kind,
                "offer": row.offer,
                "allocated": row.node.sizes.allocated.bytes(),
                "unpacked": row.unpacked,
            })
        })
        .collect();
    let mut report = json!({ "files": files, "places": places });
    if file
        .cases()
        .iter()
        .filter_map(Case::largest)
        .any(|expect| matches!(expect.shown(), Shown::Listed | Shown::Unlisted))
    {
        report["folders"] = folders
            .rows
            .iter()
            .map(|(path, row)| json!({ "path": relative(path, sandbox), "project": row.project }))
            .collect();
    }
    Ok((Some(report), picks))
}

/// The whole Files list of the last state, no filter, every page, each row with its path.
struct Listed {
    rows: Vec<(PathBuf, LargestRow)>,
}

impl Listed {
    fn read(name: &str, engine: &Engine, state: &dyn DiskState) -> Result<Self, Failure> {
        let mut rows = Vec::new();
        loop {
            let offset = u32::try_from(rows.len()).map_err(|error| failure(name, error))?;
            let query = LargestQuery { kind: None, place: None, unchanged: Unchanged::Any, offset, limit: PAGE };
            let page = engine.largest(&query).map_err(|error| failure(name, format!("Largest: {error:?}")))?;
            let done = page.rows.is_empty() || offset + u32::try_from(page.rows.len()).unwrap_or(0) >= page.total;
            for row in page.rows {
                let path = state
                    .path(row.node.id)
                    .ok_or_else(|| failure(name, format!("Largest: the row {} has no path", row.node.name)))?;
                rows.push((path, row));
            }
            if done {
                return Ok(Self { rows });
            }
        }
    }

    fn at(&self, path: &Path) -> Option<&LargestRow> {
        self.rows.iter().find(|(row, _)| row == path).map(|(_, row)| row)
    }

    fn at_or_below(&self, path: &Path) -> Option<&Path> {
        self.rows.iter().map(|(row, _)| row.as_path()).find(|row| row.starts_with(path))
    }
}

/// The whole Folders list of the last state, every page, each row with its path.
struct Folders {
    rows: Vec<(PathBuf, FolderRow)>,
}

impl Folders {
    fn read(name: &str, engine: &Engine, state: &dyn DiskState) -> Result<Self, Failure> {
        let mut rows = Vec::new();
        loop {
            let offset = u32::try_from(rows.len()).map_err(|error| failure(name, error))?;
            let page = engine
                .largest_folders(FoldersQuery { offset, limit: PAGE })
                .map_err(|error| failure(name, format!("Largest → Folders: {error:?}")))?;
            let done = page.rows.is_empty() || offset + u32::try_from(page.rows.len()).unwrap_or(0) >= page.total;
            for row in page.rows {
                let path = state
                    .path(row.node.id)
                    .ok_or_else(|| failure(name, format!("Folders: the row {} has no path", row.node.name)))?;
                rows.push((path, row));
            }
            if done {
                return Ok(Self { rows });
            }
        }
    }

    fn at(&self, path: &Path) -> Option<&FolderRow> {
        self.rows.iter().find(|(row, _)| row == path).map(|(_, row)| row)
    }
}

/// What is left to check of a case once the Files list is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    /// Nothing: the expectation holds.
    Done,
    /// The Browse row at `at` (a folder expected offered, or withheld).
    FolderRow,
}

/// Checks the case's expectation against the Files list (`lists.0`), the Folders list (`lists.1`) and the last scan's
/// coverage, and says what is left: a folder (`folder`, `at` is one in the last state) expected offered with no kind or
/// folder beside it, which the Files list does not hold, and a withheld one are checked in the Browse view.
fn check_case(
    case: &Case,
    expect: &LargestExpectation,
    at: &Path,
    folder: bool,
    (listed, folders): (&Listed, &Folders),
    not_entered: &[NotEntered],
    sandbox: &Path,
) -> Result<Next, Failure> {
    let shown = expect.at().display();
    let absent = || match listed.at_or_below(at) {
        Some(found) => Err(failure(case.name(), format!("on Largest, {} is listed", relative(found, sandbox)))),
        None => Ok(()),
    };
    match expect.shown() {
        Shown::Offered => match listed.at(at) {
            Some(row) => check_file_row(case.name(), expect, row).map(|()| Next::Done),
            None if folder && expect.kind().is_none() && expect.unpacked().is_none() => Ok(Next::FolderRow),
            None => Err(failure(case.name(), format!("on Largest, {shown} is not listed"))),
        },
        Shown::Absent => absent().map(|()| Next::Done),
        Shown::Withheld => Ok(Next::FolderRow),
        Shown::Listed => match (folders.at(at), expect.project()) {
            (None, _) => Err(failure(case.name(), format!("on Largest → Folders, {shown} is not listed"))),
            (Some(row), Some(project)) if row.project != project => {
                let problem =
                    format!("on Largest → Folders, {shown} is listed with project = {}, not {project}", row.project);
                Err(failure(case.name(), problem))
            }
            (Some(_), _) => Ok(Next::Done),
        },
        Shown::Unlisted => match folders.at(at) {
            Some(_) => Err(failure(case.name(), format!("on Largest → Folders, {shown} is listed"))),
            None => Ok(Next::Done),
        },
        Shown::NotSeen => {
            absent()?;
            let folder = guarded_folder(&sandbox.join("home"), at)
                .ok_or_else(|| failure(case.name(), format!("{shown} is in no guarded folder")))?;
            if !not_entered.iter().any(|entry| entry.capability == Capability::Folder(folder)) {
                return Err(failure(case.name(), format!("the scan entered {folder:?}: {shown} was seen")));
            }
            Ok(Next::Done)
        }
    }
}

/// An offered Files row: offered, of the expected kind, naming the expected folder beside it.
fn check_file_row(case: &str, expect: &LargestExpectation, row: &LargestRow) -> Result<(), Failure> {
    let shown = expect.at().display();
    if row.offer != Offer::Offered {
        return Err(failure(case, format!("on Largest, {shown} cannot be chosen: {:?}", row.offer)));
    }
    if let Some(kind) = expect.kind()
        && row.kind != kind
    {
        return Err(failure(case, format!("on Largest, {shown} is listed as {:?}, not as {kind:?}", row.kind)));
    }
    if expect.unpacked().is_some() && row.unpacked.as_deref() != expect.unpacked() {
        let problem = format!("on Largest, {shown} names {:?} beside it, not {:?}", row.unpacked, expect.unpacked());
        return Err(failure(case, problem));
    }
    Ok(())
}

/// The Browse row at `at`: offered for `offered`, withheld for the expected reason for `withheld`.
fn check_folder_row(case: &str, expect: &LargestExpectation, row: &OwnRow) -> Result<(), Failure> {
    let expected = match expect.withheld() {
        Some(reason) => Offer::Withheld(reason),
        None => Offer::Offered,
    };
    if row.offer != expected {
        let problem = format!("on Largest → Browse, {} is {:?}, not {expected:?}", expect.at().display(), row.offer);
        return Err(failure(case, problem));
    }
    Ok(())
}

/// The row at `at` in the Browse view of its folder (the home's first page when its folder is the home), every page.
fn folder_row(case: &Case, engine: &Engine, state: &dyn DiskState, at: &Path, home: &Path) -> Result<OwnRow, Failure> {
    let node = |path: &Path| {
        state.find(path).ok_or_else(|| failure(case.name(), format!("the scan has no node at {}", path.display())))
    };
    let target = node(at)?;
    let parent = at.parent().unwrap_or(at);
    let folder: Option<NodeId> = if parent == home { None } else { Some(node(parent)?) };
    let mut offset = 0;
    loop {
        let place = engine
            .largest_place(folder, offset, PAGE)
            .map_err(|error| failure(case.name(), format!("Largest → Browse: {error:?}")))?;
        if let Some(row) = place.rows.iter().find(|row| row.node.id == target) {
            return Ok(row.clone());
        }
        offset += u32::try_from(place.rows.len()).unwrap_or(u32::MAX);
        if place.rows.is_empty() || offset >= place.total {
            let problem = format!("Largest → Browse shows no row at {}", at.display());
            return Err(failure(case.name(), problem));
        }
    }
}

/// The guarded folder of the sandbox's home that `at` is at or below.
fn guarded_folder(home: &Path, at: &Path) -> Option<GuardedFolder> {
    [
        (GuardedFolder::Desktop, "Desktop"),
        (GuardedFolder::Documents, "Documents"),
        (GuardedFolder::Downloads, "Downloads"),
    ]
    .into_iter()
    .find(|(_, name)| at.starts_with(home.join(name)))
    .map(|(folder, _)| folder)
}

/// Picks the item of every positive of the core's own category, as a tick on the Largest screen does.
fn pick(file: &CaseFile, engine: &Engine, state: &dyn DiskState, sandbox: &Path) -> Result<Vec<Finding>, Failure> {
    let mut picks = Vec::new();
    for case in file.cases() {
        let Some(expect) = case.expect().filter(|expect| expect.category().as_str() == OWN_FILE_ID) else { continue };
        let item = sandbox.join(expect.item());
        let node = state
            .find(&item)
            .ok_or_else(|| failure(case.name(), format!("the scan has no node at {}", expect.item().display())))?;
        let picked = engine.pick_own_item(node).map_err(|error| {
            failure(case.name(), format!("{} cannot be picked: {error:?}", expect.item().display()))
        })?;
        picks.push(picked);
    }
    Ok(picks)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::GuardedFolder;

    use super::guarded_folder;

    mod negative {
        use super::{Path, guarded_folder};

        #[test]
        fn a_place_outside_the_guarded_folders_has_none() {
            let home = Path::new("/box/home");

            for at in ["/box/home/Movies", "/box/home", "/box/home/Documents2", "/box/Documents"] {
                assert_eq!(guarded_folder(home, Path::new(at)), None, "{at}");
            }
        }
    }

    mod positive {
        use super::{GuardedFolder, Path, guarded_folder};

        #[test]
        fn a_place_in_a_guarded_folder_is_its() {
            let home = Path::new("/box/home");

            assert_eq!(guarded_folder(home, Path::new("/box/home/Documents")), Some(GuardedFolder::Documents));
            assert_eq!(guarded_folder(home, Path::new("/box/home/Desktop/a/b")), Some(GuardedFolder::Desktop));
        }
    }
}
