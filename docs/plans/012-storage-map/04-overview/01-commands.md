# 012/04/01 — The commands

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app,
app/ui

## Files

- `app/tauri/src/storage_commands.rs` (new, beside `settings_commands.rs`) — three thin commands, `async` like the
  others, each one call of the engine and its `Refusal`:
  - `get_storage_map() -> Result<StorageMap, Refusal>`;
  - `get_storage_meaning(volume: u32) -> Result<Vec<MeaningGroup>, Refusal>`;
  - `get_storage_place(node: NodeId, offset: u32, limit: u32) -> Result<PlaceView, Refusal>`.
  The commands take ids, never paths (architecture: "the commands take ids, never paths").
- `app/tauri/src/lib.rs` — `mod storage_commands;` and the three names in `generate_handler!`, appended after
  `settings_commands::get_own_data`; nothing else of the file changes (plan 007 edits its setup).
- `app/tauri/build.rs` — the three names in `COMMANDS`, in the handler's order.
- `app/tauri/capabilities/default.json` — `allow-get-storage-map`, `allow-get-storage-meaning`,
  `allow-get-storage-place`.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`ts-rs`), never edited by hand.
- `app/ui/src/ipc/index.ts` — `COMMANDS` and `getStorageMap()`, `getStorageMeaning(volume)`,
  `getStoragePlace(node, offset, limit)`, typed from the bindings.
- `app/ui/src/ipc/mock.ts` — the mocked core answers the three with a map of the gallery's sample volume (a phase
  `complete` home volume of 500 GB: explained 64 GB, your folders 180 GB, outside the scanned folders 120 GB, macOS
  23 GB, swap 2 GB, disk's own 1.6 GB, free 109 GB, purgeable 18 GB), so the UI e2e and the gallery have stable data.

## Produces

The typed IPC the state (02) and the screen (04, 05) use.

## Checks

- Negative first (`app/tauri/src/storage_commands.rs` tests, through the app's test engine): `a_place_of_an_older_scan_is_refused_as_stale`,
  `an_unknown_volume_is_refused`, `the_meaning_before_any_scan_is_refused_as_no_scan`; `app/ui/src/ipc/index.test.ts`:
  `a_refusal_of_get_storage_place_becomes_a_failure`.
- Positive: `the_map_before_a_scan_is_no_scan`, `the_map_after_a_scan_adds_up` (the parts of every volume add up to its
  capacity); the existing `app/tauri/tests/config.rs` test (every command granted, no other permission) passes with
  the three grants; the bindings test passes; `index.test.ts`: `get_storage_map_calls_its_command`.
- `cargo nextest run -p fresher-app`; `npx vitest run src/ipc`.

**Changed by 04/05:** the place view takes `descent: Descent` (`through` | `stay`): `through` opens down the folders that are almost all one child, `stay` opens exactly the folder asked (a step of the breadcrumb).

## Conclusions

Built in `2bdc6a1`, reviewed clean.

- `get_storage_map`, `get_storage_meaning(volume)` and `get_storage_place(node, offset, limit)`, their grants, the
  typed IPC and `sampleStorageAnswers` for the mocks; bindings regenerated.
- Deviations: the sample volume is 499.6 GB, the sum of its parts (a map must add up). The shell's testing engine
  gained a fake 1000-byte volume, otherwise its volume map was empty.
- Measured: `cargo nextest run -p fresher-app` 55 passed.
