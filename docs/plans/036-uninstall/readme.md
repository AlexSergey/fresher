# 036 — Uninstall an app

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, system, engine, cleaner, testing, sandbox, modules, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Removing an app together with what it left is the feature nearly every tool of the field has. Fresher finds the
leftovers only after the app is gone (plan [019](../019-uninstalled-app-leftovers/readme.md)); the matching exists,
the "before" side does not.

## Goals

1. The user picks an installed app and sees everything that belongs to it, with sizes, before anything happens.
2. Personal data is kept unless the user chooses otherwise.
3. An app another manager installed is removed that manager's way, or not offered.

## Non-goals

- Apple's apps and system components; apps of other users.
- Login items and launch agents as a managed list ([`goal.md`](../../goal.md), non-goals); an app's own agent files
  are shown among its files only.
- Noticing an app dropped into the Trash: it needs background work.

## Design

- A sidebar entry **Apps**: installed apps with their real size — the bundle plus the data the existing modules tie to
  its bundle id (`app-caches`, `app-leftovers`' matching, the storage map's app lines) — as one number with a
  breakdown (bundle, support files, caches, logs), largest first, and the last time used. A filter "not opened for"
  makes the most useful sentence the app can say: "you have not opened this in 8 months and it takes 14 GB".
- An app's page links to its cleanable categories, so cleaning its cache is one click away from uninstalling it.
- An app whose footprint grows between scans is named by watch mode: plan [010](../010-history/readme.md)'s, on
  this plan's numbers.
- **Uninstall…** on an app opens the normal confirmation with two choices, each with its sentence (rule 7):
  - "Remove the app, keep its settings and data" — the bundle, caches, logs, saved state (the default);
  - "Remove the app and everything it left" — also settings (Medium) and the support folder (High), listed item by
    item.
- The same proof as plan 019, run before removal: bundle id, helpers' ids, the same refusals (files another installed
  app shares, links). A group container is offered only when the team identifier ties it to this app alone. The
  app's own launch agents and helper tools are listed among its files.
- Moving another developer's bundle needs the system's App Management permission: asked for at the first uninstall,
  with the reason, never earlier (plan [006](../006-permissions/readme.md) left it to this plan).
- **A fact to check in the design**: the last-used date — Spotlight's metadata or the launch database — whichever
  answers without an extra permission.
- The app and its helpers must be closed; the app is never quit for the user (`running`, left out with the reason).
- An app from the App Store is removed like any other bundle. An app a package manager installed is shown with "installed by
  `<manager>`; uninstall it there" and not offered; running the manager's own uninstall command is a later step. A bundle the user cannot move without administrator rights is not
  offered, with the reason.
- Everything goes to the Trash; the cleanup journal puts the whole app back.

**Sandbox.** `fixtures/sandboxes/uninstall.toml` and `uninstall-open.toml`, built on `[[case.app]]`, negatives first:
an Apple app, an app under `System/Applications`, a package manager's app (an install receipt in `world.toml`), an
open app, a second app sharing a support folder, a look-alike id; then an app with caches, settings and data under
each choice. Golden reports hold both choices; core e2e covers put back; the finish walks `EXPECTED.md` in sandbox
mode, then one real app on the developer's Mac.

## Plans (after approval)

| #   | Plan       | What                                                               |
| --- | ---------- | ------------------------------------------------------------------ |
| 01  | Footprint  | an installed app's files and sizes from the existing matching      |
| 02  | Removal    | the two choices as a confirmed plan, the refusals, put back        |
| 03  | Screen     | the Apps list, the app's page, the confirmation                    |
| 04  | End to end | case files, golden reports, e2e                                    |
| 05  | Finish     | docs, the walk                                                     |

Depends on plans 008, 009, 019, 027 and 030.

## Ideas this touches

This plan replaces the ideas "Smart Uninstaller" and "Application Footprint" (2026-11-15, the user's); everything
they held is above. [App Resource Profile](../../ideas/app-resource-profile.md) would extend the Apps list.

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **Keeping settings and data is the default choice.**
2. **Package-managed apps are shown, not removed** in the first version.
3. **No drag and drop of an app onto the window** in the first version, and no Trash watcher at all.
4. **The Apps list shows size and last used only**; no resource profile.

## Conclusions

Not written yet.
