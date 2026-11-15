# Privacy

## No network

- **Rule:** the app makes no network requests: no telemetry, no analytics, no crash upload, no update check (until an
  update mechanism is agreed with the user).
- **Why:** the app sees the whole file system; nothing about it leaves the Mac.
- **Enforced by:** no HTTP client in the dependency tree (`deny.toml` bans), the Tauri capabilities allow no network,
  the UI's Content Security Policy allows no remote origin (plan 001).

## Local data only

- **Rule:** scan results and snapshots are stored only in the app's own folder in `~/Library/Application Support/`.
  Logs never contain file contents.
- **Why:** paths alone already reveal a lot about a person.
- **Enforced by:** the store crate (planned); review.

## Least permissions

- **Rule:** the app asks for Full Disk Access only when a scan needs it, and explains why in plain English; it works
  with reduced coverage without it and says what it could not see.
- **Why:** trust; macOS shows the permission prompt to the user.
- **Enforced by:** the shell (planned).
