# 004/05 — Walker

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner, testing, engine, app

The full scan: the large-tree generator and the bench harness first, so every later step measures; then the
boundaries and the portable reader, the workers and the aggregator, errors and completeness checked against the
reference walk, the claims and the commit in the engine, and finally the app wired to the real scanner. After this
plan the app scans the disk for real (in full every time; rescans come in 07). The design:
[`../00-design.md`](../00-design.md), sections 3, 7 and 13.

Depends on 03 (the macOS reader for the benches and the app) and 04 (the tree and the matcher). Runs in parallel with
06/02–06/03. Every test runs on a `FixtureTree` or a generated tree in a temporary folder.

## Steps

| #   | Step                                                              | Status   | Shared files                                                          |
| --- | ----------------------------------------------------------------- | -------- | --------------------------------------------------------------------- |
| 01  | [Large trees and the bench harness](./01-large-trees-and-benches.md) | ✔️ done | `crates/testing/`, `crates/scanner/Cargo.toml`, `package.json`, `Cargo.lock`, `docs/testing.md` |
| 02  | [Roots, boundaries and the portable reader](./02-boundaries-and-std-reader.md) | ✔️ done | `crates/testing/src/fixture_tree.rs`                  |
| 03  | [Workers and the aggregator](./03-workers-and-aggregator.md)      | ✔️ done | —                                                                     |
| 04  | [Errors, completeness and the reference](./04-errors-and-reference.md) | ✔️ done | —                                                                |
| 05  | [Claims and commit](./05-claims-and-commit.md)                    | ✔️ done | `crates/engine/`                                                      |
| 06  | [The app on the real scanner](./06-app-wiring.md)                 | ✔️ done | `app/tauri/`, `crates/e2e/`, `fixtures/`, `docs/hacks/README.md`      |

In order.

## Conclusions

- The full scan is built and wired: the generated trees and the bench harness, the portable reader and the counting
  reader, the roots and boundaries, the workers and the aggregator, errors and completeness checked against the
  reference walk (on every fixture home, on random trees, with the portable and the macOS reader), the claims and the
  committed `TreeState`, and the app scanning with the macOS reader. Each step is one commit through the hooks.
- **Numbers** (release, the developer's machine): `SMALL` built in 7.9 s, `LARGE` in 529.5 s (≈ 300 MB of disk); a
  full scan of `SMALL` with one thread 470 ms; of `LARGE` 23.4 s with 12 threads, **15.4 s with 24 (the best)**, 16.7 s
  with 36, 18.0 s with 48, ≈ 245 s with one; peak memory 160 MB at 12 threads (bar 200 MB), 333 MB after the
  24–48-thread runs; `Matcher::entry` ≈ 50 ns an entry under load (28 ns idle in 04).
- **For the user: the bar "full scan of `LARGE` ≤ 10 s" is not met, and the thread count is in question.** A profile
  shows the walk waiting on the kernel (92 % of samples in `getattrlistbulk`, 0.1 % on the queue's lock): `LARGE`'s 3
  million entries do not fit the kernel's metadata cache (`kern.maxvnodes` 263 168), so every scan reads cold, unlike
  01's warm trees. On such a tree two threads per core beat one, the opposite of 01; the default stays one per core
  (at most 16) until the user decides between the bar on a warm tree, a cold-tree bar, or more threads.
  **Taken (2026-11-16, the user asked for the recommended calls; to check):** the 10 s bar is measured on a warm tree,
  as 01 set it; a cold full scan is recorded with no bar in 08/03, where the user also decides whether a cold-scan
  target is wanted; the default stays one thread per core (at most 16), because 24 or more threads took the peak memory
  to 333 MB, over the 200 MB bar. 08/03 measures one and two threads per core on the developer's home, cold and warm.
- **Deviations** (each in its step): the aggregator queues the subfolders (it alone makes nodes), so the queue counts
  outstanding jobs; the default thread count follows 01's call 13, not the step's "3 × the cores"; the dropped roots are
  logged, not in the summary (no field); a vanished folder keeps its node as a `Failed` one; the whole claims' insides
  are not counted in the summary, as in the reference walk.
- **Known issue fixed**: the reference walk now finds markers and siblings among the listed names, compared exactly
  (it took `cachedir.tag` for `CACHEDIR.TAG` on a volume that ignores case), and its whole sizes do not enter an
  avoided folder. The property test found and kept the seed of a bug fixed before 03's commit (a marker deep inside a
  whole claim taken as the claim's own).
- **Left for the user**: the by-hand `npm run tauri dev` scan and the time of a full scan of the developer's home.

