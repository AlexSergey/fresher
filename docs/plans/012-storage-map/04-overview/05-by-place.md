# 012/04/05 — By place

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/product/place-list/` (new) — `PlaceList`: the `PlaceView`'s path as a breadcrumb (each folder of the
  descent a step, "Library › Containers › com.example.app"; a step opens that folder, without descending again), then
  the rows in a `VirtualList` of `TreeRow`s: name, size (`honestSize`), a thin bar of the row's share of the folder,
  and "9 GB explained" when any of it is (the explained part of the thin bar in Iris). A folder row opens it; a file or
  link row opens nothing. A row whose folder was not read says why (the core's text of `NotScanned`). No checkbox,
  no cleanup: the user acts from the module pages (design, call 3). Stories: a home, a descent, a partial folder, an
  unreadable folder, an empty folder.
- `app/ui/src/screens/overview/overview-screen.tsx` — "By place" shows `PlaceList` from the volume's first root (a
  volume with several roots lists them first, as rows); the folder opened is kept in the navigation state, so leaving
  the overview and coming back keeps the place.
- `app/ui/src/state/navigation.ts` — the overview's view and the opened folder in its screen state.
- `app/ui/src/locales/en.json` — the chrome: "{size} explained", the breadcrumb's label, "Back to {folder}".
- `app/ui/src/gallery/` — `PlaceList` registered.

## Produces

The second view; the overview is complete.

## Checks

- Negative first: `place-list.test.tsx` — `a_row_has_no_checkbox`, `a_file_row_does_not_open`,
  `an_unreadable_folder_says_why`, `a_breadcrumb_step_does_not_descend_again`; `overview-screen.test.tsx` —
  `a_stale_place_reopens_the_root` (with 02's hook).
- Positive: `place-list.test.tsx` — `rows_come_largest_first`, `the_explained_share_is_drawn`,
  `the_descent_is_one_breadcrumb`; `overview-screen.test.tsx` — `switching_views_keeps_the_total`,
  `the_opened_folder_survives_leaving_the_overview`; keyboard: the rows and the breadcrumb are reachable and open with
  Enter (the existing focus rules of `screens/keep-focus.ts`).
- `npx vitest run src/product/place-list src/screens/overview src/state/navigation`; the UI e2e on the mocked core.

## Conclusions

Built in `504067d`, reviewed clean.

- `PlaceList` with a breadcrumb, `FolderRow` with its explained share, `RootList` for a volume with several roots.
- Deviations: `Descent` was added to the core, the engine and the command; the core's texts for `NotScanned` did not
  exist and were added. Bug fixed: pages after the first asked the node first requested, not the folder the descent
  settled on. Final wave: no crumbs while the shown node is not the asked one; the place is refused while a scan
  runs (the scanning map has no places).
- Parked: the one-render crumbs have no test (React's `act` never exposes it).
