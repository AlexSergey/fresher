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

## Places that prompt are avoided until plan 006

- **What:** the app's scanner has a standing avoid list: the home folder's places whose opening may raise a system
  prompt (Desktop, Documents, Downloads, Pictures, Movies, Music, `Library/Containers`, `Library/Group Containers`,
  `Library/Mobile Documents`, `Library/CloudStorage`, `Library/Calendars`, the address book folder). It is added to
  every walk request's own `avoid` (the engine's request carries none yet); each place is reported as not scanned.
- **Where:** `crates/macos/src/places.rs` (`places_that_prompt`), `app/tauri/src/compose.rs`
  (`ScannerOptions::standing_avoid`).
- **Why:** a first scan must not raise a wall of system prompts; which places to open is decided by plan 006 with the
  capability state, which then feeds the request's `avoid` and the standing list goes.
- **If cleaned up:** removing the list before plan 006 makes the first scan raise a prompt for each protected folder.

## Access answers Unknown until plan 006

- **What:** `UnknownAccess` answers `Unknown` for Full Disk Access, whatever macOS says.
- **Where:** `app/tauri/src/adapters.rs` (`UnknownAccess`), `app/tauri/src/compose.rs`.
- **Why:** asking macOS needs the FFI module of `fresher-macos` (plans 004/02 and 006); the UI must not claim access
  it cannot check.
- **If cleaned up:** answering `Granted` or `Denied` without a probe shows the user a permission state that is not
  true.

## Command arguments by value

- **What:** `#![expect(clippy::needless_pass_by_value)]` on the shell's commands.
- **Where:** `app/tauri/src/commands.rs`.
- **Why:** `#[tauri::command]` hands a command its `State` and its inputs by value; a reference cannot be taken.
- **If cleaned up:** removing the `expect` fails Clippy on every command.

## The demo module linked in release builds

- **What:** `fresher-module-demo` is a normal dependency of the app, but it is registered only under
  `cfg(debug_assertions)`.
- **Where:** `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`.
- **Why:** Cargo does not support `cfg(debug_assertions)` in a target-specific dependency table; plan 007's dev-tools
  feature takes the module over.
- **If cleaned up:** removing the `cfg` registers made-up findings in the shipped app; removing the dependency breaks
  debug builds.

## Review cannot remove yet

- **What:** the review screen's **Move to Trash…** button is disabled, with a plain line under it saying removal comes
  in a later version; the screen mounts no `ConfirmDialog` and no `ResultSummary` (both are complete in the gallery).
- **Where:** `app/ui/src/screens/review/review-screen.tsx`.
- **Why:** the core has no cleanup command until plan [008](../plans/008-safe-removal/readme.md); the dialog must show
  the core's cleanup plan, never the UI's own sums.
- **If cleaned up:** enabling the button without a core plan would show the confirmation with the UI's preview sums and
  nothing behind its confirm; plan 008 enables it and feeds `ConfirmDialog` from the core's plan.
