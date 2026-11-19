//! The storage map of a case file's sandbox, read after the scan and before the cleanup: its parts add up to the fake
//! volume's capacity and both views have one total, and every `[case.map]` holds in the by-place view.

use std::path::Path;

use fresher_core::{
    ByteSize, Descent, DiskSpace, DiskState, Finding, FindingsQuery, MeaningGroup, NodeId, PartKind, PlaceRow,
    PlaceView, Sizes, VolumeMap,
};
use fresher_engine::Engine;
use fresher_sandbox::{Case, CaseFile, MapExpectation, MapPartName};
use fresher_testing::FakeVolumes;
use serde_json::{Value, json};

use crate::harness::{Failure, failure, path_of, relative};

const MIB: u64 = 1024 * 1024;

/// The fake volume's capacity: 1 GiB.
const CAPACITY: u64 = 1024 * MIB;

/// The fake volume the sandbox is on: 1 GiB, the files of the tree and 64 MiB more used, 256 MiB free, so the
/// remainder and Free are the same on every machine. The tree's size is capped to what the volume can hold: a sparse
/// file of a case file (2 GB) counts its logical size but takes no block.
pub(crate) fn sandbox_disk(tree_size: u64) -> DiskSpace {
    let room = CAPACITY - 64 * MIB - 256 * MIB;
    FakeVolumes::disk(
        "Sandbox",
        ByteSize::new(CAPACITY),
        ByteSize::new(tree_size.min(room) + 64 * MIB),
        ByteSize::new(256 * MIB),
    )
}

/// Checks the map of the scanned sandbox and every case's `[case.map]`; the report of the map's parts and of the place
/// rows of every `at`.
pub(crate) fn check_map(
    name: &str,
    file: &CaseFile,
    engine: &Engine,
    sandbox: &Path,
    findings: &[Finding],
) -> Result<Value, Failure> {
    let state = engine
        .disk_state()
        .map_err(|error| failure(name, format!("{error:?}")))?
        .ok_or_else(|| failure(name, "the scan committed no disk state"))?;
    let map = engine.storage_map().map_err(|error| failure(name, format!("the map: {error:?}")))?;
    let mut volumes = Vec::new();
    for (index, volume) in map.volumes.iter().enumerate() {
        let groups = engine.storage_meaning(index).map_err(|error| failure(name, format!("by meaning: {error:?}")))?;
        check_totals(name, volume, &groups, &*state)?;
        volumes.push(volume_report(volume));
    }
    let mut places = Vec::new();
    for case in file.cases() {
        if let Some(expect) = case.map() {
            places.push(check_place(case, expect, engine, &*state, sandbox, findings)?);
        }
    }
    Ok(json!({ "volumes": volumes, "places": places }))
}

/// The parts add up to the fake volume's capacity, and so do the by-meaning groups and the roots' sizes with the parts
/// that are no file of the scan.
///
/// These are invariants of the engine (a `VolumeMap` is built only when its parts add up): no case file can break
/// them, so they have no failing test of their own here; the engine's tests cover them.
fn check_totals(name: &str, volume: &VolumeMap, groups: &[MeaningGroup], state: &dyn DiskState) -> Result<(), Failure> {
    let parts: u64 = volume.parts().iter().map(|part| part.sizes.allocated.bytes()).sum();
    if volume.capacity().bytes() != CAPACITY || parts != CAPACITY {
        let problem = format!("the map's parts add up to {parts} of {}, not to {CAPACITY}", volume.capacity().bytes());
        return Err(failure(name, problem));
    }
    let by_meaning: u64 = groups.iter().map(|group| group_size(group).bytes()).sum();
    let scanned: u64 =
        volume.roots().iter().filter_map(|&root| state.node(root)).map(|node| node.sizes.allocated.bytes()).sum();
    let not_files: u64 = volume
        .parts()
        .iter()
        .filter(|part| !matches!(part.kind, PartKind::Explained | PartKind::YourFolders))
        .map(|part| part.sizes.allocated.bytes())
        .sum();
    if by_meaning != CAPACITY || scanned + not_files != CAPACITY {
        let problem =
            format!("by meaning {by_meaning}, by place {}, the map {CAPACITY}: not one total", scanned + not_files);
        return Err(failure(name, problem));
    }
    Ok(())
}

fn group_size(group: &MeaningGroup) -> ByteSize {
    match group {
        MeaningGroup::App { sizes, .. } => sizes.allocated,
        MeaningGroup::Category(share) => share.sizes.allocated,
        MeaningGroup::Place(share) => share.sizes.allocated,
        MeaningGroup::Part(part) => part.sizes.allocated,
    }
}

/// The place `at` opened as the overview opens a folder row: the rows that hold the case's files are explained in
/// full, or not at all with no finding at, below or above `at`. A place that opens with no rows (a folder the scan
/// does not go into, such as a claimed one, a folder of small files only, which the scan folds into it, or a file) is
/// checked by its own row, in its folder's view.
fn check_place(
    case: &Case,
    expect: &MapExpectation,
    engine: &Engine,
    state: &dyn DiskState,
    sandbox: &Path,
    findings: &[Finding],
) -> Result<Value, Failure> {
    let at = sandbox.join(expect.at());
    let shown = expect.at().display();
    let claimed = |found: &&Path| found.starts_with(&at) || at.starts_with(found);
    if expect.part() == MapPartName::YourFolders
        && let Some(found) = findings.iter().filter_map(path_of).find(claimed)
    {
        let problem = format!("by place, {shown} is explained: {} is offered", relative(found, sandbox));
        return Err(failure(case.name(), problem));
    }
    let find = |path: &Path| {
        state.find(path).ok_or_else(|| failure(case.name(), format!("the scan has no node at {}", path.display())))
    };
    let node = find(&at)?;
    let view = open(case, engine, node, Descent::Through)?;
    let rows: Vec<PlaceRow> = if view.total == 0 {
        let folder = find(at.parent().unwrap_or(&at))?;
        let around = open(case, engine, folder, Descent::Stay)?;
        around.rows.into_iter().filter(|row| row.node.id == node).collect()
    } else {
        view.rows.clone()
    };
    let rows: Vec<ShownRow> =
        rows.iter().map(|row| ShownRow::of(case, row, state, sandbox)).collect::<Result<_, _>>()?;
    let own: Vec<&Path> = case.tree().paths().chain(case.tree().markers()).collect();
    check_rows(case.name(), &own, expect, &rows)?;
    let path: Vec<String> = view
        .path
        .iter()
        .map(|node| {
            let path = state.path(node.id).ok_or_else(|| failure(case.name(), "by place: a folder has no path"))?;
            Ok(relative(&path, sandbox))
        })
        .collect::<Result<_, Failure>>()?;
    let rows: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "path": row.path,
                "logical": row.sizes.logical.bytes(),
                "allocated": row.sizes.allocated.bytes(),
                "explained": row.explained.allocated.bytes(),
            })
        })
        .collect();
    Ok(json!({ "case": case.name(), "at": relative(&at, sandbox), "path": path, "rows": rows }))
}

/// A row of the by-place view as the check reads it: its path relative to the sandbox, its size and how much of it
/// is explained.
#[derive(Debug)]
struct ShownRow {
    path: String,
    sizes: Sizes,
    explained: Sizes,
}

impl ShownRow {
    fn of(case: &Case, row: &PlaceRow, state: &dyn DiskState, sandbox: &Path) -> Result<Self, Failure> {
        let path = state
            .path(row.node.id)
            .ok_or_else(|| failure(case.name(), format!("by place: the row {} has no path", row.node.name)))?;
        Ok(Self { path: relative(&path, sandbox), sizes: row.node.sizes, explained: row.explained })
    }
}

/// The rows that hold the case's files (`own`: its paths and markers) are explained in full, or for `your-folders`
/// not at all; at least one row holds them.
fn check_rows(case: &str, own: &[&Path], expect: &MapExpectation, rows: &[ShownRow]) -> Result<(), Failure> {
    let checked: Vec<&ShownRow> =
        rows.iter().filter(|row| own.iter().any(|mine| mine.starts_with(&row.path))).collect();
    if checked.is_empty() {
        let problem = format!("by place, {} shows none of the case's files", expect.at().display());
        return Err(failure(case, problem));
    }
    for row in checked {
        let holds = match expect.part() {
            MapPartName::YourFolders => {
                row.explained.allocated == ByteSize::ZERO && row.explained.logical == ByteSize::ZERO
            }
            MapPartName::Explained => {
                row.explained.allocated == row.sizes.allocated && row.explained.logical == row.sizes.logical
            }
        };
        if !holds {
            let problem = format!(
                "by place, {} of the {} bytes of {} are explained, expected {:?}",
                row.explained.allocated.bytes(),
                row.sizes.allocated.bytes(),
                row.path,
                expect.part()
            );
            return Err(failure(case, problem));
        }
    }
    Ok(())
}

/// A place view holds all its rows: the check reads one page.
fn one_page(case: &str, view: &PlaceView) -> Result<(), Failure> {
    if view.total as usize > view.rows.len() {
        return Err(failure(case, format!("by place: {} rows, more than one page", view.total)));
    }
    Ok(())
}

/// One page of the place `node`, all its rows.
fn open(case: &Case, engine: &Engine, node: NodeId, descent: Descent) -> Result<PlaceView, Failure> {
    let view = engine
        .storage_place(node, descent, 0, FindingsQuery::MAX_LIMIT)
        .map_err(|error| failure(case.name(), format!("by place: {error:?}")))?;
    one_page(case.name(), &view)?;
    Ok(view)
}

fn volume_report(volume: &VolumeMap) -> Value {
    let parts: Vec<Value> = volume
        .parts()
        .iter()
        .map(|part| json!({ "part": part.kind, "allocated": part.sizes.allocated.bytes() }))
        .collect();
    json!({ "name": volume.name(), "capacity": volume.capacity().bytes(), "parts": parts })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::{ByteSize, Sizes};
    use fresher_sandbox::{CaseFile, MapExpectation};

    use super::ShownRow;

    /// The map expectation of a one-case file whose case writes `home/a/one.bin` and `home/a/two.bin`.
    fn expectation(kind: &str, part: &str) -> MapExpectation {
        let expect = if kind == "positive" {
            "[case.expect]\nitem = \"home/a\"\ncategory = \"trial.caches\"\ndanger = \"safe\"\nafter-cleanup = \"in-trash\"\n"
        } else {
            ""
        };
        let text = format!(
            "feature = \"trial\"\n[[case]]\nname = \"c\"\nkind = \"{kind}\"\n[[case.file]]\npath = \"home/a/one.bin\"\n\
             [[case.file]]\npath = \"home/a/two.bin\"\n{expect}[case.map]\npart = \"{part}\"\n"
        );
        CaseFile::parse(&text).unwrap().cases()[0].map().unwrap().clone()
    }

    const OWN: [&str; 2] = ["home/a/one.bin", "home/a/two.bin"];

    fn own() -> Vec<&'static Path> {
        OWN.iter().map(Path::new).collect()
    }

    fn row(path: &str, size: u64, explained: u64) -> ShownRow {
        let sizes = |bytes| Sizes::new(ByteSize::new(bytes), ByteSize::new(bytes), ByteSize::new(bytes));
        ShownRow { path: path.to_owned(), sizes: sizes(size), explained: sizes(explained) }
    }

    mod negative {
        use fresher_core::PlaceView;

        use super::{expectation, own, row};
        use crate::map::{check_rows, one_page};

        #[test]
        fn a_place_that_shows_none_of_the_cases_files_fails() {
            let rows = [row("home/a/other.bin", 4096, 0), row("home/b", 4096, 0)];

            let failure = check_rows("c", &own(), &expectation("negative", "your-folders"), &rows).unwrap_err();

            assert_eq!(failure.case, "c");
            assert!(failure.problem.contains("home/a shows none of the case's files"), "{failure}");
        }

        #[test]
        fn a_your_folders_row_with_anything_explained_fails() {
            let rows = [row("home/a/one.bin", 4096, 0), row("home/a/two.bin", 8192, 4096)];

            let failure = check_rows("c", &own(), &expectation("negative", "your-folders"), &rows).unwrap_err();

            assert!(failure.problem.contains("4096 of the 8192 bytes of home/a/two.bin are explained"), "{failure}");
        }

        #[test]
        fn an_explained_row_not_explained_in_full_fails() {
            let rows = [row("home/a", 8192, 4096)];

            let failure = check_rows("c", &own(), &expectation("positive", "explained"), &rows).unwrap_err();

            assert!(failure.problem.contains("4096 of the 8192 bytes of home/a are explained, expected Explained"));
        }

        #[test]
        fn a_place_of_more_than_one_page_fails() {
            let view = PlaceView { path: Vec::new(), rows: Vec::new(), offset: 0, total: 501 };

            let failure = one_page("c", &view).unwrap_err();

            assert!(failure.problem.contains("501 rows, more than one page"), "{failure}");
        }
    }

    mod positive {
        use super::{expectation, own, row};
        use crate::map::check_rows;

        #[test]
        fn rows_of_other_files_are_not_checked() {
            let rows = [row("home/a/one.bin", 4096, 0), row("home/b", 4096, 4096)];

            assert!(check_rows("c", &own(), &expectation("negative", "your-folders"), &rows).is_ok());
        }

        #[test]
        fn a_folder_row_explained_in_full_holds() {
            let rows = [row("home/a", 8192, 8192)];

            assert!(check_rows("c", &own(), &expectation("positive", "explained"), &rows).is_ok());
        }
    }
}
