# 006/01/01 — Prompts, refusals and the Full Disk Access check

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** NO_COMMIT, docs

## Files

- `NO_COMMIT/006-probes/` — a small Rust binary (its own `Cargo.toml`, outside the workspace), ad-hoc signed with a
  fixed bundle id, run from Finder (not from a terminal, so macOS attributes access to the probe itself). For each
  candidate place it opens the folder read-only, lists nothing, closes it, and prints the `errno` it got and whether a
  prompt appeared (noted by hand).
- `docs/plans/006-permissions/00-design.md` — "Probe results": the table below, filled in.

## What is measured

On the current macOS (the development Mac), with Full Disk Access off, then on; the same on macOS 13 if a virtual
machine of it is available, otherwise marked "from Apple's documentation, unverified":

| Question                                                                                                  | Answer recorded as                    |
| --------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| Which places under the home folder prompt on first access: Desktop, Documents, Downloads, `~/Library/Containers/<other app>`, `~/Library/Group Containers/<other group>`, iCloud Drive (`~/Library/Mobile Documents`), the Photos library, `~/Library/Mail`, `~/Library/Safari`, `~/Library/Messages`, the Trash | prompts / refuses with `EPERM` / opens |
| Does listing `~/Library/Containers` itself prompt, or only entering one container?                        | the level that prompts                |
| A removable volume and a network volume as a scan root                                                    | prompts / opens                       |
| Which file or folder tells Full Disk Access without a prompt and exists on every Mac (the system's privacy database under `/Library/Application Support/com.apple.TCC/`, `~/Library/Safari`, …) | the ordered candidate list            |
| After denying a prompt: is the answer remembered, and is the next access a silent `EPERM`?                | yes / no                              |
| After turning Full Disk Access on in System Settings while the probe runs: does the running process see it, or only after "Quit & Reopen"? | live / after relaunch |
| Is `EPERM` (privacy) always told apart from `EACCES` (POSIX permissions) on a folder?                     | yes / the exceptions                  |
| Does any guarded place need an FFI call to check (anything beyond `open`)?                                | none / the call                       |

## Produces

The guarded-place table that 03/01 encodes, the Full Disk Access candidates, and whether "Reopen Fresher" is needed
(design, section 4). If an answer contradicts the design, the design is changed in the same commit and the change is
listed as a new agent's call.

## Checks

- "Probe results" has an answer for every row above, each marked measured (with the macOS version) or unverified.
- No probe file is staged (`git status` shows nothing under `NO_COMMIT/`).
- The probe reads no file contents: it opens and closes only.
