# Rules — read before implementing any task

Every plan, design and change in this repository is checked against these rules before it is built. They follow from
[`goal.md`](./goal.md).

## 1. The user's files come first

The product deletes files from a real person's disk. A false positive (something removed that should not have been) is
far worse than a missed gigabyte. When in doubt, a rule does not claim the path, or claims it with a higher danger level.

## 2. Nothing is deleted without explicit confirmation

Scanning is read-only. Deletion runs only from a plan the user confirmed item by item (or category by category), in the
same session, with the size and the highest danger level shown. No background, scheduled or "smart" deletion. Trash by
default; permanent delete only as a separate explicit choice. Details:
[`restrictions/deletion-safety.md`](./restrictions/deletion-safety.md).

## 3. No "Other" — every category is named and explained

Every category has a plain-English name, a description of what it is, which app produces it, what happens when it is
removed, whether it comes back, and a danger level. A category without all of them is not shipped. Unrecognized space is
shown as its real folders, never as a catch-all. The catalogue: [`categories/`](./categories/README.md).

## 4. Fast by design

The scan is parallel, streams results, and never blocks the UI. Scan time and memory on a reference home folder are
measured numbers; a change that can move them measures before and after and writes the result in its plan.

## 5. No hardcoding of one machine

A rule describes a kind of leftover (for example "data folders of an app version older than the installed one"), not one
path on the author's Mac. Paths, version patterns and danger levels live in the rule's data, and every rule is tested on
fixture trees, not on the real home folder.

## 6. Private by default

No network access, no telemetry, no crash upload. The scan results, the scan cache and the settings stay on the Mac. Details:
[`restrictions/privacy.md`](./restrictions/privacy.md).

## 7. Simple interface

Every screen answers one question. A number on screen has a label in plain English; detail is shown on demand, not by
default. The visual direction is Dark Premium ([`goal.md`](./goal.md), section 6).

Every choice offered to the user (a cleanup option, a button, a setting) carries a very clear, short, plain-English
description of what it does and what the user keeps or loses — one or two sentences, no jargon (the user, 2026-11-15).
Example: "Deletes only caches and logs. Your settings stay, in case you install the app again."

## 8. Tested to the bar

Coverage is at least 90 % of lines in the Rust crates and in the UI. Every rule has negative tests (paths it must not
claim) before positive ones. Details: [`testing.md`](./testing.md).

## 9. Third-party code in the app only under permissive licences

MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, Unlicense, CC0-1.0. Anything else is asked about
before it is added. Development tools that never ship in the app may be under any open-source licence. Details:
[`restrictions/dependencies.md`](./restrictions/dependencies.md).
