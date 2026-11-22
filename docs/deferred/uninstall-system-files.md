# More of the uninstall: system-wide files, other names, package managers

**Agreed:** 2026-11-22, by the user's delegation of plan [036](../plans/036-uninstall/readme.md) ·
**Due at:** when the user asks for it after the first release

## What is postponed

Plan 036 moves an app's bundle and the files of the user's `~/Library` named by its bundle id. Postponed:

| Part | Why not in plan 036 | What doing it needs |
| ---- | ------------------- | ------------------- |
| An app's system-wide files: `/Library/LaunchDaemons`, `/Library/PrivilegedHelperTools`, `/Library/Application Support/<id>` | they need an administrator, and the app never elevates ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)) | shown on the app's page with their sizes and "needs an administrator", never offered; the user removes them in Finder; a place each in the footprint table |
| Files kept under the app's name rather than its id (`Application Support/Editor`) | the rules tie a folder to an app by id only, and a name can be another app's | a table of names to ids, the same one [`installed-app-caches-more-places.md`](./installed-app-caches-more-places.md) needs, each row checked on a real install |
| Package managers beyond Homebrew casks, and running their own uninstall command | only the cask link is read, and Fresher runs no command that removes anything | a reader per manager, and a decision on whether Fresher may run a manager's uninstall for the user (today it shows the command to copy) |
| Watch mode's growing apps | plan [010](../plans/010-history/readme.md) is not built | the history compares the Apps list's numbers when it is built |
