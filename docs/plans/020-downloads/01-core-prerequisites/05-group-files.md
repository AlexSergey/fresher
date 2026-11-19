# 020/01/05 — The group Your files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, app/ui

Downloads is neither what macOS keeps for itself, nor what removed apps leave, nor development: a group for the
user's own files, after Apps. The large-files and duplicate finders belong there later. The design:
[`../00-design.md`](../00-design.md), section 1.5 (agent's call 9).

## Files

- `crates/core/src/group.rs` — `ModuleGroup::Files` between `Apps` and `Development` (the variants' order is the
  screen's), doc "Files you downloaded or keep yourself: old installers, big downloads."; `ALL` gains it (five);
  `as_str` → `"files"`.
- `crates/core/locales/en.json` — `core.group.files.name` "Your files", `core.group.files.description` "Files you
  downloaded or keep yourself, such as old installers and big downloads."
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`ModuleGroup` gains `"files"`), never edited by hand.
- `app/ui/src/product/module-groups.ts` — `GROUP_ORDER` `['system', 'apps', 'files', 'development', 'debug']`.

## Produces

`ModuleGroup::Files`, for 03/01 (the module's group) and 04/03 (the sidebar).

## Checks

- `group.rs`: the existing tests updated (`ALL` in order, every group's keys exist in the core's dictionary); the
  negative test still refuses `"downloads"` as a group name.
- `app/ui/src/product/module-groups.test.ts` (or the existing tests of `inGroups`): negatives first — an unknown group
  still comes last; then a `files` module sits after `apps` and before `development`.
- The bindings test (`app/tauri/tests/bindings.rs`) passes after the regeneration.
- `cargo nextest run -p fresher-core -p fresher-app`; `npx vitest run app/ui/src/product/module-groups.test.ts`; clippy;
  fmt; `npm run lint`.

## Conclusions

`ModuleGroup::Files` sits between `Apps` and `Development`, with its two texts. The engine registry's group check
lists it too (an exhaustive match, no behaviour change). The bindings are regenerated.
