# 027/03/03 — Review in zones

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

Review groups the selection by the same four danger zones as Scan, with the same heads, and the warning icon on
Medium and High (feedback item 16). The design: [`../00-design.md`](../00-design.md), section 3 (agent's call 10;
decision 1).

## Files

- `app/ui/src/screens/review/review-groups.ts` — `reviewZones(groups: readonly ReviewGroup[]): readonly { readonly
  groups: readonly ReviewGroup[]; readonly level: DangerLevel }[]`: by each group's `highest`, in `ZONE_ORDER`
  (`product/danger-zone/zones.ts`), empty zones left out, each zone's groups in their largest-first order.
- `app/ui/src/screens/review/review-screen.tsx` — after the summary, one `DangerZone` (no `selectAll`) per zone of
  `reviewZones(groups)`, holding its `ReviewGroupSection`s; the summary, the refusal notice, the confirmation and the
  cleanup panel unchanged.
- `app/ui/src/screens/review/review-group.tsx` and `review-screen.module.css` — the group's name an `h3`; spacing
  between zones.
- Baselines: `visual.spec.ts`'s review cases and `axe.spec.ts` unchanged in code; the review screenshots regenerated.

## Produces

Nothing later steps use.

## Checks

- `review-groups.test.ts` (new or in `review-screen.test.tsx`), negatives first:
  - `describe('negative cases')`: `it('leaves out a zone with no group')`; `it('never puts a group in a lower zone
    than its highest level')`.
  - `describe('positive cases')`: `it('orders the zones Safe to High')`; `it('keeps the groups largest first inside a
    zone')`.
- `review-screen.test.tsx`, negatives first: no Select all on Review; no warning icon on a Safe zone; then a
  selection across Safe and High shows two zones, High's head with the warning icon and the core's meaning.
- `npm run test --workspace app/ui -- src/screens/review`; `npm run lint:ts`; `npm run knip`; `npx eslint` and
  `npx prettier --check` on the changed files.
- E2E: `flows.spec.ts`'s `from a scan to the review of what was ticked` asserts the zones' heads in order; new
  positive `unticking the last row of a zone removes the zone`. `npm run test:e2e:update --workspace app/ui --
  visual.spec.ts contrast.spec.ts` (look at each PNG), then `npm run test:e2e --workspace app/ui`.

## Conclusions

Not written yet.
