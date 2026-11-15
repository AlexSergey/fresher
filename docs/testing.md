# Testing

What every part is tested with, and the bars it must pass. Test structure (negative cases first, then positive, in
separate blocks) is in the root [`CLAUDE.md`](../CLAUDE.md). Set up by plan
[001/04](./plans/001-repository-setup/04-testing-architecture/readme.md).

## Bars

- **Coverage ≥ 90 %**: lines in the Rust crates (`cargo llvm-cov`, `app/tauri/src/main.rs` excluded); lines, branches,
  functions and statements in the UI (Vitest, `src/main.tsx` and the test helpers excluded). The pre-push hook fails
  under it.
- **Every rule is tested both ways**: paths it must NOT claim come first (look-alike folders, a current version, a
  symlink out of the root), then the paths it must claim.
- **Deterministic**: no wall clock (the `Clock` port, `FakeClock` in tests), no network, no real home folder. Every test
  runs on a temporary fixture tree; the Trash and external commands are ports with fakes.
- **Nothing is ever deleted outside a temporary folder** in a test: `FakeTrash` moves items into a folder inside the
  fixture tree.

## Kinds of tests

| Kind           | What it proves                                                                   | Rust core                                   | UI                                         |
| -------------- | -------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------ |
| Unit           | one function or module                                                           | `#[cfg(test)] mod tests`, cargo-nextest     | Vitest + Testing Library + happy-dom       |
| Property-based | a rule holds for ANY valid input (sizes add up, versions order, no double count) | proptest (`mod property`)                   | fast-check (where logic exists)            |
| Integration    | one crate through its public API, on fixture trees                               | the crate's `tests/`                        | —                                          |
| Golden reports | a fixture home gives exactly the expected report                                 | `crates/e2e` against `fixtures/golden/`     | —                                          |
| Core e2e       | scan → classify → snapshot → diff → confirm → clean, end to end                  | `crates/e2e` on fixture homes               | —                                          |
| UI e2e         | the user flows of the interface, on a mocked core                                | —                                           | Playwright on the Vite build, mocked IPC   |
| Visual         | the Dark Premium screens did not change unnoticed                                | —                                           | Playwright screenshots, ≤ 50 pixels differ |
| Performance    | the scan stays within its time and memory budget                                 | criterion benches on a large generated tree | —                                          |

The full app (WebView + Rust) cannot be driven by WebDriver on macOS (`tauri-driver` supports Linux and Windows only),
so the UI e2e run against the web build with the Tauri IPC mocked, and the core e2e cover the Rust side end to end.
Until the scanner exists, the core e2e prove the harness: a fixture home is built and read back against its golden
report.

## Where things live

- Rust unit tests sit next to the code (`#[cfg(test)] mod tests { mod negative {…} mod positive {…} }`); integration
  tests in each crate's `tests/`. Helpers of an integration test go in a `#[cfg(test)]` module, so Clippy treats them
  as test code.
- `crates/testing` — shared test helpers: `FixtureTree` (a fake home folder from TOML), `TreeReport` (a deterministic
  listing for golden files), `FakeClock`, `FakeTrash`, `FakeCommandRunner`. The ports they fake (`Clock`, `Trash`,
  `CommandRunner`) live in `fresher-core`.
- `crates/e2e` — the core's end-to-end and golden tests (a crate with tests only).
- `fixtures/homes/<name>.toml` — fixture home folders; `fixtures/golden/<name>.json` — the expected reports.
- UI unit tests: `*.test.ts(x)` next to the component; the core is mocked with `mockCore` from `src/ipc/mock.ts`, by
  the command keys of `src/ipc/index.ts` (the only place the Rust command names are spelled).
- UI e2e and visual tests: `app/ui/e2e/`, the core mocked with `e2e/core-mock.ts`; screenshots in
  `app/ui/e2e/screenshots/`, per browser. WebKit is the reference engine (the app runs in WebKit on macOS); Chromium
  runs too.

## Fixture homes

A fixture home is a TOML file; every path is relative and must stay inside the root.

```toml
# A file with a logical size (sparse: it costs no disk) and a modification time in seconds since the Unix epoch.
[[file]]
path = "Library/Caches/com.example.editor/Cache.db"
size = 52428800
modified = 1735689600

# An empty folder.
[[dir]]
path = "Library/Logs"

# A symlink: listed, never followed. The target may point outside the root.
[[symlink]]
path = "Library/Caches/system"
target = "/Library/Caches"
```

`FixtureTree::from_toml(path)` builds it in a temporary folder that is removed when the tree is dropped;
`tree.root()` is the only folder a test may scan or clean.

## Commands

| Command                        | Runs                                                                                  |
| ------------------------------ | ------------------------------------------------------------------------------------- |
| `npm test`                     | Rust tests, UI unit tests, the repository scripts' tests                              |
| `npm run test:rust`            | `cargo nextest run --workspace --locked`                                              |
| `npm run test:ui`              | `vitest run` in `app/ui`                                                              |
| `npm run test:scripts`         | `node --test scripts/*.test.ts`                                                       |
| `npm run test:cov`             | Rust and UI tests with coverage and the 90 % floors (reports in `coverage/`)          |
| `npm run test:e2e`             | `crates/e2e` and the Playwright suite                                                 |
| `npm run test:e2e:core`        | `cargo nextest run -p fresher-e2e --locked`                                           |
| `npm run test:e2e:ui`          | Playwright (needs `npx playwright install webkit chromium` once)                      |
| `cargo nextest run -p <crate>` | one crate                                                                             |
| Golden files                   | `UPDATE_GOLDEN=1 npm run test:e2e:core`; review the diff before committing            |
| Screenshots                    | `npm run test:e2e:update --workspace app/ui`; look at the new image before committing |
