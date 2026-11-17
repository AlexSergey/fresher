# 012/04/03 — Neutral tones

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/design/data/segment.ts` — `Segment.level: Level` becomes `Segment.tone: Tone`, with
  `type Tone = Level | 'plain' | 'system' | 'free' | 'unknown'`: the danger levels keep their colours; `plain` (your
  folders) a quiet graphite, `system` (macOS, swap, the disk's own space) a darker graphite, `free` an outline with no
  fill, `unknown` (the remainder, not scanned yet) a hatched graphite. Light comes only from Iris and the danger levels
  (goal, section 6): the neutral tones carry none. A rename touches every caller of `Segment` (the storage ring, the
  usage bar, `scan-hero`, `chart-stories.ts`, their tests); no behaviour changes for them.
- `app/ui/src/design/data/usage-bar/usage-bar.tsx`, `usage-bar.module.css` — `data-tone` instead of `data-level`, the
  four new tones as tokens of `design/tokens/`; the legend's dot follows the tone; a segment under 0.5 % of the total
  is drawn at a minimum visible width only when its size is not zero, and its legend row is always there.
- `app/ui/src/design/data/storage-ring/` — `data-tone` likewise (same tokens).
- `app/ui/src/design/data/usage-bar/usage-bar.stories.tsx` — a story of a whole volume with every tone.
- `app/ui/src/design/data/segment.test.ts`, `usage-bar.test.tsx` — the tones.

`design/` stays generic: it knows tones, never parts of a map (the import boundaries of `eslint.config.ts`).

## Produces

The tones the map's bar (04) draws its parts in.

## Checks

- Negative first: `a_zero_segment_is_not_drawn` (kept), `a_tiny_segment_keeps_its_legend_row`,
  `free_has_no_fill` (the CSS module's rule, through the rendered `data-tone`).
- Positive: `each_tone_is_drawn_with_its_token`, `the_accessible_name_lists_every_part` (unchanged rule:
  `chartLabel`); the gallery's screenshots of the usage bar and the ring updated on purpose
  (`npm run test:e2e`, the gallery project).
- `npx vitest run src/design/data`; `npm run lint`.

## Conclusions

Built in `32b4b73`, reviewed clean.

- `Segment.tone` (levels, `plain`, `system`, `free`, `unknown`) with `--tone-*` tokens on the bar and the ring; a
  whole-volume story and gallery screenshot.
- Parked: the ring's `unknown` stroke is not hatched, as the ring shows no `unknown` yet.
- Measured: `npx vitest run` 847 passed at the step; Playwright gallery passed.
