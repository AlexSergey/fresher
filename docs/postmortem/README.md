# Postmortem

Attempts that did not help, kept for history. Each entry names the problem, what was tried and the negative result, so
the next session does not repeat the same attempt.

## Format

One file per attempt, `<topic>.md`:

```markdown
# <Attempt>

## Problem

<The problem, with a link to the known issue or the plan.>

## What was tried

<The approach, the branch or commit if it exists.>

## Result

<Why it did not work: the error, the measurement, the side effect.>

## Lesson

<What to do instead, or what to check before trying again.>
```

## Entries

| Attempt                                                                    | Date       | Result                                      |
| -------------------------------------------------------------------------- | ---------- | ------------------------------------------- |
| [A Rust build cache shared by the worktrees](./shared-rust-build-cache.md) | 2026-11-15 | sccache gave no hits across `target/` paths |
