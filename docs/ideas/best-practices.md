# Best practices of the field

**Since:** 2026-11-15

What the best open-source cleaners, disk analyzers, duplicate finders, developer-artifact cleaners and uninstallers do
well, and what their users complain about — described as ideas only. Clean room: no project is named and no code is
taken ([`../restrictions/repository.md`](../restrictions/repository.md), "Clean room"). Architecture and implementation
techniques are in [`architecture/`](./architecture/README.md).

## What the field does well

### Cleaners

- A dry run for every action that deletes, and a debug view that says why each item was picked or skipped.
- An operation log and a history of every cleanup, readable by the user.
- A protection list of paths that are never touched; when it cannot be read, the clean stops instead of running
  without it.
- Unidentified folders are left out of the results entirely rather than offered as junk.
- Every path checked against a list of allowed roots; symlinks resolved before validation and again right before the
  move to the Trash.
- A written "trust contract" in the product: no fake urgency, no "speed up your Mac" claims, no telemetry.
- Purgeable space shown in the storage breakdown but never counted as junk.
- Badges that say how an item comes back: rebuilds by itself, re-downloads, costs a login, or is gone for good — and,
  for anything the app cannot remove, the exact command or Settings path the user can use.
- Growth tracking next to each size and a history of what came back after a cleanup.
- Declarative, versioned cleanup rules, each shipped only after its safety boundary is checked on a real system.
- Risky categories hidden unless the user asks for them, and never cleaned unattended.
- Cache folders emptied but kept: apps expect the folder to exist.
- Model stores (local AI models) measured by the bytes actually freed, shared blobs counted once.

### Disk analyzers

- Recursing into the biggest entries automatically, without a depth setting.
- A parent's share drawn as a shadow behind its children's bars.
- Sunburst and treemap views that build live during the scan, with an adjustable number of levels.
- An age heatmap: the treemap coloured by last-modified date.
- A "collector" tray: items are gathered while browsing, reviewed together, then moved to the Trash.
- Saved scans that reopen instantly, and scans stored in SQLite for later comparison. *We keep plain files, not
  SQLite — [plan 002, decision 1](../plans/002-base-architecture/00-design.md).*
- A read-only mode, and a setting that switches off permanent delete entirely.
- Ignore files in gitignore syntax.
- Sizes marked complete, partial (`42 GB+`) or unknown — an unmeasured size is never shown as 0.

### Duplicate finders

- Staged filtering — size, then the first block, then the last block, then the full hash — so most files are never read
  in full.
- A persistent hash cache that makes later scans fast.
- Two steps: a report the user reviews, then an action on it (remove, hard-link, or replace with an APFS clone).
- Freeing space with copy-on-write clones instead of deleting, so every path keeps working.
- Reference folders that are never deleted from; the copy with Finder tags or comments is preferred.
- Separate tools for empty folders, empty files, broken symlinks and similar images, videos and music.

### Developer-artifact cleaners

- Projects detected by their marker files (`package.json`, `Cargo.toml`, …), each type with its own artifact folders.
- A filter for projects not touched in N days.
- Build artifacts removed by age, and those built by toolchains no longer installed.
- A warning on `node_modules` folders that installed apps need to run.
- Xcode data split by kind and by iOS version, with a warning that the layout is undocumented and may change.

### Uninstallers

- A monitor that notices an app moved to the Trash and offers to clean its leftovers.
- Every leftover shows why it matched and how confident the match is.
- Personal data kept by default; the user opts in path by path.
- Package-managed apps handed to their package manager to uninstall.
- Synced and iCloud paths refused; files used by another app shown but never pre-selected.
- A history that can put trashed items back where they were.

### Cleaners for other systems

The same problem on other operating systems gives ideas that carry over to macOS:

- The user's own folders are **hard-blocked** as a whole: Documents, Desktop, Pictures, Videos, Music, Downloads (the
  folder itself) and the home folder can never be wiped, whatever a rule says.
- Links are never followed — neither symlinks nor the system's own link kind — so a link cannot point the cleaner
  somewhere unexpected.
- Files in use are skipped, never forced.
- "Safe, but unticked": a safe category whose rebuild costs time (GPU shader caches) is offered but not pre-selected.
- A **demo mode** with made-up data: nothing on the machine is read or touched; safe for recordings and for tests.
- One command that runs the demo and saves a screenshot of every screen — the README and the site stay current.
- A read-only scan written to a text report, for bug reports the user chooses to send.
- Optional background checks, off by default: a low-space alert that only reads free space, and a weekly cache check
  that waits for idle time, pauses on battery and stops after a few minutes; at most one reminder a day; a reminder
  always opens a fresh scan, nothing is deleted automatically.
- Large game libraries uninstalled through their launcher, never by deleting their files.

## Ideas worth taking, ranked

1. **How it comes back** — rebuilds, re-downloads, costs a login, gone for good — with the total that would need the
   network to return. Complements the danger level.
2. **Growth and "what came back"** — our watch mode, shown per item, with a history of what returned after a cleanup.
3. **Honest sizes** — complete, partial or unknown; never 0 for unmeasured.
4. **Why it matched, and how sure.**
5. **Reasons for every skip**, and a list of what was refused.
6. **Fail closed** — no protection or ignore list, no clean.
7. **Allowed roots and a re-check right before removal** — already in
   [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md).
8. **Rules as data** — declarative, versioned, each with a written safety boundary and fixture tests (rule 5 of
   [`../rules.md`](../rules.md)).
9. **Trash only, never empty it, put back from history** —
   [`cleanup-preview-and-safe-removal.md`](./cleanup-preview-and-safe-removal.md).
10. **Show the exact action** in the confirmation: every item, its size, the operation that will run.
11. **Never pre-select** large or old files, shared files, synced or iCloud paths, virtual machine bundles.
12. **Uninstall keeps personal data by default** — [plan 036](../plans/036-uninstall/readme.md).
13. **Exclude with one click**, protecting the folders above it too — [`ignore-list.md`](./ignore-list.md).
14. **Purgeable space and snapshots in their own honest category** —
    [`system-data-analyzer.md`](./system-data-analyzer.md).
15. **No double counting** of APFS clones, hard links and shared blobs.
16. **Live chart and a collector tray.**
17. **Age heatmap** on the treemap.
18. **Fast rescans** from saved snapshots and a hash cache.
19. **Duplicates keep a protected copy** and re-verify content before acting —
    [`duplicate-finder.md`](./duplicate-finder.md).
20. **Shrink instead of delete** — APFS compression of rarely-read folders, clones for duplicates.
21. **Dev artifacts by age and by installed toolchains** — [`stale-project-cleanup.md`](./stale-project-cleanup.md).
22. **Warn on `node_modules` that installed apps need.**
23. ~~**Watch the Trash for apps**~~ — rejected: it needs a background service ([`../goal.md`](../goal.md), section 8).
24. **No fake urgency**, written into the principles. Built by plan [039](../plans/039-agreement-and-wording/readme.md): a short agreement before the first cleanup, a calm danger zone in Review, a tick of its own for High items, help for "removed something you need", a writing guide and a check over every text.
25. **Empty a cache folder, keep the folder.**
26. **Hard-blocked user folders** — the home folder and its standard folders can never be removed as a whole, a guard
    below every rule.
27. **Demo mode and automatic screenshots** — made-up data for recordings, tests and an always-current README. Recordings are built (plan [038](../plans/038-demo-recordings/readme.md)); automatic screenshots stay an idea. The README's category table is generated from `docs/categories/` and checked (plan [040](../plans/040-readme/readme.md)); its picture comes from plan 038's recordings.
28. **"Safe, but unticked"** — safe categories whose rebuild costs time are offered, not pre-selected (the Metal shader
    cache on macOS).
29. **A read-only scan report** the user can attach to a bug report.
30. ~~**Optional low-space alert**~~ — rejected: no background work of any kind (the user, 2026-11-15;
    [`../goal.md`](../goal.md), section 8).

No popular open-source manager of login and launch items exists — a gap
([`startup-and-login-items-manager.md`](./startup-and-login-items-manager.md)).

## What users complain about — and our answer

| Complaint seen in the field                                                    | Our answer                                                                           |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ |
| Real user data deleted as "cache" (an AI tool's memory, apps' licence keys)    | unknown is not junk; negative tests per rule; app data and `HTTPStorages` never Safe |
| Over-eager uninstall matching (shared browser profiles, files matched by name) | match by bundle and team identifier only, never by a loose name; shared folders kept |
| Valid launch agents deleted as "orphaned"; a VPN broken after an uninstall     | an orphan only when its program is gone; system and network extensions never offered |
| Permanent delete where the Trash was expected                                  | Trash first, always; permanent delete is a separate explicit choice                  |
| The Trash emptied automatically                                                | the app never empties the Trash                                                      |
| Deletes that followed a symlink                                                | symlinks never followed; re-resolved right before removal                            |
| Promised purgeable space that could not be freed                               | purgeable shown as information, never as cleanable                                   |
| Scans hanging on cloud and network mounts                                      | cloud and network paths skipped or time-limited; every scan cancellable              |
| Fake download sites pushing malware                                            | one official download location; signed and notarized builds                          |
| A single maintainer burning out                                                | open governance, documented decisions, contributors welcome                          |
| Opaque "System Data", alarm counters, "speed up" claims                        | no "Other", plain-English categories, no fake urgency                                |

## Open questions

- "How it comes back" as a second axis next to the danger level, or part of the category's description?
- A public, versioned rules library others can contribute to?
- "Shrink instead of delete" (APFS compression) — in scope?
- ~~Optional background checks~~ — decided 2026-11-15: no. Everything must be transparent; the app does only what the
  user asks, while it is open ([`../goal.md`](../goal.md), section 8).
