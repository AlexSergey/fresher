# 011/01/03 — The UI shows only texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/product/problem-text/` (new; component, stories, test) — `ProblemText` renders `tText(text)`; when
  `text.args.detail` is a `text` arg, a disclosure "Show the details" opens it below in a quiet monospace line. It
  computes nothing: the sentence and the detail come from the core.
- `app/ui/src/product/result-summary/result-summary.tsx` — a failed item's reason through `ProblemText`.
- `app/ui/src/product/module-section/module-section.tsx` — a failed module's reason through `ProblemText`.
- `app/ui/src/locales/en.json` — `ui.problem.show-detail`: "Show the details"; `ui.problem.hide-detail`: "Hide the
  details".
- `eslint.config.ts` — for `app/ui/src/{screens,product}/**`, `no-restricted-syntax` on a member `.message` of a
  value typed or named as an error and on `error.code` of a refusal, with the message "show the core's text
  (`tText(refusal.text)`), never an error's code or message ([`docs/architecture/README.md`](../../../architecture/README.md), UI)".
  `app/ui/src/ipc/index.ts` stays outside it: `CommandFailed`'s message is the developer's console line.

## Checks

- Negative (`problem-text.test.tsx`): `shows no disclosure without a detail`, `keeps the detail closed until asked`,
  `never shows the detail inside the sentence`.
- Positive: `opens the detail on demand`, `shows the core's sentence as it is`; `result-summary` and `module-section`
  tests: a failed reason with a detail shows the disclosure.
- The lint: a file of `product/` that renders `error.message` fails `npm run lint` (checked once by hand on a scratch
  edit, not committed).
- `npx vitest run app/ui/src/product/problem-text app/ui/src/product/result-summary app/ui/src/product/module-section`;
  `npm run lint`; the UI e2e screenshots of the cleanup result and the module section unchanged without a detail.

## Conclusions

Built (c6d9af7). `product/problem-text` shows a text and, behind "Show the details" (one stable "Details" since the
final review), its `detail` arg; the result summary and the module section use it. The lint is `errorDetailConfig` in
`eslint.config.ts`: a `no-restricted-syntax` rule on `.message` and `.code` of values named `e`, `err`, `error`,
`reason`, `failure`, `cause` and on `.error.message|code` of a refusal, in `screens/` and `product/`. It is a name-based
tripwire, not a guarantee: an aliased or destructured read slips through. Two existing reads (`state/storage.ts`,
`ipc/index.ts`) are outside its scope and stay. Minor findings left for later and fixed in the final review:
a stable disclosure label, `aria-controls` on a hidden element, the key order of `en.json`.

### Agent's calls taken during the build (to check)

1. The lint is name-based, a tripwire and not a guarantee. Cost if wrong: an aliased or destructured read of an error slips through; review must catch it.

