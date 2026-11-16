# 004/06 — Cache

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** store, scanner

The user's requirement: a super lightweight cache — capped, safe to lose, never cluttering the disk. The store owns
the files (fixed slots, two generations, an envelope with a checksum, a cap, rotation by rename, nothing ever
deleted); the scanner owns the body (the tree in depth-first order, variable-length integers, front-coded names) and
makes it fit the cap. The design: [`../00-design.md`](../00-design.md), section 10.

Step 01 depends on 02 only and runs in parallel with 03 and 04 (it owns `crates/store/src/scan_cache.rs` and
`envelope.rs`). Steps 02 and 03 depend on 04 (the tree) and run in parallel with 05 (they own
`crates/scanner/src/codec/`). Every test runs on a `tempfile::TempDir` as the data folder.

## Steps

| #   | Step                                                      | Status   | Shared files                                                   |
| --- | --------------------------------------------------------- | -------- | -------------------------------------------------------------- |
| 01  | [Cache files in the store](./01-cache-files.md)           | ✔️ done | `crates/store/src/lib.rs`, `crates/store/Cargo.toml`, `Cargo.lock`, `docs/architecture/README.md` |
| 02  | [The tree codec](./02-tree-codec.md)                      | ✔️ done | `crates/scanner/src/lib.rs`                                    |
| 03  | [Fitting the cap](./03-fitting-the-cap.md)                | ✔️ done | —                                                              |

01 first (or in parallel with 02: the codec is tested against `MemoryScanCache`); 03 after 02.

## Conclusions

- The store keeps the files (01) and the scanner writes, reads and fits the body (02, 03); each step one commit
  through the hooks. The body is not wired into `Scanner` yet: 07/01 makes `commit` save it through `fit` and the
  rescan and the restore load it through `decode`.
- **The cap is on the file on disk**, compressed (the user's decision; the agent's reading confirmed by the user on
  2026-11-16): the store refuses a sealed file over 8 MiB
  and bounds the body before compression at 64 MiB; the scanner's `fit` measures the body compressed at the store's
  level and collapses only when the file would pass the cap. 07/01 passes both limits from the composition (the
  port carries only the file cap).
- **Numbers**: 22.1 bytes a folder node in the body (bar: under 24); on a synthetic 556 000-folder tree, encode
  18 ms, decode 44 ms, compression 25 ms, `fit` to a 2 MiB file 86 ms. The benches on `LARGE` come with 07/01.
- **Deviations**, each in its step: the cap on the file and a separate body limit, no `generations` field and an
  owned body (01); the link and clone groups written whole, kept files in the same depth-first list as folders, a
  node count instead of a kept-file count (02); `fit` compresses to measure and takes the body limit too (03).
- `docs/dependencies.md` names the scanner for `miniz_oxide` (03), and for `tracing` and `proptest`, which plan
  004/04 added without updating it.
