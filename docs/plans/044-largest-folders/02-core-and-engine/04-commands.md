# 044/02/04 — `get_largest_folders` and `reveal_node`

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** engine, app/tauri, app/ui (bindings
only)

The two commands of Folders: the page and Show in Finder by id. The design: [`../00-design.md`](../00-design.md),
section 7; agent's call 12.

## Read first

02/03's commit; `crates/engine/src/reveal.rs` (`reveal_finding`, its tests and `scanned` rig);
`crates/engine/src/largest/mod.rs` (`committed_for_largest`, `home_in`, `Ground::is_gone`);
`app/tauri/src/{largest_commands.rs,lib.rs,commands.rs,apps_commands.rs,dev_runs/commands.rs}`, `app/tauri/build.rs`
(the allowlist), `app/tauri/tests/bindings.rs`; `docs/restrictions/privacy.md` (the UI names ids, never paths).

## Files

- `crates/engine/src/reveal.rs` — `Engine::reveal_node(&self, node: NodeId) -> Result<(), CommandError>`: the state
  from `committed_for_largest` (`NoScan`, `Busy`); `StaleNode` for a node of another state or none; `NotOwnItem` for
  the home, a path outside it (`home_in`) or one `is_gone`; then `ports.system_ui.reveal(&path)`, a failure
  `EngineError::SystemUi`. Its doc: Show in Finder of a Largest row; the UI names the id, the engine holds the path.
- `app/tauri/src/largest_commands.rs` — `get_largest_folders(engine, query: FoldersQuery) -> Result<FoldersPage,
  Refusal>` and `reveal_node(engine, node: NodeId) -> Result<(), Refusal>`; the module doc names Folders.
- `app/tauri/src/lib.rs` — both registered after `pick_own_item`; `app/tauri/build.rs` and
  `app/tauri/src/dev_runs/commands.rs` — both in the lists.
- `app/tauri/tests/bindings.rs` — `FoldersQuery`, `FoldersPage`, `FolderRow`, `LeftFor` declared;
  `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` rewrites `app/ui/src/ipc/bindings.generated.ts`.

## Interfaces produced

- The commands `get_largest_folders` and `reveal_node`, and their types in the bindings; 03/02 wraps them.

## Tests (negatives first)

`crates/engine/src/reveal.rs`:
- `mod negative`: `reveal_of_a_node_before_a_scan_is_refused`, `reveal_of_a_node_while_scanning_is_refused`,
  `reveal_of_a_node_of_another_state_is_refused`, `reveal_of_the_home_or_outside_it_is_refused`,
  `reveal_of_a_removed_path_is_refused`, `a_failed_reveal_of_a_node_is_reported` (each but the last: no
  `SystemUiCall`).
- `mod positive`: `show_in_finder_reveals_the_nodes_path`.

`app/tauri/src/largest_commands.rs`:
- `mod negative`: `no_scan_is_refused` and `a_stale_node_is_refused` also ask `get_largest_folders` and `reveal_node`.
- `mod positive`: `the_folders_list_answers_at_the_default_size` (`min_size` 1 GB, `days` 365, nothing listed in the
  trial home).

## Checks

- `cargo nextest run -p fresher-engine -p fresher-app`
- `bindings.generated.ts` holds the four types; `npm run lint:ts` (nothing in the UI uses them yet)
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet (04/01 lists the commands).

Commit: `feat(app): the Folders page and Show in Finder by node`.
