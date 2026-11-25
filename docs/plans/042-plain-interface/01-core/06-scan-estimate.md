# 042/01/06 — The scan's estimate

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, engine, app/ui (mocks)

The scan's progress event says how much the scan will most likely walk, so the scanning card can fill its swoosh. The
design: [`../00-design.md`](../00-design.md), section 5; agent's call 13. Mockup:
[`scanning.dc.html`](../../../mockups/0.1.0/scanning.dc.html).

## Read first

`crates/core/src/event.rs` (`Event::ScanProgress`), `crates/engine/src/session.rs` (where it is published, line ~99,
and what `progress.bytes` counts: allocated or logical), `crates/engine/src/results.rs` (`volumes()`, `state()`,
`roots`), `crates/engine/src/storage/roots.rs` (`RootVolumes`), `app/ui/src/state/reduce.ts` (how the event is kept),
`app/ui/e2e/core-mock.ts` (its progress events).

## Files

- `crates/core/src/event.rs` — `ScanProgress { scan, entries, bytes, expected: Option<ByteSize> }`; doc comment: "the
  size the scan will most likely walk, in the measure of `bytes`; `None` when nothing tells".
- `crates/engine/src/session.rs` (or a small `crates/engine/src/estimate.rs` if it grows past a function) — taken once
  at the scan's start, under the results' lock: when the last committed state's roots are the scan's roots (as paths,
  `~` filled in), the sum of their nodes' sizes in `bytes`' measure; otherwise the sum of the used space of the
  distinct volumes holding the roots (`RootVolumes`), when known; otherwise `None`. Every `ScanProgress` of the scan
  carries it.
- `app/ui/src/ipc/bindings.generated.ts` (regenerated); `app/ui/src/state/reduce.ts` keeps `bytes` and `expected` of
  the last progress beside `entries` (state shape in `state/store.ts` or where `progress` lives); `core-mock.ts` and
  the UI fixtures state `expected` (a number in the scanning scripts, `null` in one).

## Tests (negatives first)

Engine: negative `a_first_scan_without_volumes_has_no_estimate`; `a_scan_of_other_roots_does_not_use_the_last_state`;
positive `a_second_scan_estimates_by_the_last_state`, `a_first_scan_estimates_by_the_volumes_used_space`,
`every_progress_of_a_scan_carries_the_same_estimate`. UI: `reduce.test.ts` positive `a_progress_keeps_its_bytes_and_
estimate` (named in the project's `it('...')` style).

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core -p fresher-engine -p fresher-app`, clippy, fmt;
`npm run lint:ts`; `npx vitest run src/state`.

Commit: `feat(engine): the scan's progress carries an estimate`.

## Conclusions

- `ScanProgress.expected`, from `crates/engine/src/estimate.rs`: the last state's roots' logical sizes when every scan
  root is a root of that state (a subset match: the walked places are extra roots), else the used space of the roots'
  volumes, else none; nested roots count once.
- The estimate leaves out the walked places, which the progress counts, so the percent can pass 100: the UI clamps it
  and holds at 99 % (03/02). The volume branch mixes used space with logical bytes, a first-scan approximation.
