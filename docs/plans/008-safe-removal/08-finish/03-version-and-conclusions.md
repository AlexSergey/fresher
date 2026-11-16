# 008/08/03 — Version and conclusions

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** repo, docs

## Files

- The version files — `Cargo.toml` (`[workspace.package]`), `app/tauri/tauri.conf.json`, the root and `app/ui`
  `package.json` — to 0.5.0, unless another plan of the 0.5.0 group is still open (then the last of them moves it).
- `docs/plans/008-safe-removal/readme.md` and each plan's readme — the conclusions: what was built, what was measured
  (01's answers, the property tests' run of 10 000 cases, the open-files snapshot's time), the decisions taken on the
  way, the deviations from the design, the known issues, what is left to later plans (007's scenarios and panel,
  009's app ownership, 010's reading of the journal, 012's nodes).
- `docs/plans/README.md` — the plan's row: its status.
- `docs/ideas/README.md` and the ideas this plan became (cleanup preview and safe removal, cleanup journal) — marked
  as built by this plan.

## Produces

A plan ready for the user's look and the squash-merge into `main`.

## Checks

- `npm run pre-push` passes on the branch's last commit.
- The acceptance of the global plan, item by item, with its evidence in the conclusions.
- `cargo clean` after the merge ([`../../../workflow.md`](../../../workflow.md)).

## Conclusions

Done on 2026-11-16.

- **The version** moved from 0.3.0 (plan 004's; plan 006 never moved it, its 05/03 is not built) to 0.5.0 in
  `Cargo.toml` (`[workspace.package]`), `app/tauri/tauri.conf.json` and both `package.json`; only the version lines of
  `Cargo.lock` and `package-lock.json` changed (regenerated). No other plan of the 0.5.0 group is open; 006's 0.4.0
  target stays for its own rest.
- **The conclusions** are in the plan's readme and in each plan's readme; `docs/plans/README.md` marks 008 done and 006
  in progress; the two ideas this plan became link it.
- **The merge**: at the user's request of 2026-11-16, 008 and the built parts of plan 006 go into `main` together
  ([`../../../workflow.md`](../../../workflow.md)); the sandbox run is deferred (02).

