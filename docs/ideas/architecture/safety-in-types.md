# Safety in the types

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

Plan [008](../../plans/008-safe-removal/readme.md) already holds rule 2 of [`../../rules.md`](../../rules.md) in the
type system: the cleaner's only entry point takes a confirmed plan, and only a confirmation can make one. A few gaps
remain where a refactor could still break a promise without the compiler noticing. These are additions to plans
[002](../../plans/002-base-architecture/readme.md) and 008, not a second design.

## Direction

- **`Evidence` instead of a free-text reason.** Plan 002's `NewFinding` carries `reason: Text`. An enum of the facts
  the rule relied on — `NewerVersionInstalled { app, installed, this }`, `MarkerFile { marker }`,
  `NotUsedBy { checked_folders }`, … — renders the same text, and is also what the re-check right before removal
  compares against: "still a marker folder", "still not used". A text cannot be re-checked.
- **The cleaner removes a `Verified`, not a path.** `verify(item) -> Result<Verified, Skipped>` runs plan 008's checks
  right before each item. `Verified` is not `Clone` and is consumed by the removal call, so one check cannot be reused
  for a later removal. The identity check goes through an open file descriptor (`O_NOFOLLOW`, then `fstat`), so the
  item checked is the item removed.
- **Permanent removal needs a token.** `DeletePermanently(PermanentConsent)`, where `PermanentConsent` is built only by
  plan 008's separate permanent-delete confirmation: a module can describe a removal (plan 002's `Removal`) but can
  never ask for a permanent one.
- **The summary comes back.** With the ids, the UI sends back the summary the user saw (item count, total, highest
  danger level); the core rebuilds it and refuses on any mismatch — "confirmed 3 items" can never remove 300, even if
  the UI built the id list wrong.
- **Compile-fail tests** (`trybuild`, MIT / Apache-2.0) next to plan 008's property tests: a confirmed plan cannot be
  built from findings, the trash call does not accept a `Path`, a `Verified` cannot be used twice.

What the types cannot do: the time between the check and the move to the Trash still exists (the trash API takes a
URL). Plan 008's threat model names it; checking through the open descriptor keeps it small.

Open question:

- Plan 008 lets an unconfirmed plan expire after a short time. The re-check already covers changes made in between:
  does expiry add safety, or only make the user confirm again?
