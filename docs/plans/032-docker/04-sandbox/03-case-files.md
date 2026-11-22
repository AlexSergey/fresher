# 032/04/03 — The case files and the registration

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, app, fixtures, docs

Read first: [`../00-design.md`](../00-design.md) sections 3, 4 and 8, `docs/testing.md` ("The case file"),
`fixtures/sandboxes/app-caches-app-open.toml` (a case file's shape), `app/tauri/src/compose.rs` (the registration
lines), `app/tauri/src/commands.rs` and `app/tauri/tests/sandboxes.rs` (the module lists and one test per case
file), 04/01 and 04/02 (`[[case.tool]]`, `access = { other-apps-data = … }`, `written`, `[[case.insight]]`), 02/03's
golden (the answer text to reuse).

Every case is negative: nothing of Docker's is ever offered or cleaned. The answers below are the design's four lines
(section 3); copy them from `fixtures/golden/docker.json`'s test, not by hand.

## Files

- `fixtures/sandboxes/docker.toml` (new), `feature = "docker"`, `module = "docker"`:
  1. "Docker Desktop not installed, a stale socket left: nothing said, nothing run" — `home/.docker/run/docker.sock`
     stated `kind = "socket"`; no app.
  2. "a disk file at no known place is not Docker's" — `home/VMs/Docker.raw`, sparse, 1 GiB.
  3. "OrbStack running: what is inside, a command for each unused part, nothing run but the read" — `[[case.app]]`
     `Applications/OrbStack.app`, `bundle-id = "dev.kdrag0n.MacVirt"`, `name = "OrbStack"`, `running = true`;
     `home/.orbstack/run/docker.sock` stated `kind = "socket"`; `home/Library/Group Containers/HUAQ24HBR6.dev.orbstack/
     data/data.img` sparse, `size` 8 GiB, `written` 4 MiB; `access = { other-apps-data = "granted" }`; `[[case.tool]]`
     `program = "Applications/OrbStack.app/Contents/MacOS/xbin/docker"`, `args = ["--context", "orbstack", "system",
     "df", "--format", "json"]`, `stdout` the four lines with Containers at `0B`; `[[case.insight]]` `identity =
     "orbstack"`, `figures = 2`, `parts = 4`, `commands = 3`.
- `fixtures/sandboxes/docker-stopped.toml` (new):
  1. "Docker Desktop closed: its disk file's numbers, never asked" — `Applications/Docker.app` (`com.docker.docker`,
     `Docker`), not running; `home/.docker/run/docker.sock` stated `kind = "socket"`; `home/Library/Containers/
     com.docker.docker/Data/vms/0/data/Docker.raw` sparse, `size` 64 GiB, `written` 4 MiB; `access = {
     other-apps-data = "granted" }`; no tool; insight `docker-desktop`, `figures = 2`, `parts = 0`, `commands = 0`.
  2. "OrbStack open with its engine stopped: never asked" — OrbStack installed, `running = true`, no socket, no disk
     file; no tool; insight `orbstack`, `figures = 0`, `parts = 0`, `commands = 0`.
- `fixtures/sandboxes/docker-no-answer.toml` (new):
  1. "Docker Desktop not answering: no numbers, nothing else run" — Docker Desktop installed and running, its socket;
     `[[case.tool]]` `program = "Applications/Docker.app/Contents/Resources/bin/docker"`, `args = ["--context",
     "desktop-linux", "system", "df", "--format", "json"]`, `status = 1`, `stdout = ""`; insight `docker-desktop`,
     `figures = 0`, `parts = 0`, `commands = 0`.
  2. "OrbStack answering what Fresher cannot read: no numbers rather than a guess" — OrbStack running, its socket; the
     read answered with status 0 and `stdout = "Images 3 2 3GB\n"`; insight `orbstack`, `figures = 0`, `parts = 0`,
     `commands = 0`.
- `fixtures/sandboxes/docker-left-behind.toml` (new):
  1. "Docker Desktop removed, its disk file left: its numbers, nothing run" — no app; `Docker.raw` as in
     `docker-stopped.toml`; `access = { other-apps-data = "granted" }`; insight `docker-desktop`, `figures = 2`,
     `parts = 0`, `commands = 0`.
- `fixtures/golden/sandbox-docker{,-stopped,-no-answer,-left-behind}.json` (new) — `UPDATE_GOLDEN=1 npm run
  sandbox:test -- docker`, reviewed: no finding, the insights as expected.
- **Registration**: `app/tauri/Cargo.toml` (`fresher-module-docker`), `app/tauri/src/compose.rs`
  (`registry.register(Box::new(DockerModule::new()?))?;` after the other Development modules), every list of
  registered module ids the app's tests keep (`app/tauri/src/commands.rs`, `crates/sandbox/src/expected.rs` if it
  lists modules, `app/tauri/tests/sandboxes.rs`); `app/tauri/tests/sandboxes.rs` gets one test per new case file
  (its macro), so `every_case_file_has_a_test` and `every_registered_module_has_a_case_file` pass.
- `docs/dependencies.md` — `fresher-module-docker` in `fresher-app`'s dependencies, as for the other modules.

## Produces

The module in every build, its four case files and goldens, for 05 and 06.

## Tests and checks

- `npm run sandbox:test` passes for the four new files and every other one.
- **By-hand mutation checks** (each reverted, each written in this step's conclusions): change `commands = 3` to 4 in
  `docker.toml` — the autotest fails with both numbers; change the stated `args` of the OrbStack tool (for example
  `json` to `table`) — the autotest fails naming the unstated call the module made; remove `[[case.insight]]` from
  `docker-stopped.toml`'s first case — the autotest fails naming the insight no case expects.
- `npm run sandbox -- docker` generates a sandbox whose `EXPECTED.md` names the insight lines.

## Done when

- `cargo nextest run -p fresher-app -p fresher-sandbox`; `npm run sandbox:test`; clippy; fmt; `npm run check:machete`.
- One commit: `test(sandbox): docker's four case files; the module registered in the app`.

## Conclusions

Done: `docker`, `docker-stopped`, `docker-no-answer` and `docker-left-behind` case files, their four goldens (no
finding, the insights as expected; every other golden unchanged), the module registered in `fresher-app` (compose,
the two module-id lists, `fresher-module-docker` dependency) and one test per case file.

By-hand mutation checks, each reverted:

- `commands = 3` changed to 4 in `docker.toml`: the autotest fails with "insight `orbstack` of `docker`: commands:
  expected 4, described 3".
- the OrbStack tool's `json` changed to `table`: the autotest fails with "a program the case file does not state was
  run: .../OrbStack.app/Contents/MacOS/xbin/docker --context orbstack system df --format json".
- `[[case.insight]]` removed from `docker-stopped.toml`'s first case: the autotest fails with "insight
  `docker-desktop` of `docker` is described but no case expects it".

`npm run sandbox -- docker` writes a sandbox whose `EXPECTED.md` names the insight (its card, 2 numbers, 4 parts, 3
commands).
