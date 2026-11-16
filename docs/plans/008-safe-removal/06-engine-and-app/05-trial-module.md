# 008/06/05 — The trial module

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** modules, app, docs

No real module exists yet, and the `demo` module's findings are made up (no file behind them), so nothing could be
cleaned in a development build. The `trial` module claims folders that only a developer makes, so a debug build can
clean a sandbox folder for real; the core e2e (06) uses it too. Like `demo`, it is registered in debug builds only.

## Files

- `modules/trial/` (new, `fresher-module-trial`) — rules as data (`rules.toml`, matched by `fresher-rules`):
  - `trial.caches` (Safe): a folder named `fresher-trial-cache`, claimed whole, removed with `MoveToTrash`;
  - `trial.kept` (Low): a folder named `fresher-trial-kept`, emptied and kept (`EmptyFolder`);
  - `trial.tool` (Medium): one object of a made-up tool `fresher-trial-tool` listed in `finish` through the command
    runner, removed with a fixed argument list (the fake runner answers it in tests; in the app the tool does not
    exist, so the category is unavailable — plan 006's tool check);
  - each rule's "must not match" names folders that only resemble it (`fresher-trial-cache2`,
    `fresher-trial-cache.txt`);
  - its texts in `locales/en.json`: plain English, saying these are folders made only for trying the app.
- `modules/trial/tests/` — the contract test with `Catalogue::DevOnly`, fixture tests negative first, its golden
  report `fixtures/golden/trial.json`.
- `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` — registered under `cfg(debug_assertions)`, next to `demo`.
- `docs/hacks/README.md` — the entry "The demo module linked in release builds" names `trial` too.

## Produces

Something real to clean in a development build with a sandbox folder, and the module the core e2e scans.

## Checks

- Negative (first): `a_look_alike_name_is_not_claimed`, `a_file_with_the_name_is_not_claimed`,
  `a_link_named_like_the_folder_is_not_claimed`.
- Positive: `a_trial_cache_folder_is_claimed_whole`, `a_kept_folder_is_emptied_not_removed`,
  `the_tool_object_is_removed_with_a_fixed_argument_list`; the contract test and the golden report.
- A release build registers neither `demo` nor `trial` (the existing test of the composition extended).

## Conclusions

- Built as written: `modules/trial/` (`fresher-module-trial`; `rules.toml` with the rules `cache` and `kept`, the tool
  in `src/tool.rs`), `tests/contract.rs` (`Catalogue::DevOnly`, every text used, every path removal matched) and
  `tests/fixtures.rs` on `fixtures/homes/trial-unclaimed.toml` (negative) and `fixtures/homes/trial.toml`, with the
  golden report `fixtures/golden/trial.json` (`UPDATE_GOLDEN=1` rewrites it). Registered in `compose.rs` under
  `cfg(debug_assertions)` next to `demo`; the composition test now expects `["demo", "trial"]` with
  `debug_assertions` and no module without it, and passes under `cargo nextest run -p fresher-app --release`. The
  app's command test of the module list names `trial` too.
- Not spelled out by the step, taken the safest simple way:
  - **A trial folder is claimed only right next to the sandbox marker** (`sibling = ".fresher-sandbox"` in both
    rules): a developer's folder of that name anywhere else is never even shown. The module cannot read the marker's
    signature (no IO); the cleaner's `Reach::Sandbox` checks it again before anything is removed. Extra negative test:
    `a_folder_without_the_sandbox_marker_next_to_it_is_not_claimed` (no marker, or a marker only further up).
  - **`must-not-match` is a glob**, `fresher-trial-cache?*` and `fresher-trial-kept?*`, covering `…2` and `….txt`.
  - **The tool**: the category `trial.tool` needs `Tool(fresher-trial-tool)`, checked with `fresher-trial-tool
    version`; `finish` runs `fresher-trial-tool list`, one object per line as `<id> <bytes>`, and removes one with
    `fresher-trial-tool remove <id>`. A line that does not read so, or whose id is not a plain name (letters, digits,
    `-`, `_`, `.`, starting with a letter or digit, so never a flag), is not claimed. A tool that is not installed
    lists nothing; a failed listing fails the module alone with `trial.error.failed` (extra negative test
    `a_tool_that_cannot_list_its_objects_fails_the_module_alone`).
  - **The texts** of a rule finding are the folder's name (`trial.rule.<id>.title`) and a reason saying it is a folder
    made only for trying the app, inside a sandbox folder; the tool's finding uses `trial.object.title` and `.reason`.
  - **Docs**: besides the hacks entry, the lines that named `demo` as the only dev-only module now name `trial` too
    (`docs/categories/README.md`, `docs/architecture/README.md`, `docs/architecture/modules.md`, `docs/testing.md`);
    `docs/dependencies.md` lists `module-trial` under `thiserror`.
- For step 06: the rules claim a folder whole (`Claim::Whole`), so the walk does not go inside a claimed trial cache
  and a trial folder nested in another is never matched; and every trial folder of the e2e home must sit directly in
  a folder that carries the sandbox marker.
