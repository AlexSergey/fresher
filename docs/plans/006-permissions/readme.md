# 006 — Permissions and privileges

**Status:** 📝 draft · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** core, macos, engine, cleaner, app

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "…and for privileges and permissions, since not every action is possible without
permissions."

## Goals

1. The app always knows what it may see and do, and says so honestly: reduced coverage is shown, never hidden.
2. A permission is asked for only when the user starts something that needs it, with a plain-English reason and what it
   unlocks ([`restrictions/privacy.md`](../../restrictions/privacy.md), "Least permissions").
3. Everything works without any permission, with less coverage.
4. Modules declare what they need; none of them talks to macOS about permissions.

## Non-goals

- Running anything as administrator (see "Privileges").
- Signing and notarization: the release plan. This plan only records what they must provide.

## Design

### What macOS guards

| Permission                    | What it unlocks                                                        | How it is obtained                                      |
| ----------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------- |
| none                          | most of the home folder, `~/Library/Caches`, developer folders         | —                                                       |
| Files and Folders             | Desktop, Documents, Downloads, removable and network volumes           | a system prompt on first access, one per place          |
| Other apps' data              | the containers of other apps                                           | a system prompt on first access                         |
| Full Disk Access              | all of the above, plus Mail, Messages, Safari and similar protected data | only by the user, in System Settings                  |
| App Management                | changing or removing other apps' bundles                               | a system prompt, or System Settings                     |
| Administrator                 | items owned by the system or another user                              | never taken by the app                                  |

Each system prompt shows a reason string from the app's `Info.plist`; they are written in plain English in this plan.
The exact behaviour of each prompt on macOS 13 to the current version is checked in the first step.

### Capabilities

- The core has one enum, `Capability` (`FullDiskAccess`, `DesktopFolder`, …, `AppManagement`, `DockerCli`), and one
  port, `AccessService`: the state of a capability is `Granted`, `Denied`, `NotAsked` or `NotApplicable`.
- A module's descriptor says which capability each of its categories and actions needs, as required or as "sees more
  with". The engine combines that with the current state; modules never check anything themselves.
- `fresher-macos` implements the port. Checks are read-only and **never cause a prompt**: Full Disk Access is detected
  by trying to read a file only it allows; the per-folder state is learnt from the scan's own errors, not by poking the
  folder.
- The state is re-checked when the window gains focus — the user may grant something in System Settings while the app is
  open — and announced with `AccessChanged`.
- An external tool counts as a capability too: Docker's sections need the Docker CLI and a running Docker.

### What the user sees

- **No wall of prompts at first launch.** A scan starts with what is already allowed.
- When a scan could not see something, the result says what and why: "312 folders could not be read. Full Disk Access
  lets Mac Fresher see them." — with a button that opens the right pane of System Settings and a short "how".
- A section or action that lacks its capability is visible but disabled, with one sentence on what is missing. Nothing
  is silently absent.
- Sizes affected by an unreadable folder are partial (`42 GB+`).
- The Permissions group of [settings](../005-settings/readme.md) lists every capability, its state and what it unlocks.
- No nagging: a reminder appears only next to the result it limits.

### Privileges

- **The app never runs as administrator**: no privileged helper, no password prompt, no `sudo`. A helper would be a
  background component, against [`goal.md`](../../goal.md), section 8, and a root-level deleter is the largest risk the
  product could carry.
- An item that needs administrator rights is shown with its size and marked "needs an administrator"; the app gives the
  exact Finder step or command for the user to run themselves. It is never selected for cleaning.
- The app is not sandboxed (a sandboxed app cannot scan the disk), so it is distributed outside the Mac App Store,
  signed and notarized.
- Grants are tied to the app's signature: a stable signing identity is needed for releases, and development builds lose
  their grants on rebuild — recorded in the developer docs.

### Before a deletion

The cleaner checks the capability for every item right before acting, next to its other re-checks
([`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)). An item it may not remove is skipped and
reported with the reason; it never retries with more rights.

### Testing

`AccessService` has a fake in `fresher-testing`: every state of every capability is a test case, and the UI states are
in the gallery. No test asks macOS for anything.

## Plans (after approval)

| #   | Plan                  | What                                                                              |
| --- | --------------------- | --------------------------------------------------------------------------------- |
| 01  | Probe the system      | throwaway probes in `NO_COMMIT/`: each prompt and check on the supported macOS versions |
| 02  | Capability model      | the enum, the port, the fake, module declarations, the engine's combination       |
| 03  | macOS implementation  | the checks, the focus re-check, opening System Settings, the reason strings       |
| 04  | UI                    | the notices, disabled states, the Permissions group, the first-run explanation    |
| 05  | Cleaner preflight     | the per-item check, "needs an administrator" items, the developer docs            |

Depends on plan 002; the scan of plan [004](../004-fast-scan/readme.md) feeds it its errors. It can run in parallel with
plans 003 and 004.

## Ideas this touches

[macOS specifics](../../ideas/architecture/macos.md) (proposed decision 12, Full Disk Access by probing),
[scanning](../../ideas/architecture/scanning.md) (decision 11, errors by kind), the
[smart uninstaller](../../ideas/smart-uninstaller.md) (App Management), the
[external drive analyzer](../../ideas/external-drive-analyzer.md) (removable and network volumes), and "the exact
command the user can run" from [best practices](../../ideas/best-practices.md).

## Decisions

None yet.

## Agent's calls (to check)

1. **No administrator rights, ever**: such items are shown with instructions, not cleaned.
2. **No prompts at first launch**; a permission is offered only beside the result it limits.
3. **Checks never trigger a prompt.**
4. **Capabilities are declared by modules and resolved by the engine.**
5. **External tools (the Docker CLI) are capabilities too.**
6. **Not sandboxed; outside the Mac App Store.**
7. **Target version 0.4.0.**

## Conclusions

Not written yet.
