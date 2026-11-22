# 041 — The security audit

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** every crate and module, app/tauri, app/ui, scripts, repo, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved. **Taken up after every feature of the first release is
implemented** (the user, 2026-11-22); until then it only collects what to look at.

## Intent

The user's task (translated): "One more separate plan — your independent security audit. A draft for now; we come back
to this plan after all the features are implemented."

The app reads the whole disk and deletes from it. Every safety rule so far was checked by the people and agents who
wrote the code it guards. Before strangers run it, someone who did not write it must try to break it.

## Goals

1. Every promise of [`restrictions/`](../../restrictions/README.md) is attacked, not re-read: a finding is a
   reproduction, not an opinion.
2. Nothing found stays in chat: each finding is fixed, or written down as a known issue with its reason.
3. The attacks stay in the repository as tests, so the next change cannot quietly reopen them.
4. The result is published: what was looked at, how, what was found, what was not looked at.

## Non-goals

- A certificate or a third party's seal: this is an agent's audit, and the report says so.
- New features, refactors, style: only what a finding needs.
- The security of macOS, of the tools the app reads, or of the user's own Mac.

## Design

### Independence

The audit is run by agents started fresh, with no memory of how the code was built and no access to the plans'
reasoning — only the code, the restrictions and the threat model. One agent attacks, another checks each finding by
reproducing it, a third reads the fix. The session that built a part never audits that part. A finding the checker
cannot reproduce is recorded as unconfirmed, not dropped.

### The threat model (written first)

Who could hurt the user through this app, and what they control:

| Who                                   | Controls                                                    | Wants                                              |
| ------------------------------------- | ----------------------------------------------------------- | -------------------------------------------------- |
| Another program on the Mac            | files, links and folder names under the home, timing        | the app deleting something else, or reading for it |
| A hostile file tree (a cloned project, an archive, a mounted volume) | names, links, sizes, the content the modules parse | a wrong claim, a crash, a walk out of the root |
| A tampered data folder                | the app's own settings, cache and cleanup journal           | a put back or a cleanup aimed at a path of its choice |
| A dependency                          | code inside the app                                         | the network, or the disk beyond what it is given   |
| A bug of our own                      | nothing — it is already inside                              | —                                                  |

### What is attacked

- **Removal**: links and hard links swapped between the scan, the confirmation and the act; a parent replaced by a
  link; a path that leaves its root through `..`, a firmlink or a mount; the protected list from every side; an item
  of several paths with one part swapped; put back writing where the journal says.
- **The scan and the rules**: every claim's proof — a folder dressed as a leftover, a bundle with a forged identifier,
  a name with control characters or of enormous length, a tree deep or wide enough to exhaust memory.
- **What the modules parse**: property lists, manifests, tool versions, pins — truncated, oversized, recursive,
  wrongly encoded; the small-file reader's limit.
- **External tools**: which program actually runs when the app calls one (the search path, a look-alike in a project
  folder), what reaches its arguments, what the app believes from its answer.
- **The shell**: every command the interface can call, with arguments the interface would never send; the
  capabilities, the content policy, the private API in use; whether a page could ever load anything foreign.
- **Development-only code**: the sandbox mode, the demo player, the run record, the dry run — proved absent from a
  release build, and proved unable to be switched on by an environment variable there.
- **Privacy**: the release binary and its dependency tree searched for anything that can open a connection; what is
  written to logs, records and reports, checked for paths and contents that should not be there.
- **The supply chain and the release**: locked versions, advisories (the open ones in
  [known issues](../../known-issues/README.md)), build scripts and procedural macros that run at build time, the
  signing and the hardened runtime's entitlements, the workflow's secrets and permissions.
- **The data folder**: every file the app reads back, edited by hand to lie.

### How

1. **Read** each area against its restriction, listing where the code enforces it and where it only assumes it.
2. **Attack sandboxes**: adversarial case files, `fixtures/sandboxes/attack-<area>.toml`, each case a negative — the
   attack must end with nothing removed and a reason shown. What a case file cannot express (a swap in the middle of
   an act) becomes a test with a hook at the exact moment, in the cleaner's own test suite.
3. **Property and fuzz tests** for every parser and for path handling, kept in the repository and run on a budget.
4. **The release build itself**: the audit's last pass runs on the signed build, not on a debug one.

### What comes out

- `docs/security/`: the threat model, the audit report (scope, method, findings by severity with their state, what
  was not covered), and how to report a problem.
- Each finding: a failing test first, then the fix, then the checker's confirmation. One that is accepted instead goes
  to [`known-issues/`](../../known-issues/README.md) with what was tried.
- Critical and high findings block the release; the plan is done when none is open.
- The restrictions' "enforced by" lines are corrected wherever the audit found them wrong.

## Plans (after approval)

| #   | Plan         | What                                                                |
| --- | ------------ | ------------------------------------------------------------------- |
| 01  | Threat model | the table above in full, the scope, the severity scale              |
| 02  | Removal      | the cleaner, the journal, put back, the protected list              |
| 03  | Scan and parsing | the walk, the modules' proofs, every parser, external tools     |
| 04  | Shell and privacy | commands, capabilities, development-only code, the network     |
| 05  | Supply chain and release | dependencies, the build, signing, the workflow          |
| 06  | Fixes        | a test, a fix and a confirmation per finding                        |
| 07  | Report       | `docs/security/`, the corrected restrictions, the second pass       |

02–05 are independent and run in parallel. Depends on every plan of the first release being built, and on plan
[013](../013-release/readme.md) for 05 and the last pass.

## Ideas this touches

"Fail closed" and "allowed roots and a re-check right before removal" of
[best practices](../../ideas/best-practices.md), 6 and 7; the safety section of plan [040](../040-readme/readme.md)
links to the report.

## Open questions

None now. When the plan is taken up: whether to also invite outside readers (a public call for review of the cleaner
before the release) — it costs nothing but time, and names the project in public before it ships.

## Decisions

1. **The audit waits for the features** (the user, 2026-11-22): this draft is returned to after everything planned for
   the first release is implemented.

## Agent's calls (to check)

1. **Fresh agents, split into attacker, checker and fix reader**; the builder of a part never audits it.
2. **A finding is a reproduction** kept as a test.
3. **Critical and high findings block the release.**
4. **The report is public and says what it is**: an agent's audit, with its limits.
5. **The last pass runs on the release build.**

## Conclusions

Not written yet.
