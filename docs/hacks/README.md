# Hacks

Every place where the code takes a shortcut or makes a non-obvious decision, and why. Without this list an agent
"cleans up" exactly the code that must stay as it is. Before removing an `#[expect(...)]`, an `eslint-disable`, a
`deny.toml` skip, an override or an odd ordering, check here.

## Format

```markdown
## <Short name>

- **What:** the shortcut or decision.
- **Where:** file path(s).
- **Why:** the reason.
- **If cleaned up:** what breaks when someone "fixes" it.
```

## TypeScript 6, not 7

- **What:** the UI is pinned to TypeScript 6.0.3 while 7.x (the native compiler) is out.
- **Where:** `app/ui/package.json`.
- **Why:** typescript-eslint's type-aware rules (`strictTypeChecked`) run on the TypeScript 6 compiler API; prod-forge
  stays on 6.0.3 for the same reason.
- **If cleaned up:** upgrading to 7 before typescript-eslint supports it breaks `npm run lint`.

## No CSP in development

- **What:** `app.security.devCsp` is `null`.
- **Where:** `app/tauri/tauri.conf.json`.
- **Why:** Vite's dev server injects inline scripts and opens a WebSocket for hot reload; the production `csp` stays
  `default-src 'self'` with no remote origin.
- **If cleaned up:** setting the production CSP for dev breaks hot reload and the dev page.

## React lint plugins of the ESLint 10 era

- **What:** `@eslint-react/eslint-plugin` instead of `eslint-plugin-react` + `eslint-plugin-react-hooks`, and
  `eslint-plugin-jsx-a11y-x` instead of `eslint-plugin-jsx-a11y`.
- **Where:** `eslint.config.ts`, `package.json`.
- **Why:** `eslint-plugin-react` 7.37 and `eslint-plugin-jsx-a11y` 6.10 do not support ESLint 10 (their peer ranges stop
  at 9); `@eslint-react`'s preset already holds the rules of hooks, so `react-hooks` only doubled every report.
- **If cleaned up:** going back to the classic plugins needs ESLint 9 or a forced peer install.

## The app icon over 1 MiB

- **What:** `app/tauri/icons/icon.icns` (1.03 MB) is the one file allowed over the 1 MiB limit.
- **Where:** `SIZE_EXCEPTIONS` in `scripts/check-staged.ts`.
- **Why:** macOS needs the 1024 px size in the icon set for Retina displays.
- **If cleaned up:** removing the exception blocks every commit that touches the icon.

## node:test for the repository scripts

- **What:** `scripts/*.test.ts` use `node:test`, not Vitest; ESLint's Vitest rules apply to `app/ui` only and
  `no-floating-promises` is off for the scripts' tests.
- **Where:** `scripts/check-staged.test.ts`, `eslint.config.ts` (`scriptTestOverrides`).
- **Why:** the scripts run on Node's own TypeScript support with no build; `node --test` needs no dependency.
- **If cleaned up:** applying the Vitest rules to them reports every `node:test` import and `describe` call.

## Sandbox mode avoids the places that prompt

- **What:** in sandbox mode the scanner keeps a standing avoid list: the guarded places of both the sandbox's home and
  the real home (Desktop, Documents, Downloads, Pictures, Movies, Music, `Library/Containers`,
  `Library/Group Containers`, `Library/Mobile Documents`, `Library/CloudStorage`, `Library/Calendars`, the address book
  folder), added to every walk request's own `avoid`; each place is reported as not scanned. The sandbox home's places
  of the guarded folders its `world.toml` states `access` for are left out of that list: the engine's coverage
  decides them as on a Mac. The real home's places stay on it whatever the world says. Its access is the world's when
  it states any (`World::access`: Full Disk Access `Unknown`, a stated folder as stated, every other permission
  `NotAsked`, the stated folders guarded under the sandbox's home), else the stand-in `UnknownAccess` (Full Disk Access
  `Unknown`, every other permission `NotAsked`, no guarded place, never tries one). The normal app has no standing
  list: `MacAccess` names the guarded places and the engine's request carries the ones the capability state keeps
  closed.
- **Where:** `app/tauri/src/sandbox.rs` (`compose`, `SandboxPorts::real`), `app/tauri/src/adapters.rs`
  (`UnknownAccess`), `crates/sandbox/src/world.rs` (`World::access`, `World::stated_places`),
  `crates/macos/src/privacy/places.rs` (`places_that_prompt`).
- **Why:** the sandbox's world is not the Mac's, so the Mac's access is never asked for it; and a scan root outside
  the sandbox, such as the real home, must raise no system prompt (plan 006, agent's call 30).
- **If cleaned up:** a sandbox scan of a root outside the sandbox raises real system prompts, and a sandbox shows the
  Mac's permission state instead of its own.

## Development builds clean only in a sandbox folder

- **What:** outside a sandbox, the app's cleaner gets `Reach::DryRun` in every debug build: every check runs on the
  real disk and every item ends "a dry run", nothing is moved or deleted, and its Trash (`DryRunTrash`) refuses every
  call. A test gets `Reach::Sandbox`, a sandbox folder or the dry run (a development build on the real Mac
  answers "a dry run"). Under `Reach::Sandbox` an item is removed only below a folder that holds a valid
  `.fresher-sandbox` marker, never in the home folder itself; every other item is left out with "Development build:
  removal works only in a sandbox folder." Only a release build gets `Reach::RealDisk`.
- **Where:** `app/tauri/src/compose.rs` (`reach`), `app/tauri/src/dry_run.rs`, `crates/cleaner/src/reach.rs`.
- **Why:** a development build runs half-finished code against the developer's real disk; it must not be able to
  remove anything there (plan 008, goal 5; plan 026). The token that allows the real disk can be made only without
  `debug_assertions`, and a test of `fresher-cleaner` fails if it is made anywhere but in `compose.rs`. To try a real
  cleanup in a development build, use sandbox mode, below ([`../architecture/removal.md`](../architecture/removal.md),
  "Reach and the sandbox").
- **Sandbox mode:** a debug build started with `FRESHER_SANDBOX=<sandbox folder>` gets `Reach::SandboxFolder`
  instead: it removes only strictly inside that folder, while it holds its marker, with the real Mac's protection list
  joined with the sandbox home's, and moves items into the sandbox's `home/.Trash` (the system's Trash only with
  `FRESHER_SANDBOX_TRASH=real`); its home, known folders and data folder are the sandbox's too
  (`app/tauri/src/sandbox.rs`, [`../architecture/removal.md`](../architecture/removal.md)).
- **If cleaned up:** giving a debug build or a test the real disk lets a bug in a rule, the plan builder or the UI
  remove the developer's own files.

## Command arguments by value

- **What:** `#![expect(clippy::needless_pass_by_value)]` on the shell's commands.
- **Where:** `app/tauri/src/commands.rs`.
- **Why:** `#[tauri::command]` hands a command its `State` and its inputs by value; a reference cannot be taken.
- **If cleaned up:** removing the `expect` fails Clippy on every command.

## The demo module linked in release builds

- **What:** `fresher-module-demo` and `fresher-module-trial` (plan 008/06/05) are normal dependencies of the app, but
  they are registered only under `cfg(debug_assertions)`; the composition test checks that a build without it
  registers neither. `fresher-sandbox` and `fresher-testing` (sandbox mode, plan 007/02/02) are normal dependencies
  too, used only by `src/sandbox.rs`, which is compiled under `cfg(debug_assertions)` only; a test run by
  `cargo test --release` checks that a release build ignores `FRESHER_SANDBOX`.
- **Where:** `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`, `app/tauri/src/sandbox.rs`.
- **Why:** Cargo does not support `cfg(debug_assertions)` in a target-specific dependency table, and plan 007 keeps
  `cfg(debug_assertions)` rather than a cargo feature for dev code (its decision 6).
- **If cleaned up:** removing the `cfg` registers made-up findings and the trial folders in the shipped app; removing
  the dependencies breaks debug builds.
