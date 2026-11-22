# 033/03/01 — The docs, the conclusions and the review list

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs, modules

Every doc the plan's change affects, as built (the docs an earlier step already updated are checked, not rewritten);
the automated checks on the branch's last commit; the plan's conclusions; and this plan's section of the user's review
list. The sandbox walk and the dry run on the developer's Mac are the user's: they are written as unchecked boxes, not
run. The design: [`../00-design.md`](../00-design.md).

## Files

- `docs/architecture/README.md` — `fresher-module-logs` in the crate list, one line.
- `docs/architecture/modules.md` — the module `logs` among the examples: one decision per file in `on_match` (the
  shape of `temp`), six `Claim::Item` interests under `KnownFolder::Logs` so nothing is made opaque, and the split of
  `Logs` with `jetbrains` and `app-leftovers` carried by their whole claims and refused by name here; the place's
  contract (the second exception of "Unknown is not junk").
- `docs/restrictions/deletion-safety.md` — the second exception written in 01/02, checked against what was built.
- `docs/categories/logs-app-logs.md`, `docs/categories/logs-crash-reports.md` — as built (the rows, the texts, the
  setting); their catalogue rows in `docs/categories/README.md`. `docs/categories/jetbrains-logs.md` and
  `docs/categories/app-leftovers-caches.md` — one sentence each under "How the app decides": other old files of
  `~/Library/Logs` are offered by "Old logs of your apps"; this one stays the module's own.
- `docs/testing.md` — the case file (02/01), `logs.rs` (02/02) and the UI scan (02/03), checked.
- `docs/manual-testing.md` — the walk items below as entries of the consolidated checklist, linking to this step.
- `docs/dependencies.md` — checked: `fresher-module-logs` in the used-by columns of `thiserror`, `fresher-testing`,
  `proptest`, `serde_json` (and `fresher-core` if listed); no new dependency.
- `docs/ideas/system-junk-cleaner.md` — its line on plan 033: what was built (the user's logs and crash reports);
  still an idea: device software updates, Mail's downloads, the system-wide places. `docs/ideas/README.md`: its row if
  it states a status.
- `docs/deferred/logs-more-places.md` (new) and its row in `docs/deferred/README.md` — "More places of logs", agreed
  2026-11-22 (the user's delegation), due "after plan 033's run on the developer's Mac": (1) an installed app's logs
  under its bundle id (`~/Library/Logs/<id>/`) — `app-leftovers` claims those folders whole, so a file inside is never
  reported to another module; doing it needs either a second whole interest `Under(Logs, "*")` `.only_bundle_ids()` in
  `logs` that offers the folder emptied (`Removal::EmptyFolder`) when nothing inside changed for the days and no
  installed-app check of `app-leftovers` claims it, as plan 030 did for caches, or a core change that lets an item
  claim see inside another module's whole claim; (2) files deeper than six names below `Logs`; (3) the system-wide
  logs, which need administrator rights and stay out by the restriction.
- `docs/plans/033-logs/` — every step's and plan's conclusions written as built (status 🔨 in progress, "awaiting the
  user's walk"); the global `readme.md` Conclusions: what was built, what changed from the design and why, the numbers
  (Rust tests and coverage, `modules/logs` coverage, UI tests and coverage, Playwright, the no-core-change diff), the
  calls taken while building. `00-design.md` Conclusions: a short paragraph pointing to the global readme. The global
  readme's status and its row in `docs/plans/README.md`: 🔨 in progress (✔️ done only after the user's walk).
- `docs/deferred/user-review-2026-11-22.md` — create it if absent (title "The user's review of 2026-11-22", one
  section per plan, with a row in `docs/deferred/README.md`: agreed 2026-11-22, due "the user's next session", what
  "walks, unverified facts and agent's calls of the plans built on 2026-11-22"); otherwise append. This plan's section,
  `## 033 — Logs and crash reports`, short bullets:
  - **Walks by hand**: `rm -rf sandbox/logs && npm run dev:sandbox -- logs`, walk `EXPECTED.md` line by line,
    negatives first (nothing of a negative offered by `logs`; the fresh, six-day, socket, system and locked cases read
    as kept with their reasons); each positive offered at Safe with its title and reason; select all, Review names
    Safe, confirm to the Trash, every log folder still there, Cleanups → put back; the setting raised to 14 days and a
    rescan; ignore one item, the category, the module, and restore each. `npm run dev:sandbox -- jetbrains` and
    `-- app-leftovers`: their logs are still offered by their own modules only.
  - **The dry run on the developer's Mac** (`npm run dev`, a scan of `~`): every offered file under `~/Library/Logs`,
    older than 7 days (`find ~/Library/Logs -type f -mtime +7 | wc -l` beside the findings' count); nothing under
    `Logs/JetBrains` or a `com.*`-shaped folder; the largest ten findings looked into (read-only) for state that is not
    a log; how many findings a real Mac shows and whether Details stays usable (agent's call 4); the scan time beside
    plan 030's (agent's call 6).
  - **Facts not verified**: the macOS version the reports' layout was read on (27.2.1); whether Console still lists a
    report after it is put back from the Trash; whether any app on the Mac writes logs deeper than six names.
  - **Agent's calls worth a second look**: 1 (the second exception of "Unknown is not junk": every file of `Logs`
    whatever its name, at Safe), 2 (installed apps' id-named log folders deferred, against the draft), 3, 5, 7.
  - **Known gaps**: the deferred item `logs-more-places.md`; the logs of other users' processes held open are not
    seen by the cleaner (known issue).

## Checks

- `npm run pre-push` on the branch's last commit (lint, guards, knip, machete, deny, script tests, tests with coverage);
  Rust and UI coverage at or above 90 %; `modules/logs` at or above 90 %.
- `git diff 97d9b314 HEAD -- crates/core crates/scanner crates/cleaner crates/engine` is empty.
- `npm run test:e2e` (core and UI, both browser projects).
- `npm run lint` (Markdown and links); every relative link of the changed docs resolves.
- Commit: `docs(docs): plan 033 — docs as built, conclusions and the review list`.

## Conclusions

Done: the docs are as built, the checks are green, the conclusions are written, and the user's review list is
[`docs/deferred/user-review-2026-11-22.md`](../../../deferred/user-review-2026-11-22.md), section 033. The deferred item
[`logs-more-places.md`](../../../deferred/logs-more-places.md) is new. The manual checklist has 1.15 (the sandbox walk)
and 2.13 (the dry run). The plan is 🔨 until the user's walk.
