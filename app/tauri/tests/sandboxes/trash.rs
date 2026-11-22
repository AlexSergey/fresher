//! The Trash view of a case file's run: what the engine says waits in each Trash before and after the cleanup, in the
//! report without any path, and its two invariants. Nothing came from cleanups before the first one; after it, the
//! sandbox's Trash (`home/.Trash`, line 0) holds as its share exactly the allocated sizes of the cleanup's items found
//! in the Trash, in that one line: a split of one Trash into two lines fails.

use fresher_core::{ByteSize, CleanupDetail, TrashState, TrashView};
use serde_json::{Value, json};

use crate::harness::{Failure, failure};

/// The lines of `view` as JSON: the sizes on disk and the date, no path.
pub(crate) fn report(view: &TrashView) -> Value {
    view.trashes
        .iter()
        .map(|line| {
            json!({
                "index": line.index,
                "volume": line.volume,
                "waiting": line.waiting.map(|waiting| waiting.allocated),
                "fromCleanups": line.from_cleanups.allocated,
                "goneBy": line.gone_by,
            })
        })
        .collect()
}

/// Before the cleanup: no line has a share, as nothing came from cleanups before the first one.
pub(crate) fn check_before(name: &str, view: &TrashView) -> Result<(), Failure> {
    match view.trashes.iter().find(|line| line.from_cleanups.allocated > ByteSize::ZERO) {
        None => Ok(()),
        Some(line) => Err(failure(
            name,
            format!(
                "the Trash before the cleanup: expected 0 bytes from cleanups, line {} has {}",
                line.index,
                line.from_cleanups.allocated.bytes()
            ),
        )),
    }
}

/// After the cleanup (`detail`, `None` when none ran): the share is the allocated sizes of the cleanup's items found
/// in the Trash, all of it in line 0, whose `waiting` is known and holds at least that; no share when that is zero.
pub(crate) fn check_after(name: &str, view: &TrashView, detail: Option<&CleanupDetail>) -> Result<(), Failure> {
    let expected: u64 = detail
        .into_iter()
        .flat_map(|detail| &detail.items)
        .filter(|item| item.trash == Some(TrashState::InTrash))
        .map(|item| item.sizes.allocated.bytes())
        .sum();
    let with_share: Vec<_> = view.trashes.iter().filter(|line| line.from_cleanups.allocated > ByteSize::ZERO).collect();
    let problem = |text: String| Err(failure(name, format!("the Trash after the cleanup: {text}")));
    match with_share.as_slice() {
        [] if expected == 0 => Ok(()),
        [] => problem(format!("expected {expected} bytes from cleanups in line 0, no line has any")),
        [line] => {
            let share = line.from_cleanups.allocated.bytes();
            if expected == 0 {
                return problem(format!("expected 0 bytes from cleanups, line {} has {share}", line.index));
            }
            if line.index != 0 {
                return problem(format!(
                    "expected {expected} bytes from cleanups in line 0, line {} has {share}",
                    line.index
                ));
            }
            if share != expected {
                return problem(format!("expected {expected} bytes from cleanups in line 0, it has {share}"));
            }
            match line.waiting {
                Some(waiting) if waiting.allocated.bytes() >= share => Ok(()),
                Some(waiting) => problem(format!(
                    "line 0 holds {share} bytes from cleanups but only {} waiting",
                    waiting.allocated.bytes()
                )),
                None => problem(format!("line 0 holds {share} bytes from cleanups and no waiting measure")),
            }
        }
        lines => {
            let listed: Vec<String> = lines
                .iter()
                .map(|line| format!("line {} has {}", line.index, line.from_cleanups.allocated.bytes()))
                .collect();
            problem(format!("expected {expected} bytes from cleanups in line 0 only, {}", listed.join(", ")))
        }
    }
}
