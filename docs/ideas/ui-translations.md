# Translations of the interface

**Since:** 2026-11-19 · **Proposed by:** the agent

## Problem

The interface is English only. The people who need a disk cleaner most are not all developers and not all read
English comfortably; a translation is also the easiest first contribution an outsider can make.

## Direction

- The interface follows the system language, English when a string is missing.
- The core already sends texts by key (`dev-caches.title.<row id>`): a language is one file of those keys; a
  contributor adds one file and nothing else, with a short guide in the docs.
- Russian first (the user's own language), then the languages contributors bring; contributors are credited.
- Plural forms per language (Russian has three); sizes and dates in the system's locale format.

Open questions: whether descriptions and danger texts of categories are translated too (they are long and change with
the rules); how a missing key is caught before a release.
