# A special entry shown as a file in the browse view

**Agreed:** 2026-11-17, by the controller of plan 017 (review of step 01/02) · **Due at:** the next change to the
browse view's entry kinds

## What is postponed

The core knows a fourth entry kind since plan 017/01/02: `EntryKind::Other`, a socket, a named pipe or a device. The
scanner keeps it in the tree, but the browse view's `NodeKind` (`crates/scanner/src/state/mod.rs`, `kind_of`) has
folders, files, links and collapsed folders only, so a special entry that is a node is shown as a file.

## What doing it needs

- A `NodeKind` for a special entry, read from the node's flags, carried to the UI's bindings.
- The browse view showing it as itself (its own icon and words), never offered; the state's tests for a pipe node.
