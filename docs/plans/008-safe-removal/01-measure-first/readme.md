# 008/01 — Measure first

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos (probes only)

Before any product code, throwaway probes answer what the design rests on: what the system Trash call does on each
kind of volume and what it returns, whether the app can see and put back what it moved without extra permissions, how
links, mount points and open files behave, and how soon freed space shows. The probes are small Rust binaries in
`NO_COMMIT/probes/008/` (ignored by git, never committed); only their answers reach the repository, in step 03. The
design: [`../00-design.md`](../00-design.md), sections 5, 6 and 8.

Rules for the probes:

- **Nothing outside their own folders.** A probe creates its test items in a temporary folder or on a disk image it
  mounts itself (`hdiutil attach -nobrowse`), and moves to the Trash only what it created. It never touches an item it
  did not make, and it puts back or deletes what it trashed at the end.
- The developer's home folder is read only for counts (processes, open files); no name or path reaches a file or the
  plan.
- No probe changes a permission or answers a system prompt for the user; a prompt that appears is the finding.
- The probes may use `unsafe` and any crate freely: they are scratch work; plan 03 rewrites what survives behind
  `fresher-macos`'s `ffi` module.
- Each probe runs twice where privacy matters: as a binary started from Finder without Full Disk Access, and with it
  (plan 006's call 23: a binary run from a terminal borrows the terminal's grants).

## Steps

| #   | Step                                                            | Status   | Shared files                               |
| --- | --------------------------------------------------------------- | -------- | ------------------------------------------ |
| 01  | [The Trash and put back](./01-trash-and-put-back.md)            | ✔️ done    | —                                          |
| 02  | [Links, mounts and open files](./02-links-mounts-open-files.md) | ✔️ done    | —                                          |
| 03  | [The answers into the design](./03-answers-into-the-design.md)  | ✔️ done    | `docs/plans/008-safe-removal/00-design.md` |

Steps 01 and 02 are independent and may run in parallel (separate probe folders); 03 waits for both.

## Conclusions

Done on 2026-11-16 on one machine (Mac15,7, Apple M3 Pro, macOS 27.2.1 (26B131), internal SSD, APFS), from a terminal
without Full Disk Access. The answers are in the design, "Measured (01)"; each agent's call they touch carries its
outcome.

- **The plan goes on**: no premise breaks. The Trash call follows no link inside the item, keeps its identity, never
  deletes when it refuses, and needs no permission; put back and `locate` work without Full Disk Access; the
  open-files snapshot is fast (7–19 ms) and raises no prompt; freed space shows at once.
- **Changed**: the post-check after a move takes the landed path from the kept handle, because a parent swapped for a
  link makes the call move the item behind it and sometimes return no path (call 37, a change to plan 02's `Trash`
  port — for the user to confirm); put back adds `RENAME_NOFOLLOW_ANY` (call 23); mapped files count as in use
  (call 38); the process's name comes from its app bundle (call 35); `NoTrash` is code 3328.
- **Needs the user** (08/02 by hand): Finder's own Put Back on screen, the sound of the call, a run with Full Disk
  Access, an exFAT or FAT32 drive (their images do not mount on this macOS) and a network share; macOS versions
  before 27.
- The probes stay in `NO_COMMIT/probes/008/` (`trash`, `guard`), with their `RESULTS.md`.
