# 015/03/02 — The run by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## The run by hand

- [ ] `npm run dev:sandbox -- trial`: the sidebar shows Debug with `demo` and `trial`; scan; Settings → Ignored groups →
  Debug on: the sidebar and the overview update at once, nothing of the two is offered in Review; off: everything is
  back without a rescan. The same with the module `trial` alone, and Restore from its page. Quit and open again: the
  lists are kept. A cleanup prepared before an ignore is refused after it.
- [ ] `npm run dev` on the real Mac (no module of the group Development is registered until plan 016): only Debug shows;
  the three tabs read well with VoiceOver; the tabs answer the arrow keys.
- [ ] `EXPECTED.md` of `trial` walked as before: nothing changed in what is offered.

## Files

- `docs/plans/015-module-groups/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built,
  what changed from the design and why, the measured test counts and coverage, the agent's calls taken while building.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- [ ] `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet: the run by hand waits for the user. Its results are written here, on `main`.
