# 003/01/03 — The motion system

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/design/tokens/motion.css` — the design's motion tokens (durations, easings, distances, scales, stagger);
  `[data-motion='reduced']` and `@media (prefers-reduced-motion: reduce)` set distances to 0, scales to 1, durations to
  at most `--duration-fast` and pause loops; `[data-motion='off']` sets every duration and delay to 0 and
  `animation-play-state: paused` on every loop at its first frame.
- `app/ui/src/design/tokens/motion.ts` — the same values for JavaScript (`DURATION`, `EASING`, `STAGGER`), `as const`.
- `app/ui/src/design/motion/use-motion.ts` — `type MotionMode = 'full' | 'reduced' | 'off'`; `readMotion()` (the
  `data-motion` attribute wins, then the media query) and `useMotion()` (live, through `useSyncExternalStore` on the
  media query and a `MutationObserver` on the attribute).
- `app/ui/src/design/motion/use-presence.ts` — `usePresence(open, { exit: Keyframe[], duration })` →
  `{ mounted, ref }`: keeps the element mounted until its exit animation (`element.animate`) finishes; `off` unmounts
  at once, `reduced` uses an opacity-only exit.
- `app/ui/src/design/motion/use-animated-number.ts` — `useAnimatedNumber(target, { duration })` → the value to show:
  a `requestAnimationFrame` tween (ease-out cubic) that retargets from the shown value; `reduced` and `off` return the
  target.
- `app/ui/src/design/motion/use-flip-list.ts` — `useFlipList(keys, { measure | positions })`: before a reorder it keeps
  each key's position (measured with one batched `getBoundingClientRect` pass, or given by the caller), after it plays
  `translate(dx, dy) → none` with `--duration-move` and `--ease-standard` through WAAPI; a key that is new is left to
  its entrance; `will-change` only while playing.
- `app/ui/src/design/motion/use-spotlight.ts` — `useSpotlight(ref)`: on `pointermove` stores the position and writes
  `--spot-x` / `--spot-y` once per frame; nothing under `off`.
- `app/ui/src/design/motion/index.ts` — the public API of the folder.
- `app/ui/src/design/tokens/motion.test.ts` — `motion.css` and `motion.ts` agree.
- `app/ui/src/test/animation.ts` — test helpers: a fake `requestAnimationFrame` driven by the fake clock, a stub
  `Element.prototype.animate` that records keyframes and resolves `finished` on demand (happy-dom has no WAAPI), and
  `setMotion(mode)`.

## Produces

The motion tokens, the motion mode and the five hooks every animated component uses.

## Checks

- Negative: `use-motion: an unknown data-motion value is full`, `use-presence: a closed element that was never open
  is not mounted`, `use-presence: reopening during the exit keeps the element`, `use-animated-number: off returns the
  target at once`, `use-animated-number: a target change mid-tween starts from the shown value, not from the old
  start`, `use-flip-list: a new key does not animate`, `use-flip-list: an unchanged order plays nothing`,
  `use-spotlight: writes nothing under off`.
- Positive: `use-motion follows the media query and the attribute`, `use-presence unmounts after the exit finishes`,
  `use-animated-number reaches the target after the duration`, `use-flip-list plays the inverted translation for a
  moved key`, `use-spotlight writes once per frame`.
- `npx vitest run src/design src/test` passes; the CSS guard passes on `motion.css`.
