# 002/05/04 — The UI's IPC module and store

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/ipc/index.ts` — `COMMANDS` gains every command; one typed function per command using the generated
  types; `subscribe(onEvent)` creates the `Channel<Event>`. Still the only module importing `@tauri-apps/api`.
- `app/ui/src/ipc/mock.ts` — `mockCore` answers by the new keys; a helper to push events into a subscribed channel.
- `app/ui/src/state/store.ts` — a store of our own on `useSyncExternalStore`: `reduce(state, event)` (a pure function)
  keeps the engine status, findings by category, module outcomes and the dictionary; `useCore(selector)`.
- `app/ui/src/app.tsx` — a bare screen: the version, a "Scan" button, the scan state, the findings grouped by category
  with their size, danger level and texts through `tText`; a refused command shows its `CommandError` text. No styling
  beyond the existing `styles.css` (plan 003 does the design).
- `app/ui/e2e/core-mock.ts`, `app/ui/e2e/*.spec.ts` — the smoke e2e drives a scan on the mocked core streaming demo-like
  events; the screenshot is updated and looked at.

## Produces

The IPC module and store plan 003 builds its screens on.

## Checks

- Negative (`describe('negative cases')`): `ignores an event of another scan`, `shows the refusal text when a scan is
  busy`, `keeps findings when the scan is cancelled and marks it partial`.
- Positive: `adds findings in batches`, `groups findings by category`, `shows every label from the dictionary`.
- `npx vitest run src/state src/ipc` and `npm run test:e2e:ui` pass; UI coverage ≥ 90 %.

## Notes from the implementation

- **The IPC module** (`src/ipc/index.ts`) has one typed function per command, each through one `call` that turns a
  rejection into `CommandRefused` (`src/ipc/refusal.ts`). `CommandRefused.text` is what the core's
  `CommandError::text()` gives: key `core.error.<code>`, the error's other fields as arguments (a number as a count,
  anything else as text). The UI rebuilds that `Text` from the serialized error because the core sends the error, not
  its text (to check: the core could send `text` with the error). A failure that is not a `CommandError` (the IPC
  itself) shows as `internal`.
- **The store** is three files in `src/state/`: `reduce.ts` (`reduce(state, action)`, pure; an action is a core
  `Event` or an answer of a command: `dictionary-loaded`, `overview-loaded`, `command-refused`), `store.ts`
  (`createCoreStore`, `CoreContext`, `useCore(selector)` on `useSyncExternalStore`) and `connect.ts` (`connectCore`
  subscribes, loads the dictionary and the overview once per store, even under React's StrictMode; `requestScan`).
  `App` takes the store as a prop (`main.tsx` creates it), so each test has its own.
- The state keeps the followed scan's id, the engine status, the findings grouped by category in the order they came,
  the module outcomes, the scan outcome, the progress, the dictionary and the last refusal (cleared when a scan
  starts). An event of another scan is ignored; `setting-changed` and `access-changed` change nothing until plans 005
  and 006.
- **The screen**: the version, a "Scan" button (always enabled, so a second scan shows the core's refusal), the scan
  state (`ui.scan.ready`, `ui.scan.running`, or `core.scan.outcome.<kind>` with a failure's reason), the refusal in
  an `alert`, and per category a heading (`<category>.name`) and one line per finding: title, allocated size
  (`formatSize`), danger level name (`core.danger.<level>.name`) and reason. The text keys follow the core's
  conventions; no text is written in the UI.
- **ESLint's `camelcase`** allows the three snake_case fields of the generated types (`full_disk_access`,
  `highest_danger`, `last_scan`), because the Rust field names go over the wire (to check: `rename_all =
  "camelCase"` on the core's structs instead).
- **The mocks**: `mockCore` answers `subscribe` itself and keeps the channel; `pushEvents` streams events through it;
  `refuse(error)` rejects with a plain `CommandError`. The e2e mock implements `transformCallback` and streams a
  script's events through the subscribed channel after `start_scan`. `src/test/demo-scan.ts` holds a demo scan and its
  texts for both.
- The screenshots changed with the screen: `start-screen` (the Scan button and the ready text) and a new
  `finished-scan`, looked at in both engines.
- `npx vitest run`: 42 tests; coverage 100 % statements, functions and lines, 97.3 % branches.
- **Fixed after review**: `connectCore` subscribed and asked for the overview at once, and `overview-loaded` overwrote
  the status whatever the events had said. Now the overview is asked for after `subscribe` resolves, and the state
  keeps `seen`, the newest scan the events told of and whether it finished: an overview of an older scan, or one still
  "scanning" a scan whose `scan-finished` came, is ignored, and an idle overview does not end a scan the events started.
  Adding findings copied the whole group per finding (O(N²)); a batch is now grouped in one pass and each group it
  touches is copied once (50,000 findings in batches of 256: 2.4 s before, 20 ms after). The store still keeps every
  finding: `docs/known-issues/ui-findings-kept-in-memory.md`. `mockCore` takes an optional `subscribe` answer, so a
  test can delay it.
