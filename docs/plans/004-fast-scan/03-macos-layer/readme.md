# 004/03 — macOS layer

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos, workspace, scripts

`fresher-macos`: the only crate with `unsafe`, and the macOS adapters of the core's OS ports. Every FFI call lives in
its module `ffi`, one unsafe operation per block, each with a `// SAFETY:` comment stating the invariant; everything
else — parsing the kernel's buffers, error mapping, the adapters — is safe Rust. The design:
[`../00-design.md`](../00-design.md), sections 1 and 3.

Depends on 02 (the ports) and on the answers of 01. Runs in parallel with 04 and 06/01: it owns `crates/macos/`; its
first step also touches the workspace's lint setup and every crate root, so that step is merged first in the wave.
Tests run on temporary folders and on disk images the test creates (`hdiutil`, in tests marked as such); nothing
reads the real home folder.

## Steps

| #   | Step                                                          | Status   | Shared files                                                                                      |
| --- | ------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------- |
| 01  | [The crate and the unsafe boundary](./01-crate-and-unsafe.md) | ✔️ done | root `Cargo.toml`, every crate root, `scripts/check-crate-deps.ts`, `CLAUDE.md`, `docs/quality.md` |
| 02  | [The bulk read](./02-bulk-read.md)                            | ✔️ done | —                                                                                                 |
| 03  | [Volumes and the places that prompt](./03-volumes-and-prompts.md) | ✔️ done | —                                                                                             |
| 04  | [The change journal](./04-change-journal.md)                  | ✔️ done | —                                                                                                 |

01 first; then 02, 03 and 04 in any order.

## Conclusions

- `fresher-macos` is in place: the only crate with `unsafe`, all of it in `src/ffi/`, one operation per block with its
  `// SAFETY:` comment; the workspace lint is `deny`, every other crate root forbids `unsafe`, and the crate guard
  checks it. The crate gives `MacFolderReader` (`FolderReader`), `MacVolumes` (`Volumes`), `MacChangeJournal`
  (`ChangeJournal`), `io_policy`, `peak_memory`, `places_that_prompt` and, behind `fixtures`, `clone_file`.
- No contract of the core was missing.
- Deviations, each in its step: the bulk read asks for the extended flags instead of the private size, which is read
  per clone (01's measurement, as the design says); a folder swapped for a link is `Replaced` although the open fails
  with `ENOTDIR`; a path past `PATH_MAX` is opened name by name; a volume with no UUID gets its id from its type and
  mount source (`fsid_t` is private in `libc`); the journal flushes after `HistoryDone`, not before, so a cancel is
  heard during a long replay; the disk image tests skip themselves when `hdiutil` is missing instead of `#[ignore]`.
- Tests run on temporary folders and on disk images the tests mount with `-nobrowse`; nothing reads the home folder.
  69 tests in the crate, 96 % of its lines covered.
