# Safety in the types

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

Most guarantees of [`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md) are enforced "by
the cleaner (planned)" or "by review". Review misses things: one refactor that builds a clean plan straight from scan
results, or passes a path to the trash function without the re-check, and the core promise is gone. The compiler can
hold most of these rules instead.

## Direction

Make the unsafe paths impossible to write, not just forbidden:

- **`Finding` carries `Evidence`** — the facts the rule relied on, as an enum, not a string:
  `NewerVersionInstalled { app, installed, this }`, `MarkerFile { marker }`, `NotUsedBy { checked_folders }`, … The
  UI's "why it matched" ([`../best-practices.md`](../best-practices.md), 4) is rendered from it, and it is the record
  the re-check compares against.
- **`Finding` carries the item's identity at scan time**: device, inode, kind, private size, modification time.
- **`Confirmation` is built only by the core**, from the user's selection (opaque finding ids from the UI) and the
  summary the user saw (item count, total, highest danger level). The UI sends the summary back; the core rebuilds it
  and refuses on any mismatch — "confirmed 3 items" can never delete 300.
- **`CleanPlan` has private fields and one constructor**, `CleanPlan::from(Confirmation)`. A plan is tied to the scan
  it came from: a new scan makes older plans invalid.
- **`Disposal` is an enum**: `MoveToTrash` or `DeletePermanently(PermanentConsent)`, where `PermanentConsent` comes
  only from the separate permanent-delete confirmation.
- **The cleaner deletes a `Verified`, not a path.** `verify(item) -> Result<Verified, Skipped>` runs the re-check
  (same identity, not a symlink, not protected, not in use, size not grown past the confirmed margin) right before
  removal. `Verified` is not `Clone` and is consumed by the delete call, so one check cannot be reused for later.
- **Compile-fail tests** (`trybuild`, MIT / Apache-2.0) prove that a `CleanPlan` cannot be made from a `Finding`, and
  that the trash call does not accept a `Path`.

What the types cannot do: the time between `verify` and the move to the Trash still exists (the trash API takes a
URL). The plan for the cleaner keeps that window small and writes it into the threat model
([rules as data](./rules-as-data.md), "Protected paths").

Open questions:

- Is the UI side trusted? Proposed: no — the webview holds only opaque ids, all plans live in the core.
- Does a confirmation expire with time, or only with a new scan? Proposed: only with a new scan, and the re-check
  covers changes in between.
