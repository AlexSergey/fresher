# The "Reopen Fresher" button is forgotten when its card is rebuilt

**Status:** open · **Updated:** 2026-11-19

## Symptom

After "Open System Settings" on the Full Disk Access card or row, "Reopen Fresher" appears beside it. When the card
is rebuilt (the user leaves the overview or the settings screen and comes back, or a new scan replaces the notice),
the button is gone until "Open System Settings" is clicked again.

## Cause

Whether settings were opened is state of the component (`useState` in `coverage-notice.tsx` and `permission-rows.tsx`),
not of the store, so it is lost when the component unmounts.

## What was tried

Nothing: clicking "Open System Settings" again brings it back, and the pane opens again, which does no harm.

## Options left

- Keep "settings were opened for Full Disk Access" in the store for the session.
- Show "Reopen Fresher" whenever Full Disk Access is missing, with the note that it is needed only after a grant.
