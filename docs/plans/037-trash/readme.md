# 037 — The Trash holds the freed space

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, engine, store, testing, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Every tool of the field deals with the Trash. Fresher moves items there and stops, so after a cleanup the disk is as
full as before: macOS frees the space only when the Trash is emptied. The user checks the Storage settings, sees
nothing changed, and stops trusting every other number in the app. A first release cannot ship that.

## Goals

1. The result of a cleanup says what was moved and what was actually freed, and why they differ.
2. The user sees how much waits in the Trash, and how much of it this app put there.
3. The user is told where to look and that emptying is theirs to do.

## Non-goals

- The Trash as a cleanable category: what the user put there is theirs.
- Emptying the Trash, in any form: the app never does it and never offers it (decision 1).

## Design

- **Wording**: "Moved 20 GB to the Trash. You get the space back when you empty it." The result shows both numbers —
  moved, and freed on the volume as measured — so a zero reads as expected.
- **A Trash line on the scan screen**, information only: "Waiting in the Trash: 23 GB, 20 GB of it from cleanups
  here". The second number comes from the cleanup journal (plan [008](../008-safe-removal/readme.md)), re-checked
  against what is still there.
- **Open Trash** on the result and on that line opens it in Finder, with the sentence "Look at what is there and
  empty it yourself if you no longer need it". That is all the app does about the Trash.
- **History is not fooled**: items the app moved are known from the journal, so the next scan reports them as
  cleaned, not as "shrank by itself", and the Trash's growth is attributed to no app (the comparison itself is plan
  [010](../010-history/readme.md)'s).
- **Per volume**: an item trashed on another volume waits in that volume's Trash; numbers are shown per volume.
- The confirmation already says, in one sentence, that Trash means the space returns on emptying (settled in plan
  008's design). The permanent choice stays as it is; its description says "frees the space now; you cannot put these
  items back".
- **A fact to check in the design**: whether Finder's "remove items from the Trash after 30 days" setting can be read
  without a prompt; if it can, the line adds "the space comes back by itself on <date>".

**Sandbox.** `fixtures/sandboxes/trash.toml` on the `trial` module, negatives first: an empty Trash (no line), items
in the Trash the app did not put there (counted in the first number only), a journal entry whose item was already
emptied (not counted); then a cleanup whose items wait in the sandbox's `home/.Trash`. `fill = "data"` files make the
freed number measurable. Golden report, UI e2e of both wordings, and the finish walks `EXPECTED.md` in sandbox mode
with and without `--real-trash`.

## Plans (after approval)

| #   | Plan       | What                                                             |
| --- | ---------- | ---------------------------------------------------------------- |
| 01  | Numbers    | the Trash's size per volume, the journal's share, moved vs freed |
| 02  | Screens    | the result, the Trash line, Open Trash, the confirmation's line  |
| 03  | End to end | the case file, golden reports, e2e                               |
| 04  | Finish     | docs, the walk                                                   |

Depends on plans 008, 012 and 027.

## Ideas this touches

This plan replaces the idea "The Trash holds the freed space" (2026-11-15, the agent's proposal); everything it held
is above. [External Drive Analyzer](../../ideas/external-drive-analyzer.md): the numbers per volume.

## Open questions

None.

## Decisions

1. **The app does not let the user empty the Trash** (the user, 2026-11-22): it only says that something waits
   there and invites the user to look and, if they want, empty it themselves. In line with "Trash only, never empty
   it" ([best practices](../../ideas/best-practices.md), 9).

## Agent's calls (to check)

1. **"Moved", not "freed"**, with both numbers on the result.
2. **The Trash is information, never a category.**
3. **Open Trash is the only action**, on the result and on the Trash line.

## Conclusions

Not written yet.
