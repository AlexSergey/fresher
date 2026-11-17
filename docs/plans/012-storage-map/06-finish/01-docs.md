# 012/06/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## Files

- `docs/architecture/README.md` — the `Volumes` row of "Ports" (`space`: kind, capacity, free, purgeable, neighbours;
  `FakeVolumes::with_space`); "Data", "Queries": `owned`, `find`, the shares, `storage_map`, `storage_meaning`,
  `storage_place` and their commands; "UI", "State": the map in the store, the two views, the tones; a short "The
  storage map" section: the parts, the remainder and its reasons, the three phases, one claim per byte.
- `docs/architecture/modules.md` — `NewFinding::for_app`: a module that knows the app names it.
- `docs/deferred/volume-facts.md` and its row in `docs/deferred/README.md` — removed (the README's rule: done work's
  row and file go in the same change); what was not done (a disk image told apart) is written in the conclusions.
- `docs/ideas/storage-explainer.md`, `system-data-analyzer.md`, `application-footprint.md`,
  `trash-holds-freed-space.md`, `large-files-finder.md` — each links to this plan for the part it took, and says what
  stays an idea (snapshot listing and thinning; the last-used date; the journal's share of the Trash; the chart).
- `docs/dependencies.md` — checked again for the `objc2-foundation` features of 02/02.
- `docs/testing.md` — checked again for "Sandboxes" (05/02).
- `docs/hacks/README.md` — nothing new expected (02/01's stand-in is gone by 02/03); a shortcut taken while building
  is written there.
- `docs/categories/README.md` — unchanged: the map adds no category (a sentence there that promises the map is
  updated if one exists).

## Produces

The docs as built.

## Checks

- Every link resolves; `npm run lint` passes; no product or project is named (clean room); no doc speaks of an "Other"
  part.

## Conclusions

Done in `96d2a72`.

- "The storage map" in `docs/architecture/README.md`, the engine's modules, the testing doc; the volume-facts deferred
  item closed; two deferred items written (roots-explained line, way up from the meaning view); five ideas link here.
- The step texts changed by 04/05 carry a note (`Descent`); plan 05 notes it waits for 007.
- `npm run lint` passes.
