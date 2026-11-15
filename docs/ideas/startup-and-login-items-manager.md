# Startup & Login Items Manager

**Since:** 2026-11-15

## Problem

Login items, launch agents and launch daemons start silently, slow the Mac down and keep running after their app is
gone.

## Direction

- List login items, user launch agents (`~/Library/LaunchAgents`), system launch agents and daemons
  (`/Library/LaunchAgents`, `/Library/LaunchDaemons`), background items, each with the app it belongs to and whether
  that app still exists.
- Disable (reversible) before remove; an item of an uninstalled app is pointed out as a leftover
  ([`uninstalled-app-leftovers.md`](./uninstalled-app-leftovers.md)).

## Conflict with the goal

[`../goal.md`](../goal.md) lists "startup item managers" among the **non-goals**. Taking this idea up needs the user to
change that line first. A narrower version fits the goal as it is: only **orphaned** launch agents and login items of
uninstalled apps, as part of the leftovers finder.

Open questions:

- Full manager (change the goal) or only orphaned items (fits today)?
