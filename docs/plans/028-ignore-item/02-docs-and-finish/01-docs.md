# 028/02/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs

Every doc that says the Ignore of one item is missing, or describes the finding list's keyboard model, says what the
UI is now. The design: [`../00-design.md`](../00-design.md), "Docs to update".

## Files

- `docs/deferred/ignore-an-item-button.md` — deleted (`git rm`); its row leaves `docs/deferred/README.md`.
- `docs/ideas/ignore-list.md` — a dated line under plan 015's: "**Taken by plan [028](../plans/028-ignore-item/readme.md)**
  (2026-11-21): a single item is ignored from its row's settings menu on a category's details, and restored in
  Settings → Ignored → Ignored items." Its two links to the deleted file point to plan 028 instead (the user's text of
  the idea below them is unchanged).
- `docs/architecture/README.md` —
  - the Scan and details bullet: a finding row on a category's details has a settings menu ("Ignore this item"),
    Review's rows have none;
  - the lists bullet: the finding pages start again on an ignore change (`CoreState.ignoreChanges`) as on a
    cleanup's removal;
  - the design layer's virtual list: one tab stop on the rows, the focused row's controls after it, keys on a control
    inside a row are the control's; `Menu.tabIndex`.
- `docs/testing.md` — the ignore mock answers `ignore` with a finding and `restore_ignored` with a path, and
  `get_findings` leaves ignored paths out (the category totals are not recomputed); the Scan screen's flows name the
  row's menu (Escape, Ignore, Restore, by keyboard alone, none on Review).
- `docs/manual-testing.md` — section 1.1 (the trial sandbox) gains one item: on a category's details, ignore one
  item from its row's menu (by pointer, then by keyboard); the row leaves and the focus stays in the list; Settings →
  Ignored → Ignored items lists the path; Restore brings the row back without a rescan; a cleanup prepared after it
  does not list the item.

## Produces

Docs that match the code; no deferred row left for this item.

## Checks

- `grep -rn "ignore-an-item-button" docs | grep -v "^docs/plans/"` shows nothing (the plans keep their links as
  history, plan 027's precedent).
- `npx prettier --check` on the changed docs; every new relative link opened by hand.
- `npm run pre-commit` passes on the commit.

## Conclusions

Done in 5d771d65 and 40c669ac: the deferred note is deleted with its row, the architecture, testing and manual-testing
docs and the idea file say what the UI is now. Plans 005 and 015 keep their dead links (history).
