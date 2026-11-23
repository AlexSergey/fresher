# 039/04/03 — The interface's texts and the hints

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** app/ui, scripts, docs

A pass over the interface's dictionary, screen by screen, against `docs/writing.md` (written by 04/01); the hints
every control, number and icon lacks (rule 7); then the check joins pre-push. A procedure over the screens as they
stand when this step runs. The design: [`../00-design.md`](../00-design.md), section 4.

## What may and may not change

The rules of 04/02, "What may and may not change", hold here too: never a key, a placeholder, a plural pair's shape,
or what a text promises; a text whose meaning looks wrong is listed, not rewritten in passing.

## The procedure

1. Run `npm run check:texts` and keep the interface's part of its output.
2. **Screen by screen**, in the sidebar's order and then the dialogs: the frame and the sidebar; Scan (hero, progress,
   zones, cards, the danger-zone question); a category's Details; the storage map; Review (summary, parts, groups);
   the confirmation (titles, how, left out, the High and permanent ticks); the cleanup's progress and result, the help
   dialog; Cleanups and a cleanup; Settings (every group, About, the agreement sheet, the licence dialog); the
   permission card; every other screen the sidebar lists then — on 740cf3c3: Largest, Files and Folders (plan 035), the
   Apps list and an app's page with its uninstall choices, not-offered reasons and App Management notice (plan 036),
   the Trash line on Scan and in the result (plan 037), an insight screen and the "Described, not cleaned" section
   (plan 032), the demo player's refusal (plan 038, development only). For each: list the
   `ui.*` keys it renders (grep the screen's folder for `t('ui.`), read each against its shape, fix every check
   violation. Listed, not rewritten in passing: `ui.confirm.how.tool` names Docker as a tool that removes, though
   Fresher removes nothing of Docker's since plan 032 (design call 19).
3. **Hints.** On each screen, list every control (button, icon button, switch, checkbox, select, segmented control),
   every number and every icon whose meaning is not written on screen next to it. Each one gets a description: the
   primitive's `description` prop where it has one (`Checkbox`, `Switch`, `DangerBadge`), otherwise the `Tooltip`
   primitive (`design/primitives/tooltip/`) wired with `aria-describedby`, so it is reachable by keyboard focus and
   read by VoiceOver. An icon button always has an accessible name and a tooltip. A hint is one sentence in the guide's
   "hint" shape, a new key `ui.<component>.<control>.hint` (or `.description` where the primitive's prop is named so).
   Keep a table in the conclusions: screen, control, the hint's key.
4. Unit tests for each hint added: "the <control> has a description" (by its accessible description, not by a class).
5. The screenshot specs: `npm run test:e2e:ui -- --update-snapshots`, every changed image looked at; the axe spec
   passes (no hint may break a landmark or a name).
6. The demo scenarios press `ui.*` texts (on 740cf3c3: "Scan", "Review", "Move to Trash…", "Move to Trash",
   "I understand", "Cleanups", "Cleanup of …", "Put back all still in the Trash", "Select …"): a changed one is changed
   in `fixtures/demos/scenarios/*.toml` too, with `cargo nextest run -p fresher-sandbox scenario` passing, and
   `npm run demo:check -- all` where the development app can start.
7. When `npm run check:texts` reports nothing for **every** dictionary: add it to `package.json`'s `pre-push`, after
   `check:crate-deps` (`... && npm run check:crate-deps && npm run check:texts && npm run knip && ...`), and to
   `docs/quality.md`'s pre-push list; the guards' table drops "joins pre-push in 04/03".

## Files

- `app/ui/src/locales/en.json` — the texts and the new hints.
- The components that gain a hint, with their tests; stories where a story shows the control.
- Tests asserting a literal `ui.*` text — updated to the new text.
- `app/ui/e2e/screenshots/` — the changed baselines.
- `fixtures/demos/scenarios/*.toml` — a target whose text changed.
- `package.json`, `docs/quality.md` — the check in pre-push.

## Checks

- `npm run check:texts` reports nothing; `npm run pre-push` passes (it now runs the check); `npm run test:e2e:ui` in
  WebKit and Chromium, axe clean, every changed baseline looked at; `npm run test:cov:ui` at or above 90 %.
- Keyboard: every new tooltip shows on focus (`app/ui/e2e/keyboard.spec.ts` gains one case for one hint per kind of
  control: icon button, number, badge).

## Conclusions

As built: the hints and `check:texts` in `pre-push` (commits 65ceebc1 and 7b33843f), after a fix round for the hints.

- **Check counts:** 15 findings before (all `app/ui`), 0 after, for every dictionary. No interface text was rewritten.
  Two rule tweaks, with tests: a `{placeholder}` name is not read as a lower-case "trash" (`ui.build-banner.sandbox*`),
  and the category-parts rule skips `ui.` keys (they label a screen, they are not a category). The sentence rule still
  applies to `ui.` keys; the nine headings and labels that fired are `EXEMPT` in `scripts/check-texts.ts`, one entry each
  with its reason (`ui.confirm.left-out`, `ui.finding-list.failed.title`, `ui.scan-hero.failed`,
  `ui.scan-progress.module.failed`, `ui.screen.category.{what,made-by,if-removed,comes-back}`,
  `ui.screen.scan.left-out.title`). Tests pin the scoping: an unterminated text under a non-`ui` sentence key and under a
  non-exempt `ui` key are both reported; lower-case "trash" is reported next to a placeholder.
- **Interface texts changed:** 0. Every `ui.*` text was read against the guide, screen by screen; none broke the voice,
  the shapes or the glossary.

| Screen or dialog | Key prefixes read | Changed |
| --- | --- | --- |
| Frame, sidebar, toasts, quit question | `ui.app`, `ui.screen.frame`, `ui.quit`, `ui.core`, `ui.build-banner` | none |
| Scan (hero, progress, zones, cards, danger-zone question, ignored, described, left out) | `ui.scan-hero`, `ui.scan-progress`, `ui.danger-zone`, `ui.category-card`, `ui.screen.scan`, `ui.storage-map`, `ui.coverage`, `ui.access`, `ui.trash` | none |
| A category's Details and an insight | `ui.screen.category`, `ui.finding-list`, `ui.left-out-lines`, `ui.insight`, `ui.insight-card`, `ui.size` | none |
| Review, selection bar | `ui.screen.review`, `ui.selection-bar`, `ui.size` | none |
| Confirmation, progress, result, help | `ui.confirm`, `ui.cleanup`, `ui.result`, `ui.recovery`, `ui.problem` | none (Docker line listed below) |
| Cleanups and a cleanup | `ui.screen.cleanups`, `ui.cleanup-row`, `ui.own-item-row` | none (one listed) |
| Largest, Files and Folders | `ui.screen.largest`, `ui.own-item-row` | none |
| Apps and an app's page | `ui.screen.apps`, `ui.screen.app`, `ui.product.*` | none |
| Settings, About, agreement sheet, licence | `ui.screen.settings`, `ui.settings-row`, `ui.settings`, `ui.agreement`, `ui.licence`, `ui.permission`, `ui.permissions` | none |

- **Hints added** (existing hints stay: icon buttons show their label, a path its full text, a partial size its reason,
  a finding's level its reason). Each hint is a focusable anchor or a described control: a tooltip on focus and hover,
  `aria-describedby` for assistive technology; `Select` and `SegmentedControl` gained an optional `description` prop.

| Screen | Control | Hint |
| --- | --- | --- |
| Scan, category card | the danger level badge | `core.danger.<level>.meaning` (the core's text) |
| Scan, hero ring | the found size in the centre | `ui.scan-hero.found.hint` |
| Selection bar | the total | `ui.selection-bar.total.hint` |
| Selection bar | the highest level badge | `core.danger.<level>.meaning` |
| Largest | the Kind, Place and Not changed for selects | `ui.screen.largest.filter.{kind,place,unchanged}.hint` |
| Apps | the "Not used for" segmented control | `ui.screen.apps.filter.hint` |

  Unit tests check each by its accessible description; the keyboard spec has three cases (icon button, badge, numbers;
  a Largest filter; the Apps filter).
- **Not given a hint:** the sidebar's Review count (it sits inside the Review button, a nested focus stop is invalid;
  its accessible name already says "N items selected"); the storage map's part sizes and the ring legend (each has its
  name beside it and its own explanation that opens from the "i"); the Largest "Views" tabs (named by their words).
- **Cost to know:** every category card now has one more tab stop (its level badge); the selection bar two, the hero one.
- **Baselines:** the three focus baselines that changed (CategoryCard, CategoryGrid, SelectionBar) exist for WebKit only
  (the sweep keeps its focus photographs for WebKit); looked at. No other baseline changed.
- **Listed, not rewritten:** `ui.confirm.how.tool` names Docker as a tool that removes (design call 19);
  `ui.screen.cleanups.failed-action` ("That did not work") is vaguer than the refusal shape asks for.
  No demo scenario target text changed.
- **Pre-push:** `npm run check:texts` joined `package.json`'s `pre-push` and `docs/quality.md` in the commit below.
