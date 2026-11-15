# Testing

What every part is tested with, and the bars it must pass. Test structure (negative cases first, then positive, in
separate blocks) is in the root [`CLAUDE.md`](../CLAUDE.md).

> **Status:** the target state, set up by plan [001](./plans/001-repository-setup/readme.md) (✅ approved).

## Bars

- **Coverage ≥ 90 %**: lines in the Rust crates (`cargo llvm-cov`); lines, branches, functions and statements in the UI
  (Vitest). The pre-push hook fails under it.
- **Every rule is tested both ways**: paths it must NOT claim come first (look-alike folders, a current version, a
  symlink out of the root), then the paths it must claim.
- **Deterministic**: no wall clock (the clock is injected), no network, no real home folder. Every test runs on a
  temporary fixture tree; the Trash and the Docker CLI are fakes behind traits.
- **Nothing is ever deleted outside a temporary folder** in a test: the cleaner's tests delete into a fake Trash inside
  the fixture tree.

## Kinds of tests

| Kind           | What it proves                                                                   | Rust core                                   | UI                                       |
| -------------- | -------------------------------------------------------------------------------- | ------------------------------------------- | ---------------------------------------- |
| Unit           | one function or module                                                           | `#[cfg(test)] mod tests`, cargo-nextest     | Vitest + Testing Library                 |
| Property-based | a rule holds for ANY valid input (sizes add up, versions order, no double count) | proptest                                    | fast-check (where logic exists)          |
| Integration    | one crate through its public API, on fixture trees                               | the crate's `tests/`                        | —                                        |
| Golden reports | a fixture home gives exactly the expected categories, sizes and danger levels    | `crates/e2e` against `fixtures/golden/`     | —                                        |
| Core e2e       | scan → classify → snapshot → diff → confirm → clean, end to end                  | `crates/e2e` on generated fixture homes     | —                                        |
| UI e2e         | the user flows of the interface, on a mocked core                                | —                                           | Playwright on the Vite build, mocked IPC |
| Visual         | the Dark Premium screens did not change unnoticed                                | —                                           | Playwright screenshots, with tolerance   |
| Performance    | the scan stays within its time and memory budget                                 | criterion benches on a large generated tree | —                                        |

The full app (WebView + Rust) cannot be driven by WebDriver on macOS (`tauri-driver` supports Linux and Windows only),
so the UI e2e run against the web build with the Tauri IPC mocked, and the core e2e cover the Rust side end to end.

## Where things live

- Rust unit tests sit next to the code (`#[cfg(test)] mod tests`); integration tests in each crate's `tests/`.
- `crates/testing` — shared test helpers: the fixture tree builder (sparse files, so a "10 GB cache" costs nothing),
  the fake clock, the fake Trash, the fake Docker CLI.
- `crates/e2e` — the core's end-to-end and golden tests (a crate with tests only).
- `fixtures/homes/<name>.toml` — fixture home folders described as data; `fixtures/golden/<name>.json` — the expected
  reports.
- UI unit tests: `*.test.ts(x)` next to the component; UI e2e and visual tests: `app/ui/e2e/`.

## Commands

| Command                        | Runs                                                           |
| ------------------------------ | -------------------------------------------------------------- |
| `npm test`                     | Rust tests (`cargo nextest run --workspace`) and UI unit tests |
| `npm run test:rust`            | `cargo nextest run --workspace --locked`                       |
| `npm run test:ui`              | `vitest run` in `app/ui`                                       |
| `npm run test:cov`             | both with coverage and the 90 % floors                         |
| `npm run test:e2e`             | `crates/e2e` and the Playwright suite                          |
| `cargo nextest run -p <crate>` | one crate                                                      |
