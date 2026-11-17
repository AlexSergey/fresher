# 007/05/01 — The docs and the rule

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** docs

## Files

- `docs/rules.md` — rule 8 ("Tested to the bar") gains: every module and every category ships its sandbox case file,
  `fixtures/sandboxes/<feature>.toml`, negative cases first, with its golden report.
- `docs/plans/README.md` — "Format": a plan that adds or changes a module or a category lists its case file among its
  files, and its finish runs `npm run dev:sandbox -- <feature>` with `EXPECTED.md` by hand.
- `docs/testing.md` — "Sandboxes" completed (the case format, the generator, sandbox mode, the harness, the commands
  `sandbox`, `dev:sandbox`, `sandbox:test`); the "Kinds of tests" table gains the sandbox tests.
- `docs/architecture/README.md` and `docs/architecture/removal.md` — sandbox mode: what the composition changes,
  `Reach::SandboxFolder`, the joined protection.
- `docs/restrictions/deletion-safety.md` — "Enforced by" of the sandbox: the third reach.
- `docs/hacks/README.md` — the entries touched by 02/02, checked once more.
- `docs/deferred/dev-debug-panel.md` (new) and its row — the debug panel and the developer command line beyond the
  generator, with the reasons of the design's section 9; due when the deferred
  [world](../../../deferred/sandbox-world.md) is built.
- `docs/deferred/removal-sandbox-run.md` — "What doing it needs" points to `npm run dev:sandbox -- trial --real-trash`
  instead of the local script; the parts the autotests now hold are struck from it.

## Produces

The docs as built and the rule later plans are checked against.

## Checks

- Every link resolves; `npm run lint` (Prettier on the docs) passes.

## Conclusions

The docs name the sandbox, the case file rule and the new known issue (the scan offers an item in the Trash); a new
deferred item (the development debug panel) was added with its rows, and the generator's built signature replaced the
stale text of its step. Final review: `removal.md` names `npm run dev`.
