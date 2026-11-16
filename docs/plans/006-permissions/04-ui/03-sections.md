# 006/04/03 — Limited and unavailable sections

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- The overview's `CategoryCard` and the generic module page's `ModuleSection` (plan 003) render a category's
  `Availability`:
  - `Ready` — as today.
  - `Limited` — the size as `SizeText` partial (`42 GB+`, the core already marked it partial) and a tooltip "Some of it
    could not be seen: <capability name>."
  - `Unavailable` — the card stays in its place, disabled (not removed, not hidden), with the one sentence
    `core.access.<id>.missing` ("Docker is installed but not running. Start Docker and scan again.") and, for an OS
    permission, the same action as its notice.
- A module that ended `ModuleFinished { Unavailable }` shows the same sentence on its page instead of an empty list.
- Tests next to the components, as plan 003 lays them out.

## Produces

Rule "nothing is silently absent" of the design, section 8.

## Checks

- Negative (first): `an_unavailable_category_is_shown_disabled_not_hidden`, `an_unavailable_category_cannot_be_selected`,
  `a_ready_category_shows_no_access_text`, `an_unavailable_module_page_is_not_an_empty_list`.
- Positive: `a_limited_category_shows_a_partial_size_and_why`, `an_unavailable_category_names_what_is_missing`,
  the disabled card is reachable by keyboard and reads its reason to VoiceOver (`aria-disabled` with a description,
  not `disabled`, so focus can reach it).
- Gallery states and screenshots: each of the three availabilities on a card and on a module page.
