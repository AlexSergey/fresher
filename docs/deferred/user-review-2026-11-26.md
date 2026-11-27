# The user's review of 2026-11-26

**Agreed:** 2026-11-26, by the user ("I cannot now, write it down as deferred, I will look tomorrow") ·
**Due at:** the user's next session (2026-11-27)

## What is postponed

The walk of plan [043](../plans/043-build-flags/readme.md), step
[03/03](../plans/043-build-flags/03-turn-on/03-walk.md). Steps 01/01 to 03/02 are built on `feat/043-build-flags` and
reviewed. High is offered and Largest is shown. The branch is not merged until the walk passes and the conclusions
are written. When the user has looked at a section, it is removed, and the file and its row with the last one.

## The walk

1. `npm run dev:sandbox -- app-leftovers`, `-- downloads`, `-- large-files`, `-- uninstall`. In each, walk its
   `EXPECTED.md` and the cases HL-01 to HL-04 of
   [`../test-cases/high-and-largest.md`](../test-cases/high-and-largest.md). Look closely at the Review flow of the
   app-leftovers High tile: it was written from the e2e specs, not from the screen.
2. `npm run dev` on the real home: Largest is in the sidebar, and the "Only if you are sure" group appears when a
   module finds something at High.
3. `FRESHER_FLAGS=high=withheld,largest=hidden npm run dev`: the app is 0.1.0's again (no High group, no Largest).
   `FRESHER_FLAGS=largest=nope npm run dev`: the build stops, naming the flag and the value.
4. `node scripts/release-version.ts notes 0.1.0`: the text ends with "This build", with both flags on.

The dry run of the [review of 2026-11-24](./user-review-2026-11-24.md) expects no High and no Largest. That held for
0.1.0's flags. With this branch it holds only under step 3's `FRESHER_FLAGS`.

## Rulings worth a second look

These calls were taken while building, without the user:

- The build script orders flags by name, because the TOML reader sorts tables. The guard keeps the file's order.
  Either way the order is cosmetic.
- The test shell (`app/tauri/src/testing.rs`) does not manage `BuildFlags`. Tauri keeps the first managed value, so
  each `app_info` test manages its own flags.
- `SandboxMode::withholding_high()` was added so that `high-withheld.toml` still runs as a build that withholds High.
- Flag names must match `[a-z][a-z0-9]*`, and each `type` must be a PascalCase name. Both readers refuse anything else.
- The README now says that 0.1.0 did not offer High and 0.2.0 does, before 0.2.0 is tagged.
- The HL cases were checked against the UI's texts and the e2e specs, not on the screen. Where the step's words
  differed from the app, the case says what the app does (the uninstall choices, "Select all", the Folders tab of
  Largest).

## After the walk

Write the conclusions of every level of plan 043 and set the statuses to done. Then run `npm run pre-push`,
squash-merge into `main`, and run `cargo clean`.
