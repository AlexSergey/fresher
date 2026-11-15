# A Rust build cache shared by the git worktrees

## Problem

Every agent works in its own git worktree, and every worktree has its own `target/`. A new worktree compiles the whole
dependency tree, Tauri included, from scratch, twice in `npm run pre-push` (Clippy, then the coverage build), so the
hook took a long time on 2026-11-15.

## What was tried

1. **One `CARGO_TARGET_DIR` for every worktree.** Rejected before it was built: the worktrees hold different versions
   of the same crates, which land on the same artifacts; cargo judges our crates fresh by file times, so a worktree could
   run its tests on another worktree's build and pass. Parallel builds would also wait on one lock.
2. **sccache 0.19.0** as `rustc-wrapper` in `~/.cargo/config.toml`, then with `SCCACHE_BASEDIRS` set to the main
   checkout and a worktree.

## Result

sccache hit only when the build ran in the same `target/` path: a rebuild after removing `target/` took every crate
from the cache. Two different `target/` paths, in one checkout or across the main checkout and a worktree, gave 0 %
Rust hits and the same time (55–60 s for a cold `cargo clippy --workspace --all-targets`), with and without
`SCCACHE_BASEDIRS`. sccache was removed.

## Lesson

- `cargo clean` only when a global plan is done ([`../workflow.md`](../workflow.md), "End of work"), so the worktree of
  a plan keeps a warm `target/` for every later check.
- Before trying a cache again, measure two cold builds in two different `target/` paths and look at the Rust hit rate.
- Not tried yet: cloning a warm `target/` into a new worktree with an APFS copy (`cp -c`); it must first be shown not
  to leave stale builds of our crates.
