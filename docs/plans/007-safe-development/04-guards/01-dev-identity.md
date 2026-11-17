# 007/04/01 — The development identity

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, repo

## Files

- `app/tauri/tauri.dev.conf.json` (new) — merged over `tauri.conf.json` by `tauri dev --config`: the identifier
  `io.github.alexsergey.fresher.dev` and the window title "Fresher (development)". Its own data folder, single-instance
  lock and permission grants follow from the identifier.
- `package.json` — `"dev": "tauri dev --config app/tauri/tauri.dev.conf.json"`.
- `app/tauri/src/lib.rs` — in a debug build the setup refuses to start when the identifier does not end in `.dev`
  ("a development build must run with tauri.dev.conf.json: npm run dev"), so a forgotten flag never opens the release's
  data folder. A release build has no such check.
- `docs/architecture/README.md` ("Data") — where a development build keeps its data.

## Produces

The configuration that `npm run dev:sandbox` (02/04) passes too.

## Checks

- Negative: `a_debug_build_with_the_release_identifier_is_refused` (the identifier check as a pure function).
- Positive: `the_development_identifier_is_accepted`; by hand: `npm run dev` starts, its data folder is
  `~/Library/Application Support/io.github.alexsergey.fresher.dev`, and a release build's folder is not created or
  changed.
- `cargo nextest run -p fresher-app`.

## Conclusions

`tauri.dev.conf.json` gives the development build its own identifier (`...fresher.dev`), title and data folder. Its window
block repeats the release one, so a drift test (added with 04/02) compares them. Final review: the generator also protects the
development identifier's data folder.
