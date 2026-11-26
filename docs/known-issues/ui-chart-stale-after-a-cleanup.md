# The disk chart keeps its numbers after a cleanup

**Status:** fixed (the map takes out what a cleanup removed; the volume is read again) · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 (`npm run dev:sandbox -- trial --real-trash`): after a cleanup to the
Trash and then a permanent delete of a 100 MB folder, the disk chart on the scan screen still showed the numbers it
had before the cleanups.

## Cause

The engine, not the UI. The UI marks its state stale on `findings-removed` and `cleanup-finished`, and the refresher
asks `get_storage_map` again (`app/ui/src/state/refresh.ts`); the engine answered the same map, so the reducer kept the
old one. The complete map (`complete` in `crates/engine/src/storage/query.rs`) is built only from the last scan's tree
sizes, its shares (what the modules explain, "Can be freed") and the volumes read at the scan's commit. A cleanup took
its findings out of the results and the overview, but changed none of these: "Can be freed" kept the removed bytes, and
Free kept the volume's space as it was at the scan. The sandbox's real-volume rounding was not the cause: the
"Can be freed" part is the sandbox's own sizes, and it did not move either.

Two tests reproduced it (`crates/engine/src/storage/query_tests.rs`):
`a_move_to_the_trash_leaves_what_can_be_freed_and_stays_in_your_folders` and
`a_permanent_delete_leaves_the_map_with_the_volume_read_again` saw the map after the cleanup equal to the one before.

## Fix

The cleanup keeps for the map every path it removed whole (the paths that went and every folder emptied with no entry
left), with how it went, and at its end reads the volumes of the state's roots again
(`Results::keep_cleaned`, `crates/engine/src/cleanup/operation.rs`). The complete map takes the tree's sizes of those
paths out (`cleaned_under` in `query.rs`): a permanent delete leaves the root and Explained; a move to the Trash leaves
Explained and stays in the root among Your folders, since the Trash is the user's folder. With the volume read again,
Free and the remainder follow. Nothing is read on a query and nothing is scanned: the reads are part of the cleanup the
user confirmed. The next scan forgets the list with the state.

Left as it was:

- the by-meaning groups (`storage_meaning`, which no screen asks for now) stay those of the scan until the next one;
- a put back does not bring its bytes back to Explained, as it does not bring its findings back;
- a cleanup its tool runs (`ItemOutcome::ToolDone`) moves only Free: it removes no path of the tree, so "Can be freed"
  keeps its bytes until the next scan;
- a root that is neither the home folder nor a volume still counts the bytes moved to the Trash among its Your
  folders, though the Trash that holds them is outside it.

## What was tried

Nothing else.
