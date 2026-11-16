# 003/08/02 — Accessibility

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/e2e/keyboard.spec.ts` — every gallery story's interactive component reached and operated by keyboard alone;
  the focus ring visible on each focus stop (a screenshot of the focused state per component, WebKit); a dialog traps
  the focus and returns it; no focus is lost on a screen change.
- `app/ui/e2e/contrast.spec.ts` — under `prefers-contrast: more`: glows and noise are off, borders stronger
  (screenshots of the overview and a card); the contrast test of 01/02 is re-run on the final tokens.
- `app/ui/e2e/zoom.spec.ts` — the overview, the module page and the confirmation at 125 % and 150 % page zoom: nothing
  overlaps or is cut (screenshots).
- Fixes found by the pass, each in the component or screen it concerns, with a test that failed before.
- `@axe-core/playwright` as a dev-only dependency (MPL-2.0, allowed by the user on 2026-11-15; never shipped) and
  `app/ui/e2e/axe.spec.ts` over every screen and gallery story, with no violation of WCAG 2.1 AA allowed.

## Produces

The accessibility part of "done", checked across the app, and the list of what VoiceOver says.

## Checks

- `npm run test:e2e:ui` passes with the new specs.
- **By hand with VoiceOver** (the agent cannot drive it; written in the plan's "Checks by hand"): the overview reads
  its heading, the hero's "Found" and its size, the cards with name, level and size; a scan announces its progress at
  most every 5 seconds and its end once; the module page's rows read title, level and size, and their position in the
  list; the confirmation dialog reads the count, the size and the highest level before its buttons; Cancel is
  focused first.
- **By hand with "Reduce motion" on**: nothing slides, scales or rotates; the scan shows its static state and text.

## Conclusions

Done on 2026-11-16. Four new specs (`axe`, `keyboard`, `contrast`, `zoom`) pass in WebKit and Chromium with the rest
of `npm run test:e2e:ui`; every new or changed screenshot was looked at (47 new: 36 focused looks, 4 under increased
contrast, 6 at page zoom, the story `Dialog/from-a-button`; 2 changed: the failed overview, without the toolbar's Scan
again, in both engines). One new dev-only dependency, `@axe-core/playwright` 4.14.0 (MPL-2.0, allowed on
2026-11-15; never shipped). No new token, icon or text of the app; the four level glows joined the increased-contrast
block. The flows spec matches a card by its new name ("Open Example caches …").

What the pass found, each fixed in the component it concerns, with a test that failed before:

- **Label in Name (axe, WCAG 2.5.3)**: a category card's button was named "Open Example caches" by `aria-label`
  while it shows the name, what it is, the level, the size and the count. The button is now named by its content: a
  visually hidden "Open {name}", then what it shows; the visible name is `aria-hidden` (it is in the hidden text) and
  the `aria-describedby` that repeated the name is gone. VoiceOver reads "Open Example caches, Example data: …, Safe,
  1.8 GB, 2 items, button". Unit tests read the visible name from the hidden-to-AT span; Playwright's substring name
  match kept the e2e locators.
- **Contrast (axe)**: the active sidebar row's trailing size, tertiary text on the pressed pill, was 4.48 : 1; it is
  secondary text there now, and the pair `--text-secondary` on `--surface-pressed` joined the token contrast test.
- **Scrollable region (axe)**: a dialog's long body without a control of its own could not be scrolled by keyboard.
  It takes the focus then (`region` named by the title), with the rule the scroll area already had, now one hook
  (`design/layout/scroll-area/use-overflow-focus.ts`) for both.
- **Focus dropped (keyboard)**: Scan in the ring turns into Stop, and Stop ends the scan; the focused button left the
  page and the focus fell to `<body>`. The frame now keeps it: when the focused element leaves the current screen, the
  focus goes to the screen's heading (`screens/keep-focus.ts`, used by every screen layer).
- **Glows under increased contrast**: the level glows (`--level-*-glow`: the card's icon halo, the result's and the
  empty state's light) stayed on; they are transparent under `prefers-contrast: more` with the other glows.
- **Zoom**: at 125 % the usage bar's legend broke "750 MB" over two lines; a legend size no longer wraps (the name
  gives way with its ellipsis).
- The VirtualList stories gave each row's checkbox its own tab stop, unlike the app's finding rows; they now pass
  `tabIndex={-1}` as `FindingRow` does (the row is the list's one tab stop).

The specs:

- `axe.spec.ts`: eleven screen states and every gallery story (in four parallel parts), WCAG 2.0/2.1 A and AA tags.
  One exemption, written in the spec: a contrast finding on the text of a greyed-out row whose own control is
  disabled (WCAG 1.4.3 sets no contrast for inactive components; ARIA gives a list item no disabled state, so the
  checker cannot tell).
- `keyboard.spec.ts`: every story, by component in eight parallel parts — each focus stop is reached by Tab (then
  Shift+Tab from the first: a modal dialog does not loop), looks different focused than blurred, and a button, link,
  checkbox, radio or switch answers its key with a click (swallowed, so the story stays as it is); one picture of the
  focused look per component (WebKit, 36). A dialog keeps the focus from the page behind and gives it back to its
  opener (new story `Dialog/from-a-button`); a screen change, a scan started by keyboard and a scan stopped by
  keyboard never drop the focus.
- `contrast.spec.ts`: under `prefers-contrast: more` no element casts coloured light (a blurred or spread coloured
  shadow, a coloured radial gradient) and the noise is off; without it both are drawn; a card's border is stronger;
  the overview and a card photographed. `npx vitest run src/design/tokens/contrast.test.ts` passes on the final
  tokens.
- `zoom.spec.ts`: the overview, the module page and the confirmation at 125 % and 150 % (the 1100 × 720 window as
  that many times fewer CSS pixels at that device scale): no text cut without an ellipsis, no number broken over two
  lines, no sideways scroll; photographed in WebKit.

Also fixed here, found in 07:

- **One control per action after a failed scan**: the toolbar's Scan again shows only after a finished or partial
  scan; a failed scan keeps the hero's Try again beside its reason (07/02's rule "no duplicate buttons").
- **Scanning chips of switched-off modules**: the store keeps `scan-started`'s module list (`scanModules`), and the
  chips are those modules; a scan learned of from an overview only (no `scan-started` seen) falls back to every module
  the core lists.

Agent's calls (to check): the focus falls back to the screen's heading, not to the control that took the place of the
one that left (Stop for Scan): one rule for every screen, no refs through the product components; the screenshots
of a spec are taken after its policy check, since Playwright's screenshot adds an inline style the CSP refuses; the
gallery spec's test got a 120 s timeout (more than a hundred stories under a full parallel run).

`docs/dependencies.md` does not exist on this plan branch yet (it came to `main` after the branch was cut); its row
for `@axe-core/playwright` (4.14.0, MPL-2.0, "Build and checks only", the UI's npm packages) is to be added when the
plan branch takes `main`.

Checks left by hand (written in the plan's "Checks by hand"):

- **VoiceOver**: the overview reads its heading, the hero's "Found" and its size, each card as "Open …" with what it
  is, its level and size; a scan announces its progress at most every 5 seconds and its end once; Scan pressed with
  the keyboard lands on the "Overview" heading; the module page's rows read title, level, size and their position;
  the confirmation dialog reads the count, the size and the highest level before its buttons, Cancel focused first;
  a long dialog body is announced as a region named by the dialog's title.
- **Reduce motion on**: nothing slides, scales or rotates; the scan shows its static state and text.
- **Increase contrast on** in the real window: the translucent sidebar and the dialog's backdrop blur (Playwright's
  WebKit draws no `backdrop-filter`).

