# 012/04 — The overview screen

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app,
app/ui

The commands, the UI's state on the overview's refresh beat, the bar with its neutral tones, and both views on the
overview screen. The design: [`../00-design.md`](../00-design.md), sections 4 and 6; the visual language: plan
[003](../../003-base-ui/00-design.md), "Graphite and Iris".

## Steps

| #   | Step                                          | Status                                   | Shared files                                                                                                           |
| --- | --------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| 01  | [The commands](./01-commands.md)              | ✔️ done | `app/tauri/src/lib.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/{index,mock}.ts`, `bindings.generated.ts` |
| 02  | [The state](./02-state.md)                    | ✔️ done | `app/ui/src/state/{refresh,store,reduce}.ts`                                                                           |
| 03  | [Neutral tones](./03-tones.md)                | ✔️ done | `app/ui/src/design/data/{segment.ts,usage-bar/,storage-ring/,chart-stories.ts}`, `app/ui/src/product/scan-hero/`       |
| 04  | [The bar and by meaning](./04-by-meaning.md)  | ✔️ done | `app/ui/src/screens/overview/`, `app/ui/src/state/navigation.ts`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/` |
| 05  | [By place](./05-by-place.md)                  | ✔️ done | `app/ui/src/screens/overview/`, `app/ui/src/state/navigation.ts`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/` |

03 from the start (the design folder only); 01 after 03/02; then 02, 04, 05 in order (04 and 05 both edit the overview
screen).

## Conclusions

Built on `feat/012-storage-map` on 2026-11-17, task by task, each commit through the hooks and reviewed; a whole-branch
review led to one fix wave and a re-review.

- **What is there now**: the three commands, the state, the neutral tones, the bar, the view by meaning with its cards
  and the view by place with its breadcrumb.
- **Changed from the design while building**: the cards are the core's category groups, one for every counted
  category, with "more under apps" when part of it sits under apps (final review); only the home volume gets the
  by-meaning list; the views refetch on the content of the volume's map, not on the phase alone; `Descent` and the
  `NotScanned` texts were added to the core (04/05).
- **Measured**: `npm run pre-push` passes (Rust 1731 passed, 5 skipped, lines at or above 90 %; UI 926 tests, lines
  97.83 %); `npm run test:e2e` 223 passed.
