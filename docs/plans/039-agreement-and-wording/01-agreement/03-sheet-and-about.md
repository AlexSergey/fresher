# 039/01/03 — The sheet and About

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** app/ui, sandbox (demo scenario)

The interface of the agreement: it is asked before the first cleanup is prepared, read again under Settings → About,
and the full licence is one click away. The engine is the guard (01/01); the sheet is how a person meets it. The
design: [`../00-design.md`](../00-design.md), section 1; agent's calls 8, 10 and 13.

## Read first

`app/ui/src/ipc/index.ts` (`COMMANDS`, one typed function per command), `app/ui/src/ipc/mock.ts`,
`app/ui/src/state/{reduce.ts,store.ts,connect.ts,cleanup-actions.ts}`, `app/ui/src/screens/app-frame.tsx` (where the
quit question renders at frame level), `app/ui/src/design/layout/sheet/` and `dialog/`,
`app/ui/src/screens/settings/settings-screen.tsx` (the About group), `app/ui/e2e/core-mock.ts` and
`app/ui/e2e/flows.spec.ts`, `visual.spec.ts`, `axe.spec.ts`; the e2e mock's parts in `app/ui/e2e/support/`
(`cleanup-mock.ts`, `apps-mock.ts`: one file per part, wired in `core-mock.ts`) and `app/ui/e2e/core-mock.spec.ts`;
`app/ui/src/ipc/mock.ts` (`mockCore`: a command without an answer fails; `sampleStorageAnswers` is the pattern for
shared answers); plan 036's uninstall: `prepareUninstallPlan` in `cleanup-actions.ts`, `screens/apps/app-screen.tsx`
(its own `ConfirmDialog`, no Review) and `app/ui/e2e/apps.spec.ts`; plan 038's demo:
`fixtures/demos/scenarios/cleanup.toml`, `crates/sandbox/src/scenario/traced.rs` (every target traced to a text key)
and `app/ui/src/demo/`.

## Files

- `app/ui/src/ipc/index.ts` — `getAgreement(): Promise<AgreementView>`, `acceptAgreement(version: number):
  Promise<AgreementView>`; their entries in `COMMANDS`. `app/ui/src/ipc/mock.ts` exports `sampleAgreementAnswers`
  (`getAgreement` answering an accepted view, as the core builds it: the five lines' keys, `changes: null`), and every
  unit test whose tree runs `connect.ts` spreads it into `mockCore` (find them with
  `grep -rln 'mockCore(' app/ui/src`; agent's call 13).
- `app/ui/src/state/` — the store holds `agreement: { readonly view: AgreementView | null; readonly asking: null |
  AgreementNext }`, where `AgreementNext` is a discriminated union of what to do once accepted (`{ kind: 'prepare' }`,
  and `{ kind: 'uninstall'; app: AppId; choice: UninstallChoice }`). Actions `agreement-loaded`, `agreement-asked`,
  `agreement-dismissed`. `connect.ts` loads the view once, beside the dictionary; a failed load leaves it `null` and is
  not the frame's error (the gate loads it again when it asks).
- `app/ui/src/state/cleanup-actions.ts` — `prepare` and `prepareUninstallPlan` first ask: when the view is not
  accepted (`accepted?.version !== view.version`), they dispatch `agreement-asked` with their next step and call
  nothing else; when it is `null`, they call `getAgreement` first (then decide; an unreachable core is the frame's
  error, as today). Neither dispatches `cleanup-preparing` before the agreement is accepted. New `acceptAgreementAndGoOn(store)`: `acceptAgreement(view.version)`, `agreement-loaded` with the
  answer, `agreement-dismissed`, then runs the held next step; a refusal (`agreement-changed`) loads the view again and
  keeps the sheet open with the new text; `declineAgreement(store)` only dismisses.
- `app/ui/src/product/agreement-sheet/` (new: `agreement-sheet.tsx`, `.module.css`, `.stories.tsx`, `.test.tsx`) —
  props `{ view: AgreementView; onAccept; onDecline; onLicence }`. On `Sheet`: title (`view.title`), the "what
  changed" notice when `view.changes` is set (`ui.agreement.changed` then the core's text), the intro, the five lines
  as a list, a link-styled button "Read the full licence" (`ui.agreement.licence`), then the footer "Not now"
  (`ui.agreement.later`, secondary, first focus) and "I understand" (`ui.agreement.accept`, primary). Escape and the
  close button decline. Static: no motion beyond the sheet's own entrance. Every text from the core's view or the UI
  dictionary.
- `app/ui/src/product/licence-dialog/` (new) — a `Dialog` titled `ui.licence.title` with two sections,
  `ui.licence.mit` and `ui.licence.apache`, each the file's text in a scrollable, selectable `<pre>`; the texts are
  imported at build time from the repository root (`../../../../../LICENSE-MIT?raw` and `LICENSE-APACHE?raw`, the
  depth checked). If Vite's dev server refuses a file outside `app/ui`, add the repository root to `server.fs.allow`
  in `vite.config.ts`; if `tsc` needs it, a `vite/client` types reference for `?raw` already exists or is added. No
  network, no browser.
- `app/ui/src/screens/app-frame.tsx` — renders `AgreementSheet` (and its `LicenceDialog`) when
  `agreement.asking !== null`, beside the quit question.
- `app/ui/src/screens/settings/settings-screen.tsx` — the About group: the five lines (the same component's list part
  or a small `AgreementText` shared by both), then `ui.agreement.accepted-on` ("You accepted it on {date}.", the date
  formatted as the Cleanups screen formats dates) or `ui.agreement.not-accepted` ("Not accepted yet. Fresher asks
  before your first cleanup."), and the "Read the full licence" button; the existing licence line stays.
- `app/ui/src/locales/en.json` — exactly: `ui.agreement.accept` "I understand"; `ui.agreement.later` "Not now";
  `ui.agreement.licence` "Read the full licence"; `ui.agreement.changed` "What changed since you last accepted it:";
  `ui.agreement.accepted-on` "You accepted it on {date}."; `ui.agreement.not-accepted` "Not accepted yet. Fresher asks
  before your first cleanup."; `ui.licence.title` "Licence"; `ui.licence.mit` "MIT licence"; `ui.licence.apache`
  "Apache License 2.0"; `ui.licence.close` "Close".
- `app/ui/e2e/support/agreement-mock.ts` (new, the shape of the other parts) wired in `app/ui/e2e/core-mock.ts` — the
  agreement in the mock, accepted by default; a script option `agreement: 'not-accepted'` answering a view without
  acceptance until `acceptAgreement` is called, which records the call and answers the accepted view. The mock never
  answers a `changes` text (the core has no second version yet; agent's call 10). `core-mock.spec.ts` gains a case for
  each state.
- `app/ui/src/screens/apps/app-screen.tsx` — nothing changes in it: the gate is in `prepareUninstallPlan`; the frame's
  sheet covers the app page as it covers Review.
- `fixtures/demos/scenarios/cleanup.toml` — after the step that clicks "Move to Trash…" and its pause, two steps:
  `click = { role = "button", name = "I understand" }` (traced to `ui.agreement.accept`) and `pause = 800`; the head
  comment names the sheet among the screens ("accepts the agreement, as a user would, the first time"). The world's
  data folder is fresh on every run, so the sheet always shows; nothing is stated in the world (design call 16). The
  pauses stay under 20 s. `overview.toml` cleans nothing and is unchanged.
- `app/ui/e2e/flows.spec.ts` — negative first: "Not now starts no cleanup" (not accepted, select, Move to Trash…,
  Not now: no confirmation, no `prepareCleanup` call, Review unchanged); "Escape on the agreement starts no cleanup";
  positive: "the first cleanup asks for the agreement and goes on after I understand" (the confirmation opens after
  it); "the agreement is not asked again once accepted"; "Settings shows the agreement and when it was accepted";
  "the full licence opens from the agreement and from About".
- `app/ui/e2e/apps.spec.ts` — negative: "Not now on an uninstall starts nothing" (not accepted, Uninstall, Not now:
  no `prepareUninstall` call, the page unchanged); positive: "an uninstall asks for the agreement first and goes on
  after I understand".
- `app/ui/e2e/visual.spec.ts`, `axe.spec.ts` — the sheet, the licence dialog, About (accepted and not); the new
  baselines in `app/ui/e2e/screenshots/`, looked at before committing.

## Tests (unit, negatives first)

- The reducer and the actions: "prepare asks for the agreement when it is not accepted"; "prepare loads the
  agreement when it is not loaded"; "prepare asks when an older version was accepted"; "an uninstall asks before it
  prepares"; "declining runs nothing"; "a refused acceptance keeps the sheet open with the new view"; positive:
  "accepting runs the held prepare"; "accepting runs the held uninstall with its app and choice"; "prepare goes
  straight on when accepted".
- `agreement-sheet.test.tsx`: negative — "shows no changes notice without changes"; "Escape declines"; positive —
  "shows the five lines in order"; "shows what changed when the core says"; "Not now has the first focus";
  "I understand accepts".
- `licence-dialog.test.tsx`: "shows both licences' texts".
- `settings-screen.test.tsx`: "About says the agreement is not accepted yet"; "About shows the acceptance date".

## Checks

- `npx vitest run` on the changed test files, then `npm run test:cov:ui` (90 % of lines); `npm run lint:ts`;
  `npm run lint:code`; `npm run test:e2e:ui` in WebKit and Chromium; axe clean; the gallery shows the sheet's stories.
- `npm run knip` (the new exports are used).
- The demo: `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-sandbox scenario` (every target traced, none
  permanent), `npx vitest run app/ui/src/demo`; `npm run demo:check -- cleanup` where the development app can start
  (it plays without recording and needs no permission); otherwise it goes to the user's walk list of 05/01.

## Conclusions

- The sheet (`product/agreement-sheet/`), the licence dialog (both licence texts bundled with `?raw`; the app opens no
  URL), the frame-level gate (`screens/agreement-gate.tsx`) and Settings, About (the five lines, the date, the licence
  button; "Not accepted yet" also when only an older version was accepted). The gate lives in
  `state/cleanup-actions.ts`: it asks the core before a prepare and opens the sheet; one acceptance per click (an
  in-flight guard added by the final fix wave); the sheet is not the guard.
- The mocks accept by default; `agreement: 'not-accepted'` (`e2e/support/agreement-mock.ts`) refuses a prepare until
  the acceptance, as the core does. No `changes` text is in a mock.
- The `cleanup` demo scenario clicks "I understand" after "Move to Trash…"; the world states nothing. The GIF shows
  the sheet for a moment. `npm run demo:check -- cleanup` was not run (it needs the development app on screen): it is
  on the user's list.
- The licence `<pre>` blocks do not scroll on their own; the dialog's body scrolls both.
