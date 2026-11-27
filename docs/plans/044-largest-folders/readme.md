# 044 — Big folders left alone, on the Largest screen

**Status:** 🔨 in progress · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, scanner,
engine, modules, testing, sandbox, e2e, app/tauri, app/ui, docs

Written as a one-file draft on 2026-11-26; the user answered its questions the same day and took every recommended
decision; turned into plans and steps from a read of the code on `feat/044-largest-folders` (f12bdcf4). The design,
with its agent's calls: [`00-design.md`](./00-design.md). From the first tester's feedback
([`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)); takes the idea
[Big projects left alone](../../ideas/big-stale-projects.md). Needs [043](../043-build-flags/readme.md) (Largest
shown). Branch `feat/044-largest-folders`, squash-merged into `main` once, when plan 04 is done.

## Intent

The user's task (translated, 2026-11-26): "A section like Large Files that shows files and folders, does not offer to
remove, only opens the Finder." Then: "further by the recommended ones"; and,
told that nothing else in the app browses folders: the old folder tree stays, read-only, as a third tab "Browse",
with no ticks, only going into folders and Show in Finder; only files stay removable on Largest.

## Goals

1. Largest has three tabs: **Files** (as today, removable at High after the confirmation), **Folders**: the home's
   folders over a size (default 1 GB) where nothing changed for the stale projects' days (default 365), and
   **Browse**: today's folder tree, read-only (go into a folder, Show in Finder).
2. A folder row shows its name, where it is, its size, how long it was left alone and whether it is a project; its
   only action is **Show in Finder**. Nothing on Folders or Browse is ticked, picked or removed: only files stay
   removable on Largest.
3. The highest qualifying folder is listed, never one inside it; the home's standard folders, `Library`, the Trash,
   apps, packages, hidden, protected, ignored and explained places never are.
4. A change is the later of a file's modified and status-change times. The newest change is folded into the scan tree during the walk, kept in the cache and by a rescan: the list reads
   nothing from the disk.
5. A mockup is approved before the screen is built; the screen has a bar, a separator and tiles above the rows.

## Plans

| #   | Plan                                                                  | Status   | Depends on | What                                                                                          |
| --- | --------------------------------------------------------------------- | -------- | ---------- | --------------------------------------------------------------------------------------------- |
| 01  | [The newest change in the scan tree](./01-newest/readme.md)           | ✅ approved | —          | `Node.newest` from the fold and `freeze`; the cache's format 7; the rescan                    |
| 02  | [The query, the setting and the commands](./02-core-and-engine/readme.md) | ✅ approved | 01     | `DiskState::left_alone`, `core.largest.folder-min-size`, `Engine::largest_folders`, `reveal_node`, the sandbox cases |
| 03  | [The Folders and Browse tabs](./03-ui/readme.md)                      | ✅ approved | 02         | the mockup, the IPC and the mock, `FolderRow`, the tabs, Browse read-only, e2e                |
| 04  | [Docs, test cases and the walk](./04-docs-and-walk/readme.md)         | ✅ approved | 03         | architecture, categories, deferred, idea; LF-01 to LF-05; the user's walk; conclusions        |

### Order

```
01/01 ─ 01/02 ─ 02/01 ─ 02/02 ─ 02/03 ─ 02/04 ─ 02/05 ─ 03/01 ─ 03/02 ─ 03/03 ─ 03/04 ─ 03/05 ─ 04/01 ─ 04/02 ─ 04/03
```

Every step is one commit and leaves the workspace green (`cargo nextest`, clippy, fmt, `npm run lint:ts`, the unit
tests and the e2e specs it touches). The steps run one at a time: 02/03 to 03/05 share the bindings, and
`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand. 03/03 starts only after the
user approved 03/01's mockup. Browse becomes read-only in 03/05, after it moved to its own tab in 03/04.

## Conclusions

To be written when plan 04 is done: what was built, what changed from the design, the calls taken while building,
the measured scan time and memory, and the user's walk.
