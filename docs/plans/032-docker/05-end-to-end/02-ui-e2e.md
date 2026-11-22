# 032/05/02 — The UI e2e

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui, e2e

Read first: [`../00-design.md`](../00-design.md) section 7, `docs/testing.md` (the UI e2e, visual baselines, axe),
`app/ui/e2e/core-mock.ts` (how a scenario's events are stated; a mock states only what the core can produce together),
`app/ui/e2e/flows.spec.ts`, `visual.spec.ts`, `axe.spec.ts`, and 03/01's components. 01/01 is built (the event's
`insights`); 02/02 is the source of truth for what the module sends (copy the shape from
`fixtures/golden/docker.json`).

## Files

- `app/ui/e2e/core-mock.ts` — a scenario `docker` (the name the mock's other scenarios follow): a scan whose
  `module-finished` for module `docker` carries the `orbstack` insight of `fixtures/golden/docker.json` (two figures,
  four parts, three commands) and, in a second scenario `docker-stopped`, the `stopped` insight of Docker Desktop (two
  figures, no part), beside findings of another module so the zones are not empty. The `docker` module listed in
  `get_modules` with no category and its `docker.enabled` texts in the dictionary.
- `app/ui/e2e/flows.spec.ts` — the flows below, in both browser projects.
- `app/ui/e2e/visual.spec.ts` — baselines: the scan screen with the section, the insight screen (answered), the
  insight screen (stopped). The baselines committed and looked at.
- `app/ui/e2e/axe.spec.ts` — the insight screen and the scan screen with the section have no violation.
- `app/ui/e2e/screenshots/` — the new baselines.

## Tests (negative first)

`flows.spec.ts`, `describe('negative cases')`:

- `the described section has no checkbox and adds nothing to the selection` — select-all in every zone, the review's
  count does not change with the section present.
- `the insight screen has no selection bar and Review never lists Docker` — open Details, then Review: no item of
  module `docker`.
- `a part without a command has no Copy button` (containers at 0 B).
- `the section is absent before a scan and while scanning`.

`describe('positive cases')`:

- `the scan screen shows the described section with the card's figures`.
- `Details opens the insight screen with its parts, levels and commands`.
- `Copy puts the command line on the clipboard and says Copied` — grant the clipboard permission in chromium
  (`context.grantPermissions(['clipboard-read', 'clipboard-write'])`) and read it back; in WebKit, where reading the
  clipboard is not granted, assert the "Copied." or "Could not copy…" status and the selectable line instead.
- `Back returns to the scan screen`.
- `a stopped runtime's card says to start it and its screen lists no part`.

## Done when

- `npm run test:e2e -- --project chromium --project webkit` for the touched specs; the visual baselines regenerated
  only for the new screens and looked at; `npm run lint`.
- One commit: `test(ui): the described section, the insight screen and Copy, end to end`.

## Conclusions

Not written yet.
