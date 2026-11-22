# 034 — Browser and web-engine caches

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Browser caches are what a person expects first from a cleaner, and apps built on a web engine keep the same kind of
cache in their support folder. Plan [030](../030-installed-app-caches/readme.md) left both out because the folders are
named after the app, not its bundle id ([deferred](../../deferred/installed-app-caches-more-places.md)).

## Goals

1. The cache of each installed browser, and of each known web-engine app, is offered per app and per profile.
2. Only caches: nothing the user would call their data.
3. A folder is claimed only when a table ties it to an installed app.

## Non-goals

- History, cookies, saved passwords, sessions, extensions, site storage: privacy cleaning is not this product.
- Safari and other Apple apps (protected containers): they stay deferred.
- A generic search for cache-looking folders inside any app's folder: a rule that cannot tell does not claim.

## Design

The module `app-caches` gains a table and two categories; nothing else in it changes shape:

| Category                | Level | What                                                                              |
| ----------------------- | ----- | --------------------------------------------------------------------------------- |
| `app-caches.browsers`   | Low   | a browser's cache folders, a finding a profile                                    |
| `app-caches.web-engine` | Low   | the cache folders inside the support folder of a known app built on a web engine  |

- **The table** is rule data: a bundle id, the folder under `Library/Caches` or `Library/Application Support`, and the
  exact names inside it that are caches (the network cache, the code cache, the graphics cache). A name not in the
  table is never touched, so a profile's data beside its cache is out of reach by construction.
- Emptied, the folder stays (`Removal::EmptyFolder`), only while the app and its helpers are closed (`app-open`).
- First rows: the common browsers on a web engine with profiles, and the handful of chat and editor apps that are
  usually the largest; each row is checked on a real install before it is added. Contributors add rows.

**Sandbox.** `fixtures/sandboxes/app-caches-browsers.toml` and `app-caches-browsers-open.toml`, negatives first: the
folder with no installed app behind it, a profile's history and cookie files beside the cache, a cache-named folder of
an app that is not in the table, a link named like a cache, the browser open; then a browser's two profiles and one
web-engine app. Golden reports, core e2e flow, and the finish walks `EXPECTED.md` in sandbox mode.

## Plans (after approval)

| #   | Plan       | What                                                       |
| --- | ---------- | ---------------------------------------------------------- |
| 01  | Table      | the rows, the owner check, the two categories, the texts   |
| 02  | End to end | case files, golden reports, the core e2e flow, the UI      |
| 03  | Finish     | docs, the deferred item's row narrowed, the walk           |

Depends on plan 030.

## Ideas this touches

[Caches of installed apps](../../ideas/installed-app-caches.md).

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **Caches only, by exact name from a table**; never history, cookies or sessions.
2. **Known apps only**; no generic finder.
3. **Safari stays deferred.**
4. **Part of `app-caches`**, not a new module.

## Conclusions

Not written yet.
