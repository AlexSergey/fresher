# Writing guide

The one voice of every text Fresher shows: the dictionaries of the core, of the modules and of the UI. The rules a
script can hold are marked "(checked)" and run by `npm run check:texts`.

## 1. The voice

- Plain and calm. Second person: "you", "your". Present tense, active voice. "Fresher" for the app.
- No jargon a person who never opened Terminal would not know. A technical name appears only where it is the name of
  the thing, such as a tool's.
- No exclamation marks (checked).
- No fear words (checked: the banned list, section 6).
- Numbers and sizes come from the core, never typed into a sentence; a placeholder stands for them.

## 2. The shapes

- **A category** has five parts: `name` (what it is, a noun phrase), `what` (what the files are), `made-by` (which app
  or part of macOS makes them), `if-removed` (what happens, what you keep, what you lose), `comes-back` (whether and
  when it returns). All five are present (checked; not for the `ui.` texts, which label screens).
- **A choice or a setting**: `label` (what it is or does) and `description` (what it does and what you keep or lose,
  one or two sentences). A description for every label (checked), except in the UI's own `ui.` texts and for a part
  described by its `detail` and `effect`.
- **A left-out or skip reason**: what stopped Fresher, then what you can do.
- **A result**: what happened, what did not, what next.
- **A refusal**: what was refused, that nothing was removed when it is a cleanup, what to do.
- **A hint**: what the control does or what the number counts, in one sentence.
- **The agreement**: its lines change only with a new version and a "what changed" text; see
  [`restrictions/deletion-safety.md`](./restrictions/deletion-safety.md), "The agreement".

## 3. The glossary

One word for one thing.

| Word                | Means                                                                                |
| ------------------- | ------------------------------------------------------------------------------------ |
| remove              | the general act                                                                      |
| move to the Trash   | the default way to remove                                                            |
| delete permanently  | the Trash is skipped; "delete" is used only for this (checked: a text that says "permanent" or sits under a delete key passes)                       |
| empty               | a folder's contents go, the folder stays                                             |
| put back            | return an item from the Trash to where it was                                        |
| left out            | kept out of the list before a cleanup                                                |
| skipped             | kept at the cleanup                                                                  |
| danger level        | Safe, Low, Medium or High                                                            |
| danger zone         | the part of a list holding Medium and High                                           |
| the Trash           | always capitalised (checked)                                                         |
| folder              | never "directory" (checked)                                                          |

## 4. Lengths (checked)

By the key's last segment (a plural `.one` or `.other` counts as the segment before it), in characters, placeholders
counted as written: `name` 48, `label` 48, `title` 64, `made-by` 160, `description` 200, `what` 220, `comes-back` 220,
`if-removed` 240, any other 240.

## 5. Form (checked)

- No double space and no space at either end.
- `description`, `what`, `made-by`, `if-removed`, `comes-back`, and the texts under `left-out.`, `skip.` and `failed.`
  end with ".", "…" or ":" (a text followed by a list or an address).

## 6. The banned words (checked)

Whole words, any case: "junk", "garbage", "simply", "easily", "please", "oops", "sorry", "click here", "at risk",
"you must", "directory". The list is `BANNED` in `scripts/check-texts.ts`; this section mirrors it.

## 7. Examples

A cache category:

- Before: "Junk files your apps left behind!"
- After: "Files an app keeps to start faster. It makes them again when it needs them."

A setting:

- Before: "Simply turn this on to delete things easily."
- After: "Moves what you confirm to the Trash. You can put it back until you empty the Trash."

A skip reason:

- Before: "Sorry, this item could not be removed."
- After: "The app was using this item, so Fresher kept it. Quit the app and clean up again."

## 8. How it is checked

`npm run check:texts` reads every English dictionary. The module contract (`crates/testing/src/contract.rs`) holds
what a module's texts must have. Exemptions sit in the script (`EXEMPT`) with their reason (headings and one-word statuses that are not sentences);
a new one is a reviewed change. A dictionary that cannot be read fails the check.
