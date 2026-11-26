# Clearing the app's own data

**Agreed:** 2026-11-16, delegated to the agent by the user (plan 005, decision 5) · **Due at:** when the scan cache
or the stale temporary files cause a problem, or 0.2.0 (moved from before the first release by the user, 2026-11-26)

## What is postponed

"Clear the scan cache" in the "Your Fresher data" group, and the removal of stale temporary files
([known issue](../known-issues/store-stale-temporary-files.md)).

## What doing it needs

Only `fresher-cleaner` may delete ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)). Either
the cleaner gets a narrow operation for the app's own data folder (regular files only, never a link, names matching
the store's own files exactly), or the user agrees an exception to the deletion lint for the data folder, with its
reason in the restriction. The scan cache can also be made empty by writing an empty cache through the store, which
deletes nothing; that is the first option to try.
