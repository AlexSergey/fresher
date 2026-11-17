# The scan offers an item inside the Trash, and the cleanup leaves it out

**Status:** open · **Updated:** 2026-11-17

## Symptom

A cache inside `~/.Trash` is found by the scan and offered, with its size, in the findings. When the cleanup runs, the
cleaner leaves it out as protected (the Trash is on the protection list). Seen in sandbox mode with a trial cache in
the sandbox's `home/.Trash`.

## Cause

The scanner walks the scan root and the rules claim what they recognise; nothing tells them that the Trash is not
a place to offer from. The cleaner's protection list refuses the Trash, so the two disagree.

## What was tried

Nothing yet. The case `cleanup-guards` of `fixtures/sandboxes/cleanup-guards.toml` records the behaviour today: the
item is offered and left out as protected.

## Options left

An open product question: should the scan skip the Trash (never offer what the cleanup would refuse)? If so, the
walk's avoid list or the rules' claim would carry it, and the case would turn into a negative.
