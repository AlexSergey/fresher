# 005/04/02 — Finish

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** docs, repo, app/ui (e2e)

## Files

- `app/ui/e2e/` — the settings flows on the mocked core: change and reset a setting, add and remove a folder, ignore
  and restore, "Rescan to apply".
- `docs/architecture/README.md` — the settings: groups, effects, the ignore list; `docs/restrictions/deletion-safety.md`
  — the ignore list as a guard input; `docs/testing.md` — the new tests.
- `docs/ideas/{user-settings,ignore-list}.md` — link to this plan; `docs/known-issues/store-stale-temporary-files.md`
  and the deferred rows of the design's decisions.
- The version files: 0.5.5.
- The conclusions of every plan and of this global plan.

## Checks

`npm run pre-push` on the last commit; the UI e2e tests.
