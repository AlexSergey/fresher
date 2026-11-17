# 007/01 — Case files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** testing, sandbox, repo

The declarative description of a feature's cases and the one generator that writes it into a sandbox folder. The
fixture vocabulary of `FixtureTree` is reused, not copied: it learns to build into a given folder, and three fields any
fixture may use. The design: [`../00-design.md`](../00-design.md), sections 1–3 and 7.

Runs in parallel with 02/01 and 04. Nothing here reads the real home folder: every test writes into a temporary folder.

## Steps

| #   | Step                                                     | Status   | Shared files                                                                    |
| --- | -------------------------------------------------------- | -------- | ------------------------------------------------------------------------------- |
| 01  | [The tree vocabulary, reusable](./01-tree-vocabulary.md) | ✅ approved | `crates/testing/src/{fixture_tree,lib}.rs`, `docs/testing.md` ("Fixture homes") |
| 02  | [The case format](./02-case-format.md)                   | ✅ approved | `Cargo.toml` (members), `Cargo.lock`, `scripts/check-crate-deps.ts`, `.commitlintrc.cjs`             |
| 03  | [The generator](./03-generator.md)                       | ✅ approved | `package.json` (`sandbox`), `.gitignore`                                        |

In order.

## Conclusions

**Built:** the tree vocabulary as a public `TreeDescription`, the case format and the generator in the new `fresher-sandbox`
crate (`npm run sandbox -- <feature|all>`), writing a sandbox folder with `EXPECTED.md`.

**Measured:** `fresher-testing` 133 tests after 01/01 (136 at the end), `fresher-sandbox` 18 after 01/02 and 47 at the end;
every commit passed the hooks.

**Left:** nothing here; the marker check the generator duplicated moved to 02/01.

**Agent's calls taken during the build (to check):**

- `EXPECTED.md` names categories by id, not title: the sandbox crate cannot read the modules' texts (crate guard). Costs, if wrong: a less friendly checklist; revisit with the run in the app.
