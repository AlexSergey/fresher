# 007/02/04 — The command

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** repo

## Files

- `scripts/sandbox-dev.ts` (new) — `npm run dev:sandbox -- <feature|path> [--real-trash]`: a feature name means
  `sandbox/<feature>/`, generated first with `npm run sandbox -- <feature>` when it is missing; a path is used as it
  is. Then `tauri dev --config app/tauri/tauri.dev.conf.json` with `FRESHER_SANDBOX` (absolute) and, for
  `--real-trash`, `FRESHER_SANDBOX_TRASH=real`. It refuses an unknown flag; the app itself checks the folder.
- `scripts/sandbox-dev.test.ts` — its argument handling.
- `package.json` — `"dev:sandbox": "node scripts/sandbox-dev.ts"`.

## Produces

The one command a developer or an agent types to open the app on a feature's cases.

## Checks

- Negative: `an_unknown_flag_is_refused`, `no_argument_is_refused`.
- Positive: `a_feature_name_points_into_the_sandbox_folder`, `real_trash_sets_its_variable`.
- `npm run test:scripts`; by hand: `npm run dev:sandbox -- trial` opens the window with the sandbox banner.

## Conclusions

`npm run dev:sandbox -- <feature|path>` generates a missing sandbox, sets `FRESHER_SANDBOX` (absolute) and the fake Trash,
and starts the development build; `--real-trash` uses the real one. Review findings: a feature name is plain (no `.`, `..`),
a relative path resolves against `INIT_CWD`, and an ambient trash variable is unset without `--real-trash`. Final review: the
fake Trash never replaces an earlier session's item (it persists in the sandbox's home), and the docs say `dev:sandbox` and
`npm run dev` share one identifier (a running window takes the second launch).
