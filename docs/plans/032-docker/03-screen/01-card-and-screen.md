# 032/03/01 — The card, the section and the insight screen

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

Read first: `CLAUDE.md` (TypeScript / React rules), `docs/rules.md` rule 7, [`../00-design.md`](../00-design.md)
section 7, plan 027's design (`docs/plans/027-scan-screen/00-design.md`, the scan screen), and as the shapes to follow
`app/ui/src/screens/scan/left-out-only.tsx` (a section under the zones, hidden modules filtered),
`app/ui/src/screens/category/category-screen.tsx` (a screen with Back, `PageHeader`, `DangerBadge`, `SizeText`),
`app/ui/src/product/category-card/` (a product card with stories and tests), `app/ui/src/state/navigation.ts`.
01/01 is built: `Insight` is in `bindings.generated.ts` and `CoreState.modules[].insights` holds them.

Nothing here is a category: no checkbox, no danger zone, no selection, no Review. Every sentence of an insight is the
core's text; the UI renders it.

## Files

- `app/ui/src/product/insight-card/insight-card.tsx` (+ `.module.css`, `.stories.tsx`, `.test.tsx`) (new) — props
  `{ readonly insight: Insight; readonly onDetails: () => void }`: the title (`tText`), the summary, the figures as a
  short list (label, `SizeText` of the size), a **Details** button (`ui.insight-card.details` "Details",
  `aria-describedby` the title). A heading level the section sets (`h3`). Graphite and Iris tokens only, in the shape
  of the category card, without its checkbox and danger colour. Stories: answered (three figures), stopped (two),
  nothing known (none). Registered in `app/ui/src/gallery/registry.ts`.
- `app/ui/src/screens/scan/described-only.tsx` (+ `.module.css`, `.test.tsx`) (new) — `DescribedOnly`: a section
  with the heading `ui.screen.scan.described.title` "Described, not cleaned" and the sentence
  `ui.screen.scan.described.text` "Fresher shows what these take and how you can free it yourself. It never removes any
  of it."; one `InsightCard` per insight of `state.modules` in their order (module, then insight), skipping the modules
  the overview's `hiddenModules` lists; **Details** navigates to `{ kind: 'insight', module, identity }`. Renders nothing
  while scanning or when there is no insight.
- `app/ui/src/screens/scan/scan-screen.tsx` — renders `<DescribedOnly />` right after `LeftOutOnly`, under the same
  condition (`views`).
- `app/ui/src/state/navigation.ts` — `Screen` gains `{ readonly kind: 'insight'; readonly module: ModuleId; readonly
  identity: string }`; a screen whose insight is not among the current modules' insights falls back to the start, as a
  category no longer listed does.
- `app/ui/src/screens/screen-stack.tsx` — `screenKey` gives `insight:<module>/<identity>`.
- `app/ui/src/screens/app-frame.tsx` (and `frame-sidebar.tsx` if it maps every kind) — renders `InsightScreen` for the
  `insight` kind, with the insight found in `state.modules`.
- `app/ui/src/screens/insight/insight-screen.tsx` (+ `.module.css`, `.test.tsx`) (new) — props `{ readonly insight:
  Insight }`: Back (`ui.insight.back` "Back", to the scan screen), `PageHeader` with the title and the summary; the
  figures; then each part as a row: its label (`h3`), `DangerBadge` with the level's name (`levelName`), `SizeText`
  of its size, the detail, the effect, and when it has a command: the line in a `<code>` that is selectable
  (`user-select: text`) and a **Copy** button (`ui.insight.copy` "Copy", `aria-describedby` the part's label); then
  the notes as paragraphs. No checkbox, no selection bar.
- `app/ui/src/screens/insight/copy-text.ts` (+ `.test.ts`) (new) — `copyText(text: string): Promise<boolean>`:
  `navigator.clipboard.writeText(text)`; `true` when it resolves, `false` when it rejects or the API is missing. Not a
  call to the core, so not in `ipc/`. The screen announces `ui.insight.copied` "Copied." on `true` and
  `ui.insight.copy-failed` "Could not copy. Select the command and copy it." on `false` through the app's
  `Announcer`, and shows the same words beside the button for a few seconds.
- `app/ui/src/locales/en.json` — the keys above, plain English.

## Produces

The section, the card, the screen and Copy, for 05/02.

## Tests (negative first)

- `insight-card.test.tsx` — `describe('negative cases')`: `shows no figure list when the insight has no figure`,
  `has no checkbox`; `describe('positive cases')`: `shows the title, the summary and every figure with its size`,
  `calls onDetails from Details`.
- `described-only.test.tsx` — negative: `renders nothing before a scan`, `renders nothing while scanning`,
  `renders nothing when no module described anything`, `leaves out a hidden module's insights`; positive:
  `shows one card per insight in module order`, `Details opens the insight's screen`.
- `insight-screen.test.tsx` — negative: `a part without a command has no Copy button`, `has no checkbox or selection
  bar`, `says it could not copy when the clipboard refuses`; positive: `shows each part with its level, size, detail
  and effect`, `shows the command line as the core gave it`, `Copy writes the line and says Copied`, `Back returns to
  the scan screen`, `shows the notes`.
- `navigation.test.ts` — negative: `an insight screen whose insight is gone falls back to the start`; positive:
  `navigates to an insight screen`.
- `copy-text.test.ts` — negative: `answers false without the clipboard API`, `answers false when writing is refused`;
  positive: `answers true when the text is written`.

## Done when

- `npx vitest run` for the touched files, then the UI's whole unit run once; `npm run lint`; coverage of the UI at or
  above 90 %; the gallery builds.
- `rg -i docker app/ui/src` finds nothing outside test data.
- One commit: `feat(ui): the described-not-cleaned section and the insight screen with Copy`.

## Conclusions

Not written yet.
