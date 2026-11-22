# 031/02/01 — The case files and the registration

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Two files, since an open app is shared by every case of a file (as
`dev-caches-xcode-open.toml`). The module is registered in the app here: `every_registered_module_has_a_case_file` in
`app/tauri/tests/sandboxes.rs` fails for a registered module without a case file. No change to `crates/sandbox`. The
design: [`../00-design.md`](../00-design.md), section 8. Read first: `fixtures/sandboxes/app-caches.toml`,
`app-caches-app-open.toml`, `dev-caches-xcode-open.toml`, `app/tauri/tests/sandboxes.rs` and its `sandboxes/harness.rs`,
and the vocabulary of `crates/sandbox/src/case.rs` (`[[case.file]]`, `[[case.symlink]]`, `kind = "socket"`,
`[[case.running]]`, `[case.expect]`, `[[case.left-out]]`, `[case.map]`).

## Files

- `fixtures/sandboxes/xcode.toml` (new) — `feature = "xcode"`, `module = "xcode"`; a head comment: what the file
  covers, that every path is under `home/Library/Developer/`, that files are small with `fill = "data"`, that simulator
  devices and DerivedData are not this module's. With these exact names (paths under `home/Library/Developer/`):
  - negative, in this order:
    - "a folder that is not a system version is never claimed" — `Xcode/iOS DeviceSupport/Notes/Symbols/x`;
    - "a device support folder without its symbols is never claimed" — `Xcode/iOS DeviceSupport/17.0
      (21A329)/Info.plist`;
    - "a link where a version's folder would be is never claimed" — `[[case.symlink]]` `Xcode/iOS DeviceSupport/16.4
      (20E247)` → `../../../../Projects/kept`, a file `home/Projects/kept/Symbols/x`; after the cleanup the link and its
      target are unchanged;
    - "Xcode's own settings are never claimed" — `Xcode/UserData/KeyBindings/Default.idekeybindings`;
    - "a folder named like an archive without its contents list is never claimed" —
      `Xcode/Archives/2026-02-01/Fake.xcarchive/Products/x`;
    - "simulator devices are not offered" — `CoreSimulator/Devices/00000000-0000-4000-8000-000000000001/device.plist`;
    - "a device support folder holding a socket is kept" — `Xcode/iOS DeviceSupport/16.0 (20A362)/Symbols/x` and a
      socket `Xcode/iOS DeviceSupport/16.0 (20A362)/s` (`kind = "socket"`); left out `xcode.left-out.special` 1 in
      `xcode.device-support`;
  - positive:
    - "a device support version goes to the Trash" — `Xcode/iOS DeviceSupport/15.0 (19A346)/Symbols/x` and
      `Info.plist`; item `…/15.0 (19A346)`, `xcode.device-support`, low, `in-trash` (with the socket case's 16.0 it is
      one of the two newest iOS versions, so its reason is the "recent" one: the level stays Low);
    - "a watch's device support goes to the Trash" — `Xcode/watchOS DeviceSupport/Watch6,1 10.0 (21R355)/Symbols/x`;
      low, `in-trash`;
    - "the previews are emptied" — `Xcode/UserData/Previews/Simulator Devices/device_set.plist` and
      `Xcode/UserData/Previews/Simulator Devices/a/data`; item `Xcode/UserData/Previews`, `xcode.previews`, safe,
      `emptied`;
    - "the simulators' caches are emptied" — `CoreSimulator/Caches/dyld/24D70/runtime/cache`; item
      `CoreSimulator/Caches`, `xcode.simulator-caches`, safe, `emptied`;
    - "an archive goes to the Trash" — `Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive/Info.plist` and
      `…/dSYMs/Shop.app.dSYM/x`; item the `.xcarchive` folder, `xcode.archives`, high, `in-trash`; `[case.map]`
      `part = "explained"` at that folder.
- `fixtures/sandboxes/xcode-open.toml` (new) — `feature = "xcode-open"`, `module = "xcode"`; one `[[case.running]]`
  `name = "Xcode"`, `bundle-ids = ["com.apple.dt.Xcode"]` in the first case; negatives only:
  - "a device support folder is kept while Xcode is open" — `Xcode/iOS DeviceSupport/14.0 (18A373)/Symbols/x`; left out
    `xcode.left-out.app-open` 1 in `xcode.device-support`;
  - "an archive is kept while Xcode is open" — `Xcode/Archives/2026-01-15/Notes 15-01-2026, 09.00.xcarchive/Info.plist`;
    left out `xcode.left-out.app-open` 1 in `xcode.archives`;
  - "the simulators' caches are kept while Xcode is open" — `CoreSimulator/Caches/dyld/23A344/cache`; left out
    `xcode.left-out.app-open` 1 in `xcode.simulator-caches`.
  File paths apart from `xcode.toml`'s (one world per file; `npm run sandbox -- all` joins every file).
- `fixtures/golden/sandbox-xcode.json`, `sandbox-xcode-open.json` (new) — `UPDATE_GOLDEN=1 npm run sandbox:test`,
  reviewed: the findings with their app, the left-out counts, the map, the cleanup's outcomes (`Moved` for device
  support and the archive, `Emptied` for the two caches), the tree after. After the rebase onto plan 037 they are
  regenerated, never merged by hand.
- `app/tauri/tests/sandboxes.rs` — `xcode => "xcode"` and `xcode_open => "xcode-open"` in `case_files!`, sorted by
  file name.
- `app/tauri/Cargo.toml` (`fresher-module-xcode`), `app/tauri/src/compose.rs` (`Xcode(#[from]
  fresher_module_xcode::XcodeError)` in the compose error, `Box::new(fresher_module_xcode::XcodeModule::new()?)` in
  `modules()` after the last feature module), the two expected-id lists of
  `the_engine_runs_the_feature_modules_always_and_the_dev_only_ones_in_debug_builds_only` and the list of
  `answers_the_modules_and_the_access_status` in `app/tauri/src/commands.rs` — `"xcode"` after the last feature
  module's id.
- `docs/testing.md` — the two case files in the list of case files ("`xcode.toml` and `xcode-open.toml`: the `xcode`
  module. …", in the shape of the `app-caches` paragraph).

## Produces

The module in the app and its case files, for 03/02 (the walk by hand).

## Checks

- `npm run sandbox:test` passes for both files and every other case file; the other files' goldens are unchanged.
- `npm run sandbox -- all` leaves out no file it did not already leave out (compare with `main`'s output).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`high` → `medium` on the archive case)
  and a left-out's items changed (`1` → `2` on the socket case) each make the autotest fail; then reverted.
- `npm run sandbox -- xcode` writes the sandbox; `EXPECTED.md` lists every case, negatives first, each negative "never
  offered by `xcode`".
- `cargo nextest run -p fresher-app -p fresher-module-xcode`; clippy; fmt.

## Conclusions

Built as the step says: `xcode.toml` (7 negatives, 5 positives) and `xcode-open.toml` (3 negatives), their goldens
(reviewed) and the registration in `app/tauri` after the last feature module. By-hand mutations, each reverted: the
archive's danger changed to Medium fails with "the danger Medium is below the category's High"; the socket case's
items changed from 1 to 2 fails with "expected 2 items, got 1". `npm run sandbox:test`: 66 of 66 after the rebase onto
034. `npm run sandbox -- all` leaves out the same five files as `main` (the three `unused-toolchains-*` and the two
`local-ai-models-ollama-*`) and writes 242 cases. With Xcode open in the joined world (`xcode-open`), the positives of
`xcode` are left out as `app-open` in `all`; walk them by name (known issue `repo-sandbox-all-worlds-interact`).
