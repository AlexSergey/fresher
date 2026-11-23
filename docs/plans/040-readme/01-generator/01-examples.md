# 040/01/01 — The example line in every category

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every category file gains one line the README's table shows in its "Where, for example" column: what a developer
recognizes on their own disk. Documentation only; no code, no test changes. The design:
[`../00-design.md`](../00-design.md), section 2, "The `**Example:**` line".

## Context

- The catalogue is `docs/categories/`: one file per category, `<module>-<name>.md`, listed in the "Catalogue" table of
  `docs/categories/README.md`. Each file opens with `# <title>`, a blank line, then the header line
  `**Danger level:** <level> · **Rule:** … · **Since:** 0.1.0`.
- The only code that reads these files is `crates/testing/src/contract.rs` (`catalogue_problem`): it takes the first
  word after `**Danger level:**`. A new line does not disturb it; do not change that file.
- `docs/` is in `.prettierignore`: Prettier does not reformat these files.

## Files

- Every `docs/categories/*.md` except `README.md` (37 files on 33d51a6c) — after the header line, a blank line and
  then exactly one line:

  ```markdown
  **Example:** <text>
  ```

  Rules for `<text>`: one line; at most 80 characters (counted on the raw text, backticks included); no `|`; plain
  English; paths in backticks with `~` for the home folder; true of what the file's "Where it lives" and "How the app
  decides" say (a threshold named must be the default the file states). It is an example, not the full rule.

  Suggested texts (check each against its file and correct it where the file says otherwise):

  | File                                       | Example                                                           |
  | ------------------------------------------ | ----------------------------------------------------------------- |
  | `app-caches-browsers.md`                   | Chrome's and Firefox's caches in `~/Library/Caches`               |
  | `app-caches-library-caches.md`             | `~/Library/Caches/com.example.Editor` of an app you still use     |
  | `app-caches-web-engine.md`                 | `Cache` and `Code Cache` of web-based apps in `Application Support` |
  | `app-leftovers-caches.md`                  | `~/Library/Caches/com.example.App` of an app you removed          |
  | `app-leftovers-settings.md`                | `~/Library/Preferences/com.example.App.plist` of a removed app    |
  | `app-leftovers-data.md`                    | `~/Library/Application Support/<app>` of an app you removed       |
  | `dev-caches-cargo.md`                      | `~/.cargo/registry/cache`                                         |
  | `dev-caches-cocoapods.md`                  | `~/Library/Caches/CocoaPods`                                      |
  | `dev-caches-go-build.md`                   | `~/Library/Caches/go-build`                                       |
  | `dev-caches-gradle.md`                     | `~/.gradle/caches`                                                |
  | `dev-caches-homebrew.md`                   | `~/Library/Caches/Homebrew/downloads`                             |
  | `dev-caches-npm.md`                        | `~/.npm/_cacache`                                                 |
  | `dev-caches-pip.md`                        | `~/Library/Caches/pip`                                            |
  | `dev-caches-xcode-derived-data.md`         | `~/Library/Developer/Xcode/DerivedData/<project>-…`               |
  | `dev-caches-yarn.md`                       | `~/Library/Caches/Yarn`                                           |
  | `downloads-installers.md`                  | a `.dmg` or `.pkg` in `~/Downloads` of an app already installed   |
  | `downloads-large-untouched.md`             | a file of 1 GB or more in `~/Downloads` untouched for 30 days     |
  | `downloads-macos-installers.md`            | `Install macOS Sonoma.app` in `~/Downloads`                       |
  | `downloads-safari-resumable.md`            | a paused Safari download, `<name>.download`, in `~/Downloads`     |
  | `downloads-unfinished.md`                  | a `.crdownload` or `.part` file in `~/Downloads`                  |
  | `jetbrains-caches.md`                      | `~/Library/Caches/JetBrains/<IDE><older version>`                 |
  | `jetbrains-logs.md`                        | `~/Library/Logs/JetBrains/<IDE><older version>`                   |
  | `jetbrains-settings.md`                    | `~/Library/Application Support/JetBrains/<IDE><older version>`    |
  | `jetbrains-uninstalled.md`                 | `~/Library/Application Support/JetBrains/<IDE you removed>`       |
  | `local-ai-models-hugging-face-datasets.md` | `~/.cache/huggingface/hub/datasets--<owner>--<name>`              |
  | `local-ai-models-hugging-face-models.md`   | `~/.cache/huggingface/hub/models--<owner>--<name>`                |
  | `local-ai-models-lm-studio.md`             | `~/.lmstudio/models/<publisher>/<model>`                          |
  | `local-ai-models-ollama.md`                | a model in `~/.ollama/models`                                     |
  | `logs-app-logs.md`                         | logs in `~/Library/Logs` untouched for a week                     |
  | `logs-crash-reports.md`                    | `~/Library/Logs/DiagnosticReports` older than a week              |
  | `stale-projects-build-output.md`           | `target` or `build` of a project you have not changed in months   |
  | `stale-projects-dependencies.md`           | `node_modules` of a project you have not changed in months        |
  | `stale-projects-python-environments.md`    | `.venv` of a Python project you have not changed in months        |
  | `stale-projects-rust-build-cache.md`       | `target/debug/incremental` of a Rust project                      |
  | `temp-old-files.md`                        | items in your temporary folder untouched for a week               |
  | `unused-toolchains-node.md`                | a Node version in `~/.nvm/versions/node` no project uses          |
  | `unused-toolchains-rust.md`                | a toolchain in `~/.rustup/toolchains` no project uses             |

  A category file that arrived on `main` after 33d51a6c and is in the worktree gets its line too (04/01 does the same
  for later ones).

- `docs/categories/README.md`, section "Format" — the template gains the line after the header line:

  ```markdown
  **Example:** <one line, at most 80 characters, no `|`: what a user recognizes, shown in the README's table>
  ```

  and one sentence below the template: "The `**Example:**` line is shown in the README's "What it finds" table
  (`npm run readme:generate`); `npm run check:readme` refuses a file without it."

## Produces

Every category file with a valid `**Example:**` line, for 01/02's generator and check.

## Checks

- `grep -L '^\*\*Example:\*\* ' docs/categories/*.md` prints only `docs/categories/README.md`.
- `grep -h '^\*\*Example:\*\* ' docs/categories/*.md | awk '{ sub(/^\*\*Example:\*\* /, ""); if (length($0) > 80 || index($0, "|")) print }'`
  prints nothing.
- `cargo nextest run -p fresher-testing` and one module's contract test (`cargo nextest run -p
  fresher-module-dev-caches --test contract`) still pass (`CARGO_TARGET_DIR` as in the global constraints).
- `npm run check:typos`.
- Commit: `docs(docs): an example line in every category file`.

## Conclusions

Done (a388e30d). Every category file has an `**Example:**` line after its header line, at most 80 characters, no
`|`. The three files that arrived with plans 035 and 036 (`core-own-file`, `core-uninstall-keep-data`,
`core-uninstall-everything`) got theirs in 04/01.
