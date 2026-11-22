# 037/03/01 — The sandbox: invariants and case files

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, app, docs

The sandbox autotest asks the engine's Trash view after the scan and after the cleanup of **every** case file, checks
two invariants, and writes both views into the golden report. Two case files of their own, on the `trial` module,
show an empty Trash and a Trash holding the user's own file. The case file format does not change (agent's call 7).
The design: [`../00-design.md`](../00-design.md), section 7.

Read first: `docs/testing.md` "Sandboxes" and "The case file", `app/tauri/tests/sandboxes.rs`,
`app/tauri/tests/sandboxes/harness.rs` (`run`, `clean`, `cleanup_report`), `app/tauri/tests/sandboxes/left_out.rs`
(a report part in a module of its own), `fixtures/sandboxes/trial.toml`, `fixtures/sandboxes/cleanup-guards.toml`
(a file inside `home/.Trash`), `fixtures/golden/sandbox-trial.json`.

## Files

- `app/tauri/tests/sandboxes/trash.rs` (new) — test code, wired in `sandboxes.rs` with `#[cfg(test)] #[path =
  "sandboxes/trash.rs"] mod trash;`:
  - `pub(crate) fn report(view: &TrashView) -> Value` — the lines as JSON (`index`, `volume`, `waiting` allocated or
    null, `fromCleanups` allocated, `goneBy`), no path.
  - `pub(crate) fn check_before(name: &str, view: &TrashView) -> Result<(), Failure>` — no line has a share above zero
    ("nothing came from cleanups before the first one").
  - `pub(crate) fn check_after(name: &str, view: &TrashView, detail: Option<&CleanupDetail>) -> Result<(), Failure>` —
    the expected share is the sum of `sizes.allocated` of the detail's items whose `trash` is `Some(InTrash)` (zero
    with no cleanup); when it is above zero, exactly one line has a share, it is line 0 (the sandbox's `home/.Trash`),
    its `fromCleanups.allocated` equals it, and its `waiting` is `Some` and not below it; when it is zero, no line has a
    share. Each failure names the case file and both numbers.
- `app/tauri/tests/sandboxes/harness.rs` — in `run`: `let trash_before = engine.trash();` after the scan's checks
  (before `clean`), `check_before`; `let trash_after = engine.trash();` after the cleanup, `check_after`; the report
  gains `"trash": { "before": …, "after": … }`.
- `app/tauri/tests/sandboxes.rs` — in `mod negative`: `check_before` refuses a view with a share; `check_after`
  refuses a share that differs from the detail's, a share in a line other than 0, a share with no `waiting`, and a
  second line with a share (views and details built in the test). In `case_files!`: `trash => "trash"` and
  `trash_empty => "trash-empty"`, sorted.
- `fixtures/sandboxes/trash-empty.toml` (new) — `feature = "trash-empty"`, `module = "trial"`, a head comment: "The
  Trash line with an empty Trash: nothing is said before the cleanup; after it, everything in the Trash is from
  cleanups here."
  - negative: "a plain file next to a marker is never offered" — marker `home/trash-empty-plain`, file
    `home/trash-empty-plain/keep.txt` (512 bytes).
  - positive: "a trial cache goes to the Trash and is all the Trash holds" — marker `home/trash-empty-work`, file
    `home/trash-empty-work/fresher-trial-cache/blob.bin`, 1 MiB, `fill = "data"`; expect `trial.caches`, `safe`,
    `in-trash`.
- `fixtures/sandboxes/trash.toml` (new) — `feature = "trash"`, `module = "trial"`, head comment: "The Trash line: what
  the user put in the Trash counts in the first number only; a cleanup adds to both."
  - negative: "a file you put in the Trash is never offered and counts only in the first number" — file
    `home/.Trash/old-report.pdf`, 2 MiB, `fill = "data"`.
  - positive: "a trial cache goes to the Trash and adds to both numbers" — marker `home/trash-work`, file
    `home/trash-work/fresher-trial-cache/blob.bin`, 4 MiB, `fill = "data"`; expect `trial.caches`, `safe`,
    `in-trash`.
  The names are apart from every other file's, so `npm run sandbox -- all` leaves out none of them.
- `fixtures/golden/sandbox-trash.json`, `fixtures/golden/sandbox-trash-empty.json` (new) and **every**
  `fixtures/golden/sandbox-*.json` — `UPDATE_GOLDEN=1 npm run sandbox:test`; the diff is read: every other golden
  only gains its `trash` section; `trash-empty`'s `before` is `[]` and its `after` one line whose `waiting` equals its
  `fromCleanups`; `trash`'s `before` one line with `waiting` the PDF's blocks and `fromCleanups` 0, and its `after`
  the PDF's blocks plus the cache's, with `fromCleanups` the cache's. `goneBy` is `null` everywhere (the autotest's
  fake command runner answers no `defaults`).
- `docs/testing.md` — "Sandboxes": the two case files in the list (one sentence each, as above), and one paragraph
  on the Trash view in every golden and its two invariants (what they check, that a split of one Trash into two lines
  fails them).

## Checks

- `npm run sandbox:test` green for every case file.
- By hand, once, each reverted: change `fromCleanups` in `fixtures/golden/sandbox-trash.json` → the run fails naming
  the file; make `check_after` expect line 1 → the run fails. Write both in the conclusions.
- `npm run sandbox -- trash` and `-- trash-empty` generate their sandboxes; `EXPECTED.md` reads one checkbox per
  case, negatives first.
- clippy (`--all-targets`), fmt.

## Conclusions

- Built as written. Every case file's run reads the Trash view after the scan's checks and after the cleanup, checks
  both invariants and writes `trash.before` and `trash.after` into its report; `sandbox:test` passes for all 34 case
  files (76 tests, eight new negatives on the invariants).
- The goldens: the 32 existing ones only gained their `trash` section (checked by comparing each with its previous
  version once the section is taken out). A file with nothing moved to the Trash has `[]` both times; every other
  `after` is one line, index 0, volume `Sandbox`, `waiting` equal to `fromCleanups`, except where the case file puts
  a file of its own in `home/.Trash` (`cleanup-guards`: 4 KiB before, 4 KiB more after; `storage-map`: 4 MiB before,
  4 MiB more after). `trash-empty`: `before` `[]`, `after` 1 MiB both numbers. `trash`: `before` 2 MiB waiting and 0
  from cleanups, `after` 6 MiB waiting and 4 MiB from cleanups. `goneBy` is `null` everywhere.
- By hand, each reverted: `fromCleanups` of `sandbox-trash.json` changed by one byte → `trash.toml: the report
  differs from …/sandbox-trash.json`; `check_after` made to expect line 1 → `trash.toml: case \`trash\`: the Trash
  after the cleanup: expected 4194304 bytes from cleanups in line 0, line 0 has 4194304`.
- `npm run sandbox -- trash` and `-- trash-empty` write their sandboxes; each `EXPECTED.md` has one checkbox per case,
  the negative first; `npm run sandbox -- all` keeps both files.
