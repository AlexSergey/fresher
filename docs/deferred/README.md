# Deferred

Agreed work postponed to a known stage. Unlike [`../ideas/`](../ideas/README.md), deferred work is decided: it waits
for its turn. Before starting a new stage (a plan, a release), check whether a row here is due and remind the user.

## Format

One file per item, `<topic>.md`: what is postponed, who agreed and when, the stage it is due at, and what doing it
needs. When it is done, its row and file are removed in the same change.

| Item                                                             | Agreed     | Due at                   | What                                                                            |
| ---------------------------------------------------------------- | ---------- | ------------------------ | ------------------------------------------------------------------------------- |
| [Release versioning](./release-versioning.md) | 2026-11-16 | before the first release | the numbering from 0.1.0 and an automated release (bump, changelog, tag) |
| [Licence notices](./licence-notices.md)                          | 2026-11-15 | before the first release | ship the licences of every Rust and npm dependency in the app (MPL-2.0 first)   |
| [The fast scan on the developer's home](./fast-scan-home-run.md) | 2026-11-16 | before the first release | plan 004's budget run by hand: the home folder's scan, rescans, restore, clones |
| [The first real cleanups in a sandbox](./removal-sandbox-run.md) | 2026-11-16 | before the first release | plan 008's run by hand: the real Trash, put back, Finder's Put Back, skips, quit |
| [A project's last activity](./project-last-activity.md)          | 2026-11-16 | the stale-project module | the newest change of a project's own files, not only its folder's               |
| [Settings export and import](./settings-export-import.md)        | 2026-11-16 | when a user asks         | Export… and Import… with a preview of the changes; today the file itself       |
| [Clearing the app's own data](./own-data-clearing.md)            | 2026-11-16 | before the first release | clear the scan cache and stale temporary files without breaking the deletion rule |
| [A setting for the default removal](./default-removal-setting.md) | 2026-11-16 | when the user asks       | pre-select a permanent delete in the confirmation (rule 2 says no today)        |
| [Ignore on a single item](./ignore-an-item-button.md)            | 2026-11-16 | with the row's actions   | the Ignore button on one finding; the core already ignores and protects an item |
| [The world outside the disk in a sandbox](./sandbox-world.md) | 2026-11-16 | the first module that reads apps, processes, tools or the clock | plan 007's former plan 06: installed apps, running apps, open files, tools, access and the clock per case file |
| [The debug panel and the developer command line](./dev-debug-panel.md) | 2026-11-17 | when the deferred world is built | the panel's knobs need the world's fakes; the command line beyond the generator |
| [A root's explained line in the by-place roots list](./roots-explained-line.md) | 2026-11-17 | when several roots are common | the explained size of each root in the by-place roots list; needs a core change |
| [A way up from a folder opened from the by-meaning view](./way-up-from-meaning.md) | 2026-11-17 | with the next change to the overview | a breadcrumb that goes above the folder the by-meaning view opened |
