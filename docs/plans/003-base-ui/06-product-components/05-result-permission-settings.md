# 003/06/05 — Result, permission and settings rows

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `product/result-summary/` — `ResultSummary`, the cleaned moment: props `freed: Sizes`, `moved: number`,
  `skipped: readonly { title: Text; reason: Text }[]`, `method: 'trash' | 'permanent'`, `onOpenTrash?`, `onDone`.
  - Choreography: the check mark in a Safe-tinted circle draws itself (`stroke-dashoffset`, `--duration-fast`); then
    "Freed" and the freed size count up from 0 (`AnimatedNumber`, `--duration-deliberate`, `--type-display`); behind
    the number one soft green light wave (a radial gradient scaled from 0.6 to 1.4 while fading from 50 % to 0, over
    900 ms, once) and the `AmbientLight` crossfades to its `safe` tone and back. Then, with no motion: "Moved N items
    to the Trash" (or "Deleted N items permanently"), the skipped items each with its reason, **Open Trash** (Trash
    only) and **Done**.
  - Serious, not festive: no confetti, no bounce, no sound; under reduced motion the final values with one
    `--duration-fast` fade.
- `product/permission-card/` — `PermissionCard`: props `access: Access` (`denied` or `unknown`), `onLearnMore?`; a
  `Notice`-like card (warning tone for denied, info for unknown): what the app cannot see without Full Disk Access and
  that it still works with less, in two plain sentences; plan 006 adds the action that opens System Settings.
- `product/settings-row/` — `SettingsRow`: props `descriptor: SettingDescriptor`, `value: SettingValue`, `problem:
  SettingProblem | null`, `readOnly: boolean`, `onChange(value)`. Label and description from the dictionary
  (`<key>.label`, `<key>.description`), the control by kind: `switch` → `Switch`; `number` → `TextField` with the min,
  max and unit, checked as typed; `choice` → `Select` (or `SegmentedControl` for 2–3 options); `folders` → the list
  shown read-only (plan 005 adds editing). A problem shows as the field's error; a read-only file disables the control
  and says why.
- `locales/en.json` — `ui.result.*`, `ui.permission.*`, `ui.settings-row.*`.

## Produces

`ResultSummary` complete for plan 008; `PermissionCard` for the overview (07/02); `SettingsRow` for the settings
screen (07/05) and plan 005.

## Checks

- Negative: `the freed number does not count under reduced motion`, `the light wave plays once`, `Open Trash is absent
  after a permanent delete`, `a skipped item always shows its reason`, `a number outside its limits is not sent`,
  `a read-only settings file disables every control and says why`, `the permission card never blocks the screen` (no
  focus trap, no modal).
- Positive: `the freed size reaches its value`, `Done calls onDone`, `each setting kind renders its control`, `a
  switch change calls onChange with the new value`, `a setting's problem shows as its error`, `the permission card
  differs for denied and unknown`.
- Stories: result all moved, some skipped, permanent; permission denied, unknown; settings row switch, number, choice,
  folders, invalid, read-only; screenshots looked at (the result's end state with motion off).

## Conclusions

- Built as planned: `ResultSummary`, `PermissionCard` and `SettingsRow` in `product/`, each with tests (negative
  first), stories and WebKit screenshots looked at (the result's end state with motion off). One new icon (`Lock`, the
  read-only note); no new dependency and no new token.
- **ResultSummary**: the check is the `Check` icon whose path draws by `stroke-dashoffset` (full motion only). The
  number mounts at 0 and is handed the freed size (`allocated`, with `formatSize`, so a partial size keeps its plus)
  on the next frame, so `AnimatedNumber` counts it up over `--duration-deliberate`. The wave is a radial gradient of
  `--level-safe-glow` that plays once (900 ms, as written) and is removed on `animationend`; while it is on the page
  the `AmbientLight` shows its `safe` tone, then goes back to `iris`. Under reduced motion there is no draw, no count
  and no wave: the whole summary fades in once over `--duration-fast`. The moved line, the skipped items and the
  buttons are always rendered, with no motion (not delayed), so assistive technology reads them at once.
- **PermissionCard** is a `Notice` (warning for denied, info for unknown) with an optional "Learn more" action; it is
  never live, modal or focused.
- **SettingsRow** is a `role="group"` named by the label and described by the description, the error and the
  read-only note. Agent's calls (to check):
  - a choice's option label is `<key>.options.<option>` in the core's dictionary (no convention existed); an option
    the dictionary does not name is shown as written. Two or three options → `SegmentedControl`, more → `Select`;
  - a number is a `TextField` of type `number` whose description is its limits ("From 1 to 365 days"); a typed value
    that is not a whole number in its limits shows "Enter a whole number from 1 to 365." and is not sent; a new value
    from the outside replaces what was typed. A number's problem is the field's error; the other kinds show it in
    words under the control;
  - `readOnly` is a boolean, so the row says one reason for every read-only file ("the settings file cannot be changed
    safely by this version of the app"); the screen (07/05) says which (`SettingsFile`);
  - a value of another kind than the descriptor's shows the descriptor's default.
- The stories set the core dictionary (the demo texts plus their own) in the story's render, so a story shown alone has
  its texts whatever other stories the gallery loads.
