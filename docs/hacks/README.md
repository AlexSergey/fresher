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
