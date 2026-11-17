# 015/02/02 — Settings: the three tabs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/design/layout/tabs/` (new) — `Tabs`: an ARIA `tablist` of `tab`s and one `tabpanel`, the selected tab
  the only tab stop, Left and Right (Home and End) move between tabs, a label may carry a count; its stories and test.
- `app/ui/src/screens/settings/ignored-group.tsx` — the Ignored group becomes the three tabs: Ignored groups, Ignored
  modules, Ignored items; it opens on the first tab with an entry, else on Ignored groups, or on the tab the overview's
  link names.
- `app/ui/src/screens/settings/ignored-groups.tsx` (new) — every group with a registered module: its name, its
  description, a `Switch` labelled "Ignore {group}"; on → `ignore({ kind: 'group', group })`, off →
  `restoreIgnored(…)`; an ignored row shows **Restore** too.
- `app/ui/src/screens/settings/ignored-modules.tsx` (new) — every module under its group's name, with the same switch
  and Restore; a module of an ignored group: the switch on and disabled, "Ignored with its group: {group}".
- `app/ui/src/screens/settings/ignored-items.tsx` (new) — plan 005's categories and paths, moved unchanged from
  `ignored-group.tsx`.
- `app/ui/src/screens/settings/settings-screen.tsx` — passes the modules to the group.
- `app/ui/src/locales/en.json` — the tabs' names, the switch's labels, the sentences of section 3, the note.
- `app/ui/src/gallery/` — the tabs' stories.

## Produces

The tabs 02/04 drives.

## Checks

- Vitest, negatives first: a read-only file disables every switch and Restore; a module of an ignored group cannot be
  switched; a group with no module is not listed; "Nothing ignored" in Ignored items when empty. Positive: each switch
  sends the right target; Restore sends the restore; the counts on the tabs; the tab chosen on open.
- `Tabs`: arrow keys, Home and End, one tab stop, the panel labelled by its tab.
- `npx vitest run src/screens/settings/ src/design/layout/tabs/`; the gallery's screenshots and `axe.spec.ts` on the
  new stories.

## Conclusions

Built as written: the design `Tabs` (`design/layout/tabs/`: tablist, one tab stop, Left/Right wrapping, Home/End,
panel labelled by its tab, "Label · N" when the count is above zero) with stories, and the Settings files
`ignored-group.tsx`, `ignored-groups.tsx`, `ignored-modules.tsx`, `ignored-items.tsx`, `ignore-row.tsx`. A module or a
group no longer registered is still listed with Restore. vitest 976 passed; the gallery, WebKit and Chromium 233
passed. The new tabs were not in the existing screenshots (below the clip) until 02/04.
