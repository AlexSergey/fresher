# 018/01 — Core prerequisites

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** core, scanner, testing, docs

What the code lacks before a module can recognize a tool's cache by the tool's own sign inside it: a matched folder
reports which names it holds, and whether a held `CACHEDIR.TAG` is valid. The design:
[`../00-design.md`](../00-design.md), section 1 (agent's call 1).

## Steps

| #   | Step                                                   | Status      | Shared files                                                                                                   |
| --- | ------------------------------------------------------ | ----------- | -------------------------------------------------------------------------------------------------------------- |
| 01  | [What a matched folder holds](./01-holds.md)           | ✅ approved | `crates/core/src/{interest,lib}.rs`, `crates/scanner/src/{matcher/,walk/,tree/,codec/,rescan/}`, `crates/testing/src/naive_walk.rs`, `docs/architecture/modules.md` |

First, before every other step.

## Conclusions

Built as 01/01 says: a matched folder reports the names it holds and a held tag's validity, in the walk, the cache
(tree format 6), the rescan and the reference walk. The scan bench is to be retaken on a quiet Mac (04/02).
