# Rules over one tree

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

[Scanning](./scanning.md) builds one arena tree of the whole scan, but nothing yet says how the rules use it. If each
rule walks its own paths, the same folders are read several times, scan time grows with every rule added (rule 4 of
[`../../rules.md`](../../rules.md)), and two rules can claim the same folder — its bytes are then counted twice and
the "unrecognized" remainder ([`../../goal.md`](../../goal.md), section 1) cannot be computed exactly.

## Direction

- **The scanner is the only code that walks the disk.** Rules never call the file system to list folders; they match
  nodes of the tree the scanner built.
- **Rules declare anchors**: path patterns under a root (`~/Library/Caches/*`, `~/.rustup/toolchains/*`) or marker
  files (`Cargo.toml`, `CACHEDIR.TAG`, [rules as data](./rules-as-data.md)). The engine indexes all anchors once (a
  prefix trie over path components), so the cost per node does not grow with the number of rules.
- **A claim marks a subtree** with one finding. **Unrecognized space is a remainder**: a folder's size minus the sizes
  of its claimed descendants — exact by construction, shown as the biggest unclaimed folders, never as "Other".
- **Overlapping claims are a bug**, caught by a test that runs every rule over every fixture home and fails on a node
  claimed twice. At run time, if it still happens, the higher danger level wins (rule 1 of
  [`../../rules.md`](../../rules.md)) and the conflict is logged.
- **Small reads on demand.** Some rules must read a file to decide (`Info.plist`, `pyvenv.cfg`, an AVD's `config.ini`,
  `rust-toolchain.toml`). They get a read-only reader with a size cap, never a general file system handle — the
  privacy rule "read only what a rule needs" ([`../../goal.md`](../../goal.md), section 7) enforced by the API.
- **Probes for what is not on the tree.** Facts that come from a program, not from files (the Docker CLI, `tmutil`,
  `xcrun simctl`) come from probes behind traits ([`../../architecture/README.md`](../../architecture/README.md)). A
  probe's finding may point at a tree node it lives inside (Docker's categories inside its disk image), so nothing is
  counted twice ([`../container-disk-image.md`](../container-disk-image.md)).
- **Streaming.** A subtree is complete when all its children are walked; rules anchored in it can run then, so
  findings stream to the UI during the scan instead of after it.

Open questions:

- Run rules during the walk (on completed subtrees) or after it? During is faster to first result; after is simpler.
  Proposed: after, for the first version; measure before changing it.
- How the results reach the UI: generated TypeScript types from the Rust types (for example `specta` or `ts-rs`, both
  MIT) and findings streamed through a Tauri channel — a decision for the plan that builds the first IPC command.
