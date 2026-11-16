# 003/08 — Proof

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui, docs, repo

What turns "it looks right" into "it is right": the performance budget measured, the accessibility pass by keyboard,
VoiceOver and contrast, the docs that describe what was built, and the version. The design:
[`../00-design.md`](../00-design.md), "Performance budget", "Accessibility".

Depends on plan 07. 01 and 02 run in parallel; 03 last, then the global plan's conclusions.

## Steps

| #   | Step                                              | Status   | Depends on | Shared files                                                                           |
| --- | ------------------------------------------------- | -------- | ---------- | -------------------------------------------------------------------------------------- |
| 01  | [Performance](./01-performance.md)                | ✔️ done    | —          | `app/ui/package.json` (`test:perf`), `app/ui/playwright.config.ts`                     |
| 02  | [Accessibility](./02-accessibility.md)            | ✔️ done    | —          | components and screens it fixes                                                        |
| 03  | [Docs and version](./03-docs-and-version.md)      | ✔️ done    | 01, 02     | `docs/architecture/README.md`, `docs/testing.md`, `docs/quality.md`, the version files |

## Conclusions

Done on 2026-11-16; each step's conclusions are in its file.

- **Performance** (01): `npm run test:perf --workspace app/ui` measures the scan of 50,000 findings, the screen
  changes, the dialog, the selection bar, a re-sort of the cards and a scroll through 100,000 rows in WebKit (where
  the budget is held) and in Chromium (long tasks and the JS heap). On the reference Mac (MacBook Pro, M3 Pro, macOS
  27.2.1, Playwright's WebKit 27.2 at 60 Hz) every scenario has a p95 of 18 ms, the idle page's own, but the overview
  during the scan under heavy machine load (22 ms, one frame of 86 ms in one run of three; 13 reruns, 8 of them on a
  quiet Mac, had no frame over 50 ms). At most 27 list rows in the DOM; the JS heap grows by about 2 MB over the
  scan; the bundle is 99.1 kB of JS (budget 200) and 12.4 kB of CSS (budget 40), gzipped. Nothing was changed for it.
- **Accessibility** (02): new specs `axe`, `keyboard`, `contrast` and `zoom` in both engines; seven defects found and
  fixed, each with a test that failed before (a card's name, a contrast pair, a scrollable dialog body, the focus
  dropped when Scan becomes Stop, the level glows under increased contrast, a wrapped legend size at 125 %, the list
  stories' tab stops), and two leftovers of 07 (one control per action after a failed scan, the scanning chips of
  switched-off modules). One new dev-only dependency, `@axe-core/playwright` 4.14.0 (MPL-2.0, allowed 2026-11-15).
- **Docs and version** (03): the docs describe the UI as built; the version is 0.2.0, `Fresher.app` is 6.94 MiB.
- Agent's calls to check: 01's p95 bar held against the idle page's p95 and its reading of the one 86 ms frame;
  02's focus fallback to the screen's heading and its one axe exemption. The checks by hand left are in the global
  plan's conclusions.
