# 002/06 — Module kit

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** rules, testing, modules

Everything a new module needs, so that a module is written to a pattern rather than invented: the rule format and its
matcher in `fresher-rules`, the test harness and the category contract in `fresher-testing`, the template and the
checklist. The design: [`../00-design.md`](../00-design.md), "The module kit".

Steps 01–03 depend on plan 01 only and run in wave 2, in parallel with 02, 03, 04 and 07/01: they own `crates/rules/`
and `crates/testing/`. Step 04 needs 02/05 (the `demo` module) and 06/03.

## Steps

| #   | Step                                                           | Status      | Depends on   | Shared files                                     |
| --- | -------------------------------------------------------------- | ----------- | ------------ | ------------------------------------------------ |
| 01  | [The rule format](./01-rule-format.md)                         | ✔️ done     | 002/01       | `Cargo.lock`                                     |
| 02  | [The matcher](./02-matcher.md)                                 | ✔️ done     | 01           | —                                                |
| 03  | [Test harness and category contract](./03-harness.md)          | ✔️ done     | 002/01       | `Cargo.lock`                                     |
| 04  | [Template, checklist and the demo through the kit](./04-template.md) | ✔️ done     | 03, 002/02/05 | `modules/demo/` (tests and dev-dependency), `docs/categories/README.md` |

Step 03 is independent of 01 and 02 (it touches only `crates/testing/`) and may run beside them.

## Conclusions

- **Built**: `fresher-rules` (the rule format in TOML, validated against the module's descriptor, and a pure matcher
  with our own `*`/`?` glob, version order and `CACHEDIR.TAG`), `fresher-testing`'s module harness (`NaiveWalk`,
  `run_module` with golden reports, the category contract and the text checks), the template and checklist in
  [`docs/architecture/modules.md`](../../../architecture/modules.md), and the `demo` module checked through the kit.
  A new module is now written to a pattern: descriptor, rules or code, `locales/en.json`, one contract test.
- **Deviations from the design**, each in its step's notes: rule ids are kebab-case and key a rule's texts (01);
  `CACHEDIR.TAG` costs one bounded read in `evaluate`, until the walk of plan 004 reports the tag's validity (02);
  `run_module` returns a `Result` and takes `&Ports` (03); the contract requires the texts of the `<module>.enabled`
  switch (04).
- **For plan 004**: the scanner is tested against `NaiveWalk` on the same fixtures; it should report whether a
  `CACHEDIR.TAG` is valid, so that `fresher-rules` reads nothing from the disk.
- **For the first real modules (plans 009+)**: they follow the checklist, use `Catalogue::Required` and add their
  categories to [`docs/categories/`](../../../categories/README.md) in the same change.
- **For 07/02**: link `docs/architecture/modules.md` from the architecture README, and describe `fresher-rules` as the
  shared matcher (it depends on the core only, not on the scanner).
