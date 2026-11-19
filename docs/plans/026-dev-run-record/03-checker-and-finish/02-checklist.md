# 026/03/02 — The checklist around the record

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs

[`docs/manual-testing.md`](../../../manual-testing.md) section 2 asks the user to compare the window with shell
commands, line by line. With the dry run and the record, the user starts the development build, scans, presses clean,
and hands over one file; the agent runs the checker. This step rewrites section 2 around that run, keeps by hand only
what the disk cannot answer, and corrects what section 0 says a development build does. The design:
[`../00-design.md`](../00-design.md), "Intent" and section 4.

## Files

- `docs/manual-testing.md` —
  - the header's "Last compiled" date: the day of this step;
  - section 0, "Start the app": `npm run dev` "…shows a development banner, makes every cleanup a dry run (every
    check runs, nothing is moved: each item ends 'a dry run') and records the run in `NO_COMMIT/dev-runs/`"; `npm run
    dev:sandbox` "…cleans for real inside the sandbox and is recorded too (`…_sandbox-<name>.json`)"; a raw `tauri dev` (no launcher) is not offered by any script;
  - section 0, "Safety": the first bullet becomes "A development build never moves anything outside a sandbox: on the
    real Mac a cleanup is a dry run. A release build (`npm run build`) can, and is used by hand only in sections 3 and
    5." The read-only commands bullet stays;
  - section 2 is renamed "## 2. The real Mac: one recorded run" and rewritten:
    - **2.0 The run** (first, everything else of section 2 reads its record), a numbered list with a checkbox each:
      1. Quit any development window. Settings, Permissions: Downloads on (the system may ask: allow it). Note whether
         the terminal has Full Disk Access (the record notes it too).
      2. `npm run dev`. The banner shows. Scan `~` once; wait for the end.
      3. On every module's page select everything offered; Review: the dialog names the count, the size and the
         highest danger; Confirm to the Trash. Every item that passes its checks ends "Development build: a dry run…"; an item a check refuses keeps its own
      reason; nothing is freed.
         Finder: two of the offered folders are still where they were.
      4. Once more with "Delete permanently" chosen (when offered): the same, nothing gone. (Dropped by the final
         review's fixes: the real-home run uses the Trash only and first checks the banner and one "a dry run".)
      5. Cleanups: the two dry runs are listed; Put back answers that nothing went to the Trash.
      6. JetBrains (2.2), if an IDE is installed: open it, scan again, quit it, scan again (the record keeps all the
         scans).
      7. Quit. Hand the agent the record: `ls -t NO_COMMIT/dev-runs/*_mac.json | head -1`.
      8. The agent runs `npm run dev-runs:check -- <that file>` and goes through `<file>.check.md` with you; each
         difference is written in the conclusions of the step it belongs to (a module's 04/02 or 05/02, or 026/03/03).
    - **What the record and the checker answer** (a short list, so the user does not do it by hand): every offered
      path exists, is no link, is a file or a folder, is inside the roots and no protected or whole folder, and its
      size matches the disk; stale projects: the folder is an artifact or a Rust cache, no `src`, `.git`, `dist`, not
      in `~/Library`, an app or a hidden home folder, the project's folders and branches older than the days, the
      cache older than 7 days; downloads: first level of Downloads only; temporary files: in `T` or `/private/tmp`,
      unchanged for the days; the left-out lines with their counts and sizes; the dry run moved nothing.
    - **2.1 to 2.8**: each keeps its link to its step and only the items the disk cannot answer, with their commands:
      - 2.1 Storage map: unchanged (the map is not in the record);
      - 2.2 JetBrains: the version in use left out and "The IDE is open…" read from the record's two scans by the
        agent; by hand the bundle ids and versions (`mdls`) and the agent's calls they confirm;
      - 2.3 Temporary files: the sweep's log and plists, Finder's Put Back of a scratch file (calls 12, 13); the
        listing commands of ownership and age go (the checker does them);
      - 2.4 Developer caches: the facts "to verify" from the tools, the optional Finder cleanup; the `du` and `ls`
        comparisons go;
      - 2.5 Uninstalled app leftovers: per offered id, that no app is installed (`mdfind`, the apps folders) — the
        checker does not know the installed apps; the uncounted ids; shared frameworks; the disk image and the backup
        disk; the module's `finish` time; the Finder cleanup;
      - 2.6 Downloads and installers: the first scan after switching Downloads on offers items; an installer's app is
        installed at its version or newer (the About window); the facts "to verify" of the browsers and receipts; the
        Finder cleanup;
      - 2.7 Stale projects: this repository's `target/` over a week; the facts "to verify" (an editor saving in place,
        git actions and `refs/heads`, Finder's `.DS_Store`, hard-linked `node_modules`, incremental folders' times);
      - 2.8 Scan time: unchanged.
  - section 7 ("Plan step, status, section"): a row for 026/03/03 pointing at 2.0.
- `docs/testing.md` — "The development run record" (02/03) gains the checker: `npm run dev-runs:check --
  <record.json>` writes `<record>.check.md`, read-only on the disk; the commands table gains its row.

## Produces

The checklist the user follows in 03/03.

## Checks

- `npx prettier --check docs/manual-testing.md docs/testing.md`; every link of section 2 resolves (`grep -o
  '](\./[^)]*)' docs/manual-testing.md` and `ls` each path).
- Read section 2 once from top to bottom as the user would: every command named exists (`npm run dev`, `npm run
  dev-runs:check`), no item asks for a comparison the checker already makes, and every item removed from 2.1 to 2.8 is
  in "What the record and the checker answer".

## Conclusions

Section 2 of `docs/manual-testing.md` is now 2.0 (the run, eight boxes) plus the by-hand remainder of 2.1 to 2.8; the
list of what the checker answers is in 2.0. Section 0 says a development build makes every cleanup a dry run. The
row of 026/03/03 is in section 7; `docs/testing.md` gains the checker. No `dev:app` is left in the plan's files.
