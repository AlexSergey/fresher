# 044/01 — The newest change in the scan tree

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** scanner, docs

Every folder of the tree knows the newest change of anything inside it, small files included, from the walk itself,
the cache and a rescan. Nothing reads it yet (plan 02 does). The design: [`../00-design.md`](../00-design.md),
sections 1 and 2; agent's calls 1 to 4.

## Steps

| #   | Step                                                     | Status   | Shared files |
| --- | -------------------------------------------------------- | -------- | ------------ |
| 01  | [`newest` from the fold and `freeze`](./01-newest-in-the-tree.md) | ✔️ done | `crates/scanner/src/tree/{node.rs,fold.rs,apply.rs,arena.rs}` and their tests |
| 02  | [The cache's format 7 and the rescan](./02-cache-and-rescan.md)   | ✔️ done | `crates/scanner/src/codec/{header.rs,encode.rs,decode.rs,testing.rs}`, `crates/scanner/src/rescan/patch.rs` and their tests |

01, then 02.

## Conclusions

**Measured** (after steps 01 and 02, `LARGE`, release, 12 threads, one run, load about 10): cold full scan 21.65 s
(plan 004: 22.8 s); rescan with nothing changed 36.3 ms, after 1 000 changed folders 206 ms (bars 0.5 s / 1 s); cache
body 26 B a node (bar 32 B; was 24 B); peak memory 226 → 238 MB with the tree kept, encoded and fitted (+8 B a node for `newest`; plan 004 records 143 MB
for a bare full scan and 226 MB in this configuration).
