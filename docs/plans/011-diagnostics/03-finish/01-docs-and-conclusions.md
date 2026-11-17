# 011/03/01 — Docs, a run by hand, conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs,
repo

## Files

- `docs/architecture/README.md` — a short section "Problems and reasons": the three forms of a problem the user sees
  and their carriers, the rules of the shape (a text is the only thing shown; the sentence says what happened and
  what to do; a technical detail is the `detail` arg, shown on demand; an error's `Display` is for the log), and the
  table of every reason type with where it is made and where it is shown (`Coverage`, `NotScanned`,
  `RemainderReason`, `SkipReason`, `Failure`, `BlockedReason`, `ModuleOutcome`, `LeftOutCount`). In "UI": the lint of
  01/03.
- `docs/architecture/modules.md` — in "The `Module` trait in short": `ModuleError::Failed` with a declared problem,
  `out.left_out(…)` and what it is not for (an unreadable folder, a missing permission, a path no rule knows); in
  "Tests": the contract's two new problems; in "The checklist": two boxes (every failure and left-out text declared,
  plain and with its next step; an item recognized and not offered is reported, never dropped).
- `docs/testing.md` — "Sandboxes": a case file does not check left-outs yet; the first module that reports one adds
  the expectation to the case format (agent's call 6 of the design).
- `docs/plans/README.md` — the row of 011: ✔️ done, 0.1.0, once the user has looked at the result.
- This plan's readmes — the conclusions: what was built, what changed from the design, what was measured, what is
  left, and the agent's calls taken during the build.

## The run by hand (the user's; the agent writes down what happened)

- [ ] `npm run dev`, a scan: the demo module's category shows its left-out line ("1 item left out (150 MB): The app
      that made it is open. Quit the app, then scan again.").
- [ ] Quit and start again (the scan restored from the cache): no left-out line shows until the next scan (left-outs
      are event-only, design call 4).
- [ ] `FRESHER_SANDBOX=relative npm run dev`: the start-failure dialog shows the plain sentence and, under it, the
      sandbox's refusal; the main window is gone, no UI flashes; the app does not exit before the dialog is
      dismissed (Cmd-Q waits too); after dismissing it exits with code 1; nothing was written.
- [ ] A tool's failure in a sandbox is not reachable by hand yet; the UI e2e screenshot of a failed item with
      "Details" stands in.
- [ ] `npm run pre-push`, `npm run test:e2e` and `npm run sandbox:test` pass on the branch's last commit.

## Checks

- `npm run pre-push` passes on the branch's last commit, coverage at or above 90 %; `npm run test:e2e` and
  `npm run sandbox:test` pass, the sandbox goldens unchanged.

## Conclusions

Docs written as built: "Problems and reasons" in `docs/architecture/README.md` (the three forms, the rules, the table of
reason types, left-outs, the failed start) and the UI lint in its boundaries; the module API, contract and two checklist
boxes in `docs/architecture/modules.md`; the case-file note in `docs/testing.md`; row 011 of `docs/plans/README.md` (in
progress, 0.1.0). Corrections recorded: design call 4 stands, reworded in `00-design.md` (left-outs are event-only:
after a restart no left-out line shows until the next scan); the type is `LeftOutItem`; the lint is a tripwire.
`trial.error.failed` now says what to do next.

The run by hand and the final checks wait for the user. This step and plan 011 stay in progress until they are ticked.

### Agent's calls taken during the build (to check)

1. `trial.error.failed` got a next step ("Scan again; if it fails again, the tool is not available on this Mac."). Cost if wrong: a wording the user may prefer otherwise; the text is trial's own and dev-only.

