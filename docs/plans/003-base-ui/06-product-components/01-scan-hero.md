# 003/06/01 — The scan hero

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `product/scan-hero/` — `ScanHero`: props `phase: 'idle' | 'scanning' | 'finished' | 'partial' | 'failed'`,
  `overview: Overview | null`, `progress`, `failure: Text | null`, `onScan`, `onStop`. A `Surface` (elevation 1,
  `--radius-xl`) over an `AmbientLight` (breathing while scanning).
  - Left: `StorageRing` with one segment per `CategoryTotal` (value: allocated size, level: `highestDanger`, label:
    the category's name), `total` = `overview.found`. The centre:
    - idle: the primary **Scan** button (large) on the empty track, with its static glow;
    - scanning, finished, partial: the label "Found", `AnimatedNumber` of `found` (`deliberate` on the first fill,
      `slow` after) in `--type-hero`, and "in N items · M categories" (plurals from the dictionary);
    - failed: `ErrorState` with the core's text and "Try again".
  - Right: three `StatTile`s (found, items, categories) and the `UsageBar` with its legend, both from the same
    segments; `ScanProgress` under them while scanning.
  - The choreography of the design: Scan pressed → the button presses and fades out (`--duration-fast`), the number
    fades in at 0, the sweep fades in (`--duration-base`); arcs grow as totals arrive; on finish the sweep fades out,
    the ring plays its one sheen, the ambient light stops breathing.
- `product/scan-progress/` — `ScanProgress`: "Looked at N items" (`AnimatedNumber`, tabular), a `ProgressBar`
  (indeterminate: the core reports no total), one chip per module of the scan — waiting (dim), running (an Iris dot
  with the scan's breath), done (`Check`), failed (`CircleAlert` and the core's reason in its tooltip), cancelled; a
  **Stop** button. It feeds `useAnnounce` with "Found X so far" (key `scan`, every 5 s) and the outcome as final.
- `locales/en.json` — `ui.scan-hero.*`, `ui.scan-progress.*` (labels, the plurals of items and categories, the
  announcements).

## Produces

The overview's hero (07/02).

## Checks

- Negative: `idle shows the Scan button and no number`, `the sweep is off when not scanning`, `a failed scan shows the
  core's text and no ring segments`, `an unknown found size shows — and never 0`, `the counting number is hidden from
  assistive technology`, `the announcer is not fed more than once per 5 seconds`.
- Positive: `Scan calls onScan`, `Stop calls onStop`, `segments follow the totals in the core's order and their
  highest levels`, `the centre shows found with its plural line`, `a finished scan announces its outcome once`, `module
  chips show done and failed with the reason`.
- Stories: idle, scanning (sweep and breath frozen), finished with the demo totals, finished with 12 categories,
  partial, failed, nothing found; scan progress starting, running, a module failed, cancelled; screenshots looked at.

## Conclusions

- Built: `ScanHero` and `ScanProgress`, with unit tests (negative first), stories for every listed state (seven of the
  hero, four of the progress) and WebKit screenshots, which were looked at. No new dependency, no new token, no new
  icon (the chips use `Check`, `CircleAlert` and `Minus`, Stop uses `Square`).
- `product/sizes.ts` turns the core's `Sizes` into `SizeText` props (allocated bytes; a lower bound "at least …" with
  the core's `core.completeness.partial` as its hint; unknown is a dash, never 0), a short label for the charts and a
  count format; its texts are `ui.size.*`. Step 02 and later product components reuse it.
- The hero size is in `--type-hero` with its unit a step smaller beside it: "102.6 GB" in the hero type does not fit
  inside the 232 px ring. So the hero counts with `useAnimatedNumber` itself (the number split from its unit), hidden
  from assistive technology beside a visually hidden copy of the real value; it remounts when the size falls (a new
  scan), so it never counts down. The first fill counts over `deliberate`, later totals over `slow`.
- Deviation: a failed scan leaves the ring empty and shows `ErrorState` (the core's text, "Try again" → `onScan`) in
  the right column instead of the ring's centre: the error state with a sentence of the core does not fit in the
  ring. The stat tiles show a dash ("Not scanned yet" to assistive technology) before a scan and after a failure.
- Deviation: the announcements are a hook of the hero (`use-scan-announcements.ts`, keys `ui.scan-hero.announce.*`),
  not of `ScanProgress`: the progress is shown only while scanning, so it is gone when the outcome must be spoken.
  "Found X so far" is fed whenever the found size grows, under the key `scan`; the `Announcer` speaks one per 5 s
  (the test checks the live region). The outcome is the core's `core.scan.outcome.*` text with the found size (or
  the core's failure text), spoken once, `final`, when the phase leaves `scanning`.
- The Scan button fades out over `--duration-fast` (`usePresence`) and is `inert` while it does; "Found" fades in
  over `--duration-base` in the same grid cell. The sweep, the sheen and the breathing light come from `StorageRing`
  and `AmbientLight` (`breathing` while scanning).
- Agent's calls: the hero's props are `phase`, `overview`, `progress: { entries, modules } | null`, `failure`,
  `onScan`, `onStop`; a module chip's state is `waiting | running` or the core's `ModuleOutcome` (the screen derives
  it). The item count is the sum of the totals' counts and the category count their number. A module's name is
  `<module>.enabled.label`, as the sidebar of 07/01 names it. A failed chip takes the focus, so its reason (a
  tooltip, linked as its description) is reachable from the keyboard.
- The stories share `scan-hero/story-data.ts`: the demo overview (finished, running, partial, nothing found), a
  generated scan of 12 categories (`largeScan`) and the core's texts they need, set by each story as it renders.
