# More places of logs

**Agreed:** 2026-11-22, by the agent under the user's delegation of 2026-11-22 (plan 033; to check) ·
**Due at:** after plan 033's run on the developer's Mac

## What is postponed

Plan [033](../plans/033-logs/readme.md) offers the old files of the user's `~/Library/Logs`, six names deep at most,
outside `JetBrains` and the folders shaped as a bundle id. Three parts are postponed:

| Part | Why not in plan 033 | What doing it needs |
| ---- | ------------------- | ------------------- |
| An installed app's logs under its bundle id (`~/Library/Logs/<id>/`) | `app-leftovers` claims every such folder whole, so the walk never shows a file inside it to another module | either a second whole interest `Under(Logs, "*")` with `.only_bundle_ids()` in `logs`, offering the folder emptied (`Removal::EmptyFolder`) when nothing inside changed for the days and no installed-app check of `app-leftovers` claims it (as plan 030 did for caches), or a core change that lets an item claim see inside another module's whole claim |
| Files deeper than six names below `Logs` | the interests stop at six names, so a match stays a bounded set | a look at the real Mac for logs that deep; one more interest per depth, or a recursive pattern in the core |
| The system-wide logs (`/Library/Logs`, `/private/var/log`) | they need administrator rights, and the restriction keeps them out | the separate permission step of the idea [System Junk Cleaner](../ideas/system-junk-cleaner.md) |
