# 002/07/01 — The dependency guard

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** repo

## Files

- `scripts/check-crate-deps.ts` — runs `cargo metadata --format-version 1 --no-deps --locked`, reads each workspace
  package's declared dependencies on other workspace packages, and fails (listing every bad edge) when:
  - a normal or build dependency is outside the table of the design (`core` → nothing, `engine` → core, `store` →
    core, …; `fresher-app` and `fresher-e2e` → anything);
  - a `modules/*` crate depends on another `modules/*` crate, or on anything but `fresher-core` and `fresher-rules`;
  - a dev-dependency is anything but `fresher-testing`, the crate's own allowed table, or (for `fresher-e2e`)
    anything;
  - a workspace crate is not in the table at all (a new crate is added to the table on purpose, with its row in
    `docs/architecture/`).
  The table is a constant in the script, mirrored in [`../../../architecture/README.md`](../../../architecture/README.md).
- `scripts/check-crate-deps.test.ts` — `node --test` on made-up `cargo metadata` JSON.
- `package.json` — `check:crate-deps`; added to `pre-push` (after `lint`).
- `docs/quality.md` — the guard's row in "Repository guards".

## Produces

The modularity of the design, enforced on every push.

## Checks

- Negative: `fails on a module depending on another module`, `fails on the engine depending on the store`,
  `fails on the core depending on anything in the workspace`, `fails on an unknown workspace crate`,
  `fails on a dev-dependency outside the table`.
- Positive: `passes the current workspace` (runs on the real `cargo metadata`), `allows fresher-testing as a
  dev-dependency anywhere`, `allows the app and e2e to depend on everything`.
- `npm run test:scripts` and `npm run check:crate-deps` pass.
