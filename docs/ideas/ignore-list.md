# Ignore list

**Since:** 2026-11-15

**Taken by plan [005](../plans/005-settings/readme.md)** (2026-11-16): a category is ignored by its id and an item by its
path; ignored things leave every answer and are never removed; the settings list them with Restore; no expiry, and an
ignore holds while the path is the same. The Ignore button on a single item is
[deferred](../deferred/ignore-an-item-button.md).

**Taken by plan [015](../plans/015-module-groups/readme.md)** (2026-11-17): a whole module and a whole group of
modules can be ignored, from the module's page and from Settings (three tabs: groups, modules, items); an ignored
module stays in the sidebar, dimmed, and is restored without a rescan. The Ignore button on a single item stays
[deferred](../deferred/ignore-an-item-button.md).

## Problem

Some suggestions are right in general and wrong for a particular person. Example: the Docker section offers to remove
old and inactive images, but the user keeps them on purpose. Without a way to say "skip this", the same suggestion comes
back on every scan and becomes noise.

## Direction

- An **Ignore** button on a section (for example the whole Docker section) and on a single item (one image, one
  project).
- The choice is remembered in the settings ([`user-settings.md`](./user-settings.md)); the next scans skip it.
- Ignored things are not silently lost: the settings screen lists everything ignored, with a "restore" button that
  brings it back into the results.
- An ignored section or item is skipped entirely: not suggested for cleanup and not tracked in watch mode (the user,
  2026-11-15: "if we ignore it, we do not watch it"). Restoring it starts tracking again from the next scan; no growth
  is shown for the ignored period.
- The main screen shows a quiet note like "2 sections ignored", so the user knows the totals exclude them.

Open questions:

- Ignore by path, by rule, or both? Proposed: a section ignores a rule; an item ignores a stable identity (a path for
  files, an image ID or name:tag for Docker).
- When an ignored item changes (a Docker image rebuilt under the same tag), does the ignore still apply?
- An expiry for an ignore ("ignore for 3 months")?
