# 037/01/03 — The commands and the IPC

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app, app/tauri, app/ui

Two thin commands over the engine — `get_trash` (a read) and `open_trash` (an action) — registered everywhere a
command is, with the TypeScript types generated and the typed IPC calls. No path crosses the IPC. The design:
[`../00-design.md`](../00-design.md), section 4.

Read first: `app/tauri/src/commands.rs` (`get_cleanups`, `reveal_in_trash` and their tests), `app/tauri/src/lib.rs`
(the handler list), `app/tauri/build.rs` (`COMMANDS`), `app/tauri/capabilities/default.json`,
`app/tauri/src/dev_runs/commands.rs` (`ACTIONS`, `READS` and the test that every command is in one list),
`app/tauri/tests/bindings.rs`, `app/ui/src/ipc/index.ts`, `app/ui/src/ipc/index.test.ts`.

## Files

- `app/tauri/src/commands.rs` —
  - `/// What waits in each Trash and how much of it this app's cleanups put there: the scan screen's Trash line.`
    `#[tauri::command(async)] pub(crate) fn get_trash(engine: State<'_, Engine>) -> TrashView`.
  - `/// "Open Trash": opens a Trash of the last answer in Finder; takes its index, never a path. Never empties it.`
    `#[tauri::command(async)] pub(crate) fn open_trash(engine: State<'_, Engine>, index: u32) -> Result<(), Refusal>`.
  - Tests beside the cleanup commands', negatives first: `open_trash` of an index never answered is refused with
    `unknown-trash`; then `get_trash` on a fresh engine answers no line, and `open_trash(0)` after a `get_trash` whose
    answer has a line opens it (through the test shell's fake system UI, as `reveal_finding`'s test does; if the shell
    has no Trash line to give, the positive is the engine's, and here only the refusal is tested).
- `app/tauri/src/lib.rs` — both in the handler list, beside `reveal_in_trash`.
- `app/tauri/build.rs` — `"get_trash"` and `"open_trash"` in `COMMANDS`.
- `app/tauri/capabilities/default.json` — `"allow-get-trash"`, `"allow-open-trash"`.
- `app/tauri/src/dev_runs/commands.rs` — `"open_trash"` in `ACTIONS` (19), `"get_trash"` in `READS` (15).
- `app/tauri/tests/bindings.rs` — `TrashView` and `TrashLine` in the imports and `declarations()`, after
  `CleanupsPage`; `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` rewrites
  `app/ui/src/ipc/bindings.generated.ts` (never edited by hand); the diff is looked at: the two types, and
  `unknown-trash` in `CommandError`.
- `app/ui/src/ipc/index.ts` — `getTrash: 'get_trash'` and `openTrash: 'open_trash'` in `COMMANDS` (sorted as the
  others); `export const getTrash = (): Promise<TrashView> => call('getTrash');` and
  `export const openTrash = (index: number): Promise<void> => call('openTrash', { index });` with a one-line comment
  each, as their neighbours; `TrashView` imported from the bindings.
- `app/ui/src/ipc/index.test.ts` — the two calls send their command names and arguments (as the file tests the
  others); `COMMANDS` matches the Rust handler list if the file checks that.
- Any exhaustive UI handling of `CommandError` kinds that `tsc` reports (a text per kind): `unknown-trash` handled
  as `unknown-cleanup-item` is.

## Checks

- `cargo nextest run -p fresher-app` (the bindings test byte for byte, the dev-runs classification test, the commands'
  tests); clippy; fmt.
- `npm run typecheck` (or the repository's `tsc` script) and `npx vitest run app/ui/src/ipc`.
- `npm run lint` passes (knip sees `getTrash` and `openTrash` unused until 02/01: if knip fails on them, move the
  two exports into 02/01 with their first caller instead, and say so in the conclusions).

## Conclusions

Built: `get_trash` (read) and `open_trash` (action, refused `unknown-trash` for an index not in the last answer) in
`app/tauri/src/commands.rs`, `build.rs`, the capability file and the dev-runs classification; the typed calls
`getTrash` and `openTrash` in `app/ui/src/ipc/`; the bindings regenerated. `fresher-app` 247 tests, `src/ipc` 14.

What changed: the positive `open_trash(0)` test is not in the shell (its Trash view has no line); the engine's and
the core e2e's tests prove it, the shell tests the refusal and the empty view of a fresh engine.
