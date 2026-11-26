# The "Reopen Fresher" button is forgotten when its card is rebuilt

**Status:** fixed (`fix(ui)`: "settings were opened for Full Disk Access" lives in the UI store for the session) · **Updated:** 2026-11-26

## Symptom

After "Open System Settings" on the Full Disk Access card or row, "Reopen Fresher" appears beside it. When the card
is rebuilt (the user leaves Scan or the settings screen and comes back, or a new scan replaces the notice),
the button is gone until "Open System Settings" is clicked again.

## Cause

Whether settings were opened is state of the component (`useState` in `coverage-notice.tsx` and `permission-rows.tsx`),
not of the store, so it is lost when the component unmounts.

## Fix

The flag lives in the UI store (`fullDiskSettingsOpened`, set by the `full-disk-settings-opened` action) for the session
and is not saved. "Open System Settings" on the Full Disk Access card and on its row sets it; both read it, so
"Reopen Fresher" stays after the card is rebuilt and shows on the card and the row alike.
