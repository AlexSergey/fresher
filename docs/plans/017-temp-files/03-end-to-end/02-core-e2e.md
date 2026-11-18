# 017/03/02 — The core e2e flows

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** e2e

## Files

- `crates/e2e/tests/temp.rs` (new) — the engine with the real scanner (its reader wrapped by the facts lens where a
  flow states a fact), the `temp` module, the real settings file and the real `DiskCleaner`, in a fixture sandbox
  whose two places lie beside its home (outside the scan root), with `FakeClock`, `FakeTrash`, `FakeRunningApps`,
  `FakeInstalledApps` and `FakeOpenFiles`. The nine flows of the design's section 9, "Core e2e", negatives first
  within each flow.
- `fixtures/homes/temp-flow.toml` (new) and `fixtures/golden/temp-flow.json` (new, reviewed).
- `crates/e2e/Cargo.toml` — `fresher-module-temp` as a dev-dependency.

Flow 2 (an unpacked folder) and flow 3 (a pipe and a socket) use the real disk only: `utimes` for the old
modification times, `mkfifo` for the pipe, a socket bound in the fixture's own temporary folder (its path within the
socket's 104 bytes, checked first); nothing is stated, so they prove what the lens cannot.

## Produces

The module's core flows, for the acceptance.

## Checks

- `npm run test:e2e:core` passes; the temp flows pass 150 runs in a row (`cargo nextest run -p fresher-e2e --test temp
  --retries 0` in a loop; a name filter `temp` selects nothing, since no test's name holds it), as plan 016's did.
- Flow 3 ends within its timeout: no open of a pipe blocks the scan or the guard.
- No flow writes outside its temporary folder (the e2e crate's guard).

## Conclusions

Built (495a417): `crates/e2e/tests/temp.rs` (13 tests over the nine flows, and the golden), its harness
`crates/e2e/tests/temp_home/` (a fixture sandbox opened with `fresher_sandbox::Sandbox::open`, the real scanner through
the facts lens, the real settings, cache and journal files, the real `DiskCleaner`), `fixtures/homes/temp-flow.toml` and
`fixtures/golden/temp-flow.json`. The temp flows passed 150 runs in a row (`--test temp`); `npm run test:e2e:core` 67
passed.

Changed from the design, each accepted by the review:

- The cleaner is `Reach::SandboxFolder`, as sandbox mode composes it: with `Reach::Sandbox` the marker would have to lie
  inside a place, where the module would match it. `docs/testing.md` says so since 04/01.
- Flow 5 follows decision 21: a restore emits nothing inside a place, so nothing is offered and nothing is left out (the
  module's `scan-again` row is a second guard); the design's section 9 was corrected in 04/01.
- Flow 4: a file rewritten by a new file renamed over it after the plan is skipped `Replaced`; there is no `Changed`
  reason, and `Grown` would need more than 64 MiB per run. A rewrite in place under 64 MiB still goes to the Trash, the
  residual already written in [`removal.md`](../../../architecture/removal.md).
- Flows 2 and 3 set their clocks from the real entries' status change plus one or eight days (read from the disk, never
  the wall clock), proving the age by status change both ways on real times, a real socket and a real pipe.
- The harness copies five helpers of the JetBrains one (deferred item, done by plan 018/03/02).
