# More on the Largest screen

**Agreed:** 2026-11-22, by the user's delegation of plan [035](../plans/035-large-files/readme.md) ·
**Due at:** after plan 035's walk on the developer's Mac

## What is postponed

The screen shows "Changed", not "last opened", and offers nothing but the file's name, kind, place, size and date.
Postponed:

- **"Last opened"**: Spotlight's last-used date is mostly empty on the developer's Mac (macOS 27.2.1), so the design
  uses the file's modified time. It needs the last-used date through a port, asked per page, and the access time kept
  for kept files, measured.
- **"The app that made it"**: Spotlight's creator, or the package's handler, through a port.
- **A treemap** of the same rows.
- **A sync client's folder marker** in the home: a legacy sync client's folder is offered like any other folder today.
- **Places outside the home**: other scan roots and volumes.
- **A folder's "Changed" from its newest file**: today it is the folder's own time
  ([`folder-age-from-children.md`](./folder-age-from-children.md)).

## What doing it needs

Each item needs a fact measured on a real Mac (the first two), a port with a fake, or a design decision; see plan 035's
[`00-design.md`](../plans/035-large-files/00-design.md) for the table of facts.
