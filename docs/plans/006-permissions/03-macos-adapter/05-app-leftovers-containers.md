# 006/03/05 — Containers in `app-leftovers`, before they are opened

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app-leftovers, docs

The deferred item [`deferred/app-leftovers-containers.md`](../../../deferred/app-leftovers-containers.md) is due at
plan 006, "before `Library/Containers` is scanned". Step 03/03 is the one that lets the walk enter it (with data from
other apps or Full Disk Access granted), so this step runs **before 03/03** (agent's call 31). It takes the item's
agreed remedy: the `container` row stays not offered. A container keeps proving its id (the design's open question 1,
option (a); if the user picks (b), the row also stops being a proof, and the tests below change accordingly).

It also makes the module honest about what it cannot see: an unseen container hides proof, so every category of the
module declares `SeesMore(OtherAppsData)` (design, section 5).

## What exists already

- `modules/app-leftovers/places.toml` — the `container` row (`folder = "containers"`, `proof = true`,
  `optional = true`), category `app-leftovers.data`.
- `modules/app-leftovers/src/places.rs` — `Row { id, category, folder, path, kind, suffix, proof, optional }` read
  from the table by `Places::parse`, checked row by row.
- `modules/app-leftovers/src/decide.rs` — `decide_id` (rows 1–12: a proof row's folder proves the id) and
  `decide_item` (rows 13–15) per leftover of an offered id.
- `modules/app-leftovers/src/lib.rs` — the categories from `CATEGORIES` as `CategoryDescriptor::new(id, danger)`,
  no needs.
- `modules/app-leftovers/src/tests.rs` — `a_container_proves_an_id_once_containers_are_scanned` (asserts
  `container/com.example.Gone` among the identities), `the_containers_left_unread_keep_nothing_since_they_are_optional`.
- `crates/e2e/tests/app_leftovers.rs` — flows with a container-only app; flow 8,
  `containers_are_not_opened_until_asked`; goldens `fixtures/golden/app-leftovers.json` and
  `app-leftovers-flow.json`.
- `fixtures/sandboxes/app-leftovers.toml` — its header says no container is opened in sandbox mode; case "a container
  is not opened"; golden `fixtures/golden/sandbox-app-leftovers.json`.
- `fresher_core::CategoryDescriptor::with_needs(Vec<Need>)` and `Need::sees_more(Capability)` (plan 006/02).

## Files

- `modules/app-leftovers/places.toml` — a new row key `offered` (default `true`); the `container` row gets
  `offered = false`, and the table's header comment explains it (a container is read as proof and for the age rule,
  never offered, until the helper list covers every nested bundle of an installed app).
- `modules/app-leftovers/src/places.rs` — `Row::offered: bool`, read with its default; `Places::parse` refuses a row
  that is neither offered nor a proof (`PlacesError::RowWithoutUse { id }`: such a row would only cost a walk).
- `modules/app-leftovers/src/lib.rs` — the loop over ids (after `decide_id`): a leftover of a row that is not offered
  is dropped from both the `IdVerdict::LeftOut` branch (`leave_out`) and the `IdVerdict::Offer` branch (no
  `decide_item`, no finding), so it is never a finding and never a left-out item. `decide_id` still gets every
  leftover (proof, `ScanAgain`, `UsedRecently`); `decide.rs` does not change.
- `modules/app-leftovers/src/lib.rs` (`AppLeftovers::new`) — each category
  `.with_needs(vec![Need::sees_more(Capability::OtherAppsData)])`.
- `modules/app-leftovers/src/tests.rs`, `src/places/tests.rs` — the tests below;
  `a_container_proves_an_id_once_containers_are_scanned` becomes `a_container_still_proves_its_id` (the container no
  longer among the identities).
- `crates/e2e/tests/app_leftovers.rs` and its goldens — the container-only app: proven, its other leftovers offered,
  its container not; flow 8 unchanged in meaning. Goldens regenerated and read.
- `fixtures/sandboxes/app-leftovers.toml` and `fixtures/golden/sandbox-app-leftovers.json` — the header names the
  rule; the golden shows the three categories `Limited` (sandbox mode's access answers `NotAsked` for other apps'
  data), so their totals are partial (`+`). Regenerated and read.
- `docs/categories/app-leftovers-{caches,settings,data}.md` — a container proves its app once Fresher may open
  `Containers` (data from other apps, or Full Disk Access), and is never offered itself yet; each category sees more
  with data from other apps.
- `docs/deferred/app-leftovers-containers.md` and the row in `docs/deferred/README.md` — "Due at" becomes "when
  `InstalledApp::helpers` covers every nested bundle", and "What is postponed" says what 006/03/05 did (not offered)
  and what is left (offering containers).

## Produces

`app-leftovers` safe for 03/03 to open `Library/Containers`; its categories declare `SeesMore(OtherAppsData)`.

## Checks

- Negative (first): `a_container_is_never_offered` (a scanned container of a removed, proven app: no finding, no
  left-out item, under any verdict), `a_recent_container_keeps_its_id` (age rule), `a_row_neither_offered_nor_a_proof_is_refused`,
  the module's contract test (`assert_module_contract`, no `check_removals_matched` problem).
- Positive: `a_container_still_proves_its_id` (the id's caches offered, the container not),
  `every_category_sees_more_with_other_apps_data` (the descriptor's needs), `the_shipped_table_parses`.
- `cargo nextest run -p fresher-module-app-leftovers -p fresher-e2e`; `npm run sandbox:test`; the changed goldens are
  read, not only regenerated; `docs/categories` guard (`Catalogue::Required`) passes.

## Conclusions

- Built as written, open question 1 taken as option (a): the `container` row has `offered = false` and stays a proof.
  `places.toml` gains the row key `offered` (default `true`, header comment); `Row::offered`; `Places::parse` refuses a
  row neither offered nor a proof (`PlacesError::RowWithoutUse`). In `finish`, `decide_id` still gets every leftover;
  the leftovers of rows not offered are then dropped once, before both branches, so a container is never a finding nor
  a left-out item. `decide.rs` is unchanged. Each of the three categories declares
  `Need::sees_more(Capability::OtherAppsData)`.
- Decision: the text `app-leftovers.title.container` is removed from `locales/en.json` (no finding uses it any more, and
  `assert_texts_used` refuses an unused text), and `app-leftovers.data.what` no longer names containers. The deferred
  item says both come back when containers are offered.
- Decision: `fixtures/golden/sandbox-app-leftovers.json` did not change on regeneration. The sandbox golden records the
  findings, left-outs, map, cleanup and tree after, not the category totals or their availability, and sandbox mode
  never opens `Library/Containers`, so nothing in it moves; the `Limited` state that `UnknownAccess` (`NotAsked` for
  data from other apps) gives the three categories is not visible there. It is covered by the descriptor test here and
  by the engine's `Availability` tests. Only the case file's header changed (it names the rule).
- Tests: negative `a_container_is_never_offered` (offered, a socket inside, running, a place unread),
  `a_recent_container_keeps_its_id`, `a_row_neither_offered_nor_a_proof_is_refused`; positive
  `a_container_still_proves_its_id` (renamed from `a_container_proves_an_id_once_containers_are_scanned`),
  `every_category_sees_more_with_other_apps_data`, `the_shipped_table_parses`; the shipped-table test also checks that
  `container` alone is not offered. The contract test passes unchanged but for the titles given (10 → 9).
- Fixture tests and e2e: the expected findings lose the two containers (14 → 12 in the fixture homes; 12 → 10 items
  in the e2e plan, the containers checked to stay after the cleanup); every left-out count of the data category drops
  to the support folder alone (`(_, 3, _)` → `(_, 1, _)`, `(_, 2, _)` → `(_, 1, _)`); flow 8 now offers the same nine
  leftovers of the removed app as a scan with the containers seen. Goldens `app-leftovers.json` and
  `app-leftovers-flow.json` regenerated and read: only the container findings, their journal records and the totals
  went, and the containers stay in the tree after the cleanup.
- Docs: `docs/categories/app-leftovers-{caches,settings,data}.md` (row 2, never offered, a closing paragraph on
  containers and data from other apps), `docs/deferred/app-leftovers-containers.md` and its row (due when
  `InstalledApp::helpers` covers every nested bundle), the hack "places that prompt" in `docs/hacks/README.md` (its
  cost to `app-leftovers`), the fixture homes' headers.
- Checked: `cargo nextest run -p fresher-module-app-leftovers -p fresher-e2e` 219 passed; `npm run sandbox:test` 42
  passed; `cargo nextest run -p fresher-app` 136 passed; `cargo fmt --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Final review (controller's ruling): open question 1 switched to option (b), agent's call 33. The `container` row is
  dropped from `places.toml` (a row with `proof = false` and `offered = false` would be refused by
  `RowWithoutUse`, and loosening that rule to keep a dead row was the worse change); the header says why and what
  brings it back. The `offered` key, `RowWithoutUse` and the filter in `finish` stay, unused by the shipped table, for
  a row read only as a proof (new test `a_proof_that_is_not_offered_is_read`). The module reads no container, so the
  three categories drop `SeesMore(OtherAppsData)`; the minor fix "no title key for a row not offered" is moot (no such
  row). `app-leftovers.data.what` already names support folders only, which is right under (b).
- Tests after (b): `a_container_is_never_offered`, `a_recent_container_keeps_its_id`, `a_container_still_proves_its_id`,
  `every_category_sees_more_with_other_apps_data`, `the_containers_left_unread_keep_nothing_since_they_are_optional`
  and the decide test `a_container_is_a_proof` are gone; new negative
  `the_containers_are_no_place_of_the_module_and_need_no_permission`; the shipped-table tests count nine rows and
  eight interests. Fixture homes: `com.example.Boxed` (a container and its caches) is no longer claimed (12 → 11
  findings); `with_the_containers_avoided_the_answers_are_the_same`. Core e2e: the plan of flow 1 has 9 items (was
  10), the caches left-out counts drop by Boxed's one, flow 2's rescan claims nothing (the saved state, the removed
  app's only proof, went with the caches), flow 8 offers the same as a scan with the containers seen. The step 03/03
  e2e checks `Limited`/`Ready` become `without_data_from_other_apps_every_category_is_still_ready`; `Limited` and
  `Ready` stay covered on the real engine by `modules_report_the_availability_of_each_category` and the session's
  limited-category tests. Goldens `app-leftovers.json` and `app-leftovers-flow.json` regenerated and read: only
  Boxed's caches finding, its journal and put-back records and the totals went, and its caches stay in the tree
  after the cleanup. The sandbox golden did not change.
