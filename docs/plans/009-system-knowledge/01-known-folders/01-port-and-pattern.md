# 009/01/01 — The port and the pattern

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, scanner, rules, testing, e2e

## Files

- `crates/core/src/known_folder.rs` (new) — `KnownFolder` (serde kebab-case, `ALL`, `as_str`) and the port
  `KnownFolders { fn locate(&self, folder) -> Option<PathBuf> }`.
- `crates/core/src/interest.rs` — `Pattern::Under(KnownFolder, String)` replaces `Pattern::UnderHome(String)`;
  `InterestError::OutsideHome` becomes `OutsideFolder`.
- `crates/scanner/src/scanner.rs`, `matcher/compile.rs`, `matcher/fingerprint.rs` — `AnchorResolver` is
  `Fn(KnownFolder) -> Option<PathBuf>`; `known_folders_anchor(&dyn KnownFolders)` canonicalizes each folder once and
  replaces `home_anchor`; the home is `KnownFolder::Home`; the fingerprint names the folder.
- `crates/rules/src/{format,rules}.rs` — `[rule.look] under = { folder = "…", path = "…" }` replaces `under-home`.
- `crates/testing/src/known_folders.rs` (new) — `FakeKnownFolders::home_layout(root)` and `.with(folder, path)`;
  `NaiveWalk` resolves `Under` through it.
- Every test, bench and e2e that wrote `UnderHome(x)` writes `Under(KnownFolder::Home, x)`.

## Produces

The port the app composes in 02, and the pattern every module uses from now on.

## Checks

- Negative: an interest under a folder with an empty, absolute or `..` path is refused; an `Under` interest whose
  folder the system does not have matches nothing; a rule with an unknown folder name is refused; two interest sets
  that differ only in the folder have different fingerprints.
- Positive: the walk (scanner and `NaiveWalk`) matches `Under(AppCaches, "*")` in a fixture home; `~` in a root is
  the `Home` folder; the rule format reads `under`.
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-rules -p fresher-testing -p fresher-e2e`.

## Conclusions

- Built as written. `KnownFolder` has fourteen folders (serde kebab-case, `ALL`, `as_str`); `Pattern::Under` replaced
  `UnderHome` everywhere; `InterestError::OutsideFolder` and `RuleError::PathOutOfFolder` replaced the home variants.
- The scanner: `known_folders_anchor(&dyn KnownFolders)` canonicalizes every folder once; `home_anchor(home)` stays
  for tests, benches and the e2e, and resolves the home folder alone. The fingerprint encodes the folder's name, so
  every cache written before this change is discarded once (one full scan).
- `FakeKnownFolders::home_layout(home)` gives the user's folders; the apps folders of every user and of the system
  are absent until `with` sets them. `NaiveWalk` resolves through it (`with_known_folders` overrides).
- The scanner's reference test now has an `AppCaches` interest (matches like the home path) and a
  `SystemApplications` one (resolves nowhere, matches nothing) in both walks. 1390 tests pass.
- A test environment note: a `target/bench-trees` copied by an APFS clone fails the reader comparison (clones share
  blocks); it is rebuilt when deleted.
