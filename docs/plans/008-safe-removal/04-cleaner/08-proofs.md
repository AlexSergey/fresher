# 008/04/08 — Proofs and the threat model

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner, docs

## Files

- `crates/cleaner/tests/never.rs` (new) — the property tests of the design, section 13 (`proptest`, failing seeds
  kept in `never.proptest-regressions`):
  - a generator of fixture trees: nested folders and files, links pointing in, out and up, a folder standing in for
    each protected group (a fake `Protection` names them: some `never`, some `whole`, one `except`), an ignored folder,
    and a sibling folder outside the roots;
  - a generator of plans: items drawn from the tree's paths and from made-up ones (outside the roots, through links,
    `..`, the roots themselves, protected places and their ancestors), each with a random action, and a random method;
  - for each run of `execute` (and of `rehearse` at both depths) with `FakeTrash`: every moved or deleted original path
    was a plan item, strictly inside a root physically, not protected, not inside nor an ancestor of a protected or
    ignored path, not a `whole` folder; the outside folder and every protected subtree list byte for byte as before; a
    rehearsal leaves the whole tree as it was.
- `crates/cleaner/tests/types.rs`, `crates/cleaner/tests/compile_fail/` (new) — `verified_clone.rs`,
  `verified_used_twice.rs`, `verified_literal.rs`, `remove_without_act.rs`, each with its `.stderr`.
- `docs/architecture/removal.md` (new) — for developers: the crate and its one entry point, the flow, the reach and
  the sandbox (how to clean in a development build), the guard's checks in order with their skip reasons, the kinds of
  removal and the deletion point, the threat model (the design's list, with what each test proves), fail closed, put
  back, and what a new kind of removal must add. Linked from `docs/architecture/README.md` in 08/01.

## Produces

The proof that the cleaner keeps its promises for any plan, and the page a developer reads before touching it.

## Checks

- `cargo nextest run -p fresher-cleaner` runs the property tests at their default case count in the hooks; a run of
  10 000 cases is made once by hand and its time written in the step's conclusions.
- The compile-fail cases fail with the expected messages.
- Coverage of `fresher-cleaner` at or above 90 % of lines (`npm run test:cov:rust`).
- `docs/architecture/removal.md` names every check, skip reason and test file that exists, and nothing that does not.

## Conclusions

- **`tests/never.rs`**: each case builds a tree in a temporary folder — a scan root holding the sandbox marker, with a
  `never` place, its carve-out, a `whole` folder and an ignored folder at fixed places inside it (one of them below
  `scan/a`, one below `scan/b`, so their ancestors are tested too), a folder outside the root, and random files,
  folders and links (to the outside folder, to a file outside, into the root, to the protected place, up to the
  tree's own folder, up by `../../..`) — and a plan of 1 to 11 items drawn from every entry of the tree and from
  made-up paths (the tree's folder, `/`, `/etc/hosts`, `scan/..`, `scan/a/../b`, `scan/./a`, a sibling `scan2`, the
  canonical `/private/…` form, paths through each link), each removed, emptied or given to a tool, with a random
  method. It runs `rehearse` at both depths and `execute` with `FakeTrash`, and holds: a rehearsal changes nothing;
  every acted-on item and every entry that went is allowed by a model written as plain path comparisons (strictly
  inside the root, not in or above the `never` place unless inside its carve-out, not the carve-out, the `whole`
  folder or above them, not in or above the ignored folder); every entry that went lies at or inside an acted-on plan
  item (strictly inside for `Empty`), compared without resolving links, so a followed link or a resolved `..` would
  fail it; the outside folder and the protected and ignored subtrees list exactly as before (path, file id, link
  target, bytes); the fixed folders keep their file ids; no tool runs. In 256 cases about 15 % of items are acted on
  and the rest are skipped for every reason the guard has (sampled once by hand). **10 000 cases ran once by hand in
  54 s** (`PROPTEST_CASES=10000`), all passing; no failing seed exists yet, so `never.proptest-regressions` is made by
  proptest when one is found.
- **`tests/types.rs`** with `tests/compile_fail/`: `verified_clone` (`Verified` is not `Clone`), `verified_used_twice`
  (a removal that takes it consumes it), `verified_literal` (its fields are private: only the guard makes one),
  `remove_without_act` (the removals are private to the crate: no code outside it can act except through
  `Cleaner::execute` with a confirmed plan; inside, each removal takes an `Act`). Each `.stderr` was reviewed.
- **Coverage of `fresher-cleaner`**: 96 to 100 % of lines in every file of `src/` (`cargo llvm-cov nextest -p
  fresher-cleaner`), above the 90 % bar.
- **`docs/architecture/removal.md`** written; it names the guard's eleven checks with their files and reasons, the
  reasons the plan builder gives instead (`Covered`, `NotTheSubject`), the four test files and the four compile-fail
  cases. The link from `docs/architecture/README.md` is left to 08/01, which owns that file.
- Added: `proptest` and `trybuild` as the crate's dev-dependencies (`docs/dependencies.md` updated).
