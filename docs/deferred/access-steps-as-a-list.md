# The steps of a permission sent as a list

**Agreed:** 2026-11-19 (plan 006 final review) · **Due at:** the next change to the core's access texts

## What is postponed

The core sends the steps that grant a permission (`core.access.<id>.how`) as one sentence, "1. … 2. … 3. …", and the
coverage notice splits it into a list with a pattern (`stepsOf` in `app/ui/src/screens/permission/coverage-notice.tsx`).
The UI then parses the core's prose, which breaks as soon as a step holds a number followed by a dot, or a language
numbers its steps differently.

## What doing it needs

The core sends the steps as a list of texts (one key per step, or a list in the access status), the notice renders
them as they come and `stepsOf` goes; the texts' tests check every step exists.
