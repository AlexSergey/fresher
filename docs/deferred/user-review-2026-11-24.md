# The user's review of 2026-11-24

**Agreed:** 2026-11-24, by the user's delegation (build autonomously, write down what the user must look at) ·
**Due at:** the user's next session

## What is postponed

Walks, rulings and gaps of plan [042](../plans/042-plain-interface/readme.md), built on 2026-11-24 and awaiting the
user's walk. When the user has looked at a section, it is removed, and the file and its row with the last one.

## Walks by hand

- The dry run on the real Mac (`npm run dev`, a scan of `~`): no High anywhere (no data folders, no "Only if you are
  sure" group, no app data in Apps), no Largest in the sidebar; the scanning card's percent on a first scan (the
  estimate is the volume's used space, so it may race ahead or wait at 99 %).
- The demo GIFs wait for this walk ([deferred](./demo-scenarios-for-new-screens.md)).

## Rulings worth a second look

The design's agent's calls ([`00-design.md`](../plans/042-plain-interface/00-design.md), "Agent's calls"):

1. The High switch is set by the app, not a setting; development builds withhold too.
2. Group names "Safe to remove", "Usually safe", "Look before removing", "Only if you are sure".
3. The five parts of the disk are merged in the core.
4. A "Look before removing" item is never ticked for you: a Look tile ticked on Scan sends nothing until you tick its
   items in Review (or the tile's all-checkbox there).
5. Largest leaves the sidebar while High is withheld; Apps stays, with "Keep my settings" only.
6. This plan goes before the security audit (041).
7. High comes back in a later release; no user setting now.
8. A fixed set of 14 icon keys; every category names one.
9. The wording pass is the last step.
10. A sixth category text, the tile's `line` (at most 50 characters, no full stop).
11. The unread remainder and other volumes count as "Your files and apps".
12. Withheld also hides a High category's left-outs and its place in module info and choices.
13. The progress estimate comes from the core, held at 99 % until the end; the module chips are gone.
14. Docker stays described, never offered.
15. Details shows who makes it under "What it is", keeps a scrolling list, and has no Scan button.
16. A menu item's description is its hint. Reversed by the user (2026-11-25): menu items have no hints.
17. Settings: Fresher's own data on General; a module's switch last on its page; the search matches section names.
18. The e2e mock offers High by default; the new specs use a `withheld` helper.
19. "2 minutes ago" is formatted by the interface from the core's time.
20. The bar and the sidebar badge count what goes; with nothing ticked, the categories to check.

Calls taken while building:

- **High hidden entirely**: the Docker card loses its volumes line (an insight part at High), an app's page loses
  its data folders and their sizes, and an Apps row leaves those parts out of its list and its size.
- **Look items are never pre-ticked**, even after "Remove everything" or "Keep settings": those choices bring their
  Look categories to Review to check.
- A category whose level the overview does not tell is treated as Look (never selected whole for you).
- The group's Select all description sits in a tooltip, not as a visible second line.
- The selection bar's level badge is gone on every screen (the mockup has none); the confirmation still names the
  highest level.
- Review's Look line reads "tick each one you have checked": "checked" can read as "ticked" or as "looked at".
- Review's picked tiles (Largest's own files) read "1 picked" and the group's meaning, not "tick each one".
- Review's rows have the same menu as the details' rows, under a gear button (the details use `⋯`).
- The bar's region is "Ready to remove"; the summary bar reads "Size of each group: …".
- The ignored modules tab is "Not looked for"; the ignored groups tab "Sections".
- The confirmation's button reads "Yes, move to the Trash"; Review's reads "Move to the Trash".
- The `~` short path: the core now sends the home it scans (`AppInfo.home`); the full path stays in the tooltip.
- The number field still sends a valid typed number at once; only the slider sends on release.
- Dimmed tree items in the Settings search use a slightly brighter grey than the mockup, to pass contrast.
- The content's side inset is 32 px (the mockup's 36 px), to keep every screen's column aligned; the details' title
  is 22 px (mockup 24 px); the menu button is 24 px (mockup 28 px).
- The sidebar's mark keeps the mockup's small-size drawing (a thicker band, a bigger star); `logo.svg` stays the
  large-size source.
- The app icon fills its canvas without the usual rounded tile and margin of a macOS app icon.
- The trial tool's command stays `fresher-trial-tool`: a common name could run a stranger's program from `PATH` in
  sandbox mode.
- The demo module's names ("Example caches") were kept; the trial's items are now "Sample app cache" and "Old sample
  downloads".
- A committed value the core refuses or changes stays shown on the slider until the next move (rare: the slider is
  bounded).
- A module page's switch row reads "Look for {module}" with "Off: the scan skips them and nothing of them is
  offered." (interface texts); the tree, the title and the description keep the module's own texts (ruling C-7).
- Review's Safe and Usually safe tile lines use the core's group meaning word for word, not the mockup's shortened
  line (ruling C-9).
- The details' Back button shows "Scan" and is named "Back to Scan" for screen readers.
- No "modules failed" line on the scanning card: a failed or cancelled module makes the scan partial, and the partial
  notice says so.
- Finding rows drop the level badge on Review and the details; the level stays in the row's accessible name (C-6).
- The swoosh shows a sweeping dash when no estimate exists (`null`); today the screen always passes a number.
- The core's uninstall categories carry the `app` icon; a category no module lists falls back to `generic` (D-6).
- The wording review's jargon and clarity findings were fixed in the same round, not left for polish.
- The final review ran before this step, so these lists describe the fixed code; its last leak (Apps rows) was fixed
  here, and all baselines were taken again from scratch here.

Calls with no visible effect (code shape), one line each:

- C-1: the five parts' key check lives beside its fixture in `app/ui/src/test/`.
- C-2: the mockup's `#313741` is the token `--border-solid` (`--border-strong` was taken), with `--border-divider`.
- C-3: the scan phase type and story helpers moved so each step stayed green; the old hero was deleted in 03/02.
- C-4 and D-2: the slider gained `onCommit` on release; native arrow keys are not tested in jsdom.
- C-5: "2 minutes ago" is formatted in the interface with an injected wall clock, no new core text argument.
- C-8: the High refusal is the first check of every prepare, before the agreement.
- D-1: the safety test asserts exactly `NothingToClean` for a High category and `UnknownFinding` for a High id.
- D-3: a module choice drops from "to check" a Look category it no longer holds.
- D-4: the selection's counts are summed in the interface, as before (the selection is interface state).
- D-5: only texts and types no kept code reads were deleted.
- D-7: the existing `IconShape` type was reused.
- D-8: the map's detailed parts stay in the core for its queries; the interface draws only the five.
- The bindings fix was its own commit in 08/02.

## Texts

- The group names and lines, the five part names and their "i" texts, and the category lines (now on every tile).
- The core's group lines start with a capital after the dot ("Safe to remove · Apps make these again on their own");
  the mockup writes them in lower case.
- The wording pass (08/01 and 08/02) rewrote 271 core and module texts and 39 interface texts (counted by those steps;
  the plan's conclusions count 44 interface texts changed since before 08/01, the later fixes included); its tables
  of before and after (the 15 most important each, plus the review round) are in the plan's conclusions of
  [08/01](../plans/042-plain-interface/08-wording/01-core-module-texts.md) and
  [08/02](../plans/042-plain-interface/08-wording/02-interface-texts.md). Detail shortened away: App caches' note on
  music and maps kept for offline use; JetBrains' "restored or partial scan" case folded into one sentence.

## The logo

- The sidebar's mark beside "Fresher" at its small size.
- The app icon in the Dock and in Finder (no rounded tile, see above).
- The social preview (`assets/social-preview.png`): upload it by hand on GitHub, as plan 040 says.

## Known gaps

- Withheld does not refuse a lower-level finding whose path is, or holds, a withheld finding's path; no module makes
  such a pair today ([`deletion-safety.md`](../restrictions/deletion-safety.md)).
- The core still answers the Largest queries while High is withheld; only the pick is refused and the screen hidden.
- The e2e `withheld` helper does not change a script's cleanup or Largest answers.
- The mocks keep answers for the storage meaning query, which no screen asks now.
- The progress estimate on a first scan of a full disk can be far off.
- The demo GIFs show the screens before this plan.

## User decisions after the walk (2026-11-25)

- Menu items have no hints (reverses ruling 16).
- No menu has "Stop looking for <module>…": a module is turned off only in Settings, Ignored, Not looked for (its
  switch on its own page).
