# 044/03/05 — Browse is read-only

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, engine, e2e, app/ui

The tree stays as Browse, but nothing on it can be ticked: its rows lose their picks and gain Show in Finder. Only
Files' rows stay removable. The design: [`../00-design.md`](../00-design.md), section 7; agent's calls 14, 20 and 21.

## Read first

03/04's commit; `crates/core/src/largest/page.rs` (`OwnRow`); `crates/engine/src/largest/{place.rs,
largest_tests.rs}`, `crates/engine/src/own_tests.rs`; `crates/e2e/tests/{largest.rs,largest_home/mod.rs}`;
`app/ui/src/product/own-item-row/`, `app/ui/src/screens/largest/{browse-view.tsx,use-own-pick.ts}`;
`app/ui/e2e/support/largest-mock.ts`, `app/ui/e2e/{flows,visual,axe,keyboard,core-mock}.spec.ts` (Browse's parts).

## Files

- `crates/core/src/largest/page.rs` — `OwnRow.picked` goes; `OwnRow`'s doc: one row of Browse, read-only; `offer`
  says why a row is not on Files or does not open, `opens` whether it opens. `OwnPlace` unchanged.
- `crates/engine/src/largest/place.rs` — `own_place` no longer asks `Ground::picked`; the module doc: Browse,
  read-only. `Engine::largest_place`, `get_largest_place` and `pick_own_item` keep their signatures; `pickable`
  still refuses a plain folder (02/03), so no Browse row is pickable through a crafted call either unless Files
  offers it.
- `crates/engine/src/largest/largest_tests.rs`, `crates/engine/src/own_tests.rs`, `crates/e2e/tests/largest.rs`,
  `crates/e2e/tests/largest_home/mod.rs` — the tests that read `picked` on a Browse row read it on Files; the golden
  `fixtures/golden/largest-flow.json` rewritten with `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test
  largest` if it changed, the diff read first.
- Bindings regenerated (`OwnRow` without `picked`).
- `app/ui/src/product/own-item-row/own-item-row.tsx` — a Browse row (`OwnRow`) has no checkbox and no empty
  checkbox place; it shows the reason of a row that does not open as its note, keeps Open on a folder that opens and
  gains `onReveal` ("Show in Finder", `ui.folder-row.reveal-label` as its name). A Files row (`LargestRow`) is
  unchanged. Its test and stories follow.
- `app/ui/src/screens/largest/browse-view.tsx` — no `pick`; `onReveal` → `revealNode(row.node.id)`, a refusal
  dispatched as `command-failed`; `largest-screen.tsx` no longer passes `pick` to it.
- `app/ui/e2e/support/largest-mock.ts` — `folderRowOf`, `fileRowOf` without `picked`; `reveal_node` also answers a
  node of the tree. `app/ui/e2e/core-mock.spec.ts` follows.
- `app/ui/e2e/flows.spec.ts` — the flows that ticked a row of the tree (:1023's checkbox count, :3262's) become
  "Browse has no checkbox" and "Show in Finder on a Browse row reveals it"; `visual.spec.ts` re-makes
  `largest-folders-{chromium,webkit}.png` (no checkbox place); `axe.spec.ts`, `keyboard.spec.ts` follow.

## Interfaces produced

- A read-only Browse: `OwnRow` without `picked`; `OwnItemRow`'s `onReveal`.

## Tests (negatives first)

- `crates/engine/src/largest/largest_tests.rs`: negative `a_browse_row_carries_no_pick` (the JSON of an `OwnRow` has
  no `picked`); the opening and withheld tests unchanged.
- `own-item-row.test.tsx`, negative: `a Browse row has no checkbox`, `a Browse row has no empty checkbox place`;
  positive: `a Browse row shows why it does not open`, `a Browse row calls onReveal from Show in Finder`, `a Files
  row still has its checkbox`.
- `browse-view.test.tsx`, negative: `reports a refused reveal`; positive: `reveals a row by its id`, `opens a folder`.
- e2e: as in "Files".

## Checks

- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-app -p fresher-e2e`, `npm run sandbox:test`
- `npx vitest run src/product/own-item-row src/screens/largest` (from `app/ui`)
- `npx playwright test --project gallery --project webkit --project chromium e2e/flows.spec.ts e2e/visual.spec.ts
  e2e/axe.spec.ts e2e/keyboard.spec.ts e2e/core-mock.spec.ts`
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `npm run lint:ts`,
  `npm run lint:code`, `npm run knip`, `npm run check:texts`

Commit: `feat(ui): Largest's Browse is read-only, with Show in Finder`.
