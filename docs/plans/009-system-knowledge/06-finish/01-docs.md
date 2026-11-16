# 009/06/01 — The docs

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** docs

## Files

- `docs/architecture/README.md` — the crate `fresher-system`, the new ports and their fakes, the facts per scan.
- `docs/architecture/modules.md` — `Pattern::Under` and known folders, `cx.system()`, the cancel token.
- `docs/architecture/removal.md` — check 10 with app ownership.
- `docs/known-issues/cleaner-other-users-processes.md` — the first option done.
- `docs/deferred/` — the volume facts (plan 012), the project's last activity.
- `docs/dependencies.md` — only if a dependency changed.

## Checks

The docs name what the code does; no product is named.

## Conclusions

- `architecture/README.md`: the crate `fresher-system` in the table, the new ports and fakes, the system's facts per
  scan in the layout rules, `Clock` and `CommandRunner` now in `fresher-system`.
- `architecture/modules.md`: known folders and `Pattern::Under` (and the rule format's `under`), the cancel token for
  tools, a section on `cx.system()`.
- `architecture/removal.md`: check 10 with app ownership, the threat table's row.
- `known-issues/cleaner-other-users-processes.md`: narrowed, its first option done.
- `deferred/`: the volume facts (plan 012) and a project's precise last activity, written with the plan (decisions 3
  and 6).
- `dependencies.md`: `fresher-system` added to the users of `tempfile`, `thiserror`, `tracing` and `proptest`; no new
  dependency.
- The design's section 5 and step 05/01 say how the artifacts are found as built.
