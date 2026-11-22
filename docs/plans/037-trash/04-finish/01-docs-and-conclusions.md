# 037/04/01 — The docs, the walks and the conclusions

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every doc the plan's change affects, as built; the walks by hand written for the user (not walked by the agent); the
conclusions with the numbers; and the plan's section of `docs/deferred/user-review-2026-11-22.md`. Read the plan's
readme, `00-design.md` and every step's conclusions first; the docs describe what was built, not the design.

## Files

- `docs/architecture/README.md` — "Ports": `FinderSettings` (what it reads, `MacFinderSettings` through `defaults`,
  never a write) and `SystemUi::open_folder`; "Data": the Trash view (what each number is, where it comes from, that
  no path crosses the IPC); "UI": the Trash line on the scan screen and its refresh triggers. One sentence where the
  Trash is described: the app never empties a Trash, and nothing in the core can.
- `docs/architecture/removal.md` — checked, not changed (the cleaner is untouched); if it names what the app does
  after a move, one sentence pointing to the Trash line.
- `docs/testing.md` — checked: 03/01's paragraph and case files; the core e2e `tests/trash.rs` in its list; the UI
  scenarios if the file lists them.
- `docs/manual-testing.md` — a section **1.17 The Trash line** and an item **2.15** (the next free numbers; check),
  each linking to this step, holding the walks below as checkboxes.
- `docs/known-issues/app-trash-waiting-stale-after-emptying.md` (new) and its row in `docs/known-issues/README.md` —
  symptom: after the user empties the Trash in Finder, the first number keeps the last scan's measure until the next
  scan (the share drops at once); cause: no walk of the Trash of its own (agent's call 4); tried: nothing; options
  left: a shallow measure of the Trash's top-level entries on each look, or "as of the last scan" in the sentence.
- `docs/plans/010-history/readme.md` (a draft) — one sentence under its design: the journal names every item a
  cleanup moved, so the comparison reports their space as cleaned by Fresher, and the growth of the Trash folders is
  attributed to no app (plan 037, section 6).
- `docs/ideas/README.md` and the idea file "The Trash holds the freed space" if they still name it as an idea: the row
  says it became plan 037 (the draft already says it replaced the idea; check, do not duplicate).
- `docs/plans/README.md` — the row of 037: 🔨 in progress (the status the conclusions leave), its modules as in the
  plan's readme.
- `docs/plans/037-trash/` — the readme's, every plan's and every step's **Conclusions**, as built: what was built,
  what changed from the design and why, the numbers (Rust tests and coverage, the engine file's coverage, UI tests
  and coverage, Playwright runs, the sandbox mutation checks of 03/01), the calls taken while building, the facts
  still to verify. Statuses 🔨 with "awaiting the user's walk".
- `docs/deferred/user-review-2026-11-22.md` — create it if absent (heading "# The user's review of 2026-11-22", one
  line saying it collects what the user should look at after the autonomous session of 2026-11-22, one section per
  plan) and add its row to `docs/deferred/README.md` (Agreed 2026-11-22, Due at "the user's review", What "the walks,
  facts and agent's calls of the plans built on 2026-11-22"); otherwise append. The section **## 037 — The Trash
  holds the freed space**, short bullets:
  - **Walks by hand**: the sandbox walks and the real-Mac dry run below (link to `docs/manual-testing.md` 1.17 and
    2.15).
  - **Facts not verified**: Finder's 30-day key (`FXRemoveOldTrashItems` prints `1` with the setting on); `open
    ~/.Trash` opens Finder's Trash window with and without Full Disk Access; a USB disk's `.Trashes/<uid>` gets its
    own line named after the disk; the time `get_trash` takes on a Mac with a long history.
  - **Agent's calls worth a second look**: 3 (Show in the Trash kept on the result, Open Trash only on the line), 4
    (the scan's measure, stale after emptying), 5 (the date for the app's share only), 7 (every golden gains a
    `trash` section), 10 (refresh on focus), 11 (the history's per-page sum); and every call taken while building.
  - **Known gaps**: the stale first number (the known issue); the history's per-page sum; a second volume's Trash
    measured only when a scan root holds it; sandbox dates follow the generation's clock
    (`docs/known-issues/app-sandbox-scan-dated-at-generation.md`).

## The walks (written for the user, not walked here)

In the sandbox:

- [ ] `rm -rf sandbox/trash-empty && npm run dev:sandbox -- trash-empty`. Scan: no Trash line. Clean the trial cache
  to the Trash: the result says "Moved to the Trash", "You get the space back when you empty the Trash.", and the
  invitation under Show in the Trash; back on the scan screen the line reads "Waiting in the Trash: 1 MB, all of it
  from cleanups here." Open Trash opens `sandbox/trash-empty/home/.Trash` in Finder.
- [ ] `rm -rf sandbox/trash && npm run dev:sandbox -- trash`. Scan: "Waiting in the Trash: 2 MB." Clean: "Waiting in
  the Trash: 6 MB, 4 MB of it from cleanups here." Remove the cache's entry from `home/.Trash` in Finder and come back
  to the window: the share is gone, the first number stays until a new scan (the known issue). Put back from
  Cleanups: the share is gone.
- [ ] Both again with `--real-trash`: the moved items wait in the Mac's Trash; a second line reads "Waiting in the
  Trash from cleanups here: … The rest of the Trash was not measured."; Open Trash on it opens the Mac's Trash. Put
  the items back afterwards.
- [ ] No button anywhere says Empty; the Trash line and the result only invite.

On the developer's Mac (`npm run dev`, a dry run):

- [ ] With Full Disk Access: the line's first number against `du -sh ~/.Trash`; without it: the line shows only a
  share, or nothing.
- [ ] Finder → Settings → Advanced → "Remove items from the Trash after 30 days" on: `defaults read com.apple.finder
  FXRemoveOldTrashItems` prints `1`; the line shows a date only when a share is there.
- [ ] With a USB disk holding something in its Trash and scanned: a line of its own, named after the disk.
- [ ] Open Trash opens Finder's Trash window.

## Checks

- `npm run lint` (Markdown, links); `npx prettier --check` on the changed docs; every relative link resolves.
- `npm run pre-push` on the branch's last commit; `git diff 97d9b314 -- crates/cleaner crates/scanner crates/store
  crates/rules modules` empty.

## Conclusions

Done: the architecture README (Ports, Data, UI), a sentence in `removal.md`, `testing.md` (the core e2e `trash.rs`,
the UI samples and scenarios), `manual-testing.md` 1.17 and 2.15, the known issue
[`app-trash-waiting-stale-after-emptying`](../../../known-issues/app-trash-waiting-stale-after-emptying.md) and its
row, one sentence in plan 010's draft, the plans' row (🔨), every Conclusions section, and the plan's section of
[`user-review-2026-11-22.md`](../../../deferred/user-review-2026-11-22.md). The ideas had no row for this idea
(nothing to change).

The chromium baseline `category-details-app-leftovers-chosen`: on the final commit it passes alone and fails when its
whole spec (`visual.spec.ts`, chromium) runs; the diff shows the whole page drawn at another offset, and the details
screen has no Trash line, so it is not a change of 037. It could not be run on main from this branch; it is the known
flaky baseline of the contract and of 1.14.

Checks: see the plan's readme, "Conclusions". Awaiting the user's walk.
