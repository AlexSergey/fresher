# 004/01 — Measure first

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos (probes only)

Before any product code, throwaway probes answer the questions the design rests on: what the bulk read returns and
how fast, whether the change journal records what we need, how volumes, links, clones and privacy prompts behave. The
probes are small Rust binaries in `NO_COMMIT/probes/004/` (ignored by git, never committed); only their numbers reach
the repository, in step 04. The design: [`../00-design.md`](../00-design.md), sections 3, 4, 8 and 13.

Rules for the probes:

- **Read-only.** A probe writes only inside its own temporary folder (a generated tree, a test folder for the journal).
- **The developer's own home folder** may be read, read-only, for counts and timings; no name or path is written to a
  file or printed into the plan. Nothing is sent anywhere.
- No probe changes a permission or answers a system prompt for the user; a prompt that appears is the finding.
- The probes may use `unsafe` freely: they are scratch work, and step 03 of plan 03 rewrites what survives behind the
  `ffi` module with `// SAFETY:` comments.

## Steps

| #   | Step                                                             | Status   | Shared files                 |
| --- | ---------------------------------------------------------------- | -------- | ---------------------------- |
| 01  | [The bulk read](./01-bulk-read.md)                               | ✔️ done    | —                            |
| 02  | [The change journal](./02-change-journal.md)                     | ✔️ done    | —                            |
| 03  | [Volumes, links, clones and prompts](./03-volumes-links-prompts.md) | ✔️ done    | —                         |
| 04  | [The numbers into the design](./04-numbers-into-the-design.md)   | ✔️ done    | `docs/plans/004-fast-scan/`  |

Steps 01–03 are independent and may run in parallel (separate probe folders); 04 waits for all three.

## Conclusions

Done on 2026-11-15, on one machine (Apple M3 Pro, 12 cores, internal SSD, APFS, macOS 27.2.1), from a terminal
without Full Disk Access. The numbers are in the design, section "Measured (01)"; the bars of section 13 are set from
them; each agent's call 01 was to verify carries its outcome.

- **The plan goes on.** The change journal records a file growing in place (append, overwrite, `truncate`) as an
  event for its folder — the case the product exists to find.
- **Confirmed**: the journal for rescans (call 1), the 1 MiB folding threshold (2), the `Mutex` queue (13), open by
  path with an identity check (14), the IO policy (15), `EPERM`/`EACCES` (16), the sampled check (25), which turns
  out load-bearing: a position older than the journal gives no flag.
- **Changed**: the thread count — one per core, at most 16, not 3 × the cores (more threads were slower); the private
  size is read only for files flagged as clones (asking it for every file doubled the walk); small hard-linked and
  cloned files are folded, their identity kept in the link table (keeping them as records doubled the nodes); a path
  past `PATH_MAX` is opened in steps; a folder removed after its open is caught by its link count; reading the
  journal is bounded by the position's age and a time budget (one developer day of history took 77 s to read,
  against a 22 s full scan of the same home).
- **Asked**: compressing the cache (call 8) — deflate level 1 saves 60 %; open question 1 of the design.
- **Not answered from a terminal** (plan 006/01 runs them by hand from Finder): which home folders raise a prompt for
  a fresh app, cloud (dataless) files, removable and network volumes, the journal with Full Disk Access and after a
  sleep, and every macOS version from 13 to 26.
- The probes stay in `NO_COMMIT/probes/004/` (bulk read, journal, volumes), with their `RESULTS.md`.
