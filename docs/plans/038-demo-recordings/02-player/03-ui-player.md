# 038/02/03 — The UI's player

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

In a development build, the UI asks the shell for a scenario; with one, it plays the steps on the real interface with
a drawn pointer, checks the page for private texts after each step, and reports the outcome. All of it is a lazy chunk
imported only under `import.meta.env.DEV`: the release bundle has none of it. The design:
[`../00-design.md`](../00-design.md), section 2, agent's calls 5 and 11.

Read first: 02/02 (as built: `demoPlay`, `demoFinished`, the generated types), `app/ui/src/main.tsx`,
`app/ui/src/state/store.ts` and `reduce.ts` (where a finished scan and a finished cleanup are seen), `app/ui/src/state/cleanup.ts`,
`app/ui/src/product/build-banner/`, `app/ui/e2e/support/` (how the e2e tests find elements by role and name, for the
same meaning of "accessible name"), `app/ui/vitest.config.ts`, the CSS tokens in `app/ui/src/design/tokens/`.

## Files

All under `app/ui/src/demo/` (new), each with its test beside it, negatives first:

- `finder.ts` — `findTarget(root: ParentNode, target: Target): Found`, `type Found = { kind: 'one'; element:
  HTMLElement } | { kind: 'none' } | { kind: 'many'; count: number }`. An element has a role when its `role`
  attribute names it, or implicitly: `button` (a `<button>`, an `<input type="button|submit">`), `link` (`<a href>`),
  `checkbox` (`<input type="checkbox">`), `heading` (`<h1>`–`<h6>`), `dialog` (`<dialog>`), `row` (`<tr>`),
  `region` (`<section>` with an accessible name); `tab` and `switch` by attribute only. Its accessible name is
  `aria-label`, else the texts of `aria-labelledby`'s ids joined by a space, else (for an input) its `<label>`'s text,
  else its text content; whitespace collapsed and trimmed. `NameMatch` `exact` compares the whole name, `start` its
  start. Only visible elements count: none inside `[hidden]`, `[inert]`, `[aria-hidden="true"]`, or with a computed
  `display: none` or `visibility: hidden` on it or an ancestor.
- `pointer.ts` + `pointer.module.css` — `createPointer(document: Document): Pointer`, `type Pointer = { readonly
  moveTo: (x: number, y: number) => Promise<void>; readonly press: () => Promise<void>; readonly remove: () => void }`.
  A fixed, `pointer-events: none`, topmost element with an arrow drawn in inline SVG (the design tokens' colours, a
  thin outline that reads on light and dark), starting at the window's centre; `moveTo` glides in `GLIDE_MS = 600`
  with an ease-in-out transform transition and resolves on `transitionend` (or after the duration when no transition
  runs, as in tests); `press` scales it down and back in 160 ms. It carries `data-demo-player` (the marker the release
  bundle check greps for).
- `steps.ts` — `runStep(step: Step, deps: StepDeps): Promise<StepResult>`, `type StepResult = { kind: 'done' } | {
  kind: 'failed'; reason: string }`; `StepDeps` = `{ root, pointer, store: CoreStore, sleep(ms), now() }` (injected so
  tests use fake time). Constants `TARGET_TIMEOUT_MS = 5000`, `STATE_TIMEOUT_MS = 60000`, `POLL_MS = 50`.
  - `click`: wait until the target is found exactly once (`none` until the timeout → "no element is the {role}
    “{name}”"; `many` → "{count} elements are the {role} “{name}”"), glide to its centre, press, `element.click()`.
  - `hover`: the same without the click; dispatch `pointerover`, `pointerenter`, `mouseover`, `mouseenter` (bubbling
    as the browser does) on it.
  - `scroll`: glide to the target, then `element.scrollBy({ top: by, behavior: 'smooth' })` and wait 400 ms.
  - `wait` a state: poll the store until it holds (`scan-finished`: the followed scan finished with an outcome;
    `cleanup-finished`: the cleanup state reached its result) or fail "the app did not reach {state} in 60 s"; `wait` a
    target: until found exactly once, failing as `click` does.
  - `pause`: sleep.
- `player.ts` — `play(play: DemoPlay, deps: PlayerDeps): Promise<DemoOutcome>`: `LEAD_IN_MS = 1500`, then each step in
  order; after each, the page's text (`document.body.textContent`, which includes tooltips and titles rendered into the
  DOM) must contain none of `play.private` (else "the page shows a private path after step N"). The first failure
  ends the play: `{ kind: 'failed', step: n, reason }` (n from 1). Else `{ kind: 'finished', elapsedMs }` measured from
  the end of the lead-in with `deps.now()`.
- `start.ts` — `startDemo(store: CoreStore): Promise<void>`: `demoPlay()`; `null` → nothing at all. Else set
  `document.documentElement.dataset.demo = 'playing'`, create the pointer, `play(...)`, remove the pointer, then
  `demoFinished(outcome)`. A rejected `demoPlay` is ignored (an old shell); a thrown error during the play is reported
  as `failed` at the step in hand.
- `app/ui/src/main.tsx` — keep the store in a constant; after `render`, `if (import.meta.env.DEV && inApp()) { void
  import('./demo/start').then(({ startDemo }) => startDemo(store)); }` with a comment: debug builds only, the release
  bundle has no player.
- `app/ui/src/product/build-banner/build-banner.module.css` — the banner is hidden while a demo plays:
  `:global(html[data-demo='playing']) .banner { display: none; }` (use the banner's real class name).

## Produces

The player, for 03/01 (the recorder starts the app and reads its outcome) and 04/01 (the scenarios).

## Checks

- `finder.test.ts`, negatives first: no element with the role; the name differs; two buttons with the same name →
  `many` with 2; a matching button inside `[hidden]`, inside `aria-hidden="true"`, with `display: none` → `none`; a
  `div` with the text is not a button. Then: an implicit button by its text; an `aria-label` wins over the text;
  `aria-labelledby`; a `name-start` match; a `role="switch"`.
- `steps.test.ts` (fake `sleep`/`now`, a real `createCoreStore`), negatives first: a click whose target never appears
  fails after 5 s with the message; two targets fail at once; a state that never comes fails after 60 s. Then: a click
  glides and clicks (a handler sees it); a hover fires `mouseenter`; a scroll calls `scrollBy`; `scan-finished` is seen
  once the store gets the finishing action; a target wait resolves when the element is added.
- `player.test.ts`: a private text on the page fails at that step; a failing second step reports `step: 2`; a played
  scenario reports `finished` with the elapsed time.
- `start.test.ts` (the IPC module mocked as other tests do): `null` → no pointer, no `data-demo`, no `demoFinished`;
  a scenario → `data-demo="playing"` while playing and `demoFinished` called with the outcome.
- `build-banner` test: still renders; (the CSS rule is checked by hand in 04/02).
- `npx vitest run src/demo src/product/build-banner`; `npm run lint:ts`; `npm run lint:code`; `npm run knip`.
- `npm run build --workspace app/ui` then `grep -rl "data-demo-player" app/ui/dist` finds nothing (the chunk is not
  in the release bundle); write the result in the conclusions.
- UI coverage stays at or above 90 % (`npm run test:cov:ui`).

## Conclusions

Built as briefed, in `app/ui/src/demo/`: `finder`, `pointer`, `steps`, `player`, `start`, each with its test, and
the banner rule in `build-banner.module.css`. `main.tsx` imports `./demo/start` lazily under
`import.meta.env.DEV && inApp()`.

- `npm run build --workspace app/ui` then `grep -rl "data-demo-player" app/ui/dist` finds nothing; the build has one
  JS chunk and no demo chunk.
  The marker is set with `setAttribute('data-demo-player', '')`, so the string is literal in any bundle that holds
  the player (set as `dataset.demoPlayer` it would never appear and the grep would prove nothing).
- UI coverage: 98.5 % lines, 92.5 % branches.
- A hover dispatches `pointerover` and `mouseover` bubbling and `pointerenter` and `mouseenter` not bubbling, as the
  browser does. An empty private text is ignored (it would match every page). `POLL_MS` is not exported (nothing
  else uses it).
- The perf specs of the full Playwright run are timing-bound and failed under load; they were not part of this step.

2026-11-25, the first `npm run demo:check -- all`: both scenarios failed at step 1, "the page shows a private path".
The check reads `textContent`, which holds hidden text, and the banner rule only hid the banner (`display: none`): its
line, with the sandbox's folder and its Trash, stayed in the page. Then the review rows failed: a path shown from `~`
keeps its full path as its accessible text and tooltip. Fixed in the UI, the check kept as strict:

- `CoreState.demo`, set by the action `demo-started` that `startDemo` dispatches before it plays. While it is set, the
  frame renders no banner and a cleanup's result says nothing of where the sandbox's Trash is. The CSS rule is gone.
- `PathFormContext` (`design/data/path-text/path-form.ts`): how `PathText` writes a path, shown, in its tooltip and as
  its accessible text; as it is by default. `DemoPathForm` (`screens/demo-path-form.tsx`, in `AppProviders`) writes
  every path from the home as `~` while a demo plays.
- Tests, negatives first: `reduce.test.ts`, `start.test.ts`, `path-text.test.tsx`, `app-frame.test.tsx` (no banner
  while a demo plays) and `review-screen.test.tsx` (no full path in the review and its confirmation; no Trash
  sentence in the result).
