# 035 — The largest files

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, scanner, engine, testing, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

A handful of forgotten files often holds most of the space, and a list of them is expected of every tool in the
field. Since plan [027](../027-scan-screen/readme.md) removed the view by place, the app shows nothing outside its
categories; the core's place queries wait without a UI
([deferred](../../deferred/place-queries-without-ui.md)).

## Goals

1. One screen answers "what are my biggest files and folders", from the scan already made.
2. The user's own files are never called junk and never pre-selected.
3. What the app could not see is said, not hidden.

## Non-goals

- A treemap or sunburst, duplicates, similar images: later, on the same data.
- Anything inside `Library`, app bundles, or a place a module already explains.

## Design

- A sidebar entry **Largest**: a ranked list of the files and folders of the home that no module claims, largest
  first. A package (a photo library, a virtual machine, a project bundle) is one row, never opened.
- Each row: name, place, kind in plain English ("Disk image", "Archive", "Video"), size, last opened. Filters: kind,
  "not opened for", minimum size (100 MB by default, a setting).
- A folder row drills down into its children — the core's place queries get their UI back, which closes the deferred
  item.
- A selected row goes through the normal Review and confirmation, at **High**, with the sentence "This is your own
  file; the app cannot tell whether you need it". Trash only by default, as everywhere.
- Never offered, shown with the reason: a file kept in the cloud and not on the disk, a synced folder's root, a
  package in use, the standard folders themselves.
- Guarded folders (Desktop, Documents, Downloads) are read only with the access the user already gave; without it the
  screen says which folders it did not see and how to allow them (plan [006](../006-permissions/readme.md)).
- The list comes from the scan's stored result: no second walk. If the design finds the stored result lacks file rows,
  the scan keeps the top N per place — measured before and after (rule 4).

**Sandbox.** `fixtures/sandboxes/large-files.toml`, sparse files so gigabytes cost nothing, negatives first: a large
file a module claims, a large file under `Library`, a package's inner file, a cloud-only file, a file below the
minimum, a guarded folder denied; then a large old disk image, a large folder, an archive beside its extracted folder.
The case file carries a `[case.map]` as `storage-map.toml` does. Golden report, UI e2e of the screen, and the finish
walks `EXPECTED.md` in sandbox mode.

## Plans (after approval)

| #   | Plan       | What                                                              |
| --- | ---------- | ----------------------------------------------------------------- |
| 01  | Query      | the ranked rows, kinds, last opened, the exclusions               |
| 02  | Selection  | a row as a confirmed item at High through the existing cleaner    |
| 03  | Screen     | the list, filters, drill-down, what was not seen                  |
| 04  | End to end | the case file, golden reports, e2e, the bench if the walk changes |
| 05  | Finish     | docs, the deferred item removed, the walk                         |

Depends on plans 004, 006, 008, 012 and 027.

## Ideas this touches

[Large Files Finder](../../ideas/large-files-finder.md); the "big download not opened" rule of plan
[020](../020-downloads/readme.md) stays a category of its own.

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **A screen, not a module**: these rows are the user's files, not a category of leftovers.
2. **High, never pre-selected**, through the normal confirmation.
3. **No chart in the first version.**
4. **100 MB minimum**, a setting.

## Conclusions

Not written yet.
