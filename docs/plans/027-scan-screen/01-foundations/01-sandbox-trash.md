# 027/01/01 — The sandbox's Trash

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** core, app/tauri, app/ui, e2e

In sandbox mode the banner says where the sandbox's cleanups go, and a finished cleanup to the Trash says it again
(feedback item 18). The design: [`../00-design.md`](../00-design.md), sections 7 and 8 (agent's calls 1 and 2).

## Files

- `crates/core/src/app_info.rs` —
  - `Build::Sandbox { path, simulated, trash: Text }` (`trash`: "where its cleanups move items, as the sentence the
    banner and a cleanup's result show"); serde stays `kebab-case` tag with `camelCase` fields.
  - `pub enum SandboxTrashAt<'a> { Sandbox(&'a Path), System }` with `pub fn text(&self) -> Text`:
    `core.build.trash.sandbox` with the arg `path` (`TextArg::Path`, the folder as given), or
    `core.build.trash.system` with no arg. Not serialised. Re-exported from `crates/core/src/lib.rs` beside `Build`.
- `crates/core/locales/en.json` — the six new keys and the changed description of
  [`../00-design.md`](../00-design.md), section 8 (zone names, trash sentences, `core.storage.part.your-folders
  .description`), kept in the file's sorted order.
- `crates/core/src/texts.rs` (tests) — `core_keys()`: the zone key of every level beside its name and meaning; the
  keys of `SandboxTrashAt::Sandbox(Path::new("/s/home/.Trash")).text()` and `SandboxTrashAt::System.text()`.
  `texts_with_args()`: the sandbox sentence. `names_every_danger_level` gains a sibling `names_every_zone`
  (`["Safe to remove", "Low risk", "Medium risk", "High risk"]`).
- `app/tauri/src/sandbox.rs` — `build(sandbox, trash: SandboxTrash)`: `SandboxTrash::Sandbox` gives
  `SandboxTrashAt::Sandbox(<the sandbox's Trash folder>)` — the same path the sandbox's Trash port is given, taken
  from the one function that builds it (extract one if the path is built inline; never a second `join(".Trash")`) —
  and `SandboxTrash::Real` gives `SandboxTrashAt::System`. Its caller (`compose.rs` or `sandbox::compose`) passes the
  mode's trash.
- `app/tauri/src/sandbox/tests*` and `app/tauri/src/commands.rs` (its `app_info` test) — the new field.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated
  (`UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`), never edited by hand.
- `app/ui/src/ipc/mock.ts`, `app/ui/e2e/core-mock.ts` and any fixture that builds a sandbox `Build` — the `trash`
  text (`{ key: 'core.build.trash.sandbox', args: { path: { kind: 'path', value: '<sandbox>/home/.Trash' } } }`).
- `app/ui/src/product/build-banner/build-banner.tsx` — the sandbox text gains ` · ` and `tText(build.trash)`: new
  `ui.build-banner.sandbox` "Sandbox · {path} · {trash}" and `ui.build-banner.sandbox-simulated`
  "Sandbox · {path} · {trash} · Simulated: {parts}." (the `trash` arg is the rendered core sentence). Its stories gain
  the field; `BuildBanner/sandbox` baseline regenerated.
- `app/ui/src/product/result-summary/result-summary.tsx` (+ CSS if needed, story `trash-in-sandbox`, test) —
  optional `trashNote?: string`, a line under the result's lines when `report.method === 'trash'` and the note is
  given; nothing otherwise.
- `app/ui/src/screens/review/cleanup-panel.tsx` — passes `trashNote` = `tText(info.build.trash)` when
  `useAppInfo()` says sandbox, else nothing.
- `app/ui/src/locales/en.json` — the two banner strings above.

## Produces

`Build::Sandbox.trash` (a core `Text`) for any later screen; `ResultSummary`'s `trashNote`.

## Checks

- Rust, negatives first:
  - `app_info.rs` `mod negative`: `a_build_outside_a_sandbox_serializes_no_path` stays and also asserts no `trash`;
    `mod positive`: `a_sandbox_build_serializes_its_trash_sentence` (key and `path` arg in the JSON);
    `the_systems_trash_has_no_path_arg`.
  - `sandbox` tests: `negative::the_real_trash_names_no_sandbox_path`; `positive::the_own_trash_names_its_folder`
    (the path equals the Trash port's folder).
  - `texts.rs`: the existing key, placeholder and unused-key tests pass with the new keys; `names_every_zone`.
- UI, negatives first: `build-banner.test.tsx` (new if absent): `describe('negative cases')` — a development and a
  release build show no trash line; `describe('positive cases')` — a sandbox build shows its folder and the core's
  sentence, with and without simulated parts. `result-summary` test: no note for a permanent cleanup or without
  `trashNote`; the note for a Trash cleanup with it. `cleanup-panel` covered through `review-screen.test.tsx` (a
  sandbox build's finished Trash cleanup shows the line).
- Commands (in this worktree):
  - `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p fresher-core -p fresher-app`
  - `UPDATE_BINDINGS=1 CARGO_TARGET_DIR=… cargo nextest run -p fresher-app --test bindings`, then the diff of
    `bindings.generated.ts` reviewed (one field).
  - `CARGO_TARGET_DIR=… cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --all --check`.
  - `npm run test --workspace app/ui -- src/product/build-banner src/product/result-summary src/screens/review`
  - `npm run lint:ts`; `npm run knip`; `npx eslint` and `npx prettier --check` on the changed files.
  - `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts visual.spec.ts` for the banner and result
    baselines (look at each changed PNG), then `npm run test:e2e --workspace app/ui -- gallery.spec.ts visual.spec.ts axe.spec.ts`.

## Conclusions

Not written yet.
