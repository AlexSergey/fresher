# 003/04/04 — Light and texture

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/effects/ambient-light/` — `AmbientLight`: an absolutely placed layer behind a screen's hero: a wide radial
  Iris glow from the top centre (`radial-gradient(60% 50% at 50% 0%, …)` at 14 %) and a faint second glow
  (`tone: 'iris' | 'safe'`; `safe` is the cleaned moment's brief green). `breathing` animates its opacity between 0.7
  and 1 over `--duration-breath` (a loop allowed only while work runs); a tone change crossfades two layers by opacity
  over `--duration-deliberate`. Static under reduced motion; off under increased contrast. `aria-hidden`.
- `design/effects/spotlight/` — `Spotlight`: a `::before` layer of a card holding a 240 px radial highlight (white at
  5 %), moved by `transform: translate(var(--spot-x), var(--spot-y))` from `useSpotlight`, faded in on hover over
  `--duration-fast`; none under reduced motion or off, nor on keyboard focus. Exposed as a `useSpotlightProps()`
  that a card spreads on its root, so it adds no element.
- `design/base/effects.css` — the noise layer is tuned here against the real surfaces (opacity 3 %, blend `overlay`
  checked against `soft-light` in screenshots), and turned off under increased contrast.

## Produces

The light of the overview's hero and the cleaned moment (06/01, 06/05), and the cards' hover (06/02).

## Checks

- Negative: `the ambient light does not breathe unless asked`, `the ambient light is still under reduced motion`,
  `the spotlight writes nothing without a pointer`, `the noise and the glows are off under increased contrast` (the
  media query emulated in Playwright).
- Positive: `the tone change crossfades`, `the spotlight follows the pointer once per frame`.
- Stories: ambient light at rest, breathing (frozen), safe tone; a raised surface with the spotlight at its centre;
  the window with and without noise; screenshots looked at side by side to choose the noise blend.

## Conclusions

- Built as planned: `AmbientLight` (`design/effects/ambient-light/`), `useSpotlightProps()`
  (`design/effects/spotlight/`), the noise's tuning in `design/base/effects.css`; unit tests (negative first), an e2e
  spec (`e2e/effects.spec.ts`, WebKit and Chromium) emulating `prefers-contrast: more`, stories and screenshots.
- **Colours without new tokens:** each glow is a radial gradient of an existing semantic glow colour
  (`--accent-glow`, Iris 500 at 35 %; `--level-safe-glow`, green at 40 %) on a layer whose opacity sets the strength:
  14 % for the wide glow (`60% 50% at 50% 0%`), about 6 % for the faint inner one (`30% 28% at 50% 0%`). The spotlight
  is `--text-on-fill` (white) shown at opacity 0.05. Both tones are always drawn and crossfade by opacity over
  `--duration-deliberate`.
- **Breathing** (opacity 1 → 0.7, alternate, `--duration-breath`) is switched on by `data-breathing` only when asked and
  the motion mode is `full`: under reduced motion and motion off the light is still, so the "breathing (frozen)"
  story is the rest state.
- **The spotlight** is the card root's `::before` (240 px, `isolation: isolate` and `z-index: -1`, so it lies over the
  card's fill and under its content; the card gets `overflow: hidden` to keep it inside the radius). It shows on
  `:hover` (or the gallery's `data-hover`), never on focus, and is not drawn under reduced motion. Under motion off it
  is drawn but does not follow the pointer (`useSpotlight` writes nothing), so the gallery can show it at a place the
  story sets.
- **The noise blend — agent's call:** the three blends were photographed in WebKit on the real surfaces and measured
  (the grain is the difference between the image and its 4 px blur, in grey levels, on a flat part of the main pane):
  none 0.24, normal 0.75, `overlay` 0.37, `soft-light` 0.37. On these dark surfaces `overlay` and `soft-light` give the
  same image (a mean difference of 0.04 levels), and their grain is too weak to break the gradients' bands, which stay
  visible when the image is brightened; the normal blend dithers them. The noise stays at 3 % with the normal blend.
- `:root[data-noise='off']` turns the noise off for the page; only the "window without noise" story sets it.
