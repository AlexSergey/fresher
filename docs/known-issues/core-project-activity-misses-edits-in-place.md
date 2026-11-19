# A project's last activity does not see a file edited in place

**Status:** open · **Updated:** 2026-11-19

## Symptom

A project whose files are only ever edited in place (no new file, no rename, no commit) for the whole period looks
untouched to the stale projects module, so its artifact folders (`node_modules`, `target`, `.venv` and the like) can be
offered while the user still works in it.

## Cause

A project's last activity is the newest modification time of its folders, outside its artifact folders and `.git`,
or the newest folder time of `.git/refs/heads`. A folder's time moves when a name is added, removed or renamed in it,
and an editor that writes a file in place moves no folder's time. The walk keeps no time for a file under 1 MiB, so
files cannot be counted; status changes are not used because a copy or a migration moves every one of them at once
(decision 5 and agent's calls 1 to 3 of plan [021](../plans/021-stale-projects/00-design.md)).

## What was tried

On 2026-11-19, over 19 git projects on the developer's Mac, the newest folder time was never older than the newest
file time. Most editors save through a temporary file and a rename, which moves the folder. Which editors save in
place is checked by hand in plan 021 step 04/02.

## Options left

The harm is limited: the project must show no folder change and no commit for 365 days (a setting, at least 30), only
folders its tools make again are offered (Low, Medium for an environment), nothing is offered without the user's
confirmation for that exact set, and it goes to the Trash.

- Count the newest time of the files the walk does keep (the big ones), tying the answer to the fold threshold.
- Read the project's few top-level files' times after the walk (a new port, and a restored scan cannot make that read).
- A per-folder switch in the settings to keep a project, which the ignore list ([idea](../ideas/ignore-list.md)) would give.
