# 016/02/02 — The decision

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** modules

## Files

- `modules/jetbrains/src/decide.rs` (new) — pure: `decide(products, product, folder, installed, running) -> Verdict`,
  where `Verdict` is `Offer { current, app }` or `LeftOut(Reason)`; the rows of the design, section 4, in their order
  (eight since the unknown-app row of 2026-11-17). `installed` and `running` are the scan's lists; `Err` of either is
  row 1.
- `modules/jetbrains/src/places.rs` (new) — the three places, their known folders, categories and identities.
- `modules/jetbrains/src/lib.rs` — `on_match` takes a match whose `kind` is a folder and whose name parses and decides
  it at once with `cx.system()`'s lists, pushing a `NewFinding` (the design's table: category by place, identity,
  `MoveToTrash`, title and reason with `{product}`, `{version}`, `{current}`, `for_app`) or `out.left_out(…)`;
  `finish` has nothing left to do. Changed on 2026-11-17 from a list gathered for `finish` (the design's agent's call
  12: a cancelled scan would carry its folders into the next one).

## Produces

The module's behaviour, for 02/03's fixtures and every test after.

## Checks

- Unit (`decide.rs`), negatives first, one test per row: the lists failing; a running app (installed or not) of the
  product; no installed app; an installed app with an unknown version; the version in use; a newer folder; then an older
  folder offered; two installed versions side by side; the precedence (running over not installed and current); an
  early access bundle id counts as the product.
- Unit (`lib.rs`): a link and a file named like a version folder are not kept; a match of an unknown product is not
  kept; the same matches in two orders give the same findings and left-outs (`run_module`-free, on a recording sink).
- `cargo nextest run -p fresher-module-jetbrains`.

## Conclusions

Built (`decide.rs`, `places.rs`): a pure `decide`, eight rows in the order of the design's table. Deviations:
(1) each folder is decided in `on_match`, not gathered for `finish`, because `finish` is not called after a cancelled
walk and the module lives across scans (agent's call 12); (2) the Toolbox app is exempt from the "unknown vendor app is
open" row; (3) `Verdict::Offer` also carries the app. The early access ids (`-EAP`) were added for every product,
and `other-apps` for the Toolbox app. 64 tests at the time.
