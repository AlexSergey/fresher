# Release versioning

**Agreed:** 2026-11-16, by the user · **Due at:** before the first release (plan 013)

## What is postponed

The version was reset to 0.1.0 on 2026-11-16 and stays there while the app is built: plans no longer bump it. Nobody
has tested the app or looked for its bugs yet, so a large number would promise too much. How versions grow from
there, and how a release is cut, is decided when the first release is prepared.

## What doing it needs

- Choose the numbering from 0.1.0 on: the first build for testers is a 0.x release; 1.0.0 comes only after real
  testing.
- Automate the release: bump the version in every place that holds it (`Cargo.toml` of the workspace, both
  `package.json`, `app/tauri/tauri.conf.json`, the lock files), the changelog from the Conventional Commits, the tag.
  A candidate is `release-it` (a dev tool; its licence is checked as for any dependency in
  [`../dependencies.md`](../dependencies.md)).
- Write the chosen process into [`../workflow.md`](../workflow.md) and plan 013.
