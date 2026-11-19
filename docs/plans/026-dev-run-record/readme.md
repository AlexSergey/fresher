# 026 — The development run record

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app, app/tauri, core, engine,
cleaner, testing, scripts, docs

A one-file draft, approved by the user the same day; plans, steps and `00-design.md` follow it.

## Intent

The user's idea, 2026-11-19 (translated): "We already have a development mode. Without a sandbox it works on the Mac;
in development mode we delete nothing, but we write to `NO_COMMIT` what was found and what could be cleaned (when the
clean button is pressed); also every user action in development mode and the list of settings — one big JSON
(`settings`, `actions`, `clear` by group). Add the build number to every run, and the build number and the date to the
file name, to know when and how it was run, and so that nothing is overwritten."

The runs by hand on the real Mac (`docs/manual-testing.md`, section 2) then become: start the development build, scan,
press clean, hand over the file. The agent checks every finding and every would-be removal against the disk with a
script instead of the user reading the window.

## Goals

1. A development build (never a release build) records each run to its own file under `NO_COMMIT/dev-runs/`: the
   session (build, macOS, roots), the settings, the access, every action that reached the core, every scan's findings,
   left-out lines, coverage and projects, and every cleanup the user confirmed, grouped by module group.
2. On the real Mac a confirmed cleanup is a **dry run**: the plan is built and every item goes through the cleaner's
   checks right before removal, as in a release build; only the move to the Trash is replaced by a record
   (`would-move`, or `skipped` with the reason). In sandbox mode the cleanup is real, as today, and recorded too.
3. A file is never overwritten: its name carries the date and time, the build and the mode.
4. A checker script reads a record and checks each finding and each would-be removal against the disk, read-only, and
   writes the differences next to the record.

## Non-goals

- Anything in a release build: the opt-in session log of the deferred item
  [session-log-file](../../deferred/session-log-file.md) may reuse this later, with its own decisions (privacy text,
  location, rotation).
- Recording clicks that never reach the core (a checkbox, a scroll): the record is of what the core was asked to do.
- Sending a record anywhere: it stays in `NO_COMMIT/`, which git ignores.

## Design

### The file

`NO_COMMIT/dev-runs/<YYYY-MM-DD_HH-MM-SS>_<version>+<commit>[-dirty]_<mac|sandbox-<name>>.json`, for example
`2026-11-20_10-04-31_0.1.0+26e4cb0-dirty_mac.json`, created new (never opened for writing if it exists; a second run in
the same second gets `-2`). Beside it, `….events.jsonl`: one line per event, appended as it happens, so a crash loses
nothing; the `.json` is rewritten atomically (write a temporary file, rename) after every scan and every cleanup.

```jsonc
{
  "session": {
    "started": "2026-11-20T10:04:31Z", "build": { "version": "0.1.0", "commit": "26e4cb0", "dirty": true },
    "mode": "mac", "macos": "27.2.1", "roots": ["/Users/…"], "fullDiskAccess": "granted"
  },
  "settings": { "core.scan.roots": ["~"], "stale-projects.unused-for-days": 365 },
  "access": { "folder.downloads": { "state": "granted", "switch": true } },
  "actions": [
    { "at": "…", "kind": "setting-changed", "key": "…", "from": 365, "to": 30 },
    { "at": "…", "kind": "scan-started", "scan": 1 },
    { "at": "…", "kind": "module-ignored", "module": "…" },
    { "at": "…", "kind": "cleanup-confirmed", "cleanup": 1 }
  ],
  "scans": [
    { "id": 1, "kind": "full", "outcome": "completed", "durationMs": 41230, "coverage": { },
      "modules": { "<module>": { "availability": "available", "findings": [ ], "leftOut": [ ] } },
      "projects": [ { "root": "…", "kinds": ["node"], "lastActivity": { "known": "…" } } ] }
  ],
  "clear": [
    { "id": 1, "scan": 1, "removal": "trash", "dryRun": true,
      "groups": { "<group>": [ { "module": "…", "category": "…", "danger": "low", "path": "…", "size": 0,
                                  "outcome": "would-move" } ] } }
  ]
}
```

- A finding is written as the core sends it to the UI (category, danger, subject, sizes, reason key and arguments,
  removal), plus the reason's English text, so the file reads without the dictionary.
- The build's commit and dirty flag come from the launcher (below), not from a build script: nothing new in the
  release build.

### Where it lives

- **The recorder** sits in the shell (`app/tauri`), behind the one typed command layer: every command that changes
  something (scan, rescan, restore, a setting, ignore, restore an ignored item, confirm a cleanup, put back) appends its
  action; the engine's events it already forwards to the UI (scan finished, coverage, findings) fill `scans`. The core
  gains nothing but serialisation where a type lacks it.
- **Development builds only**: the recorder and the dry-run Trash are compiled under `cfg(debug_assertions)`; a
  release build has neither. It writes only when the launcher names the folder (`FRESHER_DEV_RUNS`), so tests and a
  bare `tauri dev` write nothing.
- **The launcher**: `npm run dev` and `npm run dev:sandbox` go through one script that sets `FRESHER_DEV_RUNS` to
  `<repository>/NO_COMMIT/dev-runs`, and `FRESHER_BUILD_COMMIT` / `FRESHER_BUILD_DIRTY` from `git rev-parse --short
  HEAD` and `git status --porcelain`.
- **The dry run**: on the real Mac the composition gives the cleaner a recording Trash port (a sibling of the fake
  Trash of the tests) instead of the system Trash: `move_to_trash` checks nothing more and moves nothing, it records
  `would-move`; every check before it runs as in a release build. Put back of a dry run answers "nothing was moved".
  The cleaner's sandbox-only reach stays as it is for real moves; the dry run is what lets the real Mac's plan run to
  the end.

### The checker

`npm run dev-runs:check -- <file>` (a TypeScript script in `scripts/`, read-only): for every finding and every
`would-move` item, `lstat` the path (exists, not a link, a folder or a file as stated), the size against a walk of it,
and the module's own rule where it can be checked from the disk (a stale project's newest folder and `refs/heads`
older than the days; an installer older than its app; nothing under `~/Library`, an app bundle, `.git`, `src`, `dist`);
writes `<file>.check.md` with every difference. Run by the agent on the user's record.

## Plans (after approval)

| #   | Plan               | What                                                                                     |
| --- | ------------------ | ---------------------------------------------------------------------------------------- |
| 01  | The recorder       | the file, its name, events and snapshot, settings, access, actions, scans; the launcher  |
| 02  | The dry run        | the recording Trash on the real Mac in a development build; `clear` by group; put back   |
| 03  | Checker and finish | the checker script, `docs/manual-testing.md` section 2 rewritten around it, docs         |

Depends on plans 003 (commands), 006 (access), 007 (sandbox mode), 008 (cleaner, journal).

## Decisions

The user's, of 2026-11-19 (translated): the idea above; "add the build number to every run, and include the build number
and the date in the file name, so we know when and how it was run and nothing is overwritten"; work autonomously with
the recommended decisions.

## Agent's calls (to check)

1. **The build number is `<version>+<commit>[-dirty]`**: the version alone does not change while the app is built.
2. **Two files per run**: an append-only `.events.jsonl` (safe against a crash) and the snapshot `.json` in the user's
   shape.
3. **Only actions that reach the core** are recorded, not clicks.
4. **The dry run happens only on the real Mac**; a sandbox cleans for real, as today, and is recorded.
5. **Development builds only, and only with `FRESHER_DEV_RUNS` set by the launcher.**
6. **The checker is the agent's tool**, read-only, writing a report next to the record.
