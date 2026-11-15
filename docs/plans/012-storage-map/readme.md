# 012 — The storage map: where the space went

**Status:** 📝 draft · **Version:** 0.5.0 · **Date:** 2026-11-15 · **Modules:** core, engine, system, app/ui

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The first screen must answer "where did my space go" in one look ([`goal.md`](../../goal.md), sections 1 and 6). That
needs one model of the whole disk into which every module's findings fit, whose parts add up to what the Mac itself
reports, and which has no "Other".

## Goals

1. Every byte of the volume sits in exactly one named place; the parts add up to the volume's used space.
2. Two views of the same numbers: by meaning (apps and categories) and by place (folders).
3. Space no module recognizes is shown as its real folders.
4. Space that is not files — snapshots, purgeable space, swap — is named and explained, never offered as junk.
5. What the app could not measure is said plainly, with the reason.

## Non-goals

- The treemap and other rich charts (the [large files finder](../../ideas/large-files-finder.md)); cleaning anything
  from this screen's system parts.

## Design

### The parts of a volume

| Part                  | What it is                                                              | Cleanable            |
| --------------------- | ----------------------------------------------------------------------- | -------------------- |
| Explained             | findings of the modules, grouped app → category → item                  | by its danger level  |
| Your folders          | everything no module claimed, as the real folder tree                   | no suggestion; the user may act on items |
| macOS                 | the system itself                                                       | never                |
| Kept by macOS         | local snapshots, purgeable space, swap and sleep files — each its own named line with a plain explanation | never; information |
| Not readable          | folders the app was not allowed to read, with the count and the permission that would show them | — |
| Free                  | free space                                                              | —                    |

- **One claim per byte.** The claims of plan [004](../004-fast-scan/readme.md) decide the owner of every folder; a
  nested claim is subtracted from the one around it; hard links and clones are counted once. A property test holds the
  invariant: the leaves add up to the total, in both views.
- **The reconciliation line.** The parts are added up and compared with the volume's used space. A remaining
  difference is shown as its own honest line ("Not measured: 1.2 GB — the file system's own bookkeeping and folders
  that changed during the scan"), never folded into another part and never hidden.
- **Which number.** The headline is the space taken on disk. "Freed if removed" is a separate number, from the private
  size, because clones and snapshots make the two differ. The logical size is detail on demand.
- **Incomplete is marked**: a part with unreadable folders inside shows `42 GB+`.

### Two views

- **By meaning**: apps and categories, the largest first, with growth since the last scan when plan
  [010](../010-history/readme.md) exists. An app's line is its bundle plus everything attributed to it
  (plan [009](../009-system-knowledge/readme.md)).
- **By place**: the folder tree, the largest first, descending automatically through a folder that is almost entirely
  one child. A folder shows which of its parts are explained.
- Switching the view never changes the total.

### Where it lives

The engine builds the map from the tree, the claims and the volume facts, behind a read-only port; it updates as the
scan streams in, so the screen fills live. The UI draws it with the usage bar and lists of plan
[003](../003-base-ui/readme.md).

## Plans (after approval)

| #   | Plan                  | What                                                                     |
| --- | --------------------- | ------------------------------------------------------------------------ |
| 01  | The model             | the parts, attribution to apps, the invariants and their property tests  |
| 02  | Volume reconciliation | snapshots, purgeable space, swap, the reconciliation line                |
| 03  | Unclaimed space       | the folder view, auto-descent, partial sizes                             |
| 04  | The overview screen   | both views, live filling during a scan                                   |

Depends on plans 002, 003, 004 and 009.

## Ideas this touches

The [storage explainer](../../ideas/storage-explainer.md) and the
[system data analyzer](../../ideas/system-data-analyzer.md) (both become this plan), the
[application footprint](../../ideas/application-footprint.md) (the per-app line),
[macOS specifics](../../ideas/architecture/macos.md) (purgeable space, local snapshots).

## Decisions

None yet.

## Agent's calls (to check)

1. **"Not readable" and "Not measured" are shown as honest lines with their reasons.** They are not a catch-all
   category — nothing is classified into them — but they sit close to rule 3 of [`rules.md`](../../rules.md); to
   confirm with the user.
2. **The headline number is the space taken on disk**; "freed if removed" is separate.
3. **Two views over one model**, with the same total.
4. **Unclaimed space carries no cleanup suggestion.**
5. **Snapshots, purgeable space and swap are information only.**
6. **Target version 0.5.0.**

## Conclusions

Not written yet.
