# 015/01/02 — Modules under their group

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/screens/use-modules.ts` — `modulesByGroup(modules)`: the groups that have a module, in the core's order
  (`development` before `debug`), each with its modules in registration order. Pure, unit-tested.
- `app/ui/src/screens/frame-sidebar.tsx` — one `NavSection` per group, titled `core.group.<group>.name`, instead of
  the single "Modules" section; the key `ui.screen.frame.modules` is removed.
- `app/ui/src/screens/overview/module-progress.ts` and `app/ui/src/product/scan-progress/` — the chips grouped, the
  group's name before its chips.
- `app/ui/src/screens/settings/settings-groups.ts` and `settings-screen.tsx` — the modules' settings under their
  group's name.
- `app/ui/src/locales/en.json`; the stories of the sidebar and the scan progress (`app/ui/src/gallery/`).

## Produces

The grouped sidebar 02/03 adds the ignored look to.

## Checks

- Vitest, negatives first: no section for a group without a module; an unknown group (a newer core) is shown last
  under its key, never dropped. Positive: Development before Debug; registration order inside a group; the chips and
  the settings grouped the same.
- `npx vitest run src/screens/` and the touched product folders; the screenshots of the sidebar updated and reviewed
  (`npm run test:e2e:update --workspace app/ui`).

## Conclusions

Built as written: `product/module-groups.ts` (`inGroups`, `modulesByGroup`: known order, an unknown group last), one
sidebar section per group, the scan's chips per group (a labelled list each), Settings' module sections under a group
heading. `moduleProgress` drops ids the core does not list. About 80 screenshots changed (the sidebar is on every
screen) and were looked at. vitest 956 passed, Playwright 233 passed.
