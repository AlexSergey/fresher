# 003/03/04 — Feedback

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `design/feedback/notice/` — `Notice`: `tone: 'info' | 'warning' | 'danger'`, `title`, `children`, optional
  `action` (a `Button`); an inline banner on the level's tint with its icon (`Info`, `TriangleAlert`, `OctagonAlert`);
  `role="status"` (info) or `role="alert"` (warning, danger) only when it appears after the page did.
- `design/feedback/empty-state/` — `EmptyState`: an icon in a soft Iris halo, a title, one sentence, an optional
  action; centred in its container.
- `design/feedback/error-state/` — `ErrorState`: the same layout with `CircleAlert`, the core's text as given, and a
  "Try again" action from the caller.
- `design/feedback/toast/` — `Toaster` (one region at the bottom centre, `aria-live="polite"`, elevation 3, solid) and
  `useToasts()` → `show({ tone, text, action? })`; a toast rises `--distance-2` and fades in, leaves by fading
  (`usePresence`); info and success hide after 5 s (paused while hovered or focused), an error stays until closed; at
  most 3 at once, the oldest leaves first.
- `design/feedback/announcer/` — `Announcer` (one visually hidden polite live region) and `useAnnounce()` →
  `announce(text, { key, minInterval })`: the same `key` speaks at most once per `minInterval` (default 5 s), the last
  text of a burst is the one spoken; `announce(text, { final: true })` speaks at once.

## Produces

The feedback the screens use: a partial-scan notice, empty and failed lists, toasts for refused commands, the scan's
spoken progress.

## Checks

- Negative: `a notice present from the first render is not announced`, `an error toast does not hide by itself`, `a
  toast does not hide while hovered`, `a fourth toast pushes out the oldest`, `the announcer does not speak the same key
  twice within its interval`.
- Positive: `a warning notice that appears is an alert`, `the error state shows the given text and calls Try again`,
  `an info toast hides after 5 seconds` (fake clock), `the announcer speaks the last text of a burst`, `a final
  announcement speaks at once`.
- Stories: notice info, warning, danger, with action; empty state; error state; toast info, success, error, with
  action; screenshots looked at.

## Conclusions

Done on 2026-11-16. `Notice`, `EmptyState`, `ErrorState`, `Toast` with `Toaster` and `useToasts()`, `Announcer` with
`useAnnounce()` in `design/feedback/`, and `FeedbackAction` (`{ label, onClick }`, the one action shape of the folder).
33 unit tests (every check above, plus the action of each component, a toast paused by the focus, the close button,
the fading exit, a waiting announcement cleared at unmount, separate keys, the hooks outside their provider); 10
gallery stories photographed in WebKit and looked at. One icon added: `CircleCheck` (the success toast).

Decided while building:

- **`Notice` takes `live`**: a component cannot tell by itself whether it came with the page or after it, so the
  caller says so (`live` when it appears later, as the partial-scan notice does); only then is it a `status` (info) or
  an `alert` (warning, danger). Without `live` it has no role and is read with the page.
- **The tones' colours**: info on Low's cyan, warning on Medium's amber, danger on High's red — the levels' tint,
  border and fg tokens; no new token. `ErrorState` is the `EmptyState` with `tone="danger"` (`CircleAlert` in High's
  glow instead of the Iris halo).
- **`Toaster` is the provider** (`<Toaster closeLabel label>{app}</Toaster>`): it renders the region after its
  children, a `<section>` named by `label` with `aria-live="polite"`. `Toast` is its presentational card, which the
  gallery photographs (the fixed region would fall outside the story's frame). The action closes the toast after
  running.
- **The 5 s timer starts again in full** when the pointer and the focus leave a toast, rather than resuming where it
  stopped: simpler, and it never hides a toast just after the user looked away.
- **At most 3 open**: a fourth closes the oldest open one, which fades out (`usePresence`, `--duration-fast`) while
  the new one rises in.
- **`announce(text)` without a key speaks at once.** A repeated text is spoken again: every other message ends with a
  no-break space, so the region always changes. The rate uses `Date.now()`, faked in tests.
- **The hooks throw outside their provider**: a missing `Toaster` or `Announcer` is a wiring mistake, caught at once.
