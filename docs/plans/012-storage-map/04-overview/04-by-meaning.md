# 012/04/04 — The bar and by meaning

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/product/storage-map/` (new) — `StorageMap` (product: the core's types by props, texts through `i18n`):
  the volume's name and kind ("Macintosh HD · internal"), the `UsageBar` of its parts with the legend, and under it
  one line per non-zero part: name, size (`honestSize`: `42 GB+` when partial), and on demand (a disclosure) its
  description, "freed if removed" for `Explained` (`reclaimable`), the remainder's reasons as plain lines (the
  unreadable count with the permission's name and the permission card's action when plan 006 offers one), the
  purgeable note beside Free, and `core.storage.changed` when `diskChanged`. The tones: `Explained` by the highest
  danger level of its categories, `YourFolders` `plain`, the system's parts `system`, the remainder and not scanned yet
  `unknown`, Free `free`. During a scan the part sizes animate with `AnimatedNumber` (the motion hooks of plan 003).
  `storage-map.stories.tsx` and `story-data.ts`: no scan, scanning, complete, outside scan, not measured, without
  volume facts, two volumes.
- `app/ui/src/product/meaning-list/` (new) — `MeaningList`: the groups of `MeaningGroup[]` in their order — an app
  row (name, size, "not installed" when so; opens to its categories and data folders, each a `ListRow`), then the
  explained categories as the existing `CategoryGrid` (its selection and its opening of a module page unchanged), then
  the places (the Trash with its sentence, the largest folders; a row opens the place view at that folder), then the
  system lines. No row outside the categories offers a cleanup. Stories and tests.
- `app/ui/src/screens/overview/overview-screen.tsx` — under the scan hero: `StorageMap` for each volume (the home's
  first; a second volume collapsed under its name), then a two-option switch "By meaning · By place" (the design's
  segmented primitive if one exists, else two `Button`s with `aria-pressed`), then `MeaningList`. The `CategoryGrid`
  moves into `MeaningList`; the hero, the notices, the permission card and the ignored note stay as they are. The
  switch's choice is kept per session in the navigation state, not in settings.
- `app/ui/src/locales/en.json` — the chrome only: "By meaning", "By place", "Show details", the volume's kind
  separator; every part's name and description comes from the core.
- `app/ui/src/gallery/` — the two new components registered.

**Built:** only the home volume (the first) gets the list by meaning; another volume shows its map only, its
Explained in a neutral tone. Once the core answers, the cards are the core's `category` groups with the core's sizes,
the overview's totals giving each its count and danger level; before an answer (and during a scan, whose answer holds
only the volume's lines) the cards are the overview's totals. A failed answer is shown as a notice in the list.
Once the core answers, every counted category the core lists for the volume has a card, as a category or under an
app: the card's size is the core's share outside the apps (so the leaves add up), and when some of the category sits
under apps the card says "{size} more under apps", since its count and its selection take the whole category. The
views (by meaning, by place, the roots) are asked again whenever the volume's map changes, not only its phase: an
ignore or a restore changes the parts in the same state, and a scan's progress moves the bar.

## Produces

The overview's map and its first view; 05 adds the second.

## Checks

- Negative first: `storage-map.test.tsx` — `a_zero_part_has_no_line`, `no_part_but_explained_offers_a_cleanup`,
  `the_remainder_lists_its_reasons_not_a_size_per_reason`, `no_text_says_other`;
  `meaning-list.test.tsx` — `a_place_row_has_no_checkbox`, `an_uninstalled_app_says_so`;
  `overview-screen.test.tsx` — `before_a_scan_the_map_shows_the_volume_only`.
- Positive: `storage-map.test.tsx` — `the_parts_add_up_to_the_bar` (the widths' sum is 100 %),
  `a_partial_part_shows_a_plus`, `freed_if_removed_is_on_demand`, `purgeable_is_beside_free`;
  `meaning-list.test.tsx` — `apps_come_first_then_categories_then_places`, `a_category_row_opens_its_module`,
  `a_place_row_opens_the_place_view`; `overview-screen.test.tsx` — `the_bar_fills_while_scanning` (two store updates),
  `two_volumes_show_two_bars_home_first`; axe on the overview (the existing accessibility test).
- `npx vitest run src/product/storage-map src/product/meaning-list src/screens/overview`; the UI e2e of the overview
  on the mocked core (`npm run test:e2e`), screenshots updated on purpose.

## Conclusions

Built in `4e9150e`, fixed in `a5df0dc`, and again in the final wave `4bb8531`.

- The storage bar with a line per non-zero part and its reasons; the by-meaning list (apps, cards, places, the
  volume's lines); the "By meaning | By place" control; the map is asked once with the first overview.
- Review changed it: cards come from the core's category groups, the totals give only count and danger; a failed
  meaning answer shows a notice. Only the home volume gets the by-meaning list; other volumes show their bar.
- Final review: a card exists for every counted category, with the core's non-app share and, when part of it sits
  under apps, a line "{size} more under apps", since a selection acts on the whole category.
- Measured: e2e 217 at the step, 223 at the end; UI 926 tests.
