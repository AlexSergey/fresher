# A look-alike Docker bundle can swap the signed CLI between its check and its run

**Status:** open · **Updated:** 2026-11-25

## Symptom

The `docker` module runs a runtime's CLI only from a bundle with the runtime's id whose CLI passes
`/usr/bin/codesign --verify` against the developer's team. A same-user program can plant a look-alike bundle with
that id that holds a copy of, or a link to, the genuine signed CLI: the check passes, and the program swaps the file
or retargets the link after the check and before the run. Its own program then runs as Fresher's child, with
Fresher's Full Disk Access. It can retry at every scan. Found by the security audit of 2026-11-25 (SCAN-1, the fix
reader's residual).

## Cause

The signature covers the CLI file alone, not its bundle, and macOS has no way to run the exact file that was checked
(no exec of a held file): the check and the run are two steps on a path the attacker's folder holds.

## What was tried

- The first fix (plan 041): the developer's signature on the CLI. It stops a planted program, not a planted copy of
  the genuine one.
- Running the CLI only from the bundle of the running app: rejected, because the attacker can start its look-alike
  bundle too (its main program may be a copy of the genuine one), so a running bundle proves no more than an
  installed one.

Kept for 0.1.0 because it needs a same-user program that already runs and plants a bundle, the runtime's app
running and its socket present, and a won race; the read's command is fixed (`--host`, `--config` of an empty
system folder), so the genuine CLI itself is not steered.

**Accepted for 0.1.0 by the user, 2026-11-25.**

## Options left

- Read the engine's socket directly (`GET /system/df` over the local socket, parsed by the module): no program runs
  at all, so nothing can be swapped. Needs a small HTTP-over-socket reader and a new parser for the engine's JSON.
- Run the CLI only from a bundle whose every folder down to the CLI is owned by root and not writable by the user
  (a bundle installed by an administrator), and refuse a link anywhere on the path.
