# 042/03/02 — The scanning card

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

While a scan runs the disk card's place holds the scanning card: the swoosh fills with the progress, the percent, what
the scan looks at and what it found so far; the tiles found so far follow, without controls. The design:
[`../00-design.md`](../00-design.md), sections 5 and 7; agent's call 13. Mockup:
[`scanning.dc.html`](../../../mockups/0.1.0/scanning.dc.html).

## Read first

01/06 (the estimate in the state), 03/01's screen, `screens/scan/{module-progress.ts,phase.ts}`,
`product/scan-progress/` (the module chips), `product/scan-hero/use-scan-announcements.ts` if 03/01 kept it, the
swoosh of 02/04, `e2e/visual.spec.ts` ("scanning, paused mid-stream"), `e2e/core-mock.ts` (scanning scripts).

## Files

- `screens/scan/progress.ts` — `scanPercent(bytes, expected, modules)`: with an estimate, `min(floor(100 * bytes /
  expected), 99)`; without, `floor(99 * finished / started)` of the modules; 0 before anything. `lookingAt(modules)`:
  the first started module not finished (its id), or `null`.
- `product/scanning-card/scanning-card.tsx` (+ css, stories, test) — `ScanningCard({ volumeName, percent, module,
  found })`: a `section` labelled `ui.scanning.label` "Scanning", height 226 px, radius 18 px, 1 px border
  `--tile-border`, background `--surface-card`, overflow hidden; `SwooshProgress` filling it (`value = percent / 100`,
  label `ui.scanning.progress` "Scanning, {percent} percent"); top-left (28 px, 22 px; column, gap 4 px):
  `ui.scanning.title` "Scanning {name}" (13 px, `--text-secondary`), `ui.scanning.percent` "{percent}%" (34 px, 700,
  letter-spacing -0.5 px), and, when a module is known, `ui.scanning.looking-at` "Looking at {module}" (13 px,
  `--text-legend`, the module's `<module>.enabled.label`); bottom-right (28 px, 22 px; column, end, gap 2 px): the
  found size (22 px, 700, `--part-can-be-freed`) over `ui.scanning.found` "found so far" (12 px, `--text-secondary`).
  The screen reader announcements of the found size, as today (`ui.scan-hero.announce.*`, kept or renamed with
  their keys moved to `ui.scanning.announce.*`).
- `screens/scan/scan-screen.tsx` — while scanning: the toolbar's action is `ui.scan-progress.stop` "Stop" (secondary);
  the scanning card for the first volume; the separator; the tiles found so far in one grid without groups, each tile
  with `scanning` (no checkbox, no Details, no menu; `category-tile` gains the prop) and one placeholder tile at
  opacity 0.45 (skeleton icon, two lines and a size, `aria-hidden`).
- Delete `product/scan-progress/` (the module chips) and its texts and baselines if nothing else uses it. A module that
  failed or was cancelled is still told after the scan by what already tells it (the coverage notices, the
  unavailable tiles); if nothing does, the screen shows one line under the separator, `ui.screen.scan.modules-failed`
  "{count} parts of the scan could not finish." — the report says which.
- `app/ui/src/locales/en.json` — the `ui.scanning.*` texts; deleted keys removed.
- `e2e/` — `visual.spec.ts` "scanning, paused mid-stream" (new baseline), `scan.spec.ts` (Stop during a scan; the
  percent reads from the mock's bytes and estimate), `motion.spec.ts` if it pauses the hero's light.

## Tests (negatives first)

`progress.test.ts`: negative "nothing walked is 0"; "an estimate smaller than the bytes holds at 99"; "without an
estimate or modules it is 0"; positive "bytes over the estimate"; "without an estimate, the modules finished".
`scanning-card.test.tsx`: negative "no module known shows no Looking at line"; positive "shows the name, percent,
module and found size"; "its progress bar reads the percent". `scan-screen.test.tsx`: negative "a tile found while
scanning has no checkbox"; positive "while scanning the card holds the swoosh and the toolbar holds Stop".

## Checks

`npx vitest run src/screens/scan src/product/scanning-card`; test:cov:ui; lint:ts, lint:code, knip; `npx playwright
test e2e/scan.spec.ts e2e/visual.spec.ts e2e/motion.spec.ts e2e/axe.spec.ts --project webkit --project chromium`
(look at the scanning baseline against the mockup).

Commit: `feat(ui): the scanning card fills the swoosh`.

## Conclusions

- The scanning card: the swoosh from `scanPercent` (bytes against the estimate, at most 99; without one, modules
  finished), the percent, "Looking at …", the size found so far; the tiles found so far without controls and one
  placeholder. The old hero, its progress and module chips are deleted.
- No "modules failed" line: a failed or cancelled module leaves the scan partial and the partial notice says so.
