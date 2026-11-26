# 017/01/02 — A fourth entry kind

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
core, macos, scanner, cleaner, docs

Design section 4; fixes the [known issue](../../../known-issues/fixed/core-fifo-read-as-file.md) "A FIFO is read as a
regular file".

## Files

- `crates/core/src/interest.rs` — `EntryKind::Other`: "a socket, a named pipe or a device: reported as itself, never
  read, never a marker". Every exhaustive `match` follows; `crates/testing/src/tree_report.rs`'s own `EntryKind` gains
  it for the goldens.
- `crates/macos/src/bulk/parse.rs` — the object type `VSOCK`, `VFIFO`, `VCHR`, `VBLK` → `Other` (today they fall to a
  file); `bulk/reader.rs`'s test compares with `lstat`'s file type.
- `crates/scanner/src/` — the reference walk (`NaiveWalk`) reads `Other` from `file_type()`; the matcher never takes an
  `Other` entry as a marker file (`CACHEDIR.TAG` or any other); a `Pattern::Under` or `FolderName` interest matches it as
  itself; its size is its own (zero data); the codec stores the kind (the cache's format version goes up; an older
  cache is a `BrokenCache` full scan, logged).
- `crates/cleaner/src/guard/identity.rs` (check 7) — an item whose kind is `Other` now is refused as changed, whatever
  was confirmed; `crates/cleaner/src/paths.rs` keeps `O_NONBLOCK` (comment points at the test).
- `docs/known-issues/fixed/core-fifo-read-as-file.md` — status `fixed`, the link to this step; its row in the README.

## Produces

`EntryKind::Other` in every `Match`, for 01/03's `Inside.special` and the module.

## Checks

- Negative first: a pipe named `CACHEDIR.TAG` (made with `mkfifo` in the test) is not a marker for the matcher nor the
  reference walk; a socket (bound in the test's own temporary folder, whose path fits the socket's 104 bytes) is never
  a file; the guard opens a pipe with no writer and returns at once (the test fails on a 2-second timeout); a
  confirmed file replaced by a pipe is skipped as changed.
- Positive: `Under(Home, "x/*")` matches a pipe and a socket as `Other`; the bulk reader and the reference walk agree
  on every kind (the existing property test with the new kind); a cache round trip keeps `Other`.
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-scanner -p fresher-cleaner`; every existing golden
  unchanged (no fixture holds a special file).

## Conclusions

Built (9771652, review fixes f066821): `EntryKind::Other`. The bulk read maps every object type but a regular file, a
folder and a link to `Other` (sockets, pipes and devices, and the odd types too: fail closed); the portable reader,
`identity_at`, the reference walk and the cleaner's own kind read it from the file type. The tree keeps an `Other` node
like a link (only when hit, its own size; node flag `Special`, tree format 3). The cleaner's check 7 refuses an `Other`
entry first, whatever was confirmed and in the rehearsal too; the guard keeps `O_NONBLOCK`. The known issue is fixed.

Beyond the brief, fail closed: a rule never claims an `Other` match (`crates/rules/src/evaluate.rs`), and
`fresher-testing` makes pipes (`make_pipe`). Left for later: the browse view's `NodeKind` still shows a special entry as
a file ([deferred](../../../deferred/special-entry-in-browse-view.md)). Ruling: the bulk reader and the reference walk
are compared on fixed trees holding a pipe and a socket here; 01/03 added the property test of the bulk reader.

Checked by mutation: the guard's pipe test fails without `O_NONBLOCK`; the macOS reader's tests fail with the old
mapping; each rules test fails with only one of the two kinds guarded. No golden changed.
