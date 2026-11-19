# 006/05/03 — Docs and conclusions

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs

The version is **not** moved: the user's decision of 2026-11-16 keeps it at 0.1.0 while the app is built
([`deferred/release-versioning.md`](../../../deferred/release-versioning.md); agent's call 32 supersedes call 18). The
version files (`Cargo.toml` of the workspace, `app/tauri/tauri.conf.json`, both `package.json`) stay as they are.

## What exists already

- `docs/architecture/README.md` — the ports table has `AccessProbe` ("plan 006 (`fresher-macos`); `Unknown` and
  `NotAsked` until then") and `SystemUi` ("plan 006 (`/usr/bin/open`); `Unsupported` until then") rows, which 03/03
  points at `MacAccess` and `MacSystemUi` in `fresher-system`. There is no `docs/architecture/permissions.md`.
- `docs/architecture/modules.md` — names `check_removals_matched` (05/01); not `with_needs` or `with_tool_check`.
- `docs/restrictions/privacy.md`, "Least permissions" — "Enforced by: the shell (planned)".
- `docs/restrictions/deletion-safety.md` — no rule about administrator rights.
- `docs/testing.md` — the list of fakes has no `FakeAccess` or `FakeSystemUi` (both in
  `crates/testing/src/fake_access.rs`).
- `docs/hacks/README.md` — after 03/03, one access entry: "Sandbox mode avoids the places that prompt".
- `docs/deferred/app-leftovers-containers.md` — re-dated by 03/05.

## Files

- `docs/architecture/permissions.md` (new) — the capability model in short (capabilities, states, needs,
  availability, coverage, removability); where the macOS knowledge lives (`fresher-macos::privacy`: the table of
  guarded places, the checks, the panes) and the adapters (`fresher-system`: `MacAccess`, `MacSystemUi`); the rule
  "checks never prompt; a prompt only inside a scan the user started, for a place the user opted into"; how a module
  declares a need (`CategoryDescriptor::with_needs`, `ModuleDescriptor::with_tool_check`, the `app-leftovers`
  example); that the table of places came from documentation and what the user's run of plan 01 confirmed or changed;
  and **developer builds**: grants follow the code signature, an ad-hoc rebuild loses them, `npm run tauri dev` is
  attributed to the terminal app, the local signing identity of plan 013/01, sandbox mode's stand-in access, and how to
  test each state (the gallery and `FakeAccess` first; a real grant by hand, reset with
  `tccutil reset All <bundle id>` on a development Mac only: `io.github.alexsergey.fresher.dev` for `npm run dev`,
  `io.github.alexsergey.fresher` for a built app).
- `docs/architecture/README.md` — a link to `permissions.md`.
- `docs/architecture/modules.md` — the descriptor's `with_needs` and `with_tool_check`, and one line in the checklist
  ("every category that cannot see everything without a permission declares it").
- `docs/restrictions/privacy.md` — "Least permissions": enforced by the engine's coverage step and its tests, the
  `Info.plist` test, and review; no longer "planned".
- `docs/restrictions/deletion-safety.md` — a rule "No administrator rights": the app never elevates; an item it may not
  remove is shown, never selected; enforced by `Removability`, the plan builder's `NotRemovable` skip, the `Info.plist`
  test (no privileged helper) and review.
- `docs/testing.md` — `FakeAccess` and `FakeSystemUi` in the list of fakes.
- `docs/plans/006-permissions/readme.md` and `00-design.md` — the conclusions: what was built, what was measured and
  checked by hand (or that the user's run of plan 01 is still open), deviations as agent's calls; every plan and step
  status updated.

## Produces

The written contract every later module follows; the plan's conclusions for the user to look at.

## Checks

- `npm run pre-push` passes on the branch's last commit; coverage ≥ 90 % in the Rust crates and the UI.
- Every link in the changed docs resolves; the docs name no other product (clean room).
- The checks by hand of the global plan's "Acceptance" are done on a real Mac by the user and written in the
  conclusions, or listed there as open with plan 01's run.
- No version file changed.
- `cargo clean` only when the global plan is done ([`../../../workflow.md`](../../../workflow.md)).

## Conclusions

- Written as the step lists: `docs/architecture/permissions.md` (new: the model in short, where the macOS knowledge
  lives, the rule that a check never prompts, how a module declares a need, the state of the table of places, administrator
  rights, developer builds and how to test each state); a link to it and to the adapters in `docs/architecture/README.md`;
  `with_needs`, `with_tool_check` and one checklist line in `docs/architecture/modules.md`; "Least permissions" in
  `docs/restrictions/privacy.md` no longer "planned"; the rule "No administrator rights" in
  `docs/restrictions/deletion-safety.md`; `FakeAccess` and `FakeSystemUi` in `docs/testing.md`.
- Already true when this step started and checked, not redone: the `AccessProbe` and `SystemUi` rows of the ports table
  point at `MacAccess` and `MacSystemUi` (03/03), and the hacks list has its one access entry.
- Small extra: the `reveal_finding` sentence of `docs/architecture/README.md` was rewrapped to the doc's width.
- Decision: `permissions.md` says plainly that the table of places, the Full Disk Access candidates and the pane
  addresses are from documentation and that the user's run of plan 01 is still open; it does not claim a measurement.
  Where the run disagrees, the three constants and that page change together.
- Decision: the status of plan 03 (the macOS adapter) was moved to done with all its steps; plan 01 stays approved until
  the user's run.
- No version file changed: the version stays 0.1.0 (the user's decision of 2026-11-16).
- Not run here: `npm run pre-push` and `cargo clean` belong to the end of the global plan, after the user has looked at
  it. The by-hand checks are listed as open in the global readme, "By hand".
