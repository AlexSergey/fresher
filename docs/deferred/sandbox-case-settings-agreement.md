# A sandbox that starts with the agreement accepted

**Agreed:** 2026-11-23, by the agent under the user's delegation of 2026-11-22 (plan 039; to check) ·
**Due at:** the next change to `crates/sandbox`

## What is postponed

A sandbox case file's `[settings]` table can write any setting, `core.agreement.version` and `core.agreement.accepted-at`
included. The sandbox harness test fails such a file (a fresh sandbox must meet the agreement), but `npm run sandbox`
and `npm run dev:sandbox` would still build a sandbox that has the agreement accepted, so a hand-built sandbox could
skip the sheet.

Refusing it at generation needs a change to `crates/sandbox/src/case.rs`, which plan
[039](../plans/039-agreement-and-wording/readme.md) may not change. No shipped case file does it.

## What doing it needs

`case.rs` refuses a `[settings]` key of the agreement when it parses the file, with a test for each of the two keys.
