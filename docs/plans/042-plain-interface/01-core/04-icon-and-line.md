# 042/01/04 — A category's icon and one line

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 ·
**Modules:** core, testing, modules (lib.rs and locales), app/ui (bindings, fixtures), docs

Every category names one of 14 icon keys in its core data, and gains a sixth text, `line`: what its tile says under
its name. The UI draws the shape of each key (plan 02). The design: [`../00-design.md`](../00-design.md), section 3;
agent's calls 8 and 10. Mockups: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) and
[`review.dc.html`](../../../mockups/0.1.0/review.dc.html) (the tiles' icons and lines).

## Read first

`crates/core/src/category.rs` (`CategoryDescriptor`, `CategoryText`), `crates/core/src/largest/own_file.rs`,
`crates/core/src/apps/categories.rs`, `crates/testing/src/contract.rs` (`assert_module_contract`, the five texts),
`scripts/check-texts.ts` (`PART_SEGMENTS`: a category's texts), each module's `src/lib.rs` where its descriptors are
built (`rg "CategoryDescriptor::new" modules`), each module's `locales/en.json`, `crates/core/locales/en.json`
(`core.own-file.*`, `core.uninstall-*`), `docs/architecture/modules.md` (the category's texts), `docs/writing.md`
section 2.

## Files

- `crates/core/src/category.rs`:
  - `pub enum CategoryIcon { Cache, Build, Download, Project, Toolchain, Log, Temporary, App, Model, Editor, Browser,
    Device, Archive, Generic }` (`Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS`, kebab-case), with `ALL`.
  - `CategoryDescriptor` gains `icon: CategoryIcon` (`Generic` from `new`), `#[must_use] pub fn with_icon(mut self,
    icon) -> Self`, `pub const fn icon(&self)`; it serializes as `"icon"`.
  - `CategoryText::Line` (segment `line`), `ALL` of six, in the order name, line, what, made-by, if-removed,
    comes-back; the existing tests updated.
- `crates/testing/src/contract.rs` — every category has its six texts; a module's category whose icon is `Generic` is
  a contract failure ("category <id> names no icon"), one test each way.
- `scripts/check-texts.ts` — `line` joins the category's text segments wherever the script lists them (no new rule
  yet: 08/03 adds its length); its test file updated if it lists the five.
- Each module's `src/lib.rs` — `.with_icon(CategoryIcon::<Key>)` on every descriptor, by this table; core categories
  in `crates/core/src/largest/own_file.rs` and `crates/core/src/apps/categories.rs`:

  | Category | Icon | `line` (at most 50 characters, no full stop) |
  | --- | --- | --- |
  | `app-caches.library-caches` | cache | Copies apps keep to open things faster |
  | `app-caches.browsers` | browser | Pages and images browsers keep to load faster |
  | `app-caches.web-engine` | browser | Caches of apps built on web pages |
  | `app-leftovers.caches` | cache | Caches of apps you have removed |
  | `app-leftovers.settings` | app | Settings of apps you have removed |
  | `app-leftovers.data` | app | Data of apps you have removed |
  | `demo.caches` | cache | Example caches of the demo module |
  | `demo.downloads` | download | Example downloads of the demo module |
  | `demo.images` | archive | Example images of the demo module |
  | `demo.volumes` | archive | Example volumes of the demo module |
  | `dev-caches.npm` | download | Packages npm downloaded before |
  | `dev-caches.yarn` | download | Packages Yarn downloaded before |
  | `dev-caches.homebrew` | download | Installers Homebrew downloaded before |
  | `dev-caches.cargo` | download | Crates Cargo downloaded before |
  | `dev-caches.cocoapods` | download | Pods CocoaPods downloaded before |
  | `dev-caches.gradle` | build | Files Gradle keeps to build faster |
  | `dev-caches.pip` | download | Packages pip downloaded before |
  | `dev-caches.go-build` | build | Files Go keeps to build faster |
  | `dev-caches.xcode-derived-data` | build | Leftovers of past app builds |
  | `downloads.unfinished` | download | Downloads that never finished |
  | `downloads.safari-resumable` | download | Downloads Safari can pick up again |
  | `downloads.installers` | download | Installers of apps you already have |
  | `downloads.macos-installers` | download | Installers of older macOS versions |
  | `downloads.large-untouched` | download | Big downloads you have never opened |
  | `jetbrains.caches` | editor | Caches of IDE versions you no longer use |
  | `jetbrains.logs` | log | Logs of IDE versions you no longer use |
  | `jetbrains.settings` | editor | Settings of IDE versions you no longer use |
  | `jetbrains.uninstalled` | editor | Data of JetBrains IDEs you removed |
  | `local-ai-models.hugging-face-models` | model | AI models you downloaded from Hugging Face |
  | `local-ai-models.hugging-face-datasets` | model | Datasets you downloaded from Hugging Face |
  | `local-ai-models.lm-studio` | model | Models you downloaded in LM Studio |
  | `local-ai-models.ollama` | model | Models you downloaded with Ollama |
  | `logs.app-logs` | log | Records apps wrote about what they did |
  | `logs.crash-reports` | log | Reports of apps that quit unexpectedly |
  | `stale-projects.dependencies` | project | Packages of projects you have not touched |
  | `stale-projects.build-output` | project | Build output of projects you have not touched |
  | `stale-projects.python-environments` | project | Python setups of projects you have not touched |
  | `stale-projects.rust-build-cache` | project | Build files of Rust projects you have not touched |
  | `temp.old-files` | temporary | Temporary files apps no longer need |
  | `trial.caches` | cache | Caches made for the trial sandbox |
  | `trial.kept` | archive | Folders the trial empties but keeps |
  | `trial.tool` | toolchain | Items a trial tool removes for you |
  | `unused-toolchains.rust` | toolchain | Rust versions no project of yours uses |
  | `unused-toolchains.node` | toolchain | No project of yours asks for them |
  | `xcode.device-support` | device | Files Xcode made for devices you used |
  | `xcode.previews` | build | Files Xcode makes to show previews |
  | `xcode.simulator-caches` | device | Caches of the iOS simulators |
  | `xcode.archives` | archive | Builds you archived to share or upload |
  | `core.own-file` | generic | A file you picked yourself |
  | `core.uninstall-keep-data` | app | The app, with its data kept |
  | `core.uninstall-everything` | app | The app and everything it left |

  A module category may not be `generic` (the contract refuses it). A category the table misses (one added
  since b12959dc) gets the closest key and a line in the same voice; the report names it.
- Each module's `locales/en.json` and `crates/core/locales/en.json` — the `<category>.line` texts above, kept in the
  dictionary's sorted order; `core_keys` gains the three core lines.
- `app/ui/src/ipc/bindings.generated.ts` (regenerated: `CategoryIcon`, `icon` on `CategoryDescriptor`); every UI
  fixture that builds a `CategoryDescriptor` gains an `icon` (`app/ui/src/test/`, `e2e/core-mock.ts`, stories).
- `docs/architecture/modules.md` — a category has six texts (the `line` beside the name) and an icon key from the
  fixed set; a sentence on what the UI does with them.

## Tests (negatives first)

`category.rs`: negative `a_new_category_has_the_generic_icon`; positive `with_icon_sets_it_and_it_serializes`,
`derives_the_six_category_text_keys`. `contract.rs`: negative `a_module_category_without_an_icon_fails_the_contract`,
`a_category_without_its_line_fails_the_contract`; positive `a_module_with_icons_and_lines_passes`. Every module's
contract test passes unchanged in code (it reads the texts and the icon).

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run --workspace` (every module's contract test, the sandboxes), clippy, fmt;
`npm run test:scripts`; `npm run lint:ts`; the UI unit tests that build categories
(`npx vitest run src/product src/screens`). `git diff main -- modules` shows only `src/lib.rs` and `locales/en.json`
lines.

Commit: `feat(core): every category names an icon and a one-line text`.

## Conclusions

- `CategoryIcon` (14 keys) and `CategoryDescriptor::with_icon`; the contract refuses a module category without an
  icon (`ContractProblem::NoIcon`). `CategoryText::Line` is the sixth text; every module and the core's three
  categories have one.
- The core's uninstall categories are `app`, `core.own-file` is `generic` (ruling D-6: they never appear on a tile).
- Module dictionaries keep their reading order (the line after the name). `docs/architecture/modules.md`,
  `writing.md`, `testing.md`, `quality.md` and the core catalogue files say six texts.
