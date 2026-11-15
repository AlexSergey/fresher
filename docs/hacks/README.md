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
