# Privacy

## No network

- **Rule:** the app makes no network requests: no telemetry, no analytics, no crash upload, no update check (until an
  update mechanism is agreed with the user).
- **Why:** the app sees the whole file system; nothing about it leaves the Mac.
- **Enforced by:** no HTTP, TLS, websocket or socket crate in the macOS dependency tree (`deny.toml` bans, by name);
  the built executable imports no socket, resolver or system networking call (`scripts/check-bundle.ts`, run by
  `npm run build`, so in CI's build job); the Tauri capabilities allow no network,
  the UI's Content Security Policy allows no remote origin and no form target (plan 001), and the window's navigation
  guard (`app/tauri/src/navigation.rs`) cancels any address but the app's own page (plan 041). The webview's own
  network is held by the CSP and the guard alone: no Rust check can see it.
- **Rule:** the Docker CLI the `docker` module runs never follows the user's own Docker configuration: it reads
  through the local socket the walk saw (`--host unix://<socket>`, never `--context`) with the system's always-empty,
  root-owned `/private/var/empty` as its config folder (`--config`), so no context, credential helper, hook or plugin
  folder of the user's can send it to a remote engine (`ssh://`, `tcp://`) or start another program through it.
- **Enforced by:** `ask` in `modules/docker/src/lib.rs`; the exact command is asserted by the module's tests
  (`the_read_names_the_seen_socket_and_an_empty_config_never_a_context`), its fixture tests and the e2e flow
  (`crates/e2e/tests/docker.rs`). The shown commands keep `--context`: the user runs them, not the app.

## Local data only

- **Rule:** scan results, the scan cache, history and settings are stored only in the app's own folder in
  `~/Library/Application Support/`. Logs never contain file contents. The webview keeps no data of its own.
- **Why:** paths alone already reveal a lot about a person.
- **Enforced by:** `fresher-store`, the only crate that touches the app's data folder (the app hands it the folder);
  the window runs `incognito`, so the webview writes no cookies, caches or storage to the disk (plan 002); review.
- **Rule:** a module may read the contents of a small text file its own matches showed (a pointer or a version file of
  a developer tool): at most 4 KiB, a regular file only, never through a link, read anew at each call; its bytes are
  never logged, stored in the scan cache or the data folder, or sent anywhere.
- **Enforced by:** the contract of `SystemFacts::small_file` (`SmallFiles`, one bounded no-follow read) and its tests;
  that a module reads only files its matches showed is a rule of review
  ([`../architecture/modules.md`](../architecture/modules.md)).
- **Rule, as used:** the `local-ai-models` module reads the manifests of Ollama's models this way (small JSON lists of
  layer names, at most 4 KiB each, never through a link, only files its own matches showed), once per scan in `finish`,
  to tell which layers models share; it never logs, keeps or sends their bytes: only the digests and sizes it parses
  are used, and a model's name comes from the file's path. Tests: `modules/local-ai-models/src/ollama/manifest/tests.rs` (the 4 KiB edge, a link, not
  text) and the fixture home `fixtures/homes/local-ai-models-ollama-long.toml` (a manifest one byte over the limit).
- **Rule:** a development build started by `npm run dev` writes a record of each run (real paths of the Mac, never file
  contents) to `NO_COMMIT/dev-runs/` of the repository, which git ignores; a release build has no such code.
- **Enforced by:** `#[cfg(debug_assertions)]` on `app/tauri/src/dev_runs/`, the variable set only by `scripts/dev.ts`,
  `.gitignore`, the tests of `compose.rs` (`a_release_build_ignores_the_dev_runs_variable`).

## No other program runs with the app's permissions

- **Rule:** a program the app starts runs as its child, with its privacy permissions (Full Disk Access when granted),
  so only a program the app can vouch for is started: every program by its absolute path, never a name looked up in
  `PATH`; a program inside an app bundle (a Docker runtime's CLI) only when it carries the valid signature of the
  developer the module's table names.
- **Why:** another program of the user can plant a look-alike bundle (any bundle id is free to write, and
  `/Applications` is writable by an administrator without a password) or set the app's `PATH`, and would then run
  with the access the user granted to this app alone.
- **Enforced by:** `ProcessCommandRunner` refuses a bare program name as not installed (`crates/system/src/process.rs`,
  `a_program_named_without_its_absolute_path_is_never_looked_up`); the docker module asks `InstalledApps::signed_by`
  before it runs a CLI (`modules/docker/src/lib.rs`, the look-alike tests of `modules/docker/src/tests.rs`;
  `crates/system/src/code_check.rs`). Known limit: the signature covers the CLI file, not its bundle, and the check
  and the run are two steps. A look-alike bundle the attacker owns can hold a copy of, or a link to, the genuine signed
  CLI and swap it between the check and the run, with no write to the genuine bundle:
  [`../known-issues/app-docker-cli-checked-then-run.md`](../known-issues/app-docker-cli-checked-then-run.md).

## Least permissions

- **Rule:** the app asks for Full Disk Access only when a scan needs it, and explains why in plain English; it works
  with reduced coverage without it and says what it could not see.
- **Why:** trust; macOS shows the permission prompt to the user.
- **Enforced by:** the engine's coverage step and its tests (a scan never enters a guarded place that is not granted
  or opted into, and its result names what was not seen); the `Info.plist` test, which allows exactly five usage
  descriptions (`app/tauri/tests/info_plist.rs`); review. How it works: [`../architecture/permissions.md`](../architecture/permissions.md).

## No foreign code under the app's access

- **Rule:** code that runs inside Fresher runs with its Full Disk Access, so the app runs only its own code. The
  release bundle is signed as a whole with the hardened runtime and no entitlements: ad hoc (`signingIdentity: "-"`)
  for 0.1.0, which ships without an Apple Developer ID and without notarization. The system then refuses a library
  injected at launch (`DYLD_INSERT_LIBRARIES`), and the bundle's signature verifies (`codesign --verify --deep
  --strict`), so a downloaded copy asks the user to allow it in System Settings (Privacy & Security, Open Anyway)
  instead of reporting it damaged. Users are never told to remove the quarantine attribute.
- **Why:** a same-user program without Full Disk Access could otherwise borrow Fresher's by starting it with a planted
  library; a bundle with a broken signature teaches users to turn off the system's checks.
- **Enforced by:** `bundle.macOS` in `app/tauri/tauri.conf.json` (`signingIdentity`, `hardenedRuntime`, no
  `entitlements`) and its test (`app/tauri/tests/config.rs`); `scripts/check-bundle.ts` (`npm run check:bundle`),
  which `npm run build` runs on the bundle it built, so CI's build job fails on a bundle without the `runtime` flag,
  with entitlements, or whose strict verify fails (plan 041); `npm run build:release`, run by the release workflow,
  checks the universal app and the app inside its disk image the same way (plan 013).
- **Rule:** the core and the modules start an external program only by its absolute path, never by a bare name the
  system would look up in `PATH`, which a same-user program can change for the app.
- **Enforced by:** the crate dependency guard (`scripts/check-crate-deps.ts`, `npm run check:crate-deps`, in the
  pre-push hook and CI), which fails on a bare program given to `Command::new` or to the command port's `run` in
  `crates/*/src` and `modules/*/src` outside tests (plan 041).
