# 035/04/02 — The sandbox case file

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, app, fixtures, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the screen ships its case file, negatives first, with its golden
report, and the autotest checks what Largest lists, withholds and does not see. A new optional table on a case,
`[case.largest]`, read by `fresher-sandbox` and checked by the app's sandbox autotest (as `[case.map]` is), and the
`EXPECTED.md` lines for the walk by hand. The design: [`../00-design.md`](../00-design.md), section 9, and agent's
call 6 (small files filled with data, a 10 MB minimum).

Read first: `docs/testing.md`, "Sandboxes" and "The case file"; `crates/sandbox/src/case.rs` (`RawCase`, `RawMap`,
`MapExpectation`: the model of an optional table), `crates/sandbox/src/expected.rs`, `crates/sandbox/src/error.rs`,
`app/tauri/tests/sandboxes.rs` (`case_files!`), `app/tauri/tests/sandboxes/harness.rs` (`run`), and
`app/tauri/tests/sandboxes/map.rs` (`check_map`, the model of the new check), `fixtures/sandboxes/storage-map.toml`.

## Files

- `crates/sandbox/src/case.rs` — `[case.largest]` on any case, `deny_unknown_fields`:

  ```toml
  [case.largest]
  shown = "offered"          # offered | absent | withheld | not-seen
  at = "home/VMs/Old.utm"    # optional for offered and absent (default: the case's first file); required otherwise
  kind = "virtual-machine"   # optional, offered only: an ItemKind as serialized
  withheld = "standard-folder"   # required for withheld, refused otherwise: a Withheld as serialized
  unpacked = "Photos"        # optional, offered only: the folder the row names beside an archive
  ```

  `pub struct LargestExpectation { shown: Shown, at: PathBuf, kind: Option<ItemKind>, withheld: Option<Withheld>,
  unpacked: Option<String> }`, `pub enum Shown { Offered, Absent, Withheld, NotSeen }`, `Case::largest()`; each
  refusal a `CaseError` variant with its message (an unknown `shown`, `withheld` without `shown = "withheld"` or
  missing with it, `kind` or `unpacked` without `offered`, an `at` that is not a plain path inside the sandbox). The
  list of a case's non-file keys in the comment above `RawCase` gains `largest`.
- `crates/sandbox/src/expected.rs` — one line per `[case.largest]` under its case in `EXPECTED.md`:
  - offered: "On Largest, `<at>` is listed as <kind's text> and can be chosen." (+ " It names `<unpacked>` beside it.");
  - absent: "On Largest, nothing at or below `<at>` is listed.";
  - withheld: "On Largest → Folders, `<at>` cannot be chosen: <the reason's sentence>.";
  - not-seen: "On Largest, nothing in `<at>` is listed, and the notice above the list says Fresher did not look there
    and how to allow it.";
  the texts read from `crates/core/locales/en.json` as the generator reads the others (or the reason's id when the
  generator reads no texts: follow what it does). Its tests, negatives first.
- `fixtures/sandboxes/large-files.toml` (new) — `feature = "large-files"`, `module = "trial"`, `[settings]`
  `"core.largest.min-size" = "10-mb"`; a head comment: what the file covers, that the list ranks by allocated bytes so
  listed files are filled with data (12 to 16 MiB), that nothing lies in `Downloads` or `Desktop` (other case files'
  worlds deny them in `npm run sandbox -- all`), that a guarded folder denied is this file's own `Documents`, and that
  an app bundle's inside is the core e2e's (a `[[case.app]]` writes only its own small bundle).
  These cases, these names, in this order:
  - negative:
    - "a file under Library is never listed" — `home/Library/Application Support/Big/data.bin` (12 MiB data);
      `shown = "absent"`;
    - "a file in the Trash is never listed" — `home/.Trash/old.dmg` (12 MiB data); `absent`;
    - "a hidden folder is never listed" — `home/.tool/disk.img` (12 MiB data); `absent`;
    - "a package's inner file is not a row of its own" — `home/VMs/Inner.utm/Images/disk.qcow2` (12 MiB data);
      `absent` at `home/VMs/Inner.utm/Images`;
    - "a file below the minimum is not listed" — `home/Archives/small.zip` (8 MiB data); `absent`;
    - "a standard folder cannot be chosen" — `home/Movies/notes.txt` (4 KiB); `shown = "withheld"`,
      `at = "home/Movies"`, `withheld = "standard-folder"`;
    - "nothing in a denied guarded folder is seen" — `access = { documents = "denied" }`,
      `home/Documents/report.iso` (12 MiB data); `shown = "not-seen"`, `at = "home/Documents"`;
  - positive:
    - "a trial cache is its module's and never listed" — a marker `home/project`, `home/project/fresher-trial-cache/blob.bin`
      (12 MiB data); `[case.expect]` `trial.caches`, safe, in-trash; `[case.largest]` `absent` at
      `home/project/fresher-trial-cache`;
    - "a large disk image is listed and goes to the Trash" — `home/Images/Installer.dmg` (16 MiB data);
      `[case.expect]` item it, `core.own-file`, high, in-trash; `[case.largest]` `offered`, `kind = "disk-image"`;
    - "a package is one row" — `home/VMs/Old.utm/Images/disk.qcow2` (14 MiB data); `[case.expect]` item
      `home/VMs/Old.utm`, `core.own-file`, high, in-trash; `[case.largest]` `offered`, `at = "home/VMs/Old.utm"`,
      `kind = "virtual-machine"`;
    - "an archive beside its unpacked folder is pointed out" — `home/Archives/Photos.zip` (12 MiB data) and
      `home/Archives/Photos/one.jpg` (4 KiB); `[case.largest]` `offered`, `at = "home/Archives/Photos.zip"`,
      `kind = "archive"`, `unpacked = "Photos"`; `[case.expect]` item the zip, `core.own-file`, high, in-trash;
    - "a large folder can be chosen whole" — `home/Old projects/site/data.bin` (12 MiB data); `[case.expect]` item
      `home/Old projects`, `core.own-file`, high, in-trash; `[case.largest]` `offered`, `at = "home/Old projects"`
      (a folder: checked in the Folders view of the home).
- `fixtures/golden/sandbox-large-files.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (the trial cache and
  the picked rows), the map, the `largest` section, the cleanup, the tree after.
- `app/tauri/tests/sandboxes/largest.rs` (new) — `check_largest(name, file, engine, sandbox) -> Result<(Option<Value>,
  Vec<Finding>), Failure>`: only for a file with a `[case.largest]` (else `(None, vec![])`); asks
  `engine.largest(&LargestQuery { kind: None, place: None, unchanged: Any, offset: 0, limit: 100 })` (more pages while
  `total` says so) and `engine.largest_place(…)` of each `withheld` or folder `at`'s parent; checks each case:
  offered — a Files row at `at` offered, of `kind`, with `unpacked`, or for a folder a Folders row at `at` offered;
  absent — no Files row at or below `at`; withheld — the Folders row at `at` withheld with that reason;
  not-seen — no Files row at or below `at`, and the last scan's `coverage.not_entered` holds the guarded folder of
  `at`. Then it picks (`engine.pick_own_item`) the node of every positive whose expectation's category is
  `core.own-file` and answers those findings; the report: the Files rows (path from the sandbox, kind, offer) and the
  checked places.
- `app/tauri/tests/sandboxes/harness.rs` — in `run`: the categories given to `file.check` include
  `fresher_core::own_file()`; after `findings(name, &engine)` and before the positives' checks, `check_largest`, its
  findings appended to `findings`; `report["largest"]` set only when it answered a section (so no other golden
  changes). One `mod largest;` in `sandboxes.rs`.
- `app/tauri/tests/sandboxes.rs` — `large_files => "large-files"` in `case_files!`, sorted by file name; failing
  tests, negatives first, as the map's: `a_largest_expectation_that_does_not_hold_fails` (one per `shown`: a listed
  file expected absent, an absent file expected offered, a wrong reason, a seen folder expected not seen).
- `docs/testing.md` — `[case.largest]` in the case file's reference block and its paragraph; `large-files.toml` in the
  list of case files.

## Checks

- `cargo nextest run -p fresher-sandbox`; `npm run sandbox:test` (every case file, the new one included);
  `npm run sandbox -- large-files` writes the sandbox and its `EXPECTED.md`, read once.
- By hand, once per kind, each reverted, written in this step's conclusions: a `kind` changed, a `withheld` reason
  changed, an `absent` changed to `offered`, the `not-seen` case's access changed to `granted`: each fails the
  autotest.
- `git diff -- fixtures/golden` touches only `sandbox-large-files.json`.
- Commit: `test(sandbox): the largest files' case file and the largest expectation`.

## Conclusions

- `[case.largest]` is read by `crates/sandbox/src/largest.rs` (`LargestExpectation`, `Shown`; `case.rs` holds 1 900
  lines, so the table has a file of its own) with eight refusals, each a `CaseError::Largest*` naming its case; `at`
  must be one of the case's paths or their folders, as the map's is. A case's `category` may be the core's
  `core.own-file` (no module owns the `core` namespace, so it is matched by its id).
- `EXPECTED.md` names the kind and the reason by their texts in the core's dictionary (the sandbox crate links the
  core's, unlike the modules'), their keys when it cannot be read.
- `fixtures/sandboxes/large-files.toml` has the brief's 12 cases with two changes: the trial cache sits in
  `home/cache-project` (`storage-map.toml` writes `home/project`, so `all` would have left one of the two out), and the
  standard folder is `home/Public`, since sandbox mode keeps out of `Movies`, `Music` and `Pictures` (only Full Disk
  Access opens them), which Largest then withholds as `not-seen`.
- The not-seen case found that sandbox mode never decided a folder its world states: the scan's root is `~` and the
  stated access matched the guarded places against it unexpanded, so a denied folder was walked. `World::access` now
  answers a `StatedAccess` that reads `~` as the sandbox's home, as the system's probe does. The one other report this
  changes is `sandbox-downloads-denied.json`: its denied `Downloads` is no longer walked, and the map's remainder gains
  the reason "1 folder not read" (7 lines); every other golden is unchanged.
- The harness (`app/tauri/tests/sandboxes/largest.rs`) reads the whole Files list, checks every case (a folder expected
  offered with no kind is checked by its Folders row), picks every `core.own-file` positive, and its findings join the
  scan's before the positives' checks. Failing runs: a listed file expected absent or of another kind, a file in
  `Library` expected offered, a wrong reason, a granted folder expected not seen (by the list, and by the coverage).
- By hand on `large-files.toml`, each reverted: `kind = "archive"` for the disk image fails "listed as DiskImage, not
  as Archive"; `withheld = "protected"` fails "home/Public is Withheld(StandardFolder), not Withheld(Protected)"; the
  Trash's file `offered` fails "home/.Trash/old.dmg is not listed"; `documents = "granted"` fails
  "home/Documents/report.iso is listed".
- The flows of 04/01 now join this Mac's real protection (`fresher_macos::protection` with the user's temporary folder
  carved out) instead of a hand-copied list of standard folders.
