# 004/04 — Tree and matcher

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

The two pure parts of the scanner, with no IO and no threads: the arena the aggregator builds the tree in, and the
compiled matcher the workers ask about every entry. Both are tested on values built in memory, so the walker (05)
starts from parts that already hold their invariants. The design: [`../00-design.md`](../00-design.md), sections 4–6.

Depends on 02. Runs in parallel with 03 and 06/01: it owns `crates/scanner/src/tree/` and
`crates/scanner/src/matcher/`.

## Steps

| #   | Step                                                          | Status   | Shared files                                                    |
| --- | ------------------------------------------------------------- | -------- | --------------------------------------------------------------- |
| 01  | [The arena](./01-arena.md)                                    | ✔️ done | `crates/scanner/Cargo.toml`, `crates/scanner/src/lib.rs`, `Cargo.lock` |
| 02  | [The compiled matcher](./02-matcher.md)                       | ✔️ done | `crates/scanner/src/lib.rs`                                     |
| 03  | [Folding, links and totals](./03-folding-links-totals.md)     | ✔️ done | —                                                               |

01 and 02 are independent; 03 after 01.

## Conclusions

- The tree and the matcher are built, pure and tested on values in memory and on fixture trees, each step one commit
  through the hooks. Both are crate-private (`#[expect(dead_code)]` until the walker of 05 uses them).
- **Numbers**: a node is 72 bytes; 83 heap bytes a node on a tree of 100 000 nodes with 12-byte names (bar: under
  96); `Matcher::entry` 28 ns an entry on a 64-interest set (release; bar: 50 ns).
- **Deviations**, each in its step: a node holds one set of sums (its own bytes until `freeze`, its subtree's after)
  instead of `own` and `total`, to stay under the bar, and `freeze` is one-way; small hard links and clones are folded
  and listed in the link and clone tables, as the design's measured call 21 says, not kept as the step said; a clone
  group counts the largest shared part of its members; the fingerprint also covers the resolved anchors.
- **For the walker (05)**: `Matcher::compile` → `root_state` / `child_state` / `entry` / `folder`; `fold` →
  `Tree::apply` (which gives the new nodes); the marker hits of a folder come from its own listing, with sibling masks
  from its parent's (carried in the job); `claims_whole` → read the subtree to the end and `apply_opaque`;
  `checks_cachedir_tag` → read the tag and set `Hit::marker`; `freeze` at the end.
- **Found, for the main session**: the reference walk finds markers and siblings through the file system, so on a
  volume that ignores case it accepts `cachedir.tag` for `CACHEDIR.TAG` while the matcher compares names exactly; and
  `RawEntry` cannot tell a FIFO from a regular file, so a FIFO named like a marker is a marker for the matcher (the
  walker's tag read refuses it). Neither is changed here: the first lives in `fresher-testing`, the second in the core.
