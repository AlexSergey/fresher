# 042/08/02 — The interface texts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e, sandbox (demo trace)

Every text of `app/ui/src/locales/en.json`, read on the new screens and rewritten shorter; unused keys removed. The
design: [`../00-design.md`](../00-design.md), section 12.

## Read first

`docs/writing.md` (after 08/01), `app/ui/src/locales/en.json`, the mockups' texts (`docs/mockups/0.1.0/*.dc.html`:
where a mockup gives a text, it wins), `fixtures/demos/scenarios/*.toml` (targets named by text), `e2e/` (tests that
find an element by its text).

## The procedure

1. Screen by screen (Scan, the scanning card, Review, the confirmation, the agreement sheet, the results, Cleanups,
   Apps, Details, Settings and its search, the menus, the notices): every label, hint, heading and sentence in the
   shapes of `docs/writing.md`; a hint one sentence; a heading 2 to 5 words.
2. Remove every key nothing reads (`rg` each key's use; `knip` does not see JSON keys).
3. The e2e specs, the unit tests and the demo scenarios that find an element by a changed text follow it; the demo
   trace test passes.

## Checks

`npm run check:texts`; `npx vitest run`; test:cov:ui; lint:ts; `CARGO_TARGET_DIR=<target> cargo nextest run -p
fresher-sandbox` (the demo trace); `npm run test:e2e` (update the baselines where a text changed; look at each).

Commit: `feat(ui): plainer texts on every screen`.

## Conclusions

- 39 interface texts rewritten; no `ui.` key was unused. The bindings file is written by its generator only
  (lint-staged no longer formats `*.generated.ts`). The four `ui.screen.settings.value.*` texts lost their trailing
  space.
- The most important: the bar's region "Removal" → "Ready to remove"; "By group: {parts}" → "Size of each group:
  {parts}"; picked tiles "1 of 1 checked" → "1 picked"; "Described, not cleaned" → "Shown, never removed"; "The core
  is not answering" → "Fresher is not answering"; "Rescan to apply" → "Scan again to apply it" / "Scan now"; the
  ignored tabs → "Sections", "Not looked for" (08/03), "Items"; "Explained on Scan" → "Can be freed on Scan"; the
  confirmation's button → "Yes, move to the Trash" (08/03).
