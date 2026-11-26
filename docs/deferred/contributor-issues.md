# Issues for contributors

**Agreed:** 2026-11-26, by the user · **Due at:** when 0.1.0 is published

## What is postponed

The first GitHub issues a newcomer can pick up, made when 0.1.0 is published so that people who come from the release
find them. Each is written in English, links the repository's own documents, and names no other product.

**`good first issue`, `new category`: one issue per tool**, each a cache of a developer tool the `dev-caches` module
does not cover yet, built as the module's existing categories are (a category file with its danger level, negative
tests first, a sandbox case file): pnpm's store, Bun's cache, Maven's local repository, Composer's cache, Swift Package
Manager's cache, Dart and Flutter's pub cache, Poetry's cache, uv's cache, NuGet's packages, RubyGems' cache.

**`good first issue`, `design`:** the disk image's window ([`dmg-window-design.md`](./dmg-window-design.md)).

**`help wanted`:**

- the confirmation hides the choice of method ([known issue](../known-issues/ui-confirm-dialog-hides-the-method.md));
- the disk chart keeps its numbers after a cleanup ([known issue](../known-issues/ui-chart-stale-after-a-cleanup.md));
- Finder's own Put Back for what Fresher moved
  ([known issue](../known-issues/cleaner-finder-put-back-unreliable.md));
- the UI e2e tests that fail at times in WebKit on CI: the gallery's part 3
  ([known issue](../known-issues/ui-axe-gallery-part-3-webkit-timeout.md)) and "a screen change never drops the
  focus".

## What doing it needs

1. The labels `new category` and `design` (the others exist).
2. One issue per item above, each with what to build, where the pattern is, and how it is checked.
3. A short "Where to start" in `.github/CONTRIBUTING.md`: the labels, and to say on an issue before working on it.
