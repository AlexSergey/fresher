# 039/02/02 — Review, the confirmation and Scan

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** core, app/ui

Review in two parts, "Safe to remove" and "Danger zone"; the summary names how far the selection reaches; the
confirmation of a High plan asks for its own tick; Scan's "Select all" of a Medium or High zone asks first. Structure
and one colour mark the zone, not alarm words. The design: [`../00-design.md`](../00-design.md), section 2; agent's
calls 4, 6 and 12.

## Read first

`app/ui/src/screens/review/{review-screen.tsx,review-groups.ts,review-group.tsx,review-screen.module.css}`,
`app/ui/src/product/danger-zone/{danger-zone.tsx,zones.ts,danger-zone.module.css}`,
`app/ui/src/product/confirm-dialog/confirm-dialog.tsx` (the permanent tick, reset at each opening),
`app/ui/src/screens/scan/{scan-zones.tsx,zones.ts}`, `app/ui/src/design/layout/dialog/`, the danger tokens in
`app/ui/src/design/tokens/`, `crates/core/src/texts.rs` (`core_keys`), 02/01's commit; the second owner of the
confirmation, `app/ui/src/screens/apps/app-screen.tsx` (plan 036: an uninstall is confirmed there, without Review;
"Remove the app and everything it left" is High), with `app/ui/src/test/app-detail.ts` (`uninstallPlanOf`) and
`app/ui/e2e/apps.spec.ts`; the Largest picks (plan 035: `core.own-file`, always High, reviewed as the group "Your
own files") in `app/ui/e2e/support/largest-mock.ts` and `app/ui/src/test/large-scan.ts`.

## Files

- `crates/core/locales/en.json` and `crates/core/src/texts.rs` (`core_keys`) — exactly:
  `core.danger.part.safe.name` "Safe to remove"; `core.danger.part.safe.meaning` "Each of these comes back by itself
  or with a download."; `core.danger.part.danger.name` "Danger zone"; `core.danger.part.danger.meaning` "Each of these
  may hold settings or work that do not come back by themselves. Check them one by one."
- `app/ui/src/product/danger-zone/zones.ts` — `export type DangerPart = 'danger' | 'safe';`
  `export const PART_OF: Readonly<Record<DangerLevel, DangerPart>> = { high: 'danger', low: 'safe', medium: 'danger',
  safe: 'safe' };` `export const PART_ORDER: readonly DangerPart[] = ['safe', 'danger'];` (the comment: a grouping for
  display; the names and meanings are the core's).
- `app/ui/src/product/danger-zone/danger-zone.tsx` — an optional `nested?: boolean`: inside a part the heading is an
  `h3` with the level's name (`core.danger.<level>.name`) instead of the zone's `h2`; the meaning stays.
- `app/ui/src/screens/review/review-groups.ts` — `reviewParts(zones): readonly ReviewPart[]`, `ReviewPart = { part:
  DangerPart; highest: DangerLevel; zones: readonly ReviewZone[] }`, in `PART_ORDER`, an empty part left out.
- `app/ui/src/screens/review/review-part.tsx` (new) — a `section` labelled by its heading (`h2`,
  `core.danger.part.<part>.name`) and its meaning; `data-part` and `data-level={highest}`; the danger part has a
  border in its highest level's colour (CSS tokens only, the CSS guard holds); its zones rendered `nested`.
- `app/ui/src/screens/review/review-screen.tsx` — renders the parts; the summary, when the highest level is Medium or
  High, adds under the badge `ui.screen.review.reaches` ("Your selection reaches the danger zone: {meaning}" with the
  level's meaning from the core), before the button.
- `app/ui/src/product/confirm-dialog/confirm-dialog.tsx` — when `plan.summary.highest === 'high'` (and the plan has
  items), a checkbox `ui.confirm.high` "I understand these may be the only copy of my data" with
  `ui.confirm.high-description` "Fresher cannot tell whether you keep a copy elsewhere. Check before you go on.",
  above the permanent choice and separate from it; cleared at every opening (as `permanent`); the confirm button is
  disabled until it is ticked (`aria-disabled` with the description naming why, or `disabled`, as the `Button`
  primitive supports); `onConfirm(method, highRiskUnderstood)`. The tick shows whatever the plan's way: a plan whose
  permanent choice is not offered (an uninstall: `permanentAvailable` false) or whose items go by their tools shows it
  too, when it is High. Both owners pass it to `confirm`: `screens/review/review-screen.tsx` and
  `screens/apps/app-screen.tsx`.
- `app/ui/src/screens/scan/scan-zones.tsx` — for a `medium` or `high` zone, a "Select all" that would select opens a
  `Dialog` first: title `ui.danger-zone.ask.title` ("Select everything at {level}?"), text the level's meaning then
  `ui.danger-zone.ask.text` ("Nothing is removed until you confirm it in Review."), buttons `ui.danger-zone.ask.cancel`
  ("Cancel", first focus) and `ui.danger-zone.ask.confirm` ("Select all"). Unticking, and Safe and Low zones, never
  ask.
- `app/ui/src/locales/en.json` — the `ui.screen.review.reaches`, `ui.confirm.high`, `ui.confirm.high-description` and
  `ui.danger-zone.ask.*` texts above.
- `app/ui/e2e/core-mock.ts` — no new scan: the scripts that already state a High finding are reused
  (`SCRIPTS.downloads`, the large untouched file; the Largest pick of `largest-mock.ts`; the app-leftovers data; the
  uninstall of everything of `apps-mock.ts`), their plans with `highest: 'high'` as `planOf`/`uninstallPlanOf` make
  them.
- The e2e flows that already confirm a High plan tick "I understand these may be the only copy of my data" before
  "Move to Trash" (02/01's mock refuses them otherwise), on 740cf3c3: in `flows.spec.ts`, "the installers and the big
  file go to the Trash and come back from the Cleanups", "a cleanup moves the archive and empties the caches, and put
  back brings the archive back", "a picked file goes to the Trash and leaves the list", "a picked file comes back by
  put back"; any other the run finds (a confirmation that names High). No flow is weakened: each ticks as a user
  would.
- `app/ui/e2e/flows.spec.ts` — negative first: "a High plan cannot be confirmed without its tick" (the button stays
  inactive; no `confirmCleanup` call); "the High tick is cleared when the confirmation opens again"; "Select all in
  the danger zone asks first and Cancel selects nothing"; "nothing in the danger zone is selected before you pick it";
  positive: "Review shows the safe part and the danger zone apart"; "Review names how far the selection reaches";
  "the High tick lets the cleanup go on and is sent with it"; "Select all in the danger zone selects after the
  question"; "a picked file is reviewed in the danger zone" (Largest).
- `app/ui/e2e/apps.spec.ts` — negative first: "an uninstall of everything cannot be confirmed without its tick";
  "an uninstall that keeps the data shows no High tick" (Medium); positive: "an uninstall of everything goes on
  after the tick".
- `app/ui/e2e/visual.spec.ts`, `axe.spec.ts` — Review with both parts, the confirmation of a High plan (tick off and
  on), the Scan question; baselines regenerated for every screen these change, each looked at (on 740cf3c3 also
  "the confirmation of an uninstall with everything", the Review of a Largest pick, the Xcode and downloads Reviews
  and confirmations whose plan is High).

## Tests (unit, negatives first)

- `review-groups.test.ts`: "a selection without Medium or High has no danger part"; "a part with no zone is left out";
  positive: "Medium and High go to the danger part, Safe and Low to the safe part, in that order"; "a part's highest
  level is its riskiest zone's".
- `confirm-dialog.test.tsx`: negative — "a plan below High shows no High tick"; "a High plan cannot be confirmed
  without the tick"; "the High tick is cleared on reopening"; "the High tick and the permanent tick are separate";
  positive — "a ticked High plan confirms with the risk understood"; "a High plan without the permanent choice still
  shows the tick" (an uninstall).
- `app-screen.test.tsx`: "an uninstall of everything is confirmed with the risk understood".
- `review-screen.test.tsx`: "the reach line shows only from Medium"; `scan-zones` tests: "Select all of a Safe zone
  does not ask"; "unticking a High zone does not ask"; "Select all of a High zone asks, then selects".
- `danger-zone.test.tsx`: "a nested zone is a third-level heading named by its level".

## Checks

- `npx vitest run` on the changed test files, `npm run test:cov:ui` (90 %), `npm run lint:ts`, `npm run lint:code`,
  the CSS guard; `npm run test:e2e:ui` in WebKit and Chromium, axe clean; `CARGO_TARGET_DIR=<target> cargo nextest run
  -p fresher-core` (the core's keys).

## Conclusions

- Review has two parts, "Safe to remove" (Safe, Low) and "Danger zone" (Medium, High), from `PART_OF` in
  `product/danger-zone/zones.ts` (the names and meanings are the core's `core.danger.part.*`). The danger part is set
  apart by a border and its highest level's colour; the summary says when the selection reaches it. Category headings
  moved from `h3` to `h4` for a correct heading order.
- The confirmation shows the High tick above and apart from the permanent choice, cleared at every opening; the
  confirm button is inactive until it is ticked, in Review and on an app's page. The four High flows tick it.
- Scan's Select all of a Medium or High zone asks first (Cancel has the first focus); unticking and Safe or Low never
  ask. After the final review the question re-reads the zone's categories when it is answered.
- Nothing is pre-selected. The "reach" line is checked on a High selection; its absence below Medium by the Review
  flow of 'from a scan to the review of what was ticked'.
