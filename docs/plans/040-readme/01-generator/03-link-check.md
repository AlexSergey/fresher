# 040/01/03 — The link and image check

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** scripts

`npm run check:readme` also fails on a dead relative link or anchor, an image outside `assets/`, a remote image other
than a shields.io badge, and a claim of something not built. Over `README.md` and every `.github/*.md`. The design:
[`../00-design.md`](../00-design.md), sections 3 and 5.

## Context

- 01/02 made `scripts/readme-catalogue.ts` (pure, `Problem`), `scripts/readme.ts` (`run(mode, root)`, the only file
  system code) and their tests. Same conventions here: pure functions in their own file, negatives first, temporary
  folders, never the real repository in a test.
- GitHub's heading slug: the heading text without its `#`s, lower case, every character that is not a letter, a digit,
  a space, an underscore or a hyphen removed, each space made a hyphen. Example:
  `### 8. Transparent: it does only what the user asks` → `8-transparent-it-does-only-what-the-user-asks`. Inline code
  backticks are removed with the rest.
- No network: absolute links are never fetched.

## Files

- `scripts/readme-links.ts` (new) — pure:

  ```ts
  export type LinkKind = 'image' | 'link';
  export type Reference = { readonly kind: LinkKind; readonly target: string; readonly line: number };
  export type LinkProblemKind =
    | 'dead-anchor' | 'dead-link' | 'image-outside-assets' | 'missing-image' | 'remote-image' | 'unbuilt-claim';

  export const ALLOWED_IMAGE_HOST = 'https://img.shields.io/';
  export const UNBUILT_CLAIMS: readonly { readonly pattern: RegExp; readonly reason: string }[] = [
    { pattern: /watch mode/iu, reason: 'watch mode is not built (plan 010)' },
  ];

  export const slugOf = (heading: string): string;
  export const headingSlugs = (markdown: string): ReadonlySet<string>;
  export const findReferences = (markdown: string): readonly Reference[];
  export const checkReferences = (input: {
    readonly file: string;                         // repository-relative, `README.md` or `.github/SECURITY.md`
    readonly markdown: string;
    readonly exists: (repoPath: string) => boolean; // repository-relative path → exists in the working tree
    readonly slugsOf: (repoPath: string) => ReadonlySet<string> | null; // a Markdown file's heading slugs
  }) => readonly Problem[];
  ```

  - `findReferences` finds `[text](target)` and `<a href="target">` as links, `![alt](src)` and `<img src="src">` as
    images (an image wrapped in a link yields both), with their line numbers; code blocks fenced with three backticks
    are skipped; a target's optional `"title"` is dropped.
  - `checkReferences`:
    - a link starting `http://`, `https://` or `mailto:` is not checked;
    - a link `#anchor` must be in the file's own slugs (`dead-anchor`);
    - a relative link is resolved against the file's folder (`path.posix`), without `?…` and `#…`; it must exist
      (`dead-link`); with a `#anchor` into a `.md` file, the anchor must be in that file's slugs (`dead-anchor`);
    - an image starting with `ALLOWED_IMAGE_HOST` passes; any other `http(s)://` image is `remote-image`; a relative
      image must resolve under `assets/` (`image-outside-assets`) and exist (`missing-image`);
    - each `UNBUILT_CLAIMS` pattern found in the text outside code blocks is `unbuilt-claim`, with its reason;
    - `where` is `<file>:<line>`; `Problem` is the type of `readme-catalogue.ts`, its `kind` widened to
      `ProblemKind | LinkProblemKind` (change the type there).
- `scripts/readme.ts` — `run` also checks `README.md` and every `.github/*.md` that exists (not the issue templates,
  which are YAML), with `exists` and `slugsOf` reading the working tree under `root`; in both modes (a generate that
  finds a link problem still writes the block, and returns the problem).
- `scripts/readme-links.test.ts` (new); `scripts/readme.test.ts` — one more case.

## Tests

`scripts/readme-links.test.ts`:

- Negatives: `[x](docs/none.md)` → `dead-link`; `[x](#nowhere)` → `dead-anchor`; `[x](docs/goal.md#nowhere)` with a
  goal file whose slugs lack it → `dead-anchor`; `![x](docs/pic.png)` → `image-outside-assets`;
  `![x](assets/none.png)` → `missing-image`; `<img src="https://example.com/a.png">` → `remote-image`; a README
  saying "Watch mode tells you…" → `unbuilt-claim`; a `../` link climbing out of the repository → `dead-link`.
- Positives: `slugOf` on the example heading above and on `` ## `npm` caches `` (`npm-caches`); a link inside a fenced
  code block is ignored; `[x](docs/goal.md#8-transparent-it-does-only-what-the-user-asks)` with that slug passes;
  `<img src="assets/logo.png">` that exists passes; a shields.io badge passes; `https://` and `mailto:` links pass
  unchecked; from `.github/SECURITY.md`, `[x](../docs/rules.md)` resolves to `docs/rules.md`.

`scripts/readme.test.ts`: `check` on a temporary tree whose README links a missing file returns `dead-link` beside no
`stale-block`.

## Produces

The complete `check:readme`, for 02/01 to wire into `pre-push` and CI.

## Checks

- `node --test scripts/readme-*.test.ts scripts/readme.test.ts`; `npm run test:scripts`; `npm run lint:ts`;
  `npm run lint:code`; `npm run knip`.
- `node scripts/readme.ts check` on the repository: the today's README has no markers (`markers`) and claims watch
  mode (`unbuilt-claim`) — both expected until 02/01; no other problem (if the current README has a dead link, note it:
  02/01 replaces the text).
- Commit: `feat(repo): check the README's links, images and claims`.

## Conclusions

Done (32d796d5, then d923e671). `scripts/readme-links.ts` checks relative links, anchors, images and the claims
list. Anchors are matched case-insensitively and URL-decoded, repeated headings are numbered like GitHub numbers them,
`_` stays in a slug, `[top](#)` is valid. After the whole-branch review: a link's file must exist with exactly the
written case (a folder listing per segment, so a case-insensitive disk does not hide a link that breaks on GitHub),
and reference-style link definitions are checked too. `scripts/readme.ts` prints `<where>: <kind>: <message>`.
