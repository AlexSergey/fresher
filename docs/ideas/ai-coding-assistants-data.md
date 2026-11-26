# AI coding assistants' data

**Since:** 2026-11-26 · From the first tester's feedback on 0.1.0, taken up by the user

## Problem

AI coding assistants that run in the terminal (Claude Code, Codex and the like) keep their own data in the home folder
and grow it with every session: conversation transcripts, snapshots of the files they edited, shell snapshots, caches
and logs. They also leave behind the scratch projects they were asked to create, which nobody opens again.

## Direction

- One module for the assistants' own folders (for example `~/.claude/` and `~/.codex/`), one category per kind of
  data, each with its danger level. Old sessions' transcripts and file snapshots are the bulk; settings, memory,
  skills, plugins and anything the user wrote are never offered.
- Age by the session's last write, the threshold in Settings like the stale projects' days; a session of the last days
  is left out with its reason.
- The folders' layout must be checked on a real install of each tool and pinned in fixtures; an unknown file is not
  claimed (no "Other").
- While the assistant runs, its folders are left out, as Xcode's are while Xcode is open.
- The scratch projects they leave are projects like any other: covered by
  [Stale project cleanup](./stale-project-cleanup.md) for their regenerable parts and by
  [Big projects left alone](./big-stale-projects.md) for the folder itself.

Open questions:

- Are old transcripts "Usually safe" or "Look before removing"? Some users keep them as a record of the work.
- Does a tool offer its own cleanup command or setting (a retention period) to prefer over deleting files?
