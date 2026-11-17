# 007/05/02 — A run by hand, the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** repo, docs

## Files

- `docs/plans/README.md` — the row of 007: ✔️ done, 0.1.0 (once the user has looked at the result).
- This plan's readmes — the conclusions: what was built, what was measured, the run by hand, what was left.

## The run by hand

By the user, with the agent preparing and writing down the results (counts and what happened, no other path):

1. `npm run sandbox -- trial`, then `npm run dev:sandbox -- trial`: the banner, a scan, `EXPECTED.md` ticked case by
   case; a cleanup to the sandbox's Trash; put back from the Cleanups screen.
2. `npm run sandbox -- trial` again: the removed items come back, nothing else changes.
3. `npm run dev:sandbox -- trial --real-trash`: one cleanup to the real Trash, put back.
4. `npm run dev`: the development banner; the data folder is the development one.
5. A release build (`npm run build`) started with `FRESHER_SANDBOX` set: no banner, the variable ignored.

## Checks

- `npm run pre-push` passes on the branch's last commit; `cargo test -p fresher-app --release` passes the release-only
  test of 02/02; `npm run test:e2e` passes.

## Conclusions

**The run by hand is waiting on the user.** The agent prepared it; nothing below is ticked yet. Write the counts and what
happened beside each item.

- [ ] 1. `npm run sandbox -- trial`, then `npm run dev:sandbox -- trial`: the banner, a scan, `EXPECTED.md` ticked case by
  case; a cleanup to the sandbox's Trash; put back from the Cleanups screen.
- [ ] 2. `npm run sandbox -- trial` again: the removed items come back, nothing else changes.
- [ ] 3. `npm run dev:sandbox -- trial --real-trash`: one cleanup to the real Trash, put back.
- [ ] 4. `npm run dev`: the development banner; the data folder is the development one.
- [ ] 5. A release build (`npm run build`) started with `FRESHER_SANDBOX` set: no banner, the variable ignored.

Checks: `npm run pre-push` and `npm run test:e2e` passed on the final fix wave; `cargo test -p fresher-app --release` runs
the release-gating tests only (see the agent's call 5 in the plan's conclusions). Two reminders for the run: `dev:sandbox`
and `npm run dev` share one identifier, so quit the first window before starting the second; the Documents folder of a
sandbox is never scanned (so `EXPECTED.md` has no Documents positive).
